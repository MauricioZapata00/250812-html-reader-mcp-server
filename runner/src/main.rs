use std::sync::Arc;
use std::io::{self, BufRead, BufReader};
use serde_json::{json, Value};
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;

use domain::model::request::McpRequest;
use application::service::{
    content_fetch_service::ContentFetchService,
    content_parse_service::ContentParseService,
};
use application::use_case::fetch_web_content_use_case::FetchWebContentUseCase;
use infrastructure::{
    client::http_client::HttpClient,
    adapter::html_parser_adapter::HtmlParserAdapter,
    mcp::server::McpServer,
};

type HttpContentFetcher = HttpClient;
type HtmlContentParser = HtmlParserAdapter;
type FetchService = ContentFetchService<HttpContentFetcher>;
type ParseService = ContentParseService<HtmlContentParser>;
type WebContentUseCase = FetchWebContentUseCase<HttpContentFetcher, HtmlContentParser>;
type AppMcpServer = McpServer<HttpContentFetcher, HtmlContentParser>;

struct AppState {
    mcp_server: AppMcpServer,
}

impl AppState {
    fn new() -> Self {
        let http_client = HttpClient::new();
        let http_client_arc = Arc::new(http_client);

        let html_parser = HtmlParserAdapter::new();
        let html_parser_arc = Arc::new(html_parser);

        let fetch_service = ContentFetchService::new(http_client_arc.clone());
        let fetch_service_arc = Arc::new(fetch_service);

        let parse_service = ContentParseService::new(html_parser_arc.clone());
        let parse_service_arc = Arc::new(parse_service);

        let web_content_use_case = FetchWebContentUseCase::new(
            fetch_service_arc,
            parse_service_arc,
        );
        let web_content_use_case_arc = Arc::new(web_content_use_case);

        let mcp_server = McpServer::new(web_content_use_case_arc);

        Self { mcp_server }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr) // Log to stderr to avoid interfering with MCP protocol on stdout
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    info!("Starting HTML MCP Reader server");

    // Initialize application state
    let state = AppState::new();

    info!("MCP server initialized, waiting for requests...");

    // Read JSON-RPC requests from stdin and write responses to stdout
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = line?;
        
        if line.trim().is_empty() {
            continue;
        }

        debug!("Received request: {}", line);

        match parse_request(&line) {
            Ok(request) => {
                let response = state.mcp_server.handle_request(request).await;
                let response_json = serde_json::to_string(&response)?;
                
                println!("{}", response_json);
                
                debug!("Sent response: {}", response_json);
            }
            Err(error) => {
                error!("Failed to parse request: {}", error);
                
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", error)
                    }
                });
                
                println!("{}", serde_json::to_string(&error_response)?);
            }
        }
    }

    info!("MCP server shutting down");
    Ok(())
}

fn parse_request(line: &str) -> Result<McpRequest, String> {
    let value: Value = serde_json::from_str(line)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let id = value.get("id")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("id").and_then(|v| v.as_i64()).map(|i| Box::leak(i.to_string().into_boxed_str()) as &str))
        .unwrap_or("unknown")
        .to_string();

    let method = value.get("method")
        .and_then(|v| v.as_str())
        .ok_or("Missing method field")?
        .to_string();

    let params = value.get("params")
        .cloned()
        .unwrap_or(json!({}));

    Ok(McpRequest { id, method, params })
}
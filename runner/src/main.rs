use std::sync::Arc;
use std::io::{self, BufRead, BufReader, Write};
use serde_json::{json, Value};
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;
use clap::{Parser, Subcommand};
use axum::serve;
use tokio::net::TcpListener;

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
    api::server::ApiServer,
};

type AppMcpServer = McpServer<HttpClient, HtmlParserAdapter>;
type AppApiServer = ApiServer<HttpClient, HtmlParserAdapter>;

#[derive(Parser)]
#[command(name = "html-mcp-reader")]
#[command(about = "HTML content fetching server - supports both MCP and REST API modes")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as MCP server (JSON-RPC over stdin/stdout)
    Mcp,
    /// Run as REST API server (HTTP endpoints)
    Api {
        /// Port to listen on
        #[arg(short, long, default_value = "8085")]
        port: u16,
    },
}

struct AppState {
    mcp_server: AppMcpServer,
    api_server: AppApiServer,
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

        let mcp_server = McpServer::new(web_content_use_case_arc.clone());
        let api_server = ApiServer::new(web_content_use_case_arc);

        Self { mcp_server, api_server }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    // Initialize application state
    let state = AppState::new();

    match cli.command {
        Some(Commands::Mcp) => {
            run_mcp_server(state).await
        }
        Some(Commands::Api { port }) => {
            run_api_server(state, port).await
        }
        None => {
            // Default behavior: check if stdin is available (MCP mode) or run as API
            if atty::is(atty::Stream::Stdin) {
                // Running in terminal, default to API mode
                info!("No command specified and running in terminal. Starting API server on port 8085");
                info!("Use 'cargo run -- mcp' to run as MCP server");
                info!("Use 'cargo run -- api --port <PORT>' to run as API server on specific port");
                run_api_server(state, 8085).await
            } else {
                // Stdin available, assume MCP mode
                info!("Stdin detected, running as MCP server");
                run_mcp_server(state).await
            }
        }
    }
}

async fn run_mcp_server(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTML MCP Reader server");
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
                io::stdout().flush().unwrap();
                
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
                io::stdout().flush().unwrap();
            }
        }
    }

    info!("MCP server shutting down");
    Ok(())
}

async fn run_api_server(state: AppState, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting HTML API Reader server");

    // Create router
    let app = state.api_server.create_router();

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("REST API server listening on {}", addr);
    info!("Health check available at: http://{}/health", addr);
    info!("Fetch endpoint available at: http://{}/api/fetch", addr);

    serve(listener, app).await?;

    info!("API server shutting down");
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
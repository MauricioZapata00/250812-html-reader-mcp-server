use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{info, error, debug};
use domain::model::{
    request::{FetchContentRequest, McpRequest},
    response::{McpResponse, McpError, ToolCapabilities},
};
use application::use_case::fetch_web_content_use_case::FetchWebContentUseCase;
use domain::port::{content_fetcher::ContentFetcher, content_parser::ContentParser};

pub struct McpServer<F, P>
where
    F: ContentFetcher,
    P: ContentParser,
{
    fetch_use_case: Arc<FetchWebContentUseCase<F, P>>,
}

impl<F, P> McpServer<F, P>
where
    F: ContentFetcher,
    P: ContentParser,
{
    pub fn new(fetch_use_case: Arc<FetchWebContentUseCase<F, P>>) -> Self {
        Self { fetch_use_case }
    }

    pub async fn handle_request(&self, request: McpRequest) -> Value {
        debug!("Handling MCP request: {}", request.method);

        match request.method.as_str() {
            "tools/list" => self.handle_tools_list(request.id).await,
            "tools/call" => self.handle_tools_call(request).await,
            "initialize" => self.handle_initialize(request.id).await,
            _ => self.handle_unknown_method(request.id, &request.method).await,
        }
    }

    async fn handle_tools_list(&self, id: String) -> Value {
        info!("Handling tools/list request");

        let tools = vec![ToolCapabilities {
            name: "fetch_web_content".to_string(),
            description: "Fetch and extract content from web pages. Supports HTML parsing and text extraction.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch content from"
                    },
                    "extract_text_only": {
                        "type": "boolean",
                        "description": "Whether to extract only text content (default: true)",
                        "default": true
                    },
                    "follow_redirects": {
                        "type": "boolean", 
                        "description": "Whether to follow HTTP redirects (default: true)",
                        "default": true
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Request timeout in seconds (default: 30, max: 300)",
                        "default": 30,
                        "minimum": 1,
                        "maximum": 300
                    },
                    "user_agent": {
                        "type": "string",
                        "description": "Custom User-Agent header (optional)"
                    }
                },
                "required": ["url"]
            })
        }];

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    async fn handle_tools_call(&self, request: McpRequest) -> Value {
        info!("Handling tools/call request");

        let tool_name = request.params.get("name").and_then(|v| v.as_str());
        let arguments = request.params.get("arguments");

        if tool_name != Some("fetch_web_content") {
            return json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32601,
                    "message": format!("Unknown tool: {:?}", tool_name)
                }
            });
        }

        let Some(args) = arguments else {
            return json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32602,
                    "message": "Missing arguments"
                }
            });
        };

        let fetch_request = match self.parse_fetch_request(args) {
            Ok(req) => req,
            Err(error_msg) => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "error": {
                        "code": -32602,
                        "message": error_msg
                    }
                });
            }
        };

        let response = self.fetch_use_case.execute(fetch_request).await;

        json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "result": response.result,
            "error": response.error
        })
    }

    async fn handle_initialize(&self, id: String) -> Value {
        info!("Handling initialize request");

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "html-mcp-reader",
                    "version": "0.1.0"
                }
            }
        })
    }

    async fn handle_unknown_method(&self, id: String, method: &str) -> Value {
        error!("Unknown method: {}", method);

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        })
    }

    fn parse_fetch_request(&self, args: &Value) -> Result<FetchContentRequest, String> {
        let url = args.get("url")
            .and_then(|v| v.as_str())
            .ok_or("Missing required field: url")?
            .to_string();

        let extract_text_only = args.get("extract_text_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let follow_redirects = args.get("follow_redirects")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let timeout_seconds = args.get("timeout_seconds")
            .and_then(|v| v.as_u64());

        let user_agent = args.get("user_agent")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(FetchContentRequest {
            url,
            extract_text_only,
            follow_redirects,
            timeout_seconds,
            user_agent,
        })
    }
}
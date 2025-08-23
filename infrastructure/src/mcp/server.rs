use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{info, error, debug};
use domain::model::{
    request::{FetchContentRequest, McpRequest},
    response::ToolCapabilities,
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
            extract_text_only: Some(extract_text_only),
            follow_redirects: Some(follow_redirects),
            timeout_seconds,
            user_agent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    use domain::model::content::{ContentMetadata, HtmlContent};
    use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError, ContentFetcherResult};
    use domain::port::content_parser::{ContentParser, ContentParserResult};
    use application::service::{
        content_fetch_service::ContentFetchService,
        content_parse_service::ContentParseService,
    };
    use application::use_case::fetch_web_content_use_case::FetchWebContentUseCase;

    struct MockContentFetcher {
        should_succeed: bool,
        return_error: Option<ContentFetcherError>,
    }

    impl MockContentFetcher {
        fn new_success() -> Self {
            Self {
                should_succeed: true,
                return_error: None,
            }
        }

        fn new_with_error(error: ContentFetcherError) -> Self {
            Self {
                should_succeed: false,
                return_error: Some(error),
            }
        }
    }

    #[async_trait]
    impl ContentFetcher for MockContentFetcher {
        async fn fetch_content(&self, request: FetchContentRequest) -> ContentFetcherResult<HtmlContent> {
            if self.should_succeed {
                let metadata = ContentMetadata {
                    content_type: "text/html".to_string(),
                    status_code: 200,
                    content_length: Some(100),
                    last_modified: None,
                    charset: Some("utf-8".to_string()),
                };

                Ok(HtmlContent {
                    url: request.url,
                    title: Some("Test Title".to_string()),
                    text_content: "Test content".to_string(),
                    raw_html: "<html><body>Test</body></html>".to_string(),
                    metadata,
                })
            } else {
                Err(self.return_error.as_ref().unwrap().clone())
            }
        }
    }

    struct MockContentParser;

    #[async_trait]
    impl ContentParser for MockContentParser {
        async fn parse_html(&self, raw_html: &str, url: &str) -> ContentParserResult<HtmlContent> {
            let metadata = ContentMetadata {
                content_type: "text/html".to_string(),
                status_code: 200,
                content_length: Some(raw_html.len()),
                last_modified: None,
                charset: Some("utf-8".to_string()),
            };

            Ok(HtmlContent {
                url: url.to_string(),
                title: Some("Parsed Title".to_string()),
                text_content: "Parsed content".to_string(),
                raw_html: raw_html.to_string(),
                metadata,
            })
        }

        async fn extract_text(&self, html_content: &HtmlContent) -> ContentParserResult<String> {
            Ok(html_content.text_content.clone())
        }
    }

    fn create_server() -> McpServer<MockContentFetcher, MockContentParser> {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser);
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = Arc::new(FetchWebContentUseCase::new(fetch_service, parse_service));
        
        McpServer::new(use_case)
    }

    fn create_failing_server() -> McpServer<MockContentFetcher, MockContentParser> {
        let error = ContentFetcherError::Network("Connection failed".to_string());
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser);
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = Arc::new(FetchWebContentUseCase::new(fetch_service, parse_service));
        
        McpServer::new(use_case)
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/list".to_string(),
            params: json!({}),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert!(response["result"]["tools"].is_array());
        
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "fetch_web_content");
        assert!(tools[0]["description"].is_string());
        assert!(tools[0]["input_schema"]["properties"]["url"].is_object());
    }

    #[tokio::test]
    async fn test_handle_tools_call_success() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "fetch_web_content",
                "arguments": {
                    "url": "https://example.com",
                    "extract_text_only": true
                }
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert!(response["result"].is_object());
        assert!(response["error"].is_null());
        
        let result = &response["result"];
        assert_eq!(result["success"], true);
        assert_eq!(result["content"]["url"], "https://example.com");
    }

    #[tokio::test]
    async fn test_handle_tools_call_network_error() {
        let server = create_failing_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "fetch_web_content",
                "arguments": {
                    "url": "https://example.com"
                }
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert!(response["result"].is_null());
        assert!(response["error"].is_object());
        
        let error = &response["error"];
        assert_eq!(error["code"], -32001);
        assert!(error["message"].as_str().unwrap().contains("Network error"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_tool() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "unknown_tool",
                "arguments": {}
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert_eq!(response["error"]["code"], -32601);
        assert!(response["error"]["message"].as_str().unwrap().contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_arguments() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "fetch_web_content"
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"].as_str().unwrap().contains("Missing arguments"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_url() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "fetch_web_content",
                "arguments": {
                    "extract_text_only": true
                }
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert_eq!(response["error"]["code"], -32602);
        assert!(response["error"]["message"].as_str().unwrap().contains("Missing required field: url"));
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "initialize".to_string(),
            params: json!({}),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(response["result"]["serverInfo"]["name"], "html-mcp-reader");
        assert_eq!(response["result"]["serverInfo"]["version"], "0.1.0");
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "unknown/method".to_string(),
            params: json!({}),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert_eq!(response["error"]["code"], -32601);
        assert!(response["error"]["message"].as_str().unwrap().contains("Method not found"));
    }

    #[tokio::test]
    async fn test_parse_fetch_request_defaults() {
        let server = create_server();
        let args = json!({
            "url": "https://example.com"
        });

        let result = server.parse_fetch_request(&args);
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.extract_text_only, Some(true));
        assert_eq!(request.follow_redirects, Some(true));
        assert_eq!(request.timeout_seconds, None);
        assert_eq!(request.user_agent, None);
    }

    #[tokio::test]
    async fn test_parse_fetch_request_custom_values() {
        let server = create_server();
        let args = json!({
            "url": "https://example.com",
            "extract_text_only": false,
            "follow_redirects": false,
            "timeout_seconds": 60,
            "user_agent": "Custom Agent"
        });

        let result = server.parse_fetch_request(&args);
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.extract_text_only, Some(false));
        assert_eq!(request.follow_redirects, Some(false));
        assert_eq!(request.timeout_seconds, Some(60));
        assert_eq!(request.user_agent, Some("Custom Agent".to_string()));
    }

    #[tokio::test]
    async fn test_parse_fetch_request_invalid_types() {
        let server = create_server();
        
        // Test invalid boolean
        let args = json!({
            "url": "https://example.com",
            "extract_text_only": "not_a_boolean"
        });

        let result = server.parse_fetch_request(&args);
        assert!(result.is_ok()); // Should use default value

        let request = result.unwrap();
        assert_eq!(request.extract_text_only, Some(true)); // Should use default
    }

    #[tokio::test]
    async fn test_server_creation() {
        let _server = create_server();
    }

    #[tokio::test]
    async fn test_tools_call_with_all_parameters() {
        let server = create_server();
        let request = McpRequest {
            id: "test-id".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "fetch_web_content",
                "arguments": {
                    "url": "https://example.com",
                    "extract_text_only": false,
                    "follow_redirects": false,
                    "timeout_seconds": 45,
                    "user_agent": "Test Agent"
                }
            }),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test-id");
        assert!(response["result"].is_object());
        assert!(response["error"].is_null());
    }
}
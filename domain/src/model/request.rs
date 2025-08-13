use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchContentRequest {
    pub url: String,
    pub extract_text_only: bool,
    pub follow_redirects: bool,
    pub timeout_seconds: Option<u64>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl Default for FetchContentRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            extract_text_only: true,
            follow_redirects: true,
            timeout_seconds: Some(30),
            user_agent: Some("html-mcp-reader/0.1.0".to_string()),
        }
    }
}
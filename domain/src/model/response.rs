use serde::{Deserialize, Serialize};
use super::content::HtmlContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse<T> {
    pub id: String,
    pub result: Option<T>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchContentResponse {
    pub content: HtmlContent,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
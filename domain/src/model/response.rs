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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::content::{HtmlContent, ContentMetadata};
    use serde_json;

    #[test]
    fn test_mcp_response_success() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(100),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let fetch_response = FetchContentResponse {
            content,
            success: true,
            message: Some("Success".to_string()),
        };

        let mcp_response = McpResponse {
            id: "123".to_string(),
            result: Some(fetch_response),
            error: None,
        };

        assert_eq!(mcp_response.id, "123");
        assert!(mcp_response.result.is_some());
        assert!(mcp_response.error.is_none());
        assert!(mcp_response.result.as_ref().unwrap().success);
    }

    #[test]
    fn test_mcp_response_error() {
        let mcp_error = McpError {
            code: -32001,
            message: "Network error".to_string(),
            data: None,
        };

        let mcp_response: McpResponse<FetchContentResponse> = McpResponse {
            id: "456".to_string(),
            result: None,
            error: Some(mcp_error),
        };

        assert_eq!(mcp_response.id, "456");
        assert!(mcp_response.result.is_none());
        assert!(mcp_response.error.is_some());
        assert_eq!(mcp_response.error.as_ref().unwrap().code, -32001);
    }

    #[test]
    fn test_mcp_error_creation() {
        let error = McpError {
            code: -32602,
            message: "Invalid parameters".to_string(),
            data: Some(serde_json::json!({"detail": "URL is required"})),
        };

        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid parameters");
        assert!(error.data.is_some());
        assert_eq!(error.data.as_ref().unwrap()["detail"], "URL is required");
    }

    #[test]
    fn test_fetch_content_response_success() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(100),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let response = FetchContentResponse {
            content,
            success: true,
            message: Some("Successfully fetched".to_string()),
        };

        assert!(response.success);
        assert_eq!(response.content.url, "https://example.com");
        assert_eq!(response.message, Some("Successfully fetched".to_string()));
    }

    #[test]
    fn test_fetch_content_response_failure() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 404,
            content_length: None,
            last_modified: None,
            charset: None,
        };

        let content = HtmlContent {
            url: "https://example.com/404".to_string(),
            title: None,
            text_content: "".to_string(),
            raw_html: "".to_string(),
            metadata,
        };

        let response = FetchContentResponse {
            content,
            success: false,
            message: Some("Not found".to_string()),
        };

        assert!(!response.success);
        assert_eq!(response.content.url, "https://example.com/404");
        assert_eq!(response.message, Some("Not found".to_string()));
    }

    #[test]
    fn test_tool_capabilities() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"}
            },
            "required": ["url"]
        });

        let capabilities = ToolCapabilities {
            name: "fetch_web_content".to_string(),
            description: "Fetch content from a web URL".to_string(),
            input_schema: schema,
        };

        assert_eq!(capabilities.name, "fetch_web_content");
        assert_eq!(capabilities.description, "Fetch content from a web URL");
        assert_eq!(capabilities.input_schema["type"], "object");
    }

    #[test]
    fn test_serialization_deserialization() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(100),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let fetch_response = FetchContentResponse {
            content,
            success: true,
            message: Some("Success".to_string()),
        };

        let mcp_response = McpResponse {
            id: "test-id".to_string(),
            result: Some(fetch_response),
            error: None,
        };

        let serialized = serde_json::to_string(&mcp_response).unwrap();
        let deserialized: McpResponse<FetchContentResponse> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(mcp_response.id, deserialized.id);
        assert_eq!(mcp_response.result.is_some(), deserialized.result.is_some());
        assert_eq!(mcp_response.error.is_none(), deserialized.error.is_none());
    }

    #[test]
    fn test_mcp_response_clone() {
        let error = McpError {
            code: -32001,
            message: "Test error".to_string(),
            data: None,
        };

        let response: McpResponse<FetchContentResponse> = McpResponse {
            id: "clone-test".to_string(),
            result: None,
            error: Some(error),
        };

        let cloned = response.clone();
        assert_eq!(response.id, cloned.id);
        assert_eq!(response.result.is_none(), cloned.result.is_none());
        assert_eq!(response.error.as_ref().unwrap().code, cloned.error.as_ref().unwrap().code);
    }

    #[test]
    fn test_empty_message_response() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(0),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: None,
            text_content: "".to_string(),
            raw_html: "".to_string(),
            metadata,
        };

        let response = FetchContentResponse {
            content,
            success: true,
            message: None,
        };

        assert!(response.success);
        assert!(response.message.is_none());
        assert_eq!(response.content.text_content, "");
    }

    #[test]
    fn test_error_codes() {
        let errors = vec![
            (-32001, "Network error"),
            (-32002, "Timeout error"),
            (-32003, "HTTP error"),
            (-32004, "Parse error"),
            (-32602, "Invalid parameters"),
        ];

        for (code, message) in errors {
            let error = McpError {
                code,
                message: message.to_string(),
                data: None,
            };
            assert_eq!(error.code, code);
            assert_eq!(error.message, message);
        }
    }
}
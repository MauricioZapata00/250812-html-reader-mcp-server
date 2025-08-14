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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_fetch_content_request_default() {
        let request = FetchContentRequest::default();
        
        assert_eq!(request.url, "");
        assert_eq!(request.extract_text_only, true);
        assert_eq!(request.follow_redirects, true);
        assert_eq!(request.timeout_seconds, Some(30));
        assert_eq!(request.user_agent, Some("html-mcp-reader/0.1.0".to_string()));
    }

    #[test]
    fn test_fetch_content_request_custom() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: false,
            follow_redirects: false,
            timeout_seconds: Some(60),
            user_agent: Some("custom-agent/1.0".to_string()),
        };

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.extract_text_only, false);
        assert_eq!(request.follow_redirects, false);
        assert_eq!(request.timeout_seconds, Some(60));
        assert_eq!(request.user_agent, Some("custom-agent/1.0".to_string()));
    }

    #[test]
    fn test_fetch_content_request_edge_cases() {
        let request = FetchContentRequest {
            url: "".to_string(),
            extract_text_only: true,
            follow_redirects: true,
            timeout_seconds: None,
            user_agent: None,
        };

        assert_eq!(request.url, "");
        assert_eq!(request.timeout_seconds, None);
        assert_eq!(request.user_agent, None);
    }

    #[test]
    fn test_fetch_content_request_zero_timeout() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: true,
            follow_redirects: true,
            timeout_seconds: Some(0),
            user_agent: Some("test".to_string()),
        };

        assert_eq!(request.timeout_seconds, Some(0));
    }

    #[test]
    fn test_fetch_content_request_serialization() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: false,
            follow_redirects: true,
            timeout_seconds: Some(45),
            user_agent: Some("test-agent".to_string()),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: FetchContentRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.url, deserialized.url);
        assert_eq!(request.extract_text_only, deserialized.extract_text_only);
        assert_eq!(request.follow_redirects, deserialized.follow_redirects);
        assert_eq!(request.timeout_seconds, deserialized.timeout_seconds);
        assert_eq!(request.user_agent, deserialized.user_agent);
    }

    #[test]
    fn test_mcp_request_creation() {
        let params = serde_json::json!({
            "url": "https://example.com",
            "extract_text_only": true
        });

        let request = McpRequest {
            id: "123".to_string(),
            method: "tools/call".to_string(),
            params,
        };

        assert_eq!(request.id, "123");
        assert_eq!(request.method, "tools/call");
        assert_eq!(request.params["url"], "https://example.com");
        assert_eq!(request.params["extract_text_only"], true);
    }

    #[test]
    fn test_mcp_request_empty_params() {
        let request = McpRequest {
            id: "456".to_string(),
            method: "initialize".to_string(),
            params: serde_json::Value::Null,
        };

        assert_eq!(request.id, "456");
        assert_eq!(request.method, "initialize");
        assert_eq!(request.params, serde_json::Value::Null);
    }

    #[test]
    fn test_mcp_request_serialization() {
        let params = serde_json::json!({
            "test": "value",
            "number": 42
        });

        let request = McpRequest {
            id: "test-id".to_string(),
            method: "test-method".to_string(),
            params,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.params, deserialized.params);
    }

    #[test]
    fn test_fetch_content_request_clone() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: true,
            follow_redirects: false,
            timeout_seconds: Some(120),
            user_agent: Some("clone-test".to_string()),
        };

        let cloned = request.clone();
        assert_eq!(request.url, cloned.url);
        assert_eq!(request.extract_text_only, cloned.extract_text_only);
        assert_eq!(request.follow_redirects, cloned.follow_redirects);
        assert_eq!(request.timeout_seconds, cloned.timeout_seconds);
        assert_eq!(request.user_agent, cloned.user_agent);
    }

    #[test]
    fn test_fetch_content_request_long_url() {
        let long_url = format!("https://example.com/{}", "a".repeat(1000));
        let request = FetchContentRequest {
            url: long_url.clone(),
            extract_text_only: true,
            follow_redirects: true,
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        assert_eq!(request.url, long_url);
        assert_eq!(request.url.len(), 1020); // "https://example.com/" + 1000 'a's
    }

    #[test]
    fn test_fetch_content_request_extreme_timeout() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: true,
            follow_redirects: true,
            timeout_seconds: Some(u64::MAX),
            user_agent: Some("test".to_string()),
        };

        assert_eq!(request.timeout_seconds, Some(u64::MAX));
    }
}
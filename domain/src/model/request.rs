use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchContentRequest {
    pub url: String,
    pub extract_text_only: Option<bool>,
    pub follow_redirects: Option<bool>,
    pub timeout_seconds: Option<u64>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
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
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("html-api-reader/0.1.0".to_string()),
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
        assert_eq!(request.extract_text_only, Some(true));
        assert_eq!(request.follow_redirects, Some(true));
        assert_eq!(request.timeout_seconds, Some(30));
        assert_eq!(request.user_agent, Some("html-api-reader/0.1.0".to_string()));
    }

    #[test]
    fn test_fetch_content_request_custom() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(false),
            follow_redirects: Some(false),
            timeout_seconds: Some(60),
            user_agent: Some("custom-agent/1.0".to_string()),
        };

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.extract_text_only, Some(false));
        assert_eq!(request.follow_redirects, Some(false));
        assert_eq!(request.timeout_seconds, Some(60));
        assert_eq!(request.user_agent, Some("custom-agent/1.0".to_string()));
    }

    #[test]
    fn test_fetch_content_request_edge_cases() {
        let request = FetchContentRequest {
            url: "".to_string(),
            extract_text_only: None,
            follow_redirects: None,
            timeout_seconds: None,
            user_agent: None,
        };

        assert_eq!(request.url, "");
        assert_eq!(request.extract_text_only, None);
        assert_eq!(request.follow_redirects, None);
        assert_eq!(request.timeout_seconds, None);
        assert_eq!(request.user_agent, None);
    }

    #[test]
    fn test_fetch_content_request_serialization() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(false),
            follow_redirects: Some(true),
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
    fn test_api_error_response() {
        let error = ApiErrorResponse {
            error: "INVALID_URL".to_string(),
            message: "The provided URL is not valid".to_string(),
        };

        assert_eq!(error.error, "INVALID_URL");
        assert_eq!(error.message, "The provided URL is not valid");
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0");
    }

    #[test]
    fn test_fetch_content_request_minimal() {
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: None,
            follow_redirects: None,
            timeout_seconds: None,
            user_agent: None,
        };

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.extract_text_only, None);
        assert_eq!(request.follow_redirects, None);
        assert_eq!(request.timeout_seconds, None);
        assert_eq!(request.user_agent, None);
    }

    #[test]
    fn test_responses_serialization() {
        let error = ApiErrorResponse {
            error: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
        };

        let health = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
        };

        let error_json = serde_json::to_string(&error).unwrap();
        let health_json = serde_json::to_string(&health).unwrap();

        let error_deserialized: ApiErrorResponse = serde_json::from_str(&error_json).unwrap();
        let health_deserialized: HealthResponse = serde_json::from_str(&health_json).unwrap();

        assert_eq!(error.error, error_deserialized.error);
        assert_eq!(error.message, error_deserialized.message);
        assert_eq!(health.status, health_deserialized.status);
        assert_eq!(health.version, health_deserialized.version);
    }
}
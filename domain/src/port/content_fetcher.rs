use async_trait::async_trait;
use crate::model::{content::HtmlContent, request::FetchContentRequest};

pub type ContentFetcherResult<T> = Result<T, ContentFetcherError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ContentFetcherError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Timeout: Request timed out after {0} seconds")]
    Timeout(u64),
    #[error("HTTP error: {status} - {message}")]
    Http { status: u16, message: String },
    #[error("Parse error: {0}")]
    Parse(String),
}

#[async_trait]
pub trait ContentFetcher: Send + Sync {
    async fn fetch_content(&self, request: FetchContentRequest) -> ContentFetcherResult<HtmlContent>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_fetcher_error_network() {
        let error = ContentFetcherError::Network("Connection refused".to_string());
        assert_eq!(error.to_string(), "Network error: Connection refused");
    }

    #[test]
    fn test_content_fetcher_error_invalid_url() {
        let error = ContentFetcherError::InvalidUrl("not-a-url".to_string());
        assert_eq!(error.to_string(), "Invalid URL: not-a-url");
    }

    #[test]
    fn test_content_fetcher_error_timeout() {
        let error = ContentFetcherError::Timeout(30);
        assert_eq!(error.to_string(), "Timeout: Request timed out after 30 seconds");
    }

    #[test]
    fn test_content_fetcher_error_http() {
        let error = ContentFetcherError::Http {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert_eq!(error.to_string(), "HTTP error: 404 - Not Found");
    }

    #[test]
    fn test_content_fetcher_error_parse() {
        let error = ContentFetcherError::Parse("Invalid JSON".to_string());
        assert_eq!(error.to_string(), "Parse error: Invalid JSON");
    }

    #[test]
    fn test_content_fetcher_error_debug() {
        let error = ContentFetcherError::Network("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Network"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_content_fetcher_result_ok() {
        use crate::model::content::{HtmlContent, ContentMetadata};
        
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(100),
            last_modified: None,
            charset: Some("utf-8".to_string()),
            javascript_detected: None,
            fetch_method: None,
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let result: ContentFetcherResult<HtmlContent> = Ok(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_content_fetcher_result_err() {
        let error = ContentFetcherError::Network("Connection failed".to_string());
        let result: ContentFetcherResult<HtmlContent> = Err(error);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.to_string(), "Network error: Connection failed");
        }
    }

    #[test]
    fn test_content_fetcher_error_zero_timeout() {
        let error = ContentFetcherError::Timeout(0);
        assert_eq!(error.to_string(), "Timeout: Request timed out after 0 seconds");
    }

    #[test]
    fn test_content_fetcher_error_extreme_status() {
        let error = ContentFetcherError::Http {
            status: u16::MAX,
            message: "Unknown status".to_string(),
        };
        assert_eq!(error.to_string(), format!("HTTP error: {} - Unknown status", u16::MAX));
    }

    #[test]
    fn test_content_fetcher_error_empty_strings() {
        let network_error = ContentFetcherError::Network("".to_string());
        assert_eq!(network_error.to_string(), "Network error: ");

        let url_error = ContentFetcherError::InvalidUrl("".to_string());
        assert_eq!(url_error.to_string(), "Invalid URL: ");

        let parse_error = ContentFetcherError::Parse("".to_string());
        assert_eq!(parse_error.to_string(), "Parse error: ");

        let http_error = ContentFetcherError::Http {
            status: 500,
            message: "".to_string(),
        };
        assert_eq!(http_error.to_string(), "HTTP error: 500 - ");
    }
}
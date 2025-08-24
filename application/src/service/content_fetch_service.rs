use std::sync::Arc;
use tracing::info;
use domain::model::{content::HtmlContent, request::FetchContentRequest};
use domain::port::content_fetcher::{ContentFetcher, ContentFetcherResult};

pub struct ContentFetchService<F>
where
    F: ContentFetcher,
{
    content_fetcher: Arc<F>,
}

impl<F> ContentFetchService<F>
where
    F: ContentFetcher,
{
    pub fn new(content_fetcher: Arc<F>) -> Self {
        Self { content_fetcher }
    }

    pub async fn fetch_and_process_content(
        &self,
        request: FetchContentRequest,
    ) -> ContentFetcherResult<HtmlContent> {
        info!("Fetching content from URL: {}", request.url);
        
        let content = self.content_fetcher.fetch_content(request).await?;
        
        info!("Successfully fetched content from URL: {}", content.url);
        Ok(content)
    }

    pub async fn validate_request(&self, request: &FetchContentRequest) -> Result<(), String> {
        if request.url.is_empty() {
            return Err("URL cannot be empty".to_string());
        }

        if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        if let Some(timeout) = request.timeout_seconds {
            if timeout > 300 {
                return Err("Timeout cannot exceed 300 seconds".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    use domain::model::content::{ContentMetadata, HtmlContent};
    use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError, ContentFetcherResult};

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
            javascript_detected: None,
            fetch_method: None,
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


    #[tokio::test]
    async fn test_fetch_and_process_content_success() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.fetch_and_process_content(request).await;
        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.title, Some("Test Title".to_string()));
        assert_eq!(content.text_content, "Test content");
    }

    #[tokio::test]
    async fn test_fetch_and_process_content_network_error() {
        let error = ContentFetcherError::Network("Connection refused".to_string());
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.fetch_and_process_content(request).await;
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err.to_string(), "Network error: Connection refused");
        }
    }

    #[tokio::test]
    async fn test_fetch_and_process_content_timeout_error() {
        let error = ContentFetcherError::Timeout(30);
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.fetch_and_process_content(request).await;
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err.to_string(), "Timeout: Request timed out after 30 seconds");
        }
    }

    #[tokio::test]
    async fn test_fetch_and_process_content_http_error() {
        let error = ContentFetcherError::Http {
            status: 404,
            message: "Not Found".to_string(),
        };
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com/404".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.fetch_and_process_content(request).await;
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(err.to_string(), "HTTP error: 404 - Not Found");
        }
    }

    #[tokio::test]
    async fn test_validate_request_success() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_request_empty_url() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL cannot be empty");
    }

    #[tokio::test]
    async fn test_validate_request_invalid_protocol() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "ftp://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "URL must start with http:// or https://");
    }

    #[tokio::test]
    async fn test_validate_request_http_protocol() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "http://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_request_timeout_too_high() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(400),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Timeout cannot exceed 300 seconds");
    }

    #[tokio::test]
    async fn test_validate_request_timeout_at_limit() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(300),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_request_no_timeout() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: None,
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_request_zero_timeout() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let service = ContentFetchService::new(fetcher);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(0),
            user_agent: Some("test".to_string()),
        };

        let result = service.validate_request(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_creation() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let _service = ContentFetchService::new(fetcher);
    }
}
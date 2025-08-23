use std::sync::Arc;
use tracing::{info, error};
use domain::model::{
    request::FetchContentRequest,
    response::{FetchContentResponse, McpResponse, McpError},
    content::HtmlContent,
};
use domain::port::{
    content_fetcher::{ContentFetcher, ContentFetcherError},
    content_parser::ContentParser,
};
use crate::service::{
    content_fetch_service::ContentFetchService,
    content_parse_service::ContentParseService,
};

pub struct FetchWebContentUseCase<F, P>
where
    F: ContentFetcher,
    P: ContentParser,
{
    fetch_service: Arc<ContentFetchService<F>>,
    _parse_service: Arc<ContentParseService<P>>, // Keep for potential future use
}

impl<F, P> FetchWebContentUseCase<F, P>
where
    F: ContentFetcher,
    P: ContentParser,
{
    pub fn new(
        fetch_service: Arc<ContentFetchService<F>>,
        parse_service: Arc<ContentParseService<P>>,
    ) -> Self {
        Self {
            fetch_service,
            _parse_service: parse_service,
        }
    }

    pub async fn execute_for_api(&self, request: FetchContentRequest) -> Result<HtmlContent, String> {
        // Convert optional fields to required ones with defaults
        let processed_request = FetchContentRequest {
            url: request.url.clone(),
            extract_text_only: request.extract_text_only.or(Some(true)),
            follow_redirects: request.follow_redirects.or(Some(true)),
            timeout_seconds: request.timeout_seconds.or(Some(30)),
            user_agent: request.user_agent.or(Some("html-api-reader/0.1.0".to_string())),
        };

        if let Err(validation_error) = self.fetch_service.validate_request(&processed_request).await {
            return Err(format!("Invalid parameters: {}", validation_error));
        }

        match self.fetch_service.fetch_and_process_content(processed_request).await {
            Ok(content) => {
                info!("Successfully fetched content from: {}", content.url);
                Ok(content)
            }
            Err(error) => {
                error!("Failed to fetch content: {:?}", error);
                let message = match error {
                    ContentFetcherError::Network(msg) => format!("Network error: {}", msg),
                    ContentFetcherError::InvalidUrl(msg) => format!("Invalid URL: {}", msg),
                    ContentFetcherError::Timeout(seconds) => format!("Request timeout after {} seconds", seconds),
                    ContentFetcherError::Http { status, message } => format!("HTTP {}: {}", status, message),
                    ContentFetcherError::Parse(msg) => format!("Parse error: {}", msg),
                };
                Err(message)
            }
        }
    }

    pub async fn execute(&self, request: FetchContentRequest) -> McpResponse<FetchContentResponse> {
        let request_id = uuid::Uuid::new_v4().to_string();

        if let Err(validation_error) = self.fetch_service.validate_request(&request).await {
            return McpResponse {
                id: request_id,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: format!("Invalid parameters: {}", validation_error),
                    data: None,
                }),
            };
        }

        match self.fetch_service.fetch_and_process_content(request).await {
            Ok(content) => {
                info!("Successfully fetched content from: {}", content.url);
                McpResponse {
                    id: request_id,
                    result: Some(FetchContentResponse {
                        content,
                        success: true,
                        message: Some("Content fetched successfully".to_string()),
                    }),
                    error: None,
                }
            }
            Err(error) => {
                error!("Failed to fetch content: {:?}", error);
                let (code, message) = match error {
                    ContentFetcherError::Network(msg) => (-32001, format!("Network error: {}", msg)),
                    ContentFetcherError::InvalidUrl(msg) => (-32602, format!("Invalid URL: {}", msg)),
                    ContentFetcherError::Timeout(seconds) => (-32002, format!("Request timeout after {} seconds", seconds)),
                    ContentFetcherError::Http { status, message } => (-32003, format!("HTTP {}: {}", status, message)),
                    ContentFetcherError::Parse(msg) => (-32004, format!("Parse error: {}", msg)),
                };

                McpResponse {
                    id: request_id,
                    result: None,
                    error: Some(McpError {
                        code,
                        message,
                        data: None,
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    use domain::model::content::{ContentMetadata, HtmlContent};
    use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError, ContentFetcherResult};
    use domain::port::content_parser::{ContentParser, ContentParserError, ContentParserResult};
    use crate::service::{
        content_fetch_service::ContentFetchService,
        content_parse_service::ContentParseService,
    };

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

    struct MockContentParser {
        should_succeed: bool,
    }

    impl MockContentParser {
        fn new_success() -> Self {
            Self { should_succeed: true }
        }
    }

    #[async_trait]
    impl ContentParser for MockContentParser {
        async fn parse_html(&self, raw_html: &str, url: &str) -> ContentParserResult<HtmlContent> {
            if self.should_succeed {
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
            } else {
                Err(ContentParserError::Parse("Parse failed".to_string()))
            }
        }

        async fn extract_text(&self, html_content: &HtmlContent) -> ContentParserResult<String> {
            if self.should_succeed {
                Ok(html_content.text_content.clone())
            } else {
                Err(ContentParserError::Parse("Text extraction failed".to_string()))
            }
        }
    }


    #[tokio::test]
    async fn test_execute_success() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        assert!(result.success);
        assert_eq!(result.content.url, "https://example.com");
        assert_eq!(result.message, Some("Content fetched successfully".to_string()));
    }

    #[tokio::test]
    async fn test_execute_validation_error() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "".to_string(), // Invalid empty URL
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Invalid parameters"));
        assert!(error.message.contains("URL cannot be empty"));
    }

    #[tokio::test]
    async fn test_execute_network_error() {
        let error = ContentFetcherError::Network("Connection refused".to_string());
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32001);
        assert!(error.message.contains("Network error"));
    }

    #[tokio::test]
    async fn test_execute_timeout_error() {
        let error = ContentFetcherError::Timeout(30);
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32002);
        assert!(error.message.contains("Request timeout after 30 seconds"));
    }

    #[tokio::test]
    async fn test_execute_http_error() {
        let error = ContentFetcherError::Http {
            status: 404,
            message: "Not Found".to_string(),
        };
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com/404".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32003);
        assert!(error.message.contains("HTTP 404: Not Found"));
    }

    #[tokio::test]
    async fn test_execute_invalid_url_error() {
        let error = ContentFetcherError::InvalidUrl("not-a-url".to_string());
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Invalid URL"));
    }

    #[tokio::test]
    async fn test_execute_parse_error() {
        let error = ContentFetcherError::Parse("Parse failed".to_string());
        let fetcher = Arc::new(MockContentFetcher::new_with_error(error));
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32004);
        assert!(error.message.contains("Parse error"));
    }

    #[tokio::test]
    async fn test_execute_invalid_protocol() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "ftp://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("URL must start with http:// or https://"));
    }

    #[tokio::test]
    async fn test_execute_timeout_too_high() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let use_case = FetchWebContentUseCase::new(fetch_service, parse_service);

        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(400), // Too high
            user_agent: Some("test".to_string()),
        };

        let response = use_case.execute(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Timeout cannot exceed 300 seconds"));
    }

    #[tokio::test]
    async fn test_use_case_creation() {
        let fetcher = Arc::new(MockContentFetcher::new_success());
        let parser = Arc::new(MockContentParser::new_success());
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        
        let _use_case = FetchWebContentUseCase::new(fetch_service, parse_service);
    }
}
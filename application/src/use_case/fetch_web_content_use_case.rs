use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};
use domain::model::{
    content::HtmlContent,
    request::FetchContentRequest,
    response::{FetchContentResponse, McpResponse, McpError},
};
use domain::port::{
    content_fetcher::{ContentFetcher, ContentFetcherError},
    content_parser::{ContentParser, ContentParserError},
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
    parse_service: Arc<ContentParseService<P>>,
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
            parse_service,
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
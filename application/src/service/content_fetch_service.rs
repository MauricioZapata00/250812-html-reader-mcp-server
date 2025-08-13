use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};
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
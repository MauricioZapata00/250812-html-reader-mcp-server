use async_trait::async_trait;
use crate::model::{content::HtmlContent, request::FetchContentRequest};

pub type ContentFetcherResult<T> = Result<T, ContentFetcherError>;

#[derive(Debug, thiserror::Error)]
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
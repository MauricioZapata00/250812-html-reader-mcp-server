use async_trait::async_trait;
use crate::model::content::HtmlContent;

pub type ContentParserResult<T> = Result<T, ContentParserError>;

#[derive(Debug, thiserror::Error)]
pub enum ContentParserError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

#[async_trait]
pub trait ContentParser: Send + Sync {
    async fn parse_html(&self, raw_html: &str, url: &str) -> ContentParserResult<HtmlContent>;
    async fn extract_text(&self, html_content: &HtmlContent) -> ContentParserResult<String>;
}
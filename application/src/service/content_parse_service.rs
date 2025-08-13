use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};
use domain::model::content::HtmlContent;
use domain::port::content_parser::{ContentParser, ContentParserResult};

pub struct ContentParseService<P>
where
    P: ContentParser,
{
    content_parser: Arc<P>,
}

impl<P> ContentParseService<P>
where
    P: ContentParser,
{
    pub fn new(content_parser: Arc<P>) -> Self {
        Self { content_parser }
    }

    pub async fn parse_html_content(
        &self,
        raw_html: &str,
        url: &str,
    ) -> ContentParserResult<HtmlContent> {
        info!("Parsing HTML content for URL: {}", url);
        
        let content = self.content_parser.parse_html(raw_html, url).await?;
        
        info!("Successfully parsed HTML content for URL: {}", url);
        Ok(content)
    }

    pub async fn extract_text_only(
        &self,
        html_content: &HtmlContent,
    ) -> ContentParserResult<String> {
        info!("Extracting text from HTML content for URL: {}", html_content.url);
        
        let text = self.content_parser.extract_text(html_content).await?;
        
        info!("Successfully extracted text content");
        Ok(text)
    }
}
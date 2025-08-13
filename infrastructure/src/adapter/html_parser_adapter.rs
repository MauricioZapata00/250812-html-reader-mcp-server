use async_trait::async_trait;
use scraper::{Html, Selector};
use tracing::{info, debug};
use domain::model::content::{HtmlContent, ContentMetadata};
use domain::port::content_parser::{ContentParser, ContentParserResult, ContentParserError};

pub struct HtmlParserAdapter;

impl HtmlParserAdapter {
    pub fn new() -> Self {
        Self
    }

    fn extract_title_from_raw_html(&self, raw_html: &str) -> Option<String> {
        let document = Html::parse_document(raw_html);
        let title_selector = Selector::parse("title").ok()?;
        document
            .select(&title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|title| !title.is_empty())
    }

    fn extract_meta_description(&self, raw_html: &str) -> Option<String> {
        let document = Html::parse_document(raw_html);
        let meta_selector = Selector::parse("meta[name='description']").ok()?;
        document
            .select(&meta_selector)
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(|content| content.to_string())
    }

    fn clean_text_content(&self, text: String) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl ContentParser for HtmlParserAdapter {
    async fn parse_html(&self, raw_html: &str, url: &str) -> ContentParserResult<HtmlContent> {
        debug!("Parsing HTML content for URL: {}", url);

        let title = self.extract_title_from_raw_html(raw_html);
        let text_content = self.extract_text_from_html(raw_html)?;

        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200, // This should come from the HTTP response
            content_length: Some(raw_html.len()),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        info!("Successfully parsed HTML content with {} characters", text_content.len());

        Ok(HtmlContent {
            url: url.to_string(),
            title,
            text_content,
            raw_html: raw_html.to_string(),
            metadata,
        })
    }

    async fn extract_text(&self, html_content: &HtmlContent) -> ContentParserResult<String> {
        self.extract_text_from_html(&html_content.raw_html)
    }
}

impl HtmlParserAdapter {
    fn extract_text_from_html(&self, raw_html: &str) -> ContentParserResult<String> {
        let document = Html::parse_document(raw_html);
        
        // Use a simple approach: select all text content and filter out script/style
        let body_selector = Selector::parse("body").unwrap();
        
        let text_content = if let Some(body) = document.select(&body_selector).next() {
            // Get text from body, which automatically excludes script/style content
            body.text().collect::<Vec<_>>().join(" ")
        } else {
            // Fallback: get all text from document
            document.root_element().text().collect::<Vec<_>>().join(" ")
        };

        let cleaned_text = self.clean_text_content(text_content);
        Ok(cleaned_text)
    }
}
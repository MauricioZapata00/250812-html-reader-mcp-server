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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_parser_error_parse() {
        let error = ContentParserError::Parse("Failed to parse HTML".to_string());
        assert_eq!(error.to_string(), "Parse error: Failed to parse HTML");
    }

    #[test]
    fn test_content_parser_error_invalid_html() {
        let error = ContentParserError::InvalidHtml("Malformed HTML tag".to_string());
        assert_eq!(error.to_string(), "Invalid HTML: Malformed HTML tag");
    }

    #[test]
    fn test_content_parser_error_encoding() {
        let error = ContentParserError::Encoding("Invalid UTF-8 sequence".to_string());
        assert_eq!(error.to_string(), "Encoding error: Invalid UTF-8 sequence");
    }

    #[test]
    fn test_content_parser_error_debug() {
        let error = ContentParserError::Parse("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Parse"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_content_parser_result_ok() {
        use crate::model::content::{HtmlContent, ContentMetadata};
        
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(100),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let result: ContentParserResult<HtmlContent> = Ok(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_content_parser_result_err() {
        let error = ContentParserError::Parse("Failed to parse".to_string());
        let result: ContentParserResult<String> = Err(error);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.to_string(), "Parse error: Failed to parse");
        }
    }

    #[test]
    fn test_content_parser_error_empty_strings() {
        let parse_error = ContentParserError::Parse("".to_string());
        assert_eq!(parse_error.to_string(), "Parse error: ");

        let html_error = ContentParserError::InvalidHtml("".to_string());
        assert_eq!(html_error.to_string(), "Invalid HTML: ");

        let encoding_error = ContentParserError::Encoding("".to_string());
        assert_eq!(encoding_error.to_string(), "Encoding error: ");
    }

    #[test]
    fn test_content_parser_error_long_messages() {
        let long_message = "a".repeat(1000);
        
        let parse_error = ContentParserError::Parse(long_message.clone());
        assert_eq!(parse_error.to_string(), format!("Parse error: {}", long_message));

        let html_error = ContentParserError::InvalidHtml(long_message.clone());
        assert_eq!(html_error.to_string(), format!("Invalid HTML: {}", long_message));

        let encoding_error = ContentParserError::Encoding(long_message.clone());
        assert_eq!(encoding_error.to_string(), format!("Encoding error: {}", long_message));
    }
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlContent {
    pub url: String,
    pub title: Option<String>,
    pub text_content: String,
    pub raw_html: String,
    pub metadata: ContentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub content_type: String,
    pub status_code: u16,
    pub content_length: Option<usize>,
    pub last_modified: Option<String>,
    pub charset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Html,
    PlainText,
    Json,
    Xml,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_html_content_creation() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(1024),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.title, Some("Test Title".to_string()));
        assert_eq!(content.text_content, "Test content");
        assert_eq!(content.raw_html, "<html><body>Test</body></html>");
        assert_eq!(content.metadata.status_code, 200);
    }

    #[test]
    fn test_html_content_with_none_title() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 404,
            content_length: None,
            last_modified: None,
            charset: None,
        };

        let content = HtmlContent {
            url: "https://example.com/404".to_string(),
            title: None,
            text_content: "Not found".to_string(),
            raw_html: "<html><body>404</body></html>".to_string(),
            metadata,
        };

        assert_eq!(content.title, None);
        assert_eq!(content.metadata.content_length, None);
        assert_eq!(content.metadata.last_modified, None);
        assert_eq!(content.metadata.charset, None);
    }

    #[test]
    fn test_content_metadata_edge_cases() {
        let metadata = ContentMetadata {
            content_type: "".to_string(),
            status_code: 0,
            content_length: Some(0),
            last_modified: Some("".to_string()),
            charset: Some("".to_string()),
        };

        assert_eq!(metadata.content_type, "");
        assert_eq!(metadata.status_code, 0);
        assert_eq!(metadata.content_length, Some(0));
        assert_eq!(metadata.last_modified, Some("".to_string()));
        assert_eq!(metadata.charset, Some("".to_string()));
    }

    #[test]
    fn test_content_type_default() {
        let default_type = ContentType::default();
        assert!(matches!(default_type, ContentType::Html));
    }

    #[test]
    fn test_content_type_variants() {
        let html = ContentType::Html;
        let text = ContentType::PlainText;
        let json = ContentType::Json;
        let xml = ContentType::Xml;

        assert!(matches!(html, ContentType::Html));
        assert!(matches!(text, ContentType::PlainText));
        assert!(matches!(json, ContentType::Json));
        assert!(matches!(xml, ContentType::Xml));
    }

    #[test]
    fn test_html_content_serialization() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(1024),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: HtmlContent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(content.url, deserialized.url);
        assert_eq!(content.title, deserialized.title);
        assert_eq!(content.text_content, deserialized.text_content);
        assert_eq!(content.raw_html, deserialized.raw_html);
        assert_eq!(content.metadata.status_code, deserialized.metadata.status_code);
    }

    #[test]
    fn test_content_type_serialization() {
        let content_types = vec![
            ContentType::Html,
            ContentType::PlainText,
            ContentType::Json,
            ContentType::Xml,
        ];

        for content_type in content_types {
            let serialized = serde_json::to_string(&content_type).unwrap();
            let deserialized: ContentType = serde_json::from_str(&serialized).unwrap();
            assert!(matches!(
                (content_type, deserialized),
                (ContentType::Html, ContentType::Html)
                | (ContentType::PlainText, ContentType::PlainText)
                | (ContentType::Json, ContentType::Json)
                | (ContentType::Xml, ContentType::Xml)
            ));
        }
    }

    #[test]
    fn test_html_content_clone() {
        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(1024),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text_content: "Test content".to_string(),
            raw_html: "<html><body>Test</body></html>".to_string(),
            metadata,
        };

        let cloned = content.clone();
        assert_eq!(content.url, cloned.url);
        assert_eq!(content.title, cloned.title);
        assert_eq!(content.text_content, cloned.text_content);
        assert_eq!(content.raw_html, cloned.raw_html);
    }

    #[test]
    fn test_large_content_handling() {
        let large_text = "a".repeat(1_000_000);
        let large_html = format!("<html><body>{}</body></html>", large_text);

        let metadata = ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(large_html.len()),
            last_modified: None,
            charset: Some("utf-8".to_string()),
        };

        let content = HtmlContent {
            url: "https://example.com/large".to_string(),
            title: Some("Large Content".to_string()),
            text_content: large_text.clone(),
            raw_html: large_html.clone(),
            metadata,
        };

        assert_eq!(content.text_content.len(), 1_000_000);
        assert_eq!(content.raw_html.len(), large_html.len());
        assert_eq!(content.metadata.content_length, Some(large_html.len()));
    }
}
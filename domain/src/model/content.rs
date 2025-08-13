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
use std::time::Duration;
use async_trait::async_trait;
use reqwest::{Client, Response};
use tracing::{info, error, debug};
use domain::model::{
    content::{HtmlContent, ContentMetadata},
    request::FetchContentRequest,
};
use domain::port::content_fetcher::{ContentFetcher, ContentFetcherResult, ContentFetcherError};

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("html-mcp-reader/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    async fn build_request(&self, request: &FetchContentRequest) -> Result<reqwest::Request, ContentFetcherError> {
        let mut req_builder = self.client.get(&request.url);

        if let Some(timeout) = request.timeout_seconds {
            req_builder = req_builder.timeout(Duration::from_secs(timeout));
        }

        if let Some(user_agent) = &request.user_agent {
            req_builder = req_builder.header("User-Agent", user_agent);
        }

        req_builder = req_builder.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8");

        req_builder.build().map_err(|e| {
            ContentFetcherError::Network(format!("Failed to build request: {}", e))
        })
    }

    async fn execute_request(&self, req: reqwest::Request) -> Result<Response, ContentFetcherError> {
        debug!("Executing HTTP request to: {}", req.url());
        
        self.client.execute(req).await.map_err(|e| {
            if e.is_timeout() {
                ContentFetcherError::Timeout(30) // Default timeout
            } else if e.is_connect() {
                ContentFetcherError::Network(format!("Connection failed: {}", e))
            } else {
                ContentFetcherError::Network(format!("Request failed: {}", e))
            }
        })
    }

    fn create_metadata(&self, response: &Response) -> ContentMetadata {
        ContentMetadata {
            content_type: response
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("text/html")
                .to_string(),
            status_code: response.status().as_u16(),
            content_length: response.content_length().map(|l| l as usize),
            last_modified: response
                .headers()
                .get("last-modified")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            charset: None, // Could be extracted from content-type header
        }
    }
}

#[async_trait]
impl ContentFetcher for HttpClient {
    async fn fetch_content(&self, request: FetchContentRequest) -> ContentFetcherResult<HtmlContent> {
        info!("Fetching content from URL: {}", request.url);

        let req = self.build_request(&request).await?;
        let response = self.execute_request(req).await?;

        if !response.status().is_success() {
            return Err(ContentFetcherError::Http {
                status: response.status().as_u16(),
                message: format!("HTTP {} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("Unknown")),
            });
        }

        let metadata = self.create_metadata(&response);
        let final_url = response.url().to_string();
        
        let raw_html = response.text().await.map_err(|e| {
            ContentFetcherError::Network(format!("Failed to read response body: {}", e))
        })?;

        // Extract title using basic regex for now
        let title = extract_title(&raw_html);
        
        // Extract text content if requested
        let text_content = if request.extract_text_only {
            extract_text_content(&raw_html)
        } else {
            raw_html.clone()
        };

        info!("Successfully fetched {} bytes from {}", raw_html.len(), final_url);

        Ok(HtmlContent {
            url: final_url,
            title,
            text_content,
            raw_html,
            metadata,
        })
    }
}

fn extract_title(html: &str) -> Option<String> {
    use regex::Regex;
    
    let title_regex = Regex::new(r"<title[^>]*>([^<]*)</title>").ok()?;
    title_regex
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| html_escape::decode_html_entities(m.as_str().trim()).to_string())
}

fn extract_text_content(html: &str) -> String {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    
    // Remove script and style elements
    let script_selector = Selector::parse("script, style").unwrap();
    let text_selector = Selector::parse("body").unwrap();
    
    let body = document.select(&text_selector).next();
    
    if let Some(body_element) = body {
        body_element.text().collect::<Vec<_>>().join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        // Fallback: extract all text
        document.root_element().text().collect::<Vec<_>>().join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
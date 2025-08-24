use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use domain::model::content::BrowserOptions;
use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;

pub struct BrowserContentFetcher {
    browser: Arc<Browser>,
}

impl BrowserContentFetcher {
    pub async fn new() -> Result<Self, ContentFetcherError> {
        // Try to find Chrome/Chromium executable
        let chrome_paths = vec![
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome", 
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            "/opt/google/chrome/chrome",
            "/snap/bin/chromium",
        ];
        
        let chrome_path = chrome_paths.iter()
            .find(|path| std::path::Path::new(path).exists())
            .cloned();
        
        // Create unique profile directory for each instance with timestamp
        let profile_dir = format!("/tmp/html-mcp-reader-chrome-{}-{}", 
            std::process::id(), 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        );
        
        let mut config_builder = BrowserConfig::builder()
            .args(vec![
                "--no-sandbox",
                "--disable-setuid-sandbox", 
                "--disable-dev-shm-usage",
                "--disable-gpu",
                "--disable-extensions",
                "--disable-default-apps",
                "--disable-sync",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-web-security",
                "--disable-features=VizDisplayCompositor",
                "--headless", // Force headless mode for server environment
                "--disable-background-timer-throttling",
                "--disable-backgrounding-occluded-windows",
                "--disable-renderer-backgrounding",
                "--remote-debugging-port=0", // Use any available port
                "--disable-process-singleton-dialog", // Disable singleton warnings
                &format!("--user-data-dir={}", profile_dir),
            ]);
            
        if let Some(path) = chrome_path {
            config_builder = config_builder.chrome_executable(path);
        }
        
        let browser_config = config_builder.build().unwrap();
            
        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| {
                ContentFetcherError::Network(format!("Failed to launch Chrome browser: {}. Make sure Chrome/Chromium is installed.", e))
            })?;

        // Spawn the browser handler
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            browser: Arc::new(browser),
        })
    }

    pub async fn fetch_with_browser(
        &self,
        url: &str,
        options: &BrowserOptions,
    ) -> Result<String, ContentFetcherError> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| ContentFetcherError::Network(format!("Failed to create page: {}", e)))?;

        // Configure page based on options
        // Note: Request interception is more complex in chromiumoxide
        // For now, we'll skip image blocking to keep it simple

        if let Some(user_agent) = &options.user_agent {
            page.set_user_agent(user_agent)
                .await
                .map_err(|e| ContentFetcherError::Network(format!("Failed to set user agent: {}", e)))?;
        }

        // Navigate to the page
        page.goto(url)
            .await
            .map_err(|e| ContentFetcherError::Network(format!("Failed to navigate to {}: {}", url, e)))?;

        // Wait for JavaScript execution if requested
        if options.wait_for_js {
            tokio::time::sleep(Duration::from_millis(options.timeout_ms)).await;
        }

        // Wait for specific selector if provided
        if let Some(selector) = &options.wait_for_selector {
            let timeout_duration = Duration::from_millis(options.timeout_ms);
            
            tokio::time::timeout(timeout_duration, async {
                loop {
                    if let Ok(_element) = page.find_element(selector).await {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await
            .map_err(|_| {
                ContentFetcherError::Timeout(30)
            })?;
        }

        // Get the page content after JavaScript execution
        let html = page
            .content()
            .await
            .map_err(|e| ContentFetcherError::Network(format!("Failed to get page content: {}", e)))?;

        Ok(html)
    }

    pub async fn detect_javascript(&self, html: &str) -> bool {
        let indicators = [
            "react", "vue", "angular", "next.js",
            "data-reactroot", "ng-app", "v-app",
            "__NUXT__", "__NEXT_DATA__",
            "src=\"/_next/", "chunk-vendors",
            "<script", "javascript:",
            "document.addEventListener",
            "window.onload",
            "$(document).ready",
        ];

        let html_lower = html.to_lowercase();
        indicators.iter().any(|&indicator| html_lower.contains(indicator))
    }

    fn extract_title(&self, html: &str) -> Option<String> {
        use regex::Regex;
        
        let title_regex = Regex::new(r"<title[^>]*>([^<]*)</title>").ok()?;
        title_regex
            .captures(html)
            .and_then(|caps| caps.get(1))
            .map(|m| html_escape::decode_html_entities(m.as_str().trim()).to_string())
    }

    fn extract_text_content(&self, html: &str) -> String {
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
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
}

#[async_trait]
impl ContentFetcher for BrowserContentFetcher {
    async fn fetch_content(&self, request: domain::model::request::FetchContentRequest) -> Result<domain::model::content::HtmlContent, ContentFetcherError> {
        let default_options = BrowserOptions {
            wait_for_js: true,
            timeout_ms: request.timeout_seconds.unwrap_or(10).saturating_mul(1000) as u64,
            wait_for_selector: None,
            disable_images: true,
            user_agent: request.user_agent.clone().or_else(|| Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string())),
        };

        let raw_html = self.fetch_with_browser(&request.url, &default_options).await?;
        
        // Extract title using basic regex
        let title = self.extract_title(&raw_html);
        
        // Extract text content if requested
        let text_content = if request.extract_text_only.unwrap_or(true) {
            self.extract_text_content(&raw_html)
        } else {
            raw_html.clone()
        };

        let metadata = domain::model::content::ContentMetadata {
            content_type: "text/html".to_string(),
            status_code: 200,
            content_length: Some(raw_html.len()),
            last_modified: None,
            charset: Some("utf-8".to_string()),
            javascript_detected: Some(true),
            fetch_method: Some(domain::model::content::FetchMethod::Browser),
        };

        Ok(domain::model::content::HtmlContent {
            url: request.url.clone(),
            title,
            text_content,
            raw_html,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_javascript_detection() {
        let fetcher = BrowserContentFetcher::new().await.unwrap();
        
        // Test with JavaScript content
        let js_html = r#"<html><body><script>console.log('test');</script></body></html>"#;
        assert!(fetcher.detect_javascript(js_html).await);

        // Test with React content
        let react_html = r#"<html><body><div data-reactroot></div></body></html>"#;
        assert!(fetcher.detect_javascript(react_html).await);

        // Test with plain HTML
        let plain_html = r#"<html><body><p>Just plain text</p></body></html>"#;
        assert!(!fetcher.detect_javascript(plain_html).await);
    }

    #[test]
    fn test_browser_options_creation() {
        let options = BrowserOptions {
            wait_for_js: true,
            timeout_ms: 5000,
            wait_for_selector: Some("#content".to_string()),
            disable_images: false,
            user_agent: Some("test-agent".to_string()),
        };

        assert_eq!(options.wait_for_js, true);
        assert_eq!(options.timeout_ms, 5000);
        assert_eq!(options.wait_for_selector, Some("#content".to_string()));
        assert_eq!(options.disable_images, false);
        assert_eq!(options.user_agent, Some("test-agent".to_string()));
    }
}
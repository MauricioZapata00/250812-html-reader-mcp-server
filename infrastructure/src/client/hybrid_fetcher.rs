use async_trait::async_trait;
use domain::model::content::{BrowserOptions, FetchMethod};
use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError};
use std::sync::Arc;

use super::browser_client::BrowserContentFetcher;
use super::http_client::HttpClient;

pub struct HybridContentFetcher {
    http_fetcher: Arc<HttpClient>,
    browser_fetcher: Arc<BrowserContentFetcher>,
    browser_options: BrowserOptions,
}

impl HybridContentFetcher {
    pub async fn new(browser_options: Option<BrowserOptions>) -> Result<Self, ContentFetcherError> {
        let http_fetcher = Arc::new(HttpClient::new());
        let browser_fetcher = Arc::new(BrowserContentFetcher::new().await?);
        
        let default_browser_options = BrowserOptions {
            wait_for_js: true,
            timeout_ms: 10000,
            wait_for_selector: None,
            disable_images: true,
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()),
        };

        Ok(Self {
            http_fetcher,
            browser_fetcher,
            browser_options: browser_options.unwrap_or(default_browser_options),
        })
    }

    pub async fn fetch_with_method(
        &self,
        request: &domain::model::request::FetchContentRequest,
        method: FetchMethod,
    ) -> Result<domain::model::content::HtmlContent, ContentFetcherError> {
        match method {
            FetchMethod::Static => self.http_fetcher.fetch_content(request.clone()).await,
            FetchMethod::Browser => self.browser_fetcher.fetch_content(request.clone()).await,
        }
    }

    pub async fn detect_and_fetch(&self, request: &domain::model::request::FetchContentRequest) -> Result<(domain::model::content::HtmlContent, FetchMethod), ContentFetcherError> {
        // First try with static fetcher
        let static_content = self.http_fetcher.fetch_content(request.clone()).await?;
        
        // Check if JavaScript is detected
        let has_javascript = self.browser_fetcher.detect_javascript(&static_content.raw_html).await;
        
        if has_javascript {
            // Try browser fetcher for JavaScript content, fallback to static if it fails
            match self.browser_fetcher.fetch_content(request.clone()).await {
                Ok(mut browser_content) => {
                    browser_content.metadata.javascript_detected = Some(true);
                    browser_content.metadata.fetch_method = Some(FetchMethod::Browser);
                    Ok((browser_content, FetchMethod::Browser))
                }
                Err(_) => {
                    // Browser failed, use static content as fallback
                    let mut static_result = static_content;
                    static_result.metadata.javascript_detected = Some(true);
                    static_result.metadata.fetch_method = Some(FetchMethod::Static);
                    Ok((static_result, FetchMethod::Static))
                }
            }
        } else {
            // Use static content for plain HTML
            let mut static_result = static_content;
            static_result.metadata.javascript_detected = Some(false);
            static_result.metadata.fetch_method = Some(FetchMethod::Static);
            Ok((static_result, FetchMethod::Static))
        }
    }

    pub async fn is_javascript_heavy(&self, html: &str) -> bool {
        self.browser_fetcher.detect_javascript(html).await
    }

    pub fn set_browser_options(&mut self, options: BrowserOptions) {
        self.browser_options = options;
    }
}

#[async_trait]
impl ContentFetcher for HybridContentFetcher {
    async fn fetch_content(&self, request: domain::model::request::FetchContentRequest) -> Result<domain::model::content::HtmlContent, ContentFetcherError> {
        let (content, _method) = self.detect_and_fetch(&request).await?;
        Ok(content)
    }
}

pub struct JavaScriptDetector;

impl JavaScriptDetector {
    pub fn detect_spa_frameworks(html: &str) -> Vec<String> {
        let mut detected_frameworks = Vec::new();
        let html_lower = html.to_lowercase();

        let framework_indicators = [
            ("React", vec!["data-reactroot", "__REACT", "react.production", "react.development"]),
            ("Vue", vec!["v-app", "__VUE__", "vue.js", "vue.runtime"]),
            ("Angular", vec!["ng-app", "ng-version", "_angular", "angular.js"]),
            ("Next.js", vec!["__NEXT_DATA__", "_next/", "next.js"]),
            ("Nuxt", vec!["__NUXT__", "_nuxt/", "nuxt.js"]),
            ("Svelte", vec!["svelte", "_svelte"]),
            ("jQuery", vec!["jquery", "$(", "jQuery"]),
        ];

        for (framework, indicators) in framework_indicators {
            if indicators.iter().any(|&indicator| html_lower.contains(indicator)) {
                detected_frameworks.push(framework.to_string());
            }
        }

        detected_frameworks
    }

    pub fn has_significant_javascript(html: &str) -> bool {
        let html_lower = html.to_lowercase();
        
        // Count JavaScript indicators
        let js_indicators = [
            "<script",
            "javascript:",
            "document.addEventListener",
            "window.onload",
            "$(document)",
            "fetch(",
            "xhr",
            "xmlhttprequest",
        ];

        let js_count = js_indicators
            .iter()
            .map(|&indicator| html_lower.matches(indicator).count())
            .sum::<usize>();

        // Consider it JavaScript-heavy if there are more than 2 indicators
        js_count > 2
    }

    pub fn extract_script_content(html: &str) -> Vec<String> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let script_selector = Selector::parse("script").unwrap();
        
        document
            .select(&script_selector)
            .filter_map(|element| {
                let text = element.inner_html();
                if !text.trim().is_empty() && !text.contains("src=") {
                    Some(text)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_spa_frameworks() {
        let react_html = r#"<div data-reactroot><script>window.__REACT</script></div>"#;
        let frameworks = JavaScriptDetector::detect_spa_frameworks(react_html);
        assert!(frameworks.contains(&"React".to_string()));

        let vue_html = r#"<div id="app" v-app><script>window.__VUE__</script></div>"#;
        let frameworks = JavaScriptDetector::detect_spa_frameworks(vue_html);
        assert!(frameworks.contains(&"Vue".to_string()));
    }

    #[test]
    fn test_has_significant_javascript() {
        let js_heavy = r#"
            <script>document.addEventListener('DOMContentLoaded', function() {});</script>
            <script>window.onload = function() {};</script>
            <script>fetch('/api/data');</script>
        "#;
        assert!(JavaScriptDetector::has_significant_javascript(js_heavy));

        let plain_html = r#"<html><body><p>Just plain text</p></body></html>"#;
        assert!(!JavaScriptDetector::has_significant_javascript(plain_html));
    }

    #[test]
    fn test_extract_script_content() {
        let html = r#"
            <html>
                <body>
                    <script>console.log('inline script');</script>
                    <script src="external.js"></script>
                    <script>var data = {key: 'value'};</script>
                </body>
            </html>
        "#;
        
        let scripts = JavaScriptDetector::extract_script_content(html);
        assert_eq!(scripts.len(), 2); // Should only get inline scripts, not external ones
        assert!(scripts.iter().any(|s| s.contains("console.log")));
        assert!(scripts.iter().any(|s| s.contains("var data")));
    }
}
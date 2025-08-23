use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tracing::{info, error};
use tower_http::cors::CorsLayer;

use domain::model::{
    request::{FetchContentRequest, ApiErrorResponse, HealthResponse},
    content::HtmlContent,
};
use application::use_case::fetch_web_content_use_case::FetchWebContentUseCase;
use domain::port::{content_fetcher::ContentFetcher, content_parser::ContentParser};

pub struct ApiServer<F, P>
where
    F: ContentFetcher,
    P: ContentParser,
{
    use_case: Arc<FetchWebContentUseCase<F, P>>,
}

impl<F, P> ApiServer<F, P>
where
    F: ContentFetcher + Send + Sync + 'static,
    P: ContentParser + Send + Sync + 'static,
{
    pub fn new(use_case: Arc<FetchWebContentUseCase<F, P>>) -> Self {
        Self { use_case }
    }

    pub fn create_router(self) -> Router {
        let shared_state = Arc::new(self);
        
        Router::new()
            .route("/health", get(health_check))
            .route("/api/fetch", post(fetch_content))
            .with_state(shared_state)
            .layer(CorsLayer::permissive())
    }
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn fetch_content<F, P>(
    State(server): State<Arc<ApiServer<F, P>>>,
    Json(mut request): Json<FetchContentRequest>,
) -> Result<Json<HtmlContent>, (StatusCode, Json<ApiErrorResponse>)>
where
    F: ContentFetcher + Send + Sync,
    P: ContentParser + Send + Sync,
{
    if request.url.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiErrorResponse {
                error: "INVALID_URL".to_string(),
                message: "URL cannot be empty".to_string(),
            })
        ));
    }

    // Apply defaults for optional fields
    request.extract_text_only = request.extract_text_only.or(Some(true));
    request.follow_redirects = request.follow_redirects.or(Some(true));
    request.timeout_seconds = request.timeout_seconds.or(Some(30));
    request.user_agent = request.user_agent.or(Some("html-api-reader/0.1.0".to_string()));

    // Convert optional fields to non-optional for internal processing
    let internal_request = domain::model::request::FetchContentRequest {
        url: request.url,
        extract_text_only: request.extract_text_only,
        follow_redirects: request.follow_redirects,
        timeout_seconds: request.timeout_seconds,
        user_agent: request.user_agent,
    };

    match server.use_case.execute_for_api(internal_request).await {
        Ok(content) => {
            info!("Successfully fetched content from: {}", content.url);
            Ok(Json(content))
        }
        Err(error_msg) => {
            error!("Failed to fetch content: {}", error_msg);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "FETCH_ERROR".to_string(),
                    message: error_msg,
                })
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    use domain::model::content::{ContentMetadata, HtmlContent};
    use domain::port::content_fetcher::{ContentFetcher, ContentFetcherError, ContentFetcherResult};
    use domain::port::content_parser::{ContentParser, ContentParserResult};
    use application::service::{
        content_fetch_service::ContentFetchService,
        content_parse_service::ContentParseService,
    };

    struct MockContentFetcher {
        should_succeed: bool,
    }

    impl MockContentFetcher {
        fn new_success() -> Self {
            Self { should_succeed: true }
        }

        fn new_failure() -> Self {
            Self { should_succeed: false }
        }
    }

    #[async_trait]
    impl ContentFetcher for MockContentFetcher {
        async fn fetch_content(&self, request: FetchContentRequest) -> ContentFetcherResult<HtmlContent> {
            if self.should_succeed {
                let metadata = ContentMetadata {
                    content_type: "text/html".to_string(),
                    status_code: 200,
                    content_length: Some(100),
                    last_modified: None,
                    charset: Some("utf-8".to_string()),
                };

                Ok(HtmlContent {
                    url: request.url,
                    title: Some("Test Title".to_string()),
                    text_content: "Test content".to_string(),
                    raw_html: "<html><body>Test</body></html>".to_string(),
                    metadata,
                })
            } else {
                Err(ContentFetcherError::Network("Connection failed".to_string()))
            }
        }
    }

    struct MockContentParser;

    #[async_trait]
    impl ContentParser for MockContentParser {
        async fn parse_html(&self, raw_html: &str, url: &str) -> ContentParserResult<HtmlContent> {
            let metadata = ContentMetadata {
                content_type: "text/html".to_string(),
                status_code: 200,
                content_length: Some(raw_html.len()),
                last_modified: None,
                charset: Some("utf-8".to_string()),
            };

            Ok(HtmlContent {
                url: url.to_string(),
                title: Some("Parsed Title".to_string()),
                text_content: "Parsed content".to_string(),
                raw_html: raw_html.to_string(),
                metadata,
            })
        }

        async fn extract_text(&self, html_content: &HtmlContent) -> ContentParserResult<String> {
            Ok(html_content.text_content.clone())
        }
    }

    fn create_test_server(should_succeed: bool) -> TestServer {
        let fetcher = Arc::new(if should_succeed {
            MockContentFetcher::new_success()
        } else {
            MockContentFetcher::new_failure()
        });
        let parser = Arc::new(MockContentParser);
        
        let fetch_service = Arc::new(ContentFetchService::new(fetcher));
        let parse_service = Arc::new(ContentParseService::new(parser));
        let use_case = Arc::new(FetchWebContentUseCase::new(fetch_service, parse_service));
        
        let server = ApiServer::new(use_case);
        TestServer::new(server.create_router()).unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = create_test_server(true);
        
        let response = server.get("/health").await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let health: HealthResponse = response.json();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_fetch_content_success() {
        let server = create_test_server(true);
        
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };
        
        let response = server.post("/api/fetch").json(&request).await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let content: HtmlContent = response.json();
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.title, Some("Test Title".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_content_empty_url() {
        let server = create_test_server(true);
        
        let request = FetchContentRequest {
            url: "".to_string(),
            extract_text_only: Some(true),
            follow_redirects: Some(true),
            timeout_seconds: Some(30),
            user_agent: Some("test".to_string()),
        };
        
        let response = server.post("/api/fetch").json(&request).await;
        
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        
        let error: ApiErrorResponse = response.json();
        assert_eq!(error.error, "INVALID_URL");
        assert_eq!(error.message, "URL cannot be empty");
    }

    #[tokio::test]
    async fn test_fetch_content_minimal_request() {
        let server = create_test_server(true);
        
        let request = FetchContentRequest {
            url: "https://example.com".to_string(),
            extract_text_only: None,
            follow_redirects: None,
            timeout_seconds: None,
            user_agent: None,
        };
        
        let response = server.post("/api/fetch").json(&request).await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let content: HtmlContent = response.json();
        assert_eq!(content.url, "https://example.com");
    }
}
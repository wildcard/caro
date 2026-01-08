//! HTTP client for Hugging Face Hub API
//!
//! Handles authentication, URL formatting, and streaming downloads from HF Hub.

use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Response,
};
use std::env;

/// Error types for HTTP operations
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Authentication error: {0}")]
    AuthError(String),
}

/// HTTP client wrapper for Hugging Face Hub API
pub struct HfHubClient {
    client: Client,
    auth_token: Option<String>,
}

impl HfHubClient {
    /// Base URL for Hugging Face Hub
    const HF_HUB_BASE_URL: &'static str = "https://huggingface.co";

    /// Create a new HfHubClient
    ///
    /// Automatically reads HF_TOKEN environment variable for authentication.
    pub fn new() -> Result<Self, HttpClientError> {
        let client = Client::builder()
            .user_agent("caro/1.0")
            .build()?;

        let auth_token = env::var("HF_TOKEN").ok();

        Ok(Self {
            client,
            auth_token,
        })
    }

    /// Create a new HfHubClient with custom auth token
    pub fn with_token(token: String) -> Result<Self, HttpClientError> {
        let client = Client::builder()
            .user_agent("caro/1.0")
            .build()?;

        Ok(Self {
            client,
            auth_token: Some(token),
        })
    }

    /// Format a Hugging Face Hub URL for a model file
    ///
    /// # Arguments
    /// * `model_id` - Model identifier (e.g., "meta-llama/Llama-2-7b-hf")
    /// * `filename` - File to download (e.g., "pytorch_model.bin")
    /// * `revision` - Git revision (default: "main")
    ///
    /// # Returns
    /// Full URL to the file on HF Hub
    pub fn get_file_url(
        &self,
        model_id: &str,
        filename: &str,
        revision: Option<&str>,
    ) -> Result<String, HttpClientError> {
        let revision = revision.unwrap_or("main");

        // Format: https://huggingface.co/{model_id}/resolve/{revision}/{filename}
        let url = format!(
            "{}/{}/resolve/{}/{}",
            Self::HF_HUB_BASE_URL,
            model_id,
            revision,
            filename
        );

        Ok(url)
    }

    /// Send a HEAD request to get file metadata
    ///
    /// Used to check if file exists and get its size for resume support.
    ///
    /// # Arguments
    /// * `url` - Full URL to the file
    ///
    /// # Returns
    /// Response with headers (Content-Length, ETag, etc.)
    pub async fn head_request(&self, url: &str) -> Result<Response, HttpClientError> {
        let mut request = self.client.head(url);

        // Add authentication header if available
        if let Some(token) = &self.auth_token {
            request = request.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(HttpClientError::InvalidUrl(format!(
                "HEAD request failed with status: {}",
                response.status()
            )));
        }

        Ok(response)
    }

    /// Start a streaming download
    ///
    /// Returns a Response that can be streamed chunk-by-chunk.
    ///
    /// # Arguments
    /// * `url` - Full URL to download
    /// * `start_byte` - Optional byte offset for resume (HTTP Range header)
    ///
    /// # Returns
    /// Streaming response
    pub async fn download_stream(
        &self,
        url: &str,
        start_byte: Option<u64>,
    ) -> Result<Response, HttpClientError> {
        let mut request = self.client.get(url);

        // Add authentication header if available
        if let Some(token) = &self.auth_token {
            request = request.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        // Add Range header for resume support
        if let Some(start) = start_byte {
            request = request.header("Range", format!("bytes={}-", start));
        }

        let response = request.send().await?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            // 206 = Partial Content (valid for Range requests)
            return Err(HttpClientError::InvalidUrl(format!(
                "Download request failed with status: {}",
                response.status()
            )));
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HfHubClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_token() {
        let client = HfHubClient::with_token("hf_test_token".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_get_file_url() {
        let client = HfHubClient::new().unwrap();

        let url = client
            .get_file_url("meta-llama/Llama-2-7b-hf", "pytorch_model.bin", None)
            .unwrap();

        assert_eq!(
            url,
            "https://huggingface.co/meta-llama/Llama-2-7b-hf/resolve/main/pytorch_model.bin"
        );
    }

    #[test]
    fn test_get_file_url_with_revision() {
        let client = HfHubClient::new().unwrap();

        let url = client
            .get_file_url("meta-llama/Llama-2-7b-hf", "config.json", Some("v1.0"))
            .unwrap();

        assert_eq!(
            url,
            "https://huggingface.co/meta-llama/Llama-2-7b-hf/resolve/v1.0/config.json"
        );
    }

    // Integration tests with wiremock (require wiremock feature)
    #[cfg(test)]
    mod integration_tests {
        use super::*;
        use wiremock::{
            matchers::{header, method, path},
            Mock, MockServer, ResponseTemplate,
        };

        #[tokio::test]
        async fn test_head_request_success() {
            let mock_server = MockServer::start().await;

            Mock::given(method("HEAD"))
                .and(path("/test-model/resolve/main/file.bin"))
                .respond_with(
                    ResponseTemplate::new(200).insert_header("Content-Length", "1024"),
                )
                .mount(&mock_server)
                .await;

            let client = HfHubClient::new().unwrap();
            let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

            let response = client.head_request(&url).await;
            assert!(response.is_ok());

            let response = response.unwrap();
            assert_eq!(response.status(), 200);
            assert_eq!(
                response
                    .headers()
                    .get("Content-Length")
                    .and_then(|v| v.to_str().ok()),
                Some("1024")
            );
        }

        #[tokio::test]
        async fn test_head_request_with_auth() {
            let mock_server = MockServer::start().await;

            Mock::given(method("HEAD"))
                .and(path("/test-model/resolve/main/file.bin"))
                .and(header("Authorization", "Bearer test_token"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&mock_server)
                .await;

            let client = HfHubClient::with_token("test_token".to_string()).unwrap();
            let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

            let response = client.head_request(&url).await;
            assert!(response.is_ok());
        }

        #[tokio::test]
        async fn test_download_stream_success() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/test-model/resolve/main/file.bin"))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(b"test data"))
                .mount(&mock_server)
                .await;

            let client = HfHubClient::new().unwrap();
            let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

            let response = client.download_stream(&url, None).await;
            assert!(response.is_ok());

            let response = response.unwrap();
            assert_eq!(response.status(), 200);

            let body = response.bytes().await.unwrap();
            assert_eq!(body.as_ref(), b"test data");
        }

        #[tokio::test]
        async fn test_download_stream_with_range() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/test-model/resolve/main/file.bin"))
                .and(header("Range", "bytes=100-"))
                .respond_with(ResponseTemplate::new(206).set_body_bytes(b"partial data"))
                .mount(&mock_server)
                .await;

            let client = HfHubClient::new().unwrap();
            let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

            let response = client.download_stream(&url, Some(100)).await;
            assert!(response.is_ok());

            let response = response.unwrap();
            assert_eq!(response.status(), 206); // Partial Content
        }

        #[tokio::test]
        async fn test_download_stream_with_auth() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/test-model/resolve/main/file.bin"))
                .and(header("Authorization", "Bearer test_token"))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(b"authenticated data"))
                .mount(&mock_server)
                .await;

            let client = HfHubClient::with_token("test_token".to_string()).unwrap();
            let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

            let response = client.download_stream(&url, None).await;
            assert!(response.is_ok());

            let body = response.unwrap().bytes().await.unwrap();
            assert_eq!(body.as_ref(), b"authenticated data");
        }
    }
}

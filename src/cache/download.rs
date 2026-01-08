//! Download orchestration for Hugging Face Hub files
//!
//! Manages streaming downloads with progress bars, .part file handling, and atomic completion.

use crate::cache::{CacheError, HfHubClient};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download a file from Hugging Face Hub
///
/// Downloads the file in a streaming fashion, writing chunks to disk as they arrive.
/// Uses a temporary .part file during download, then renames atomically on completion.
///
/// # Arguments
/// * `client` - HTTP client for HF Hub
/// * `url` - Full URL to download
/// * `dest_path` - Final destination path
/// * `expected_size` - Expected file size in bytes (for progress bar)
///
/// # Returns
/// Path to the downloaded file
///
/// # Example
/// ```no_run
/// use caro::cache::{HfHubClient, download_file};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = HfHubClient::new()?;
/// let url = client.get_file_url("meta-llama/Llama-2-7b-hf", "config.json", None)?;
/// let dest = PathBuf::from("/tmp/config.json");
///
/// download_file(&client, &url, &dest, Some(1024)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn download_file(
    client: &HfHubClient,
    url: &str,
    dest_path: &Path,
    expected_size: Option<u64>,
) -> Result<PathBuf, CacheError> {
    // Create .part file path
    let part_path = dest_path.with_extension("part");

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Start streaming download
    let response = client
        .download_stream(url, None)
        .await
        .map_err(|e| CacheError::DownloadFailed(e.to_string()))?;

    // Get actual content length if not provided
    let content_length = expected_size.or_else(|| {
        response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
    });

    // Create progress bar
    let progress = create_progress_bar(content_length);

    // Open .part file for writing
    let mut file = File::create(&part_path).await?;

    // Stream chunks to disk
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            CacheError::DownloadFailed(format!("Failed to read chunk: {}", e))
        })?;

        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        progress.set_position(downloaded);
    }

    // Ensure all data is written
    file.flush().await?;
    drop(file);

    progress.finish_with_message("Download complete");

    // Atomically rename .part -> final
    tokio::fs::rename(&part_path, dest_path).await?;

    Ok(dest_path.to_path_buf())
}

/// Create a progress bar for downloads
fn create_progress_bar(content_length: Option<u64>) -> ProgressBar {
    match content_length {
        Some(len) => {
            let pb = ProgressBar::new(len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                    .expect("Failed to create progress bar template")
                    .progress_chars("#>-"),
            );
            pb
        }
        None => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})")
                    .expect("Failed to create spinner template"),
            );
            pb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::HfHubClient;
    use tempfile::TempDir;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_download_file_success() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("test_file.bin");

        // Mock the download endpoint
        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(b"test file content")
                    .insert_header("Content-Length", "17"),
            )
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, Some(17)).await;
        assert!(result.is_ok());

        // Verify file was downloaded
        assert!(dest_path.exists());
        let content = tokio::fs::read_to_string(&dest_path).await.unwrap();
        assert_eq!(content, "test file content");

        // Verify .part file was removed
        let part_path = dest_path.with_extension("part");
        assert!(!part_path.exists());
    }

    #[tokio::test]
    async fn test_download_file_creates_parent_directory() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("subdir").join("file.bin");

        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"data"))
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None).await;
        assert!(result.is_ok());
        assert!(dest_path.exists());
        assert!(dest_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_download_file_streaming() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("large_file.bin");

        // Create a larger response body (1MB)
        let large_data = vec![b'A'; 1024 * 1024];

        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/large.bin"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(large_data.clone())
                    .insert_header("Content-Length", "1048576"),
            )
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/large.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, Some(1024 * 1024)).await;
        assert!(result.is_ok());

        // Verify file size
        let metadata = tokio::fs::metadata(&dest_path).await.unwrap();
        assert_eq!(metadata.len(), 1024 * 1024);
    }

    #[tokio::test]
    async fn test_download_file_without_content_length() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("unknown_size.bin");

        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"content without length"))
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None).await;
        assert!(result.is_ok());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_create_progress_bar_with_size() {
        let pb = create_progress_bar(Some(1024));
        assert_eq!(pb.length(), Some(1024));
    }

    #[test]
    fn test_create_progress_bar_without_size() {
        let pb = create_progress_bar(None);
        assert!(pb.is_finished() == false);
    }
}

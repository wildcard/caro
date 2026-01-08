//! Download orchestration for Hugging Face Hub files
//!
//! Manages streaming downloads with progress bars, .part file handling, and atomic completion.

use crate::cache::{CacheError, HfHubClient, StreamingHasher};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download a file from Hugging Face Hub with checksum validation and resume support
///
/// Downloads the file in a streaming fashion, writing chunks to disk as they arrive.
/// Uses a temporary .part file during download, then renames atomically on completion.
/// Automatically detects and resumes from existing .part files using HTTP Range requests.
/// Validates the downloaded file's SHA256 checksum (works correctly with resumed downloads).
///
/// # Resume Behavior
/// If a .part file exists at the destination, the download will automatically resume from
/// where it left off using HTTP Range requests. The checksum computation accounts for
/// the already-downloaded content, ensuring integrity validation works correctly.
///
/// # Arguments
/// * `client` - HTTP client for HF Hub
/// * `url` - Full URL to download
/// * `dest_path` - Final destination path
/// * `expected_size` - Expected file size in bytes (for progress bar)
/// * `expected_checksum` - Optional SHA256 checksum to validate against
///
/// # Returns
/// Tuple of (path to downloaded file, computed checksum)
///
/// # Errors
/// Returns `CacheError::ChecksumMismatch` if expected_checksum is provided and doesn't match
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
/// let (path, checksum) = download_file(&client, &url, &dest, Some(1024), None).await?;
/// println!("Downloaded to {:?} with checksum {}", path, checksum);
/// # Ok(())
/// # }
/// ```
pub async fn download_file(
    client: &HfHubClient,
    url: &str,
    dest_path: &Path,
    expected_size: Option<u64>,
    expected_checksum: Option<&str>,
) -> Result<(PathBuf, String), CacheError> {
    // Create .part file path
    let part_path = dest_path.with_extension("part");

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Check if partial download exists and resume if possible
    let (resume_from, mut hasher) = if part_path.exists() {
        // Get existing file size
        let metadata = tokio::fs::metadata(&part_path).await?;
        let existing_size = metadata.len();

        // Hash the existing content to continue checksum computation
        let existing_data = tokio::fs::read(&part_path).await?;
        let mut hasher = StreamingHasher::new();
        hasher.update(&existing_data);

        (Some(existing_size), hasher)
    } else {
        (None, StreamingHasher::new())
    };

    // Start streaming download (with Range header if resuming)
    let response = client
        .download_stream(url, resume_from)
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

    // Adjust content length if resuming (add already downloaded bytes)
    let total_size = if let (Some(resume), Some(remaining)) = (resume_from, content_length) {
        Some(resume + remaining)
    } else {
        content_length
    };

    // Create progress bar
    let progress = create_progress_bar(total_size);

    // Set initial progress if resuming
    if let Some(resume) = resume_from {
        progress.set_position(resume);
    }

    // Open .part file for writing (append if resuming, create otherwise)
    let mut file = if resume_from.is_some() {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&part_path)
            .await?
    } else {
        File::create(&part_path).await?
    };

    // Stream chunks to disk
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = resume_from.unwrap_or(0);

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            CacheError::DownloadFailed(format!("Failed to read chunk: {}", e))
        })?;

        file.write_all(&chunk).await?;
        hasher.update(&chunk); // Update checksum as we go
        downloaded += chunk.len() as u64;
        progress.set_position(downloaded);
    }

    // Ensure all data is written
    file.flush().await?;
    drop(file);

    progress.finish_with_message("Download complete");

    // Finalize checksum
    let actual_checksum = hasher.finalize();

    // Validate checksum if expected value provided
    if let Some(expected) = expected_checksum {
        if actual_checksum != expected {
            // Delete the invalid file
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(CacheError::ChecksumMismatch {
                model_id: url.to_string(), // Use URL as identifier
                expected: expected.to_string(),
                actual: actual_checksum,
            });
        }
    }

    // Atomically rename .part -> final
    tokio::fs::rename(&part_path, dest_path).await?;

    Ok((dest_path.to_path_buf(), actual_checksum))
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
        matchers::{header, method, path},
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

        let result = download_file(&client, &url, &dest_path, Some(17), None).await;
        assert!(result.is_ok());

        let (path, checksum) = result.unwrap();

        // Verify file was downloaded
        assert_eq!(path, dest_path);
        assert!(dest_path.exists());
        let content = tokio::fs::read_to_string(&dest_path).await.unwrap();
        assert_eq!(content, "test file content");

        // Verify checksum is valid hex string
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

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

        let result = download_file(&client, &url, &dest_path, None, None).await;
        assert!(result.is_ok());

        let (path, _checksum) = result.unwrap();
        assert_eq!(path, dest_path);
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

        let result = download_file(&client, &url, &dest_path, Some(1024 * 1024), None).await;
        assert!(result.is_ok());

        let (_path, checksum) = result.unwrap();

        // Verify file size
        let metadata = tokio::fs::metadata(&dest_path).await.unwrap();
        assert_eq!(metadata.len(), 1024 * 1024);

        // Verify checksum was computed
        assert_eq!(checksum.len(), 64);
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

        let result = download_file(&client, &url, &dest_path, None, None).await;
        assert!(result.is_ok());

        let (path, _checksum) = result.unwrap();
        assert_eq!(path, dest_path);
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

    #[tokio::test]
    async fn test_download_with_valid_checksum() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("checksum_test.bin");

        let test_content = b"test content for checksum validation";

        // Pre-compute the expected checksum
        let mut hasher = StreamingHasher::new();
        hasher.update(test_content);
        let expected_checksum = hasher.finalize();

        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(test_content))
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None, Some(&expected_checksum)).await;
        assert!(result.is_ok());

        let (path, actual_checksum) = result.unwrap();
        assert_eq!(path, dest_path);
        assert_eq!(actual_checksum, expected_checksum);
        assert!(dest_path.exists());
    }

    #[tokio::test]
    async fn test_download_with_invalid_checksum() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("checksum_mismatch.bin");

        let test_content = b"test content";
        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(test_content))
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None, Some(wrong_checksum)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CacheError::ChecksumMismatch {
                model_id,
                expected,
                actual,
            } => {
                assert!(model_id.contains("test-model"));
                assert_eq!(expected, wrong_checksum);
                assert_ne!(actual, wrong_checksum);
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }

        // Verify file was NOT created (cleaned up on failure)
        assert!(!dest_path.exists());
        let part_path = dest_path.with_extension("part");
        assert!(!part_path.exists());
    }

    #[tokio::test]
    async fn test_download_resume_from_partial() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("resume_test.bin");
        let part_path = dest_path.with_extension("part");

        // Full content is "first half" + "second half"
        let full_content = b"first halfsecond half";
        let first_half = b"first half";
        let second_half = b"second half";

        // Write first half to .part file (simulating interrupted download)
        tokio::fs::write(&part_path, first_half).await.unwrap();

        // Mock server should receive Range header and return second half
        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .and(header("Range", "bytes=10-"))
            .respond_with(
                ResponseTemplate::new(206) // 206 = Partial Content
                    .set_body_bytes(second_half)
                    .insert_header("Content-Length", "11"),
            )
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None, None).await;
        assert!(result.is_ok());

        let (path, checksum) = result.unwrap();
        assert_eq!(path, dest_path);

        // Verify final file contains both halves
        let final_content = tokio::fs::read(&dest_path).await.unwrap();
        assert_eq!(final_content, full_content);

        // Verify checksum matches full content
        let mut expected_hasher = StreamingHasher::new();
        expected_hasher.update(full_content);
        let expected_checksum = expected_hasher.finalize();
        assert_eq!(checksum, expected_checksum);

        // Verify .part file was removed
        assert!(!part_path.exists());
    }

    #[tokio::test]
    async fn test_download_resume_with_checksum_validation() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("resume_checksum.bin");
        let part_path = dest_path.with_extension("part");

        let full_content = b"complete content for checksum test";
        let first_part = b"complete content ";
        let second_part = b"for checksum test";

        // Pre-compute expected checksum of full content
        let mut expected_hasher = StreamingHasher::new();
        expected_hasher.update(full_content);
        let expected_checksum = expected_hasher.finalize();

        // Write first part to .part file
        tokio::fs::write(&part_path, first_part).await.unwrap();

        // Mock server returns second part with 206 Partial Content
        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .and(header("Range", "bytes=17-"))
            .respond_with(
                ResponseTemplate::new(206)
                    .set_body_bytes(second_part)
                    .insert_header("Content-Length", "17"),
            )
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        // Download with checksum validation
        let result =
            download_file(&client, &url, &dest_path, None, Some(&expected_checksum)).await;
        assert!(result.is_ok());

        let (path, checksum) = result.unwrap();
        assert_eq!(path, dest_path);
        assert_eq!(checksum, expected_checksum);

        // Verify final file is correct
        let final_content = tokio::fs::read(&dest_path).await.unwrap();
        assert_eq!(final_content, full_content);
    }

    #[tokio::test]
    async fn test_download_no_resume_when_no_part_file() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("no_resume.bin");
        let part_path = dest_path.with_extension("part");

        // Ensure no .part file exists
        assert!(!part_path.exists());

        let test_content = b"fresh download";

        // Mock should NOT receive Range header
        Mock::given(method("GET"))
            .and(path("/test-model/resolve/main/file.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(test_content))
            .mount(&mock_server)
            .await;

        let client = HfHubClient::new().unwrap();
        let url = format!("{}/test-model/resolve/main/file.bin", mock_server.uri());

        let result = download_file(&client, &url, &dest_path, None, None).await;
        assert!(result.is_ok());

        let (path, _checksum) = result.unwrap();
        assert_eq!(path, dest_path);

        let final_content = tokio::fs::read(&dest_path).await.unwrap();
        assert_eq!(final_content, test_content);
    }
}

//! Unit tests for download module using wiremock for HTTP mocking

use caro::cache::{download_file, CacheError, HfHubClient};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to compute SHA256 checksum of bytes
fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Helper to create a test file with known content and checksum
fn create_test_data(size: usize) -> (Vec<u8>, String) {
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    let checksum = compute_checksum(&data);
    (data, checksum)
}

#[tokio::test]
async fn test_successful_download_200_ok() {
    // T052: Test successful download (200 OK) with wiremock

    // Create test data (1KB file)
    let (test_data, expected_checksum) = create_test_data(1024);
    let test_data_len = test_data.len();

    // Setup mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test-file.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(test_data.clone())
                .insert_header("content-length", test_data_len.to_string().as_str()),
        )
        .mount(&mock_server)
        .await;

    // Create temporary destination
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test-file.bin");

    // Create client and download
    let client = HfHubClient::new().unwrap();
    let url = format!("{}/test-file.bin", mock_server.uri());

    let result = download_file(
        &client,
        &url,
        &dest_path,
        Some(test_data_len as u64),
        Some(&expected_checksum),
    )
    .await;

    // Verify success
    assert!(result.is_ok(), "Download should succeed: {:?}", result.err());
    let (path, checksum) = result.unwrap();

    // Verify path and checksum
    assert_eq!(path, dest_path);
    assert_eq!(checksum, expected_checksum);

    // Verify file contents
    let downloaded_data = std::fs::read(&dest_path).unwrap();
    assert_eq!(downloaded_data, test_data);

    // Verify .part file was cleaned up
    let part_path = dest_path.with_extension("bin.part");
    assert!(!part_path.exists(), ".part file should be removed");
}

#[tokio::test]
async fn test_resume_from_partial_download_206() {
    // T053: Test resume from partial (206 Partial Content)

    // Create test data (2KB file)
    let (test_data, expected_checksum) = create_test_data(2048);
    let test_data_len = test_data.len();

    // Simulate partial download: already have first 1KB
    let partial_size = 1024;
    let remaining_data = &test_data[partial_size..];

    // Setup mock server to respond with 206 Partial Content
    let mock_server = MockServer::start().await;

    // First request should be HEAD to check if resume is supported
    Mock::given(method("HEAD"))
        .and(path("/test-file.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("accept-ranges", "bytes")
                .insert_header("content-length", test_data_len.to_string().as_str()),
        )
        .mount(&mock_server)
        .await;

    // Second request should be GET with Range header
    Mock::given(method("GET"))
        .and(path("/test-file.bin"))
        .and(header("Range", format!("bytes={}-", partial_size).as_str()))
        .respond_with(
            ResponseTemplate::new(206)
                .set_body_bytes(remaining_data.to_vec())
                .insert_header("content-range", format!("bytes {}-{}/{}", partial_size, test_data_len - 1, test_data_len).as_str())
                .insert_header("content-length", remaining_data.len().to_string().as_str()),
        )
        .mount(&mock_server)
        .await;

    // Create temporary destination with existing .part file
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test-file.bin");
    let part_path = dest_path.with_extension("part");

    // Write partial data to .part file
    std::fs::write(&part_path, &test_data[..partial_size]).unwrap();

    // Create client and download (should resume)
    let client = HfHubClient::new().unwrap();
    let url = format!("{}/test-file.bin", mock_server.uri());

    let result = download_file(
        &client,
        &url,
        &dest_path,
        Some(test_data_len as u64),
        Some(&expected_checksum),
    )
    .await;

    // Verify success
    assert!(result.is_ok(), "Resume download should succeed: {:?}", result.err());
    let (path, checksum) = result.unwrap();

    // Verify path and checksum
    assert_eq!(path, dest_path);
    assert_eq!(checksum, expected_checksum);

    // Verify complete file contents
    let downloaded_data = std::fs::read(&dest_path).unwrap();
    assert_eq!(downloaded_data, test_data);

    // Verify .part file was cleaned up
    assert!(!part_path.exists(), ".part file should be removed after completion");
}

#[tokio::test]
async fn test_network_timeout_error() {
    // T054: Test network errors (timeouts, connection refused)

    // Create a URL that will cause connection refused (invalid port)
    let client = HfHubClient::new().unwrap();
    let invalid_url = "http://127.0.0.1:1"; // Port 1 should refuse connection

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test-file.bin");

    let result = download_file(&client, invalid_url, &dest_path, Some(1024), None).await;

    // Verify network error
    assert!(result.is_err(), "Should fail with network error");
    match result.unwrap_err() {
        CacheError::NetworkError(msg) => {
            assert!(msg.contains("connect") || msg.contains("connection"), "Error should mention connection issue: {}", msg);
        }
        other => panic!("Expected NetworkError, got: {:?}", other),
    }

    // Verify no file was created
    assert!(!dest_path.exists(), "File should not exist after network error");
}

#[tokio::test]
async fn test_connection_refused_error() {
    // T054 continued: Test connection refused specifically

    // Use localhost with a port that's definitely not listening
    let client = HfHubClient::new().unwrap();
    let invalid_url = "http://localhost:9999/test-file.bin";

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test-file.bin");

    let result = download_file(&client, invalid_url, &dest_path, Some(1024), None).await;

    // Verify network error
    assert!(result.is_err(), "Should fail with network error");
    match result.unwrap_err() {
        CacheError::NetworkError(_) => {
            // Success - got the expected error type
        }
        other => panic!("Expected NetworkError, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_authentication_failure_401() {
    // T055: Test authentication failures (401 Unauthorized)

    let mock_server = MockServer::start().await;

    // Mock 401 response
    Mock::given(method("GET"))
        .and(path("/private-file.bin"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/private-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("private-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify authentication error
    assert!(result.is_err(), "Should fail with authentication error");
    match result.unwrap_err() {
        CacheError::AuthenticationRequired => {
            // Success - got the expected error type
        }
        other => panic!("Expected AuthenticationRequired, got: {:?}", other),
    }

    // Verify no file was created
    assert!(!dest_path.exists(), "File should not exist after auth failure");
}

#[tokio::test]
async fn test_authentication_failure_403() {
    // T055 continued: Test 403 Forbidden (also treated as auth error)

    let mock_server = MockServer::start().await;

    // Mock 403 response
    Mock::given(method("GET"))
        .and(path("/forbidden-file.bin"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/forbidden-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("forbidden-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify authentication error
    assert!(result.is_err(), "Should fail with authentication error");
    match result.unwrap_err() {
        CacheError::AuthenticationRequired => {
            // Success - got the expected error type
        }
        other => panic!("Expected AuthenticationRequired, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_checksum_mismatch_detection() {
    // T056: Test checksum mismatches

    let (test_data, _actual_checksum) = create_test_data(1024);
    let test_data_len = test_data.len();

    // Use a deliberately wrong checksum
    let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test-file.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(test_data.clone())
                .insert_header("content-length", test_data_len.to_string().as_str()),
        )
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/test-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test-file.bin");

    let result = download_file(
        &client,
        &url,
        &dest_path,
        Some(test_data_len as u64),
        Some(wrong_checksum),
    )
    .await;

    // Verify checksum mismatch error
    assert!(result.is_err(), "Should fail with checksum mismatch");
    match result.unwrap_err() {
        CacheError::ChecksumMismatch {
            model_id,
            expected,
            actual,
        } => {
            assert_eq!(expected, wrong_checksum);
            assert_ne!(actual, wrong_checksum);
            // model_id should contain the file path
            assert!(model_id.contains("test-file.bin"));
        }
        other => panic!("Expected ChecksumMismatch, got: {:?}", other),
    }

    // Verify file was NOT created (implementation deletes invalid file)
    assert!(!dest_path.exists(), "File should not exist after checksum mismatch");

    // Verify .part file was also cleaned up
    let part_path = dest_path.with_extension("part");
    assert!(!part_path.exists(), ".part file should be removed after checksum mismatch");
}

#[tokio::test]
async fn test_server_error_500() {
    // T057: Test server errors (500, 503)

    let mock_server = MockServer::start().await;

    // Mock 500 Internal Server Error
    Mock::given(method("GET"))
        .and(path("/error-file.bin"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/error-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("error-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify server error
    assert!(result.is_err(), "Should fail with server error");
    match result.unwrap_err() {
        CacheError::NetworkError(msg) => {
            assert!(msg.contains("500") || msg.contains("Server error"), "Error should mention server error: {}", msg);
        }
        other => panic!("Expected NetworkError for 500 status, got: {:?}", other),
    }

    // Verify no file was created
    assert!(!dest_path.exists(), "File should not exist after server error");
}

#[tokio::test]
async fn test_server_error_503() {
    // T057 continued: Test 503 Service Unavailable

    let mock_server = MockServer::start().await;

    // Mock 503 Service Unavailable
    Mock::given(method("GET"))
        .and(path("/unavailable-file.bin"))
        .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/unavailable-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("unavailable-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify server error
    assert!(result.is_err(), "Should fail with server error");
    match result.unwrap_err() {
        CacheError::NetworkError(msg) => {
            assert!(msg.contains("503") || msg.contains("Server error"), "Error should mention server error: {}", msg);
        }
        other => panic!("Expected NetworkError for 503 status, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_404_not_found() {
    // Additional test: 404 Not Found should be handled as DownloadFailed

    let mock_server = MockServer::start().await;

    // Mock 404 Not Found
    Mock::given(method("GET"))
        .and(path("/missing-file.bin"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/missing-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("missing-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify download failed error
    assert!(result.is_err(), "Should fail with download failed");
    match result.unwrap_err() {
        CacheError::DownloadFailed(msg) => {
            assert!(msg.contains("not found"), "Error should mention not found: {}", msg);
        }
        other => panic!("Expected DownloadFailed for 404 status, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_resume_not_supported_416() {
    // Additional test: 416 Range Not Satisfiable should trigger ResumeNotSupported

    let mock_server = MockServer::start().await;

    // Mock 416 Range Not Satisfiable
    Mock::given(method("GET"))
        .and(path("/no-resume-file.bin"))
        .respond_with(ResponseTemplate::new(416).set_body_string("Range Not Satisfiable"))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/no-resume-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("no-resume-file.bin");

    let result = download_file(&client, &url, &dest_path, Some(1024), None).await;

    // Verify resume not supported error
    assert!(result.is_err(), "Should fail with resume not supported");
    match result.unwrap_err() {
        CacheError::ResumeNotSupported => {
            // Success - got the expected error type
        }
        other => panic!("Expected ResumeNotSupported for 416 status, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_download_without_checksum_validation() {
    // Additional test: Download should succeed without checksum validation

    let (test_data, _checksum) = create_test_data(512);
    let test_data_len = test_data.len();

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/no-checksum-file.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(test_data.clone())
                .insert_header("content-length", test_data_len.to_string().as_str()),
        )
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/no-checksum-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("no-checksum-file.bin");

    // Download without providing expected checksum
    let result = download_file(&client, &url, &dest_path, Some(test_data_len as u64), None).await;

    // Verify success
    assert!(result.is_ok(), "Download without checksum should succeed: {:?}", result.err());
    let (path, computed_checksum) = result.unwrap();

    // Verify path
    assert_eq!(path, dest_path);

    // Verify checksum was still computed (just not validated)
    assert!(!computed_checksum.is_empty());
    assert_eq!(computed_checksum.len(), 64); // SHA256 hex string

    // Verify file contents
    let downloaded_data = std::fs::read(&dest_path).unwrap();
    assert_eq!(downloaded_data, test_data);
}

#[tokio::test]
async fn test_download_with_unknown_size() {
    // Additional test: Download should work even without known size (no progress bar total)

    let (test_data, expected_checksum) = create_test_data(256);

    let mock_server = MockServer::start().await;

    // Don't include content-length header
    Mock::given(method("GET"))
        .and(path("/unknown-size-file.bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(test_data.clone()))
        .mount(&mock_server)
        .await;

    let client = HfHubClient::new().unwrap();
    let url = format!("{}/unknown-size-file.bin", mock_server.uri());

    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("unknown-size-file.bin");

    // Download without providing expected size (None)
    let result = download_file(&client, &url, &dest_path, None, Some(&expected_checksum)).await;

    // Verify success
    assert!(result.is_ok(), "Download with unknown size should succeed: {:?}", result.err());
    let (path, checksum) = result.unwrap();

    // Verify path and checksum
    assert_eq!(path, dest_path);
    assert_eq!(checksum, expected_checksum);

    // Verify file contents
    let downloaded_data = std::fs::read(&dest_path).unwrap();
    assert_eq!(downloaded_data, test_data);
}

// Test utilities for embedded model testing
// Shared helpers for model setup, validation, and common assertions

use std::path::PathBuf;
use std::time::Duration;

/// Get the path to a test model
///
/// This function tries multiple locations:
/// 1. HuggingFace cache directory (if model was downloaded)
/// 2. Environment variable TEST_MODEL_PATH
/// 3. Fallback to /tmp/test_model.gguf
pub fn get_test_model_path() -> PathBuf {
    // Try environment variable first
    if let Ok(path) = std::env::var("TEST_MODEL_PATH") {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return path_buf;
        }
    }

    // Try HuggingFace cache
    if let Some(hf_path) = find_hf_cached_model() {
        return hf_path;
    }

    // Fallback
    PathBuf::from("/tmp/test_model.gguf")
}

/// Find a model in the HuggingFace cache directory
fn find_hf_cached_model() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // Try Qwen model first
    let qwen_cache = format!(
        "{}/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-1.5B-Instruct-GGUF/snapshots",
        home
    );

    if let Some(model) = find_model_in_cache(&qwen_cache, "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf") {
        return Some(model);
    }

    // Try other common models
    let cache_dirs = vec![
        format!("{}/.cache/huggingface/hub", home),
    ];

    for cache_dir in cache_dirs {
        if let Some(model) = find_any_gguf_in_cache(&cache_dir) {
            return Some(model);
        }
    }

    None
}

/// Find a specific model file in a cache directory
fn find_model_in_cache(cache_dir: &str, model_name: &str) -> Option<PathBuf> {
    let path = std::path::Path::new(cache_dir);

    if !path.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let model_file = entry.path().join(model_name);
            if model_file.exists() {
                return Some(model_file);
            }
        }
    }

    None
}

/// Find any GGUF model in a cache directory (recursive)
fn find_any_gguf_in_cache(cache_dir: &str) -> Option<PathBuf> {
    let path = std::path::Path::new(cache_dir);

    if !path.exists() {
        return None;
    }

    // Walk directory to find .gguf files
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == "gguf" {
                        return Some(entry_path);
                    }
                }
            } else if entry_path.is_dir() {
                // Recurse into subdirectories
                if let Some(model) = find_any_gguf_in_cache(entry_path.to_str()?) {
                    return Some(model);
                }
            }
        }
    }

    None
}

/// Check if a test model is available
pub fn is_test_model_available() -> bool {
    get_test_model_path().exists()
}

/// Skip test if model is not available
///
/// Returns `Some(path)` if model exists, `None` if test should be skipped
pub fn require_test_model() -> Option<PathBuf> {
    let path = get_test_model_path();

    if !path.exists() {
        eprintln!("Skipping test: model not found at {:?}", path);
        eprintln!("Set TEST_MODEL_PATH environment variable or download model to HF cache");
        return None;
    }

    Some(path)
}

/// Common test assertions for inference responses
pub mod assertions {
    /// Assert that a response looks like a valid command or JSON
    pub fn assert_valid_response(response: &str) {
        assert!(!response.is_empty(), "Response should not be empty");

        // Response should be either:
        // 1. Valid JSON
        // 2. Contains shell command-like text
        // 3. Contains common shell utilities
        let is_valid = serde_json::from_str::<serde_json::Value>(response).is_ok()
            || response.contains("cmd")
            || response.contains('{')
            || has_shell_command_indicators(response);

        assert!(
            is_valid,
            "Response should be valid JSON or command-like text: {}",
            response
        );
    }

    /// Check if text contains shell command indicators
    fn has_shell_command_indicators(text: &str) -> bool {
        let common_commands = [
            "ls", "cd", "pwd", "echo", "cat", "grep", "find", "sed", "awk",
            "rm", "cp", "mv", "mkdir", "touch", "chmod", "chown",
        ];

        let text_lower = text.to_lowercase();

        for cmd in &common_commands {
            if text_lower.contains(cmd) {
                return true;
            }
        }

        // Check for path-like strings
        text.contains('/') || text.contains('.')
    }

    /// Assert that a response contains a specific command
    pub fn assert_contains_command(response: &str, expected_cmd: &str) {
        let response_lower = response.to_lowercase();
        let expected_lower = expected_cmd.to_lowercase();

        assert!(
            response_lower.contains(&expected_lower),
            "Response should contain '{}': {}",
            expected_cmd,
            response
        );
    }

    /// Assert that inference completed within a time limit
    pub fn assert_performance(duration: std::time::Duration, max_duration: std::time::Duration) {
        assert!(
            duration <= max_duration,
            "Inference took {:?}, expected <= {:?}",
            duration,
            max_duration
        );
    }
}

/// Performance measurement utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Measure the time taken to execute a closure
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure async execution time
    pub async fn measure_async<F, Fut, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Statistics for multiple measurements
    #[derive(Debug)]
    pub struct Stats {
        pub min: Duration,
        pub max: Duration,
        pub mean: Duration,
        pub median: Duration,
    }

    impl Stats {
        /// Calculate statistics from a list of durations
        pub fn from_durations(mut durations: Vec<Duration>) -> Self {
            durations.sort();

            let min = *durations.first().unwrap();
            let max = *durations.last().unwrap();

            let total: Duration = durations.iter().sum();
            let mean = total / durations.len() as u32;

            let median = durations[durations.len() / 2];

            Self {
                min,
                max,
                mean,
                median,
            }
        }
    }
}

/// Mock model utilities for testing without actual model files
pub mod mocks {
    use std::path::PathBuf;

    /// Create a mock model path (doesn't actually create the file)
    pub fn mock_model_path() -> PathBuf {
        PathBuf::from("/tmp/mock_model.gguf")
    }

    /// Create a temporary model file for testing
    ///
    /// Note: This creates an empty file, not a real model
    pub fn create_temp_model_file() -> std::io::Result<tempfile::NamedTempFile> {
        let temp_file = tempfile::Builder::new()
            .prefix("test_model_")
            .suffix(".gguf")
            .tempfile()?;

        Ok(temp_file)
    }
}

/// Configuration builders for common test scenarios
pub mod config {
    use cmdai::backends::embedded::EmbeddedConfig;

    /// Fast inference config (low quality, high speed)
    pub fn fast_config() -> EmbeddedConfig {
        EmbeddedConfig::default()
            .with_temperature(0.1)
            .with_max_tokens(50)
    }

    /// High quality config (better results, slower)
    pub fn quality_config() -> EmbeddedConfig {
        EmbeddedConfig::default()
            .with_temperature(0.7)
            .with_max_tokens(200)
            .with_top_p(0.95)
    }

    /// Deterministic config (for reproducible tests)
    pub fn deterministic_config() -> EmbeddedConfig {
        EmbeddedConfig::default()
            .with_temperature(0.0)
            .with_max_tokens(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_test_model_path() {
        let path = get_test_model_path();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_is_test_model_available() {
        // Should return a boolean without panicking
        let _ = is_test_model_available();
    }

    #[test]
    fn test_assertions() {
        // Test valid JSON response
        assertions::assert_valid_response(r#"{"cmd": "ls -la"}"#);

        // Test command-like response
        assertions::assert_valid_response("ls -la");

        // Test contains command
        assertions::assert_contains_command("ls -la /tmp", "ls");
    }

    #[test]
    fn test_performance_measurement() {
        use std::time::Duration;

        let (_result, duration) = performance::measure_time(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_stats() {
        use std::time::Duration;

        let durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
            Duration::from_millis(50),
        ];

        let stats = performance::Stats::from_durations(durations);

        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(50));
        assert_eq!(stats.median, Duration::from_millis(30));
    }

    #[test]
    fn test_mock_model_path() {
        let path = mocks::mock_model_path();
        assert!(path.to_str().unwrap().contains("mock_model"));
    }

    #[test]
    fn test_config_builders() {
        let fast = config::fast_config();
        assert_eq!(fast.temperature, 0.1);
        assert_eq!(fast.max_tokens, 50);

        let quality = config::quality_config();
        assert_eq!(quality.temperature, 0.7);
        assert_eq!(quality.max_tokens, 200);

        let deterministic = config::deterministic_config();
        assert_eq!(deterministic.temperature, 0.0);
    }
}

//! Network Speed Testing Module
//!
//! Provides functionality to test network speed and estimate model download times.
//! This helps users select the most appropriate model for their network conditions.

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Size of test data in bytes for speed test (1 MB)
const SPEED_TEST_SIZE_BYTES: u64 = 1024 * 1024;

/// Timeout for speed test in seconds
const SPEED_TEST_TIMEOUT_SECS: u64 = 30;

/// Result of a network speed test
#[derive(Debug, Clone)]
pub struct SpeedTestResult {
    /// Download speed in megabytes per second
    pub speed_mbps: f64,
    /// Time taken for the test in seconds
    pub test_duration_secs: f64,
    /// Whether the test was successful
    pub success: bool,
    /// Error message if test failed
    pub error: Option<String>,
    /// Network quality classification
    pub quality: NetworkQuality,
}

impl SpeedTestResult {
    /// Estimate download time for a file of given size in MB
    pub fn estimate_download_time_secs(&self, size_mb: u64) -> f64 {
        if self.speed_mbps <= 0.0 {
            return f64::INFINITY;
        }
        size_mb as f64 / self.speed_mbps
    }

    /// Format estimated download time as human-readable string
    pub fn format_download_time(&self, size_mb: u64) -> String {
        let secs = self.estimate_download_time_secs(size_mb);
        if secs.is_infinite() || secs > 3600.0 {
            "Unknown".to_string()
        } else if secs < 60.0 {
            format!("{:.0} seconds", secs)
        } else {
            let mins = secs / 60.0;
            format!("{:.1} minutes", mins)
        }
    }
}

impl std::fmt::Display for SpeedTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(
                f,
                "Speed: {:.2} MB/s ({}) - Test took {:.1}s",
                self.speed_mbps, self.quality, self.test_duration_secs
            )
        } else {
            write!(
                f,
                "Speed test failed: {}",
                self.error.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Network quality classification based on download speed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkQuality {
    /// Very slow connection (< 1 MB/s)
    VeryPoor,
    /// Slow connection (1-5 MB/s)
    Poor,
    /// Average connection (5-20 MB/s)
    Average,
    /// Good connection (20-50 MB/s)
    Good,
    /// Excellent connection (> 50 MB/s)
    Excellent,
}

impl NetworkQuality {
    /// Classify speed in MB/s into network quality tier
    pub fn from_speed_mbps(speed: f64) -> Self {
        if speed < 1.0 {
            Self::VeryPoor
        } else if speed < 5.0 {
            Self::Poor
        } else if speed < 20.0 {
            Self::Average
        } else if speed < 50.0 {
            Self::Good
        } else {
            Self::Excellent
        }
    }

    /// Get maximum recommended model size in MB for this network quality
    /// for "instant" use (download within ~30 seconds)
    pub fn instant_model_max_size_mb(&self) -> u64 {
        match self {
            Self::VeryPoor => 30,   // ~30 seconds at 1 MB/s
            Self::Poor => 100,     // ~30 seconds at 3 MB/s
            Self::Average => 350,  // ~30 seconds at 12 MB/s
            Self::Good => 1000,    // ~30 seconds at 35 MB/s
            Self::Excellent => 2000, // ~30 seconds at 60 MB/s
        }
    }

    /// Get maximum recommended model size in MB for background download
    /// (download within ~5 minutes)
    pub fn background_model_max_size_mb(&self) -> u64 {
        match self {
            Self::VeryPoor => 300,    // ~5 minutes at 1 MB/s
            Self::Poor => 900,        // ~5 minutes at 3 MB/s
            Self::Average => 3600,    // ~5 minutes at 12 MB/s
            Self::Good => 10000,      // ~5 minutes at 35 MB/s
            Self::Excellent => 20000, // ~5 minutes at 60 MB/s
        }
    }
}

impl std::fmt::Display for NetworkQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VeryPoor => write!(f, "Very Poor"),
            Self::Poor => write!(f, "Poor"),
            Self::Average => write!(f, "Average"),
            Self::Good => write!(f, "Good"),
            Self::Excellent => write!(f, "Excellent"),
        }
    }
}

/// Network speed tester
pub struct SpeedTester {
    client: Client,
    /// Test URL (uses a small file from Hugging Face by default)
    test_url: String,
}

impl SpeedTester {
    /// Create a new speed tester with default settings
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(SPEED_TEST_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        // Use a small model file from Hugging Face as test target
        // This is a real file that's consistently available
        let test_url =
            "https://huggingface.co/HuggingFaceTB/SmolLM-135M-Instruct-GGUF/resolve/main/smollm-135m-instruct-q4_k_m.gguf".to_string();

        Ok(Self { client, test_url })
    }

    /// Create a speed tester with a custom test URL
    pub fn with_url(url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(SPEED_TEST_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            test_url: url.into(),
        })
    }

    /// Run a quick speed test
    ///
    /// Downloads a portion of the test file and calculates download speed.
    /// The test is designed to complete quickly (within ~5 seconds for most connections).
    pub async fn run_quick_test(&self) -> SpeedTestResult {
        info!("Starting network speed test...");
        let start = Instant::now();

        match self.download_sample().await {
            Ok((bytes_downloaded, duration)) => {
                let speed_mbps = (bytes_downloaded as f64 / 1_000_000.0) / duration.as_secs_f64();
                let quality = NetworkQuality::from_speed_mbps(speed_mbps);

                debug!(
                    "Speed test complete: {:.2} MB/s ({} bytes in {:.2}s)",
                    speed_mbps,
                    bytes_downloaded,
                    duration.as_secs_f64()
                );

                SpeedTestResult {
                    speed_mbps,
                    test_duration_secs: start.elapsed().as_secs_f64(),
                    success: true,
                    error: None,
                    quality,
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                debug!("Speed test failed: {}", error_msg);

                SpeedTestResult {
                    speed_mbps: 0.0,
                    test_duration_secs: start.elapsed().as_secs_f64(),
                    success: false,
                    error: Some(error_msg),
                    quality: NetworkQuality::VeryPoor,
                }
            }
        }
    }

    /// Download a sample of data to measure speed
    async fn download_sample(&self) -> Result<(u64, Duration)> {
        // Use a range request to download only a portion of the file
        let response = self
            .client
            .get(&self.test_url)
            .header("Range", format!("bytes=0-{}", SPEED_TEST_SIZE_BYTES - 1))
            .send()
            .await
            .context("Failed to connect to download server")?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            anyhow::bail!(
                "Server returned error status: {}",
                response.status()
            );
        }

        let start = Instant::now();
        let bytes = response.bytes().await.context("Failed to download data")?;
        let duration = start.elapsed();

        Ok((bytes.len() as u64, duration))
    }

    /// Get estimated download times for common model sizes
    pub fn estimate_download_times(&self, result: &SpeedTestResult) -> Vec<(String, u64, String)> {
        let model_sizes = vec![
            ("SmolLM 135M (Tiny)", 82_u64),
            ("Qwen 0.5B (Small)", 352),
            ("TinyLlama 1.1B", 669),
            ("StarCoder 1B", 700),
            ("Qwen 1.5B (Default)", 1117),
            ("Phi-2 2.7B", 1560),
            ("Mistral 7B", 3520),
        ];

        model_sizes
            .into_iter()
            .map(|(name, size)| {
                let time_str = result.format_download_time(size);
                (name.to_string(), size, time_str)
            })
            .collect()
    }
}

impl Default for SpeedTester {
    fn default() -> Self {
        Self::new().expect("Failed to create default SpeedTester")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_quality_classification() {
        assert_eq!(NetworkQuality::from_speed_mbps(0.5), NetworkQuality::VeryPoor);
        assert_eq!(NetworkQuality::from_speed_mbps(2.0), NetworkQuality::Poor);
        assert_eq!(NetworkQuality::from_speed_mbps(10.0), NetworkQuality::Average);
        assert_eq!(NetworkQuality::from_speed_mbps(35.0), NetworkQuality::Good);
        assert_eq!(NetworkQuality::from_speed_mbps(100.0), NetworkQuality::Excellent);
    }

    #[test]
    fn test_instant_model_sizes() {
        assert!(NetworkQuality::VeryPoor.instant_model_max_size_mb() <= 100);
        assert!(NetworkQuality::Excellent.instant_model_max_size_mb() >= 1000);
    }

    #[test]
    fn test_background_model_sizes() {
        assert!(NetworkQuality::VeryPoor.background_model_max_size_mb() >= 200);
        assert!(NetworkQuality::Excellent.background_model_max_size_mb() >= 10000);
    }

    #[test]
    fn test_speed_test_result_display() {
        let result = SpeedTestResult {
            speed_mbps: 25.5,
            test_duration_secs: 2.5,
            success: true,
            error: None,
            quality: NetworkQuality::Good,
        };

        let display = format!("{}", result);
        assert!(display.contains("25.50 MB/s"));
        assert!(display.contains("Good"));
    }

    #[test]
    fn test_download_time_estimation() {
        let result = SpeedTestResult {
            speed_mbps: 10.0,
            test_duration_secs: 1.0,
            success: true,
            error: None,
            quality: NetworkQuality::Average,
        };

        // 100 MB at 10 MB/s should take 10 seconds
        let time = result.estimate_download_time_secs(100);
        assert!((time - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_format_download_time() {
        let result = SpeedTestResult {
            speed_mbps: 10.0,
            test_duration_secs: 1.0,
            success: true,
            error: None,
            quality: NetworkQuality::Average,
        };

        // 30 seconds
        assert!(result.format_download_time(300).contains("seconds"));

        // 1.5 minutes
        assert!(result.format_download_time(900).contains("minutes"));
    }
}

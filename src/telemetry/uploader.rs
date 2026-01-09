//! Batch telemetry uploader

use super::{config::TelemetryConfig, storage::TelemetryStorage};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Telemetry batch uploader
///
/// Runs in the background, periodically uploading queued events to the
/// telemetry endpoint. Respects air-gapped mode by skipping uploads when enabled.
///
/// ## Upload Schedule
///
/// - Interval: Every 1 hour
/// - Batch size: Up to 100 events per upload
/// - Timeout: 30 seconds per request
/// - Retry: Failed uploads remain queued
///
/// ## Air-Gapped Mode
///
/// When `config.air_gapped` is true, the uploader is disabled and events
/// remain in local storage until manually exported via `caro telemetry export`.
pub struct TelemetryUploader {
    storage: Arc<TelemetryStorage>,
    config: TelemetryConfig,
}

impl TelemetryUploader {
    /// Create a new telemetry uploader
    pub fn new(storage: Arc<TelemetryStorage>, config: TelemetryConfig) -> Self {
        Self { storage, config }
    }

    /// Start background upload worker
    ///
    /// Spawns a tokio task that runs indefinitely, uploading events periodically.
    /// The task will continue running until the program exits.
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // 1 hour

            loop {
                ticker.tick().await;

                if !self.config.air_gapped && self.config.enabled {
                    if let Err(e) = self.upload_batch().await {
                        tracing::warn!("Telemetry upload failed: {}", e);
                        // Continue despite errors - events remain queued
                    }
                }
            }
        });
    }

    /// Upload a batch of events
    ///
    /// Fetches up to 100 pending events, uploads them to the configured endpoint,
    /// and deletes them from local storage on success.
    ///
    /// ## Error Handling
    ///
    /// Errors during upload are logged but don't crash the app. Events remain
    /// in the queue for the next upload attempt.
    ///
    /// ## Feature Gate
    ///
    /// Requires `remote-backends` feature for HTTP client support.
    /// Without this feature, events remain in local storage for manual export.
    async fn upload_batch(&self) -> Result<()> {
        #[cfg(not(feature = "remote-backends"))]
        {
            // Without reqwest, we can't upload - events stay in local storage
            tracing::debug!(
                "Telemetry upload skipped: reqwest not available (enable remote-backends feature)"
            );
            Ok(())
        }

        #[cfg(feature = "remote-backends")]
        {
            let events = self
                .storage
                .get_pending_events(100)
                .await
                .context("Failed to fetch pending events")?;

            if events.is_empty() {
                return Ok(());
            }

            tracing::debug!("Uploading {} telemetry events", events.len());

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .context("Failed to create HTTP client")?;

            let response = client
                .post(&self.config.endpoint)
                .json(&events)
                .send()
                .await
                .context("Failed to send telemetry events")?;

            if response.status().is_success() {
                // Delete successfully uploaded events
                let event_ids: Vec<String> = events.iter().map(|e| e.id.to_string()).collect();

                self.storage
                    .delete_events(&event_ids)
                    .await
                    .context("Failed to delete uploaded events")?;

                tracing::debug!("Successfully uploaded {} telemetry events", events.len());
            } else {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read response body".to_string());

                anyhow::bail!("Telemetry upload failed with status {}: {}", status, body);
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{Event, EventType, SessionId};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_uploader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        let config = TelemetryConfig {
            enabled: true,
            level: super::super::config::TelemetryLevel::Normal,
            air_gapped: false,
            endpoint: "https://test.example.com/api".to_string(),
            first_run: false,
        };

        let uploader = TelemetryUploader::new(storage, config);
        assert!(!uploader.config.air_gapped);
    }

    #[tokio::test]
    async fn test_upload_with_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        let config = TelemetryConfig {
            enabled: true,
            level: super::super::config::TelemetryLevel::Normal,
            air_gapped: false,
            endpoint: "https://test.example.com/api".to_string(),
            first_run: false,
        };

        let uploader = TelemetryUploader::new(storage, config);

        // Should succeed with empty queue
        let result = uploader.upload_batch().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_air_gapped_mode_skips_upload() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        // Store an event
        let session_id = SessionId::generate();
        let event = Event::new(
            session_id,
            EventType::SessionStart {
                version: "1.0.0".to_string(),
                platform: "linux".to_string(),
                shell_type: "bash".to_string(),
                backend_available: vec![],
            },
        );
        storage.store_event(&event).await.unwrap();

        let config = TelemetryConfig {
            enabled: true,
            level: super::super::config::TelemetryLevel::Normal,
            air_gapped: true, // Air-gapped mode
            endpoint: "https://test.example.com/api".to_string(),
            first_run: false,
        };

        let uploader = Arc::new(TelemetryUploader::new(storage.clone(), config));

        // Start uploader (it should NOT upload in air-gapped mode)
        uploader.start();

        // Give it time to potentially upload (it shouldn't)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Event should still be in queue
        let events = storage.get_pending_events(10).await.unwrap();
        assert_eq!(events.len(), 1);
    }
}

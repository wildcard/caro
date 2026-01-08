//! Non-blocking telemetry event collector

use super::{Event, EventType, SessionId, TelemetryStorage};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Non-blocking telemetry collector
///
/// The collector provides a fire-and-forget interface for emitting telemetry
/// events. Events are queued in memory and processed asynchronously in a
/// background task, ensuring zero impact on main thread performance.
///
/// ## Performance Guarantee
///
/// - Event emission is O(1) and never blocks
/// - Target overhead: <5ms on startup
/// - Events processed asynchronously in background
///
/// ## Usage
///
/// ```no_run
/// use caro::telemetry::{TelemetryCollector, TelemetryStorage, EventType};
/// use std::sync::Arc;
///
/// # async fn example() {
/// let storage = Arc::new(TelemetryStorage::new("events.db".into()).unwrap());
/// let collector = TelemetryCollector::new(storage, true);
///
/// // Fire and forget - never blocks
/// collector.emit(EventType::SessionStart {
///     version: "1.0.0".to_string(),
///     platform: "linux".to_string(),
///     shell_type: "bash".to_string(),
///     backend_available: vec!["embedded".to_string()],
/// });
/// # }
/// ```
pub struct TelemetryCollector {
    tx: mpsc::UnboundedSender<Event>,
    session_id: SessionId,
    enabled: bool,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    ///
    /// # Arguments
    ///
    /// * `storage` - Shared storage backend for persisting events
    /// * `enabled` - Whether telemetry collection is enabled
    ///
    /// If enabled, spawns a background task to process events asynchronously.
    pub fn new(storage: Arc<TelemetryStorage>, enabled: bool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let session_id = SessionId::generate();

        if enabled {
            tokio::spawn(Self::event_loop(storage, rx));
        }

        Self {
            tx,
            session_id,
            enabled,
        }
    }

    /// Emit a telemetry event (non-blocking)
    ///
    /// This method is guaranteed to be non-blocking and will never fail.
    /// Events are queued in memory and processed asynchronously.
    ///
    /// If telemetry is disabled, this is a no-op with zero overhead.
    pub fn emit(&self, event_type: EventType) {
        if !self.enabled {
            return;
        }

        let event = Event::new(self.session_id.clone(), event_type);

        // Fire and forget - never block main thread
        // If channel is closed (shouldn't happen), silently ignore
        let _ = self.tx.send(event);
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Background event processing loop
    ///
    /// Runs in a separate task, consuming events from the channel and
    /// persisting them to storage. Errors are logged but don't crash the app.
    async fn event_loop(storage: Arc<TelemetryStorage>, mut rx: mpsc::UnboundedReceiver<Event>) {
        while let Some(event) = rx.recv().await {
            // Store event asynchronously
            if let Err(e) = storage.store_event(&event).await {
                tracing::warn!("Failed to store telemetry event: {}", e);
                // Continue processing despite errors
            }
        }
    }
}

impl Clone for TelemetryCollector {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            session_id: self.session_id.clone(),
            enabled: self.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_collector_when_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        let collector = TelemetryCollector::new(storage.clone(), false);

        // Should not crash or block
        collector.emit(EventType::SessionStart {
            version: "1.0.0".to_string(),
            platform: "linux".to_string(),
            shell_type: "bash".to_string(),
            backend_available: vec![],
        });

        // Give time for any potential async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // No events should be stored
        let events = storage.get_pending_events(10).await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_collector_when_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        let collector = TelemetryCollector::new(storage.clone(), true);

        collector.emit(EventType::SessionStart {
            version: "1.0.0".to_string(),
            platform: "linux".to_string(),
            shell_type: "bash".to_string(),
            backend_available: vec!["embedded".to_string()],
        });

        // Give time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Event should be stored
        let events = storage.get_pending_events(10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].event_type,
            EventType::SessionStart { .. }
        ));
    }

    #[tokio::test]
    async fn test_collector_clone() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(TelemetryStorage::new(db_path).unwrap());

        let collector1 = TelemetryCollector::new(storage, true);
        let collector2 = collector1.clone();

        assert_eq!(collector1.session_id(), collector2.session_id());
        assert_eq!(collector1.is_enabled(), collector2.is_enabled());
    }
}

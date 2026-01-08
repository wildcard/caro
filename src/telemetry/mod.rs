//! Privacy-first telemetry collection
//!
//! ## Privacy Guarantees
//!
//! - Never collects command content or natural language input
//! - Never collects file paths or environment variables
//! - Anonymous session IDs (rotate daily)
//! - All data redacted before storage
//! - User-controlled opt-out
//!
//! ## Architecture
//!
//! 1. Events emitted from various components
//! 2. Async collector queues to SQLite
//! 3. Batch worker uploads periodically
//! 4. Air-gapped mode: local storage only
//!
//! ## Usage
//!
//! ```no_run
//! use caro::telemetry::{TelemetryCollector, EventType};
//!
//! let collector = TelemetryCollector::new(storage, true);
//!
//! collector.emit(EventType::SessionStart {
//!     version: env!("CARGO_PKG_VERSION").to_string(),
//!     platform: std::env::consts::OS.to_string(),
//!     shell_type: "bash".to_string(),
//!     backend_available: vec!["embedded".to_string()],
//! });
//! ```

pub mod collector;
pub mod config;
pub mod consent;
pub mod events;
pub mod redaction;
pub mod storage;
pub mod uploader;

pub use collector::TelemetryCollector;
pub use config::TelemetryConfig;
pub use events::{Event, EventType, SessionId};
pub use storage::TelemetryStorage;

use std::sync::{Arc, OnceLock};

/// Global telemetry collector for convenient event emission
static GLOBAL_COLLECTOR: OnceLock<Arc<TelemetryCollector>> = OnceLock::new();

/// Set the global telemetry collector (call once at startup)
///
/// This allows components to emit events without explicitly passing
/// the collector through function signatures.
///
/// # Example
///
/// ```no_run
/// use caro::telemetry::{TelemetryCollector, set_global_collector};
/// use std::sync::Arc;
///
/// let collector = Arc::new(TelemetryCollector::new(storage, true));
/// set_global_collector(collector);
/// ```
pub fn set_global_collector(collector: Arc<TelemetryCollector>) {
    let _ = GLOBAL_COLLECTOR.set(collector);
}

/// Emit a telemetry event using the global collector
///
/// If no global collector has been set, this is a no-op.
/// Components can safely call this without checking if telemetry is enabled.
///
/// # Example
///
/// ```no_run
/// use caro::telemetry::{emit_event, EventType};
///
/// emit_event(EventType::CommandGeneration {
///     backend: "static".to_string(),
///     duration_ms: 150,
///     success: true,
///     error_category: None,
/// });
/// ```
pub fn emit_event(event_type: EventType) {
    if let Some(collector) = GLOBAL_COLLECTOR.get() {
        collector.emit(event_type);
    }
}

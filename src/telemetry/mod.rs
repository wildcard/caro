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

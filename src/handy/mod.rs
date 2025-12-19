//! Handy.Computer Integration
//!
//! This module provides integration with [Handy.Computer](https://handy.computer),
//! an open-source push-to-talk speech-to-text application by CJ Pais (@cjpais).
//!
//! # Features
//!
//! - **Detection**: Automatically detect if Handy is installed and running
//! - **Configuration**: Control integration behavior via config file
//! - **UI Integration**: Display Handy status and hints in the CLI
//!
//! # Attribution
//!
//! Handy.Computer by CJ Pais (https://github.com/cjpais/Handy)
//! - MIT License
//! - Privacy-first, offline speech-to-text
//! - Cross-platform support (macOS, Linux, Windows)
//!
//! # Example
//!
//! ```no_run
//! use cmdai::handy::{get_handy_status, HandyStatus};
//!
//! match get_handy_status() {
//!     HandyStatus::InstalledAndRunning { pid, .. } => {
//!         println!("Handy is running (PID: {})", pid);
//!     }
//!     HandyStatus::InstalledNotRunning { .. } => {
//!         println!("Handy is installed but not running");
//!     }
//!     HandyStatus::NotInstalled => {
//!         println!("Handy is not installed");
//!     }
//! }
//! ```

pub mod config;
pub mod detection;
pub mod ui;

// Re-export commonly used types
pub use config::HandyConfig;
pub use detection::{get_handy_status, is_handy_available, is_handy_installed, HandyDetector, HandyStatus};
pub use ui::{format_handy_hint, format_handy_status_line, show_handy_welcome};

/// Handy.Computer attribution information
pub const HANDY_ATTRIBUTION: &str = "Voice input integration powered by Handy.Computer by CJ Pais\nhttps://github.com/cjpais/Handy";

/// Handy.Computer website URL
pub const HANDY_WEBSITE: &str = "https://handy.computer";

/// Handy.Computer GitHub repository
pub const HANDY_GITHUB: &str = "https://github.com/cjpais/Handy";

/// Developer information
pub const HANDY_DEVELOPER: &str = "CJ Pais (@cjpais)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(HANDY_ATTRIBUTION.contains("CJ Pais"));
        assert_eq!(HANDY_WEBSITE, "https://handy.computer");
        assert_eq!(HANDY_GITHUB, "https://github.com/cjpais/Handy");
        assert_eq!(HANDY_DEVELOPER, "CJ Pais (@cjpais)");
    }
}

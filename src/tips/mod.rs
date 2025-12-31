//! Tips module for intelligent command suggestions
//!
//! Provides contextual "Did you know?" tips and alias suggestions based on
//! shell configuration, installed plugins, and community knowledge base.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Tips Module                       │
//! ├─────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────┐    │
//! │  │           Shell Intelligence                 │    │
//! │  │  ┌──────────┐ ┌──────────┐ ┌────────────┐   │    │
//! │  │  │ Detector │ │  Alias   │ │  Plugin    │   │    │
//! │  │  │          │ │  Parser  │ │  Detector  │   │    │
//! │  │  └──────────┘ └──────────┘ └────────────┘   │    │
//! │  └─────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Modules
//!
//! - [`shell`] - Shell detection, alias parsing, plugin detection
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::shell::ShellIntelligence;
//!
//! if let Some(intel) = ShellIntelligence::detect() {
//!     println!("Detected shell: {}", intel.shell_type());
//!
//!     // Find shorter alias for a command
//!     let aliases = intel.find_shorter_aliases("git status");
//!     for alias in aliases {
//!         println!("Did you know? Use `{}` instead of `{}`",
//!             alias.name, alias.expansion);
//!     }
//! }
//! ```

pub mod shell;

// Re-export commonly used types
pub use shell::{Alias, AliasParser, AliasSource, PluginDetector, PluginManager, ShellIntelligence, TipsShellType};

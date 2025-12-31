//! Automation module for plugin installation and configuration
//!
//! Provides safe installation of shell plugins with backup/rollback support.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Automation Module                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌───────────┐  ┌───────────────────┐    │
//! │  │   Installer  │  │  Config   │  │      Plans        │    │
//! │  │  Execute     │  │  Editor   │  │   Installation    │    │
//! │  │  Rollback    │  │  Backup   │  │   Definitions     │    │
//! │  └──────────────┘  └───────────┘  └───────────────────┘    │
//! │         │                │                │                  │
//! │         └────────────────┼────────────────┘                  │
//! │                          ▼                                   │
//! │                  ┌───────────────┐                           │
//! │                  │ Shell Reload  │                           │
//! │                  │ Verification  │                           │
//! │                  └───────────────┘                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use caro::tips::automation::{Installer, InstallationPlan, plugin_enable_plan};
//!
//! let plan = plugin_enable_plan("git", "Git aliases and functions");
//! let installer = Installer::new().unwrap();
//! installer.execute_dry_run(&plan);
//! ```

pub mod config_editor;
pub mod installer;
pub mod plans;
pub mod shell_reload;
pub mod types;

pub use config_editor::ConfigEditor;
pub use installer::{InstallResult, Installer};
pub use plans::{ohmyzsh_install_plan, plugin_enable_plan};
pub use shell_reload::ShellReload;
pub use types::{
    InstallStep, InstallationError, InstallationPlan, Prerequisite, RollbackPlan, VerificationStep,
};

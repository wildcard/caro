//! Paperclip AI integration for Caro.
//!
//! This module allows Caro to operate as a managed agent within a
//! [Paperclip](https://github.com/paperclipai/paperclip) orchestration
//! platform. Paperclip provides organizational structure, budget management,
//! governance, and audit trails for autonomous AI agent companies.
//!
//! # How it works
//!
//! 1. Paperclip triggers Caro with injected `PAPERCLIP_*` environment variables
//! 2. Caro detects these and enters **agent mode**
//! 3. Tasks are fetched from the Paperclip API, each containing a natural
//!    language description
//! 4. Caro converts each description to a safe shell command (using its
//!    existing backends and safety validation)
//! 5. Results are reported back to Paperclip for audit and governance

pub mod client;
pub mod config;
pub mod runner;

pub use client::{BudgetStatus, PaperclipClient, Task, TaskResult, TaskStatus};
pub use config::PaperclipConfig;
pub use runner::{HeartbeatSummary, PaperclipRunner};

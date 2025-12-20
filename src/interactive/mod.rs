//! Interactive mode for Caro - TUI interface similar to Claude Code
//!
//! This module provides a terminal user interface (TUI) for interactive sessions
//! with Caro, including a welcome screen, tips, recent activity, and REPL-like
//! command input.

mod app;
mod ui;

pub use app::{InteractiveApp, InteractiveConfig};
pub use ui::run_interactive;

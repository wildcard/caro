//! TUI component examples for the showcase
//!
//! This module contains example components that demonstrate how to build
//! isolated, testable terminal UI components.

pub mod simple_text;
pub mod command_preview;
pub mod safety_indicator;
pub mod confirmation_dialog;
pub mod progress_spinner;

// Re-export components for easier access
pub use simple_text::SimpleTextComponent;
pub use command_preview::CommandPreviewComponent;
pub use safety_indicator::SafetyIndicatorComponent;
pub use confirmation_dialog::ConfirmationDialogComponent;
pub use progress_spinner::ProgressSpinnerComponent;

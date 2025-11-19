//! TUI component examples for the showcase
//!
//! This module contains example components that demonstrate how to build
//! isolated, testable terminal UI components.
//!
//! ## Component Categories
//!
//! - **Display**: SimpleText, CommandPreview, TableSelector
//! - **Input**: ConfirmationDialog, CommandEditor
//! - **Feedback**: SafetyIndicator, ProgressSpinner, NotificationToast
//! - **Workflow**: CommandFlow
//! - **Help**: KeyboardShortcuts

// Basic display components
pub mod simple_text;
pub mod command_preview;
pub mod table_selector;

// Input components
pub mod confirmation_dialog;
pub mod command_editor;

// Feedback components
pub mod safety_indicator;
pub mod progress_spinner;
pub mod notification_toast;

// Workflow components
pub mod command_flow;

// Help components
pub mod keyboard_shortcuts;

// Re-export components for easier access
pub use simple_text::SimpleTextComponent;
pub use command_preview::CommandPreviewComponent;
pub use table_selector::TableSelectorComponent;
pub use confirmation_dialog::ConfirmationDialogComponent;
pub use command_editor::CommandEditorComponent;
pub use safety_indicator::SafetyIndicatorComponent;
pub use progress_spinner::ProgressSpinnerComponent;
pub use notification_toast::NotificationToastComponent;
pub use command_flow::CommandFlowComponent;
pub use keyboard_shortcuts::KeyboardShortcutsComponent;

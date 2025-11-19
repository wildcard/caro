//! TUI component examples for the showcase
//!
//! This module contains example components that demonstrate how to build
//! isolated, testable terminal UI components.
//!
//! ## Component Categories
//!
//! - **Display**: SimpleText, CommandPreview, TableSelector, CommandOutputViewer,
//!               HistoryTimeline, GenerationComparison, MetricDashboard
//! - **Input**: ConfirmationDialog, CommandEditor, CommandRating
//! - **Feedback**: SafetyIndicator, ProgressSpinner, NotificationToast
//! - **Workflow**: CommandFlow
//! - **Help**: KeyboardShortcuts

// Basic display components
pub mod command_output_viewer;
pub mod command_preview;
pub mod generation_comparison;
pub mod history_timeline;
pub mod metric_dashboard;
pub mod simple_text;
pub mod table_selector;

// Input components
pub mod command_editor;
pub mod command_rating;
pub mod confirmation_dialog;

// Feedback components
pub mod notification_toast;
pub mod progress_spinner;
pub mod safety_indicator;

// Workflow components
pub mod command_flow;

// Help components
pub mod keyboard_shortcuts;

// File system components
pub mod file_browser;

// Re-export components for easier access
pub use command_editor::CommandEditorComponent;
pub use command_flow::CommandFlowComponent;
pub use command_output_viewer::CommandOutputViewerComponent;
pub use command_preview::CommandPreviewComponent;
pub use command_rating::CommandRatingComponent;
pub use confirmation_dialog::ConfirmationDialogComponent;
pub use file_browser::FileBrowserComponent;
pub use generation_comparison::GenerationComparisonComponent;
pub use history_timeline::HistoryTimelineComponent;
pub use keyboard_shortcuts::KeyboardShortcutsComponent;
pub use metric_dashboard::MetricDashboardComponent;
pub use notification_toast::NotificationToastComponent;
pub use progress_spinner::ProgressSpinnerComponent;
pub use safety_indicator::SafetyIndicatorComponent;
pub use simple_text::SimpleTextComponent;
pub use table_selector::TableSelectorComponent;

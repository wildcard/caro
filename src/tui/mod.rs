/// Terminal User Interface (TUI) Module
///
/// This module implements the interactive terminal UI for cmdai using Ratatui.
///
/// # Architecture
///
/// The TUI follows a component-based architecture with Redux-inspired state management:
///
/// ```text
/// ┌─────────────────────────────────────────────────────────┐
/// │                        TuiApp                            │
/// │  - Event loop (async)                                    │
/// │  - State management (AppState)                           │
/// │  - Component coordination                                │
/// └──────────────┬──────────────────────────────────────────┘
///                │
///       ┌────────┴────────┐
///       │                 │
///  ┌────▼─────┐    ┌─────▼──────┐
///  │ Terminal │    │   Events   │
///  │ Renderer │    │  Handler   │
///  └────┬─────┘    └─────┬──────┘
///       │                │
///  ┌────▼────────────────▼─────┐
///  │     Component Tree        │
///  │  - StatusBarComponent     │
///  │  - ReplComponent          │
///  │  - HelpFooterComponent    │
///  └───────────────────────────┘
/// ```
///
/// # Usage
///
/// ```rust
/// use cmdai::tui::TuiApp;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let app = TuiApp::new()?;
///     app.run().await?;
///     Ok(())
/// }
/// ```
///
/// # Modules
///
/// - `app` - Main TUI application and event loop
/// - `components` - Reusable UI components
/// - `state` - State management (AppState, events)
/// - `backend` - Integration with CliApp backend
/// - `theme` - Colors and styling
/// - `utils` - Terminal utilities

pub mod app;
pub mod components;
pub mod state;
pub mod theme;
pub mod utils;

// Re-export main types for convenience
pub use app::TuiApp;
pub use components::{Component, StatusBarComponent, HelpFooterComponent, ReplComponent};
pub use state::{AppState, AppEvent, AppMode};

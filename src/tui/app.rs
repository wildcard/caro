/// Main TUI Application
///
/// The `TuiApp` struct manages the entire TUI lifecycle:
/// - Event loop (keyboard input, terminal resize)
/// - State management (AppState updates)
/// - Component rendering
/// - Integration with backend (CliApp)
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::time::Duration;

use crate::config::ConfigManager;
use crate::tui::{
    components::{Component, HelpFooterComponent, ReplComponent, StatusBarComponent},
    state::{AppEvent, AppMode, AppState},
    utils::{restore_terminal, setup_terminal, TerminalType},
};

/// Main TUI Application
///
/// # Example
///
/// ```rust
/// use cmdai::tui::TuiApp;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut app = TuiApp::new()?;
///     app.run().await?;
///     Ok(())
/// }
/// ```
pub struct TuiApp {
    /// Application state
    state: AppState,

    /// Terminal instance
    terminal: TerminalType,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Result<Self> {
        // Load configuration
        let config_manager = ConfigManager::new()?;
        let user_config = config_manager.load()?;

        // Setup terminal
        let terminal = setup_terminal()?;

        // Create application state
        let mut state = AppState::new(user_config);

        // Set backend status (for now, just show "Loading...")
        state.set_backend_status("Loading...".to_string(), false, None);

        Ok(Self { state, terminal })
    }

    /// Run the TUI application
    ///
    /// This is the main event loop. It:
    /// 1. Renders the current state
    /// 2. Polls for events (keyboard, resize, etc.)
    /// 3. Updates state based on events
    /// 4. Handles side effects (async operations)
    /// 5. Repeats until quit
    pub async fn run(mut self) -> Result<()> {
        // Setup panic handler to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            original_hook(panic);
        }));

        // Initialize backend status
        self.detect_backend().await;

        // Main event loop
        while !self.state.should_quit {
            // Render current state
            self.render()?;

            // Poll for events with timeout
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        let app_event: AppEvent = key.into();
                        self.handle_event(app_event).await?;
                    }
                    Event::Resize(w, h) => {
                        self.handle_event(AppEvent::Resize(w, h)).await?;
                    }
                    _ => {}
                }
            }
        }

        // Cleanup
        restore_terminal(&mut self.terminal)?;

        Ok(())
    }

    /// Handle an application event
    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        // Update state and get side effects
        let side_effects = self.state.handle_event(event)?;

        // Process side effects
        for effect in side_effects {
            self.handle_side_effect(effect).await?;
        }

        Ok(())
    }

    /// Handle a side effect (async operations)
    async fn handle_side_effect(&mut self, _effect: crate::tui::state::SideEffect) -> Result<()> {
        // TODO: Implement side effect handling
        // This will involve calling the backend (CliApp) to generate commands,
        // validate them, etc.
        //
        // For Phase 1 MVP, we'll implement basic side effects.
        // Full implementation will come in subsequent commits.

        Ok(())
    }

    /// Detect available backend
    async fn detect_backend(&mut self) {
        // TODO: Actually detect backend
        // For now, just set a placeholder
        self.state
            .set_backend_status("Mock".to_string(), true, Some("Phase 1 MVP".to_string()));
    }

    /// Render the current state to the terminal
    fn render(&mut self) -> Result<()> {
        // Clone the state we need for rendering (to avoid borrow conflicts)
        let state = self.state.clone();

        self.terminal.draw(|frame| {
            Self::render_frame(frame, &state);
        })?;
        Ok(())
    }

    /// Render a single frame (static method to avoid borrow conflicts)
    fn render_frame(frame: &mut Frame, state: &AppState) {
        let size = frame.area();

        // Main layout: Status bar (1 line), Content (flex), Help footer (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status bar
                Constraint::Min(10),   // Main content
                Constraint::Length(1), // Help footer
            ])
            .split(size);

        // Render status bar
        let status_bar = StatusBarComponent::from_state(state);
        status_bar.render(frame, chunks[0]);

        // Render main content based on mode
        match state.current_mode {
            AppMode::Repl => {
                let repl = ReplComponent::from_state(state);
                // Note: We need to pass repl_state to the render function
                // For now, ReplComponent will use default state
                repl.render(frame, chunks[1]);
            }
            AppMode::History => {
                // TODO: Implement history mode
                Self::render_placeholder(frame, chunks[1], "History Mode (Coming Soon)");
            }
            AppMode::Config => {
                // TODO: Implement config mode
                Self::render_placeholder(frame, chunks[1], "Config Mode (Coming Soon)");
            }
            AppMode::Help => {
                // TODO: Implement help mode
                Self::render_placeholder(frame, chunks[1], "Help Mode (Coming Soon)");
            }
        }

        // Render help footer
        let help_footer = HelpFooterComponent::for_mode(state.current_mode);
        help_footer.render(frame, chunks[2]);
    }

    /// Render a placeholder for unimplemented modes (static method)
    fn render_placeholder(frame: &mut Frame, area: ratatui::layout::Rect, text: &str) {
        use ratatui::{
            style::{Color, Style},
            text::Text,
            widgets::{Block, Borders, Paragraph},
        };

        let paragraph = Paragraph::new(Text::from(text))
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("TODO"));

        frame.render_widget(paragraph, area);
    }
}

// Import for panic handler
use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use std::io;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserConfiguration;

    #[test]
    fn test_app_state_initialization() {
        let config = UserConfiguration::default();
        let state = AppState::new(config);

        assert_eq!(state.current_mode, AppMode::Repl);
        assert!(!state.should_quit);
    }

    // Note: Full integration tests require a terminal, so they're limited here
}

/// REPL Component Module
///
/// The REPL (Read-Eval-Print-Loop) component is the main interactive interface
/// for cmdai TUI. It allows users to type natural language commands and see
/// generated shell commands with live validation.
///
/// # Architecture
///
/// The REPL component is composed of three sub-components:
/// - `InputArea` - Text input for natural language
/// - `ValidationPanel` - Live safety validation feedback
/// - `CommandPreviewPanel` - Generated command with explanation
use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::components::{Component, EventResult};
use crate::tui::state::{AppState, ReplState};

/// REPL component props
#[derive(Debug, Clone)]
pub struct ReplProps {
    /// Whether the component has focus
    pub focused: bool,
}

/// REPL component state
#[derive(Debug, Clone, Default)]
pub struct ReplComponentState {
    // Internal state if needed
}

/// Main REPL component
///
/// # Example
///
/// ```rust
/// let component = ReplComponent::new(ReplProps { focused: true });
/// component.render(&mut frame, content_area);
/// ```
pub struct ReplComponent {
    props: ReplProps,
    state: ReplComponentState,
}

impl ReplComponent {
    /// Create from application state
    pub fn from_state(_app_state: &AppState) -> Self {
        Self::new(ReplProps { focused: true })
    }

    /// Render input area
    fn render_input_area(&self, frame: &mut Frame, area: Rect, repl_state: &ReplState) {
        let placeholder = if repl_state.has_input() {
            ""
        } else {
            "🤖 Type your command in natural language..."
        };

        let text = if repl_state.has_input() {
            repl_state.input()
        } else {
            placeholder
        };

        let style = if repl_state.has_input() {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_widget = Paragraph::new(text)
            .style(style)
            .block(Block::default().borders(Borders::ALL).title("Input"));

        frame.render_widget(input_widget, area);

        // Render cursor if focused
        if self.props.focused && repl_state.has_input() {
            let cursor_x = area.x + repl_state.cursor_position as u16 + 1;
            let cursor_y = area.y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    /// Render validation panel
    fn render_validation_panel(&self, frame: &mut Frame, area: Rect, repl_state: &ReplState) {
        let text = if let Some(ref validation) = repl_state.validation_result {
            let icon = validation.risk_level.icon();
            let name = validation.risk_level.name();
            format!("{} {}", icon, name)
        } else if repl_state.validating {
            "⏳ Validating...".to_string()
        } else {
            "Ready".to_string()
        };

        let validation_widget = Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Validation"));

        frame.render_widget(validation_widget, area);
    }

    /// Render command preview panel
    fn render_command_preview(&self, frame: &mut Frame, area: Rect, repl_state: &ReplState) {
        let text = if let Some(ref cmd) = repl_state.generated_command {
            format!("{}\n\n💡 {}", cmd.command, cmd.explanation)
        } else if repl_state.generating {
            "⏳ Generating command...".to_string()
        } else if let Some(ref err) = repl_state.generation_error {
            format!("❌ Error: {}", err)
        } else {
            "Start typing to generate a command...".to_string()
        };

        let style = if repl_state.generated_command.is_some() {
            Style::default().fg(Color::White)
        } else if repl_state.generation_error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let preview_widget = Paragraph::new(text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Generated Command"),
        );

        frame.render_widget(preview_widget, area);
    }
}

impl Component for ReplComponent {
    type Props = ReplProps;
    type State = ReplComponentState;

    fn new(props: Self::Props) -> Self {
        Self {
            props,
            state: ReplComponentState::default(),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<EventResult> {
        match event {
            Event::Key(_) => {
                // All key events are handled by AppState, pass them through
                Ok(EventResult::Ignored)
            }
            _ => Ok(EventResult::Ignored),
        }
    }

    fn update(&mut self, _state: &AppState) -> Result<()> {
        Ok(())
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Get repl state from somewhere - for now use default
        // TODO: Pass repl_state properly when rendering
        let repl_state = ReplState::default();

        // Layout: Input (4 lines), Validation (3 lines), Preview (rest)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Input area
                Constraint::Length(3), // Validation panel
                Constraint::Min(5),    // Command preview
            ])
            .split(area);

        self.render_input_area(frame, chunks[0], &repl_state);
        self.render_validation_panel(frame, chunks[1], &repl_state);
        self.render_command_preview(frame, chunks[2], &repl_state);
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn test_repl_component_creation() {
        let component = ReplComponent::new(ReplProps { focused: true });

        assert!(component.props.focused);
    }

    #[test]
    fn test_repl_component_event_handling() {
        let mut component = ReplComponent::new(ReplProps { focused: true });

        let result = component
            .handle_event(Event::Key(KeyEvent::from(KeyCode::Char('a'))))
            .unwrap();

        // Events should be passed through to AppState
        assert_eq!(result, EventResult::Ignored);
    }
}

/// Status Bar Component
///
/// Displays current TUI state and configuration at the top of the screen.
///
/// Visual layout:
/// ```text
/// ⚙ Ollama • bash • Moderate Safety                          [?] Help
/// └─┬──┘   └─┬─┘   └───────┬──────┘                          └───┬──┘
///   │        │             │                                      │
/// Backend  Shell    Safety Level                          Help Indicator
/// ```
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::models::{SafetyLevel, ShellType};
use crate::tui::components::Component;
use crate::tui::state::AppState;

/// Status bar component props
#[derive(Debug, Clone)]
pub struct StatusBarProps {
    /// Backend name (e.g., "Ollama", "vLLM", "Embedded")
    pub backend_name: String,

    /// Whether the backend is currently available
    pub backend_available: bool,

    /// Optional model name
    pub backend_model: Option<String>,

    /// Current shell type
    pub shell: ShellType,

    /// Current safety level
    pub safety_level: SafetyLevel,

    /// Whether to show help indicator
    pub show_help: bool,
}

/// Status bar component
///
/// # Example
///
/// ```rust
/// let component = StatusBarComponent::new(StatusBarProps {
///     backend_name: "Ollama".to_string(),
///     backend_available: true,
///     backend_model: Some("qwen2.5-coder:7b".to_string()),
///     shell: ShellType::Bash,
///     safety_level: SafetyLevel::Moderate,
///     show_help: true,
/// });
///
/// component.render(&mut frame, status_bar_area);
/// ```
pub struct StatusBarComponent {
    props: StatusBarProps,
}

impl StatusBarComponent {
    /// Create from application state
    pub fn from_state(state: &AppState) -> Self {
        Self::new(StatusBarProps {
            backend_name: state.backend_status.name.clone(),
            backend_available: state.backend_status.available,
            backend_model: state.backend_status.model.clone(),
            shell: state.shell(), // Use the helper method that handles Option
            safety_level: state.config.safety_level,
            show_help: true,
        })
    }

    /// Get color for backend status
    fn backend_color(&self) -> Color {
        if self.props.backend_available {
            Color::Cyan
        } else {
            Color::Red
        }
    }

    /// Get color for safety level
    fn safety_color(&self) -> Color {
        match self.props.safety_level {
            SafetyLevel::Strict => Color::Red,
            SafetyLevel::Moderate => Color::Yellow,
            SafetyLevel::Permissive => Color::Green,
        }
    }

    /// Get safety level display text
    fn safety_text(&self) -> String {
        format!("{} Safety", self.props.safety_level)
    }
}

impl Component for StatusBarComponent {
    type Props = StatusBarProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let backend_color = self.backend_color();
        let safety_color = self.safety_color();

        // Left side: Backend • Shell • Safety
        let mut left_spans = vec![
            Span::styled("⚙ ", Style::default().fg(backend_color)),
            Span::styled(
                &self.props.backend_name,
                Style::default()
                    .fg(backend_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        // Add model name if available
        if let Some(ref model) = self.props.backend_model {
            left_spans.push(Span::styled(
                format!(" ({})", model),
                Style::default().fg(Color::DarkGray),
            ));
        }

        left_spans.extend_from_slice(&[
            Span::raw(" • "),
            Span::styled(
                self.props.shell.to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" • "),
            Span::styled(self.safety_text(), Style::default().fg(safety_color)),
        ]);

        let status_text = Line::from(left_spans);

        let paragraph = Paragraph::new(status_text)
            .style(Style::default().bg(Color::Black))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    fn state(&self) -> &Self::State {
        &()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_creation() {
        let component = StatusBarComponent::new(StatusBarProps {
            backend_name: "Ollama".to_string(),
            backend_available: true,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            show_help: true,
        });

        assert_eq!(component.props.backend_name, "Ollama");
        assert!(component.props.backend_available);
    }

    #[test]
    fn test_backend_color_available() {
        let component = StatusBarComponent::new(StatusBarProps {
            backend_name: "Ollama".to_string(),
            backend_available: true,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            show_help: false,
        });

        assert_eq!(component.backend_color(), Color::Cyan);
    }

    #[test]
    fn test_backend_color_unavailable() {
        let component = StatusBarComponent::new(StatusBarProps {
            backend_name: "Ollama".to_string(),
            backend_available: false,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            show_help: false,
        });

        assert_eq!(component.backend_color(), Color::Red);
    }

    #[test]
    fn test_safety_colors() {
        let strict = StatusBarComponent::new(StatusBarProps {
            backend_name: "Test".to_string(),
            backend_available: true,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Strict,
            show_help: false,
        });

        let moderate = StatusBarComponent::new(StatusBarProps {
            backend_name: "Test".to_string(),
            backend_available: true,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            show_help: false,
        });

        let permissive = StatusBarComponent::new(StatusBarProps {
            backend_name: "Test".to_string(),
            backend_available: true,
            backend_model: None,
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Permissive,
            show_help: false,
        });

        assert_eq!(strict.safety_color(), Color::Red);
        assert_eq!(moderate.safety_color(), Color::Yellow);
        assert_eq!(permissive.safety_color(), Color::Green);
    }
}

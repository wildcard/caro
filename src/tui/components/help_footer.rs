/// Help Footer Component
///
/// Displays context-sensitive keyboard shortcuts at the bottom of the screen.
///
/// Visual layout:
/// ```text
/// [Enter] Generate  [Ctrl+R] History  [Ctrl+C] Quit  [?] More Help
/// └──┬──┘ └───┬───┘  └──┬───┘ └──┬──┘  └──┬───┘ └─┬─┘  └─┬─┘ └───┬───┘
///   Key  Description   Key  Description  Key  Description Key  Description
/// ```

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::components::Component;
use crate::tui::state::AppMode;

/// A single keyboard shortcut
#[derive(Debug, Clone)]
pub struct Shortcut {
    /// The key(s) to press (e.g., "Enter", "Ctrl+C")
    pub key: String,

    /// Description of what the key does
    pub description: String,

    /// Whether this shortcut is currently enabled
    pub enabled: bool,
}

impl Shortcut {
    /// Create a new enabled shortcut
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            enabled: true,
        }
    }

    /// Create a new disabled shortcut
    #[allow(dead_code)]
    pub fn disabled(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            enabled: false,
        }
    }
}

/// Help footer component props
#[derive(Debug, Clone)]
pub struct HelpFooterProps {
    /// List of shortcuts to display
    pub shortcuts: Vec<Shortcut>,
}

/// Help footer component
///
/// # Example
///
/// ```rust
/// let component = HelpFooterComponent::for_mode(AppMode::Repl);
/// component.render(&mut frame, footer_area);
/// ```
pub struct HelpFooterComponent {
    props: HelpFooterProps,
}

impl HelpFooterComponent {
    /// Create help footer for a specific mode
    pub fn for_mode(mode: AppMode) -> Self {
        let shortcuts = match mode {
            AppMode::Repl => vec![
                Shortcut::new("Enter", "Generate"),
                Shortcut::new("Ctrl+L", "Clear"),
                Shortcut::new("Ctrl+C", "Quit"),
                Shortcut::new("?", "Help"),
            ],
            AppMode::History => vec![
                Shortcut::new("↑↓", "Navigate"),
                Shortcut::new("Enter", "Copy"),
                Shortcut::new("Esc", "Back"),
                Shortcut::new("/", "Search"),
            ],
            AppMode::Config => vec![
                Shortcut::new("↑↓", "Navigate"),
                Shortcut::new("Enter", "Edit"),
                Shortcut::new("S", "Save"),
                Shortcut::new("Esc", "Cancel"),
            ],
            AppMode::Help => vec![
                Shortcut::new("↑↓", "Scroll"),
                Shortcut::new("Esc", "Close"),
                Shortcut::new("Q", "Quit"),
            ],
        };

        Self::new(HelpFooterProps { shortcuts })
    }

    /// Render a single shortcut as spans
    fn render_shortcut<'a>(&self, shortcut: &'a Shortcut) -> Vec<Span<'a>> {
        let key_color = if shortcut.enabled {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let desc_color = if shortcut.enabled {
            Color::White
        } else {
            Color::DarkGray
        };

        vec![
            Span::styled(
                "[",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                shortcut.key.as_str(),
                Style::default()
                    .fg(key_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "] ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                shortcut.description.as_str(),
                Style::default().fg(desc_color),
            ),
        ]
    }
}

impl Component for HelpFooterComponent {
    type Props = HelpFooterProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        for (i, shortcut) in self.props.shortcuts.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }
            spans.extend(self.render_shortcut(shortcut));
        }

        let help_line = Line::from(spans);

        let paragraph = Paragraph::new(help_line)
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
    fn test_shortcut_creation() {
        let shortcut = Shortcut::new("Enter", "Execute");

        assert_eq!(shortcut.key, "Enter");
        assert_eq!(shortcut.description, "Execute");
        assert!(shortcut.enabled);
    }

    #[test]
    fn test_disabled_shortcut() {
        let shortcut = Shortcut::disabled("Tab", "Complete");

        assert_eq!(shortcut.key, "Tab");
        assert!(!shortcut.enabled);
    }

    #[test]
    fn test_help_footer_for_repl_mode() {
        let component = HelpFooterComponent::for_mode(AppMode::Repl);

        assert!(!component.props.shortcuts.is_empty());
        assert_eq!(component.props.shortcuts[0].key, "Enter");
        assert_eq!(component.props.shortcuts[0].description, "Generate");
    }

    #[test]
    fn test_help_footer_for_history_mode() {
        let component = HelpFooterComponent::for_mode(AppMode::History);

        assert!(!component.props.shortcuts.is_empty());
        assert_eq!(component.props.shortcuts[0].key, "↑↓");
    }

    #[test]
    fn test_help_footer_has_quit() {
        let component = HelpFooterComponent::for_mode(AppMode::Repl);

        let has_quit = component
            .props
            .shortcuts
            .iter()
            .any(|s| s.key.contains("Ctrl+C") || s.key.contains("Quit"));

        assert!(has_quit, "Help footer should show quit shortcut");
    }
}

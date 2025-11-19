//! Simple text display component example

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SimpleTextComponent;

impl ShowcaseComponent for SimpleTextComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "SimpleText",
            "Basic text display with various styling options",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Default",
                "Simple centered text with a border",
                |frame, area| {
                    let block = Block::default().borders(Borders::ALL).title("Simple Text");
                    let paragraph = Paragraph::new("Hello, Ratatui Showcase!")
                        .block(block)
                        .alignment(Alignment::Center);
                    frame.render_widget(paragraph, area);
                },
            ),
            ShowcaseStory::new("Styled", "Text with colors and modifiers", |frame, area| {
                let text = vec![
                    Line::from(vec![
                        Span::styled("Bold", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                        Span::styled("Italic", Style::default().add_modifier(Modifier::ITALIC)),
                        Span::raw(" "),
                        Span::styled(
                            "Underline",
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Red", Style::default().fg(Color::Red)),
                        Span::raw(" "),
                        Span::styled("Green", Style::default().fg(Color::Green)),
                        Span::raw(" "),
                        Span::styled("Blue", Style::default().fg(Color::Blue)),
                    ]),
                ];
                let block = Block::default().borders(Borders::ALL).title("Styled Text");
                let paragraph = Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, area);
            }),
            ShowcaseStory::new(
                "MultiLine",
                "Multiple lines with different alignments",
                |frame, area| {
                    let text = vec![
                        Line::from("This is left-aligned"),
                        Line::from(""),
                        Line::from("This text demonstrates"),
                        Line::from("multi-line rendering"),
                        Line::from("with the Paragraph widget"),
                    ];
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Multi-line Text")
                        .style(Style::default().fg(Color::Cyan));
                    let paragraph = Paragraph::new(text).block(block);
                    frame.render_widget(paragraph, area);
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_metadata_has_correct_name() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.name, "SimpleText");
    }

    #[test]
    fn test_metadata_has_correct_description() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();
        assert_eq!(
            metadata.description,
            "Basic text display with various styling options"
        );
    }

    #[test]
    fn test_metadata_has_correct_category() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.category, "Display");
    }

    #[test]
    fn test_metadata_has_correct_version() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_component_has_three_stories() {
        let component = SimpleTextComponent;
        let stories = component.stories();
        assert_eq!(
            stories.len(),
            3,
            "SimpleTextComponent should have exactly 3 stories"
        );
    }

    #[test]
    fn test_story_names_are_correct() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        let expected_names = vec!["Default", "Styled", "MultiLine"];
        let actual_names: Vec<&str> = stories.iter().map(|s| s.name.as_str()).collect();

        assert_eq!(
            actual_names, expected_names,
            "Story names should match expected values"
        );
    }

    #[test]
    fn test_story_descriptions_are_not_empty() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        for story in stories {
            assert!(
                !story.description.is_empty(),
                "Story '{}' should have a non-empty description",
                story.name
            );
        }
    }

    #[test]
    fn test_default_story_renders_without_panic() {
        let component = SimpleTextComponent;
        let stories = component.stories();
        let default_story = &stories[0];

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = frame.size();
            (default_story.render)(frame, area);
        });

        assert!(result.is_ok(), "Default story should render without errors");
    }

    #[test]
    fn test_styled_story_renders_without_panic() {
        let component = SimpleTextComponent;
        let stories = component.stories();
        let styled_story = &stories[1];

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = frame.size();
            (styled_story.render)(frame, area);
        });

        assert!(result.is_ok(), "Styled story should render without errors");
    }

    #[test]
    fn test_multiline_story_renders_without_panic() {
        let component = SimpleTextComponent;
        let stories = component.stories();
        let multiline_story = &stories[2];

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = frame.size();
            (multiline_story.render)(frame, area);
        });

        assert!(
            result.is_ok(),
            "MultiLine story should render without errors"
        );
    }

    #[test]
    fn test_all_stories_render_without_panic() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should render without errors",
                story.name
            );
        }
    }

    #[test]
    fn test_renders_with_small_terminal() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        // Test with a very small terminal size
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should handle small terminal sizes",
                story.name
            );
        }
    }

    #[test]
    fn test_renders_with_large_terminal() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        // Test with a large terminal size
        let backend = TestBackend::new(200, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should handle large terminal sizes",
                story.name
            );
        }
    }
}

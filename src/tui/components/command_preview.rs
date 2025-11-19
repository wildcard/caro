//! Command preview component for displaying generated shell commands

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CommandPreviewComponent;

impl ShowcaseComponent for CommandPreviewComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "CommandPreview",
            "Display generated shell commands with syntax highlighting",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new("Simple Command", "Basic command display", |frame, area| {
                let command = "ls -la /home/user";

                let text = vec![
                    Line::from(vec![Span::styled(
                        "Generated Command:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("$ ", Style::default().fg(Color::Green)),
                        Span::styled(command, Style::default().fg(Color::White)),
                    ]),
                ];

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Command Preview")
                    .style(Style::default().fg(Color::Green));

                let paragraph = Paragraph::new(text).block(block);
                frame.render_widget(paragraph, area);
            }),
            ShowcaseStory::new(
                "Complex Command",
                "Multi-line command with pipes and redirections",
                |frame, area| {
                    let text = vec![
                        Line::from(vec![Span::styled(
                            "Generated Command:",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("$ ", Style::default().fg(Color::Green)),
                            Span::styled(
                                "find . -name '*.rs' \\",
                                Style::default().fg(Color::White),
                            ),
                        ]),
                        Line::from(vec![
                            Span::raw("    "),
                            Span::styled("| grep -v target \\", Style::default().fg(Color::White)),
                        ]),
                        Line::from(vec![
                            Span::raw("    "),
                            Span::styled("| xargs wc -l", Style::default().fg(Color::White)),
                        ]),
                    ];

                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Command Preview - Complex")
                        .style(Style::default().fg(Color::Green));

                    let paragraph = Paragraph::new(text).block(block);
                    frame.render_widget(paragraph, area);
                },
            ),
            ShowcaseStory::new(
                "With Description",
                "Command with explanation",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(area);

                    // Description section
                    let desc_text = vec![
                        Line::from(vec![
                            Span::styled(
                                "Query: ",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("list all rust files"),
                        ]),
                        Line::from(""),
                        Line::from("This command finds all Rust source files (.rs) in the current"),
                        Line::from(
                            "directory and its subdirectories, excluding the target folder.",
                        ),
                    ];

                    let desc_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Description")
                        .style(Style::default().fg(Color::Cyan));

                    let desc_paragraph = Paragraph::new(desc_text).block(desc_block);
                    frame.render_widget(desc_paragraph, chunks[0]);

                    // Command section
                    let cmd_text = vec![Line::from(vec![
                        Span::styled("$ ", Style::default().fg(Color::Green)),
                        Span::styled(
                            "find . -name '*.rs' -not -path './target/*'",
                            Style::default().fg(Color::White),
                        ),
                    ])];

                    let cmd_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Command")
                        .style(Style::default().fg(Color::Green));

                    let cmd_paragraph = Paragraph::new(cmd_text).block(cmd_block);
                    frame.render_widget(cmd_paragraph, chunks[1]);
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
    fn test_metadata_name() {
        let component = CommandPreviewComponent;
        assert_eq!(component.metadata().name, "CommandPreview");
    }

    #[test]
    fn test_metadata_description() {
        let component = CommandPreviewComponent;
        assert_eq!(
            component.metadata().description,
            "Display generated shell commands with syntax highlighting"
        );
    }

    #[test]
    fn test_metadata_category() {
        let component = CommandPreviewComponent;
        assert_eq!(component.metadata().category, "Display");
    }

    #[test]
    fn test_metadata_version() {
        let component = CommandPreviewComponent;
        assert_eq!(component.metadata().version, "1.0.0");
    }

    #[test]
    fn test_has_three_stories() {
        let component = CommandPreviewComponent;
        assert_eq!(component.stories().len(), 3);
    }

    #[test]
    fn test_story_names() {
        let component = CommandPreviewComponent;
        let stories = component.stories();
        let names: Vec<&str> = stories.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["Simple Command", "Complex Command", "With Description"]
        );
    }

    #[test]
    fn test_all_stories_have_descriptions() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        for story in stories {
            assert!(
                !story.description.is_empty(),
                "Story '{}' should have a description",
                story.name
            );
        }
    }

    #[test]
    fn test_simple_command_story_renders() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            (stories[0].render)(frame, frame.size());
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_command_story_renders() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            (stories[1].render)(frame, frame.size());
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_with_description_story_renders() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            (stories[2].render)(frame, frame.size());
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_all_stories_render() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                (story.render)(frame, frame.size());
            });

            assert!(
                result.is_ok(),
                "Story '{}' should render without errors",
                story.name
            );
        }
    }

    #[test]
    fn test_renders_with_various_sizes() {
        let component = CommandPreviewComponent;
        let stories = component.stories();

        let sizes = vec![(40, 10), (80, 24), (120, 40), (200, 60)];

        for (width, height) in sizes {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();

            for story in &stories {
                let result = terminal.draw(|frame| {
                    (story.render)(frame, frame.size());
                });

                assert!(
                    result.is_ok(),
                    "Story '{}' should render at size {}x{}",
                    story.name,
                    width,
                    height
                );
            }
        }
    }
}

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
            ShowcaseStory::new(
                "Simple Command",
                "Basic command display",
                |frame, area| {
                    let command = "ls -la /home/user";

                    let text = vec![
                        Line::from(vec![
                            Span::styled("Generated Command:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        ]),
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
                },
            ),
            ShowcaseStory::new(
                "Complex Command",
                "Multi-line command with pipes and redirections",
                |frame, area| {
                    let text = vec![
                        Line::from(vec![
                            Span::styled("Generated Command:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("$ ", Style::default().fg(Color::Green)),
                            Span::styled("find . -name '*.rs' \\", Style::default().fg(Color::White)),
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
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Percentage(60),
                        ])
                        .split(area);

                    // Description section
                    let desc_text = vec![
                        Line::from(vec![
                            Span::styled("Query: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            Span::raw("list all rust files"),
                        ]),
                        Line::from(""),
                        Line::from("This command finds all Rust source files (.rs) in the current"),
                        Line::from("directory and its subdirectories, excluding the target folder."),
                    ];

                    let desc_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Description")
                        .style(Style::default().fg(Color::Cyan));

                    let desc_paragraph = Paragraph::new(desc_text).block(desc_block);
                    frame.render_widget(desc_paragraph, chunks[0]);

                    // Command section
                    let cmd_text = vec![
                        Line::from(vec![
                            Span::styled("$ ", Style::default().fg(Color::Green)),
                            Span::styled("find . -name '*.rs' -not -path './target/*'", Style::default().fg(Color::White)),
                        ]),
                    ];

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

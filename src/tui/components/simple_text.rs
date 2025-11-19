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
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Simple Text");
                    let paragraph = Paragraph::new("Hello, Ratatui Showcase!")
                        .block(block)
                        .alignment(Alignment::Center);
                    frame.render_widget(paragraph, area);
                },
            ),
            ShowcaseStory::new(
                "Styled",
                "Text with colors and modifiers",
                |frame, area| {
                    let text = vec![
                        Line::from(vec![
                            Span::styled("Bold", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(" "),
                            Span::styled(
                                "Italic",
                                Style::default().add_modifier(Modifier::ITALIC),
                            ),
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
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Styled Text");
                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .alignment(Alignment::Center);
                    frame.render_widget(paragraph, area);
                },
            ),
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

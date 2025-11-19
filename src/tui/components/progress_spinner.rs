//! Progress spinner component for loading states

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct ProgressSpinnerComponent;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn render_spinner(frame: &mut Frame, area: Rect, frame_idx: usize, message: &str, color: Color) {
    let spinner_char = SPINNER_FRAMES[frame_idx % SPINNER_FRAMES.len()];

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!(" {} ", spinner_char),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(message),
        ]),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Loading")
        .style(Style::default().fg(color));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

impl ShowcaseComponent for ProgressSpinnerComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "ProgressSpinner",
            "Animated spinner for loading and progress indication",
        )
        .with_category("Feedback")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Frame 0",
                "Spinner animation frame 0",
                |frame, area| {
                    render_spinner(frame, area, 0, "Loading model...", Color::Cyan);
                },
            ),
            ShowcaseStory::new(
                "Frame 1",
                "Spinner animation frame 1",
                |frame, area| {
                    render_spinner(frame, area, 1, "Loading model...", Color::Cyan);
                },
            ),
            ShowcaseStory::new(
                "Frame 2",
                "Spinner animation frame 2",
                |frame, area| {
                    render_spinner(frame, area, 2, "Loading model...", Color::Cyan);
                },
            ),
            ShowcaseStory::new(
                "Frame 3",
                "Spinner animation frame 3",
                |frame, area| {
                    render_spinner(frame, area, 3, "Loading model...", Color::Cyan);
                },
            ),
            ShowcaseStory::new(
                "Generating Command",
                "Spinner while generating a command",
                |frame, area| {
                    render_spinner(frame, area, 0, "Generating command...", Color::Green);
                },
            ),
            ShowcaseStory::new(
                "Processing",
                "Spinner during processing",
                |frame, area| {
                    render_spinner(frame, area, 5, "Processing request...", Color::Yellow);
                },
            ),
        ]
    }
}

//! Confirmation dialog component for user prompts

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct ConfirmationDialogComponent;

fn render_dialog(frame: &mut Frame, area: Rect, title: &str, message: &str, selected: bool) {
    // Center the dialog in the available area
    let dialog_width = area.width.min(60);
    let dialog_height = area.height.min(12);

    let dialog_area = Rect {
        x: area.x + (area.width.saturating_sub(dialog_width)) / 2,
        y: area.y + (area.height.saturating_sub(dialog_height)) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(dialog_area);

    // Message area
    let message_lines: Vec<Line> = message
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect();

    let message_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    let message_paragraph = Paragraph::new(message_lines)
        .block(message_block)
        .alignment(Alignment::Center);

    frame.render_widget(message_paragraph, chunks[0]);

    // Button area
    let yes_style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let no_style = if !selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };

    let buttons = vec![Line::from(vec![
        Span::raw("  "),
        Span::styled(" Yes ", yes_style),
        Span::raw("   "),
        Span::styled(" No ", no_style),
        Span::raw("  "),
    ])];

    let buttons_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let buttons_paragraph = Paragraph::new(buttons)
        .block(buttons_block)
        .alignment(Alignment::Center);

    frame.render_widget(buttons_paragraph, chunks[1]);
}

impl ShowcaseComponent for ConfirmationDialogComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "ConfirmationDialog",
            "Modal dialog for user confirmation with Yes/No buttons",
        )
        .with_category("Input")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Yes Selected",
                "Confirmation dialog with 'Yes' button highlighted",
                |frame, area| {
                    render_dialog(
                        frame,
                        area,
                        "Confirm Execution",
                        "Do you want to execute this command?\n\nls -la /home/user",
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "No Selected",
                "Confirmation dialog with 'No' button highlighted",
                |frame, area| {
                    render_dialog(
                        frame,
                        area,
                        "Confirm Execution",
                        "Do you want to execute this command?\n\nls -la /home/user",
                        false,
                    );
                },
            ),
            ShowcaseStory::new(
                "Dangerous Command",
                "Warning dialog for dangerous operations",
                |frame, area| {
                    render_dialog(
                        frame,
                        area,
                        "⚠ Warning: Dangerous Command",
                        "This command will delete files!\n\nrm -rf ./target\n\nAre you sure?",
                        false,
                    );
                },
            ),
            ShowcaseStory::new(
                "Long Message",
                "Dialog with longer explanatory text",
                |frame, area| {
                    render_dialog(
                        frame,
                        area,
                        "Confirm Action",
                        "This command will modify system files.\nMake sure you have a backup before proceeding.\n\nContinue?",
                        true,
                    );
                },
            ),
        ]
    }
}

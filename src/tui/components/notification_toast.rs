//! Notification toast/banner component
//!
//! Temporary notifications, errors, warnings, and success messages

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct NotificationToastComponent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn color(&self) -> Color {
        match self {
            ToastLevel::Info => Color::Cyan,
            ToastLevel::Success => Color::Green,
            ToastLevel::Warning => Color::Yellow,
            ToastLevel::Error => Color::Red,
        }
    }

    fn icon(&self) -> &str {
        match self {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✗",
        }
    }

    fn label(&self) -> &str {
        match self {
            ToastLevel::Info => "INFO",
            ToastLevel::Success => "SUCCESS",
            ToastLevel::Warning => "WARNING",
            ToastLevel::Error => "ERROR",
        }
    }
}

fn render_toast(
    frame: &mut Frame,
    area: Rect,
    level: ToastLevel,
    message: &str,
    position: &str,
    show_dismiss: bool,
) {
    let toast_height = 5;
    let toast_width = area.width.min(60);

    let toast_area = match position {
        "top" => Rect {
            x: area.x + (area.width.saturating_sub(toast_width)) / 2,
            y: area.y + 2,
            width: toast_width,
            height: toast_height,
        },
        "bottom" => Rect {
            x: area.x + (area.width.saturating_sub(toast_width)) / 2,
            y: area.y + area.height.saturating_sub(toast_height + 2),
            width: toast_width,
            height: toast_height,
        },
        "top-right" => Rect {
            x: area.x + area.width.saturating_sub(toast_width + 2),
            y: area.y + 2,
            width: toast_width,
            height: toast_height,
        },
        _ => Rect {
            x: area.x + (area.width.saturating_sub(toast_width)) / 2,
            y: area.y + (area.height.saturating_sub(toast_height)) / 2,
            width: toast_width,
            height: toast_height,
        },
    };

    let color = level.color();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} ", level.icon()),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                level.label(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(": "),
            Span::raw(message),
        ]),
    ];

    if show_dismiss {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Press any key to dismiss",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let toast = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(toast, toast_area);
}

fn render_banner(frame: &mut Frame, area: Rect, level: ToastLevel, message: &str, position: &str) {
    let banner_height = 3;

    let banner_area = match position {
        "top" => Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: banner_height,
        },
        "bottom" => Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(banner_height),
            width: area.width,
            height: banner_height,
        },
        _ => Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: banner_height,
        },
    };

    let color = level.color();

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", level.icon()),
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", level.label()),
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", message),
            Style::default().fg(Color::White).bg(color),
        ),
    ]);

    let banner = Paragraph::new(line)
        .style(Style::default().bg(color))
        .alignment(Alignment::Left);

    frame.render_widget(banner, banner_area);
}

impl ShowcaseComponent for NotificationToastComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "NotificationToast",
            "Temporary notifications, toasts, and banner messages",
        )
        .with_category("Feedback")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Info Toast - Center",
                "Informational toast notification centered on screen",
                |frame, area| {
                    render_toast(
                        frame,
                        area,
                        ToastLevel::Info,
                        "Loading configuration...",
                        "center",
                        false,
                    );
                },
            ),
            ShowcaseStory::new(
                "Success Toast - Top",
                "Success notification at top of screen",
                |frame, area| {
                    render_toast(
                        frame,
                        area,
                        ToastLevel::Success,
                        "Command executed successfully!",
                        "top",
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "Warning Toast - Bottom",
                "Warning notification at bottom of screen",
                |frame, area| {
                    render_toast(
                        frame,
                        area,
                        ToastLevel::Warning,
                        "This command may modify system files",
                        "bottom",
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "Error Toast - Top Right",
                "Error notification in top right corner",
                |frame, area| {
                    render_toast(
                        frame,
                        area,
                        ToastLevel::Error,
                        "Failed to connect to backend",
                        "top-right",
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "Info Banner - Top",
                "Full-width information banner at top",
                |frame, area| {
                    render_banner(
                        frame,
                        area,
                        ToastLevel::Info,
                        "Press F1 for help | Ctrl+C to exit",
                        "top",
                    );
                },
            ),
            ShowcaseStory::new(
                "Success Banner - Bottom",
                "Full-width success banner at bottom",
                |frame, area| {
                    render_banner(
                        frame,
                        area,
                        ToastLevel::Success,
                        "Configuration saved successfully",
                        "bottom",
                    );
                },
            ),
            ShowcaseStory::new(
                "Warning Banner - Top",
                "Full-width warning banner at top",
                |frame, area| {
                    render_banner(
                        frame,
                        area,
                        ToastLevel::Warning,
                        "Experimental feature enabled - Use with caution",
                        "top",
                    );
                },
            ),
            ShowcaseStory::new(
                "Error Banner - Bottom",
                "Full-width error banner at bottom",
                |frame, area| {
                    render_banner(
                        frame,
                        area,
                        ToastLevel::Error,
                        "Network connection lost - Retrying...",
                        "bottom",
                    );
                },
            ),
        ]
    }
}

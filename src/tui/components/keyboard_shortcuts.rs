//! Keyboard shortcuts reference component
//!
//! Displays available keyboard shortcuts in an organized, easy-to-read format

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct KeyboardShortcutsComponent;

struct Shortcut {
    key: &'static str,
    description: &'static str,
    category: &'static str,
}

const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        key: "Ctrl+C",
        description: "Cancel current operation / Exit",
        category: "General",
    },
    Shortcut {
        key: "Enter",
        description: "Confirm / Execute command",
        category: "General",
    },
    Shortcut {
        key: "Esc",
        description: "Cancel / Go back",
        category: "General",
    },
    Shortcut {
        key: "↑/↓",
        description: "Navigate up/down",
        category: "Navigation",
    },
    Shortcut {
        key: "←/→",
        description: "Navigate left/right",
        category: "Navigation",
    },
    Shortcut {
        key: "Tab",
        description: "Next field / Auto-complete",
        category: "Navigation",
    },
    Shortcut {
        key: "Shift+Tab",
        description: "Previous field",
        category: "Navigation",
    },
    Shortcut {
        key: "Ctrl+E",
        description: "Edit command before execution",
        category: "Editing",
    },
    Shortcut {
        key: "Ctrl+H",
        description: "Show command history",
        category: "Features",
    },
    Shortcut {
        key: "Ctrl+R",
        description: "Regenerate command",
        category: "Features",
    },
    Shortcut {
        key: "Ctrl+S",
        description: "Save command to favorites",
        category: "Features",
    },
    Shortcut {
        key: "F1",
        description: "Show help / keyboard shortcuts",
        category: "Help",
    },
];

fn render_shortcuts(
    frame: &mut Frame,
    area: Rect,
    style_variant: &str,
    show_categories: bool,
) {
    match style_variant {
        "compact" => render_compact(frame, area, show_categories),
        "detailed" => render_detailed(frame, area, show_categories),
        "grid" => render_grid(frame, area),
        _ => render_compact(frame, area, show_categories),
    }
}

fn render_compact(frame: &mut Frame, area: Rect, show_categories: bool) {
    let mut lines = vec![Line::from("")];

    if show_categories {
        let categories = ["General", "Navigation", "Editing", "Features", "Help"];

        for category in &categories {
            lines.push(Line::from(vec![Span::styled(
                format!("{}:", category),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));

            for shortcut in SHORTCUTS.iter().filter(|s| s.category == *category) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:15}", shortcut.key),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(shortcut.description),
                ]));
            }
            lines.push(Line::from(""));
        }
    } else {
        for shortcut in SHORTCUTS {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:15}", shortcut.key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(shortcut.description),
            ]));
        }
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keyboard Shortcuts")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(widget, area);
}

fn render_detailed(frame: &mut Frame, area: Rect, show_categories: bool) {
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Available Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    if show_categories {
        let categories = ["General", "Navigation", "Editing", "Features", "Help"];

        for category in &categories {
            lines.push(Line::from(vec![
                Span::raw("╔═══ "),
                Span::styled(
                    *category,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ═══"),
            ]));
            lines.push(Line::from("║"));

            for shortcut in SHORTCUTS.iter().filter(|s| s.category == *category) {
                lines.push(Line::from(vec![
                    Span::raw("║  "),
                    Span::styled(
                        format!("{:15}", shortcut.key),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                    Span::raw(shortcut.description),
                ]));
            }
            lines.push(Line::from("║"));
        }
        lines.push(Line::from("╚════════════════════════"));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(widget, area);
}

fn render_grid(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Keyboard Shortcuts Reference")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Split into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left column
    let left_shortcuts: Vec<Line> = SHORTCUTS[0..6]
        .iter()
        .flat_map(|s| {
            vec![
                Line::from(vec![
                    Span::styled(
                        s.key,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::raw(s.description),
                ]),
                Line::from(""),
            ]
        })
        .collect();

    let left_widget = Paragraph::new(left_shortcuts).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Essential"),
    );
    frame.render_widget(left_widget, columns[0]);

    // Right column
    let right_shortcuts: Vec<Line> = SHORTCUTS[6..]
        .iter()
        .flat_map(|s| {
            vec![
                Line::from(vec![
                    Span::styled(
                        s.key,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::raw(s.description),
                ]),
                Line::from(""),
            ]
        })
        .collect();

    let right_widget = Paragraph::new(right_shortcuts).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Advanced"),
    );
    frame.render_widget(right_widget, columns[1]);

    // Footer
    let footer = Paragraph::new("Press F1 anytime to show this help")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

impl ShowcaseComponent for KeyboardShortcutsComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "KeyboardShortcuts",
            "Keyboard shortcuts reference in various display formats",
        )
        .with_category("Help")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Compact List",
                "Simple compact list of all shortcuts",
                |frame, area| {
                    render_shortcuts(frame, area, "compact", false);
                },
            ),
            ShowcaseStory::new(
                "Compact with Categories",
                "Compact list organized by category",
                |frame, area| {
                    render_shortcuts(frame, area, "compact", true);
                },
            ),
            ShowcaseStory::new(
                "Detailed View",
                "Detailed view with decorative borders and categories",
                |frame, area| {
                    render_shortcuts(frame, area, "detailed", true);
                },
            ),
            ShowcaseStory::new(
                "Grid Layout",
                "Two-column grid layout with header and footer",
                |frame, area| {
                    render_shortcuts(frame, area, "grid", false);
                },
            ),
        ]
    }
}

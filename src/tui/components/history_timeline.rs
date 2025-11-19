//! History timeline component with filtering and visualization
//!
//! This component addresses the community's request for better history
//! visualization with timeline views, filtering, and search capabilities.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct HistoryTimelineComponent;

#[derive(Debug, Clone)]
struct HistoryEntry {
    timestamp: &'static str,
    time: &'static str,
    query: &'static str,
    command: &'static str,
    status: CommandStatus,
    duration: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandStatus {
    Success,
    Blocked,
    Cancelled,
    Failed,
}

impl CommandStatus {
    fn color(&self) -> Color {
        match self {
            CommandStatus::Success => Color::Green,
            CommandStatus::Blocked => Color::Red,
            CommandStatus::Cancelled => Color::Yellow,
            CommandStatus::Failed => Color::LightRed,
        }
    }

    fn icon(&self) -> &str {
        match self {
            CommandStatus::Success => "✓",
            CommandStatus::Blocked => "✗",
            CommandStatus::Cancelled => "⚠",
            CommandStatus::Failed => "✗",
        }
    }

    fn label(&self) -> &str {
        match self {
            CommandStatus::Success => "SUCCESS",
            CommandStatus::Blocked => "BLOCKED",
            CommandStatus::Cancelled => "CANCELLED",
            CommandStatus::Failed => "FAILED",
        }
    }
}

const HISTORY: &[HistoryEntry] = &[
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:32:15",
        query: "list all PDF files larger than 10MB",
        command: "find . -name '*.pdf' -size +10M -ls",
        status: CommandStatus::Success,
        duration: "0.8s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:30:42",
        query: "show disk usage in human readable format",
        command: "df -h",
        status: CommandStatus::Success,
        duration: "0.1s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:28:19",
        query: "find large log files over 100MB",
        command: "find /var/log -name '*.log' -size +100M",
        status: CommandStatus::Success,
        duration: "1.2s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:25:33",
        query: "compress all images to 85% quality",
        command: "find . -name '*.jpg' -exec convert {} -quality 85 {} \\;",
        status: CommandStatus::Cancelled,
        duration: "0.0s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:22:01",
        query: "delete all temporary files",
        command: "rm -rf /tmp/*",
        status: CommandStatus::Blocked,
        duration: "0.0s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:18:45",
        query: "count lines in all Rust files",
        command: "find . -name '*.rs' | xargs wc -l",
        status: CommandStatus::Success,
        duration: "0.3s",
    },
    HistoryEntry {
        timestamp: "2025-01-19",
        time: "14:15:22",
        query: "show running Docker containers",
        command: "docker ps",
        status: CommandStatus::Failed,
        duration: "0.1s",
    },
];

fn render_timeline(
    frame: &mut Frame,
    area: Rect,
    view_type: &str,
    filter_status: Option<CommandStatus>,
    selected_idx: Option<usize>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with filters
            Constraint::Min(5),     // Timeline content
            Constraint::Length(3),  // Footer with stats
        ])
        .split(area);

    // Header with active filters
    let filter_text = if let Some(status) = filter_status {
        format!("Filter: {} commands only", status.label())
    } else {
        "Showing all commands".to_string()
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Command History Timeline", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  │  "),
            Span::styled(filter_text, Style::default().fg(Color::Yellow)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Timeline content
    let filtered_history: Vec<&HistoryEntry> = if let Some(status) = filter_status {
        HISTORY.iter().filter(|e| e.status == status).collect()
    } else {
        HISTORY.iter().collect()
    };

    let mut lines = Vec::new();

    match view_type {
        "compact" => {
            for (idx, entry) in filtered_history.iter().enumerate() {
                let is_selected = selected_idx == Some(idx);
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", entry.status.icon()),
                        Style::default().fg(entry.status.color()).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {} ", entry.time), style.fg(Color::DarkGray)),
                    Span::styled(format!("{:40}", entry.query), style.fg(Color::White)),
                    Span::styled(format!(" {} ", entry.duration), style.fg(Color::Cyan)),
                ]));
            }
        }
        "detailed" => {
            for (idx, entry) in filtered_history.iter().enumerate() {
                let is_selected = selected_idx == Some(idx);

                // Timeline connector
                let connector = if idx == 0 { "┌" } else if idx == filtered_history.len() - 1 { "└" } else { "├" };
                let connector_color = entry.status.color();

                // Entry header
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", connector),
                        Style::default().fg(connector_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {} ", entry.status.icon()),
                        Style::default().fg(entry.status.color()).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} {} ", entry.timestamp, entry.time),
                        if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(
                        entry.status.label(),
                        Style::default().fg(entry.status.color()).add_modifier(Modifier::BOLD),
                    ),
                ]));

                // Query
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(connector_color)),
                    Span::styled("Query: ", Style::default().fg(Color::Cyan)),
                    Span::raw(entry.query),
                ]));

                // Command
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(connector_color)),
                    Span::styled("$ ", Style::default().fg(Color::Green)),
                    Span::styled(entry.command, Style::default().fg(Color::White)),
                ]));

                // Duration
                lines.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(connector_color)),
                    Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(entry.duration, Style::default().fg(Color::Cyan)),
                ]));

                // Spacer
                if idx < filtered_history.len() - 1 {
                    lines.push(Line::from(vec![
                        Span::styled(" │", Style::default().fg(connector_color)),
                    ]));
                }
            }
        }
        "stats" => {
            // Summary statistics view
            let total = filtered_history.len();
            let success = filtered_history.iter().filter(|e| e.status == CommandStatus::Success).count();
            let blocked = filtered_history.iter().filter(|e| e.status == CommandStatus::Blocked).count();
            let cancelled = filtered_history.iter().filter(|e| e.status == CommandStatus::Cancelled).count();
            let failed = filtered_history.iter().filter(|e| e.status == CommandStatus::Failed).count();

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Session Statistics", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Total Commands: ", Style::default().fg(Color::White)),
                Span::styled(total.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  ✓ Success:   ", Style::default().fg(Color::Green)),
                Span::styled(format!("{:3}", success), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:3.0}%)", (success as f32 / total as f32) * 100.0), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ✗ Blocked:   ", Style::default().fg(Color::Red)),
                Span::styled(format!("{:3}", blocked), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:3.0}%)", (blocked as f32 / total as f32) * 100.0), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ⚠ Cancelled: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:3}", cancelled), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:3.0}%)", (cancelled as f32 / total as f32) * 100.0), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ✗ Failed:    ", Style::default().fg(Color::LightRed)),
                Span::styled(format!("{:3}", failed), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:3.0}%)", (failed as f32 / total as f32) * 100.0), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Success Rate: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1}%", (success as f32 / total as f32) * 100.0),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        _ => {}
    }

    let timeline = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Timeline"),
    );
    frame.render_widget(timeline, chunks[1]);

    // Footer
    let footer_text = format!(
        "Showing {} of {} commands  │  Use ↑↓ to navigate, F to filter, S for stats",
        filtered_history.len(),
        HISTORY.len()
    );

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

impl ShowcaseComponent for HistoryTimelineComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "HistoryTimeline",
            "Timeline view of command history with filtering and statistics",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Compact View",
                "Compact timeline showing all commands in list format",
                |frame, area| {
                    render_timeline(frame, area, "compact", None, None);
                },
            ),
            ShowcaseStory::new(
                "Compact with Selection",
                "Compact view with a selected command",
                |frame, area| {
                    render_timeline(frame, area, "compact", None, Some(2));
                },
            ),
            ShowcaseStory::new(
                "Detailed View",
                "Detailed timeline with full information for each command",
                |frame, area| {
                    render_timeline(frame, area, "detailed", None, None);
                },
            ),
            ShowcaseStory::new(
                "Detailed with Selection",
                "Detailed view with highlighted selected command",
                |frame, area| {
                    render_timeline(frame, area, "detailed", None, Some(1));
                },
            ),
            ShowcaseStory::new(
                "Filter: Success Only",
                "Timeline filtered to show only successful commands",
                |frame, area| {
                    render_timeline(frame, area, "detailed", Some(CommandStatus::Success), None);
                },
            ),
            ShowcaseStory::new(
                "Filter: Blocked Only",
                "Timeline filtered to show only blocked commands",
                |frame, area| {
                    render_timeline(frame, area, "detailed", Some(CommandStatus::Blocked), None);
                },
            ),
            ShowcaseStory::new(
                "Statistics View",
                "Summary statistics with success rates and breakdowns",
                |frame, area| {
                    render_timeline(frame, area, "stats", None, None);
                },
            ),
        ]
    }
}

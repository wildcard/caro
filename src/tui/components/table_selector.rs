//! Interactive table/list selector component
//!
//! Demonstrates data tables, selection states, and keyboard navigation patterns

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub struct TableSelectorComponent;

struct CommandHistoryEntry {
    timestamp: &'static str,
    query: &'static str,
    command: &'static str,
    status: &'static str,
}

const HISTORY: &[CommandHistoryEntry] = &[
    CommandHistoryEntry {
        timestamp: "2025-01-19 14:32:15",
        query: "list all PDF files",
        command: "find . -name '*.pdf' -ls",
        status: "✓ Success",
    },
    CommandHistoryEntry {
        timestamp: "2025-01-19 14:30:42",
        query: "show disk usage",
        command: "df -h",
        status: "✓ Success",
    },
    CommandHistoryEntry {
        timestamp: "2025-01-19 14:28:19",
        query: "find large log files",
        command: "find /var/log -name '*.log' -size +100M",
        status: "✓ Success",
    },
    CommandHistoryEntry {
        timestamp: "2025-01-19 14:25:33",
        query: "compress images",
        command: "find . -name '*.jpg' -exec convert {} -quality 85 {}",
        status: "⚠ Cancelled",
    },
    CommandHistoryEntry {
        timestamp: "2025-01-19 14:22:01",
        query: "delete temp files",
        command: "rm -rf /tmp/*",
        status: "✗ Blocked",
    },
];

fn render_table(
    frame: &mut Frame,
    area: Rect,
    selected_idx: Option<usize>,
    show_header: bool,
    highlight_dangerous: bool,
) {
    let header_cells = ["Time", "Query", "Command", "Status"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows = HISTORY.iter().enumerate().map(|(idx, entry)| {
        let status_color = if entry.status.contains("Success") {
            Color::Green
        } else if entry.status.contains("Cancelled") {
            Color::Yellow
        } else {
            Color::Red
        };

        let is_selected = selected_idx == Some(idx);
        let is_dangerous = highlight_dangerous && entry.status.contains("Blocked");

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_dangerous {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };

        let cells = vec![
            Cell::from(entry.timestamp),
            Cell::from(entry.query),
            Cell::from(entry.command),
            Cell::from(entry.status).style(Style::default().fg(status_color)),
        ];

        Row::new(cells).style(style).height(1)
    });

    let widths = [
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Min(30),
        Constraint::Length(12),
    ];

    let table = if show_header {
        Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command History"),
            )
            .column_spacing(1)
    } else {
        Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command History (No Header)"),
            )
            .column_spacing(1)
    };

    frame.render_widget(table, area);
}

impl ShowcaseComponent for TableSelectorComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "TableSelector",
            "Interactive data tables with selection, sorting, and highlighting",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Default Table",
                "Basic table with headers and data",
                |frame, area| {
                    render_table(frame, area, None, true, false);
                },
            ),
            ShowcaseStory::new(
                "First Row Selected",
                "Table with first row selected",
                |frame, area| {
                    render_table(frame, area, Some(0), true, false);
                },
            ),
            ShowcaseStory::new(
                "Middle Row Selected",
                "Table with middle row selected for navigation",
                |frame, area| {
                    render_table(frame, area, Some(2), true, false);
                },
            ),
            ShowcaseStory::new(
                "Last Row Selected",
                "Table with last row selected",
                |frame, area| {
                    render_table(frame, area, Some(4), true, false);
                },
            ),
            ShowcaseStory::new(
                "Dangerous Rows Highlighted",
                "Table with blocked/dangerous commands highlighted in red",
                |frame, area| {
                    render_table(frame, area, None, true, true);
                },
            ),
            ShowcaseStory::new(
                "No Header",
                "Table without header row for compact display",
                |frame, area| {
                    render_table(frame, area, Some(1), false, false);
                },
            ),
            ShowcaseStory::new(
                "Selected Dangerous",
                "Selecting a dangerous/blocked command",
                |frame, area| {
                    render_table(frame, area, Some(4), true, true);
                },
            ),
        ]
    }
}

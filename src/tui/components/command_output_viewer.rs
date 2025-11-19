//! Command output viewer with syntax highlighting and scrolling
//!
//! This component addresses the community's need for better command output
//! visualization with features like syntax highlighting, line numbers,
//! and scrollable content for long outputs.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub struct CommandOutputViewerComponent;

fn render_output(
    frame: &mut Frame,
    area: Rect,
    output_type: &str,
    show_line_numbers: bool,
    scroll_position: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(5),     // Output area
            Constraint::Length(3),  // Footer with stats
        ])
        .split(area);

    // Header with command
    let command = match output_type {
        "success" => "find . -name '*.rs' | wc -l",
        "error" => "cat nonexistent_file.txt",
        "long" => "cat /var/log/system.log",
        _ => "ls -la",
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("$ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(command, Style::default().fg(Color::White)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Command")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(header, chunks[0]);

    // Output content
    let output_lines = get_output_lines(output_type);
    let visible_start = scroll_position;
    let visible_end = (scroll_position + chunks[1].height as usize).min(output_lines.len());

    let mut lines = Vec::new();
    for (idx, line_content) in output_lines[visible_start..visible_end].iter().enumerate() {
        let actual_line = visible_start + idx + 1;
        let mut spans = Vec::new();

        if show_line_numbers {
            spans.push(Span::styled(
                format!(" {:4} │ ", actual_line),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Add colored output based on content
        if line_content.contains("Error") || line_content.contains("error") {
            spans.push(Span::styled(*line_content, Style::default().fg(Color::Red)));
        } else if line_content.contains("Warning") || line_content.contains("warning") {
            spans.push(Span::styled(*line_content, Style::default().fg(Color::Yellow)));
        } else if line_content.contains("Success") || line_content.starts_with('+') {
            spans.push(Span::styled(*line_content, Style::default().fg(Color::Green)));
        } else if line_content.starts_with("│") || line_content.starts_with("├") || line_content.starts_with("└") {
            spans.push(Span::styled(*line_content, Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::raw(*line_content));
        }

        lines.push(Line::from(spans));
    }

    let output_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Output (Lines {}-{}/{})",
            visible_start + 1,
            visible_end,
            output_lines.len()
        ))
        .border_style(Style::default().fg(if output_type == "error" {
            Color::Red
        } else {
            Color::Green
        }));

    let output = Paragraph::new(lines).block(output_block);
    frame.render_widget(output, chunks[1]);

    // Scrollbar if content is long
    if output_lines.len() > chunks[1].height as usize {
        let mut scrollbar_state = ScrollbarState::new(output_lines.len())
            .position(scroll_position);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_stateful_widget(
            scrollbar,
            chunks[1],
            &mut scrollbar_state,
        );
    }

    // Footer with stats
    let exit_code = match output_type {
        "error" => ("1", Color::Red),
        _ => ("0", Color::Green),
    };

    let duration = match output_type {
        "long" => "2.4s",
        _ => "0.2s",
    };

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Exit Code: ", Style::default().fg(Color::DarkGray)),
            Span::styled(exit_code.0, Style::default().fg(exit_code.1).add_modifier(Modifier::BOLD)),
            Span::raw("  │  "),
            Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
            Span::styled(duration, Style::default().fg(Color::Cyan)),
            Span::raw("  │  "),
            Span::styled("Lines: ", Style::default().fg(Color::DarkGray)),
            Span::styled(output_lines.len().to_string(), Style::default().fg(Color::White)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn get_output_lines(output_type: &str) -> Vec<&'static str> {
    match output_type {
        "success" => vec![
            "src/main.rs",
            "src/lib.rs",
            "src/backends/mod.rs",
            "src/backends/remote/ollama.rs",
            "src/backends/remote/vllm.rs",
            "src/backends/embedded/cpu.rs",
            "src/safety/mod.rs",
            "src/safety/patterns.rs",
            "src/tui/mod.rs",
            "src/tui/showcase.rs",
            "",
            "Total: 42 Rust files",
        ],
        "error" => vec![
            "cat: nonexistent_file.txt: No such file or directory",
            "",
            "Error: Failed to read file",
            "  at main.rs:42:5",
            "  Caused by:",
            "    File not found: nonexistent_file.txt",
            "",
            "Suggestion: Check if the file exists or use 'ls' to list files",
        ],
        "long" => vec![
            "2025-01-19 10:15:23 [INFO] System startup initiated",
            "2025-01-19 10:15:24 [INFO] Loading configuration from /etc/cmdai/config.toml",
            "2025-01-19 10:15:24 [INFO] Initializing MLX backend for Apple Silicon",
            "2025-01-19 10:15:25 [INFO] Model loaded: mlx-community/Qwen2.5-Coder-1.5B-Instruct",
            "2025-01-19 10:15:26 [INFO] Cache directory: ~/.cache/cmdai",
            "2025-01-19 10:15:26 [INFO] Safety validation enabled",
            "2025-01-19 10:15:27 [INFO] CLI initialized successfully",
            "2025-01-19 10:16:42 [INFO] User query: list all PDF files",
            "2025-01-19 10:16:43 [INFO] Generating command...",
            "2025-01-19 10:16:44 [INFO] Generated: find . -name '*.pdf' -ls",
            "2025-01-19 10:16:44 [INFO] Safety check: SAFE",
            "2025-01-19 10:16:45 [INFO] User confirmed execution",
            "2025-01-19 10:16:45 [INFO] Executing command...",
            "2025-01-19 10:16:46 [SUCCESS] Command completed successfully",
            "2025-01-19 10:17:23 [INFO] User query: show disk usage",
            "2025-01-19 10:17:24 [INFO] Generating command...",
            "2025-01-19 10:17:25 [INFO] Generated: df -h",
            "2025-01-19 10:17:25 [INFO] Safety check: SAFE",
            "2025-01-19 10:17:26 [INFO] User confirmed execution",
            "2025-01-19 10:17:26 [SUCCESS] Command completed successfully",
            "2025-01-19 10:18:15 [WARNING] High memory usage detected: 85%",
            "2025-01-19 10:18:16 [INFO] Clearing cache...",
            "2025-01-19 10:18:17 [INFO] Cache cleared: 512MB freed",
            "2025-01-19 10:19:34 [INFO] User query: delete temp files",
            "2025-01-19 10:19:35 [INFO] Generating command...",
            "2025-01-19 10:19:36 [INFO] Generated: rm -rf /tmp/*",
            "2025-01-19 10:19:36 [ERROR] Safety check: CRITICAL - Dangerous command blocked",
            "2025-01-19 10:19:37 [INFO] Suggesting safer alternative...",
            "2025-01-19 10:20:45 [INFO] Session ended",
        ],
        "tree" => vec![
            ".",
            "├── Cargo.toml",
            "├── Cargo.lock",
            "├── README.md",
            "├── src/",
            "│   ├── main.rs",
            "│   ├── lib.rs",
            "│   ├── backends/",
            "│   │   ├── mod.rs",
            "│   │   ├── remote/",
            "│   │   │   ├── ollama.rs",
            "│   │   │   └── vllm.rs",
            "│   │   └── embedded/",
            "│   │       ├── mlx.rs",
            "│   │       └── cpu.rs",
            "│   ├── safety/",
            "│   │   ├── mod.rs",
            "│   │   └── patterns.rs",
            "│   └── tui/",
            "│       ├── mod.rs",
            "│       ├── showcase.rs",
            "│       └── components/",
            "│           ├── mod.rs",
            "│           ├── simple_text.rs",
            "│           └── command_preview.rs",
            "└── tests/",
            "    ├── integration/",
            "    └── unit/",
        ],
        _ => vec!["No output"],
    }
}

impl ShowcaseComponent for CommandOutputViewerComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "CommandOutputViewer",
            "Scrollable command output with syntax highlighting and statistics",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Success Output",
                "Successful command execution with line-numbered output",
                |frame, area| {
                    render_output(frame, area, "success", true, 0);
                },
            ),
            ShowcaseStory::new(
                "Error Output",
                "Error output with red highlighting and suggestions",
                |frame, area| {
                    render_output(frame, area, "error", true, 0);
                },
            ),
            ShowcaseStory::new(
                "Long Output - Top",
                "Long scrollable output showing top of content",
                |frame, area| {
                    render_output(frame, area, "long", true, 0);
                },
            ),
            ShowcaseStory::new(
                "Long Output - Middle",
                "Long scrollable output scrolled to middle",
                |frame, area| {
                    render_output(frame, area, "long", true, 10);
                },
            ),
            ShowcaseStory::new(
                "Long Output - Bottom",
                "Long scrollable output scrolled to bottom",
                |frame, area| {
                    render_output(frame, area, "long", true, 20);
                },
            ),
            ShowcaseStory::new(
                "Tree View",
                "Directory tree output with box drawing characters",
                |frame, area| {
                    render_output(frame, area, "tree", false, 0);
                },
            ),
            ShowcaseStory::new(
                "No Line Numbers",
                "Output without line numbers for cleaner view",
                |frame, area| {
                    render_output(frame, area, "success", false, 0);
                },
            ),
        ]
    }
}

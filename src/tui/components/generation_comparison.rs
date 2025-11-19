//! Generation comparison component for AI alternatives
//!
//! This component addresses the community's request for comparing different
//! AI-generated command alternatives side-by-side, enabling better command
//! selection and understanding of different approaches.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct GenerationComparisonComponent;

#[derive(Debug, Clone)]
struct GeneratedAlternative {
    command: &'static str,
    explanation: &'static str,
    pros: &'static [&'static str],
    cons: &'static [&'static str],
    safety_level: SafetyRating,
    performance: &'static str,
    upvotes: usize,
    model: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafetyRating {
    Safe,
    Moderate,
    Risky,
}

impl SafetyRating {
    fn color(&self) -> Color {
        match self {
            SafetyRating::Safe => Color::Green,
            SafetyRating::Moderate => Color::Yellow,
            SafetyRating::Risky => Color::Red,
        }
    }

    fn label(&self) -> &str {
        match self {
            SafetyRating::Safe => "SAFE",
            SafetyRating::Moderate => "MODERATE",
            SafetyRating::Risky => "RISKY",
        }
    }
}

fn get_alternatives(query: &str) -> Vec<GeneratedAlternative> {
    match query {
        "find large files" => vec![
            GeneratedAlternative {
                command: "find . -type f -size +100M -exec ls -lh {} \\;",
                explanation: "Uses find with exec to list large files with human-readable sizes",
                pros: &["POSIX compliant", "Works on all systems", "Shows file details"],
                cons: &["Slower due to multiple ls calls", "Verbose output"],
                safety_level: SafetyRating::Safe,
                performance: "Medium (multiple processes)",
                upvotes: 47,
                model: "MLX Qwen2.5-Coder",
            },
            GeneratedAlternative {
                command: "find . -type f -size +100M -ls",
                explanation: "Uses find's built-in -ls flag for efficient output",
                pros: &["Faster execution", "Single process", "Clean output"],
                cons: &["Less portable (not all find implementations)", "Fixed format"],
                safety_level: SafetyRating::Safe,
                performance: "Fast (single process)",
                upvotes: 32,
                model: "Ollama CodeLlama",
            },
            GeneratedAlternative {
                command: "du -ah | awk '$1 ~ /[0-9]+M/ && $1+0 > 100'",
                explanation: "Uses du and awk to filter files by size threshold",
                pros: &["Very portable", "Flexible filtering", "Can process all file types"],
                cons: &["Less precise size matching", "Requires awk", "Slower for large trees"],
                safety_level: SafetyRating::Safe,
                performance: "Slow (full directory scan)",
                upvotes: 18,
                model: "vLLM Mistral-7B",
            },
        ],
        "delete temp files" => vec![
            GeneratedAlternative {
                command: "find /tmp -type f -name '*.tmp' -mtime +7 -delete",
                explanation: "Safely deletes only .tmp files older than 7 days in /tmp",
                pros: &["Safe (limited scope)", "Preserves recent files", "POSIX compliant"],
                cons: &["Only deletes .tmp files", "Fixed time threshold"],
                safety_level: SafetyRating::Safe,
                performance: "Fast",
                upvotes: 65,
                model: "MLX Qwen2.5-Coder",
            },
            GeneratedAlternative {
                command: "rm -rf /tmp/*",
                explanation: "Removes all files in /tmp directory",
                pros: &["Simple command", "Fast execution"],
                cons: &["DANGEROUS: Deletes everything", "May break running processes", "No confirmation"],
                safety_level: SafetyRating::Risky,
                performance: "Very fast",
                upvotes: 2,
                model: "vLLM Mistral-7B",
            },
            GeneratedAlternative {
                command: "find ~/Downloads -type f -name '*.tmp' -o -name '*.cache' -delete",
                explanation: "Deletes temp and cache files from Downloads folder",
                pros: &["User-scoped (safer)", "Targets specific file types", "Good for cleanup"],
                cons: &["Limited to Downloads folder", "Might delete wanted files"],
                safety_level: SafetyRating::Moderate,
                performance: "Medium",
                upvotes: 28,
                model: "Ollama CodeLlama",
            },
        ],
        _ => vec![],
    }
}

fn render_comparison(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    selected_idx: usize,
    view: &str,
) {
    let alternatives = get_alternatives(query);

    if view == "side-by-side" && alternatives.len() >= 2 {
        render_side_by_side(frame, area, query, &alternatives, selected_idx);
    } else {
        render_detailed(frame, area, query, &alternatives, selected_idx);
    }
}

fn render_side_by_side(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    alternatives: &[GeneratedAlternative],
    selected_idx: usize,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(5),     // Comparison area
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Comparing Alternatives for: ", Style::default().fg(Color::Cyan)),
            Span::styled(query, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(header, main_chunks[0]);

    // Split into columns for side-by-side comparison
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[1]);

    for (idx, (alt, column)) in alternatives.iter().take(2).zip(columns.iter()).enumerate() {
        render_alternative_panel(frame, *column, alt, idx == selected_idx, idx + 1);
    }
}

fn render_alternative_panel(
    frame: &mut Frame,
    area: Rect,
    alt: &GeneratedAlternative,
    is_selected: bool,
    number: usize,
) {
    let border_color = if is_selected { Color::Yellow } else { alt.safety_level.color() };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Command:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("$ ", Style::default().fg(Color::Green)),
            Span::raw(alt.command),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Safety: ", Style::default().fg(Color::White)),
            Span::styled(
                alt.safety_level.label(),
                Style::default().fg(alt.safety_level.color()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(Color::White)),
            Span::styled(alt.model, Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("Votes: ", Style::default().fg(Color::White)),
            Span::styled(format!("▲ {}", alt.upvotes), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Pros:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
    ];

    for pro in alt.pros {
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::raw(*pro),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Cons:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ]));

    for con in alt.cons {
        lines.push(Line::from(vec![
            Span::styled("  ✗ ", Style::default().fg(Color::Red)),
            Span::raw(*con),
        ]));
    }

    let title = if is_selected {
        format!("► Alternative {} (Selected)", number)
    } else {
        format!("Alternative {}", number)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    let widget = Paragraph::new(lines).block(block);
    frame.render_widget(widget, area);
}

fn render_detailed(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    alternatives: &[GeneratedAlternative],
    selected_idx: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Selected alternative detail
            Constraint::Length(8),   // Other alternatives list
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Query: ", Style::default().fg(Color::Cyan)),
            Span::styled(query, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Selected alternative detail
    if let Some(alt) = alternatives.get(selected_idx) {
        let mut detail_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Command:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("$ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(alt.command, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Explanation: ", Style::default().fg(Color::Cyan)),
                Span::raw(alt.explanation),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Safety: ", Style::default().fg(Color::White)),
                Span::styled(
                    alt.safety_level.label(),
                    Style::default().fg(alt.safety_level.color()).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  │  "),
                Span::styled("Performance: ", Style::default().fg(Color::White)),
                Span::styled(alt.performance, Style::default().fg(Color::Cyan)),
                Span::raw("  │  "),
                Span::styled("Model: ", Style::default().fg(Color::White)),
                Span::styled(alt.model, Style::default().fg(Color::Magenta)),
            ]),
        ];

        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Alternative {} of {}", selected_idx + 1, alternatives.len()))
            .border_style(Style::default().fg(Color::Yellow));

        let detail = Paragraph::new(detail_lines).block(detail_block);
        frame.render_widget(detail, chunks[1]);
    }

    // Other alternatives list
    let mut list_lines = Vec::new();

    for (idx, alt) in alternatives.iter().enumerate() {
        let is_current = idx == selected_idx;
        let marker = if is_current { "►" } else { " " };

        list_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", marker), Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:2}. ", idx + 1),
                if is_current {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(
                format!("{:50}", alt.command),
                if is_current {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled("  ", Style::default()),
            Span::styled(alt.safety_level.label(), Style::default().fg(alt.safety_level.color())),
        ]));
    }

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title("All Alternatives (↑↓ to navigate)");

    let list = Paragraph::new(list_lines).block(list_block);
    frame.render_widget(list, chunks[2]);
}

impl ShowcaseComponent for GenerationComparisonComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "GenerationComparison",
            "Side-by-side comparison of AI-generated command alternatives",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Side-by-Side: Find Files",
                "Compare two alternatives for finding large files",
                |frame, area| {
                    render_comparison(frame, area, "find large files", 0, "side-by-side");
                },
            ),
            ShowcaseStory::new(
                "Side-by-Side: Selected Alt 2",
                "Second alternative selected in comparison view",
                |frame, area| {
                    render_comparison(frame, area, "find large files", 1, "side-by-side");
                },
            ),
            ShowcaseStory::new(
                "Detailed View: Safe Command",
                "Detailed view showing safe alternative with explanation",
                |frame, area| {
                    render_comparison(frame, area, "find large files", 0, "detailed");
                },
            ),
            ShowcaseStory::new(
                "Detailed View: All Alternatives",
                "Shows all three alternatives with second selected",
                |frame, area| {
                    render_comparison(frame, area, "find large files", 1, "detailed");
                },
            ),
            ShowcaseStory::new(
                "Dangerous Command Warning",
                "Comparison highlighting dangerous alternative",
                |frame, area| {
                    render_comparison(frame, area, "delete temp files", 1, "detailed");
                },
            ),
            ShowcaseStory::new(
                "Safety Comparison",
                "Side-by-side showing safe vs risky alternatives",
                |frame, area| {
                    render_comparison(frame, area, "delete temp files", 0, "side-by-side");
                },
            ),
        ]
    }
}

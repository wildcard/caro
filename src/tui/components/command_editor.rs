//! Command editor component with line numbers and syntax highlighting
//!
//! Demonstrates an editable text area suitable for command editing and refinement

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CommandEditorComponent;

fn render_editor(
    frame: &mut Frame,
    area: Rect,
    content: &[&str],
    cursor_line: Option<usize>,
    show_line_numbers: bool,
    show_syntax_highlight: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    // Header
    let header = Paragraph::new("Command Editor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(header, chunks[0]);

    // Editor content
    let mut lines = Vec::new();

    for (idx, line_content) in content.iter().enumerate() {
        let line_num = idx + 1;
        let is_cursor_line = cursor_line == Some(line_num);

        let mut spans = Vec::new();

        // Line number
        if show_line_numbers {
            let line_num_style = if is_cursor_line {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            spans.push(Span::styled(format!(" {:3} │ ", line_num), line_num_style));
        } else if is_cursor_line {
            spans.push(Span::styled(" > ", Style::default().fg(Color::Yellow)));
        } else {
            spans.push(Span::raw("   "));
        }

        // Syntax highlighting
        if show_syntax_highlight {
            highlight_syntax(line_content, &mut spans, is_cursor_line);
        } else {
            let text_style = if is_cursor_line {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(*line_content, text_style));
        }

        // Cursor indicator
        if is_cursor_line {
            spans.push(Span::styled(" ▍", Style::default().fg(Color::Yellow)));
        }

        lines.push(Line::from(spans));
    }

    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title(if show_syntax_highlight {
            "Editor (Syntax Highlighting)"
        } else {
            "Editor (Plain)"
        })
        .border_style(if cursor_line.is_some() {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });

    let editor = Paragraph::new(lines).block(editor_block);
    frame.render_widget(editor, chunks[1]);

    // Footer with controls
    let footer_text = if cursor_line.is_some() {
        "↑↓: Navigate | Enter: Execute | Ctrl+E: Edit | Esc: Cancel"
    } else {
        "Press Ctrl+E to edit command"
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn highlight_syntax(text: &str, spans: &mut Vec<Span>, is_cursor_line: bool) {
    let base_brightness = if is_cursor_line { 1.2 } else { 1.0 };

    let mut chars = text.chars().peekable();
    let mut current_word = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            // String literals
            '"' | '\'' => {
                if !current_word.is_empty() {
                    spans.push(Span::styled(
                        current_word.clone(),
                        Style::default().fg(Color::White),
                    ));
                    current_word.clear();
                }

                let quote = ch;
                let mut string_content = String::from(ch);

                while let Some(&next_ch) = chars.peek() {
                    string_content.push(next_ch);
                    chars.next();
                    if next_ch == quote {
                        break;
                    }
                }

                spans.push(Span::styled(
                    string_content,
                    Style::default().fg(Color::Green),
                ));
            }
            // Pipes and redirections
            '|' | '>' | '<' | '&' => {
                if !current_word.is_empty() {
                    add_highlighted_word(&current_word, spans, base_brightness);
                    current_word.clear();
                }
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            // Options/flags
            '-' => {
                if !current_word.is_empty() {
                    add_highlighted_word(&current_word, spans, base_brightness);
                    current_word.clear();
                }
                current_word.push(ch);

                // Collect the flag
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '-' {
                        current_word.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                spans.push(Span::styled(
                    current_word.clone(),
                    Style::default().fg(Color::Cyan),
                ));
                current_word.clear();
            }
            // Whitespace
            ' ' | '\t' => {
                if !current_word.is_empty() {
                    add_highlighted_word(&current_word, spans, base_brightness);
                    current_word.clear();
                }
                spans.push(Span::raw(ch.to_string()));
            }
            // Regular characters
            _ => {
                current_word.push(ch);
            }
        }
    }

    if !current_word.is_empty() {
        add_highlighted_word(&current_word, spans, base_brightness);
    }
}

fn add_highlighted_word(word: &str, spans: &mut Vec<Span>, _brightness: f32) {
    // Keywords/commands
    let keywords = ["find", "grep", "sed", "awk", "ls", "cat", "cd", "rm", "cp", "mv"];

    let style = if keywords.contains(&word) {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if word.starts_with('/') || word.starts_with('.') {
        // Paths
        Style::default().fg(Color::Blue)
    } else if word.starts_with('$') {
        // Variables
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    spans.push(Span::styled(word.to_string(), style));
}

impl ShowcaseComponent for CommandEditorComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "CommandEditor",
            "Multi-line command editor with syntax highlighting and line numbers",
        )
        .with_category("Input")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Simple Command",
                "Single-line command in editor",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &["find . -name '*.pdf' -size +10M"],
                        None,
                        true,
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "Multi-line Pipeline",
                "Complex multi-line command with pipes",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "find . -name '*.rs' \\",
                            "  | grep -v target \\",
                            "  | xargs wc -l \\",
                            "  | sort -n",
                        ],
                        None,
                        true,
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "With Cursor - Line 1",
                "Editing mode with cursor on first line",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "find . -name '*.rs' \\",
                            "  | grep -v target \\",
                            "  | xargs wc -l",
                        ],
                        Some(1),
                        true,
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "With Cursor - Line 2",
                "Editing mode with cursor on second line",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "find . -name '*.rs' \\",
                            "  | grep -v target \\",
                            "  | xargs wc -l",
                        ],
                        Some(2),
                        true,
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "No Line Numbers",
                "Editor without line numbers",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "cat file.txt | sed 's/old/new/g' | grep pattern",
                        ],
                        None,
                        false,
                        true,
                    );
                },
            ),
            ShowcaseStory::new(
                "No Syntax Highlighting",
                "Plain text without syntax highlighting",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "find . -name '*.pdf'",
                            "ls -la /home/user",
                        ],
                        Some(1),
                        true,
                        false,
                    );
                },
            ),
            ShowcaseStory::new(
                "Complex Shell Script",
                "Multi-line shell script with various syntax elements",
                |frame, area| {
                    render_editor(
                        frame,
                        area,
                        &[
                            "for file in *.jpg; do",
                            "  convert \"$file\" -resize 800x600 \"thumb_$file\"",
                            "done",
                        ],
                        Some(2),
                        true,
                        true,
                    );
                },
            ),
        ]
    }
}

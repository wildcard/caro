//! Interactive TUI Demo for Caro
//!
//! This Ratzilla-powered WebAssembly application provides an authentic
//! terminal experience showcasing Caro's command generation and safety validation.

use rand::seq::SliceRandom;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use ratzilla::{DomBackend, WebRenderer};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// ============================================================================
// Demo Data
// ============================================================================

/// Sample prompts users can try
const SAMPLE_PROMPTS: &[(&str, &str, SafetyLevel)] = &[
    (
        "find large files over 100MB",
        "find . -type f -size +100M -exec ls -lh {} \\;",
        SafetyLevel::Safe,
    ),
    (
        "list all PDF files",
        "find . -name \"*.pdf\" -type f",
        SafetyLevel::Safe,
    ),
    (
        "show disk usage",
        "df -h",
        SafetyLevel::Safe,
    ),
    (
        "compress log files",
        "tar -czvf logs.tar.gz *.log",
        SafetyLevel::Safe,
    ),
    (
        "delete all temp files",
        "find /tmp -type f -mtime +7 -delete",
        SafetyLevel::Moderate,
    ),
    (
        "force remove directory",
        "rm -rf /important",
        SafetyLevel::Dangerous,
    ),
    (
        "change permissions recursively",
        "chmod -R 777 /",
        SafetyLevel::Critical,
    ),
    (
        "clean up old logs",
        "find /var/log -name \"*.log\" -mtime +30 -delete",
        SafetyLevel::Moderate,
    ),
    (
        "show running processes",
        "ps aux | head -20",
        SafetyLevel::Safe,
    ),
    (
        "search for pattern in files",
        "grep -r \"TODO\" --include=\"*.rs\" .",
        SafetyLevel::Safe,
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum SafetyLevel {
    Safe,
    Moderate,
    Dangerous,
    Critical,
}

impl SafetyLevel {
    fn color(&self) -> Color {
        match self {
            SafetyLevel::Safe => Color::Green,
            SafetyLevel::Moderate => Color::Yellow,
            SafetyLevel::Dangerous => Color::Rgb(255, 165, 0), // Orange
            SafetyLevel::Critical => Color::Red,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SafetyLevel::Safe => "SAFE",
            SafetyLevel::Moderate => "MODERATE",
            SafetyLevel::Dangerous => "DANGEROUS",
            SafetyLevel::Critical => "BLOCKED",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            SafetyLevel::Safe => "[OK]",
            SafetyLevel::Moderate => "[!!]",
            SafetyLevel::Dangerous => "[!!]",
            SafetyLevel::Critical => "[XX]",
        }
    }

    fn message(&self) -> &'static str {
        match self {
            SafetyLevel::Safe => "Command validated and safe to run",
            SafetyLevel::Moderate => "Requires confirmation before execution",
            SafetyLevel::Dangerous => "Warning: This command modifies system files",
            SafetyLevel::Critical => "BLOCKED: Dangerous pattern detected",
        }
    }
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
enum AppState {
    Welcome,
    Typing,
    Thinking,
    Result,
}

struct App {
    state: AppState,
    input: String,
    cursor_pos: usize,
    current_result: Option<(String, SafetyLevel)>,
    thinking_frame: usize,
    thinking_dots: usize,
    frame_count: u64,
    history: Vec<(String, String, SafetyLevel)>,
    selected_suggestion: usize,
    show_suggestions: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Welcome,
            input: String::new(),
            cursor_pos: 0,
            current_result: None,
            thinking_frame: 0,
            thinking_dots: 0,
            frame_count: 0,
            history: Vec::new(),
            selected_suggestion: 0,
            show_suggestions: true,
        }
    }
}

impl App {
    fn handle_key(&mut self, key: &str) {
        match self.state {
            AppState::Welcome => {
                self.state = AppState::Typing;
            }
            AppState::Typing => {
                match key {
                    "Enter" => {
                        if !self.input.is_empty() {
                            self.process_input();
                        }
                    }
                    "Backspace" => {
                        if self.cursor_pos > 0 {
                            self.input.remove(self.cursor_pos - 1);
                            self.cursor_pos -= 1;
                        }
                    }
                    "ArrowLeft" => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    }
                    "ArrowRight" => {
                        if self.cursor_pos < self.input.len() {
                            self.cursor_pos += 1;
                        }
                    }
                    "ArrowUp" => {
                        if self.show_suggestions && self.selected_suggestion > 0 {
                            self.selected_suggestion -= 1;
                        }
                    }
                    "ArrowDown" => {
                        if self.show_suggestions {
                            self.selected_suggestion = (self.selected_suggestion + 1)
                                .min(SAMPLE_PROMPTS.len().saturating_sub(1));
                        }
                    }
                    "Tab" => {
                        // Auto-complete from suggestion
                        if self.show_suggestions {
                            self.input = SAMPLE_PROMPTS[self.selected_suggestion].0.to_string();
                            self.cursor_pos = self.input.len();
                        }
                    }
                    "Escape" => {
                        self.input.clear();
                        self.cursor_pos = 0;
                        self.show_suggestions = true;
                    }
                    _ if key.len() == 1 => {
                        self.input.insert(self.cursor_pos, key.chars().next().unwrap());
                        self.cursor_pos += 1;
                        self.show_suggestions = self.input.len() < 3;
                    }
                    _ => {}
                }
            }
            AppState::Thinking => {
                // Ignore input while thinking
            }
            AppState::Result => {
                match key {
                    "Enter" | "Escape" | " " => {
                        self.state = AppState::Typing;
                        self.input.clear();
                        self.cursor_pos = 0;
                        self.current_result = None;
                        self.show_suggestions = true;
                    }
                    _ => {}
                }
            }
        }
    }

    fn process_input(&mut self) {
        self.state = AppState::Thinking;
        self.thinking_frame = 0;
        self.thinking_dots = 0;
    }

    fn tick(&mut self) {
        self.frame_count += 1;

        if self.state == AppState::Thinking {
            self.thinking_frame += 1;

            // Update dots animation
            if self.thinking_frame % 10 == 0 {
                self.thinking_dots = (self.thinking_dots + 1) % 4;
            }

            // Simulate processing time (about 1.5 seconds)
            if self.thinking_frame > 45 {
                self.generate_result();
            }
        }
    }

    fn generate_result(&mut self) {
        let input_lower = self.input.to_lowercase();

        // Try to find a matching sample
        let result = SAMPLE_PROMPTS
            .iter()
            .find(|(prompt, _, _)| input_lower.contains(&prompt.to_lowercase()))
            .or_else(|| {
                // Fuzzy match on keywords
                SAMPLE_PROMPTS.iter().find(|(prompt, _, _)| {
                    let prompt_words: Vec<&str> = prompt.split_whitespace().collect();
                    let input_words: Vec<&str> = input_lower.split_whitespace().collect();
                    prompt_words.iter().any(|pw| {
                        input_words.iter().any(|iw| iw.contains(pw) || pw.contains(iw))
                    })
                })
            })
            .cloned();

        if let Some((prompt, command, safety)) = result {
            self.history.push((
                self.input.clone(),
                command.to_string(),
                safety,
            ));
            self.current_result = Some((command.to_string(), safety));
        } else {
            // Generate a generic safe response
            let generic_commands = [
                ("echo \"Processing your request...\"", SafetyLevel::Safe),
                ("ls -la", SafetyLevel::Safe),
                ("pwd", SafetyLevel::Safe),
            ];
            let cmd = generic_commands.choose(&mut rand::thread_rng()).unwrap();
            self.history.push((
                self.input.clone(),
                cmd.0.to_string(),
                cmd.1,
            ));
            self.current_result = Some((cmd.0.to_string(), cmd.1));
        }

        self.state = AppState::Result;
    }
}

// ============================================================================
// Rendering
// ============================================================================

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    draw_header(frame, chunks[0]);
    draw_main(frame, chunks[1], app);
    draw_status_bar(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" caro ", Style::default().fg(Color::Rgb(255, 140, 66)).bold()),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled("Interactive Shell Companion Demo", Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 140, 66)))
            .style(Style::default().bg(Color::Rgb(20, 20, 30))),
    );
    frame.render_widget(header, area);
}

fn draw_main(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(15, 15, 20)));

    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1)])
        .split(area)[0];

    match app.state {
        AppState::Welcome => draw_welcome(frame, inner),
        AppState::Typing => draw_typing(frame, inner, app),
        AppState::Thinking => draw_thinking(frame, inner, app),
        AppState::Result => draw_result(frame, inner, app),
    }
}

fn draw_welcome(frame: &mut Frame, area: Rect) {
    let dog_art = r#"
    / \__
   (    @\___    woof!
   /         O
  /   (_____/
 /_____/   U
"#;

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to Caro!",
            Style::default()
                .fg(Color::Rgb(255, 140, 66))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(dog_art, Style::default().fg(Color::Rgb(200, 160, 120)))),
        Line::from(""),
        Line::from(Span::styled(
            "Your loyal shell companion",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("any key", Style::default().fg(Color::Green).bold()),
            Span::styled(" to start typing a command description", Style::default().fg(Color::DarkGray)),
        ]),
    ]);

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_typing(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Prompt
            Constraint::Length(3),  // Input
            Constraint::Min(1),     // Suggestions or history
        ])
        .split(area);

    // Prompt
    let prompt = Paragraph::new(Line::from(vec![
        Span::styled("[?] ", Style::default().fg(Color::Rgb(255, 140, 66))),
        Span::styled(
            "Describe what you want to do:",
            Style::default().fg(Color::White),
        ),
    ]));
    frame.render_widget(prompt, chunks[0]);

    // Input line with cursor
    let cursor_visible = (app.frame_count / 15) % 2 == 0;
    let cursor = if cursor_visible { "_" } else { " " };

    let (before, after) = app.input.split_at(app.cursor_pos);
    let input_line = Line::from(vec![
        Span::styled("$ caro \"", Style::default().fg(Color::Green)),
        Span::styled(before, Style::default().fg(Color::Cyan)),
        Span::styled(cursor, Style::default().fg(Color::White).bold()),
        Span::styled(after, Style::default().fg(Color::Cyan)),
        Span::styled("\"", Style::default().fg(Color::Green)),
    ]);

    let input = Paragraph::new(input_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    frame.render_widget(input, chunks[1]);

    // Suggestions
    if app.show_suggestions && app.input.len() < 3 {
        draw_suggestions(frame, chunks[2], app);
    }
}

fn draw_suggestions(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![
        Line::from(Span::styled(
            "Try one of these prompts (use arrows to select, Tab to fill):",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    for (i, (prompt, _, safety)) in SAMPLE_PROMPTS.iter().take(6).enumerate() {
        let is_selected = i == app.selected_suggestion;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::White).bold()
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("\"{prompt}\""), style),
            Span::styled(" ", Style::default()),
            Span::styled(
                safety.icon(),
                Style::default().fg(safety.color()),
            ),
        ]));
    }

    let suggestions = Paragraph::new(lines);
    frame.render_widget(suggestions, area);
}

fn draw_thinking(frame: &mut Frame, area: Rect, app: &App) {
    let dots = ".".repeat(app.thinking_dots);
    let spinner_frames = ["|", "/", "-", "\\"];
    let spinner = spinner_frames[app.thinking_frame % 4];

    let text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("$ caro \"", Style::default().fg(Color::Green)),
            Span::styled(&app.input, Style::default().fg(Color::Cyan)),
            Span::styled("\"", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!(" {spinner} "),
                Style::default().fg(Color::Rgb(255, 140, 66)).bold(),
            ),
            Span::styled(
                format!("Analyzing request{dots:<3}"),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "   Detecting platform context...",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "   Validating safety patterns...",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn draw_result(frame: &mut Frame, area: Rect, app: &App) {
    let (command, safety) = app.current_result.as_ref().unwrap();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("$ caro \"", Style::default().fg(Color::Green)),
            Span::styled(&app.input, Style::default().fg(Color::Cyan)),
            Span::styled("\"", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Generated command:", Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(command, Style::default().fg(Color::Rgb(206, 145, 120))),
        ]),
        Line::from(""),
    ];

    // Safety badge
    let safety_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            format!(" {} ", safety.label()),
            Style::default()
                .fg(Color::Black)
                .bg(safety.color())
                .bold(),
        ),
        Span::styled(
            format!(" {}", safety.message()),
            Style::default().fg(safety.color()),
        ),
    ]);
    lines.push(safety_line);
    lines.push(Line::from(""));

    // Action prompt
    if *safety != SafetyLevel::Critical {
        lines.push(Line::from(vec![
            Span::styled(" Execute this command? ", Style::default().fg(Color::White)),
            Span::styled("[y/N]", Style::default().fg(Color::Green).bold()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                " This command has been blocked for your safety.",
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Press any key to try another prompt...",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.state {
        AppState::Welcome => "Press any key to start",
        AppState::Typing => "Enter: Generate | Tab: Complete | Esc: Clear",
        AppState::Thinking => "Processing...",
        AppState::Result => "Press any key to continue",
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" caro.sh ", Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 66)).bold()),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(help_text, Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Commands: {}", app.history.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(Color::Rgb(25, 25, 35))),
    );

    frame.render_widget(status, area);
}

// ============================================================================
// WASM Entry Point
// ============================================================================

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    let app = Rc::new(RefCell::new(App::default()));

    let backend = DomBackend::new()?;
    let renderer = WebRenderer::new(backend);

    // Keyboard event handling
    let app_clone = app.clone();
    renderer.on_key_event(move |key_event| {
        app_clone.borrow_mut().handle_key(&key_event.key());
    });

    // Animation loop
    let app_clone = app.clone();
    renderer.on_tick(move |frame| {
        let mut app = app_clone.borrow_mut();
        app.tick();
        draw(frame, &app);
    });

    renderer.run()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// For panic messages
mod console_error_panic_hook {
    use std::panic;

    pub fn set_once() {
        static SET: std::sync::Once = std::sync::Once::new();
        SET.call_once(|| {
            panic::set_hook(Box::new(|info| {
                let msg = info.to_string();
                web_sys::console::error_1(&msg.into());
            }));
        });
    }
}

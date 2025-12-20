//! UI rendering for interactive mode

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use super::app::{InteractiveApp, InteractiveConfig, MessageStyle};
use crate::cli::CliApp;

/// ASCII art logo for Caro
const LOGO: &str = r#"
   ██████╗ █████╗ ██████╗  ██████╗
  ██╔════╝██╔══██╗██╔══██╗██╔═══██╗
  ██║     ███████║██████╔╝██║   ██║
  ██║     ██╔══██║██╔══██╗██║   ██║
  ╚██████╗██║  ██║██║  ██║╚██████╔╝
   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ "#;

/// Version string
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the interactive TUI
pub async fn run_interactive(config: InteractiveConfig) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = InteractiveApp::new(config);

    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut InteractiveApp,
) -> anyhow::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events with a timeout to allow for async operations
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(app, key).await?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle key events
async fn handle_key_event(app: &mut InteractiveApp, key: KeyEvent) -> anyhow::Result<()> {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if app.is_processing {
                app.is_processing = false;
                app.add_output("Operation cancelled.".to_string(), MessageStyle::Warning);
            } else {
                app.should_quit = true;
            }
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            if app.input.is_empty() {
                app.should_quit = true;
            }
        }

        // Line editing
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => app.move_cursor_start(),
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => app.move_cursor_end(),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.input.clear();
            app.cursor_position = 0;
        }
        (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
            // Delete word backwards
            while app.cursor_position > 0
                && app.input.chars().nth(app.cursor_position - 1) == Some(' ') {
                app.delete_char();
            }
            while app.cursor_position > 0
                && app.input.chars().nth(app.cursor_position - 1) != Some(' ') {
                app.delete_char();
            }
        }

        // History navigation
        (KeyCode::Up, _) => app.history_previous(),
        (KeyCode::Down, _) => app.history_next(),

        // Cursor movement
        (KeyCode::Left, _) => app.move_cursor_left(),
        (KeyCode::Right, _) => app.move_cursor_right(),
        (KeyCode::Home, _) => app.move_cursor_start(),
        (KeyCode::End, _) => app.move_cursor_end(),

        // Deletion
        (KeyCode::Backspace, _) => app.delete_char(),
        (KeyCode::Delete, _) => app.delete_char_forward(),

        // Submit
        (KeyCode::Enter, _) => {
            if let Some(input) = app.submit() {
                process_input(app, &input).await?;
            }
        }

        // Character input
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.enter_char(c);
        }

        // Page up/down for scrolling
        (KeyCode::PageUp, _) => {
            for _ in 0..5 {
                app.scroll_up();
            }
        }
        (KeyCode::PageDown, _) => {
            let max_scroll = app.output_messages.len().saturating_sub(5) as u16;
            for _ in 0..5 {
                app.scroll_down(max_scroll);
            }
        }

        _ => {}
    }

    Ok(())
}

/// Process user input
async fn process_input(app: &mut InteractiveApp, input: &str) -> anyhow::Result<()> {
    // Check for slash commands
    if input.starts_with('/') {
        if app.handle_slash_command(input) {
            return Ok(());
        }
    }

    // Generate command
    app.is_processing = true;
    app.add_output(format!("> {}", input), MessageStyle::Normal);
    app.add_output("Generating command...".to_string(), MessageStyle::Info);

    // Use the CLI app to generate command
    match generate_command(input).await {
        Ok((command, explanation)) => {
            app.add_output("".to_string(), MessageStyle::Normal);
            app.add_output("Command:".to_string(), MessageStyle::Info);
            app.add_output(format!("  {}", command), MessageStyle::Command);
            if !explanation.is_empty() {
                app.add_output("".to_string(), MessageStyle::Normal);
                app.add_output("Explanation:".to_string(), MessageStyle::Info);
                app.add_output(format!("  {}", explanation), MessageStyle::Explanation);
            }
            app.add_activity(input.to_string(), Some(command));
        }
        Err(e) => {
            app.add_output(format!("Error: {}", e), MessageStyle::Error);
            app.add_activity(input.to_string(), None);
        }
    }

    app.is_processing = false;
    app.add_output("".to_string(), MessageStyle::Normal);

    Ok(())
}

/// Generate command using the backend
async fn generate_command(prompt: &str) -> anyhow::Result<(String, String)> {
    let cli_app = CliApp::new().await.map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create a simple args struct for generation
    let args = SimpleArgs {
        prompt: Some(prompt.to_string()),
        shell: None,
        safety: None,
        output: None,
        confirm: false,
        verbose: false,
        config_file: None,
        execute: false,
        dry_run: false,
        interactive: false,
    };

    let result = cli_app.run_with_args(args).await.map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok((result.generated_command, result.explanation))
}

/// Simple argument struct for command generation
struct SimpleArgs {
    prompt: Option<String>,
    shell: Option<String>,
    safety: Option<String>,
    output: Option<String>,
    confirm: bool,
    verbose: bool,
    config_file: Option<String>,
    execute: bool,
    dry_run: bool,
    interactive: bool,
}

impl crate::cli::IntoCliArgs for SimpleArgs {
    fn prompt(&self) -> Option<String> { self.prompt.clone() }
    fn shell(&self) -> Option<String> { self.shell.clone() }
    fn safety(&self) -> Option<String> { self.safety.clone() }
    fn output(&self) -> Option<String> { self.output.clone() }
    fn confirm(&self) -> bool { self.confirm }
    fn verbose(&self) -> bool { self.verbose }
    fn config_file(&self) -> Option<String> { self.config_file.clone() }
    fn execute(&self) -> bool { self.execute }
    fn dry_run(&self) -> bool { self.dry_run }
    fn interactive(&self) -> bool { self.interactive }
}

/// Render the UI
fn ui(f: &mut Frame, app: &InteractiveApp) {
    let size = f.area();

    // Check if we have enough space for the welcome screen
    let show_welcome = app.output_messages.is_empty() && app.recent_activity.is_empty();

    if show_welcome && size.height >= 20 && size.width >= 80 {
        render_welcome_screen(f, app, size);
    } else {
        render_main_screen(f, app, size);
    }
}

/// Render the welcome screen with ASCII art
fn render_welcome_screen(f: &mut Frame, app: &InteractiveApp, area: Rect) {
    // Main layout: header box and input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(14),  // Welcome box
            Constraint::Length(3),   // Welcome message
            Constraint::Min(3),      // Spacer + input
            Constraint::Length(3),   // Input area
            Constraint::Length(1),   // Help hint
        ])
        .split(area);

    // Welcome box with two columns
    let welcome_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" Caro v{} ", VERSION),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    let inner = welcome_box.inner(chunks[0]);
    f.render_widget(welcome_box, chunks[0]);

    // Split inner area into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left column: Logo and user info
    render_left_column(f, app, columns[0]);

    // Right column: Tips and recent activity
    render_right_column(f, app, columns[1]);

    // Welcome message
    let welcome_text = Paragraph::new(Line::from(vec![
        Span::raw("  Welcome to "),
        Span::styled("Caro", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - Natural Language to Shell Commands"),
    ]));
    f.render_widget(welcome_text, chunks[1]);

    // Input separator
    let separator = Paragraph::new(Line::from("─".repeat(area.width.saturating_sub(4) as usize)))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(separator, Rect::new(chunks[3].x, chunks[3].y - 1, chunks[3].width, 1));

    // Input area
    render_input(f, app, chunks[3]);

    // Help hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  ? ", Style::default().fg(Color::Yellow)),
        Span::styled("for shortcuts", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(hint, chunks[4]);
}

/// Render the left column with logo and user info
fn render_left_column(f: &mut Frame, app: &InteractiveApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Logo
            Constraint::Length(1),  // Spacer
            Constraint::Min(2),     // User info
        ])
        .split(area);

    // Logo with styling
    let logo_lines: Vec<Line> = LOGO
        .lines()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Cyan))))
        .collect();
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    // User info
    let user_name = app.config.user_name.as_deref().unwrap_or("User");
    let cwd = app.config.working_directory.display().to_string();
    let cwd_short = if cwd.len() > 30 {
        format!("...{}", &cwd[cwd.len() - 27..])
    } else {
        cwd
    };

    let user_info = vec![
        Line::from(vec![
            Span::styled("caro ", Style::default().fg(Color::Gray)),
            Span::styled(format!("v{}", VERSION), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(format!("Welcome back, {}!", user_name), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(cwd_short, Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let user_paragraph = Paragraph::new(user_info).alignment(Alignment::Center);
    f.render_widget(user_paragraph, chunks[2]);
}

/// Render the right column with tips and recent activity
fn render_right_column(f: &mut Frame, app: &InteractiveApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Tips
            Constraint::Length(1),  // Spacer
            Constraint::Min(4),     // Recent activity
        ])
        .split(area);

    // Tips section
    let tips_header = Line::from(vec![
        Span::styled("Tips for getting started", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let current_tip = app.current_tip();
    let tips_content = vec![
        tips_header,
        Line::from(""),
        Line::from(Span::styled(current_tip, Style::default().fg(Color::Gray))),
    ];

    let tips = Paragraph::new(tips_content);
    f.render_widget(tips, chunks[0]);

    // Recent activity section
    let activity_header = Line::from(vec![
        Span::styled("Recent activity", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let mut activity_lines = vec![activity_header, Line::from("")];

    if app.recent_activity.is_empty() {
        activity_lines.push(Line::from(Span::styled(
            "No recent activity",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for entry in app.recent_activity.iter().rev().take(3) {
            let time = entry.timestamp.format("%H:%M").to_string();
            activity_lines.push(Line::from(vec![
                Span::styled(format!("[{}] ", time), Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.description, Style::default().fg(Color::Gray)),
            ]));
        }
    }

    let activity = Paragraph::new(activity_lines);
    f.render_widget(activity, chunks[2]);
}

/// Render the main screen (after welcome)
fn render_main_screen(f: &mut Frame, app: &InteractiveApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(5),     // Output
            Constraint::Length(1),  // Separator
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Help hint
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled("Caro ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("v{}", VERSION), Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", app.config.working_directory.display()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(header, chunks[0]);

    // Output area
    let output_lines: Vec<Line> = app
        .output_messages
        .iter()
        .map(|msg| {
            let style = match msg.style {
                MessageStyle::Normal => Style::default(),
                MessageStyle::Success => Style::default().fg(Color::Green),
                MessageStyle::Warning => Style::default().fg(Color::Yellow),
                MessageStyle::Error => Style::default().fg(Color::Red),
                MessageStyle::Info => Style::default().fg(Color::Blue),
                MessageStyle::Command => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                MessageStyle::Explanation => Style::default().fg(Color::Gray),
            };
            Line::from(Span::styled(&msg.content, style))
        })
        .collect();

    let output = Paragraph::new(output_lines)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));
    f.render_widget(output, chunks[1]);

    // Separator
    let separator = Paragraph::new(Line::from("─".repeat(area.width.saturating_sub(4) as usize)))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(separator, chunks[2]);

    // Input
    render_input(f, app, chunks[3]);

    // Help hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("  ? ", Style::default().fg(Color::Yellow)),
        Span::styled("for help  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+C ", Style::default().fg(Color::Yellow)),
        Span::styled("cancel  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+D ", Style::default().fg(Color::Yellow)),
        Span::styled("exit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(hint, chunks[4]);
}

/// Render the input area
fn render_input(f: &mut Frame, app: &InteractiveApp, area: Rect) {
    let input_text = if app.input.is_empty() && !app.is_processing {
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::styled(
                "Type a natural language description...",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&app.input),
        ])
    };

    let input = Paragraph::new(input_text);
    f.render_widget(input, area);

    // Show cursor
    if !app.is_processing {
        let cursor_x = area.x + 2 + app.cursor_position as u16;
        f.set_cursor_position((cursor_x, area.y));
    }
}

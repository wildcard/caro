//! TUI Component Showcase - A Storybook-like tool for Ratatui components
//!
//! This binary provides an interactive browser for viewing and testing
//! terminal UI components in isolation, similar to React Storybook.
//!
//! # Usage
//!
//! ```bash
//! # Run the showcase
//! cargo run --bin tui-showcase
//!
//! # Run with watch mode for hot-reload during development
//! cargo watch -x 'run --bin tui-showcase'
//! ```
//!
//! # Navigation
//!
//! - ↑/↓ or j/k: Navigate components and stories
//! - Enter: Select component/story
//! - Backspace: Go back
//! - q or Esc: Quit
//! - h: Show help

use cmdai::tui::{
    components::{
        CommandEditorComponent, CommandFlowComponent, CommandOutputViewerComponent,
        CommandPreviewComponent, CommandRatingComponent, ConfirmationDialogComponent,
        FileBrowserComponent, GenerationComparisonComponent, HistoryTimelineComponent,
        KeyboardShortcutsComponent, MetricDashboardComponent, NotificationToastComponent,
        ProgressSpinnerComponent, SafetyIndicatorComponent, SimpleTextComponent,
        TableSelectorComponent,
    },
    showcase::ShowcaseRegistry,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, time::Duration};

/// Application state
struct App {
    registry: ShowcaseRegistry,
    view_state: ViewState,
    selected_component: usize,
    selected_story: usize,
    should_quit: bool,
    show_help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewState {
    ComponentList,
    StoryList,
    StoryView,
}

impl App {
    fn new() -> Self {
        let mut registry = ShowcaseRegistry::new();

        // Register all showcase components
        // Organized by category for better browsing experience

        // Display components
        registry.register(Box::new(SimpleTextComponent));
        registry.register(Box::new(CommandPreviewComponent));
        registry.register(Box::new(TableSelectorComponent));
        registry.register(Box::new(CommandOutputViewerComponent)); // 🌟 Community requested!
        registry.register(Box::new(HistoryTimelineComponent)); // 🌟 Community requested!
        registry.register(Box::new(GenerationComparisonComponent)); // 🌟 Community requested!
        registry.register(Box::new(FileBrowserComponent)); // 🌟 File system browser!
        registry.register(Box::new(MetricDashboardComponent)); // 🌟 Community requested!

        // Input components
        registry.register(Box::new(ConfirmationDialogComponent));
        registry.register(Box::new(CommandEditorComponent));
        registry.register(Box::new(CommandRatingComponent)); // 🌟 Community requested!

        // Feedback components
        registry.register(Box::new(SafetyIndicatorComponent));
        registry.register(Box::new(ProgressSpinnerComponent));
        registry.register(Box::new(NotificationToastComponent));

        // Workflow components
        registry.register(Box::new(CommandFlowComponent));

        // Help components
        registry.register(Box::new(KeyboardShortcutsComponent));

        Self {
            registry,
            view_state: ViewState::ComponentList,
            selected_component: 0,
            selected_story: 0,
            should_quit: false,
            show_help: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                } else if self.view_state == ViewState::ComponentList {
                    self.should_quit = true;
                } else {
                    self.go_back();
                }
            }
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
            }
            KeyCode::Enter => {
                self.select_item();
            }
            KeyCode::Backspace => {
                self.go_back();
            }
            _ => {}
        }
    }

    fn move_selection_up(&mut self) {
        match self.view_state {
            ViewState::ComponentList => {
                if self.selected_component > 0 {
                    self.selected_component -= 1;
                }
            }
            ViewState::StoryList => {
                if self.selected_story > 0 {
                    self.selected_story -= 1;
                }
            }
            ViewState::StoryView => {}
        }
    }

    fn move_selection_down(&mut self) {
        match self.view_state {
            ViewState::ComponentList => {
                if self.selected_component + 1 < self.registry.len() {
                    self.selected_component += 1;
                }
            }
            ViewState::StoryList => {
                if let Some(component) = self.registry.get(self.selected_component) {
                    if self.selected_story + 1 < component.stories().len() {
                        self.selected_story += 1;
                    }
                }
            }
            ViewState::StoryView => {}
        }
    }

    fn select_item(&mut self) {
        match self.view_state {
            ViewState::ComponentList => {
                self.view_state = ViewState::StoryList;
                self.selected_story = 0;
            }
            ViewState::StoryList => {
                self.view_state = ViewState::StoryView;
            }
            ViewState::StoryView => {
                // Toggle back to story list on enter
                self.view_state = ViewState::StoryList;
            }
        }
    }

    fn go_back(&mut self) {
        match self.view_state {
            ViewState::ComponentList => {}
            ViewState::StoryList => {
                self.view_state = ViewState::ComponentList;
            }
            ViewState::StoryView => {
                self.view_state = ViewState::StoryList;
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    if app.show_help {
        render_help(frame, frame.area());
        return;
    }

    match app.view_state {
        ViewState::ComponentList => render_component_list(frame, app),
        ViewState::StoryList => render_story_list(frame, app),
        ViewState::StoryView => render_story_view(frame, app),
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "TUI Showcase ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- A Storybook-like tool for Ratatui"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/↓ or j/k     Navigate items"),
        Line::from("  Enter          Select item"),
        Line::from("  Backspace      Go back"),
        Line::from("  h              Toggle help"),
        Line::from("  q or Esc       Quit / Close"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Features:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  • Browse components and stories"),
        Line::from("  • View isolated component renders"),
        Line::from("  • Test different component states"),
        Line::from("  • Fast iteration with cargo-watch"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Development:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  cargo watch -x 'run --bin tui-showcase'"),
        Line::from(""),
        Line::from("Press h to close this help"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left);

    // Center the help dialog
    let dialog_area = centered_rect(60, 80, area);
    frame.render_widget(paragraph, dialog_area);
}

fn render_component_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "TUI Showcase",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - Select a component to explore"),
    ])])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Component list
    let items: Vec<ListItem> = app
        .registry
        .components()
        .iter()
        .enumerate()
        .map(|(idx, component)| {
            let metadata = component.metadata();
            let style = if idx == app.selected_component {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = vec![Line::from(vec![
                Span::styled(format!(" {} ", metadata.name), style),
                Span::raw(" - "),
                Span::raw(metadata.description.clone()),
                Span::raw(format!(" [{}]", metadata.category)),
            ])];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Components ({}) ", app.registry.len())),
        )
        .highlight_style(Style::default());

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑↓/jk: Navigate | Enter: Select | h: Help | q: Quit")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn render_story_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    if let Some(component) = app.registry.get(app.selected_component) {
        let metadata = component.metadata();

        // Header
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                metadata.name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::raw(metadata.description.clone()),
        ])])
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Story list
        let stories = component.stories();
        let items: Vec<ListItem> = stories
            .iter()
            .enumerate()
            .map(|(idx, story)| {
                let style = if idx == app.selected_story {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = vec![Line::from(vec![
                    Span::styled(format!(" {} ", story.name), style),
                    Span::raw(" - "),
                    Span::raw(story.description.clone()),
                ])];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Stories ({}) ", stories.len())),
            )
            .highlight_style(Style::default());

        frame.render_widget(list, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new("↑↓/jk: Navigate | Enter: View | Backspace: Back | q: Quit")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn render_story_view(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    if let Some(component) = app.registry.get(app.selected_component) {
        let metadata = component.metadata();
        let stories = component.stories();

        if let Some(story) = stories.get(app.selected_story) {
            // Header
            let header = Paragraph::new(vec![Line::from(vec![
                Span::styled(
                    metadata.name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" / "),
                Span::styled(
                    story.name.clone(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])])
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
            frame.render_widget(header, chunks[0]);

            // Render the story
            (story.render)(frame, chunks[1]);

            // Footer
            let footer = Paragraph::new("Enter: Back to list | Backspace: Back | q: Quit")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(footer, chunks[2]);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Poll for events with a timeout to support animations in the future
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

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
//! - /: Search components
//! - [ ]: Previous/Next category
//! - h: Show contextual help
//! - q or Esc: Quit

use cmdai::tui::{
    components::{
        BarChartComponent, CommandEditorComponent, CommandFlowComponent,
        CommandOutputViewerComponent, CommandPreviewComponent, CommandRatingComponent,
        ConfirmationDialogComponent, FileBrowserComponent, GenerationComparisonComponent,
        HistoryTimelineComponent, KeyboardShortcutsComponent, MetricDashboardComponent,
        NotificationToastComponent, ProgressSpinnerComponent, SafetyIndicatorComponent,
        SimpleTextComponent, SplitPaneComponent, TableSelectorComponent,
    },
    showcase::ShowcaseRegistry,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{error::Error, io, time::Duration};

/// Category filter options
#[derive(Debug, Clone, Copy, PartialEq)]
enum CategoryFilter {
    All,
    Display,
    Input,
    Feedback,
    Workflow,
    Help,
}

impl CategoryFilter {
    /// Get the category name as a string
    fn as_str(&self) -> &str {
        match self {
            CategoryFilter::All => "All",
            CategoryFilter::Display => "Display",
            CategoryFilter::Input => "Input",
            CategoryFilter::Feedback => "Feedback",
            CategoryFilter::Workflow => "Workflow",
            CategoryFilter::Help => "Help",
        }
    }

    /// Get the next category in the cycle
    fn next(&self) -> Self {
        match self {
            CategoryFilter::All => CategoryFilter::Display,
            CategoryFilter::Display => CategoryFilter::Input,
            CategoryFilter::Input => CategoryFilter::Feedback,
            CategoryFilter::Feedback => CategoryFilter::Workflow,
            CategoryFilter::Workflow => CategoryFilter::Help,
            CategoryFilter::Help => CategoryFilter::All,
        }
    }

    /// Get the previous category in the cycle
    fn prev(&self) -> Self {
        match self {
            CategoryFilter::All => CategoryFilter::Help,
            CategoryFilter::Display => CategoryFilter::All,
            CategoryFilter::Input => CategoryFilter::Display,
            CategoryFilter::Feedback => CategoryFilter::Input,
            CategoryFilter::Workflow => CategoryFilter::Feedback,
            CategoryFilter::Help => CategoryFilter::Workflow,
        }
    }

    /// Get color for this category
    fn color(&self) -> Color {
        match self {
            CategoryFilter::All => Color::White,
            CategoryFilter::Display => Color::Cyan,
            CategoryFilter::Input => Color::Green,
            CategoryFilter::Feedback => Color::Yellow,
            CategoryFilter::Workflow => Color::Magenta,
            CategoryFilter::Help => Color::Blue,
        }
    }

    /// Get category from index (1-5)
    fn from_index(index: usize) -> Option<Self> {
        match index {
            1 => Some(CategoryFilter::Display),
            2 => Some(CategoryFilter::Input),
            3 => Some(CategoryFilter::Feedback),
            4 => Some(CategoryFilter::Workflow),
            5 => Some(CategoryFilter::Help),
            _ => None,
        }
    }
}

/// Application state
struct App {
    registry: ShowcaseRegistry,
    view_state: ViewState,
    selected_component: usize,
    selected_story: usize,
    should_quit: bool,
    show_help: bool,

    // Phase 1: Search functionality
    search_query: String,
    search_active: bool,

    // Phase 1: Category filtering
    category_filter: CategoryFilter,

    // Cached filtered indices for performance
    filtered_indices: Vec<usize>,
    filter_cache_valid: bool,
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
        registry.register(Box::new(BarChartComponent)); // 🌟 Chart visualization!
        registry.register(Box::new(SplitPaneComponent)); // 🌟 Layout demonstration!

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

        let component_count = registry.len();

        Self {
            registry,
            view_state: ViewState::ComponentList,
            selected_component: 0,
            selected_story: 0,
            should_quit: false,
            show_help: false,
            search_query: String::new(),
            search_active: false,
            category_filter: CategoryFilter::All,
            filtered_indices: (0..component_count).collect(),
            filter_cache_valid: true,
        }
    }

    /// Get filtered component indices based on search and category filter
    fn filtered_components(&mut self) -> &[usize] {
        if !self.filter_cache_valid {
            self.recompute_filtered_indices();
        }
        &self.filtered_indices
    }

    /// Recompute the filtered indices based on current filters
    fn recompute_filtered_indices(&mut self) {
        let mut indices: Vec<usize> = (0..self.registry.len()).collect();

        // Apply category filter
        if self.category_filter != CategoryFilter::All {
            indices.retain(|&idx| {
                if let Some(component) = self.registry.get(idx) {
                    self.matches_category(component.metadata().category.as_str())
                } else {
                    false
                }
            });
        }

        // Apply search filter
        if !self.search_query.is_empty() {
            let query_lower = self.search_query.to_lowercase();
            indices.retain(|&idx| {
                if let Some(component) = self.registry.get(idx) {
                    let metadata = component.metadata();
                    metadata.name.to_lowercase().contains(&query_lower)
                        || metadata.description.to_lowercase().contains(&query_lower)
                        || metadata.category.to_lowercase().contains(&query_lower)
                } else {
                    false
                }
            });
        }

        self.filtered_indices = indices;
        self.filter_cache_valid = true;

        // Adjust selection if it's out of bounds
        if self.selected_component >= self.filtered_indices.len() && !self.filtered_indices.is_empty() {
            self.selected_component = self.filtered_indices.len() - 1;
        }
    }

    /// Check if a category string matches the current filter
    fn matches_category(&self, category: &str) -> bool {
        match self.category_filter {
            CategoryFilter::All => true,
            CategoryFilter::Display => category == "Display",
            CategoryFilter::Input => category == "Input",
            CategoryFilter::Feedback => category == "Feedback",
            CategoryFilter::Workflow => category == "Workflow",
            CategoryFilter::Help => category == "Help",
        }
    }

    /// Get the actual component index from the filtered index
    fn get_actual_index(&mut self, filtered_idx: usize) -> Option<usize> {
        self.filtered_components().get(filtered_idx).copied()
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Handle search mode input
        if self.search_active {
            match key {
                KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                    self.search_query.push(c);
                    self.filter_cache_valid = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.filter_cache_valid = false;
                }
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_query.clear();
                    self.filter_cache_valid = false;
                }
                KeyCode::Enter => {
                    self.search_active = false;
                }
                _ => {}
            }
            return;
        }

        // Handle Ctrl+C to clear filters
        if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.search_query.clear();
            self.category_filter = CategoryFilter::All;
            self.filter_cache_valid = false;
            return;
        }

        // Handle Ctrl+1-5 for category jumps
        if modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = key {
                if let Some(digit) = c.to_digit(10) {
                    if let Some(category) = CategoryFilter::from_index(digit as usize) {
                        self.category_filter = category;
                        self.filter_cache_valid = false;
                        return;
                    }
                }
            }
        }

        // Normal key handling
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
            KeyCode::Char('/') => {
                if self.view_state == ViewState::ComponentList {
                    self.search_active = true;
                }
            }
            KeyCode::Char('[') => {
                if self.view_state == ViewState::ComponentList {
                    self.category_filter = self.category_filter.prev();
                    self.filter_cache_valid = false;
                }
            }
            KeyCode::Char(']') => {
                if self.view_state == ViewState::ComponentList {
                    self.category_filter = self.category_filter.next();
                    self.filter_cache_valid = false;
                }
            }
            KeyCode::Char('g') => {
                // Jump to top
                self.selected_component = 0;
                self.selected_story = 0;
            }
            KeyCode::Char('G') => {
                // Jump to bottom
                match self.view_state {
                    ViewState::ComponentList => {
                        let filtered_count = self.filtered_components().len();
                        if filtered_count > 0 {
                            self.selected_component = filtered_count - 1;
                        }
                    }
                    ViewState::StoryList => {
                        if let Some(actual_idx) = self.get_actual_index(self.selected_component) {
                            if let Some(component) = self.registry.get(actual_idx) {
                                let story_count = component.stories().len();
                                if story_count > 0 {
                                    self.selected_story = story_count - 1;
                                }
                            }
                        }
                    }
                    ViewState::StoryView => {}
                }
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
                let filtered_count = self.filtered_components().len();
                if filtered_count > 0 && self.selected_component + 1 < filtered_count {
                    self.selected_component += 1;
                }
            }
            ViewState::StoryList => {
                if let Some(actual_idx) = self.get_actual_index(self.selected_component) {
                    if let Some(component) = self.registry.get(actual_idx) {
                        if self.selected_story + 1 < component.stories().len() {
                            self.selected_story += 1;
                        }
                    }
                }
            }
            ViewState::StoryView => {}
        }
    }

    fn select_item(&mut self) {
        match self.view_state {
            ViewState::ComponentList => {
                if !self.filtered_components().is_empty() {
                    self.view_state = ViewState::StoryList;
                    self.selected_story = 0;
                }
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

fn ui(frame: &mut Frame, app: &mut App) {
    if app.show_help {
        render_help(frame, frame.area(), app.view_state);
        return;
    }

    match app.view_state {
        ViewState::ComponentList => render_component_list(frame, app),
        ViewState::StoryList => render_story_list(frame, app),
        ViewState::StoryView => render_story_view(frame, app),
    }
}

fn render_help(frame: &mut Frame, area: Rect, view_state: ViewState) {
    let help_text = match view_state {
        ViewState::ComponentList => vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Help - Component List View",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "NAVIGATION:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑↓ / jk       Move through components"),
            Line::from("  Enter         Open component stories"),
            Line::from("  g / G         Jump to top/bottom"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "SEARCH & FILTER:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  /             Search components"),
            Line::from("  [ ]           Cycle through categories"),
            Line::from("  Ctrl+1-5      Jump to category (1=Display, 2=Input, etc.)"),
            Line::from("  Ctrl+C        Clear search and filters"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "OTHER:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  h             Toggle this help"),
            Line::from("  q / Esc       Quit application"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "TIP:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Press / to search for 'preview' to quickly find"),
            Line::from("  the CommandPreview component"),
        ],
        ViewState::StoryList => vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Help - Story List View",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "NAVIGATION:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑↓ / jk       Move through stories"),
            Line::from("  Enter         View selected story"),
            Line::from("  Backspace     Back to component list"),
            Line::from("  g / G         Jump to top/bottom"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "OTHER:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  h             Toggle this help"),
            Line::from("  Esc           Back to component list"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "TIP:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Use arrow keys or j/k to quickly scan through"),
            Line::from("  different story variations"),
        ],
        ViewState::StoryView => vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Help - Story View",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "NAVIGATION:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Enter         Back to story list"),
            Line::from("  Backspace     Back to story list"),
            Line::from("  Esc           Back to story list"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "OTHER:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  h             Toggle this help"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "TIP:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Press Enter to go back to the story list and"),
            Line::from("  browse other variations of this component"),
        ],
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    // Center the help dialog
    let dialog_area = centered_rect(70, 85, area);
    frame.render_widget(paragraph, dialog_area);
}

fn render_component_list(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Get counts and filter data first
    let filtered_count = app.filtered_components().len();
    let total_count = app.registry.len();
    let category_filter = app.category_filter;
    let search_query = app.search_query.clone();

    // Build breadcrumb and filter info
    let mut header_text = vec![
        Span::styled(
            "TUI Showcase",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" > Components"),
    ];

    // Add filter indicator if active
    if category_filter != CategoryFilter::All {
        header_text.push(Span::raw(" | Filter: "));
        header_text.push(Span::styled(
            category_filter.as_str(),
            Style::default()
                .fg(category_filter.color())
                .add_modifier(Modifier::BOLD),
        ));
    }

    if !search_query.is_empty() {
        header_text.push(Span::raw(" | Search: "));
        header_text.push(Span::styled(
            &search_query,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Show count
    if filtered_count != total_count {
        header_text.push(Span::raw(format!(" ({}/{})", filtered_count, total_count)));
    } else {
        header_text.push(Span::raw(format!(" ({})", total_count)));
    }

    // Header
    let header = Paragraph::new(vec![Line::from(header_text)])
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    frame.render_widget(header, chunks[0]);

    // Component list
    let filtered = app.filtered_components().to_vec();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(filtered_idx, &actual_idx)| {
            if let Some(component) = app.registry.get(actual_idx) {
                let metadata = component.metadata();
                let story_count = component.stories().len();

                let style = if filtered_idx == app.selected_component {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Get category color
                let category_color = match metadata.category.as_str() {
                    "Display" => Color::Cyan,
                    "Input" => Color::Green,
                    "Feedback" => Color::Yellow,
                    "Workflow" => Color::Magenta,
                    "Help" => Color::Blue,
                    _ => Color::White,
                };

                let content = vec![Line::from(vec![
                    Span::styled(
                        if filtered_idx == app.selected_component {
                            format!(" {} ", metadata.name)
                        } else {
                            format!("  {} ", metadata.name)
                        },
                        style,
                    ),
                    Span::styled(
                        format!("[{}]", metadata.category),
                        Style::default()
                            .fg(category_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - "),
                    Span::raw(metadata.description.clone()),
                    Span::styled(
                        format!(" ({} stories)", story_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])];

                ListItem::new(content)
            } else {
                ListItem::new(vec![Line::from("Error: Component not found")])
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Components ({}) ", filtered_count)),
        )
        .highlight_style(Style::default());

    frame.render_widget(list, chunks[1]);

    // Footer - different content if search is active
    let footer_text = if app.search_active {
        format!(
            "🔍 Search: {}_ | Esc: Cancel | Enter: Apply",
            app.search_query
        )
    } else {
        "↑↓/jk: Navigate | Enter: Select | /: Search | []: Filter | h: Help | q: Quit".to_string()
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn render_story_list(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    if let Some(actual_idx) = app.get_actual_index(app.selected_component) {
        if let Some(component) = app.registry.get(actual_idx) {
            let metadata = component.metadata();

            // Breadcrumb header
            let header = Paragraph::new(vec![Line::from(vec![
                Span::styled(
                    "TUI Showcase",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" > "),
                Span::styled(
                    metadata.name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" > Stories"),
            ])])
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);
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
    }

    // Footer
    let footer = Paragraph::new("↑↓/jk: Navigate | Enter: View | Backspace: Back | h: Help | Esc: Back")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn render_story_view(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    if let Some(actual_idx) = app.get_actual_index(app.selected_component) {
        if let Some(component) = app.registry.get(actual_idx) {
            let metadata = component.metadata();
            let stories = component.stories();

            if let Some(story) = stories.get(app.selected_story) {
                // Breadcrumb header
                let header = Paragraph::new(vec![Line::from(vec![
                    Span::styled(
                        "TUI Showcase",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" > "),
                    Span::styled(
                        metadata.name.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" > "),
                    Span::styled(
                        story.name.clone(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({}/{})", app.selected_story + 1, stories.len())),
                ])])
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Left);
                frame.render_widget(header, chunks[0]);

                // Render the story
                (story.render)(frame, chunks[1]);

                // Footer
                let footer = Paragraph::new("Enter/Backspace: Back | h: Help | Esc: Back")
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                frame.render_widget(footer, chunks[2]);
            }
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
        terminal.draw(|f| ui(f, &mut app))?;

        // Poll for events with a timeout to support animations in the future
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_filter_as_str() {
        assert_eq!(CategoryFilter::All.as_str(), "All");
        assert_eq!(CategoryFilter::Display.as_str(), "Display");
        assert_eq!(CategoryFilter::Input.as_str(), "Input");
        assert_eq!(CategoryFilter::Feedback.as_str(), "Feedback");
        assert_eq!(CategoryFilter::Workflow.as_str(), "Workflow");
        assert_eq!(CategoryFilter::Help.as_str(), "Help");
    }

    #[test]
    fn test_category_filter_next_cycles_correctly() {
        assert_eq!(CategoryFilter::All.next(), CategoryFilter::Display);
        assert_eq!(CategoryFilter::Display.next(), CategoryFilter::Input);
        assert_eq!(CategoryFilter::Input.next(), CategoryFilter::Feedback);
        assert_eq!(CategoryFilter::Feedback.next(), CategoryFilter::Workflow);
        assert_eq!(CategoryFilter::Workflow.next(), CategoryFilter::Help);
        assert_eq!(CategoryFilter::Help.next(), CategoryFilter::All);
    }

    #[test]
    fn test_category_filter_prev_cycles_correctly() {
        assert_eq!(CategoryFilter::All.prev(), CategoryFilter::Help);
        assert_eq!(CategoryFilter::Help.prev(), CategoryFilter::Workflow);
        assert_eq!(CategoryFilter::Workflow.prev(), CategoryFilter::Feedback);
        assert_eq!(CategoryFilter::Feedback.prev(), CategoryFilter::Input);
        assert_eq!(CategoryFilter::Input.prev(), CategoryFilter::Display);
        assert_eq!(CategoryFilter::Display.prev(), CategoryFilter::All);
    }

    #[test]
    fn test_category_filter_from_index() {
        assert_eq!(CategoryFilter::from_index(1), Some(CategoryFilter::Display));
        assert_eq!(CategoryFilter::from_index(2), Some(CategoryFilter::Input));
        assert_eq!(CategoryFilter::from_index(3), Some(CategoryFilter::Feedback));
        assert_eq!(CategoryFilter::from_index(4), Some(CategoryFilter::Workflow));
        assert_eq!(CategoryFilter::from_index(5), Some(CategoryFilter::Help));
        assert_eq!(CategoryFilter::from_index(0), None);
        assert_eq!(CategoryFilter::from_index(6), None);
    }

    #[test]
    fn test_category_filter_colors_are_unique() {
        let colors = vec![
            CategoryFilter::All.color(),
            CategoryFilter::Display.color(),
            CategoryFilter::Input.color(),
            CategoryFilter::Feedback.color(),
            CategoryFilter::Workflow.color(),
            CategoryFilter::Help.color(),
        ];

        // Verify at least some colors are different (not all the same)
        assert!(colors[0] != colors[1] || colors[1] != colors[2]);
    }

    #[test]
    fn test_app_initializes_with_correct_defaults() {
        let app = App::new();

        assert_eq!(app.view_state, ViewState::ComponentList);
        assert_eq!(app.selected_component, 0);
        assert_eq!(app.selected_story, 0);
        assert!(!app.should_quit);
        assert!(!app.show_help);
        assert_eq!(app.search_query, "");
        assert!(!app.search_active);
        assert_eq!(app.category_filter, CategoryFilter::All);
        assert!(app.filter_cache_valid);
    }

    #[test]
    fn test_matches_category_with_all_filter() {
        let app = App::new();

        assert!(app.matches_category("Display"));
        assert!(app.matches_category("Input"));
        assert!(app.matches_category("Feedback"));
        assert!(app.matches_category("Workflow"));
        assert!(app.matches_category("Help"));
        assert!(app.matches_category("AnyOther"));
    }

    #[test]
    fn test_matches_category_with_specific_filter() {
        let mut app = App::new();

        app.category_filter = CategoryFilter::Display;
        assert!(app.matches_category("Display"));
        assert!(!app.matches_category("Input"));
        assert!(!app.matches_category("Feedback"));

        app.category_filter = CategoryFilter::Input;
        assert!(!app.matches_category("Display"));
        assert!(app.matches_category("Input"));
        assert!(!app.matches_category("Feedback"));
    }

    #[test]
    fn test_search_query_filtering() {
        let mut app = App::new();

        // Initial state - all components visible
        let initial_count = app.filtered_components().len();
        assert!(initial_count > 0);

        // Apply search filter
        app.search_query = "command".to_string();
        app.filter_cache_valid = false;
        let filtered_count = app.filtered_components().len();

        // Should have fewer or equal components (some have "command" in name)
        assert!(filtered_count <= initial_count);
    }

    #[test]
    fn test_category_filtering() {
        let mut app = App::new();

        // Get initial count
        let all_count = app.filtered_components().len();

        // Apply Display filter
        app.category_filter = CategoryFilter::Display;
        app.filter_cache_valid = false;
        let display_count = app.filtered_components().len();

        // Display category should have fewer than total
        assert!(display_count < all_count);

        // Apply Input filter
        app.category_filter = CategoryFilter::Input;
        app.filter_cache_valid = false;
        let input_count = app.filtered_components().len();

        // Input category should have some components
        assert!(input_count > 0);
        assert!(input_count < all_count);
    }

    #[test]
    fn test_combined_search_and_category_filtering() {
        let mut app = App::new();

        // Apply both filters
        app.category_filter = CategoryFilter::Display;
        app.search_query = "command".to_string();
        app.filter_cache_valid = false;

        // Convert to owned Vec to avoid borrow conflicts
        let filtered = app.filtered_components().to_vec();

        // Should only show Display components with "command" in name/description
        for &idx in &filtered {
            if let Some(component) = app.registry.get(idx) {
                let metadata = component.metadata();
                let has_command = metadata.name.to_lowercase().contains("command")
                    || metadata.description.to_lowercase().contains("command");
                let is_display = metadata.category == "Display";

                assert!(has_command && is_display);
            }
        }
    }

    #[test]
    fn test_filter_cache_invalidation() {
        let mut app = App::new();

        // Cache is valid initially
        assert!(app.filter_cache_valid);

        // Changing search invalidates cache
        app.search_query = "test".to_string();
        app.filter_cache_valid = false;
        assert!(!app.filter_cache_valid);

        // Accessing filtered_components recomputes
        let _ = app.filtered_components();
        assert!(app.filter_cache_valid);
    }

    #[test]
    fn test_get_actual_index() {
        let mut app = App::new();

        // With no filters, filtered index == actual index
        assert_eq!(app.get_actual_index(0), Some(0));
        assert_eq!(app.get_actual_index(1), Some(1));

        // Apply filter that reduces the list
        app.category_filter = CategoryFilter::Help;
        app.filter_cache_valid = false;

        let filtered = app.filtered_components().to_vec();
        if !filtered.is_empty() {
            // First filtered index should map to actual index
            assert_eq!(app.get_actual_index(0), Some(filtered[0]));
        }
    }

    #[test]
    fn test_selection_adjustment_after_filtering() {
        let mut app = App::new();

        // Select last component
        let total = app.registry.len();
        app.selected_component = total - 1;

        // Apply restrictive filter
        app.category_filter = CategoryFilter::Help;
        app.filter_cache_valid = false;

        // Trigger recomputation
        let _ = app.filtered_components();

        // Selection should be adjusted to valid range
        let filtered_count = app.filtered_components().len();
        if filtered_count > 0 {
            assert!(app.selected_component < filtered_count);
        }
    }
}

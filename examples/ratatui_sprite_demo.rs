//! Comprehensive Ratatui sprite animation demo
//!
//! This demo showcases integration of cmdai's sprite animation system with Ratatui,
//! featuring multiple animated sprites, keyboard controls, and performance metrics.
//!
//! Run with: cargo run --example ratatui_sprite_demo --features tui
//!
//! Controls:
//!   - Space: Pause/Resume all animations
//!   - +/=: Speed up animations
//!   - -: Slow down animations
//!   - R: Reset all animations
//!   - 1-5: Toggle individual sprites
//!   - Q/Esc: Quit

use cmdai::rendering::{
    examples::*, ratatui_widget::*, AnimationMode, Color, ColorPalette, Sprite, SpriteFrame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color as RatatuiColor, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

/// Application state
struct App {
    scene: SpriteScene,
    paused: bool,
    speed: f32,
    fps_counter: FpsCounter,
    sprite_names: Vec<String>,
    sprite_enabled: Vec<bool>,
    last_tick: Instant,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut scene = SpriteScene::new();
        let mut sprite_names = Vec::new();
        let mut sprite_enabled = Vec::new();

        // Add walking character (center-left)
        let walking = create_walking_animation()?;
        sprite_names.push("Walking Character".to_string());
        sprite_enabled.push(true);
        scene.add(AnimatedSprite::new(
            walking,
            AnimationMode::Loop,
            10,
            10,
        ));

        // Add heart animation (top-right)
        let heart = create_heart_animation()?;
        sprite_names.push("Heart Pulse".to_string());
        sprite_enabled.push(true);
        scene.add(AnimatedSprite::new(heart, AnimationMode::Loop, 60, 5));

        // Add spinning coin (center)
        let coin = create_coin_animation()?;
        sprite_names.push("Spinning Coin".to_string());
        sprite_enabled.push(true);
        scene.add(AnimatedSprite::new(coin, AnimationMode::Loop, 35, 12));

        // Add spinner (bottom-left)
        let spinner = create_spinner_animation()?;
        sprite_names.push("Spinner".to_string());
        sprite_enabled.push(true);
        scene.add(AnimatedSprite::new(
            spinner,
            AnimationMode::Loop,
            15,
            20,
        ));

        // Add idle character (bottom-right)
        let idle = create_idle_character()?;
        sprite_names.push("Idle Character".to_string());
        sprite_enabled.push(true);
        scene.add(AnimatedSprite::new(idle, AnimationMode::Loop, 55, 18));

        Ok(Self {
            scene,
            paused: false,
            speed: 1.0,
            fps_counter: FpsCounter::new(60),
            sprite_names,
            sprite_enabled,
            last_tick: Instant::now(),
        })
    }

    fn tick(&mut self) {
        if !self.paused {
            self.scene.update();
        }
        self.fps_counter.tick();
        self.last_tick = Instant::now();
    }

    fn pause_toggle(&mut self) {
        self.paused = !self.paused;
    }

    fn speed_up(&mut self) {
        self.speed = (self.speed * 1.5).min(10.0);
        self.update_sprite_speeds();
    }

    fn slow_down(&mut self) {
        self.speed = (self.speed / 1.5).max(0.1);
        self.update_sprite_speeds();
    }

    fn reset(&mut self) {
        self.speed = 1.0;
        self.update_sprite_speeds();
        for i in 0..self.scene.len() {
            if let Some(sprite) = self.scene.get_mut(i) {
                sprite.controller_mut().reset();
            }
        }
    }

    fn update_sprite_speeds(&mut self) {
        for i in 0..self.scene.len() {
            if let Some(sprite) = self.scene.get_mut(i) {
                sprite.controller_mut().set_speed(self.speed);
            }
        }
    }

    fn toggle_sprite(&mut self, index: usize) {
        if index < self.sprite_enabled.len() {
            self.sprite_enabled[index] = !self.sprite_enabled[index];
            if let Some(sprite) = self.scene.get_mut(index) {
                sprite.set_visible(self.sprite_enabled[index]);
            }
        }
    }
}

/// FPS counter for performance monitoring
struct FpsCounter {
    frames: Vec<Instant>,
    max_samples: usize,
}

impl FpsCounter {
    fn new(max_samples: usize) -> Self {
        Self {
            frames: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    fn tick(&mut self) {
        self.frames.push(Instant::now());
        if self.frames.len() > self.max_samples {
            self.frames.remove(0);
        }
    }

    fn fps(&self) -> f64 {
        if self.frames.len() < 2 {
            return 0.0;
        }

        let elapsed = self.frames.last().unwrap().duration_since(self.frames[0]);
        let secs = elapsed.as_secs_f64();

        if secs == 0.0 {
            return 0.0;
        }

        (self.frames.len() - 1) as f64 / secs
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Run app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(16); // ~60 FPS target
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char(' ') => app.pause_toggle(),
                    KeyCode::Char('+') | KeyCode::Char('=') => app.speed_up(),
                    KeyCode::Char('-') => app.slow_down(),
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('1') => app.toggle_sprite(0),
                    KeyCode::Char('2') => app.toggle_sprite(1),
                    KeyCode::Char('3') => app.toggle_sprite(2),
                    KeyCode::Char('4') => app.toggle_sprite(3),
                    KeyCode::Char('5') => app.toggle_sprite(4),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main area
            Constraint::Length(8), // Controls + Stats
        ])
        .split(f.size());

    // Title
    render_title(f, chunks[0]);

    // Main sprite area
    render_sprites(f, chunks[1], app);

    // Bottom panel (controls and stats)
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    render_controls(f, bottom_chunks[0]);
    render_stats(f, bottom_chunks[1], app);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("Ratatui Sprite Animation Demo")
        .style(
            Style::default()
                .fg(RatatuiColor::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_sprites(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("Animation Scene")
        .borders(Borders::ALL)
        .style(Style::default().fg(RatatuiColor::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render all sprites to the buffer
    let buf = f.buffer_mut();
    app.scene.render(buf);

    // Add pause indicator if paused
    if app.paused {
        let pause_text = Paragraph::new("⏸ PAUSED")
            .style(
                Style::default()
                    .fg(RatatuiColor::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);

        let pause_area = Rect {
            x: inner.x + inner.width / 2 - 6,
            y: inner.y + 1,
            width: 12,
            height: 1,
        };
        f.render_widget(pause_text, pause_area);
    }
}

fn render_controls(f: &mut Frame, area: Rect) {
    let controls = vec![
        Line::from(vec![
            Span::styled("Space", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(": Pause/Resume  "),
            Span::styled("+/-", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(": Speed  "),
            Span::styled("R", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(": Reset"),
        ]),
        Line::from(vec![
            Span::styled("1-5", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(": Toggle Sprites  "),
            Span::styled("Q/Esc", Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];

    let controls_widget = Paragraph::new(controls)
        .block(
            Block::default()
                .title("Controls")
                .borders(Borders::ALL)
                .style(Style::default().fg(RatatuiColor::White)),
        )
        .alignment(Alignment::Left);

    f.render_widget(controls_widget, area);
}

fn render_stats(f: &mut Frame, area: Rect, app: &App) {
    let fps = app.fps_counter.fps();
    let fps_color = if fps >= 50.0 {
        RatatuiColor::Green
    } else if fps >= 30.0 {
        RatatuiColor::Yellow
    } else {
        RatatuiColor::Red
    };

    let stats = vec![
        Line::from(vec![
            Span::raw("FPS: "),
            Span::styled(
                format!("{:.1}", fps),
                Style::default().fg(fps_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Speed: "),
            Span::styled(
                format!("{:.1}x", app.speed),
                Style::default().fg(RatatuiColor::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Sprites: "),
            Span::styled(
                format!(
                    "{}/{}",
                    app.sprite_enabled.iter().filter(|&&e| e).count(),
                    app.sprite_enabled.len()
                ),
                Style::default().fg(RatatuiColor::Green),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats)
        .block(
            Block::default()
                .title("Stats")
                .borders(Borders::ALL)
                .style(Style::default().fg(RatatuiColor::White)),
        )
        .alignment(Alignment::Left);

    f.render_widget(stats_widget, area);
}

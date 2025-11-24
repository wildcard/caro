# TUI Framework Integration Guide

This guide shows how to integrate cmdai's sprite animation system with popular Rust TUI (Terminal User Interface) frameworks, with a focus on [Ratatui](https://github.com/ratatui-org/ratatui) (formerly tui-rs), the most popular terminal UI library in the Rust ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Why Ratatui?](#why-ratatui)
3. [Quick Start](#quick-start)
4. [Architecture Patterns](#architecture-patterns)
5. [Widget Implementations](#widget-implementations)
6. [Event Handling](#event-handling)
7. [Performance Considerations](#performance-considerations)
8. [Complete Examples](#complete-examples)
9. [Advanced Integration](#advanced-integration)
10. [Troubleshooting](#troubleshooting)

## Overview

cmdai's sprite animation system provides a powerful foundation for rendering animated pixel art in terminal applications. When combined with Ratatui's widget system, you can create rich, interactive terminal UIs with animated sprites.

**Key Benefits:**
- Clean separation between rendering and application logic
- Efficient frame-based animations with precise timing
- Full color support (true color and 256-color modes)
- Transparency and compositing support
- Event-driven architecture for interactive applications

**Integration Approaches:**
1. **Simple Widget** - Render static sprites or current animation frame
2. **Animated Widget** - Self-updating widget with internal state
3. **Managed Animation** - Application-controlled animation lifecycle
4. **Multi-Sprite Composition** - Multiple animated sprites in one view

## Why Ratatui?

Ratatui is the most mature and actively maintained TUI framework for Rust, offering:

- **Immediate Mode Rendering**: Redraw the entire UI on each frame
- **Cross-Platform**: Works on Linux, macOS, Windows
- **Layout System**: Flexible constraint-based layouts
- **Widget Library**: Rich set of pre-built widgets
- **Backend Agnostic**: Supports crossterm, termion, and termwiz
- **Active Community**: Regular updates and excellent documentation

**Comparison with Other Frameworks:**

| Framework | Status | Best For | Integration Complexity |
|-----------|--------|----------|----------------------|
| Ratatui | Active | Production apps | Low |
| Cursive | Active | Form-heavy UIs | Medium |
| termion | Active | Low-level control | High |
| crossterm | Active | Cross-platform basics | Medium |

## Quick Start

### Adding Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
cmdai = { path = ".", features = ["tui"] }
ratatui = "0.27"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }

# For the complete demo
anyhow = "1"
```

### Minimal Example

```rust
use cmdai::rendering::{Sprite, SpriteFrame, ColorPalette};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Widget},
    layout::Rect,
};
use std::io;

fn main() -> io::Result<()> {
    // Setup terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Create sprite
    let palette = ColorPalette::from_hex_strings(&["#FF0000", "#00FF00"]).unwrap();
    let frame = SpriteFrame::new(4, 4, vec![0,1,1,0, 1,0,0,1, 1,0,0,1, 0,1,1,0], 100).unwrap();
    let sprite = Sprite::new("test".to_string(), palette, vec![frame]).unwrap();

    // Render
    terminal.draw(|f| {
        let area = f.size();
        // Render sprite at position
        render_sprite_at(&sprite, 0, f, area);
    })?;

    Ok(())
}
```

## Architecture Patterns

### Pattern 1: Static Rendering

Best for: Non-animated sprites, splash screens, static decorations

```rust
use cmdai::rendering::{Sprite, TerminalRenderer};
use ratatui::widgets::Widget;

pub struct StaticSpriteWidget<'a> {
    sprite: &'a Sprite,
    frame_index: usize,
}

impl<'a> StaticSpriteWidget<'a> {
    pub fn new(sprite: &'a Sprite) -> Self {
        Self { sprite, frame_index: 0 }
    }

    pub fn with_frame(mut self, index: usize) -> Self {
        self.frame_index = index;
        self
    }
}

impl<'a> Widget for StaticSpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = match self.sprite.frame(self.frame_index) {
            Some(f) => f,
            None => return,
        };

        let palette = self.sprite.palette();

        // Render each pixel to the buffer
        for y in 0..frame.height().min(area.height as usize) {
            for x in 0..frame.width().min(area.width as usize) {
                if let Some(pixel_idx) = frame.get_pixel(x, y) {
                    if !palette.is_transparent(pixel_idx) {
                        if let Some(color) = palette.get(pixel_idx) {
                            let cell = buf.get_mut(area.x + x as u16, area.y + y as u16);
                            cell.set_symbol("█");
                            cell.set_fg(RatatuiColor::Rgb(color.r, color.g, color.b));
                        }
                    }
                }
            }
        }
    }
}
```

**Usage:**
```rust
terminal.draw(|f| {
    let widget = StaticSpriteWidget::new(&sprite);
    f.render_widget(widget, area);
})?;
```

### Pattern 2: Managed Animation State

Best for: Full control over animation timing, coordinated multi-sprite scenes

```rust
use cmdai::rendering::{Animation, AnimationMode};
use std::time::{Duration, Instant};

pub struct AnimationController {
    animation: Animation,
    last_update: Instant,
}

impl AnimationController {
    pub fn new(sprite: Sprite, mode: AnimationMode) -> Self {
        Self {
            animation: Animation::new(sprite, mode),
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) -> bool {
        let elapsed = self.last_update.elapsed();
        let frame_duration = Duration::from_millis(
            self.animation.current_frame().duration_ms()
        );

        if elapsed >= frame_duration {
            self.last_update = Instant::now();
            self.animation.advance()
        } else {
            true
        }
    }

    pub fn current_frame(&self) -> &SpriteFrame {
        self.animation.current_frame()
    }

    pub fn palette(&self) -> &ColorPalette {
        self.animation.palette()
    }

    pub fn reset(&mut self) {
        self.animation.reset();
        self.last_update = Instant::now();
    }
}
```

**Usage:**
```rust
let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

loop {
    terminal.draw(|f| {
        let frame = controller.current_frame();
        let palette = controller.palette();
        render_frame(frame, palette, f, area);
    })?;

    if !controller.update() {
        break; // Animation complete
    }

    thread::sleep(Duration::from_millis(16)); // ~60 FPS
}
```

### Pattern 3: Self-Updating Widget

Best for: Encapsulated animated widgets, reusable components

```rust
pub struct AnimatedSpriteWidget {
    controller: AnimationController,
    x: u16,
    y: u16,
}

impl AnimatedSpriteWidget {
    pub fn new(sprite: Sprite, mode: AnimationMode, x: u16, y: u16) -> Self {
        Self {
            controller: AnimationController::new(sprite, mode),
            x,
            y,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.controller.update()
    }

    pub fn render(&self, f: &mut Frame) {
        let frame = self.controller.current_frame();
        let palette = self.controller.palette();

        // Render at (self.x, self.y)
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                if let Some(pixel_idx) = frame.get_pixel(x, y) {
                    if !palette.is_transparent(pixel_idx) {
                        if let Some(color) = palette.get(pixel_idx) {
                            // Position within frame
                            let px = self.x + x as u16;
                            let py = self.y + y as u16;

                            // Render pixel
                            let area = Rect::new(px, py, 1, 1);
                            render_pixel_at(color, f, area);
                        }
                    }
                }
            }
        }
    }
}
```

### Pattern 4: Multi-Sprite Scene

Best for: Games, complex UIs with multiple animated elements

```rust
pub struct SpriteScene {
    sprites: Vec<AnimatedSpriteWidget>,
}

impl SpriteScene {
    pub fn new() -> Self {
        Self { sprites: Vec::new() }
    }

    pub fn add_sprite(&mut self, widget: AnimatedSpriteWidget) {
        self.sprites.push(widget);
    }

    pub fn update(&mut self) {
        self.sprites.retain_mut(|sprite| sprite.tick());
    }

    pub fn render(&self, f: &mut Frame) {
        for sprite in &self.sprites {
            sprite.render(f);
        }
    }
}
```

## Widget Implementations

### Complete SpriteWidget

See `src/rendering/ratatui_widget.rs` for the full implementation that provides:

- `SpriteWidget` - Static sprite rendering
- `AnimatedSpriteWidget` - Self-contained animated sprite
- Automatic color conversion (RGB to Ratatui colors)
- Transparency support
- Bounds checking
- Frame timing management

### Helper Functions

```rust
use ratatui::style::Color as RatatuiColor;
use cmdai::rendering::Color;

/// Convert cmdai Color to Ratatui Color
pub fn to_ratatui_color(color: &Color) -> RatatuiColor {
    RatatuiColor::Rgb(color.r, color.g, color.b)
}

/// Render a single pixel
pub fn render_pixel(color: &Color, x: u16, y: u16, buf: &mut Buffer) {
    if let Some(cell) = buf.get_mut(x, y) {
        cell.set_symbol("█");
        cell.set_fg(to_ratatui_color(color));
    }
}

/// Clear an area (set to transparent)
pub fn clear_area(area: Rect, buf: &mut Buffer) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.get_mut(x, y) {
                cell.reset();
            }
        }
    }
}
```

## Event Handling

### Keyboard Controls

```rust
use crossterm::event::{self, Event, KeyCode};

pub enum AnimationControl {
    Pause,
    Resume,
    SpeedUp,
    SlowDown,
    Reset,
    Quit,
}

pub fn handle_input() -> io::Result<Option<AnimationControl>> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            return Ok(match key.code {
                KeyCode::Char(' ') => Some(AnimationControl::Pause),
                KeyCode::Char('r') => Some(AnimationControl::Resume),
                KeyCode::Char('+') | KeyCode::Char('=') => Some(AnimationControl::SpeedUp),
                KeyCode::Char('-') => Some(AnimationControl::SlowDown),
                KeyCode::Char('0') => Some(AnimationControl::Reset),
                KeyCode::Char('q') | KeyCode::Esc => Some(AnimationControl::Quit),
                _ => None,
            });
        }
    }
    Ok(None)
}
```

### Mouse Events

```rust
use crossterm::event::{MouseEvent, MouseEventKind};

pub fn handle_mouse(event: MouseEvent, sprites: &[AnimatedSpriteWidget]) -> Option<usize> {
    if let MouseEventKind::Down(_) = event.kind {
        // Find which sprite was clicked
        for (idx, sprite) in sprites.iter().enumerate() {
            if is_point_in_sprite(event.column, event.row, sprite) {
                return Some(idx);
            }
        }
    }
    None
}

fn is_point_in_sprite(x: u16, y: u16, sprite: &AnimatedSpriteWidget) -> bool {
    let frame = sprite.controller.current_frame();
    x >= sprite.x && x < sprite.x + frame.width() as u16 &&
    y >= sprite.y && y < sprite.y + frame.height() as u16
}
```

## Performance Considerations

### Frame Rate Management

**Target Frame Rates:**
- **UI Applications**: 30-60 FPS is sufficient
- **Games**: 60 FPS recommended
- **Animations Only**: Match slowest sprite's frame duration

```rust
use std::time::{Duration, Instant};

pub struct FrameTimer {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
}

impl FrameTimer {
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            frame_duration: Duration::from_millis(1000 / target_fps as u64),
            last_frame: Instant::now(),
        }
    }

    pub fn wait(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            std::thread::sleep(self.frame_duration - elapsed);
        }
        self.last_frame = Instant::now();
    }

    pub fn actual_fps(&self) -> f64 {
        1.0 / self.last_frame.elapsed().as_secs_f64()
    }
}
```

### Optimization Tips

1. **Minimize Redraws**
   - Only redraw when animation frames change or UI updates
   - Use dirty regions to track what needs updating

2. **Sprite Culling**
   ```rust
   pub fn is_sprite_visible(sprite: &AnimatedSpriteWidget, viewport: Rect) -> bool {
       let frame = sprite.controller.current_frame();
       let sprite_rect = Rect::new(
           sprite.x,
           sprite.y,
           frame.width() as u16,
           frame.height() as u16
       );
       viewport.intersects(sprite_rect)
   }
   ```

3. **Batch Updates**
   ```rust
   pub struct SpriteBatch {
       sprites: Vec<AnimatedSpriteWidget>,
       dirty: bool,
   }

   impl SpriteBatch {
       pub fn update(&mut self) {
           self.dirty = false;
           for sprite in &mut self.sprites {
               if sprite.tick() {
                   self.dirty = true;
               }
           }
       }

       pub fn needs_render(&self) -> bool {
           self.dirty
       }
   }
   ```

4. **Memory Management**
   - Pre-allocate sprite collections
   - Reuse buffer allocations
   - Use object pooling for temporary sprites

### Profiling

```rust
use std::time::Instant;

pub struct PerformanceMetrics {
    frame_times: Vec<Duration>,
    max_samples: usize,
}

impl PerformanceMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_frame(&mut self, duration: Duration) {
        self.frame_times.push(duration);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    pub fn average_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_duration: Duration = self.frame_times.iter().sum::<Duration>()
            / self.frame_times.len() as u32;
        1.0 / avg_duration.as_secs_f64()
    }

    pub fn min_fps(&self) -> f64 {
        self.frame_times.iter()
            .max()
            .map(|d| 1.0 / d.as_secs_f64())
            .unwrap_or(0.0)
    }
}
```

## Complete Examples

### Example 1: Simple Sprite Display

```rust
use cmdai::rendering::{Sprite, examples::create_idle_character};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let sprite = create_idle_character().unwrap();

    terminal.draw(|f| {
        let area = f.size();
        let widget = SpriteWidget::new(&sprite);
        f.render_widget(widget, area);
    })?;

    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}
```

### Example 2: Animated Sprite with Controls

```rust
use cmdai::rendering::{examples::create_walking_animation, AnimationMode};
use crossterm::event::{KeyCode, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;
    let sprite = create_walking_animation()?;
    let mut controller = AnimationController::new(sprite, AnimationMode::Loop);
    let mut paused = false;

    loop {
        terminal.draw(|f| {
            let frame = controller.current_frame();
            let palette = controller.palette();
            render_frame_centered(frame, palette, f);

            // Show controls
            let controls = Paragraph::new("Space: Pause | Q: Quit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(controls, bottom_area);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        // Update animation
        if !paused && !controller.update() {
            break;
        }
    }

    restore_terminal()?;
    Ok(())
}
```

### Example 3: Multi-Sprite Game Scene

```rust
use cmdai::rendering::examples::*;

struct GameState {
    scene: SpriteScene,
    score: u32,
}

impl GameState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut scene = SpriteScene::new();

        // Add player
        let player = create_walking_animation()?;
        scene.add_sprite(AnimatedSpriteWidget::new(
            player, AnimationMode::Loop, 10, 10
        ));

        // Add coins
        for i in 0..5 {
            let coin = create_coin_animation()?;
            scene.add_sprite(AnimatedSpriteWidget::new(
                coin, AnimationMode::Loop, 20 + i * 10, 5
            ));
        }

        Ok(Self { scene, score: 0 })
    }

    fn update(&mut self) {
        self.scene.update();
    }

    fn render(&self, f: &mut Frame) {
        self.scene.render(f);

        // Render score
        let score_text = Paragraph::new(format!("Score: {}", self.score))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(score_text, Rect::new(0, 0, 20, 1));
    }
}
```

See `examples/ratatui_sprite_demo.rs` for a complete working demo with:
- Multiple animated sprites
- Keyboard controls (pause, speed, reset)
- FPS counter
- Clean terminal setup/teardown
- Error handling

## Advanced Integration

### Custom Rendering Backends

If you need to use a different terminal backend:

```rust
use ratatui::backend::Backend;

pub fn render_sprite_generic<B: Backend>(
    sprite: &Sprite,
    frame_idx: usize,
    backend: &mut B,
    area: Rect,
) -> io::Result<()> {
    // Backend-agnostic rendering
    let frame = sprite.frame(frame_idx).unwrap();
    let palette = sprite.palette();

    // Use backend's draw methods
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            if let Some(pixel_idx) = frame.get_pixel(x, y) {
                if !palette.is_transparent(pixel_idx) {
                    if let Some(color) = palette.get(pixel_idx) {
                        backend.draw(
                            area.x + x as u16,
                            area.y + y as u16,
                            &[Cell::new("█").fg(to_ratatui_color(color))]
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
```

### Async Animation Updates

For applications using async/await:

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct AsyncAnimationController {
    animation: Animation,
    frame_tx: mpsc::Sender<usize>,
}

impl AsyncAnimationController {
    pub fn spawn(sprite: Sprite, mode: AnimationMode) -> mpsc::Receiver<usize> {
        let (tx, rx) = mpsc::channel(10);
        let mut animation = Animation::new(sprite, mode);

        tokio::spawn(async move {
            loop {
                let frame_duration = Duration::from_millis(
                    animation.current_frame().duration_ms()
                );
                tokio::time::sleep(frame_duration).await;

                if !animation.advance() {
                    break;
                }

                if tx.send(animation.current_frame).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        rx
    }
}
```

### Layer System

For complex scenes with multiple sprite layers:

```rust
pub struct SpriteLayer {
    sprites: Vec<AnimatedSpriteWidget>,
    z_index: i32,
    visible: bool,
}

pub struct LayeredScene {
    layers: Vec<SpriteLayer>,
}

impl LayeredScene {
    pub fn render(&self, f: &mut Frame) {
        let mut sorted_layers = self.layers.iter()
            .filter(|l| l.visible)
            .collect::<Vec<_>>();

        sorted_layers.sort_by_key(|l| l.z_index);

        for layer in sorted_layers {
            for sprite in &layer.sprites {
                sprite.render(f);
            }
        }
    }
}
```

## Troubleshooting

### Common Issues

**Issue**: Colors appear incorrect or washed out
```
Solution: Check terminal's true color support
$ echo $COLORTERM  # Should be "truecolor" or "24bit"

// Fallback to 256-color mode
let renderer = TerminalRenderer::new()
    .with_color_mode(ColorMode::Ansi256);
```

**Issue**: Animation appears choppy
```
Solution: Verify frame timing
- Check CPU usage (use `htop`)
- Increase frame duration in SpriteFrame
- Reduce number of simultaneous animations
- Profile with `cargo flamegraph`
```

**Issue**: Sprites don't render at expected position
```
Solution: Check coordinate systems
- Ratatui uses (column, row) / (x, y) ordering
- Ensure viewport bounds checking
- Account for widget borders and padding
```

**Issue**: Terminal doesn't restore after crash
```
Solution: Use proper cleanup
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

// Use this pattern:
let result = run_app();
restore_terminal()?;
result
```

### Debugging Tips

1. **Enable Logging**
   ```rust
   use tracing_subscriber;

   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::DEBUG)
       .init();
   ```

2. **Render Debug Info**
   ```rust
   pub fn render_debug_info(f: &mut Frame, controller: &AnimationController) {
       let debug = format!(
           "Frame: {}/{} | Time: {}ms",
           controller.animation.current_frame,
           controller.animation.sprite().frame_count(),
           controller.last_update.elapsed().as_millis()
       );
       let widget = Paragraph::new(debug);
       f.render_widget(widget, debug_area);
   }
   ```

3. **Capture Frame Output**
   ```bash
   # Record terminal session
   $ asciinema rec demo.cast
   $ cargo run --example ratatui_sprite_demo
   $ exit
   ```

## Further Reading

- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Crossterm Documentation](https://docs.rs/crossterm/)
- [cmdai Sprite System](./ANIMATION_SYSTEM.md)
- [Game Engine Integration](./GAME_ENGINE_INTEGRATION.md)

## Next Steps

1. Review the complete demo: `examples/ratatui_sprite_demo.rs`
2. Explore game engine integration: `docs/GAME_ENGINE_INTEGRATION.md`
3. Check out the widget implementation: `src/rendering/ratatui_widget.rs`
4. Build your own TUI application!

For questions or issues, please open an issue on the GitHub repository.

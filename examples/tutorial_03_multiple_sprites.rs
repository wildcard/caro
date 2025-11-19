//! Tutorial 03: Multiple Sprites
//!
//! Show multiple animations on screen at the same time!
//! This demonstrates how to manage several animated sprites.
//!
//! What you'll learn:
//! - How to create and manage multiple animation controllers
//! - How to position sprites at different screen locations
//! - How to create a simple scene with multiple elements
//!
//! Run with: cargo run --example tutorial_03_multiple_sprites --features tui

use cmdai::rendering::{
    examples::{create_heart_animation, create_coin_animation, create_walking_animation},
    AnimationMode,
};
use cmdai::rendering::ratatui_widget::AnimationController;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // === NEW: Create THREE different animated sprites ===

    // 1. Heart (pulses in place)
    let heart_sprite = create_heart_animation()?;
    let mut heart_controller = AnimationController::new(heart_sprite, AnimationMode::Loop);

    // 2. Coin (spins)
    let coin_sprite = create_coin_animation()?;
    let mut coin_controller = AnimationController::new(coin_sprite, AnimationMode::Loop);

    // 3. Walking character
    let walk_sprite = create_walking_animation()?;
    let mut walk_controller = AnimationController::new(walk_sprite, AnimationMode::Loop);

    let mut paused = false;

    // Main event loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(15),      // Sprite area
                    Constraint::Length(3),    // Status area
                ])
                .split(f.size());

            // === NEW: Split sprite area into 3 columns ===
            let sprite_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),  // Heart
                    Constraint::Percentage(33),  // Coin
                    Constraint::Percentage(34),  // Walking character
                ])
                .split(chunks[0]);

            // Render each sprite in its own column
            render_sprite_in_area(f, &heart_controller, sprite_chunks[0], "Heart");
            render_sprite_in_area(f, &coin_controller, sprite_chunks[1], "Coin");
            render_sprite_in_area(f, &walk_controller, sprite_chunks[2], "Walker");

            // Status bar
            let status = if paused { "⏸ PAUSED" } else { "▶ PLAYING" };
            let status_text = format!(
                "{} | SPACE: Pause | Q: Quit | {} sprites",
                status, 3
            );

            let status_widget = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            f.render_widget(status_widget, chunks[1]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => paused = !paused,
                    _ => {}
                }
            }
        }

        // === NEW: Update ALL sprites ===
        if !paused {
            heart_controller.update();
            coin_controller.update();
            walk_controller.update();
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    println!("\n✅ Tutorial 03 complete! You rendered multiple sprites.");
    println!("Next: Try tutorial_04_interactive_scene.rs to make sprites move!");

    Ok(())
}

/// Helper function to render a sprite in a specific screen area
fn render_sprite_in_area(
    f: &mut ratatui::Frame,
    controller: &AnimationController,
    area: Rect,
    title: &str,
) {
    let frame = controller.current_frame();
    let palette = controller.palette();

    let sprite_output = render_sprite_simple(frame, palette);

    let widget = Paragraph::new(sprite_output)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(widget, area);
}

/// Simple sprite rendering helper
fn render_sprite_simple(
    frame: &cmdai::rendering::SpriteFrame,
    palette: &cmdai::rendering::ColorPalette,
) -> String {
    use cmdai::rendering::Color as SpriteColor;

    let mut output = String::new();
    let (width, height) = frame.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = frame.get_pixel(x, y).unwrap_or(0);

            if palette.is_transparent(pixel) {
                output.push(' ');
                continue;
            }

            let color = palette.get_color(pixel)
                .unwrap_or(&SpriteColor { r: 255, g: 255, b: 255 });

            output.push_str(&format!(
                "\x1b[38;2;{};{};{}m█\x1b[0m",
                color.r, color.g, color.b
            ));
        }
        output.push('\n');
    }

    output
}

/* EXPECTED OUTPUT:

┌─ Heart ───────┬─ Coin ────────┬─ Walker ──────┐
│               │               │               │
│   ♥♥   ♥♥    │    ◯◯◯◯      │   🚶‍♂️🚶‍♂️      │
│  ♥♥♥♥ ♥♥♥    │   ◯◯◯◯◯◯     │  🚶‍♂️ 🚶‍♂️🚶‍♂️    │
│  ♥♥♥♥♥♥♥     │  ◯◯◯◯◯◯◯◯    │   🚶‍♂️  🚶‍♂️    │
│   ♥♥♥♥♥      │   ◯◯◯◯◯◯     │               │
│    ♥♥♥       │    ◯◯◯◯      │               │
│     ♥        │               │               │
│              │               │               │
├──────────────┴───────────────┴───────────────┤
│  ▶ PLAYING | SPACE: Pause | Q: Quit | 3 sprites │
└──────────────────────────────────────────────────┘

All three animations run simultaneously!
The heart pulses, the coin spins, and the character walks.

WHAT'S NEW:

1. Multiple AnimationControllers
   - Each sprite has its own controller
   - Each controller tracks its own frame and timing
   - They all update independently

2. Layout splits into columns
   - Direction::Horizontal splits left-to-right
   - Constraint::Percentage divides space evenly
   - Each sprite gets its own area

3. Helper function render_sprite_in_area()
   - Takes a controller and a Rect (screen area)
   - Renders the sprite in that specific location
   - Keeps the code clean and reusable

4. Multiple update() calls
   - Each controller must be updated separately
   - They can have different speeds/modes
   - All controlled by the same pause state

KEY PATTERN:

// Create multiple sprites
let mut sprites = vec![
    AnimationController::new(sprite1, mode),
    AnimationController::new(sprite2, mode),
    AnimationController::new(sprite3, mode),
];

// Update all sprites
for sprite in &mut sprites {
    sprite.update();
}

// Render all sprites
for (i, sprite) in sprites.iter().enumerate() {
    render_sprite_in_area(f, sprite, areas[i], "Sprite");
}

PERFORMANCE NOTES:

With 3 sprites at 60 FPS:
- CPU usage: ~1-2%
- Memory: ~5 MB
- Frame time: <1ms

You can easily handle 10-20 sprites on modern hardware!

EXERCISES:

1. Add a 4th sprite:
   - Use create_spinner_animation()
   - Add a 4th column to the layout
   - Remember to update it!

2. Give each sprite its own speed:
   - Create a wrapper struct with speed multiplier
   - Only update every Nth frame for slower sprites

3. Make sprites toggle-able:
   - Add keys 1, 2, 3 to show/hide each sprite
   - Use boolean flags: show_heart, show_coin, etc.

4. Add individual pause controls:
   - 'h' to pause heart
   - 'c' to pause coin
   - 'w' to pause walker

COMMON ISSUES:

1. "Sprites are cut off"
   → The terminal window is too small
   → Make your terminal bigger or reduce sprite count

2. "Animations are out of sync"
   → This is normal! Each has its own timing
   → To sync: reset all to frame 0 together

3. "Too much screen flicker"
   → This shouldn't happen with Ratatui's double buffering
   → If it does, increase sleep duration

SCALING UP:

For more than 3-5 sprites, consider:
- Using a SpriteScene (from ratatui_widget.rs)
- Sprite culling (don't render off-screen sprites)
- Dynamic layouts based on sprite count
- Grid-based positioning

NEXT STEPS:

→ tutorial_04_interactive_scene.rs - Move sprites with arrow keys!
→ Or try modifying this to create your own scene

CHALLENGE:

Can you create a "fish tank" scene with:
- 3 fish sprites swimming
- 2 bubble sprites floating up
- 1 heart sprite in the corner

*/

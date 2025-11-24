//! Tutorial 02: Keyboard Controls
//!
//! Building on Tutorial 01, this adds keyboard interactivity!
//! Press SPACE to pause/resume, Q to quit.
//!
//! What you'll learn:
//! - How to handle keyboard input in Ratatui
//! - How to pause/resume animations
//! - How to create a clean event loop
//!
//! Run with: cargo run --example tutorial_02_keyboard_controls --features tui

use cmdai::rendering::{examples::create_heart_animation, AnimationMode};
use cmdai::rendering::ratatui_widget::AnimationController;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal (same as Tutorial 01)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create animated sprite
    let sprite = create_heart_animation()?;
    let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

    // === NEW: Track pause state ===
    let mut paused = false;

    // === MODIFIED: Event loop instead of simple loop ===
    loop {
        // Draw the current frame
        terminal.draw(|f| {
            // Split screen into sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(10),      // Sprite area
                    Constraint::Length(3),    // Controls area
                ])
                .split(f.size());

            // Render the sprite
            let frame = controller.current_frame();
            let palette = controller.palette();
            let sprite_output = render_sprite_simple(frame, palette);

            let sprite_widget = Paragraph::new(sprite_output)
                .block(Block::default()
                    .title("Tutorial 02: Keyboard Controls")
                    .borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(sprite_widget, chunks[0]);

            // Show controls
            let status = if paused { "⏸ PAUSED" } else { "▶ PLAYING" };
            let controls = format!(
                "{} | SPACE: Pause/Resume | Q: Quit | FPS: {:.1}",
                status,
                controller.current_fps()
            );

            let controls_widget = Paragraph::new(controls)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            f.render_widget(controls_widget, chunks[1]);
        })?;

        // === NEW: Check for keyboard input ===
        // poll() checks if there's an event waiting, with a timeout
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    // Q or Esc to quit
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    // Space to pause/resume
                    KeyCode::Char(' ') => {
                        paused = !paused;
                    }

                    _ => {}
                }
            }
        }

        // Only update animation if not paused
        if !paused {
            controller.update();
        }

        // Frame rate limiting (60 FPS)
        std::thread::sleep(Duration::from_millis(16));
    }

    // Clean up
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    println!("\n✅ Tutorial 02 complete! You added keyboard controls.");
    println!("Next: Try tutorial_03_multiple_sprites.rs to show multiple animations!");

    Ok(())
}

/// Simple sprite rendering (same as Tutorial 01)
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

You'll see the same heart animation, but now with:

┌─ Tutorial 02: Keyboard Controls ──────────────┐
│                                               │
│              ♥♥   ♥♥                         │
│             ♥♥♥♥ ♥♥♥                         │
│             ♥♥♥♥♥♥♥                          │
│              ♥♥♥♥♥                           │
│               ♥♥♥                            │
│                ♥                             │
│                                               │
└───────────────────────────────────────────────┘
┌───────────────────────────────────────────────┐
│  ▶ PLAYING | SPACE: Pause/Resume | Q: Quit   │
│  FPS: 60.0                                    │
└───────────────────────────────────────────────┘

Press SPACE and the animation freezes!
Press SPACE again and it continues!

WHAT'S NEW:

1. event::poll() - Check if keyboard input is waiting
   - Takes a timeout (16ms = one frame)
   - Returns true if there's input to read

2. event::read() - Actually read the keyboard event
   - Returns the key that was pressed

3. match code - Handle different keys:
   - 'q' or Esc → quit the program
   - Space → toggle pause state

4. Conditional update - Only update animation if not paused
   - if !paused { controller.update(); }

5. Layout - Split screen into sections
   - Layout::default() creates a layout manager
   - Direction::Vertical splits top-to-bottom
   - Constraints define sizes (Min, Length, Percentage)

KEYBOARD INPUT PATTERNS:

// Pattern 1: Non-blocking check (what we use here)
if event::poll(Duration::from_millis(16))? {
    // Process input
}

// Pattern 2: Blocking wait (for menu-driven apps)
match event::read()? {
    Event::Key(key) => // handle key
}

// Pattern 3: With timeout (for games)
if event::poll(Duration::from_millis(0))? {
    // Instant check, no wait
}

COMMON ISSUES:

1. "The animation is choppy when I press keys"
   → This is because event::poll() can block for 16ms
   → Solution: Use a shorter timeout or async events

2. "Keys don't work"
   → Make sure you called enable_raw_mode()
   → Try pressing the key harder (just kidding!)

3. "Program doesn't quit when I press Q"
   → Check if KeyCode::Char('q') matches exactly
   → Try adding .to_lowercase() if needed

EXERCISES:

1. Add more controls:
   - '+' to speed up animation
   - '-' to slow down animation
   - 'r' to reset to first frame

2. Change the sprite:
   - Use create_coin_animation() instead
   - Try create_walking_animation()

3. Add frame counter:
   - Show current frame number in the UI
   - Hint: Add a counter that increments in update()

NEXT STEPS:

→ tutorial_03_multiple_sprites.rs - Show 3 different animations at once
→ Or jump to tutorial_04_interactive_scene.rs for movement!

*/

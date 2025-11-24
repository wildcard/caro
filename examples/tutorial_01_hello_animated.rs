//! Tutorial 01: Hello Animated World
//!
//! This is the SIMPLEST possible animated TUI app - just 10 lines of actual code!
//! It shows a single blinking heart animation in your terminal.
//!
//! What you'll learn:
//! - How to set up a basic Ratatui terminal
//! - How to load a pre-made sprite animation
//! - How to render it in a loop
//!
//! Run with: cargo run --example tutorial_01_hello_animated --features tui

use cmdai::rendering::{examples::create_heart_animation, AnimationMode};
use cmdai::rendering::ratatui_widget::AnimationController;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === STEP 1: Set up the terminal ===
    // This puts your terminal in "raw mode" where we can draw anywhere on screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // === STEP 2: Create an animated sprite ===
    // We'll use a pre-made heart animation (pulses in 3 frames)
    let sprite = create_heart_animation()?;
    let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

    // === STEP 3: Animation loop ===
    // Keep updating and drawing until we've shown 100 frames (about 10 seconds)
    for _ in 0..100 {
        // Draw the current frame
        terminal.draw(|f| {
            let frame = controller.current_frame();
            let palette = controller.palette();

            // Get the sprite as colored blocks
            let output = render_sprite_simple(frame, palette);

            // Draw at position (5, 5) on the screen
            use ratatui::widgets::{Paragraph, Block};
            let widget = Paragraph::new(output)
                .block(Block::default().title("Tutorial 01: Hello Animated World"));
            f.render_widget(widget, f.size());
        })?;

        // Update to next frame (returns false when animation ends)
        controller.update();

        // Wait a bit before next frame (60 FPS)
        std::thread::sleep(Duration::from_millis(16));
    }

    // === STEP 4: Clean up the terminal ===
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    println!("\n✅ Tutorial 01 complete! You just rendered your first animated sprite.");
    println!("Next: Try tutorial_02_keyboard_controls.rs to add interactivity!");

    Ok(())
}

/// Simple helper to convert a sprite frame to colored text
/// This is a simplified version - the real widget is more sophisticated
fn render_sprite_simple(
    frame: &cmdai::rendering::SpriteFrame,
    palette: &cmdai::rendering::ColorPalette,
) -> String {
    use cmdai::rendering::Color;

    let mut output = String::new();
    let (width, height) = frame.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = frame.get_pixel(x, y).unwrap_or(0);

            // If transparent (index 0), skip
            if palette.is_transparent(pixel) {
                output.push(' ');
                continue;
            }

            // Get the color and render as a colored block
            let color = palette.get_color(pixel).unwrap_or(&Color { r: 255, g: 255, b: 255 });

            // Use ANSI escape codes for true color
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

You should see a red/pink heart that gently pulses (gets bigger and smaller).
The animation loops forever for about 10 seconds.

The heart looks something like:

  ♥♥   ♥♥
 ♥♥♥♥ ♥♥♥
 ♥♥♥♥♥♥♥
  ♥♥♥♥♥
   ♥♥♥
    ♥

(but with colors and animation!)

COMMON MISTAKES:

1. "Error: failed to enter raw mode"
   → Make sure you're running in a terminal, not in an IDE output panel

2. "The screen is messed up after the program exits"
   → The program should clean up automatically, but if it crashes, run: `reset`

3. "I don't see colors"
   → Make sure your terminal supports true color (most modern terminals do)
   → Try iTerm2 on Mac, Windows Terminal on Windows, or GNOME Terminal on Linux

WHAT'S HAPPENING:

1. enable_raw_mode() - Lets us take full control of the terminal
2. EnterAlternateScreen - Creates a "second screen" so your shell stays clean
3. Terminal::new() - Creates Ratatui's terminal manager
4. create_heart_animation() - Loads a pre-made 6x6 pixel animated heart
5. AnimationController - Manages frame timing and updates
6. terminal.draw() - Renders one frame
7. controller.update() - Advances to next animation frame
8. Sleep 16ms - Wait for next frame (≈60 FPS)

NEXT STEPS:

→ Try tutorial_02_keyboard_controls.rs to add pause/resume
→ Or modify this code to use a different pre-made sprite:
  - create_walking_animation() - Walking character
  - create_coin_animation() - Spinning coin
  - create_spinner_animation() - Loading spinner
  - create_idle_character() - Static character

*/

# Getting Started with Terminal UI Animations

> **For Complete Beginners**: This guide assumes you've never built a terminal UI application before. We'll go step-by-step from zero to your first animated TUI app.

## Table of Contents

- [What is a Terminal UI (TUI)?](#what-is-a-terminal-ui-tui)
- [Prerequisites](#prerequisites)
- [Your First 5 Minutes](#your-first-5-minutes)
- [Understanding the Basics](#understanding-the-basics)
- [Progressive Tutorials](#progressive-tutorials)
- [Common Patterns](#common-patterns)
- [Troubleshooting](#troubleshooting)
- [Next Steps](#next-steps)

## What is a Terminal UI (TUI)?

A Terminal UI (TUI) is a text-based user interface that runs in your terminal/command line, but with:
- **Colors** and **styling**
- **Interactive** elements (clickable, keyboard navigation)
- **Animations** and dynamic content
- **Layouts** (boxes, borders, columns, rows)

Think of it like a web app, but in your terminal!

**Examples you might have seen**:
- `htop` - System monitor
- `vim` - Text editor
- `spotify-tui` - Spotify client
- Terminal file managers

**What makes this project special**:
- You can add **animated pixel art characters** to your TUI apps
- Pre-made sprites (walking characters, hearts, spinners)
- Support for custom artwork (Aseprite, ANSI art)

## Prerequisites

### What You Need

1. **Rust installed** (1.75 or newer)
   ```bash
   # Check if you have Rust
   rustc --version

   # If not, install from https://rustup.rs/
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **A terminal that supports colors**
   - **macOS**: iTerm2 (best) or Terminal.app (good)
   - **Windows**: Windows Terminal (best) or PowerShell (ok)
   - **Linux**: GNOME Terminal, Konsole, Alacritty (all good)

3. **Basic Rust knowledge**
   - You should know: variables, functions, `Result<T, E>`
   - If not, spend 30 minutes on [Rust By Example](https://doc.rust-lang.org/rust-by-example/)

### What You Don't Need

- ❌ Graphics programming experience
- ❌ Game development knowledge
- ❌ Deep understanding of terminals
- ❌ Experience with TUI frameworks

**We'll teach you everything you need!**

## Your First 5 Minutes

Let's get something running **right now**.

### Step 1: Clone or Add Dependency

**Option A: Clone this repo**
```bash
git clone https://github.com/wildcard/cmdai.git
cd cmdai
```

**Option B: Add to your own project**
```toml
# In your Cargo.toml
[dependencies]
cmdai = { path = "../cmdai", features = ["tui"] }
# Or when published:
# cmdai = { version = "0.1", features = ["tui"] }
```

### Step 2: Run Your First Animated App

```bash
cargo run --example tutorial_01_hello_animated --features tui
```

**You should see**: A red/pink heart that pulses (gets bigger and smaller)!

If it works: 🎉 **Congratulations!** You just ran your first animated TUI app!

If not: Jump to [Troubleshooting](#troubleshooting)

### Step 3: Try Keyboard Controls

```bash
cargo run --example tutorial_02_keyboard_controls --features tui
```

Now try:
- Press **SPACE** to pause/resume
- Press **Q** to quit

### Step 4: See Multiple Animations

```bash
cargo run --example tutorial_03_multiple_sprites --features tui
```

You'll see 3 animations at once: heart, coin, and walking character!

**Did all three work?**
- ✅ Yes → You're ready to build your own!
- ❌ No → See [Troubleshooting](#troubleshooting)

## Understanding the Basics

### The Core Concepts

#### 1. Terminal Setup
```rust
// These lines put your terminal in "TUI mode"
enable_raw_mode()?;  // Capture all keyboard input
execute!(stdout, EnterAlternateScreen)?;  // Use a clean screen
let terminal = Terminal::new(backend)?;  // Create Ratatui terminal
```

**What this does**:
- Takes control of your terminal
- Hides your shell prompt
- Lets you draw anywhere on screen
- Captures all keyboard input

#### 2. Animation Controller
```rust
// Load a pre-made sprite
let sprite = create_heart_animation()?;

// Create a controller that manages the animation
let mut controller = AnimationController::new(sprite, AnimationMode::Loop);
```

**What this does**:
- Loads pixel art and animation data
- Manages frame timing
- Tracks current frame
- Handles looping/one-shot modes

#### 3. The Draw Loop
```rust
loop {
    terminal.draw(|f| {
        // Your drawing code here
    })?;

    controller.update();  // Go to next frame
    sleep(Duration::from_millis(16));  // 60 FPS
}
```

**What this does**:
- Continuously redraws the screen
- Updates animation state
- Waits for next frame (16ms = 60 FPS)

#### 4. Cleanup
```rust
disable_raw_mode()?;
execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
```

**What this does**:
- Restores normal terminal mode
- Returns to your shell prompt
- Very important! Without this, your terminal stays "broken"

### The Mental Model

Think of a TUI app like a video game:

1. **Setup** - Initialize the terminal (like creating a window)
2. **Loop** - Repeatedly:
   - Read input (keyboard, mouse)
   - Update state (move things, change animations)
   - Draw frame (render everything)
   - Wait for next frame (60 FPS)
3. **Cleanup** - Restore terminal (like closing the window)

### Ratatui's Layout System

Ratatui uses a **constraint-based layout**:

```rust
// Split screen vertically into 3 parts
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),       // Top: 3 lines tall
        Constraint::Min(10),          // Middle: at least 10 lines
        Constraint::Percentage(20),   // Bottom: 20% of screen
    ])
    .split(f.size());

// Now render into each chunk
f.render_widget(header_widget, chunks[0]);  // Top
f.render_widget(content_widget, chunks[1]); // Middle
f.render_widget(footer_widget, chunks[2]);  // Bottom
```

**Layout directions**:
- `Direction::Vertical` - Split top-to-bottom (horizontal lines)
- `Direction::Horizontal` - Split left-to-right (vertical lines)

**Constraint types**:
- `Length(n)` - Exactly N lines/columns
- `Min(n)` - At least N lines/columns
- `Max(n)` - At most N lines/columns
- `Percentage(n)` - N% of available space
- `Ratio(n, m)` - N out of M parts

## Progressive Tutorials

We've created 5 tutorials that build on each other. Work through them in order!

### Tutorial 01: Hello Animated World (5 minutes)
**File**: `examples/tutorial_01_hello_animated.rs`

**What you'll learn**:
- Basic terminal setup
- Loading a pre-made sprite
- Simple animation loop

**Difficulty**: ⭐☆☆☆☆

**Run it**:
```bash
cargo run --example tutorial_01_hello_animated --features tui
```

**Key code** (just 10 lines!):
```rust
let sprite = create_heart_animation()?;
let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

loop {
    terminal.draw(|f| { /* draw sprite */ })?;
    controller.update();
    sleep(Duration::from_millis(16));
}
```

### Tutorial 02: Keyboard Controls (10 minutes)
**File**: `examples/tutorial_02_keyboard_controls.rs`

**What you'll learn**:
- Handling keyboard input
- Pause/resume animations
- Event polling

**Difficulty**: ⭐⭐☆☆☆

**New concepts**:
- `event::poll()` - Check for input
- `event::read()` - Read keyboard events
- State management (paused flag)

### Tutorial 03: Multiple Sprites (15 minutes)
**File**: `examples/tutorial_03_multiple_sprites.rs`

**What you'll learn**:
- Managing multiple animations
- Layout system (columns)
- Organizing code with helper functions

**Difficulty**: ⭐⭐⭐☆☆

**New concepts**:
- Multiple AnimationControllers
- Layout::Horizontal
- Helper functions for rendering

### Tutorial 04: Interactive Scene (coming soon!)
**What you'll learn**:
- Moving sprites with arrow keys
- Collision detection
- Building a simple game

**Difficulty**: ⭐⭐⭐⭐☆

### Tutorial 05: Complete Game (coming soon!)
**What you'll learn**:
- Game state management
- Score tracking
- Multiple scenes
- Building a full mini-game

**Difficulty**: ⭐⭐⭐⭐⭐

## Common Patterns

### Pattern 1: Simple Animation Display

**Use case**: Just show an animation, no interaction

```rust
let sprite = create_heart_animation()?;
let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

loop {
    terminal.draw(|f| {
        let frame = controller.current_frame();
        let palette = controller.palette();
        // Render sprite...
    })?;
    controller.update();
    sleep(Duration::from_millis(16));
}
```

**When to use**: Loading screens, status indicators, decorations

### Pattern 2: Interactive Animation

**Use case**: User can control the animation

```rust
let mut controller = AnimationController::new(sprite, mode);
let mut paused = false;

loop {
    // Handle input
    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char(' ') => paused = !paused,
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    // Update if not paused
    if !paused {
        controller.update();
    }

    // Draw...
}
```

**When to use**: Games, interactive demos, debugging tools

### Pattern 3: Multiple Synchronized Sprites

**Use case**: Several sprites that should sync up

```rust
let mut sprites = vec![
    AnimationController::new(sprite1, mode),
    AnimationController::new(sprite2, mode),
    AnimationController::new(sprite3, mode),
];

loop {
    // Update all sprites together
    for sprite in &mut sprites {
        sprite.update();
    }

    // Draw all sprites...
}
```

**When to use**: Chorus line, synchronized effects, formation animations

### Pattern 4: Scene with Positioned Sprites

**Use case**: Sprites at specific screen locations

```rust
struct PositionedSprite {
    controller: AnimationController,
    x: u16,
    y: u16,
}

let mut scene = vec![
    PositionedSprite { controller: heart, x: 10, y: 5 },
    PositionedSprite { controller: coin, x: 20, y: 5 },
];

terminal.draw(|f| {
    for sprite in &scene {
        let area = Rect::new(sprite.x, sprite.y, 10, 10);
        // Render sprite at position...
    }
})?;
```

**When to use**: Games, simulations, dashboards

## Troubleshooting

### "My terminal looks broken after the program exits"

**Problem**: The program crashed before cleanup

**Solution**: Type `reset` and press Enter

**Prevention**: Always use `?` operator for error handling so cleanup runs:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup...

    // Your code here (uses ? for errors)

    // Cleanup always runs due to ?
    Ok(())
}
```

### "I don't see any colors"

**Possible causes**:
1. Your terminal doesn't support true color
2. Terminal size is too small
3. Environment variable issue

**Solutions**:
```bash
# Test true color support
echo -e "\033[38;2;255;100;0mTRUECOLOR\033[0m"

# Try a different terminal
# Mac: iTerm2
# Windows: Windows Terminal
# Linux: GNOME Terminal or Alacritty
```

### "Error: failed to enter raw mode"

**Problem**: Another program is using the terminal

**Solutions**:
1. Run directly in terminal (not IDE output panel)
2. Close other terminal apps
3. Check if tmux/screen is interfering

### "Animations are choppy/laggy"

**Possible causes**:
1. Computer is under load
2. Terminal emulator is slow
3. Too many sprites

**Solutions**:
```rust
// Reduce frame rate
sleep(Duration::from_millis(33));  // 30 FPS instead of 60

// Limit sprite count
if sprites.len() > 10 {
    sprites.truncate(10);
}

// Use sprite culling (don't render off-screen sprites)
```

### "Sprite is cut off / doesn't fit"

**Problem**: Terminal window is too small

**Solutions**:
1. Make terminal window bigger
2. Use smaller sprites
3. Add size checking:

```rust
let size = f.size();
if size.width < 40 || size.height < 20 {
    // Show "window too small" message
    return;
}
```

### "Program panics with 'index out of bounds'"

**Problem**: Accessing invalid sprite data

**Common mistakes**:
```rust
// BAD: Assuming size
let pixel = frame.get_pixel(x, y).unwrap();  // ❌ Can panic!

// GOOD: Check bounds
if let Some(pixel) = frame.get_pixel(x, y) {  // ✅ Safe
    // Use pixel
}
```

## Next Steps

### Level 1: Beginner ✅
- [x] Run the 3 basic tutorials
- [ ] Modify tutorial_02 to add different controls
- [ ] Try different pre-made sprites
- [ ] Create a simple dashboard with 2-3 sprites

### Level 2: Intermediate
- [ ] Build a status indicator for your own CLI app
- [ ] Create a loading screen with multiple animations
- [ ] Add keyboard-controlled movement
- [ ] Build a simple animation viewer

### Level 3: Advanced
- [ ] Import your own Aseprite sprites
- [ ] Create a mini-game with collision detection
- [ ] Build a complex multi-screen TUI app
- [ ] Contribute back to the project!

### Resources

**Documentation**:
- [TUI Integration Guide](TUI_INTEGRATION.md) - Deep dive into Ratatui integration
- [Animation Guide](ANIMATION_GUIDE.md) - Complete API reference
- [Designer Guide](DESIGNER_GUIDE.md) - Creating custom sprites

**External Resources**:
- [Ratatui Book](https://ratatui.rs/book/) - Official Ratatui documentation
- [Crossterm Docs](https://docs.rs/crossterm/) - Terminal manipulation
- [Aseprite](https://www.aseprite.org/) - Create pixel art sprites

**Community**:
- Ratatui Discord: [discord.gg/pMCEU9hNEj](https://discord.gg/pMCEU9hNEj)
- cmdai Issues: [GitHub](https://github.com/wildcard/cmdai/issues)
- Ask questions! We're here to help 💚

### Project Ideas

**Easy**:
- System resource monitor with animated indicators
- Timer app with animated countdown
- Music player status display
- Git status dashboard

**Medium**:
- Terminal pet (Tamagotchi-style)
- Simple platformer game
- Animation showcase / gallery
- Character creator tool

**Hard**:
- Terminal RPG with animated battles
- Multi-player terminal game
- Complex data visualization
- Full application framework

### Contributing

Want to help make this better?

**Good first contributions**:
- Add more tutorial examples
- Improve error messages
- Write blog posts about your projects
- Create more pre-made sprites
- Report bugs

See [CONTRIBUTING_SPRITES.md](CONTRIBUTING_SPRITES.md) for how to contribute animations!

---

## Quick Reference Card

### Minimal Working Example

```rust
use cmdai::rendering::{examples::create_heart_animation, AnimationMode};
use cmdai::rendering::ratatui_widget::AnimationController;
use crossterm::{execute, terminal::*};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(
        CrosstermBackend::new(io::stdout())
    )?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let sprite = create_heart_animation()?;
    let mut ctrl = AnimationController::new(sprite, AnimationMode::Loop);

    for _ in 0..100 {
        terminal.draw(|f| { /* render sprite */ })?;
        ctrl.update();
        std::thread::sleep(Duration::from_millis(16));
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
```

### Common Keyboard Codes

```rust
KeyCode::Char('q')     // Letter keys
KeyCode::Esc           // Escape
KeyCode::Enter         // Enter/Return
KeyCode::Up            // Arrow up
KeyCode::Down          // Arrow down
KeyCode::Left          // Arrow left
KeyCode::Right         // Arrow right
KeyCode::Char(' ')     // Space
KeyCode::Tab           // Tab
KeyCode::Backspace     // Backspace
```

### Pre-Made Sprites

```rust
create_idle_character()    // 8x8 static character
create_walking_animation() // 8x8 walking cycle (4 frames)
create_heart_animation()   // 6x6 pulsing heart (3 frames)
create_coin_animation()    // 8x8 spinning coin (4 frames)
create_spinner_animation() // 5x5 loading spinner (8 frames)
```

---

**You're ready to build amazing terminal UIs!** 🚀

Start with Tutorial 01 and work your way up. Don't hesitate to experiment and break things - that's how you learn!

Need help? Open an issue or ask in discussions. Happy coding! 💚

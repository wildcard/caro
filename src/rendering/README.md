# Sprite Animation Rendering Module

A terminal-based pixel art sprite animation system for cmdai, using Unicode block characters and ANSI colors.

## Overview

This module provides a complete system for rendering animated pixel art characters in the terminal. Unlike ASCII art which uses text characters, this system uses colored Unicode block characters (█) to create true pixel-based graphics with full color support.

## Features

- **Color Palettes**: Define custom color palettes using hex colors (e.g., `#FF5733`)
- **Sprite Frames**: Multi-frame animations with configurable timing
- **Transparency**: Support for transparent pixels in sprites
- **Terminal Rendering**: Efficient rendering using Unicode blocks and ANSI colors
- **Animation Modes**: Play once, loop infinitely, or loop N times
- **Color Modes**: Automatic detection of true color (24-bit RGB) or 256-color terminals

## Architecture

### Core Components

1. **sprites.rs** - Data structures for sprites, frames, and color palettes
2. **terminal.rs** - Terminal rendering with ANSI escape codes
3. **animator.rs** - Animation playback and frame sequencing
4. **examples.rs** - Pre-built example sprites and animations

## Quick Start

### Creating a Simple Static Sprite

```rust
use cmdai::rendering::*;

// Define a color palette
let palette = ColorPalette::from_hex_strings(&[
    "#000000", // 0: Transparent
    "#FF0000", // 1: Red
    "#00FF00", // 2: Green
])?
.with_transparent(0);

// Create pixel data (4x4 sprite)
// Each value is an index into the color palette
let pixels = vec![
    0, 1, 1, 0,
    1, 1, 1, 1,
    0, 1, 1, 0,
    0, 2, 2, 0,
];

// Create a frame with 100ms duration
let frame = SpriteFrame::new(4, 4, pixels, 100)?;

// Create the sprite
let sprite = Sprite::new("my_sprite".to_string(), palette, vec![frame])?;

// Render it
let animator = Animator::new();
animator.render_static(&sprite)?;
```

### Creating an Animation

```rust
use cmdai::rendering::*;

// Create multiple frames
let palette = ColorPalette::from_hex_strings(&["#000000", "#FF0000"])?
    .with_transparent(0);

let frame1 = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 200)?;
let frame2 = SpriteFrame::new(2, 2, vec![1, 0, 0, 1], 200)?;

// Create sprite with multiple frames
let sprite = Sprite::new(
    "animated".to_string(),
    palette,
    vec![frame1, frame2]
)?;

// Create and play animation
let mut animation = Animation::new(sprite, AnimationMode::LoopN(3));
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### Using Pre-built Examples

```rust
use cmdai::rendering::examples::*;

// Create a walking character animation
let walking_sprite = create_walking_animation()?;
let mut animation = Animation::new(walking_sprite, AnimationMode::Loop);

let animator = Animator::new();
animator.play(&mut animation).await?;
```

## Color Palette System

### Creating Palettes

```rust
// From hex strings
let palette = ColorPalette::from_hex_strings(&[
    "#FF0000", // Red
    "#00FF00", // Green
    "#0000FF", // Blue
])?;

// From RGB values
let color = Color::new(255, 128, 0); // Orange
```

### Transparency

```rust
let palette = ColorPalette::from_hex_strings(&[
    "#000000",  // Index 0
    "#FF0000",  // Index 1
])?
.with_transparent(0); // Make index 0 transparent
```

## Animation Modes

```rust
// Play once and stop
AnimationMode::Once

// Loop indefinitely
AnimationMode::Loop

// Loop N times
AnimationMode::LoopN(5) // Loop 5 times
```

## Terminal Rendering

### Render at Specific Position

```rust
let animator = Animator::new();
let frame = sprite.frame(0).unwrap();

// Render at row 10, column 5
animator.renderer().render_frame_at(
    frame,
    sprite.palette(),
    10,
    5
)?;
```

### Terminal Control

```rust
let renderer = TerminalRenderer::new();

// Clear screen
renderer.clear_screen()?;

// Move cursor
renderer.move_cursor(10, 5)?;

// Hide/show cursor
renderer.hide_cursor()?;
renderer.show_cursor()?;
```

## Example Sprites

The module includes several pre-built example sprites:

### 1. Idle Character (8x8)
A static humanoid character sprite.
```rust
let sprite = create_idle_character()?;
```

### 2. Walking Animation (8x8, 4 frames)
A character walking with leg and arm movement.
```rust
let sprite = create_walking_animation()?;
```

### 3. Heart Pulse (6x6, 3 frames)
A pulsing heart animation.
```rust
let sprite = create_heart_animation()?;
```

### 4. Spinning Coin (8x8, 4 frames)
A coin rotating in 3D space.
```rust
let sprite = create_coin_animation()?;
```

### 5. Loading Spinner (5x5, 8 frames)
A circular loading indicator.
```rust
let sprite = create_spinner_animation()?;
```

## Running the Demo

```bash
# Run the interactive demo
cargo run --example sprite_demo

# This will show:
# - Static sprite rendering
# - Walking animation
# - Heart pulse effect
# - Spinning coin
# - Loading spinner
```

## Technical Details

### Pixel Representation

Each pixel is represented by a Unicode block character (█, U+2588) with ANSI color codes:

- **True Color Mode**: 24-bit RGB (`\x1b[38;2;R;G;Bm`)
- **256-Color Mode**: ANSI 256-color palette (`\x1b[38;5;Cm`)

The renderer automatically detects terminal capabilities and uses the best available mode.

### Performance

- Frame timing uses `tokio::time::sleep` for precise delays
- Minimal allocations during rendering
- Efficient color code caching
- Terminal updates are flushed immediately for smooth animation

### Memory Layout

Sprite pixel data is stored row-by-row in a flat `Vec<usize>`:

```
For a 3x3 sprite:
pixels = [0, 1, 2,  // Row 0
          3, 4, 5,  // Row 1
          6, 7, 8]  // Row 2

Access pixel at (x, y): pixels[y * width + x]
```

## Error Handling

All operations return `RenderResult<T>`:

```rust
pub enum RenderError {
    InvalidColor(String),       // Invalid hex color format
    InvalidDimensions(String),  // Mismatched dimensions
    AnimationError(String),     // Animation playback issues
    RenderingError(String),     // Terminal rendering errors
}
```

## Testing

Run the module tests:

```bash
# Test all rendering components
cargo test rendering

# Test specific component
cargo test rendering::sprites
cargo test rendering::terminal
cargo test rendering::animator
```

## Integration with cmdai

The rendering module is designed to be used for:

1. **Loading indicators** - Show animated spinners during model loading
2. **Status displays** - Visual feedback for command generation
3. **Success/error animations** - Visual confirmation of operations
4. **Branding** - Animated logo or mascot characters

Example integration:

```rust
use cmdai::rendering::*;

async fn show_loading() -> Result<()> {
    let spinner = examples::create_spinner_animation()?;
    let mut animation = Animation::new(spinner, AnimationMode::Loop);
    let animator = Animator::new();

    // Play in background while processing
    tokio::spawn(async move {
        animator.play(&mut animation).await
    });

    // Do work...
    Ok(())
}
```

## Future Enhancements

Potential improvements:

- [ ] Sprite sheet file format support (PNG, JSON)
- [ ] Layer compositing for complex sprites
- [ ] Sprite transformations (scale, rotate, flip)
- [ ] Collision detection for interactive sprites
- [ ] Sound effect integration
- [ ] Multiple sprites rendering simultaneously
- [ ] Palette swapping for color variations
- [ ] Sprite editor tool

## Credits

Built for the cmdai project - Natural Language to Shell Command CLI Tool.

Uses Unicode block characters and ANSI escape codes for rendering.

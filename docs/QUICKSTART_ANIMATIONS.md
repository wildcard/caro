# Quick Start: Terminal Sprite Animations

Get up and running with terminal sprite animations in 5 minutes.

## What You'll Build

A simple animated sprite that plays in your terminal using colored Unicode blocks.

## Prerequisites

- Rust toolchain installed
- A terminal with color support (most modern terminals)
- 5 minutes of your time

## Step 1: Run the Demo (30 seconds)

See what's possible by running the built-in demos:

```bash
# Make sure Rust environment is loaded
. "$HOME/.cargo/env"

# Run the sprite animation demo
cargo run --example sprite_demo

# Run the ANSI art demo
cargo run --example ansi_art_demo

# Run the DurDraw format demo
cargo run --example durdraw_demo

# Run the Aseprite format demo
cargo run --example aseprite_demo
```

## Step 2: Create Your First Animation (2 minutes)

Create a new file `examples/my_first_animation.rs`:

```rust
//! My first sprite animation
//!
//! Run with: cargo run --example my_first_animation

use cmdai::rendering::{
    Animation, AnimationMode, Animator,
    Color, ColorPalette, Sprite, SpriteFrame,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple 4x4 blinking square

    // Define colors
    let palette = ColorPalette::new(vec![
        Color::from_hex("#000000")?, // 0: Black (transparent)
        Color::from_hex("#FF0000")?, // 1: Red
        Color::from_hex("#FFFF00")?, // 2: Yellow
    ]).with_transparent(0);

    // Frame 1: Red square
    let frame1 = SpriteFrame::new(
        4, 4,
        vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ],
        500, // Display for 500ms
    )?;

    // Frame 2: Yellow square
    let frame2 = SpriteFrame::new(
        4, 4,
        vec![
            0, 0, 0, 0,
            0, 2, 2, 0,
            0, 2, 2, 0,
            0, 0, 0, 0,
        ],
        500, // Display for 500ms
    )?;

    // Create sprite with both frames
    let sprite = Sprite::new(
        "blinking_square".to_string(),
        palette,
        vec![frame1, frame2],
    )?;

    // Create animation that loops 5 times
    let mut animation = Animation::new(sprite, AnimationMode::LoopN(5));

    // Play it!
    let animator = Animator::new();
    animator.play(&mut animation).await?;

    println!("\nAnimation complete!");

    Ok(())
}
```

## Step 3: Run Your Animation (1 minute)

```bash
cargo run --example my_first_animation
```

You should see a red and yellow square blinking in your terminal!

## What Just Happened?

You created:

1. **Color Palette**: Defined colors using hex codes
2. **Frames**: Two 4x4 pixel frames with different colors
3. **Sprite**: Combined frames with the palette
4. **Animation**: Set it to loop 5 times
5. **Animator**: Played it in the terminal

## Next Steps

### Want to Learn More?

- **[Animation Guide](ANIMATION_GUIDE.md)**: Complete guide to all features
- **[Designer Guide](DESIGNER_GUIDE.md)**: Create animations with professional tools
- **[Testing Guide](TESTING_ANIMATIONS.md)**: Test and validate your animations

### Experiment with These Modifications

1. **Change Colors**: Try `#00FF00` (green) or `#0000FF` (blue)
2. **Adjust Timing**: Change `500` to `100` for faster animation
3. **Add More Frames**: Create frame3, frame4, etc.
4. **Change Animation Mode**:
   - `AnimationMode::Once` - Play once and stop
   - `AnimationMode::Loop` - Loop forever (use Ctrl+C to stop)
   - `AnimationMode::LoopN(10)` - Loop 10 times

### Try Different Sizes

Make a larger 8x8 smiley face:

```rust
let frame = SpriteFrame::new(
    8, 8,
    vec![
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        1, 1, 2, 1, 1, 2, 1, 1,  // Eyes
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 2, 1, 1, 1, 1, 2, 1,  // Smile
        1, 1, 2, 2, 2, 2, 1, 1,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
    ],
    1000,
)?;
```

## Understanding the Coordinate System

Pixels are indexed **row-by-row, left-to-right**:

```
4x4 grid:
┌─────────────┐
│ 0  1  2  3  │  Row 0
│ 4  5  6  7  │  Row 1
│ 8  9 10 11  │  Row 2
│12 13 14 15  │  Row 3
└─────────────┘
```

So for a 4x4 frame, your pixel array has 16 elements (4 × 4).

## File Format Support

The rendering system supports three file formats:

### 1. ANSI Art (.ans, .txt)
Traditional ANSI escape sequences with colors

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

let (frame, metadata) = AnsiParser::load_file("artwork.ans")?;
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

### 2. DurDraw (.dur)
Modern JSON-based format

```rust
use cmdai::rendering::DurDrawParser;

let (frame, metadata) = DurDrawParser::load_with_metadata("artwork.dur")?;
renderer.print_ansi_frame(&frame)?;
```

### 3. Aseprite (.ase, .aseprite)
Binary format from Aseprite pixel art editor

```rust
use cmdai::rendering::AsepriteParser;

let ase_file = AsepriteParser::load_file("sprite.ase")?;
let sprite = AsepriteParser::to_sprite(&ase_file)?;

let mut animation = Animation::new(sprite, AnimationMode::Loop);
animator.play(&mut animation).await?;
```

## Common Issues

### Animation Plays Too Fast
Increase the frame duration (in milliseconds):
```rust
SpriteFrame::new(width, height, pixels, 1000)? // 1 second per frame
```

### Colors Look Wrong
Check your terminal supports true color:
```bash
echo $COLORTERM  # Should show "truecolor" or "24bit"
```

### Dimension Mismatch Error
Make sure your pixel array matches width × height:
```rust
// 3x3 frame needs exactly 9 pixels
SpriteFrame::new(3, 3, vec![0,1,2,3,4,5,6,7,8], 100)?
```

## Performance Tips

- **Small sprites** (8x8 to 32x32) work best in terminals
- **Frame duration** of 50-200ms feels smooth
- **Avoid** very large sprites (>64x64) for performance

## Summary

You've learned:
- How to create a simple sprite animation
- Color palettes and hex codes
- Frame timing and animation modes
- The pixel coordinate system
- Supported file formats

Ready to create more complex animations? Check out the **[Animation Guide](ANIMATION_GUIDE.md)** next!

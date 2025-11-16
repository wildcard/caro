# Terminal Sprite Animation System - Complete Guide

A comprehensive guide to creating and rendering pixel art animations in the terminal using cmdai's sprite animation system.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Supported File Formats](#supported-file-formats)
- [Creating Animations Programmatically](#creating-animations-programmatically)
- [Working with ANSI Art Files](#working-with-ansi-art-files)
- [Working with DurDraw Files](#working-with-durdraw-files)
- [Working with Aseprite Files](#working-with-aseprite-files)
- [Animation Control](#animation-control)
- [Advanced Rendering](#advanced-rendering)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The terminal sprite animation system enables you to:

- Render pixel art using colored Unicode block characters (█)
- Create smooth animations with frame timing control
- Parse and display traditional ANSI art files
- Load modern DurDraw JSON format files
- Import Aseprite pixel art editor files
- Support both true color (24-bit RGB) and 256-color modes
- Control animation playback with multiple modes

### What Makes This Unique?

Unlike traditional terminal applications, this system:
- Uses **Unicode blocks** instead of ASCII characters for pixel-perfect rendering
- Supports **true color** (16.7 million colors) on compatible terminals
- Provides **async animation playback** using the tokio runtime
- Offers **multiple file format parsers** for maximum compatibility
- Includes **frame timing** for smooth, controlled animations

## Core Concepts

### Architecture

```
┌─────────────┐
│   Sprite    │  Contains: name, palette, frames
└──────┬──────┘
       │
       ├─► ColorPalette (array of RGB colors)
       │
       └─► SpriteFrame[] (pixel data + duration)
              │
              └─► pixels: Vec<usize> (palette indices)
```

### Key Types

#### Color

Represents an RGB color value.

```rust
use cmdai::rendering::Color;

// From RGB values (0-255)
let red = Color::new(255, 0, 0);

// From hex string
let blue = Color::from_hex("#0000FF")?;
let green = Color::from_hex("00FF00")?; // # is optional

// Convert to hex
assert_eq!(red.to_hex(), "#FF0000");

// Get ANSI 256-color approximation
let ansi_code = red.to_ansi_256(); // Returns u8
```

#### ColorPalette

A collection of colors with optional transparency.

```rust
use cmdai::rendering::ColorPalette;

// Create from Color objects
let colors = vec![
    Color::new(0, 0, 0),
    Color::new(255, 0, 0),
    Color::new(0, 255, 0),
];
let palette = ColorPalette::new(colors);

// Create from hex strings (easier)
let palette = ColorPalette::from_hex_strings(&[
    "#000000",  // Black
    "#FF0000",  // Red
    "#00FF00",  // Green
    "#0000FF",  // Blue
])?;

// Set transparency
let palette = palette.with_transparent(0); // Index 0 is transparent

// Access colors
let color = palette.get(1); // Returns Option<&Color>
let is_clear = palette.is_transparent(0); // true
```

#### SpriteFrame

A single frame of animation with pixel data and duration.

```rust
use cmdai::rendering::SpriteFrame;

// Create a 4x4 frame (16 pixels total)
let pixels = vec![
    0, 1, 1, 0,  // Row 0
    1, 2, 2, 1,  // Row 1
    1, 2, 2, 1,  // Row 2
    0, 1, 1, 0,  // Row 3
];

let frame = SpriteFrame::new(
    4,      // width
    4,      // height
    pixels, // pixel data (palette indices)
    100,    // duration in milliseconds
)?;

// Access frame properties
let width = frame.width();
let height = frame.height();
let duration = frame.duration_ms();

// Get individual pixel
let pixel_index = frame.get_pixel(x, y); // Returns Option<usize>

// Get all pixel data
let all_pixels = frame.pixels(); // Returns &[usize]
```

#### Sprite

A complete sprite with palette and one or more frames.

```rust
use cmdai::rendering::Sprite;

let sprite = Sprite::new(
    "my_sprite".to_string(),
    palette,
    vec![frame1, frame2, frame3],
)?;

// Access sprite properties
let name = sprite.name();
let (width, height) = sprite.dimensions();
let frame_count = sprite.frame_count();
let is_static = sprite.is_static(); // true if only 1 frame

// Get frames
let all_frames = sprite.frames();
let first_frame = sprite.frame(0); // Returns Option<&SpriteFrame>
```

#### Animation

Controls playback of a sprite.

```rust
use cmdai::rendering::{Animation, AnimationMode};

let mut animation = Animation::new(sprite, AnimationMode::Loop);

// Animation modes
AnimationMode::Once       // Play once and stop
AnimationMode::Loop       // Loop forever
AnimationMode::LoopN(5)   // Loop 5 times

// Control animation
animation.reset();                         // Back to frame 0
animation.set_mode(AnimationMode::Once);   // Change mode
let is_done = animation.is_complete();     // Check if finished

// Frame navigation (manual)
let current = animation.current_frame();   // Get current frame
let should_continue = animation.advance(); // Move to next frame
```

#### Animator

Renders animations to the terminal.

```rust
use cmdai::rendering::Animator;

let animator = Animator::new();

// Play animation (async)
animator.play(&mut animation).await?;

// Play at specific position
animator.play_at(&mut animation, row, col).await?;

// Render static sprite
animator.render_static(&sprite)?;

// Access renderer directly
let renderer = animator.renderer();
```

## Supported File Formats

The system supports three file formats for maximum compatibility:

| Format | Extension | Type | Best For |
|--------|-----------|------|----------|
| ANSI Art | .ans, .txt | Text with escape codes | Traditional BBS art, text art |
| DurDraw | .dur | JSON | Modern editable format, metadata |
| Aseprite | .ase, .aseprite | Binary | Professional pixel art, animations |

### Format Comparison

```
ANSI Art:
  ✓ Human-readable (text)
  ✓ Wide tool support
  ✓ SAUCE metadata
  ✗ Limited to 16 colors
  ✗ Character-based (not pixel-based)

DurDraw:
  ✓ Human-editable JSON
  ✓ Full RGB color support
  ✓ Metadata support
  ✓ Custom palettes
  ✗ Less tool support

Aseprite:
  ✓ Industry-standard pixel art tool
  ✓ Full animation support
  ✓ Layers and blending
  ✓ Professional workflow
  ✗ Binary format (not editable)
  ✗ Requires Aseprite software
```

## Creating Animations Programmatically

### Example 1: Simple Static Sprite

Create a 3x3 heart icon:

```rust
use cmdai::rendering::{Color, ColorPalette, Sprite, SpriteFrame, Animator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create palette
    let palette = ColorPalette::new(vec![
        Color::from_hex("#000000")?, // 0: Transparent
        Color::from_hex("#FF1744")?, // 1: Red
        Color::from_hex("#F50057")?, // 2: Pink
    ]).with_transparent(0);

    // Create 3x3 heart shape
    let pixels = vec![
        1, 0, 1,  // ♥ ♥
        2, 2, 2,  //  ♥♥♥
        0, 2, 0,  //   ♥
    ];

    let frame = SpriteFrame::new(3, 3, pixels, 1000)?;
    let sprite = Sprite::new("heart".to_string(), palette, vec![frame])?;

    // Render it
    let animator = Animator::new();
    animator.render_static(&sprite)?;

    Ok(())
}
```

### Example 2: Bouncing Ball Animation

Create a ball that bounces up and down:

```rust
use cmdai::rendering::{
    Animation, AnimationMode, Animator,
    Color, ColorPalette, Sprite, SpriteFrame,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let palette = ColorPalette::new(vec![
        Color::from_hex("#000000")?, // 0: Transparent
        Color::from_hex("#2196F3")?, // 1: Blue
        Color::from_hex("#64B5F6")?, // 2: Light blue
    ]).with_transparent(0);

    // Frame 1: Ball at top
    let frame1 = SpriteFrame::new(5, 5, vec![
        0, 1, 2, 1, 0,
        1, 2, 2, 2, 1,
        0, 1, 2, 1, 0,
        0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ], 100)?;

    // Frame 2: Ball in middle
    let frame2 = SpriteFrame::new(5, 5, vec![
        0, 0, 0, 0, 0,
        0, 1, 2, 1, 0,
        1, 2, 2, 2, 1,
        0, 1, 2, 1, 0,
        0, 0, 0, 0, 0,
    ], 100)?;

    // Frame 3: Ball at bottom (squashed)
    let frame3 = SpriteFrame::new(5, 5, vec![
        0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
        1, 2, 2, 2, 1,
        0, 1, 1, 1, 0,
    ], 100)?;

    // Create sprite and animation
    let sprite = Sprite::new(
        "bouncing_ball".to_string(),
        palette,
        vec![frame1, frame2, frame3, frame2.clone()], // Bounce back
    )?;

    let mut animation = Animation::new(sprite, AnimationMode::LoopN(3));
    let animator = Animator::new();
    animator.play(&mut animation).await?;

    Ok(())
}
```

### Example 3: Multi-Frame Character Walk Cycle

Create a walking character animation:

```rust
use cmdai::rendering::{Animation, AnimationMode, Animator};
use cmdai::rendering::examples::create_walking_animation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use built-in example (or create your own)
    let sprite = create_walking_animation()?;

    println!("Walking animation: {} frames", sprite.frame_count());

    let mut animation = Animation::new(sprite, AnimationMode::LoopN(2));
    let animator = Animator::new();
    animator.play(&mut animation).await?;

    Ok(())
}
```

### Creating Reusable Sprite Functions

Organize your sprites into functions:

```rust
use cmdai::rendering::{Color, ColorPalette, Sprite, SpriteFrame, RenderResult};

pub fn create_coin_sprite() -> RenderResult<Sprite> {
    let palette = ColorPalette::new(vec![
        Color::from_hex("#000000")?, // 0: Transparent
        Color::from_hex("#FFD700")?, // 1: Gold
        Color::from_hex("#FFA500")?, // 2: Orange
        Color::from_hex("#8B7500")?, // 3: Dark gold
    ]).with_transparent(0);

    // Frame 1: Front view
    let frame1 = SpriteFrame::new(6, 6, vec![
        0, 0, 1, 1, 0, 0,
        0, 1, 2, 2, 1, 0,
        1, 2, 3, 3, 2, 1,
        1, 2, 3, 3, 2, 1,
        0, 1, 2, 2, 1, 0,
        0, 0, 1, 1, 0, 0,
    ], 150)?;

    // Frame 2: Side view (thin)
    let frame2 = SpriteFrame::new(6, 6, vec![
        0, 0, 0, 0, 0, 0,
        0, 0, 1, 1, 0, 0,
        0, 1, 2, 2, 1, 0,
        0, 1, 2, 2, 1, 0,
        0, 0, 1, 1, 0, 0,
        0, 0, 0, 0, 0, 0,
    ], 150)?;

    // Frame 3: Edge view (very thin)
    let frame3 = SpriteFrame::new(6, 6, vec![
        0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
        0, 0, 1, 1, 0, 0,
        0, 0, 1, 1, 0, 0,
        0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ], 150)?;

    Sprite::new(
        "spinning_coin".to_string(),
        palette,
        vec![frame1, frame2, frame3, frame2.clone()], // Spin back
    )
}
```

## Working with ANSI Art Files

ANSI art files (.ans, .txt) contain text with ANSI escape sequences for colors.

### Loading ANSI Files

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

// Load from file
let (frame, sauce_metadata) = AnsiParser::load_file("artwork.ans")?;

// Display metadata (if available)
if let Some(metadata) = sauce_metadata {
    println!("Title: {}", metadata.title);
    println!("Author: {}", metadata.author);
    println!("Group: {}", metadata.group);
    println!("Date: {}", metadata.date);
}

// Render to terminal
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

### Parsing ANSI from Bytes

```rust
use cmdai::rendering::AnsiParser;

let ansi_data = b"\x1b[31mRed Text\x1b[0m \x1b[32mGreen Text\x1b[0m\n";
let (frame, metadata) = AnsiParser::parse_bytes(ansi_data)?;

println!("Dimensions: {}x{}", frame.width(), frame.height());
```

### Converting ANSI to Sprite

Convert ANSI art to a sprite for animation:

```rust
use cmdai::rendering::{AnsiParser, Animation, AnimationMode, Animator};

// Load ANSI file
let (frame, _) = AnsiParser::load_file("logo.ans")?;

// Convert to sprite (1000ms duration per frame)
let sprite = AnsiParser::ansi_to_sprite(&frame, "logo".to_string(), 1000)?;

// Now you can animate it
let mut animation = Animation::new(sprite, AnimationMode::Once);
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### ANSI Frame Structure

```rust
use cmdai::rendering::{AnsiFrame, AnsiCell};

let (frame, _) = AnsiParser::load_file("art.ans")?;

// Access dimensions
let width = frame.width();
let height = frame.height();

// Get individual cells
if let Some(cell) = frame.get_cell(x, y) {
    println!("Character: '{}'", cell.character);
    println!("FG Color: {}", cell.fg_color.to_hex());
    println!("BG Color: {}", cell.bg_color.to_hex());
}
```

### Creating ANSI Art Programmatically

```rust
let ansi = b"\x1b[0m\x1b[1;37;44m╔═══════╗\n\x1b[1;37;44m║ TITLE ║\n\x1b[1;37;44m╚═══════╝\x1b[0m\n";
let (frame, _) = AnsiParser::parse_bytes(ansi)?;
```

Common ANSI codes:
- `\x1b[0m` - Reset all attributes
- `\x1b[31m` - Red foreground
- `\x1b[1;33m` - Bold yellow
- `\x1b[44m` - Blue background
- `\x1b[1;37;41m` - Bold white on red

## Working with DurDraw Files

DurDraw (.dur) is a JSON-based format for ANSI art with full RGB support.

### Loading DurDraw Files

```rust
use cmdai::rendering::DurDrawParser;

// Load file
let (frame, metadata) = DurDrawParser::load_with_metadata("artwork.dur")?;

// Display metadata
println!("Title: {}", metadata.title);
println!("Author: {}", metadata.author);
println!("Dimensions: {}x{}", frame.width(), frame.height());

// Render
let renderer = TerminalRenderer::new();
renderer.print_ansi_frame(&frame)?;
```

### DurDraw File Structure

```json
{
  "version": "1.0",
  "title": "My Artwork",
  "author": "Artist Name",
  "group": "Art Collective",
  "date": "20240115",
  "width": 40,
  "height": 20,
  "palette": [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255]
  ],
  "data": [
    {
      "char": "█",
      "fg": [255, 0, 0],
      "bg": [0, 0, 0],
      "attr": 0
    }
  ]
}
```

### Creating DurDraw Files Programmatically

```rust
use cmdai::rendering::{DurDrawFile, DurDrawCell, DurDrawColor};

let dur_file = DurDrawFile {
    version: "1.0".to_string(),
    title: "Test Art".to_string(),
    author: "Demo".to_string(),
    group: "".to_string(),
    date: chrono::Local::now().format("%Y%m%d").to_string(),
    width: 5,
    height: 1,
    palette: vec![],
    data: vec![
        DurDrawCell {
            char: "H".to_string(),
            fg: DurDrawColor::Named("red".to_string()),
            bg: DurDrawColor::Rgb([0, 0, 0]),
            attr: 1, // Bold
        },
        DurDrawCell {
            char: "e".to_string(),
            fg: DurDrawColor::Hex("#00FF00".to_string()),
            bg: DurDrawColor::Rgb([0, 0, 0]),
            attr: 0,
        },
        // ... more cells
    ],
};

// Convert to ANSI frame
let frame = DurDrawParser::to_ansi_frame(&dur_file)?;

// Save to file
let json = serde_json::to_string_pretty(&dur_file)?;
std::fs::write("output.dur", json)?;
```

### DurDraw Color Formats

DurDraw supports four color format types:

```rust
use cmdai::rendering::DurDrawColor;

// 1. RGB array
let color1 = DurDrawColor::Rgb([255, 128, 0]);

// 2. Hex string
let color2 = DurDrawColor::Hex("#FF8000".to_string());

// 3. Palette index
let color3 = DurDrawColor::Index(5);

// 4. Named color
let color4 = DurDrawColor::Named("red".to_string());
```

Named colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`

### Converting ANSI to DurDraw

```rust
use cmdai::rendering::{AnsiParser, DurDrawParser};

// Load ANSI file
let (ansi_frame, _) = AnsiParser::load_file("original.ans")?;

// Convert to DurDraw
let dur_file = DurDrawParser::from_ansi_frame(
    &ansi_frame,
    "Converted Artwork".to_string(),
    "Converter".to_string(),
)?;

// Save as JSON
let json = serde_json::to_string_pretty(&dur_file)?;
std::fs::write("converted.dur", json)?;
```

## Working with Aseprite Files

Aseprite (.ase, .aseprite) is the binary format from the Aseprite pixel art editor.

### Loading Aseprite Files

```rust
use cmdai::rendering::{AsepriteParser, Animation, AnimationMode, Animator};

// Load .ase file
let ase_file = AsepriteParser::load_file("sprite.ase")?;

// Display file information
println!("Dimensions: {}x{}", ase_file.header.width, ase_file.header.height);
println!("Frames: {}", ase_file.header.frames);
println!("Color depth: {} bits", ase_file.header.color_depth);
println!("Layers: {}", ase_file.layers.len());

// Convert to Sprite
let sprite = AsepriteParser::to_sprite(&ase_file)?;

// Animate it
let mut animation = Animation::new(sprite, AnimationMode::Loop);
let animator = Animator::new();
animator.play(&mut animation).await?;
```

### Aseprite Features Supported

- **Multiple frames** with individual durations
- **Layers** with visibility and opacity
- **Alpha blending** for layer compositing
- **Zlib-compressed cel data**
- **Color palettes** (up to 256 colors)
- **Raw and linked cels**

### Aseprite Color Depths

| Depth | Type | Description |
|-------|------|-------------|
| 32-bit | RGBA | Full color with alpha channel |
| 16-bit | Grayscale | Grayscale with alpha |
| 8-bit | Indexed | Palette-based colors |

### Workflow with Aseprite

1. **Create animation in Aseprite**
   - Draw your sprite frames
   - Set frame durations
   - Use layers for organization
   - Export as .ase format

2. **Load in Rust**
   ```rust
   let ase_file = AsepriteParser::load_file("character.ase")?;
   let sprite = AsepriteParser::to_sprite(&ase_file)?;
   ```

3. **Play animation**
   ```rust
   let mut animation = Animation::new(sprite, AnimationMode::Loop);
   animator.play(&mut animation).await?;
   ```

### Accessing Aseprite Internals

```rust
use cmdai::rendering::AsepriteParser;

let ase_file = AsepriteParser::load_file("sprite.ase")?;

// Header information
let header = &ase_file.header;
println!("Magic number: 0x{:X}", header.magic);
println!("Frames: {}", header.frames);
println!("Width: {}, Height: {}", header.width, header.height);
println!("Speed: {} ms/frame", header.speed);

// Frame information
for (i, frame) in ase_file.frames.iter().enumerate() {
    println!("Frame {}: duration = {} ms", i, frame.duration);
    println!("  Cels: {}", frame.cels.len());
}

// Layer information
for layer in &ase_file.layers {
    println!("Layer: {}", layer.name);
    println!("  Visible: {}", layer.flags & 1 != 0);
    println!("  Opacity: {}", layer.opacity);
}

// Palette
if let Some(palette) = &ase_file.palette {
    println!("Palette entries: {}", palette.entries.len());
}
```

## Animation Control

### Animation Modes

```rust
use cmdai::rendering::AnimationMode;

// Play once and stop
let mode = AnimationMode::Once;

// Loop forever (use Ctrl+C to stop in terminal)
let mode = AnimationMode::Loop;

// Loop N times
let mode = AnimationMode::LoopN(5);
```

### Controlling Playback

```rust
use cmdai::rendering::{Animation, AnimationMode};

let mut animation = Animation::new(sprite, AnimationMode::LoopN(3));

// Check status
if animation.is_complete() {
    println!("Animation finished!");
}

// Change mode during playback
animation.set_mode(AnimationMode::Loop);

// Reset to beginning
animation.reset();

// Manual frame control
loop {
    let frame = animation.current_frame();
    // ... render frame ...

    if !animation.advance() {
        break; // Animation complete
    }
}
```

### Custom Frame Timing

Control animation speed by adjusting frame durations:

```rust
// Slow animation (500ms per frame)
let frame1 = SpriteFrame::new(8, 8, pixels1, 500)?;
let frame2 = SpriteFrame::new(8, 8, pixels2, 500)?;

// Fast animation (50ms per frame)
let frame1 = SpriteFrame::new(8, 8, pixels1, 50)?;
let frame2 = SpriteFrame::new(8, 8, pixels2, 50)?;

// Variable timing (useful for bounces, pauses)
let frame1 = SpriteFrame::new(8, 8, pixels1, 100)?;  // Quick
let frame2 = SpriteFrame::new(8, 8, pixels2, 100)?;  // Quick
let frame3 = SpriteFrame::new(8, 8, pixels3, 500)?;  // Pause
```

### Playing at Specific Positions

```rust
use cmdai::rendering::Animator;

let animator = Animator::new();

// Play at row 5, column 10
animator.play_at(&mut animation, 5, 10).await?;
```

## Advanced Rendering

### Direct Renderer Access

```rust
use cmdai::rendering::TerminalRenderer;

let renderer = TerminalRenderer::new();

// Render frame to string
let output = renderer.render_frame(&frame, &palette)?;
println!("{}", output);

// Print directly to terminal
renderer.print_frame(&frame, &palette)?;

// Render at specific position
renderer.render_frame_at(&frame, &palette, row, col)?;

// Screen control
renderer.clear_screen()?;
renderer.move_cursor(10, 20)?;
renderer.hide_cursor()?;
renderer.show_cursor()?;
```

### Color Mode Detection

The renderer automatically detects terminal capabilities:

```rust
let renderer = TerminalRenderer::new();

// Check environment
if std::env::var("COLORTERM").ok() == Some("truecolor".to_string()) {
    println!("Terminal supports true color!");
} else {
    println!("Using 256-color mode");
}
```

Force color mode:
```bash
# Enable true color
export COLORTERM=truecolor

# Use 256-color mode
unset COLORTERM
```

### Rendering ANSI Frames

```rust
use cmdai::rendering::{AnsiParser, TerminalRenderer};

let (frame, _) = AnsiParser::load_file("art.ans")?;
let renderer = TerminalRenderer::new();

// Render to string
let output = renderer.render_ansi_frame(&frame)?;

// Print to terminal
renderer.print_ansi_frame(&frame)?;

// Render at position
renderer.render_ansi_frame_at(&frame, 1, 1)?;
```

### Custom Rendering Loop

Create your own rendering logic:

```rust
use tokio::time::{sleep, Duration};

let animator = Animator::new();
let renderer = animator.renderer();

renderer.hide_cursor()?;
renderer.clear_screen()?;

for i in 0..10 {
    let frame = sprite.frame(i % sprite.frame_count()).unwrap();

    renderer.move_cursor(1, 1)?;
    renderer.print_frame(frame, sprite.palette())?;

    sleep(Duration::from_millis(100)).await;
}

renderer.show_cursor()?;
```

## Best Practices

### Sprite Design

1. **Keep it small**: Terminal sprites work best at 8x8 to 32x32 pixels
2. **Use transparency**: Set index 0 as transparent for clean edges
3. **Limit palette**: 8-16 colors usually looks best
4. **Test in terminal**: Colors may appear different than in editor

### Frame Timing

```rust
// Smooth animation: 60 FPS
let duration = 16; // ~16ms = 60 FPS

// Standard animation: 30 FPS
let duration = 33; // ~33ms = 30 FPS

// Retro feel: 12 FPS
let duration = 83; // ~83ms = 12 FPS

// Slow reveal: 2 FPS
let duration = 500; // 500ms = 2 FPS
```

### Color Palettes

Create reusable palettes:

```rust
// Retro 4-color palette (Game Boy style)
pub fn gameboy_palette() -> RenderResult<ColorPalette> {
    ColorPalette::from_hex_strings(&[
        "#0F380F", // Darkest green
        "#306230", // Dark green
        "#8BAC0F", // Light green
        "#9BBC0F", // Lightest green
    ])
}

// Fire palette
pub fn fire_palette() -> RenderResult<ColorPalette> {
    ColorPalette::from_hex_strings(&[
        "#000000", // Black (transparent)
        "#8B0000", // Dark red
        "#FF4500", // Orange red
        "#FFA500", // Orange
        "#FFFF00", // Yellow
        "#FFFFFF", // White (hottest)
    ])
}
```

### Performance

```rust
// Good: Reasonable size
let frame = SpriteFrame::new(16, 16, pixels, 100)?; // 256 pixels

// Caution: Large sprite
let frame = SpriteFrame::new(64, 64, pixels, 100)?; // 4096 pixels

// Avoid: Very large (terminal will struggle)
let frame = SpriteFrame::new(128, 128, pixels, 100)?; // 16384 pixels
```

### Error Handling

Always handle errors properly:

```rust
use cmdai::rendering::RenderResult;

fn create_sprite() -> RenderResult<Sprite> {
    // All these operations return Result types
    let palette = ColorPalette::from_hex_strings(&["#FF0000"])?;
    let frame = SpriteFrame::new(2, 2, vec![0, 0, 0, 0], 100)?;
    let sprite = Sprite::new("test".to_string(), palette, vec![frame])?;
    Ok(sprite)
}

// In main:
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sprite = create_sprite()?;
    // ... rest of code
    Ok(())
}
```

### Async Best Practices

The animation system uses tokio for async operations:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let animator = Animator::new();

    // Animations are async
    animator.play(&mut animation).await?;

    // Can run multiple animations sequentially
    animator.play(&mut animation1).await?;
    animator.play(&mut animation2).await?;

    Ok(())
}
```

## Troubleshooting

### Colors Look Wrong

**Problem**: Colors don't match expected values

**Solutions**:
```bash
# Check terminal color support
echo $COLORTERM

# Enable true color
export COLORTERM=truecolor

# Try different terminal emulator
# Recommended: iTerm2 (macOS), Windows Terminal, GNOME Terminal
```

### Dimension Mismatch Error

**Problem**: `Invalid dimensions` error

**Cause**: Pixel array size doesn't match width × height

```rust
// Wrong: 3x3 needs 9 pixels, but only 4 provided
let frame = SpriteFrame::new(3, 3, vec![0, 1, 2, 3], 100)?; // ERROR!

// Correct: 3x3 = 9 pixels
let frame = SpriteFrame::new(3, 3, vec![0,1,2,3,4,5,6,7,8], 100)?; // OK
```

### Animation Too Fast/Slow

**Problem**: Frame timing not right

**Solutions**:
```rust
// Increase duration (slower)
let frame = SpriteFrame::new(8, 8, pixels, 200)?; // 200ms

// Decrease duration (faster)
let frame = SpriteFrame::new(8, 8, pixels, 50)?; // 50ms

// Variable timing
let frame1 = SpriteFrame::new(8, 8, pixels1, 100)?; // Fast
let frame2 = SpriteFrame::new(8, 8, pixels2, 500)?; // Slow
```

### Invalid Palette Index

**Problem**: `Invalid palette index` error

**Cause**: Pixel value exceeds palette size

```rust
// Wrong: Palette has 3 colors (0-2), but pixel uses index 5
let palette = ColorPalette::from_hex_strings(&["#FF0000", "#00FF00", "#0000FF"])?;
let pixels = vec![0, 1, 2, 5]; // Index 5 doesn't exist!

// Correct: All indices within palette range
let pixels = vec![0, 1, 2, 1]; // All indices 0-2
```

### File Not Found

**Problem**: Cannot load file

**Solutions**:
```rust
// Use absolute path
let (frame, _) = AnsiParser::load_file("/full/path/to/file.ans")?;

// Or relative to current directory
let (frame, _) = AnsiParser::load_file("./assets/art.ans")?;

// Check if file exists first
if std::path::Path::new("art.ans").exists() {
    let (frame, _) = AnsiParser::load_file("art.ans")?;
}
```

### Transparency Not Working

**Problem**: Transparent pixels show as black

**Cause**: Forgot to set transparent index

```rust
// Wrong: No transparency set
let palette = ColorPalette::from_hex_strings(&["#000000", "#FF0000"])?;

// Correct: Set index 0 as transparent
let palette = ColorPalette::from_hex_strings(&["#000000", "#FF0000"])?
    .with_transparent(0);
```

### Compilation Errors

**Problem**: Async runtime error

**Solution**: Make sure you have `tokio` in `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

And use `#[tokio::main]`:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

## Next Steps

- **[Quick Start Guide](QUICKSTART_ANIMATIONS.md)**: Get started in 5 minutes
- **[Designer Guide](DESIGNER_GUIDE.md)**: Create animations with professional tools
- **[Testing Guide](TESTING_ANIMATIONS.md)**: Test and validate your animations

## Additional Resources

### Example Code

All examples are in the `examples/` directory:
```bash
cargo run --example sprite_demo
cargo run --example ansi_art_demo
cargo run --example durdraw_demo
cargo run --example aseprite_demo
```

### API Documentation

Generate full API docs:
```bash
cargo doc --open
```

### Source Code

- Core types: `src/rendering/sprites.rs`
- Animation: `src/rendering/animator.rs`
- Terminal: `src/rendering/terminal.rs`
- Parsers: `src/rendering/*_parser.rs`

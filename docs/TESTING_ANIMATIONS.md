# Testing Guide for Terminal Sprite Animations

A comprehensive guide to testing, validating, and debugging terminal sprite animations in cmdai.

## Table of Contents

- [Overview](#overview)
- [Running Demo Applications](#running-demo-applications)
- [Creating Test Files](#creating-test-files)
- [Local Testing](#local-testing)
- [Validation Checklist](#validation-checklist)
- [Performance Testing](#performance-testing)
- [Cross-Platform Testing](#cross-platform-testing)
- [Debugging Common Issues](#debugging-common-issues)
- [Automated Testing](#automated-testing)
- [Visual Testing](#visual-testing)
- [Best Practices](#best-practices)

## Overview

Testing terminal animations ensures:
- Animations play smoothly and correctly
- Colors render as expected across terminals
- Performance meets requirements
- Files load without errors
- Cross-platform compatibility

### Testing Philosophy

1. **Test early and often** - Don't wait until the end
2. **Test in real terminals** - Not just your development environment
3. **Test with actual data** - Use realistic sprites and animations
4. **Automate when possible** - But visual testing is still important
5. **Document issues** - Track what works and what doesn't

## Running Demo Applications

The fastest way to test the animation system is to run the built-in demos.

### Available Demos

#### 1. Sprite Demo

Tests programmatic sprite creation and animation.

```bash
# Make sure Rust environment is loaded
. "$HOME/.cargo/env"

# Run the demo
cargo run --example sprite_demo
```

**What it tests**:
- Static sprite rendering
- Multi-frame animations
- Walking cycle
- Heart pulse
- Coin spin
- Loading spinner
- Frame timing
- Animation modes (Once, Loop, LoopN)

**Expected output**:
- Press Enter prompts between demos
- Smooth animations
- Colored block characters (█)
- Performance statistics after each animation

#### 2. ANSI Art Demo

Tests ANSI escape sequence parsing and rendering.

```bash
cargo run --example ansi_art_demo
```

**What it tests**:
- ANSI escape sequence parsing
- Foreground/background colors
- Text attributes (bold, etc.)
- Box drawing characters
- SAUCE metadata parsing
- ANSI to Sprite conversion

**Expected output**:
- Colored text: "Hello World!"
- Colorful bordered boxes
- Rainbow text effects
- Gradient-like backgrounds

#### 3. DurDraw Demo

Tests JSON-based DurDraw format parsing.

```bash
cargo run --example durdraw_demo
```

**What it tests**:
- JSON parsing
- Multiple color formats (RGB, hex, named, palette index)
- Metadata support
- Custom palettes
- Bidirectional conversion (ANSI ↔ DurDraw)
- Character attributes

**Expected output**:
- JSON structure display
- Rainbow banner: "BANNER"
- 4x4 colored pattern
- Metadata information

#### 4. Aseprite Demo

Tests Aseprite binary format support.

```bash
cargo run --example aseprite_demo
```

**What it tests**:
- Aseprite format documentation
- Programmatic sprite creation
- Binary format understanding
- Frame/layer concepts
- Animation workflow

**Expected output**:
- Example sprite information
- Bouncing ball animation (4 frames)
- Documentation and instructions
- Format details

### Running All Demos

Test everything at once:

```bash
#!/bin/bash
. "$HOME/.cargo/env"

echo "=== Running All Animation Demos ==="

echo -e "\n1. Sprite Demo"
cargo run --example sprite_demo

echo -e "\n2. ANSI Art Demo"
cargo run --example ansi_art_demo

echo -e "\n3. DurDraw Demo"
cargo run --example durdraw_demo

echo -e "\n4. Aseprite Demo"
cargo run --example aseprite_demo

echo -e "\n=== All Demos Complete ==="
```

Save as `test-all-demos.sh`, make executable (`chmod +x test-all-demos.sh`), and run: `./test-all-demos.sh`

## Creating Test Files

Create dedicated test files to validate specific functionality.

### Test 1: Basic Rendering

Create `examples/test_basic_rendering.rs`:

```rust
//! Test basic sprite rendering
//!
//! Run with: cargo run --example test_basic_rendering

use cmdai::rendering::{Color, ColorPalette, Sprite, SpriteFrame, Animator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Basic Rendering ===\n");

    // Test 1: Minimal 2x2 sprite
    println!("Test 1: Minimal 2x2 sprite");
    let palette = ColorPalette::new(vec![
        Color::new(0, 0, 0),
        Color::new(255, 0, 0),
    ]);
    let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 1000)?;
    let sprite = Sprite::new("minimal".to_string(), palette, vec![frame])?;

    let animator = Animator::new();
    animator.render_static(&sprite)?;
    println!("✓ Minimal sprite renders\n");

    // Test 2: Color palette
    println!("Test 2: All palette colors");
    let palette = ColorPalette::from_hex_strings(&[
        "#FF0000", "#00FF00", "#0000FF", "#FFFF00",
        "#FF00FF", "#00FFFF", "#FFFFFF", "#000000",
    ])?;
    let pixels = vec![0,1,2,3,4,5,6,7];
    let frame = SpriteFrame::new(8, 1, pixels, 1000)?;
    let sprite = Sprite::new("colors".to_string(), palette, vec![frame])?;
    animator.render_static(&sprite)?;
    println!("✓ All colors render\n");

    println!("=== All Basic Tests Passed ===");
    Ok(())
}
```

### Test 2: Frame Timing

Create `examples/test_frame_timing.rs`:

```rust
//! Test animation frame timing accuracy
//!
//! Run with: cargo run --example test_frame_timing

use cmdai::rendering::{
    Animation, AnimationMode, Animator,
    Color, ColorPalette, Sprite, SpriteFrame,
};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Frame Timing ===\n");

    let palette = ColorPalette::new(vec![
        Color::new(0, 0, 0),
        Color::new(255, 0, 0),
        Color::new(0, 255, 0),
    ]).with_transparent(0);

    // Create 5 frames at 100ms each = 500ms total
    let mut frames = Vec::new();
    for i in 0..5 {
        let pixels = vec![
            0, 0, 0, 0,
            0, (i % 2) + 1, (i % 2) + 1, 0,
            0, (i % 2) + 1, (i % 2) + 1, 0,
            0, 0, 0, 0,
        ];
        frames.push(SpriteFrame::new(4, 4, pixels, 100)?);
    }

    let sprite = Sprite::new("timing_test".to_string(), palette, frames)?;
    let mut animation = Animation::new(sprite, AnimationMode::Once);

    println!("Playing 5 frames at 100ms each...");
    println!("Expected duration: ~500ms\n");

    let start = Instant::now();
    let animator = Animator::new();
    animator.play(&mut animation).await?;
    let elapsed = start.elapsed();

    println!("\nActual duration: {}ms", elapsed.as_millis());

    // Allow 50ms tolerance
    let expected_min = 450;
    let expected_max = 550;
    let actual = elapsed.as_millis() as u64;

    if actual >= expected_min && actual <= expected_max {
        println!("✓ Timing within acceptable range ({}-{}ms)", expected_min, expected_max);
    } else {
        println!("✗ Timing outside acceptable range!");
        println!("  Expected: {}-{}ms", expected_min, expected_max);
        println!("  Actual: {}ms", actual);
    }

    Ok(())
}
```

### Test 3: File Loading

Create `examples/test_file_loading.rs`:

```rust
//! Test loading various file formats
//!
//! Run with: cargo run --example test_file_loading

use cmdai::rendering::{AnsiParser, DurDrawParser, AsepriteParser};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: File Loading ===\n");

    // Test ANSI parsing
    println!("Test 1: ANSI byte parsing");
    let ansi_bytes = b"\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m\n";
    match AnsiParser::parse_bytes(ansi_bytes) {
        Ok((frame, _)) => {
            println!("✓ ANSI parsing works");
            println!("  Dimensions: {}x{}", frame.width(), frame.height());
        }
        Err(e) => println!("✗ ANSI parsing failed: {}", e),
    }

    // Test DurDraw JSON parsing
    println!("\nTest 2: DurDraw JSON parsing");
    let dur_json = r#"{
        "version": "1.0",
        "title": "Test",
        "author": "Test",
        "group": "",
        "date": "20240101",
        "width": 2,
        "height": 1,
        "data": [
            {"char": "█", "fg": [255,0,0], "bg": [0,0,0], "attr": 0},
            {"char": "█", "fg": [0,255,0], "bg": [0,0,0], "attr": 0}
        ],
        "palette": []
    }"#;

    match serde_json::from_str::<cmdai::rendering::DurDrawFile>(dur_json) {
        Ok(dur_file) => {
            println!("✓ DurDraw JSON parsing works");
            println!("  Title: {}", dur_file.title);
            println!("  Dimensions: {}x{}", dur_file.width, dur_file.height);
        }
        Err(e) => println!("✗ DurDraw parsing failed: {}", e),
    }

    // Test file existence checks
    println!("\nTest 3: File existence checks");
    let test_files = vec![
        ("examples/sprite_demo.rs", true),
        ("nonexistent.txt", false),
    ];

    for (path, should_exist) in test_files {
        let exists = Path::new(path).exists();
        if exists == should_exist {
            println!("✓ {} - Correctly {}", path,
                if exists { "exists" } else { "doesn't exist" });
        } else {
            println!("✗ {} - Expected {} but got {}",
                path,
                if should_exist { "exists" } else { "doesn't exist" },
                if exists { "exists" } else { "doesn't exist" });
        }
    }

    println!("\n=== File Loading Tests Complete ===");
    Ok(())
}
```

### Test 4: Error Handling

Create `examples/test_error_handling.rs`:

```rust
//! Test error handling for invalid inputs
//!
//! Run with: cargo run --example test_error_handling

use cmdai::rendering::{Color, ColorPalette, SpriteFrame};

fn main() {
    println!("=== Test: Error Handling ===\n");

    // Test 1: Invalid hex color
    println!("Test 1: Invalid hex color");
    match Color::from_hex("GGGGGG") {
        Err(_) => println!("✓ Invalid hex color rejected"),
        Ok(_) => println!("✗ Invalid hex color accepted (should fail)"),
    }

    // Test 2: Wrong hex length
    println!("\nTest 2: Wrong hex color length");
    match Color::from_hex("#FF") {
        Err(_) => println!("✓ Wrong length hex rejected"),
        Ok(_) => println!("✗ Wrong length hex accepted (should fail)"),
    }

    // Test 3: Dimension mismatch
    println!("\nTest 3: Pixel count mismatch");
    // 2x2 needs 4 pixels, but only providing 3
    match SpriteFrame::new(2, 2, vec![0, 1, 2], 100) {
        Err(_) => println!("✓ Dimension mismatch detected"),
        Ok(_) => println!("✗ Dimension mismatch not detected (should fail)"),
    }

    // Test 4: Empty palette
    println!("\nTest 4: Empty palette creation");
    let palette = ColorPalette::new(vec![]);
    if palette.is_empty() {
        println!("✓ Empty palette detected");
    } else {
        println!("✗ Empty palette not detected");
    }

    println!("\n=== Error Handling Tests Complete ===");
}
```

## Local Testing

### Manual Testing Workflow

1. **Create your animation**
2. **Write a test file** (or modify existing example)
3. **Run it**:
   ```bash
   cargo run --example my_test
   ```
4. **Observe and document**:
   - Does it render correctly?
   - Are colors accurate?
   - Is timing right?
   - Any errors in console?
5. **Iterate and fix**

### Quick Test Script

Create `quick-test.sh`:

```bash
#!/bin/bash

# Quick animation test script

echo "Building..."
. "$HOME/.cargo/env"
cargo build --example "$1" 2>&1 | grep -E "(error|warning)"

if [ $? -ne 0 ]; then
    echo "Build successful, running..."
    cargo run --example "$1"
else
    echo "Build failed, check errors above"
fi
```

Usage:
```bash
chmod +x quick-test.sh
./quick-test.sh sprite_demo
```

### Testing with Different Arguments

Test animation modes:

```rust
// Test file: examples/test_modes.rs

use cmdai::rendering::{Animation, AnimationMode, Animator};
use cmdai::rendering::examples::create_heart_animation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sprite = create_heart_animation()?;
    let animator = Animator::new();

    // Test mode from command line argument
    let args: Vec<String> = std::env::args().collect();
    let mode = if args.len() > 1 {
        match args[1].as_str() {
            "once" => AnimationMode::Once,
            "loop" => AnimationMode::Loop,
            _ => {
                let n: usize = args[1].parse().unwrap_or(3);
                AnimationMode::LoopN(n)
            }
        }
    } else {
        AnimationMode::LoopN(2)
    };

    println!("Testing with mode: {:?}", mode);
    let mut animation = Animation::new(sprite, mode);
    animator.play(&mut animation).await?;

    Ok(())
}
```

Run:
```bash
cargo run --example test_modes once
cargo run --example test_modes loop
cargo run --example test_modes 5
```

## Validation Checklist

Use this checklist for every animation:

### Visual Validation

- [ ] **Rendering**
  - [ ] Sprite renders correctly
  - [ ] No missing or corrupted pixels
  - [ ] Proper dimensions
  - [ ] Transparency works (if used)

- [ ] **Colors**
  - [ ] Colors match design
  - [ ] Colors visible in terminal
  - [ ] Palette indices correct
  - [ ] No unexpected color changes

- [ ] **Animation**
  - [ ] All frames display
  - [ ] Frame order correct
  - [ ] Smooth transitions
  - [ ] No flickering
  - [ ] Loops properly (if looping)

### Technical Validation

- [ ] **Files**
  - [ ] File loads without errors
  - [ ] Correct file format
  - [ ] File path correct
  - [ ] Metadata parsed (if applicable)

- [ ] **Performance**
  - [ ] No lag or stuttering
  - [ ] Frame rate consistent
  - [ ] Memory usage reasonable
  - [ ] CPU usage acceptable

- [ ] **Errors**
  - [ ] No console errors
  - [ ] No dimension mismatches
  - [ ] No invalid palette indices
  - [ ] Proper error messages (if any)

### Cross-Platform Validation

- [ ] **Terminals**
  - [ ] Works in primary terminal
  - [ ] Works in alternate terminals
  - [ ] Colors consistent across terminals
  - [ ] No platform-specific issues

### Validation Template

Copy this template for your tests:

```markdown
## Animation Test Report

**Animation**: [name]
**Date**: [date]
**Tester**: [your name]

### Visual Check
- [ ] Rendering: PASS / FAIL
- [ ] Colors: PASS / FAIL
- [ ] Animation: PASS / FAIL
- [ ] Comments: _____

### Technical Check
- [ ] Files: PASS / FAIL
- [ ] Performance: PASS / FAIL
- [ ] Errors: PASS / FAIL
- [ ] Comments: _____

### Platform Testing
- [ ] Primary terminal: _____
- [ ] Alternate terminals: _____
- [ ] Comments: _____

### Overall Status
- [ ] APPROVED
- [ ] NEEDS WORK
- [ ] BLOCKED

### Notes:
```

## Performance Testing

### Measuring Frame Rate

Test actual vs expected frame rate:

```rust
use tokio::time::Instant;

let start = Instant::now();
let mut frame_count = 0;

// Manually play animation
loop {
    let frame = animation.current_frame();
    animator.renderer().print_frame(frame, animation.palette())?;

    frame_count += 1;

    if !animation.advance() {
        break;
    }

    tokio::time::sleep(Duration::from_millis(frame.duration_ms())).await;
}

let elapsed = start.elapsed();
let fps = frame_count as f64 / elapsed.as_secs_f64();

println!("Frames: {}", frame_count);
println!("Time: {:.2}s", elapsed.as_secs_f64());
println!("FPS: {:.1}", fps);
```

### Performance Benchmarks

Target performance metrics:

| Metric | Target | Acceptable | Poor |
|--------|--------|------------|------|
| Frame render time | < 5ms | < 10ms | > 10ms |
| Animation startup | < 50ms | < 100ms | > 100ms |
| Memory usage | < 1MB | < 5MB | > 5MB |
| CPU usage | < 5% | < 10% | > 10% |

### Stress Testing

Test with large sprites:

```rust
// Create large sprite to test performance
let large_pixels: Vec<usize> = (0..64*64).map(|i| i % 8).collect();
let frame = SpriteFrame::new(64, 64, large_pixels, 100)?;
```

Test with many frames:

```rust
// Create animation with 60 frames
let mut frames = Vec::new();
for i in 0..60 {
    frames.push(create_frame(i)?);
}
```

### Profiling

Monitor system resources:

```bash
# macOS
top -pid $(pgrep -f sprite_demo)

# Linux
htop -p $(pgrep -f sprite_demo)

# Or use Rust profiling tools
cargo install cargo-flamegraph
cargo flamegraph --example sprite_demo
```

## Cross-Platform Testing

### Terminal Emulators to Test

**macOS**:
```bash
# iTerm2 (best color support)
# Download from: https://iterm2.com/

# Terminal.app (built-in)
# Already installed

# Alacritty (GPU-accelerated)
brew install alacritty
```

**Linux**:
```bash
# GNOME Terminal
sudo apt install gnome-terminal  # Ubuntu/Debian
sudo dnf install gnome-terminal  # Fedora

# Konsole (KDE)
sudo apt install konsole

# Alacritty
sudo apt install alacritty
```

**Windows**:
```powershell
# Windows Terminal (best)
# Install from Microsoft Store

# ConEmu
# Download from: https://conemu.github.io/

# Cmder
# Download from: https://cmder.net/
```

### Testing Color Support

Check terminal capabilities:

```bash
# Check COLORTERM
echo $COLORTERM

# Should show "truecolor" or "24bit" for best support

# Test 256 colors
for i in {0..255}; do
    printf "\x1b[48;5;%sm%3d\e[0m " "$i" "$i"
    if (( i == 15 )) || (( i > 15 )) && (( (i-15) % 6 == 0 )); then
        printf "\n"
    fi
done

# Test true color
awk 'BEGIN{
    s="/\\/\\/\\/\\/\\"; s=s s s s s s s s;
    for (colnum = 0; colnum<77; colnum++) {
        r = 255-(colnum*255/76);
        g = (colnum*510/76);
        b = (colnum*255/76);
        if (g>255) g = 510-g;
        printf "\033[48;2;%d;%d;%dm", r,g,b;
        printf "\033[38;2;%d;%d;%dm", 255-r,255-g,255-b;
        printf "%s\033[0m", substr(s,colnum+1,1);
    }
    printf "\n";
}'
```

### Platform-Specific Issues

| Platform | Issue | Workaround |
|----------|-------|------------|
| Windows cmd.exe | Limited color support | Use Windows Terminal |
| macOS Terminal.app | No true color by default | Set COLORTERM=truecolor |
| Some Linux terminals | Unicode blocks render poorly | Use different font |
| SSH sessions | Colors may not work | Enable color forwarding |

### Cross-Platform Test Script

Create `test-platforms.sh`:

```bash
#!/bin/bash

echo "=== Cross-Platform Animation Test ==="
echo "Platform: $(uname)"
echo "Terminal: $TERM"
echo "Color support: $COLORTERM"
echo ""

# Test basic rendering
echo "Running basic test..."
cargo run --example sprite_demo

# Check for errors
if [ $? -eq 0 ]; then
    echo "✓ Basic test passed"
else
    echo "✗ Basic test failed"
    exit 1
fi

# Test color rendering
echo ""
echo "Color test (you should see colors):"
printf "\x1b[31m■\x1b[32m■\x1b[34m■\x1b[33m■\x1b[35m■\x1b[36m■\x1b[0m\n"

echo ""
echo "=== Platform Test Complete ==="
```

## Debugging Common Issues

### Enable Debug Logging

Set the `RUST_LOG` environment variable:

```bash
# Show all debug logs
RUST_LOG=debug cargo run --example sprite_demo

# Show only animation module logs
RUST_LOG=cmdai::rendering=debug cargo run --example sprite_demo

# Show trace level (very verbose)
RUST_LOG=trace cargo run --example sprite_demo
```

### Debug Print Statements

Add debug output to your code:

```rust
// Debug frame information
println!("Frame: {}x{}, pixels: {}",
    frame.width(), frame.height(), frame.pixels().len());

// Debug palette
println!("Palette size: {}", palette.len());
for (i, color) in palette.colors().iter().enumerate() {
    println!("  Color {}: {}", i, color.to_hex());
}

// Debug animation state
println!("Current frame: {}/{}",
    animation.current_frame_index(),
    animation.sprite().frame_count());
```

### Common Debugging Scenarios

#### Issue: "Invalid dimensions" error

**Debug steps**:
```rust
// Print actual vs expected
let width = 4;
let height = 3;
let pixels = vec![0,1,2,3,4,5,6,7,8,9,10,11];

println!("Expected: {} pixels ({}x{})", width * height, width, height);
println!("Actual: {} pixels", pixels.len());

// Should show: Expected: 12 pixels (4x3), Actual: 12 pixels
```

#### Issue: Colors don't match

**Debug steps**:
```rust
// Print color values
for (i, color) in palette.colors().iter().enumerate() {
    println!("Palette[{}] = {} (R={}, G={}, B={})",
        i, color.to_hex(), color.r, color.g, color.b);
}

// Check pixel indices
for (i, &pixel_idx) in frame.pixels().iter().enumerate() {
    if pixel_idx >= palette.len() {
        println!("ERROR: Pixel {} uses index {}, palette only has {}",
            i, pixel_idx, palette.len());
    }
}
```

#### Issue: Animation doesn't play

**Debug steps**:
```rust
// Check animation state
println!("Animation mode: {:?}", animation.mode());
println!("Is complete: {}", animation.is_complete());
println!("Frame count: {}", animation.sprite().frame_count());

// Manually step through
for i in 0..animation.sprite().frame_count() {
    println!("Frame {}: duration = {}ms",
        i, animation.sprite().frame(i).unwrap().duration_ms());
}
```

### Debug Utilities

Create helpful debug functions:

```rust
/// Print sprite information
pub fn debug_sprite(sprite: &Sprite) {
    println!("=== Sprite Debug Info ===");
    println!("Name: {}", sprite.name());
    println!("Dimensions: {:?}", sprite.dimensions());
    println!("Frames: {}", sprite.frame_count());
    println!("Palette size: {}", sprite.palette().len());

    for (i, frame) in sprite.frames().iter().enumerate() {
        println!("  Frame {}: {}ms", i, frame.duration_ms());
    }

    println!("=========================");
}

/// Print color palette
pub fn debug_palette(palette: &ColorPalette) {
    println!("=== Palette Debug Info ===");
    println!("Colors: {}", palette.len());

    for i in 0..palette.len() {
        if let Some(color) = palette.get(i) {
            let trans = if palette.is_transparent(i) { " (TRANSPARENT)" } else { "" };
            println!("  [{}] {} = RGB({}, {}, {}){}",
                i, color.to_hex(), color.r, color.g, color.b, trans);
        }
    }

    println!("==========================");
}
```

## Automated Testing

### Unit Tests

The rendering module includes unit tests. Run them:

```bash
# Run all tests
cargo test

# Run only rendering tests
cargo test rendering::

# Run specific test
cargo test test_color_from_hex

# Run with output
cargo test -- --nocapture
```

### Integration Tests

Create `tests/animation_integration.rs`:

```rust
use cmdai::rendering::{Color, ColorPalette, Sprite, SpriteFrame};

#[test]
fn test_create_simple_sprite() {
    let palette = ColorPalette::from_hex_strings(&["#FF0000", "#00FF00"]).unwrap();
    let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 100).unwrap();
    let sprite = Sprite::new("test".to_string(), palette, vec![frame]);

    assert!(sprite.is_ok());
    let sprite = sprite.unwrap();
    assert_eq!(sprite.frame_count(), 1);
    assert_eq!(sprite.dimensions(), (2, 2));
}

#[test]
fn test_color_conversion() {
    let color = Color::from_hex("#FF5733").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 87);
    assert_eq!(color.b, 51);
    assert_eq!(color.to_hex(), "#FF5733");
}

#[test]
fn test_invalid_dimensions() {
    let palette = ColorPalette::new(vec![Color::new(0, 0, 0)]);
    // 2x2 needs 4 pixels, not 3
    let result = SpriteFrame::new(2, 2, vec![0, 1, 2], 100);
    assert!(result.is_err());
}
```

Run:
```bash
cargo test --test animation_integration
```

### Continuous Testing

Use `cargo-watch` for automatic testing:

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and test
cargo watch -x test

# Watch and run example
cargo watch -x 'run --example sprite_demo'
```

## Visual Testing

### Screenshot Testing

Document expected output:

1. **Run animation**
2. **Take screenshot** (platform-specific):
   - macOS: `Cmd+Shift+4`
   - Linux: `gnome-screenshot` or `scrot`
   - Windows: `Win+Shift+S`
3. **Save to** `docs/screenshots/`
4. **Document** in test report

### Visual Regression Testing

Compare outputs over time:

```bash
# Create baseline
cargo run --example sprite_demo > baseline.txt 2>&1

# After changes, compare
cargo run --example sprite_demo > current.txt 2>&1
diff baseline.txt current.txt
```

### Manual Visual Checklist

For each animation:

- [ ] Renders in expected position
- [ ] Colors match design spec
- [ ] No artifacts or glitches
- [ ] Smooth animation (no jitter)
- [ ] Correct size and proportions
- [ ] Transparency works (if used)
- [ ] Readable on black background
- [ ] Readable on white background

## Best Practices

### Test Organization

```
cmdai/
├── examples/
│   ├── sprite_demo.rs          # Main demos
│   ├── ansi_art_demo.rs
│   ├── durdraw_demo.rs
│   ├── aseprite_demo.rs
│   ├── test_basic_rendering.rs # Test files
│   ├── test_frame_timing.rs
│   └── test_error_handling.rs
├── tests/
│   └── animation_integration.rs # Integration tests
└── docs/
    ├── screenshots/             # Visual test results
    └── test-reports/            # Test reports
```

### Testing Workflow

1. **Before committing**:
   ```bash
   cargo test              # Run unit tests
   cargo clippy            # Check for issues
   cargo fmt --check       # Check formatting
   ```

2. **Manual testing**:
   ```bash
   cargo run --example sprite_demo
   cargo run --example ansi_art_demo
   ```

3. **Document results** in test report

4. **Commit** if all tests pass

### Test Coverage

Aim to test:

- [ ] All file formats (ANSI, DurDraw, Aseprite)
- [ ] All animation modes (Once, Loop, LoopN)
- [ ] All color modes (true color, 256-color)
- [ ] Edge cases (empty, large, malformed)
- [ ] Error conditions
- [ ] Performance benchmarks

### Continuous Improvement

- **Keep test files updated** with new features
- **Document known issues** in test reports
- **Track performance** over time
- **Add regression tests** for bugs
- **Update documentation** based on test findings

## Test Report Template

```markdown
# Animation Test Report

**Date**: 2024-01-15
**Tester**: [Name]
**Version**: [commit hash or version]
**Platform**: macOS 14.1 / iTerm2

## Tests Performed

### 1. Demo Applications
- [ ] sprite_demo - PASS
- [ ] ansi_art_demo - PASS
- [ ] durdraw_demo - PASS
- [ ] aseprite_demo - PASS

### 2. Unit Tests
```bash
cargo test
```
Result: X passed, Y failed

### 3. Visual Testing
- [ ] Colors accurate - PASS
- [ ] Animation smooth - PASS
- [ ] No artifacts - PASS

### 4. Performance
- Frame rate: 30 FPS (target: 30 FPS) ✓
- Memory usage: 2MB (target: < 5MB) ✓
- CPU usage: 3% (target: < 10%) ✓

## Issues Found
1. [Issue description]
   - Severity: High/Medium/Low
   - Steps to reproduce
   - Expected vs actual

## Recommendations
- [Suggestion 1]
- [Suggestion 2]

## Overall Assessment
- [ ] APPROVED FOR RELEASE
- [ ] NEEDS MINOR FIXES
- [ ] NEEDS MAJOR WORK

**Notes**:
```

## Additional Resources

- **[Quick Start Guide](QUICKSTART_ANIMATIONS.md)**: Get started in 5 minutes
- **[Animation Guide](ANIMATION_GUIDE.md)**: Complete API reference
- **[Designer Guide](DESIGNER_GUIDE.md)**: Create animations with design tools

## Conclusion

Regular, thorough testing ensures:
- High quality animations
- Cross-platform compatibility
- Performance optimization
- Early bug detection
- User satisfaction

**Start testing early, test often, and document everything!**

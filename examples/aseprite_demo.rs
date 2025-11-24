//! Demo of Aseprite file format parsing and rendering
//!
//! Run with: cargo run --example aseprite_demo

use cmdai::rendering::{AsepriteParser, Animator, Animation, AnimationMode, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Aseprite File Format Demo ===\n");

    println!("This demo shows how to work with Aseprite (.ase/.aseprite) files.");
    println!("Aseprite is a popular pixel art editor with a binary format that supports:");
    println!("  - Multiple animation frames");
    println!("  - Layers with opacity and blend modes");
    println!("  - Color palettes");
    println!("  - Compressed cel data (zlib)");
    println!();

    // Example 1: Creating a programmatic Aseprite-compatible sprite
    println!("Example 1: Programmatic Sprite Creation");
    println!("---------------------------------------");

    let sprite = create_example_sprite()?;
    println!("Created sprite: {}", sprite.name());
    println!("  Dimensions: {:?}", sprite.dimensions());
    println!("  Frame count: {}", sprite.frame_count());
    println!("  Palette size: {}", sprite.palette().len());
    println!();

    // Example 2: Render the sprite
    println!("Example 2: Rendering Sprite Frames");
    println!("----------------------------------");

    let renderer = TerminalRenderer::new();

    println!("Frame 0:");
    if let Some(frame) = sprite.get_frame(0) {
        let output = renderer.render_frame(frame, sprite.palette())?;
        println!("{}", output);
    }
    println!();

    // Example 3: Animation playback
    println!("Example 3: Animation Playback");
    println!("-----------------------------");
    println!("This would play the animation in a real terminal:");
    println!("  let mut animation = Animation::new(sprite, AnimationMode::Loop);");
    println!("  let animator = Animator::new();");
    println!("  animator.play(&mut animation).await?;");
    println!();

    // Example 4: File loading instructions
    println!("Example 4: Loading Aseprite Files");
    println!("---------------------------------");
    println!("To load an actual .ase or .aseprite file:");
    println!();
    println!("  use cmdai::rendering::AsepriteParser;");
    println!();
    println!("  // Load file");
    println!("  let ase_file = AsepriteParser::load_file(\"sprite.ase\")?;");
    println!();
    println!("  // Display metadata");
    println!("  println!(\"Dimensions: {}x{}\", ase_file.header.width, ase_file.header.height);");
    println!("  println!(\"Frames: {}\", ase_file.header.frames);");
    println!("  println!(\"Color depth: {} bits\", ase_file.header.color_depth);");
    println!();
    println!("  // Convert to Sprite for rendering");
    println!("  let sprite = AsepriteParser::to_sprite(&ase_file)?;");
    println!();
    println!("  // Create animation");
    println!("  let mut animation = Animation::new(sprite, AnimationMode::Loop);");
    println!("  let animator = Animator::new();");
    println!("  animator.play(&mut animation).await?;");
    println!();

    // Example 5: Format details
    println!("Example 5: Aseprite Format Details");
    println!("----------------------------------");
    println!("Binary structure:");
    println!("  - Header: 128 bytes with magic number 0xA5E0");
    println!("  - Frames: Variable length with chunks");
    println!("  - Chunks: Layer, Cel, Palette, Tags, etc.");
    println!();
    println!("Supported features:");
    println!("  ✓ Multiple frames with individual durations");
    println!("  ✓ Layer visibility and opacity");
    println!("  ✓ Alpha blending for layer compositing");
    println!("  ✓ Zlib-compressed cel data");
    println!("  ✓ Color palettes (up to 256 colors)");
    println!("  ✓ Raw and linked cels");
    println!();
    println!("Color depths:");
    println!("  - RGBA (32 bits): Full color with alpha");
    println!("  - Grayscale (16 bits): Gray + alpha");
    println!("  - Indexed (8 bits): Palette-based");
    println!();

    println!("=== Demo Complete! ===");
    println!();
    println!("Next steps:");
    println!("  1. Export pixel art from Aseprite as .ase format");
    println!("  2. Load it using AsepriteParser::load_file()");
    println!("  3. Convert to Sprite with AsepriteParser::to_sprite()");
    println!("  4. Animate with Animator::play()");

    Ok(())
}

/// Create an example sprite programmatically
fn create_example_sprite() -> Result<cmdai::rendering::Sprite, Box<dyn std::error::Error>> {
    use cmdai::rendering::{Color, ColorPalette, Sprite, SpriteFrame};

    // Create a simple 8x8 bouncing ball animation
    let palette = ColorPalette::new(vec![
        Color::from_hex("#000000")?, // 0: Black (transparent)
        Color::from_hex("#FF0000")?, // 1: Red
        Color::from_hex("#AA0000")?, // 2: Dark red
        Color::from_hex("#550000")?, // 3: Very dark red
    ]);

    // Frame 1: Ball at top
    let frame1 = SpriteFrame::new(
        8, 8,
        vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 3, 2, 2, 3, 0, 0,
            0, 3, 2, 1, 1, 2, 3, 0,
            0, 2, 1, 1, 1, 1, 2, 0,
            0, 2, 1, 1, 1, 1, 2, 0,
            0, 3, 2, 2, 2, 2, 3, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        100, // 100ms per frame
    )?;

    // Frame 2: Ball in middle
    let frame2 = SpriteFrame::new(
        8, 8,
        vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 3, 2, 2, 3, 0, 0,
            0, 3, 2, 1, 1, 2, 3, 0,
            0, 2, 1, 1, 1, 1, 2, 0,
            0, 2, 1, 1, 1, 1, 2, 0,
            0, 3, 2, 2, 2, 2, 3, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        100,
    )?;

    // Frame 3: Ball at bottom (squashed)
    let frame3 = SpriteFrame::new(
        8, 8,
        vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 3, 3, 2, 2, 3, 3, 0,
            0, 2, 1, 1, 1, 1, 2, 0,
            0, 3, 2, 2, 2, 2, 3, 0,
        ],
        100,
    )?;

    let sprite = Sprite::new(
        "bouncing_ball".to_string(),
        palette,
        vec![frame1, frame2, frame3, frame2.clone()], // Bounce back up
    )?;

    Ok(sprite)
}

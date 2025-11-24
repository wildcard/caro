//! Demo of ANSI art file parsing and rendering
//!
//! Run with: cargo run --example ansi_art_demo

use cmdai::rendering::{AnsiParser, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ANSI Art File Parser Demo ===\n");

    // Example 1: Parse simple ANSI art from bytes
    println!("Example 1: Simple ANSI Art");
    println!("---------------------------");

    let simple_ansi = create_simple_ansi_art();
    let (frame, sauce) = AnsiParser::parse_bytes(&simple_ansi)?;

    println!("Dimensions: {}x{}", frame.width(), frame.height());
    if let Some(metadata) = sauce {
        println!("SAUCE Metadata:");
        println!("  Title: {}", metadata.title);
        println!("  Author: {}", metadata.author);
        println!("  Group: {}", metadata.group);
        println!("  Date: {}", metadata.date);
    }

    println!("\nRendered output:");
    let renderer = TerminalRenderer::new();
    renderer.print_ansi_frame(&frame)?;
    println!();

    // Example 2: Colorful box
    println!("\nExample 2: Colorful Box");
    println!("---------------------------");

    let colorful_ansi = create_colorful_box();
    let (frame2, _) = AnsiParser::parse_bytes(&colorful_ansi)?;

    println!("Dimensions: {}x{}", frame2.width(), frame2.height());
    println!("\nRendered output:");
    renderer.print_ansi_frame(&frame2)?;
    println!();

    // Example 3: Convert to Sprite
    println!("\nExample 3: Convert ANSI to Sprite");
    println!("----------------------------------");

    let sprite = AnsiParser::ansi_to_sprite(&frame2, "colorful_box".to_string(), 1000)?;
    println!("Converted to sprite: {}", sprite.name());
    println!("  Palette size: {}", sprite.palette().len());
    println!("  Frame count: {}", sprite.frame_count());
    println!("  Dimensions: {:?}", sprite.dimensions());

    // Example 4: Complex colored text
    println!("\nExample 4: Complex Colored Text");
    println!("--------------------------------");

    let complex_ansi = create_complex_ansi();
    let (frame3, _) = AnsiParser::parse_bytes(&complex_ansi)?;
    renderer.print_ansi_frame(&frame3)?;
    println!();

    println!("\n=== Demo Complete! ===");
    println!("\nTo use with real ANSI files:");
    println!("  let (frame, sauce) = AnsiParser::load_file(\"artwork.ans\")?;");
    println!("  renderer.print_ansi_frame(&frame)?;");

    Ok(())
}

/// Create simple ANSI art
fn create_simple_ansi_art() -> Vec<u8> {
    let ansi = "\x1b[0m\x1b[31mHello\x1b[0m \x1b[32mWorld\x1b[0m!\n\x1b[34mThis is \x1b[1;33mbold yellow\x1b[0m\n";
    ansi.as_bytes().to_vec()
}

/// Create a colorful box
fn create_colorful_box() -> Vec<u8> {
    let mut ansi = Vec::new();

    // Top border
    ansi.extend_from_slice(b"\x1b[0m\x1b[1;37;44m");
    ansi.extend_from_slice(b"╔════════════╗\n");

    // Middle rows with different colors
    ansi.extend_from_slice(b"\x1b[1;37;41m║  Red Row   ║\n");
    ansi.extend_from_slice(b"\x1b[1;30;42m║ Green Row  ║\n");
    ansi.extend_from_slice(b"\x1b[1;37;43m║ Yellow Row ║\n");

    // Bottom border
    ansi.extend_from_slice(b"\x1b[1;37;44m");
    ansi.extend_from_slice(b"╚════════════╝\x1b[0m\n");

    ansi
}

/// Create complex ANSI art with multiple colors
fn create_complex_ansi() -> Vec<u8> {
    let mut ansi = Vec::new();

    // Rainbow text
    let colors = [31, 33, 32, 36, 34, 35]; // Red, Yellow, Green, Cyan, Blue, Magenta
    let text = "RAINBOW";

    for (i, ch) in text.chars().enumerate() {
        let color = colors[i % colors.len()];
        ansi.extend_from_slice(format!("\x1b[1;{}m{}", color, ch).as_bytes());
    }

    ansi.extend_from_slice(b"\x1b[0m\n");

    // Gradient-like effect
    for i in 0..10 {
        let bg_color = 40 + (i % 8);
        ansi.extend_from_slice(format!("\x1b[1;37;{}m  ", bg_color).as_bytes());
    }
    ansi.extend_from_slice(b"\x1b[0m\n");

    ansi
}

/// Example of creating ANSI art programmatically
#[allow(dead_code)]
fn create_ansi_smiley() -> Vec<u8> {
    let mut ansi = Vec::new();

    // Yellow circle with black features
    ansi.extend_from_slice(b"\x1b[0m");
    ansi.extend_from_slice(b"  \x1b[1;33m████████\x1b[0m  \n");
    ansi.extend_from_slice(b" \x1b[1;33m██\x1b[1;30m██\x1b[1;33m██\x1b[1;30m██\x1b[1;33m██\x1b[0m \n");
    ansi.extend_from_slice(b"\x1b[1;33m████████████\x1b[0m\n");
    ansi.extend_from_slice(b"\x1b[1;33m██\x1b[1;30m██\x1b[1;33m████\x1b[1;30m██\x1b[1;33m██\x1b[0m\n");
    ansi.extend_from_slice(b"\x1b[1;33m██\x1b[1;30m████████\x1b[1;33m██\x1b[0m\n");
    ansi.extend_from_slice(b" \x1b[1;33m██████████\x1b[0m \n");
    ansi.extend_from_slice(b"  \x1b[1;33m████████\x1b[0m  \n");

    ansi
}

//! Demo of DurDraw file format parsing and rendering
//!
//! Run with: cargo run --example durdraw_demo

use cmdai::rendering::{DurDrawParser, DurDrawFile, DurDrawCell, DurDrawColor, TerminalRenderer};
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DurDraw File Format Demo ===\n");

    // Example 1: Create a simple DurDraw file programmatically
    println!("Example 1: Creating DurDraw Data");
    println!("----------------------------------");

    let simple_dur = create_simple_durdraw();
    let json = serde_json::to_string_pretty(&simple_dur)?;
    println!("DurDraw JSON structure:");
    println!("{}\n", json);

    // Parse and render
    let frame = DurDrawParser::to_ansi_frame(&simple_dur)?;
    let metadata = DurDrawParser::to_sauce_metadata(&simple_dur);

    println!("Parsed metadata:");
    println!("  Title: {}", metadata.title);
    println!("  Author: {}", metadata.author);
    println!("  Dimensions: {}x{}", frame.width(), frame.height());
    println!("\nRendered output:");

    let renderer = TerminalRenderer::new();
    renderer.print_ansi_frame(&frame)?;
    println!();

    // Example 2: Colorful banner
    println!("\nExample 2: Colorful Banner");
    println!("---------------------------");

    let banner = create_colorful_banner();
    let frame2 = DurDrawParser::to_ansi_frame(&banner)?;
    renderer.print_ansi_frame(&frame2)?;
    println!();

    // Example 3: Complex artwork with palette
    println!("\nExample 3: Artwork with Custom Palette");
    println!("----------------------------------------");

    let artwork = create_palette_artwork();
    println!("Palette size: {}", artwork.palette.len());
    println!("Data cells: {}", artwork.data.len());

    let frame3 = DurDrawParser::to_ansi_frame(&artwork)?;
    renderer.print_ansi_frame(&frame3)?;
    println!();

    // Example 4: Round-trip conversion
    println!("\nExample 4: ANSI Frame → DurDraw → ANSI Frame");
    println!("----------------------------------------------");

    let original_frame = frame;
    let dur_from_frame = DurDrawParser::from_ansi_frame(
        &original_frame,
        "Round-trip Test".to_string(),
        "Demo".to_string(),
    )?;

    println!("Converted to DurDraw format");
    println!("  Palette entries: {}", dur_from_frame.palette.len());
    println!("  Data cells: {}", dur_from_frame.data.len());

    let recovered_frame = DurDrawParser::to_ansi_frame(&dur_from_frame)?;
    println!("  Recovered dimensions: {}x{}", recovered_frame.width(), recovered_frame.height());

    // Example 5: Color format variations
    println!("\nExample 5: Different Color Formats");
    println!("------------------------------------");

    demonstrate_color_formats();

    println!("\n=== Demo Complete! ===");
    println!("\nDurDraw format features:");
    println!("  ✓ JSON-based, human-readable");
    println!("  ✓ Full metadata support (title, author, date)");
    println!("  ✓ Multiple color formats (RGB, hex, named, palette index)");
    println!("  ✓ Character attributes (bold, blink)");
    println!("  ✓ Custom color palettes");
    println!("  ✓ Bidirectional conversion with ANSI frames");
    println!("\nTo load a .dur file:");
    println!("  let (frame, meta) = DurDrawParser::load_with_metadata(\"artwork.dur\")?;");
    println!("  renderer.print_ansi_frame(&frame)?;");

    Ok(())
}

/// Create a simple DurDraw file
fn create_simple_durdraw() -> DurDrawFile {
    DurDrawFile {
        version: "1.0".to_string(),
        title: "Hello World".to_string(),
        author: "Demo".to_string(),
        group: "cmdai".to_string(),
        date: "20240101".to_string(),
        width: 5,
        height: 1,
        data: vec![
            DurDrawCell {
                char: "H".to_string(),
                fg: DurDrawColor::Named("red".to_string()),
                bg: DurDrawColor::Rgb([0, 0, 0]),
                attr: 1, // Bold
            },
            DurDrawCell {
                char: "e".to_string(),
                fg: DurDrawColor::Named("green".to_string()),
                bg: DurDrawColor::Rgb([0, 0, 0]),
                attr: 0,
            },
            DurDrawCell {
                char: "l".to_string(),
                fg: DurDrawColor::Named("blue".to_string()),
                bg: DurDrawColor::Rgb([0, 0, 0]),
                attr: 0,
            },
            DurDrawCell {
                char: "l".to_string(),
                fg: DurDrawColor::Named("yellow".to_string()),
                bg: DurDrawColor::Rgb([0, 0, 0]),
                attr: 0,
            },
            DurDrawCell {
                char: "o".to_string(),
                fg: DurDrawColor::Named("magenta".to_string()),
                bg: DurDrawColor::Rgb([0, 0, 0]),
                attr: 0,
            },
        ],
        palette: vec![],
    }
}

/// Create a colorful banner
fn create_colorful_banner() -> DurDrawFile {
    let mut data = Vec::new();

    // Row 1: "BANNER" with rainbow colors
    let text = "BANNER";
    let colors = [
        [255, 0, 0],    // Red
        [255, 165, 0],  // Orange
        [255, 255, 0],  // Yellow
        [0, 255, 0],    // Green
        [0, 0, 255],    // Blue
        [128, 0, 128],  // Purple
    ];

    for (i, ch) in text.chars().enumerate() {
        data.push(DurDrawCell {
            char: ch.to_string(),
            fg: DurDrawColor::Rgb(colors[i % colors.len()]),
            bg: DurDrawColor::Rgb([0, 0, 0]),
            attr: 1, // Bold
        });
    }

    DurDrawFile {
        version: "1.0".to_string(),
        title: "Rainbow Banner".to_string(),
        author: "Demo".to_string(),
        group: String::new(),
        date: chrono::Local::now().format("%Y%m%d").to_string(),
        width: 6,
        height: 1,
        data,
        palette: vec![],
    }
}

/// Create artwork using a custom palette
fn create_palette_artwork() -> DurDrawFile {
    // Define a custom palette
    let palette = vec![
        DurDrawColor::Rgb([0, 0, 0]),       // 0: Black
        DurDrawColor::Rgb([255, 0, 0]),     // 1: Red
        DurDrawColor::Rgb([0, 255, 0]),     // 2: Green
        DurDrawColor::Rgb([0, 0, 255]),     // 3: Blue
        DurDrawColor::Rgb([255, 255, 0]),   // 4: Yellow
        DurDrawColor::Rgb([255, 255, 255]), // 5: White
    ];

    // Create a small pattern using palette indices
    let mut data = Vec::new();

    // 4x4 grid with palette colors
    let pattern = [
        [1, 2, 3, 4],
        [2, 3, 4, 1],
        [3, 4, 1, 2],
        [4, 1, 2, 3],
    ];

    for row in &pattern {
        for &color_idx in row {
            data.push(DurDrawCell {
                char: "█".to_string(),
                fg: DurDrawColor::Index(color_idx),
                bg: DurDrawColor::Index(0), // Black background
                attr: 0,
            });
        }
    }

    DurDrawFile {
        version: "1.0".to_string(),
        title: "Palette Art".to_string(),
        author: "Demo".to_string(),
        group: String::new(),
        date: chrono::Local::now().format("%Y%m%d").to_string(),
        width: 4,
        height: 4,
        data,
        palette,
    }
}

/// Demonstrate different color format options
fn demonstrate_color_formats() {
    println!("DurDraw supports multiple color formats:\n");

    println!("1. RGB array:     [255, 128, 64]");
    println!("   → Color { r: 255, g: 128, b: 64 }\n");

    println!("2. Hex string:    \"#FF8040\"");
    println!("   → Color { r: 255, g: 128, b: 64 }\n");

    println!("3. Palette index: 5");
    println!("   → Looks up color at palette[5]\n");

    println!("4. Named colors:  \"red\", \"green\", \"blue\"");
    println!("   → Standard ANSI color names\n");

    println!("Available named colors:");
    let names = [
        "black", "red", "green", "yellow",
        "blue", "magenta", "cyan", "white",
        "bright_red", "bright_green", "bright_yellow",
        "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
    ];

    for (i, name) in names.iter().enumerate() {
        print!("  {:<15}", name);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
    println!();
}

//! DurDraw file format parser
//!
//! DurDraw is a modern ANSI/ASCII art editor that uses a JSON-based format (.dur files)
//! for storing artwork with full color information and metadata.

use crate::rendering::{ansi_parser::*, sprites::Color, RenderError, RenderResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// DurDraw file format structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurDrawFile {
    /// File format version
    #[serde(default)]
    pub version: String,

    /// Artwork title
    #[serde(default)]
    pub title: String,

    /// Artist name
    #[serde(default)]
    pub author: String,

    /// Group/organization
    #[serde(default)]
    pub group: String,

    /// Creation date
    #[serde(default)]
    pub date: String,

    /// Canvas width in characters
    pub width: usize,

    /// Canvas height in characters
    pub height: usize,

    /// Character data (row-major order)
    pub data: Vec<DurDrawCell>,

    /// Color palette (optional)
    #[serde(default)]
    pub palette: Vec<DurDrawColor>,
}

/// A single cell in DurDraw format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurDrawCell {
    /// Character code or character
    #[serde(default)]
    pub char: String,

    /// Foreground color (RGB or palette index)
    #[serde(default)]
    pub fg: DurDrawColor,

    /// Background color (RGB or palette index)
    #[serde(default)]
    pub bg: DurDrawColor,

    /// Character attributes (bold, blink, etc.)
    #[serde(default)]
    pub attr: u8,
}

/// Color representation in DurDraw format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DurDrawColor {
    /// RGB color as array [r, g, b]
    Rgb([u8; 3]),

    /// RGB color as hex string
    Hex(String),

    /// Palette index
    Index(u8),

    /// Named color
    Named(String),
}

impl Default for DurDrawColor {
    fn default() -> Self {
        DurDrawColor::Rgb([0, 0, 0])
    }
}

impl DurDrawColor {
    /// Convert DurDrawColor to RGB Color
    pub fn to_color(&self, palette: &[Color]) -> RenderResult<Color> {
        match self {
            DurDrawColor::Rgb([r, g, b]) => Ok(Color::new(*r, *g, *b)),

            DurDrawColor::Hex(hex) => Color::from_hex(hex),

            DurDrawColor::Index(idx) => {
                palette.get(*idx as usize)
                    .cloned()
                    .ok_or_else(|| RenderError::InvalidColor(format!("Palette index {} out of range", idx)))
            }

            DurDrawColor::Named(name) => {
                // Convert named colors to RGB
                match name.to_lowercase().as_str() {
                    "black" => Ok(Color::new(0, 0, 0)),
                    "red" => Ok(Color::new(170, 0, 0)),
                    "green" => Ok(Color::new(0, 170, 0)),
                    "yellow" => Ok(Color::new(170, 85, 0)),
                    "blue" => Ok(Color::new(0, 0, 170)),
                    "magenta" => Ok(Color::new(170, 0, 170)),
                    "cyan" => Ok(Color::new(0, 170, 170)),
                    "white" => Ok(Color::new(170, 170, 170)),
                    "bright_black" | "gray" => Ok(Color::new(85, 85, 85)),
                    "bright_red" => Ok(Color::new(255, 85, 85)),
                    "bright_green" => Ok(Color::new(85, 255, 85)),
                    "bright_yellow" => Ok(Color::new(255, 255, 85)),
                    "bright_blue" => Ok(Color::new(85, 85, 255)),
                    "bright_magenta" => Ok(Color::new(255, 85, 255)),
                    "bright_cyan" => Ok(Color::new(85, 255, 255)),
                    "bright_white" => Ok(Color::new(255, 255, 255)),
                    _ => Err(RenderError::InvalidColor(format!("Unknown color name: {}", name)))
                }
            }
        }
    }
}

/// DurDraw parser
pub struct DurDrawParser;

impl DurDrawParser {
    /// Load a DurDraw file from disk
    pub fn load_file<P: AsRef<Path>>(path: P) -> RenderResult<DurDrawFile> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            RenderError::RenderingError(format!("Failed to read DurDraw file: {}", e))
        })?;

        Self::parse_json(&content)
    }

    /// Parse DurDraw JSON format
    pub fn parse_json(json: &str) -> RenderResult<DurDrawFile> {
        serde_json::from_str(json).map_err(|e| {
            RenderError::RenderingError(format!("Failed to parse DurDraw JSON: {}", e))
        })
    }

    /// Convert DurDraw file to AnsiFrame
    pub fn to_ansi_frame(dur: &DurDrawFile) -> RenderResult<AnsiFrame> {
        let mut frame = AnsiFrame::new(dur.width, dur.height);

        // Build color palette
        let palette: Vec<Color> = dur.palette.iter()
            .map(|dc| dc.to_color(&[]))
            .collect::<RenderResult<Vec<_>>>()?;

        // Process each cell
        for (idx, cell) in dur.data.iter().enumerate() {
            let y = idx / dur.width;
            let x = idx % dur.width;

            if y >= dur.height || x >= dur.width {
                continue;
            }

            // Get character (use first char or space)
            let character = cell.char.chars().next().unwrap_or(' ');

            // Convert colors
            let fg_color = cell.fg.to_color(&palette)?;
            let bg_color = cell.bg.to_color(&palette)?;

            // Parse attributes
            let bold = (cell.attr & 0x01) != 0;
            let blink = (cell.attr & 0x02) != 0;

            let ansi_cell = AnsiCell {
                character,
                fg_color,
                bg_color,
                bold,
                blink,
            };

            frame.set_cell(x, y, ansi_cell);
        }

        Ok(frame)
    }

    /// Convert DurDraw file to SAUCE metadata
    pub fn to_sauce_metadata(dur: &DurDrawFile) -> SauceMetadata {
        SauceMetadata {
            title: dur.title.clone(),
            author: dur.author.clone(),
            group: dur.group.clone(),
            date: dur.date.clone(),
            width: Some(dur.width as u16),
            height: Some(dur.height as u16),
        }
    }

    /// Load DurDraw file and return both AnsiFrame and metadata
    pub fn load_with_metadata<P: AsRef<Path>>(
        path: P,
    ) -> RenderResult<(AnsiFrame, SauceMetadata)> {
        let dur = Self::load_file(path)?;
        let frame = Self::to_ansi_frame(&dur)?;
        let metadata = Self::to_sauce_metadata(&dur);
        Ok((frame, metadata))
    }

    /// Create DurDraw file from AnsiFrame
    pub fn from_ansi_frame(
        frame: &AnsiFrame,
        title: String,
        author: String,
    ) -> RenderResult<DurDrawFile> {
        let mut data = Vec::new();
        let mut palette_map = std::collections::HashMap::new();
        let mut palette = Vec::new();

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let cell = frame.get_cell(x, y).cloned().unwrap_or_else(AnsiCell::new);

                // Build palette
                let fg_key = (cell.fg_color.r, cell.fg_color.g, cell.fg_color.b);
                let bg_key = (cell.bg_color.r, cell.bg_color.g, cell.bg_color.b);

                if !palette_map.contains_key(&fg_key) {
                    palette_map.insert(fg_key, palette.len());
                    palette.push(DurDrawColor::Rgb([
                        cell.fg_color.r,
                        cell.fg_color.g,
                        cell.fg_color.b,
                    ]));
                }

                if !palette_map.contains_key(&bg_key) {
                    palette_map.insert(bg_key, palette.len());
                    palette.push(DurDrawColor::Rgb([
                        cell.bg_color.r,
                        cell.bg_color.g,
                        cell.bg_color.b,
                    ]));
                }

                let fg_idx = palette_map[&fg_key];
                let bg_idx = palette_map[&bg_key];

                let mut attr = 0u8;
                if cell.bold {
                    attr |= 0x01;
                }
                if cell.blink {
                    attr |= 0x02;
                }

                data.push(DurDrawCell {
                    char: cell.character.to_string(),
                    fg: DurDrawColor::Index(fg_idx as u8),
                    bg: DurDrawColor::Index(bg_idx as u8),
                    attr,
                });
            }
        }

        Ok(DurDrawFile {
            version: "1.0".to_string(),
            title,
            author,
            group: String::new(),
            date: chrono::Local::now().format("%Y%m%d").to_string(),
            width: frame.width(),
            height: frame.height(),
            data,
            palette,
        })
    }

    /// Save DurDraw file to disk
    pub fn save_file<P: AsRef<Path>>(dur: &DurDrawFile, path: P) -> RenderResult<()> {
        let json = serde_json::to_string_pretty(dur).map_err(|e| {
            RenderError::RenderingError(format!("Failed to serialize DurDraw JSON: {}", e))
        })?;

        fs::write(path.as_ref(), json).map_err(|e| {
            RenderError::RenderingError(format!("Failed to write DurDraw file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_durdraw_color_rgb() {
        let color = DurDrawColor::Rgb([255, 128, 64]);
        let rgb = color.to_color(&[]).unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 64);
    }

    #[test]
    fn test_durdraw_color_hex() {
        let color = DurDrawColor::Hex("#FF8040".to_string());
        let rgb = color.to_color(&[]).unwrap();
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 64);
    }

    #[test]
    fn test_durdraw_color_named() {
        let color = DurDrawColor::Named("red".to_string());
        let rgb = color.to_color(&[]).unwrap();
        assert_eq!(rgb.r, 170);
        assert_eq!(rgb.g, 0);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_durdraw_parse_simple() {
        let json = r#"{
            "version": "1.0",
            "title": "Test",
            "author": "Tester",
            "width": 2,
            "height": 2,
            "data": [
                {"char": "A", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 0},
                {"char": "B", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0},
                {"char": "C", "fg": [0, 0, 255], "bg": [0, 0, 0], "attr": 0},
                {"char": "D", "fg": [255, 255, 0], "bg": [0, 0, 0], "attr": 0}
            ]
        }"#;

        let dur = DurDrawParser::parse_json(json).unwrap();
        assert_eq!(dur.width, 2);
        assert_eq!(dur.height, 2);
        assert_eq!(dur.data.len(), 4);
        assert_eq!(dur.title, "Test");
    }

    #[test]
    fn test_durdraw_to_ansi_frame() {
        let json = r#"{
            "width": 2,
            "height": 1,
            "data": [
                {"char": "X", "fg": [255, 0, 0], "bg": [0, 0, 0], "attr": 1},
                {"char": "Y", "fg": [0, 255, 0], "bg": [0, 0, 0], "attr": 0}
            ]
        }"#;

        let dur = DurDrawParser::parse_json(json).unwrap();
        let frame = DurDrawParser::to_ansi_frame(&dur).unwrap();

        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 1);

        let cell = frame.get_cell(0, 0).unwrap();
        assert_eq!(cell.character, 'X');
        assert_eq!(cell.fg_color.r, 255);
        assert!(cell.bold);
    }
}

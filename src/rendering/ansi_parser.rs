//! ANSI art file format parser
//!
//! Supports parsing traditional ANSI art files with escape sequences,
//! character positioning, and SAUCE metadata.

use crate::rendering::{sprites::*, RenderError, RenderResult};
use std::fs;
use std::path::Path;

/// ANSI color codes (standard 16 colors)
const ANSI_COLORS: [&str; 16] = [
    "#000000", // 0: Black
    "#AA0000", // 1: Red
    "#00AA00", // 2: Green
    "#AA5500", // 3: Brown/Yellow
    "#0000AA", // 4: Blue
    "#AA00AA", // 5: Magenta
    "#00AAAA", // 6: Cyan
    "#AAAAAA", // 7: White/Light Gray
    "#555555", // 8: Bright Black/Dark Gray
    "#FF5555", // 9: Bright Red
    "#55FF55", // 10: Bright Green
    "#FFFF55", // 11: Bright Yellow
    "#5555FF", // 12: Bright Blue
    "#FF55FF", // 13: Bright Magenta
    "#55FFFF", // 14: Bright Cyan
    "#FFFFFF", // 15: Bright White
];

/// SAUCE metadata record
#[derive(Debug, Clone)]
pub struct SauceMetadata {
    pub title: String,
    pub author: String,
    pub group: String,
    pub date: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

/// Represents a cell in ANSI art
#[derive(Debug, Clone, PartialEq)]
pub struct AnsiCell {
    pub character: char,
    pub fg_color: Color,
    pub bg_color: Color,
    pub bold: bool,
    pub blink: bool,
}

impl AnsiCell {
    fn new() -> Self {
        Self {
            character: ' ',
            fg_color: Color::new(170, 170, 170), // Default white
            bg_color: Color::new(0, 0, 0),       // Default black
            bold: false,
            blink: false,
        }
    }
}

/// ANSI art frame
pub struct AnsiFrame {
    width: usize,
    height: usize,
    cells: Vec<Vec<AnsiCell>>,
}

impl AnsiFrame {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![AnsiCell::new(); width]; height];
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&AnsiCell> {
        self.cells.get(y).and_then(|row| row.get(x))
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: AnsiCell) {
        if y < self.height && x < self.width {
            self.cells[y][x] = cell;
        }
    }
}

/// ANSI art parser
pub struct AnsiParser {
    current_fg: Color,
    current_bg: Color,
    bold: bool,
    blink: bool,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            current_fg: Color::new(170, 170, 170), // Default white
            current_bg: Color::new(0, 0, 0),       // Default black
            bold: false,
            blink: false,
        }
    }

    /// Load an ANSI file from disk
    pub fn load_file<P: AsRef<Path>>(path: P) -> RenderResult<(AnsiFrame, Option<SauceMetadata>)> {
        let content = fs::read(path.as_ref()).map_err(|e| {
            RenderError::RenderingError(format!("Failed to read ANSI file: {}", e))
        })?;

        Self::parse_bytes(&content)
    }

    /// Parse ANSI art from bytes
    pub fn parse_bytes(data: &[u8]) -> RenderResult<(AnsiFrame, Option<SauceMetadata>)> {
        // Check for SAUCE metadata
        let (ansi_data, sauce) = Self::extract_sauce(data);

        // Determine dimensions
        let (width, height) = Self::calculate_dimensions(&ansi_data, &sauce);

        let mut parser = Self::new();
        let frame = parser.parse_ansi(ansi_data, width, height)?;

        Ok((frame, sauce))
    }

    /// Extract SAUCE metadata from the end of the file
    fn extract_sauce(data: &[u8]) -> (&[u8], Option<SauceMetadata>) {
        if data.len() < 128 {
            return (data, None);
        }

        // Check for SAUCE signature at end - 128 bytes
        let sauce_start = data.len().saturating_sub(128);
        let sauce_data = &data[sauce_start..];

        if sauce_data.len() >= 5 && &sauce_data[0..5] == b"SAUCE" {
            let sauce = Self::parse_sauce(sauce_data);
            let ansi_data = &data[..sauce_start.saturating_sub(1)]; // Remove EOF marker
            (ansi_data, Some(sauce))
        } else {
            (data, None)
        }
    }

    /// Parse SAUCE metadata
    fn parse_sauce(data: &[u8]) -> SauceMetadata {
        fn read_string(data: &[u8], start: usize, len: usize) -> String {
            data.get(start..start + len)
                .map(|s| String::from_utf8_lossy(s).trim_end().to_string())
                .unwrap_or_default()
        }

        SauceMetadata {
            title: read_string(data, 7, 35),
            author: read_string(data, 42, 20),
            group: read_string(data, 62, 20),
            date: read_string(data, 82, 8),
            width: data.get(96..98).and_then(|b| Some(u16::from_le_bytes([b[0], b[1]]))),
            height: data.get(98..100).and_then(|b| Some(u16::from_le_bytes([b[0], b[1]]))),
        }
    }

    /// Calculate frame dimensions
    fn calculate_dimensions(data: &[u8], sauce: &Option<SauceMetadata>) -> (usize, usize) {
        // Try to get dimensions from SAUCE first
        if let Some(sauce) = sauce {
            if let (Some(w), Some(h)) = (sauce.width, sauce.height) {
                if w > 0 && h > 0 {
                    return (w as usize, h as usize);
                }
            }
        }

        // Otherwise scan the content for dimensions
        let mut max_x = 0;
        let mut y = 0;
        let mut x = 0;

        let mut i = 0;
        while i < data.len() {
            if data[i] == 0x1B && i + 1 < data.len() && data[i + 1] == b'[' {
                // Skip escape sequence
                i += 2;
                while i < data.len() && !data[i].is_ascii_alphabetic() {
                    i += 1;
                }
                i += 1;
            } else if data[i] == b'\n' {
                max_x = max_x.max(x);
                x = 0;
                y += 1;
                i += 1;
            } else if data[i] == b'\r' {
                i += 1;
            } else {
                x += 1;
                i += 1;
            }
        }

        max_x = max_x.max(x);
        y += 1;

        (max_x.max(80), y.max(25)) // Default to standard 80x25 if needed
    }

    /// Parse ANSI escape sequences and build frame
    fn parse_ansi(&mut self, data: &[u8], width: usize, height: usize) -> RenderResult<AnsiFrame> {
        let mut frame = AnsiFrame::new(width, height);
        let mut x = 0;
        let mut y = 0;

        let mut i = 0;
        while i < data.len() {
            if data[i] == 0x1B && i + 1 < data.len() && data[i + 1] == b'[' {
                // ANSI escape sequence
                i += 2;
                let seq_start = i;

                // Read until we hit a letter
                while i < data.len() && !data[i].is_ascii_alphabetic() {
                    i += 1;
                }

                if i < data.len() {
                    let command = data[i] as char;
                    let params = &data[seq_start..i];
                    i += 1;

                    match command {
                        'm' => {
                            // SGR - Set Graphics Rendition
                            self.apply_sgr(params);
                        }
                        'H' | 'f' => {
                            // Cursor position
                            let coords = Self::parse_params(params);
                            if coords.len() >= 2 {
                                y = (coords[0].saturating_sub(1) as usize).min(height - 1);
                                x = (coords[1].saturating_sub(1) as usize).min(width - 1);
                            }
                        }
                        'A' => {
                            // Cursor up
                            let n = Self::parse_params(params).first().copied().unwrap_or(1);
                            y = y.saturating_sub(n as usize);
                        }
                        'B' => {
                            // Cursor down
                            let n = Self::parse_params(params).first().copied().unwrap_or(1);
                            y = (y + n as usize).min(height - 1);
                        }
                        'C' => {
                            // Cursor forward
                            let n = Self::parse_params(params).first().copied().unwrap_or(1);
                            x = (x + n as usize).min(width - 1);
                        }
                        'D' => {
                            // Cursor back
                            let n = Self::parse_params(params).first().copied().unwrap_or(1);
                            x = x.saturating_sub(n as usize);
                        }
                        _ => {}
                    }
                }
            } else if data[i] == b'\n' {
                x = 0;
                y = (y + 1).min(height - 1);
                i += 1;
            } else if data[i] == b'\r' {
                x = 0;
                i += 1;
            } else {
                // Regular character
                if y < height && x < width {
                    let ch = if data[i] < 128 {
                        data[i] as char
                    } else {
                        // Try UTF-8 decode
                        match std::str::from_utf8(&data[i..]) {
                            Ok(s) => s.chars().next().unwrap_or('?'),
                            Err(_) => '?',
                        }
                    };

                    let cell = AnsiCell {
                        character: ch,
                        fg_color: self.current_fg.clone(),
                        bg_color: self.current_bg.clone(),
                        bold: self.bold,
                        blink: self.blink,
                    };

                    frame.set_cell(x, y, cell);
                    x += 1;
                }
                i += 1;
            }
        }

        Ok(frame)
    }

    /// Apply SGR (Set Graphics Rendition) parameters
    fn apply_sgr(&mut self, params: &[u8]) {
        let codes = Self::parse_params(params);

        let mut i = 0;
        while i < codes.len() {
            match codes[i] {
                0 => {
                    // Reset
                    self.current_fg = Color::new(170, 170, 170);
                    self.current_bg = Color::new(0, 0, 0);
                    self.bold = false;
                    self.blink = false;
                }
                1 => self.bold = true,
                5 => self.blink = true,
                22 => self.bold = false,
                25 => self.blink = false,
                30..=37 => {
                    // Foreground color
                    let color_idx = (codes[i] - 30) as usize;
                    self.current_fg = Self::ansi_color_to_rgb(color_idx, false);
                }
                38 => {
                    // Extended foreground color
                    if i + 2 < codes.len() && codes[i + 1] == 5 {
                        // 256-color mode
                        let color_idx = codes[i + 2] as usize;
                        self.current_fg = Self::ansi_color_to_rgb(color_idx, false);
                        i += 2;
                    }
                }
                40..=47 => {
                    // Background color
                    let color_idx = (codes[i] - 40) as usize;
                    self.current_bg = Self::ansi_color_to_rgb(color_idx, false);
                }
                48 => {
                    // Extended background color
                    if i + 2 < codes.len() && codes[i + 1] == 5 {
                        // 256-color mode
                        let color_idx = codes[i + 2] as usize;
                        self.current_bg = Self::ansi_color_to_rgb(color_idx, false);
                        i += 2;
                    }
                }
                90..=97 => {
                    // Bright foreground color
                    let color_idx = (codes[i] - 90 + 8) as usize;
                    self.current_fg = Self::ansi_color_to_rgb(color_idx, false);
                }
                100..=107 => {
                    // Bright background color
                    let color_idx = (codes[i] - 100 + 8) as usize;
                    self.current_bg = Self::ansi_color_to_rgb(color_idx, false);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Parse numeric parameters from escape sequence
    fn parse_params(params: &[u8]) -> Vec<u16> {
        if params.is_empty() {
            return vec![0];
        }

        String::from_utf8_lossy(params)
            .split(';')
            .filter_map(|s| s.parse().ok())
            .collect()
    }

    /// Convert ANSI color code to RGB
    fn ansi_color_to_rgb(code: usize, bright: bool) -> Color {
        let idx = if bright && code < 8 {
            code + 8
        } else {
            code.min(15)
        };

        Color::from_hex(ANSI_COLORS[idx]).unwrap_or_else(|_| Color::new(170, 170, 170))
    }

    /// Convert ANSI frame to Sprite
    pub fn ansi_to_sprite(
        frame: &AnsiFrame,
        name: String,
        duration_ms: u64,
    ) -> RenderResult<Sprite> {
        // Build color palette from unique colors in the frame
        let mut unique_colors = Vec::new();
        let mut color_map = std::collections::HashMap::new();

        // Add black as transparent color
        unique_colors.push(Color::new(0, 0, 0));
        color_map.insert((0, 0, 0), 0);

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                if let Some(cell) = frame.get_cell(x, y) {
                    // Add background color
                    let bg_key = (cell.bg_color.r, cell.bg_color.g, cell.bg_color.b);
                    if !color_map.contains_key(&bg_key) {
                        color_map.insert(bg_key, unique_colors.len());
                        unique_colors.push(cell.bg_color.clone());
                    }

                    // Add foreground color if character is not space
                    if cell.character != ' ' {
                        let fg_key = (cell.fg_color.r, cell.fg_color.g, cell.fg_color.b);
                        if !color_map.contains_key(&fg_key) {
                            color_map.insert(fg_key, unique_colors.len());
                            unique_colors.push(cell.fg_color.clone());
                        }
                    }
                }
            }
        }

        // Create palette
        let palette = ColorPalette::new(unique_colors).with_transparent(0);

        // Build pixel data
        let mut pixels = Vec::with_capacity(frame.width() * frame.height());
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                if let Some(cell) = frame.get_cell(x, y) {
                    let color_idx = if cell.character == ' ' {
                        // Use background color for spaces
                        let bg_key = (cell.bg_color.r, cell.bg_color.g, cell.bg_color.b);
                        *color_map.get(&bg_key).unwrap_or(&0)
                    } else {
                        // Use foreground color for characters
                        let fg_key = (cell.fg_color.r, cell.fg_color.g, cell.fg_color.b);
                        *color_map.get(&fg_key).unwrap_or(&0)
                    };
                    pixels.push(color_idx);
                } else {
                    pixels.push(0); // Transparent
                }
            }
        }

        let sprite_frame = SpriteFrame::new(frame.width(), frame.height(), pixels, duration_ms)?;
        Sprite::new(name, palette, vec![sprite_frame])
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_parsing() {
        let parser = AnsiParser::new();
        let color = AnsiParser::ansi_color_to_rgb(1, false);
        assert_eq!(color.r, 170);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_parse_params() {
        let params = b"1;37;40";
        let codes = AnsiParser::parse_params(params);
        assert_eq!(codes, vec![1, 37, 40]);
    }

    #[test]
    fn test_ansi_frame_creation() {
        let frame = AnsiFrame::new(10, 5);
        assert_eq!(frame.width(), 10);
        assert_eq!(frame.height(), 5);
    }

    #[test]
    fn test_simple_ansi_parse() {
        let ansi = b"\x1b[31mRed\x1b[0m";
        let result = AnsiParser::parse_bytes(ansi);
        assert!(result.is_ok());
    }
}

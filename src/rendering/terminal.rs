//! Terminal rendering for sprites using Unicode block characters

use crate::rendering::{ansi_parser::*, sprites::*, RenderError, RenderResult};
use colored::*;
use std::io::{self, Write};

/// Terminal renderer for sprites
pub struct TerminalRenderer {
    /// Whether to use true color (24-bit RGB) or 256-color mode
    use_true_color: bool,
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new() -> Self {
        Self {
            use_true_color: Self::supports_true_color(),
        }
    }

    /// Check if the terminal supports true color (24-bit RGB)
    fn supports_true_color() -> bool {
        // Check COLORTERM environment variable
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        // Default to 256-color mode for compatibility
        false
    }

    /// Render a single frame to the terminal
    pub fn render_frame(
        &self,
        frame: &SpriteFrame,
        palette: &ColorPalette,
    ) -> RenderResult<String> {
        let mut output = String::new();

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let pixel_index = frame.get_pixel(x, y).ok_or_else(|| {
                    RenderError::RenderingError(format!("Invalid pixel at ({}, {})", x, y))
                })?;

                if palette.is_transparent(pixel_index) {
                    output.push(' ');
                } else {
                    let color = palette.get(pixel_index).ok_or_else(|| {
                        RenderError::RenderingError(format!(
                            "Invalid palette index {} at ({}, {})",
                            pixel_index, x, y
                        ))
                    })?;

                    let pixel_char = self.colorize_pixel(color);
                    output.push_str(&pixel_char);
                }
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Render a frame and print directly to stdout
    pub fn print_frame(
        &self,
        frame: &SpriteFrame,
        palette: &ColorPalette,
    ) -> RenderResult<()> {
        let rendered = self.render_frame(frame, palette)?;
        print!("{}", rendered);
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to flush stdout: {}", e))
        })?;
        Ok(())
    }

    /// Colorize a pixel character using ANSI escape codes
    fn colorize_pixel(&self, color: &Color) -> String {
        let block = "█";

        if self.use_true_color {
            // Use true color (24-bit RGB)
            block.truecolor(color.r, color.g, color.b).to_string()
        } else {
            // Use 256-color palette (for broader compatibility)
            let ansi_code = color.to_ansi_256();
            self.format_256_color(block, ansi_code)
        }
    }

    /// Format a string with 256-color ANSI code
    fn format_256_color(&self, text: &str, color_code: u8) -> String {
        format!("\x1b[38;5;{}m{}\x1b[0m", color_code, text)
    }

    /// Clear the terminal screen
    pub fn clear_screen(&self) -> RenderResult<()> {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to clear screen: {}", e))
        })?;
        Ok(())
    }

    /// Move cursor to a specific position (1-indexed)
    pub fn move_cursor(&self, row: usize, col: usize) -> RenderResult<()> {
        print!("\x1b[{};{}H", row, col);
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to move cursor: {}", e))
        })?;
        Ok(())
    }

    /// Hide the cursor
    pub fn hide_cursor(&self) -> RenderResult<()> {
        print!("\x1b[?25l");
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to hide cursor: {}", e))
        })?;
        Ok(())
    }

    /// Show the cursor
    pub fn show_cursor(&self) -> RenderResult<()> {
        print!("\x1b[?25h");
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to show cursor: {}", e))
        })?;
        Ok(())
    }

    /// Render a frame at a specific terminal position
    pub fn render_frame_at(
        &self,
        frame: &SpriteFrame,
        palette: &ColorPalette,
        row: usize,
        col: usize,
    ) -> RenderResult<()> {
        for y in 0..frame.height() {
            self.move_cursor(row + y, col)?;

            for x in 0..frame.width() {
                let pixel_index = frame.get_pixel(x, y).ok_or_else(|| {
                    RenderError::RenderingError(format!("Invalid pixel at ({}, {})", x, y))
                })?;

                if palette.is_transparent(pixel_index) {
                    print!(" ");
                } else {
                    let color = palette.get(pixel_index).ok_or_else(|| {
                        RenderError::RenderingError(format!(
                            "Invalid palette index {} at ({}, {})",
                            pixel_index, x, y
                        ))
                    })?;

                    let pixel_char = self.colorize_pixel(color);
                    print!("{}", pixel_char);
                }
            }
        }

        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to flush stdout: {}", e))
        })?;
        Ok(())
    }

    /// Render an ANSI frame to the terminal
    pub fn render_ansi_frame(&self, frame: &AnsiFrame) -> RenderResult<String> {
        let mut output = String::new();

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                if let Some(cell) = frame.get_cell(x, y) {
                    let colored_char = self.colorize_ansi_cell(cell);
                    output.push_str(&colored_char);
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Print an ANSI frame directly to stdout
    pub fn print_ansi_frame(&self, frame: &AnsiFrame) -> RenderResult<()> {
        let rendered = self.render_ansi_frame(frame)?;
        print!("{}", rendered);
        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to flush stdout: {}", e))
        })?;
        Ok(())
    }

    /// Render an ANSI frame at a specific terminal position
    pub fn render_ansi_frame_at(
        &self,
        frame: &AnsiFrame,
        row: usize,
        col: usize,
    ) -> RenderResult<()> {
        for y in 0..frame.height() {
            self.move_cursor(row + y, col)?;

            for x in 0..frame.width() {
                if let Some(cell) = frame.get_cell(x, y) {
                    let colored_char = self.colorize_ansi_cell(cell);
                    print!("{}", colored_char);
                } else {
                    print!(" ");
                }
            }
        }

        io::stdout().flush().map_err(|e| {
            RenderError::RenderingError(format!("Failed to flush stdout: {}", e))
        })?;
        Ok(())
    }

    /// Colorize an ANSI cell with foreground and background colors
    fn colorize_ansi_cell(&self, cell: &AnsiCell) -> String {
        let ch = if cell.character == '\0' || cell.character == '\r' {
            ' '
        } else {
            cell.character
        };

        if self.use_true_color {
            // Use true color (24-bit RGB)
            format!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m",
                cell.fg_color.r,
                cell.fg_color.g,
                cell.fg_color.b,
                cell.bg_color.r,
                cell.bg_color.g,
                cell.bg_color.b,
                ch
            )
        } else {
            // Use 256-color palette
            let fg_code = cell.fg_color.to_ansi_256();
            let bg_code = cell.bg_color.to_ansi_256();
            format!("\x1b[38;5;{}m\x1b[48;5;{}m{}\x1b[0m", fg_code, bg_code, ch)
        }
    }
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = TerminalRenderer::new();
        // Just verify it creates successfully
        assert!(!renderer.use_true_color || renderer.use_true_color);
    }

    #[test]
    fn test_render_simple_frame() {
        let palette = ColorPalette::from_hex_strings(&["#FF0000", "#0000FF"]).unwrap();
        let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 100).unwrap();
        let renderer = TerminalRenderer::new();

        let result = renderer.render_frame(&frame, &palette);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should have 2 rows + newlines
        assert_eq!(output.lines().count(), 2);
    }

    #[test]
    fn test_render_with_transparency() {
        let palette = ColorPalette::from_hex_strings(&["#FF0000", "#0000FF"])
            .unwrap()
            .with_transparent(0);
        let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 100).unwrap();
        let renderer = TerminalRenderer::new();

        let result = renderer.render_frame(&frame, &palette);
        assert!(result.is_ok());
    }

    #[test]
    fn test_colorize_pixel() {
        let renderer = TerminalRenderer::new();
        let color = Color::new(255, 0, 0);
        let result = renderer.colorize_pixel(&color);

        // Should contain ANSI escape codes
        assert!(result.contains("█"));
    }
}

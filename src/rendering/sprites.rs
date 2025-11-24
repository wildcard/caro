//! Sprite data structures and color palette management

use crate::rendering::{RenderError, RenderResult};
use serde::{Deserialize, Serialize};

/// Represents a color in the palette using hex format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from a hex string (e.g., "#FF5733" or "FF5733")
    pub fn from_hex(hex: &str) -> RenderResult<Self> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(RenderError::InvalidColor(format!(
                "Hex color must be 6 characters, got {}",
                hex.len()
            )));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|e| RenderError::InvalidColor(format!("Invalid red component: {}", e)))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|e| RenderError::InvalidColor(format!("Invalid green component: {}", e)))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|e| RenderError::InvalidColor(format!("Invalid blue component: {}", e)))?;

        Ok(Self { r, g, b })
    }

    /// Convert color to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Get ANSI 256-color code (approximate)
    pub fn to_ansi_256(&self) -> u8 {
        // Convert RGB to ANSI 256 color code
        // Use 216-color cube (16 + 36*r + 6*g + b) for colors
        if self.r == self.g && self.g == self.b {
            // Grayscale
            if self.r < 8 {
                16
            } else if self.r > 248 {
                231
            } else {
                232 + ((self.r - 8) / 10)
            }
        } else {
            // Color cube (6x6x6)
            let r = (self.r as u16 * 5 / 255) as u8;
            let g = (self.g as u16 * 5 / 255) as u8;
            let b = (self.b as u16 * 5 / 255) as u8;
            16 + 36 * r + 6 * g + b
        }
    }
}

/// Color palette for a sprite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    colors: Vec<Color>,
    transparent_index: Option<usize>,
}

impl ColorPalette {
    /// Create a new color palette
    pub fn new(colors: Vec<Color>) -> Self {
        Self {
            colors,
            transparent_index: None,
        }
    }

    /// Create a palette from hex color strings
    pub fn from_hex_strings(hex_colors: &[&str]) -> RenderResult<Self> {
        let colors = hex_colors
            .iter()
            .map(|&hex| Color::from_hex(hex))
            .collect::<RenderResult<Vec<Color>>>()?;
        Ok(Self::new(colors))
    }

    /// Set which palette index represents transparency
    pub fn with_transparent(mut self, index: usize) -> Self {
        self.transparent_index = Some(index);
        self
    }

    /// Get color at palette index
    pub fn get(&self, index: usize) -> Option<&Color> {
        self.colors.get(index)
    }

    /// Check if an index represents transparency
    pub fn is_transparent(&self, index: usize) -> bool {
        self.transparent_index == Some(index)
    }

    /// Get the number of colors in the palette
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Check if palette is empty
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}

/// A single frame of sprite data
/// Each pixel is represented by an index into the color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteFrame {
    width: usize,
    height: usize,
    /// Pixel data stored row-by-row, each value is an index into the color palette
    pixels: Vec<usize>,
    /// Duration this frame should be displayed (in milliseconds)
    duration_ms: u64,
}

impl SpriteFrame {
    /// Create a new sprite frame
    pub fn new(width: usize, height: usize, pixels: Vec<usize>, duration_ms: u64) -> RenderResult<Self> {
        if pixels.len() != width * height {
            return Err(RenderError::InvalidDimensions(format!(
                "Pixel count {} doesn't match dimensions {}x{}",
                pixels.len(),
                width,
                height
            )));
        }

        Ok(Self {
            width,
            height,
            pixels,
            duration_ms,
        })
    }

    /// Get the width of the sprite frame
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the sprite frame
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the pixel color index at the given position
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[y * self.width + x])
    }

    /// Get the frame duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }

    /// Get all pixel data
    pub fn pixels(&self) -> &[usize] {
        &self.pixels
    }
}

/// A complete sprite with palette and frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    name: String,
    palette: ColorPalette,
    frames: Vec<SpriteFrame>,
}

impl Sprite {
    /// Create a new sprite
    pub fn new(name: String, palette: ColorPalette, frames: Vec<SpriteFrame>) -> RenderResult<Self> {
        if frames.is_empty() {
            return Err(RenderError::InvalidDimensions(
                "Sprite must have at least one frame".to_string(),
            ));
        }

        // Validate all frames have the same dimensions
        let (width, height) = (frames[0].width(), frames[0].height());
        for (i, frame) in frames.iter().enumerate() {
            if frame.width() != width || frame.height() != height {
                return Err(RenderError::InvalidDimensions(format!(
                    "Frame {} dimensions {}x{} don't match first frame {}x{}",
                    i,
                    frame.width(),
                    frame.height(),
                    width,
                    height
                )));
            }
        }

        Ok(Self {
            name,
            palette,
            frames,
        })
    }

    /// Get the sprite name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the color palette
    pub fn palette(&self) -> &ColorPalette {
        &self.palette
    }

    /// Get all frames
    pub fn frames(&self) -> &[SpriteFrame] {
        &self.frames
    }

    /// Get a specific frame
    pub fn frame(&self, index: usize) -> Option<&SpriteFrame> {
        self.frames.get(index)
    }

    /// Get the number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get sprite dimensions (width, height)
    pub fn dimensions(&self) -> (usize, usize) {
        if let Some(frame) = self.frames.first() {
            (frame.width(), frame.height())
        } else {
            (0, 0)
        }
    }

    /// Check if this is a static (single-frame) sprite
    pub fn is_static(&self) -> bool {
        self.frames.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF5733").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);

        let color2 = Color::from_hex("00FF00").unwrap();
        assert_eq!(color2.r, 0);
        assert_eq!(color2.g, 255);
        assert_eq!(color2.b, 0);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::new(255, 87, 51);
        assert_eq!(color.to_hex(), "#FF5733");
    }

    #[test]
    fn test_invalid_hex() {
        assert!(Color::from_hex("#FF").is_err());
        assert!(Color::from_hex("GGGGGG").is_err());
    }

    #[test]
    fn test_color_palette() {
        let palette = ColorPalette::from_hex_strings(&["#FF0000", "#00FF00", "#0000FF"]).unwrap();
        assert_eq!(palette.len(), 3);
        assert_eq!(palette.get(0).unwrap().to_hex(), "#FF0000");
    }

    #[test]
    fn test_sprite_frame() {
        // 2x2 frame with 4 pixels
        let pixels = vec![0, 1, 2, 3];
        let frame = SpriteFrame::new(2, 2, pixels, 100).unwrap();

        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 2);
        assert_eq!(frame.get_pixel(0, 0), Some(0));
        assert_eq!(frame.get_pixel(1, 1), Some(3));
        assert_eq!(frame.duration_ms(), 100);
    }

    #[test]
    fn test_sprite_frame_invalid_dimensions() {
        let pixels = vec![0, 1, 2]; // Only 3 pixels
        let result = SpriteFrame::new(2, 2, pixels, 100); // Expects 4 pixels
        assert!(result.is_err());
    }

    #[test]
    fn test_sprite_creation() {
        let palette = ColorPalette::from_hex_strings(&["#000000", "#FFFFFF"]).unwrap();
        let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 100).unwrap();
        let sprite = Sprite::new("test".to_string(), palette, vec![frame]).unwrap();

        assert_eq!(sprite.name(), "test");
        assert_eq!(sprite.frame_count(), 1);
        assert_eq!(sprite.dimensions(), (2, 2));
        assert!(sprite.is_static());
    }
}

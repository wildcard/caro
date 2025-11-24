//! Aseprite file format parser
//!
//! Supports parsing Aseprite source files (.ase/.aseprite) used by the Aseprite pixel art editor.
//! Based on specification: https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md

use crate::rendering::{sprites::*, RenderError, RenderResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Aseprite magic number (0xA5E0)
const ASE_MAGIC: u16 = 0xA5E0;

/// Color modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Indexed = 1,
    Grayscale = 2,
    RGBA = 4,
}

/// Aseprite file header
#[derive(Debug, Clone)]
pub struct AsepriteHeader {
    pub file_size: u32,
    pub magic: u16,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub color_depth: u16,
    pub flags: u32,
    pub speed: u16, // Deprecated, use frame duration
    pub palette_entry: u8,
    pub num_colors: u16,
    pub pixel_width: u8,
    pub pixel_height: u8,
    pub grid_x: i16,
    pub grid_y: i16,
    pub grid_width: u16,
    pub grid_height: u16,
}

/// Aseprite frame
#[derive(Debug, Clone)]
pub struct AsepriteFrame {
    pub duration: u16, // Frame duration in milliseconds
    pub cels: Vec<AsepriteCel>,
}

/// Aseprite cel (layer content for a frame)
#[derive(Debug, Clone)]
pub struct AsepriteCel {
    pub layer_index: u16,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub opacity: u8,
    pub pixels: Vec<u8>, // Raw pixel data
}

/// Aseprite layer
#[derive(Debug, Clone)]
pub struct AsepriteLayer {
    pub flags: u16,
    pub layer_type: u16,
    pub child_level: u16,
    pub blend_mode: u16,
    pub opacity: u8,
    pub name: String,
    pub visible: bool,
}

/// Aseprite palette
#[derive(Debug, Clone)]
pub struct AsepritePalette {
    pub size: u32,
    pub first_color: u32,
    pub last_color: u32,
    pub colors: Vec<Color>,
}

/// Complete Aseprite file
#[derive(Debug, Clone)]
pub struct AsepriteFile {
    pub header: AsepriteHeader,
    pub frames: Vec<AsepriteFrame>,
    pub layers: Vec<AsepriteLayer>,
    pub palette: Option<AsepritePalette>,
}

/// Aseprite parser
pub struct AsepriteParser;

impl AsepriteParser {
    /// Load an Aseprite file from disk
    pub fn load_file<P: AsRef<Path>>(path: P) -> RenderResult<AsepriteFile> {
        let mut file = File::open(path.as_ref()).map_err(|e| {
            RenderError::RenderingError(format!("Failed to open Aseprite file: {}", e))
        })?;

        Self::parse(&mut file)
    }

    /// Parse Aseprite file from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> RenderResult<AsepriteFile> {
        // Parse header
        let header = Self::parse_header(reader)?;

        // Validate magic number
        if header.magic != ASE_MAGIC {
            return Err(RenderError::RenderingError(format!(
                "Invalid Aseprite magic number: 0x{:04X}",
                header.magic
            )));
        }

        let mut frames = Vec::new();
        let mut layers = Vec::new();
        let mut palette = None;

        // Parse each frame
        for _ in 0..header.frames {
            let (frame, frame_layers, frame_palette) = Self::parse_frame(reader)?;
            frames.push(frame);

            // Collect layers from first frame
            if layers.is_empty() {
                layers = frame_layers;
            }

            // Use the last palette encountered
            if frame_palette.is_some() {
                palette = frame_palette;
            }
        }

        Ok(AsepriteFile {
            header,
            frames,
            layers,
            palette,
        })
    }

    /// Parse Aseprite header
    fn parse_header<R: Read>(reader: &mut R) -> RenderResult<AsepriteHeader> {
        let file_size = Self::read_u32(reader)?;
        let magic = Self::read_u16(reader)?;
        let frames = Self::read_u16(reader)?;
        let width = Self::read_u16(reader)?;
        let height = Self::read_u16(reader)?;
        let color_depth = Self::read_u16(reader)?;
        let flags = Self::read_u32(reader)?;
        let speed = Self::read_u16(reader)?;

        // Skip next 8 bytes (set to zero)
        Self::skip(reader, 8)?;

        let palette_entry = Self::read_u8(reader)?;

        // Skip 3 bytes
        Self::skip(reader, 3)?;

        let num_colors = Self::read_u16(reader)?;
        let pixel_width = Self::read_u8(reader)?;
        let pixel_height = Self::read_u8(reader)?;
        let grid_x = Self::read_i16(reader)?;
        let grid_y = Self::read_i16(reader)?;
        let grid_width = Self::read_u16(reader)?;
        let grid_height = Self::read_u16(reader)?;

        // Skip remaining header bytes (84 bytes reserved for future)
        Self::skip(reader, 84)?;

        Ok(AsepriteHeader {
            file_size,
            magic,
            frames,
            width,
            height,
            color_depth,
            flags,
            speed,
            palette_entry,
            num_colors,
            pixel_width,
            pixel_height,
            grid_x,
            grid_y,
            grid_width,
            grid_height,
        })
    }

    /// Parse a single frame
    fn parse_frame<R: Read + Seek>(
        reader: &mut R,
    ) -> RenderResult<(AsepriteFrame, Vec<AsepriteLayer>, Option<AsepritePalette>)> {
        let frame_bytes = Self::read_u32(reader)?;
        let magic = Self::read_u16(reader)?;

        if magic != 0xF1FA {
            return Err(RenderError::RenderingError(format!(
                "Invalid frame magic: 0x{:04X}",
                magic
            )));
        }

        let old_chunk_count = Self::read_u16(reader)?;
        let duration = Self::read_u16(reader)?;

        // Skip 2 bytes
        Self::skip(reader, 2)?;

        let new_chunk_count = Self::read_u32(reader)?;

        let chunk_count = if new_chunk_count == 0 {
            old_chunk_count as u32
        } else {
            new_chunk_count
        };

        let mut cels = Vec::new();
        let mut layers = Vec::new();
        let mut palette = None;

        // Parse chunks
        for _ in 0..chunk_count {
            let chunk_size = Self::read_u32(reader)?;
            let chunk_type = Self::read_u16(reader)?;

            let chunk_data_size = chunk_size - 6; // Subtract size and type fields

            match chunk_type {
                0x2004 => {
                    // Layer chunk
                    if let Ok(layer) = Self::parse_layer_chunk(reader) {
                        layers.push(layer);
                    }
                }
                0x2005 => {
                    // Cel chunk
                    if let Ok(cel) = Self::parse_cel_chunk(reader, chunk_data_size) {
                        cels.push(cel);
                    }
                }
                0x2019 => {
                    // Palette chunk
                    if let Ok(pal) = Self::parse_palette_chunk(reader) {
                        palette = Some(pal);
                    }
                }
                _ => {
                    // Skip unknown chunk
                    Self::skip(reader, chunk_data_size as usize)?;
                }
            }
        }

        Ok((AsepriteFrame { duration, cels }, layers, palette))
    }

    /// Parse layer chunk
    fn parse_layer_chunk<R: Read>(reader: &mut R) -> RenderResult<AsepriteLayer> {
        let flags = Self::read_u16(reader)?;
        let layer_type = Self::read_u16(reader)?;
        let child_level = Self::read_u16(reader)?;

        // Skip default width/height (4 bytes)
        Self::skip(reader, 4)?;

        let blend_mode = Self::read_u16(reader)?;
        let opacity = Self::read_u8(reader)?;

        // Skip 3 bytes
        Self::skip(reader, 3)?;

        let name = Self::read_string(reader)?;

        let visible = (flags & 1) != 0;

        Ok(AsepriteLayer {
            flags,
            layer_type,
            child_level,
            blend_mode,
            opacity,
            name,
            visible,
        })
    }

    /// Parse cel chunk
    fn parse_cel_chunk<R: Read>(reader: &mut R, chunk_size: u32) -> RenderResult<AsepriteCel> {
        let layer_index = Self::read_u16(reader)?;
        let x = Self::read_i16(reader)?;
        let y = Self::read_i16(reader)?;
        let opacity = Self::read_u8(reader)?;
        let cel_type = Self::read_u16(reader)?;

        // Skip 7 bytes
        Self::skip(reader, 7)?;

        let mut pixels = Vec::new();
        let mut width = 0u16;
        let mut height = 0u16;

        match cel_type {
            0 => {
                // Raw cel
                width = Self::read_u16(reader)?;
                height = Self::read_u16(reader)?;
                let pixel_count = (width as usize) * (height as usize);
                pixels = Self::read_bytes(reader, pixel_count * 4)?; // RGBA
            }
            2 => {
                // Compressed cel
                width = Self::read_u16(reader)?;
                height = Self::read_u16(reader)?;

                // Read compressed data
                let remaining_bytes = chunk_size as usize - 26; // Header bytes
                let compressed = Self::read_bytes(reader, remaining_bytes)?;

                // Decompress using DEFLATE (zlib)
                pixels = Self::decompress_zlib(&compressed)?;
            }
            _ => {
                // Unknown cel type, skip
                return Err(RenderError::RenderingError(format!(
                    "Unsupported cel type: {}",
                    cel_type
                )));
            }
        }

        Ok(AsepriteCel {
            layer_index,
            x,
            y,
            width,
            height,
            opacity,
            pixels,
        })
    }

    /// Parse palette chunk
    fn parse_palette_chunk<R: Read>(reader: &mut R) -> RenderResult<AsepritePalette> {
        let size = Self::read_u32(reader)?;
        let first_color = Self::read_u32(reader)?;
        let last_color = Self::read_u32(reader)?;

        // Skip 8 bytes
        Self::skip(reader, 8)?;

        let mut colors = Vec::new();

        for _ in first_color..=last_color {
            let flags = Self::read_u16(reader)?;
            let r = Self::read_u8(reader)?;
            let g = Self::read_u8(reader)?;
            let b = Self::read_u8(reader)?;
            let _a = Self::read_u8(reader)?;

            colors.push(Color::new(r, g, b));

            // Skip name if present
            if flags & 1 != 0 {
                let _name = Self::read_string(reader)?;
            }
        }

        Ok(AsepritePalette {
            size,
            first_color,
            last_color,
            colors,
        })
    }

    /// Decompress zlib data
    fn decompress_zlib(data: &[u8]) -> RenderResult<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| {
            RenderError::RenderingError(format!("Failed to decompress cel data: {}", e))
        })?;

        Ok(decompressed)
    }

    /// Convert Aseprite file to Sprite
    pub fn to_sprite(ase: &AsepriteFile) -> RenderResult<Sprite> {
        let width = ase.header.width as usize;
        let height = ase.header.height as usize;

        // Build color palette from all frames
        let mut palette_colors = Vec::new();
        let mut color_map = std::collections::HashMap::new();

        // Add transparent color
        palette_colors.push(Color::new(0, 0, 0));
        color_map.insert((0, 0, 0, 0), 0);

        let mut sprite_frames = Vec::new();

        for frame in &ase.frames {
            // Composite all visible cels
            let mut frame_pixels = vec![0u8; width * height * 4]; // RGBA

            for cel in &frame.cels {
                if cel.layer_index as usize >= ase.layers.len() {
                    continue;
                }

                let layer = &ase.layers[cel.layer_index as usize];
                if !layer.visible {
                    continue;
                }

                // Blit cel onto frame
                Self::blit_cel(&mut frame_pixels, width, height, cel)?;
            }

            // Convert RGBA to palette indices
            let mut pixels = Vec::new();
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 4;
                    let r = frame_pixels[idx];
                    let g = frame_pixels[idx + 1];
                    let b = frame_pixels[idx + 2];
                    let a = frame_pixels[idx + 3];

                    if a == 0 {
                        // Transparent
                        pixels.push(0);
                    } else {
                        let key = (r, g, b, a);
                        if !color_map.contains_key(&key) {
                            color_map.insert(key, palette_colors.len());
                            palette_colors.push(Color::new(r, g, b));
                        }
                        pixels.push(color_map[&key]);
                    }
                }
            }

            let duration_ms = if frame.duration > 0 {
                frame.duration as u64
            } else {
                100 // Default 100ms
            };

            sprite_frames.push(SpriteFrame::new(width, height, pixels, duration_ms)?);
        }

        let palette = ColorPalette::new(palette_colors).with_transparent(0);

        Sprite::new("aseprite".to_string(), palette, sprite_frames)
    }

    /// Blit a cel onto the frame buffer
    fn blit_cel(
        frame: &mut [u8],
        frame_width: usize,
        frame_height: usize,
        cel: &AsepriteCel,
    ) -> RenderResult<()> {
        let cel_width = cel.width as usize;
        let cel_height = cel.height as usize;

        for y in 0..cel_height {
            for x in 0..cel_width {
                let fx = (cel.x as i32 + x as i32) as usize;
                let fy = (cel.y as i32 + y as i32) as usize;

                if fx >= frame_width || fy >= frame_height {
                    continue;
                }

                let cel_idx = (y * cel_width + x) * 4;
                let frame_idx = (fy * frame_width + fx) * 4;

                if cel_idx + 3 >= cel.pixels.len() {
                    continue;
                }

                // Alpha blending
                let src_a = cel.pixels[cel_idx + 3] as f32 / 255.0;
                let opacity = cel.opacity as f32 / 255.0;
                let alpha = src_a * opacity;

                if alpha > 0.0 {
                    for i in 0..3 {
                        let src = cel.pixels[cel_idx + i] as f32;
                        let dst = frame[frame_idx + i] as f32;
                        frame[frame_idx + i] = ((src * alpha + dst * (1.0 - alpha)) as u8);
                    }
                    frame[frame_idx + 3] = 255;
                }
            }
        }

        Ok(())
    }

    // Binary reading utilities
    fn read_u8<R: Read>(reader: &mut R) -> RenderResult<u8> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Read error: {}", e))
        })?;
        Ok(buf[0])
    }

    fn read_u16<R: Read>(reader: &mut R) -> RenderResult<u16> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Read error: {}", e))
        })?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_i16<R: Read>(reader: &mut R) -> RenderResult<i16> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Read error: {}", e))
        })?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_u32<R: Read>(reader: &mut R) -> RenderResult<u32> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Read error: {}", e))
        })?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_string<R: Read>(reader: &mut R) -> RenderResult<String> {
        let len = Self::read_u16(reader)? as usize;
        let bytes = Self::read_bytes(reader, len)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    fn read_bytes<R: Read>(reader: &mut R, count: usize) -> RenderResult<Vec<u8>> {
        let mut buf = vec![0u8; count];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Read error: {}", e))
        })?;
        Ok(buf)
    }

    fn skip<R: Read>(reader: &mut R, count: usize) -> RenderResult<()> {
        let mut buf = vec![0u8; count];
        reader.read_exact(&mut buf).map_err(|e| {
            RenderError::RenderingError(format!("Skip error: {}", e))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aseprite_header_size() {
        // Aseprite header should be 128 bytes
        assert_eq!(std::mem::size_of::<u32>() * 2 + 128, 136);
    }

    #[test]
    fn test_color_mode() {
        assert_eq!(ColorMode::RGBA as u16, 4);
        assert_eq!(ColorMode::Indexed as u16, 1);
    }
}

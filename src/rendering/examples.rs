//! Example sprites and animations for demonstration

use crate::rendering::{sprites::*, RenderResult};

/// Create a simple idle character sprite (8x8 pixels)
/// A basic humanoid character standing still
pub fn create_idle_character() -> RenderResult<Sprite> {
    // Color palette: transparent, skin, shirt, pants, hair
    let palette = ColorPalette::from_hex_strings(&[
        "#000000", // 0: Transparent/background (we'll set this)
        "#FFD0B5", // 1: Skin tone
        "#FF5733", // 2: Red shirt
        "#3366CC", // 3: Blue pants
        "#8B4513", // 4: Brown hair
        "#000000", // 5: Black (outline/eyes)
    ])?
    .with_transparent(0);

    // 8x8 pixel character
    #[rustfmt::skip]
    let pixels = vec![
        0, 0, 0, 4, 4, 0, 0, 0,  // Hair top
        0, 0, 4, 4, 4, 4, 0, 0,  // Hair sides
        0, 0, 1, 1, 1, 1, 0, 0,  // Face
        0, 0, 5, 1, 1, 5, 0, 0,  // Eyes
        0, 0, 2, 2, 2, 2, 0, 0,  // Shirt top
        0, 0, 2, 1, 1, 2, 0, 0,  // Arms (skin showing)
        0, 0, 3, 3, 3, 3, 0, 0,  // Pants
        0, 0, 3, 0, 0, 3, 0, 0,  // Legs
    ];

    let frame = SpriteFrame::new(8, 8, pixels, 1000)?; // 1 second duration for static

    Sprite::new("idle_character".to_string(), palette, vec![frame])
}

/// Create a walking animation sprite (8x8 pixels, 4 frames)
pub fn create_walking_animation() -> RenderResult<Sprite> {
    let palette = ColorPalette::from_hex_strings(&[
        "#000000", // 0: Transparent
        "#FFD0B5", // 1: Skin
        "#FF5733", // 2: Red shirt
        "#3366CC", // 3: Blue pants
        "#8B4513", // 4: Brown hair
        "#000000", // 5: Black (outline)
    ])?
    .with_transparent(0);

    // Frame 1: Standing
    #[rustfmt::skip]
    let frame1_pixels = vec![
        0, 0, 0, 4, 4, 0, 0, 0,
        0, 0, 4, 4, 4, 4, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 5, 1, 1, 5, 0, 0,
        0, 0, 2, 2, 2, 2, 0, 0,
        0, 0, 2, 1, 1, 2, 0, 0,
        0, 0, 3, 3, 3, 3, 0, 0,
        0, 0, 3, 0, 0, 3, 0, 0,
    ];

    // Frame 2: Left leg forward
    #[rustfmt::skip]
    let frame2_pixels = vec![
        0, 0, 0, 4, 4, 0, 0, 0,
        0, 0, 4, 4, 4, 4, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 5, 1, 1, 5, 0, 0,
        0, 0, 2, 2, 2, 2, 0, 0,
        0, 2, 1, 2, 2, 1, 0, 0,  // Left arm forward
        0, 3, 0, 3, 3, 0, 0, 0,  // Left leg forward
        3, 0, 0, 0, 3, 0, 0, 0,
    ];

    // Frame 3: Standing (same as frame 1)
    #[rustfmt::skip]
    let frame3_pixels = vec![
        0, 0, 0, 4, 4, 0, 0, 0,
        0, 0, 4, 4, 4, 4, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 5, 1, 1, 5, 0, 0,
        0, 0, 2, 2, 2, 2, 0, 0,
        0, 0, 2, 1, 1, 2, 0, 0,
        0, 0, 3, 3, 3, 3, 0, 0,
        0, 0, 3, 0, 0, 3, 0, 0,
    ];

    // Frame 4: Right leg forward
    #[rustfmt::skip]
    let frame4_pixels = vec![
        0, 0, 0, 4, 4, 0, 0, 0,
        0, 0, 4, 4, 4, 4, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 5, 1, 1, 5, 0, 0,
        0, 0, 2, 2, 2, 2, 0, 0,
        0, 0, 1, 2, 2, 1, 2, 0,  // Right arm forward
        0, 0, 0, 3, 3, 0, 3, 0,  // Right leg forward
        0, 0, 0, 3, 0, 0, 0, 3,
    ];

    let frames = vec![
        SpriteFrame::new(8, 8, frame1_pixels, 200)?,
        SpriteFrame::new(8, 8, frame2_pixels, 200)?,
        SpriteFrame::new(8, 8, frame3_pixels, 200)?,
        SpriteFrame::new(8, 8, frame4_pixels, 200)?,
    ];

    Sprite::new("walking_character".to_string(), palette, frames)
}

/// Create a simple heart animation (6x6 pixels)
pub fn create_heart_animation() -> RenderResult<Sprite> {
    let palette = ColorPalette::from_hex_strings(&[
        "#000000", // 0: Transparent
        "#FF1744", // 1: Red
        "#FF5252", // 2: Light red
        "#C62828", // 3: Dark red
    ])?
    .with_transparent(0);

    // Frame 1: Normal size heart
    #[rustfmt::skip]
    let frame1_pixels = vec![
        0, 2, 2, 2, 2, 0,
        2, 1, 1, 1, 1, 2,
        2, 1, 1, 1, 1, 2,
        0, 2, 1, 1, 2, 0,
        0, 0, 2, 2, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];

    // Frame 2: Slightly larger (pulse)
    #[rustfmt::skip]
    let frame2_pixels = vec![
        2, 1, 1, 1, 1, 2,
        1, 2, 2, 2, 2, 1,
        1, 2, 2, 2, 2, 1,
        2, 1, 2, 2, 1, 2,
        0, 2, 1, 1, 2, 0,
        0, 0, 2, 2, 0, 0,
    ];

    // Frame 3: Smaller (pulse back)
    #[rustfmt::skip]
    let frame3_pixels = vec![
        0, 0, 0, 0, 0, 0,
        0, 3, 3, 3, 3, 0,
        0, 3, 2, 2, 3, 0,
        0, 0, 3, 3, 0, 0,
        0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];

    let frames = vec![
        SpriteFrame::new(6, 6, frame1_pixels, 400)?,
        SpriteFrame::new(6, 6, frame2_pixels, 200)?,
        SpriteFrame::new(6, 6, frame3_pixels, 400)?,
    ];

    Sprite::new("heart_pulse".to_string(), palette, frames)
}

/// Create a spinning coin animation (8x8 pixels)
pub fn create_coin_animation() -> RenderResult<Sprite> {
    let palette = ColorPalette::from_hex_strings(&[
        "#000000", // 0: Transparent
        "#FFD700", // 1: Gold
        "#FFA500", // 2: Orange (shadow)
        "#FFFF00", // 3: Yellow (highlight)
    ])?
    .with_transparent(0);

    // Frame 1: Front view (wide)
    #[rustfmt::skip]
    let frame1_pixels = vec![
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 1, 3, 3, 3, 3, 1, 0,
        1, 3, 1, 1, 1, 1, 3, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 2, 1, 1, 1, 1, 2, 1,
        0, 1, 2, 2, 2, 2, 1, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
    ];

    // Frame 2: Tilted (medium)
    #[rustfmt::skip]
    let frame2_pixels = vec![
        0, 0, 0, 1, 1, 0, 0, 0,
        0, 0, 1, 3, 3, 1, 0, 0,
        0, 1, 3, 1, 1, 3, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 2, 1, 1, 2, 1, 0,
        0, 0, 1, 2, 2, 1, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 0,
    ];

    // Frame 3: Side view (narrow)
    #[rustfmt::skip]
    let frame3_pixels = vec![
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 0,
        0, 0, 1, 3, 3, 1, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 1, 2, 2, 1, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    // Frame 4: Tilted other way
    #[rustfmt::skip]
    let frame4_pixels = vec![
        0, 0, 0, 1, 1, 0, 0, 0,
        0, 0, 1, 3, 3, 1, 0, 0,
        0, 1, 3, 1, 1, 3, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 1, 1, 1, 1, 1, 0,
        0, 1, 2, 1, 1, 2, 1, 0,
        0, 0, 1, 2, 2, 1, 0, 0,
        0, 0, 0, 1, 1, 0, 0, 0,
    ];

    let frames = vec![
        SpriteFrame::new(8, 8, frame1_pixels, 150)?,
        SpriteFrame::new(8, 8, frame2_pixels, 150)?,
        SpriteFrame::new(8, 8, frame3_pixels, 150)?,
        SpriteFrame::new(8, 8, frame4_pixels, 150)?,
    ];

    Sprite::new("spinning_coin".to_string(), palette, frames)
}

/// Create a loading spinner animation (5x5 pixels)
pub fn create_spinner_animation() -> RenderResult<Sprite> {
    let palette = ColorPalette::from_hex_strings(&[
        "#000000", // 0: Transparent
        "#00FF00", // 1: Green
        "#00AA00", // 2: Dark green
    ])?
    .with_transparent(0);

    // 8 frames for smooth rotation
    let mut frames = Vec::new();

    // Frame patterns for a circular spinner
    let patterns = [
        // Frame 1: Top
        vec![
            0, 0, 1, 0, 0,
            0, 0, 2, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        // Frame 2: Top-right
        vec![
            0, 0, 0, 0, 1,
            0, 0, 0, 2, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        // Frame 3: Right
        vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 2, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        // Frame 4: Bottom-right
        vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 2, 0,
            0, 0, 0, 0, 1,
        ],
        // Frame 5: Bottom
        vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 2, 0, 0,
            0, 0, 1, 0, 0,
        ],
        // Frame 6: Bottom-left
        vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 2, 0, 0, 0,
            1, 0, 0, 0, 0,
        ],
        // Frame 7: Left
        vec![
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 2, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        // Frame 8: Top-left
        vec![
            1, 0, 0, 0, 0,
            0, 2, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
    ];

    for pattern in patterns {
        frames.push(SpriteFrame::new(5, 5, pattern, 100)?);
    }

    Sprite::new("spinner".to_string(), palette, frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_idle_character() {
        let sprite = create_idle_character().unwrap();
        assert_eq!(sprite.name(), "idle_character");
        assert_eq!(sprite.dimensions(), (8, 8));
        assert!(sprite.is_static());
    }

    #[test]
    fn test_create_walking_animation() {
        let sprite = create_walking_animation().unwrap();
        assert_eq!(sprite.name(), "walking_character");
        assert_eq!(sprite.frame_count(), 4);
        assert!(!sprite.is_static());
    }

    #[test]
    fn test_create_heart_animation() {
        let sprite = create_heart_animation().unwrap();
        assert_eq!(sprite.name(), "heart_pulse");
        assert_eq!(sprite.frame_count(), 3);
    }

    #[test]
    fn test_create_coin_animation() {
        let sprite = create_coin_animation().unwrap();
        assert_eq!(sprite.name(), "spinning_coin");
        assert_eq!(sprite.frame_count(), 4);
    }

    #[test]
    fn test_create_spinner_animation() {
        let sprite = create_spinner_animation().unwrap();
        assert_eq!(sprite.name(), "spinner");
        assert_eq!(sprite.frame_count(), 8);
    }
}

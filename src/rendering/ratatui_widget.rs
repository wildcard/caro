//! Ratatui widget implementations for sprite rendering
//!
//! This module provides widget types for integrating cmdai sprite animations
//! with Ratatui-based terminal user interfaces.

#![cfg(feature = "tui")]

use crate::rendering::{Animation, AnimationMode, Color, ColorPalette, Sprite, SpriteFrame};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color as RatatuiColor,
    widgets::Widget,
};
use std::time::{Duration, Instant};

/// Convert a cmdai Color to a Ratatui Color
pub fn to_ratatui_color(color: &Color) -> RatatuiColor {
    RatatuiColor::Rgb(color.r, color.g, color.b)
}

/// A static sprite widget that renders a single frame
///
/// This widget is suitable for non-animated sprites or when you want
/// full control over which frame to display.
///
/// # Examples
///
/// ```no_run
/// use cmdai::rendering::{Sprite, ratatui_widget::SpriteWidget};
/// use ratatui::widgets::Widget;
///
/// fn render(sprite: &Sprite, f: &mut Frame) {
///     let widget = SpriteWidget::new(sprite);
///     f.render_widget(widget, area);
/// }
/// ```
pub struct SpriteWidget<'a> {
    sprite: &'a Sprite,
    frame_index: usize,
}

impl<'a> SpriteWidget<'a> {
    /// Create a new sprite widget
    pub fn new(sprite: &'a Sprite) -> Self {
        Self {
            sprite,
            frame_index: 0,
        }
    }

    /// Set which frame to display (useful for multi-frame sprites)
    pub fn with_frame(mut self, index: usize) -> Self {
        self.frame_index = index;
        self
    }
}

impl<'a> Widget for SpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = match self.sprite.frame(self.frame_index) {
            Some(f) => f,
            None => return,
        };

        let palette = self.sprite.palette();

        render_frame_to_buffer(frame, palette, area, buf);
    }
}

/// Render a sprite frame to a Ratatui buffer
///
/// This is a low-level function that directly renders a frame to a buffer.
/// Most users should use `SpriteWidget` instead.
pub fn render_frame_to_buffer(
    frame: &SpriteFrame,
    palette: &ColorPalette,
    area: Rect,
    buf: &mut Buffer,
) {
    let max_height = frame.height().min(area.height as usize);
    let max_width = frame.width().min(area.width as usize);

    for y in 0..max_height {
        for x in 0..max_width {
            if let Some(pixel_idx) = frame.get_pixel(x, y) {
                if !palette.is_transparent(pixel_idx) {
                    if let Some(color) = palette.get(pixel_idx) {
                        let cell_x = area.x.saturating_add(x as u16);
                        let cell_y = area.y.saturating_add(y as u16);

                        // Bounds check
                        if cell_x < area.x + area.width && cell_y < area.y + area.height {
                            let cell = buf.get_mut(cell_x, cell_y);
                            cell.set_symbol("█");
                            cell.set_fg(to_ratatui_color(color));
                        }
                    }
                }
            }
        }
    }
}

/// Animation controller for managing sprite animation state
///
/// This type manages the timing and frame progression of an animation,
/// separate from the rendering logic. This allows for flexible integration
/// with different update patterns.
///
/// # Examples
///
/// ```no_run
/// use cmdai::rendering::{Sprite, AnimationMode, ratatui_widget::AnimationController};
///
/// let mut controller = AnimationController::new(sprite, AnimationMode::Loop);
///
/// // In your main loop:
/// loop {
///     if controller.should_advance() {
///         if !controller.advance() {
///             break; // Animation complete
///         }
///     }
/// }
/// ```
pub struct AnimationController {
    animation: Animation,
    last_update: Instant,
    speed_multiplier: f32,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new(sprite: Sprite, mode: AnimationMode) -> Self {
        Self {
            animation: Animation::new(sprite, mode),
            last_update: Instant::now(),
            speed_multiplier: 1.0,
        }
    }

    /// Check if enough time has elapsed to advance to the next frame
    pub fn should_advance(&self) -> bool {
        let frame_duration = self.current_frame_duration();
        self.last_update.elapsed() >= frame_duration
    }

    /// Advance to the next frame if it's time
    ///
    /// Returns true if the animation should continue, false if complete
    pub fn update(&mut self) -> bool {
        if self.should_advance() {
            self.advance()
        } else {
            !self.animation.is_complete()
        }
    }

    /// Manually advance to the next frame
    ///
    /// Returns true if the animation should continue, false if complete
    pub fn advance(&mut self) -> bool {
        self.last_update = Instant::now();
        self.animation.advance()
    }

    /// Get the current frame
    pub fn current_frame(&self) -> &SpriteFrame {
        self.animation.current_frame()
    }

    /// Get the sprite's color palette
    pub fn palette(&self) -> &ColorPalette {
        self.animation.palette()
    }

    /// Reset the animation to the beginning
    pub fn reset(&mut self) {
        self.animation.reset();
        self.last_update = Instant::now();
    }

    /// Set the animation playback speed (1.0 = normal, 2.0 = double speed, 0.5 = half speed)
    pub fn set_speed(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier.max(0.1).min(10.0);
    }

    /// Get the current speed multiplier
    pub fn speed(&self) -> f32 {
        self.speed_multiplier
    }

    /// Get the current frame duration adjusted by speed multiplier
    fn current_frame_duration(&self) -> Duration {
        let base_duration = self.animation.current_frame().duration_ms();
        let adjusted_ms = (base_duration as f32 / self.speed_multiplier) as u64;
        Duration::from_millis(adjusted_ms)
    }

    /// Get the animation mode
    pub fn mode(&self) -> AnimationMode {
        self.animation.mode()
    }

    /// Set the animation mode
    pub fn set_mode(&mut self, mode: AnimationMode) {
        self.animation.set_mode(mode);
    }

    /// Check if the animation has completed
    pub fn is_complete(&self) -> bool {
        self.animation.is_complete()
    }

    /// Get the underlying sprite
    pub fn sprite(&self) -> &Sprite {
        self.animation.sprite()
    }

    /// Get time until next frame
    pub fn time_until_next_frame(&self) -> Duration {
        let frame_duration = self.current_frame_duration();
        let elapsed = self.last_update.elapsed();

        if elapsed >= frame_duration {
            Duration::ZERO
        } else {
            frame_duration - elapsed
        }
    }
}

/// A positioned animated sprite that manages its own state and rendering
///
/// This is a higher-level widget that combines position, animation state,
/// and rendering in a single type. Useful for game-like applications where
/// you have multiple independently animated sprites.
///
/// # Examples
///
/// ```no_run
/// use cmdai::rendering::{Sprite, AnimationMode, ratatui_widget::AnimatedSprite};
///
/// let mut sprite = AnimatedSprite::new(sprite_data, AnimationMode::Loop, 10, 5);
///
/// // In your update loop:
/// sprite.tick();
///
/// // In your render loop:
/// sprite.render_to_buffer(buf);
/// ```
pub struct AnimatedSprite {
    controller: AnimationController,
    x: u16,
    y: u16,
    visible: bool,
}

impl AnimatedSprite {
    /// Create a new animated sprite at the specified position
    pub fn new(sprite: Sprite, mode: AnimationMode, x: u16, y: u16) -> Self {
        Self {
            controller: AnimationController::new(sprite, mode),
            x,
            y,
            visible: true,
        }
    }

    /// Update the animation state
    ///
    /// Returns true if the animation should continue, false if complete
    pub fn tick(&mut self) -> bool {
        self.controller.update()
    }

    /// Render the sprite to a Ratatui buffer
    pub fn render_to_buffer(&self, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        let frame = self.controller.current_frame();
        let palette = self.controller.palette();

        let area = Rect::new(
            self.x,
            self.y,
            frame.width() as u16,
            frame.height() as u16,
        );

        render_frame_to_buffer(frame, palette, area, buf);
    }

    /// Get the sprite's position
    pub fn position(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    /// Set the sprite's position
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    /// Move the sprite by a relative amount
    pub fn translate(&mut self, dx: i16, dy: i16) {
        self.x = (self.x as i16 + dx).max(0) as u16;
        self.y = (self.y as i16 + dy).max(0) as u16;
    }

    /// Get sprite dimensions (width, height)
    pub fn dimensions(&self) -> (u16, u16) {
        let frame = self.controller.current_frame();
        (frame.width() as u16, frame.height() as u16)
    }

    /// Check if a point is within the sprite's bounds
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        let (width, height) = self.dimensions();
        x >= self.x && x < self.x + width && y >= self.y && y < self.y + height
    }

    /// Get the bounding rectangle of this sprite
    pub fn bounds(&self) -> Rect {
        let (width, height) = self.dimensions();
        Rect::new(self.x, self.y, width, height)
    }

    /// Check if this sprite intersects with another rectangle
    pub fn intersects(&self, other: Rect) -> bool {
        self.bounds().intersects(other)
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if sprite is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get mutable access to the animation controller
    pub fn controller_mut(&mut self) -> &mut AnimationController {
        &mut self.controller
    }

    /// Get immutable access to the animation controller
    pub fn controller(&self) -> &AnimationController {
        &self.controller
    }
}

/// A collection of animated sprites that can be updated and rendered together
///
/// This is useful for managing multiple sprites in a scene, such as in a game
/// or complex UI with multiple animated elements.
///
/// # Examples
///
/// ```no_run
/// use cmdai::rendering::{ratatui_widget::SpriteScene};
///
/// let mut scene = SpriteScene::new();
/// scene.add(sprite1);
/// scene.add(sprite2);
///
/// // In your update loop:
/// scene.update();
///
/// // In your render loop:
/// scene.render(buf);
/// ```
pub struct SpriteScene {
    sprites: Vec<AnimatedSprite>,
}

impl SpriteScene {
    /// Create a new empty sprite scene
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    /// Add a sprite to the scene
    pub fn add(&mut self, sprite: AnimatedSprite) {
        self.sprites.push(sprite);
    }

    /// Remove a sprite at the given index
    pub fn remove(&mut self, index: usize) -> Option<AnimatedSprite> {
        if index < self.sprites.len() {
            Some(self.sprites.remove(index))
        } else {
            None
        }
    }

    /// Update all sprites in the scene
    ///
    /// Removes sprites whose animations have completed (unless they loop)
    pub fn update(&mut self) {
        self.sprites.retain_mut(|sprite| sprite.tick());
    }

    /// Render all visible sprites to the buffer
    pub fn render(&self, buf: &mut Buffer) {
        for sprite in &self.sprites {
            sprite.render_to_buffer(buf);
        }
    }

    /// Get the number of sprites in the scene
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// Check if the scene is empty
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Get a reference to a sprite at the given index
    pub fn get(&self, index: usize) -> Option<&AnimatedSprite> {
        self.sprites.get(index)
    }

    /// Get a mutable reference to a sprite at the given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AnimatedSprite> {
        self.sprites.get_mut(index)
    }

    /// Find a sprite at the given position
    pub fn find_at_position(&self, x: u16, y: u16) -> Option<usize> {
        self.sprites
            .iter()
            .position(|sprite| sprite.contains_point(x, y))
    }

    /// Clear all sprites from the scene
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Iterate over all sprites
    pub fn iter(&self) -> impl Iterator<Item = &AnimatedSprite> {
        self.sprites.iter()
    }

    /// Iterate mutably over all sprites
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AnimatedSprite> {
        self.sprites.iter_mut()
    }
}

impl Default for SpriteScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::ColorPalette;

    fn create_test_sprite() -> Sprite {
        let palette = ColorPalette::from_hex_strings(&["#FF0000", "#00FF00"]).unwrap();
        let frame1 = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 100).unwrap();
        let frame2 = SpriteFrame::new(2, 2, vec![1, 0, 0, 1], 100).unwrap();
        Sprite::new("test".to_string(), palette, vec![frame1, frame2]).unwrap()
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::new(255, 128, 64);
        let ratatui_color = to_ratatui_color(&color);

        match ratatui_color {
            RatatuiColor::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 128);
                assert_eq!(b, 64);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_animation_controller() {
        let sprite = create_test_sprite();
        let mut controller = AnimationController::new(sprite, AnimationMode::Once);

        // Should start at frame 0
        assert!(!controller.is_complete());

        // Manually advance
        assert!(controller.advance()); // Frame 0 -> 1
        assert!(!controller.advance()); // Frame 1 -> 0 (complete)
        assert!(controller.is_complete());
    }

    #[test]
    fn test_animation_controller_speed() {
        let sprite = create_test_sprite();
        let mut controller = AnimationController::new(sprite, AnimationMode::Loop);

        controller.set_speed(2.0);
        assert_eq!(controller.speed(), 2.0);

        let normal_duration = Duration::from_millis(100);
        let fast_duration = controller.current_frame_duration();
        assert_eq!(fast_duration, Duration::from_millis(50));
    }

    #[test]
    fn test_animated_sprite() {
        let sprite = create_test_sprite();
        let mut animated = AnimatedSprite::new(sprite, AnimationMode::Loop, 5, 10);

        assert_eq!(animated.position(), (5, 10));
        assert_eq!(animated.dimensions(), (2, 2));
        assert!(animated.is_visible());

        animated.translate(3, -2);
        assert_eq!(animated.position(), (8, 8));

        assert!(animated.contains_point(8, 8));
        assert!(animated.contains_point(9, 9));
        assert!(!animated.contains_point(10, 10));
    }

    #[test]
    fn test_sprite_scene() {
        let mut scene = SpriteScene::new();
        assert!(scene.is_empty());

        let sprite1 = create_test_sprite();
        let sprite2 = create_test_sprite();

        scene.add(AnimatedSprite::new(sprite1, AnimationMode::Loop, 0, 0));
        scene.add(AnimatedSprite::new(sprite2, AnimationMode::Loop, 10, 10));

        assert_eq!(scene.len(), 2);

        // Find sprite at position
        assert_eq!(scene.find_at_position(0, 0), Some(0));
        assert_eq!(scene.find_at_position(10, 10), Some(1));
        assert_eq!(scene.find_at_position(5, 5), None);

        scene.clear();
        assert!(scene.is_empty());
    }

    #[test]
    fn test_bounds_and_intersection() {
        let sprite = create_test_sprite();
        let animated = AnimatedSprite::new(sprite, AnimationMode::Loop, 5, 5);

        let bounds = animated.bounds();
        assert_eq!(bounds.x, 5);
        assert_eq!(bounds.y, 5);
        assert_eq!(bounds.width, 2);
        assert_eq!(bounds.height, 2);

        let overlapping = Rect::new(6, 6, 5, 5);
        assert!(animated.intersects(overlapping));

        let non_overlapping = Rect::new(20, 20, 5, 5);
        assert!(!animated.intersects(non_overlapping));
    }
}

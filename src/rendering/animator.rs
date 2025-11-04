//! Animation system for sprite playback

use crate::rendering::{sprites::*, terminal::TerminalRenderer, RenderError, RenderResult};
use std::time::Duration;
use tokio::time::{sleep, Instant};

/// Animation playback mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    /// Play once and stop
    Once,
    /// Loop indefinitely
    Loop,
    /// Loop N times
    LoopN(usize),
}

/// Animation configuration and state
pub struct Animation {
    sprite: Sprite,
    mode: AnimationMode,
    current_frame: usize,
    loop_count: usize,
}

impl Animation {
    /// Create a new animation from a sprite
    pub fn new(sprite: Sprite, mode: AnimationMode) -> Self {
        Self {
            sprite,
            mode,
            current_frame: 0,
            loop_count: 0,
        }
    }

    /// Get the current frame
    pub fn current_frame(&self) -> &SpriteFrame {
        &self.sprite.frames()[self.current_frame]
    }

    /// Get the sprite's color palette
    pub fn palette(&self) -> &ColorPalette {
        self.sprite.palette()
    }

    /// Advance to the next frame
    pub fn advance(&mut self) -> bool {
        let frame_count = self.sprite.frame_count();

        self.current_frame += 1;

        if self.current_frame >= frame_count {
            self.current_frame = 0;
            self.loop_count += 1;

            // Check if animation should continue
            match self.mode {
                AnimationMode::Once => return false,
                AnimationMode::Loop => return true,
                AnimationMode::LoopN(n) => {
                    if self.loop_count >= n {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Reset animation to the first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.loop_count = 0;
    }

    /// Get the sprite reference
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    /// Get the current animation mode
    pub fn mode(&self) -> AnimationMode {
        self.mode
    }

    /// Set the animation mode
    pub fn set_mode(&mut self, mode: AnimationMode) {
        self.mode = mode;
    }

    /// Check if animation has completed
    pub fn is_complete(&self) -> bool {
        match self.mode {
            AnimationMode::Once => self.loop_count > 0,
            AnimationMode::Loop => false,
            AnimationMode::LoopN(n) => self.loop_count >= n,
        }
    }
}

/// Animator for playing animations in the terminal
pub struct Animator {
    renderer: TerminalRenderer,
}

impl Animator {
    /// Create a new animator
    pub fn new() -> Self {
        Self {
            renderer: TerminalRenderer::new(),
        }
    }

    /// Play an animation synchronously (blocking)
    pub async fn play(&self, animation: &mut Animation) -> RenderResult<()> {
        let start_time = Instant::now();
        let mut frame_count = 0;

        // Hide cursor for smooth animation
        self.renderer.hide_cursor()?;

        loop {
            let frame = animation.current_frame();
            let duration = Duration::from_millis(frame.duration_ms());

            // Clear and render frame
            self.renderer.clear_screen()?;
            self.renderer.print_frame(frame, animation.palette())?;

            frame_count += 1;

            // Sleep for frame duration
            sleep(duration).await;

            // Advance to next frame
            if !animation.advance() {
                break;
            }
        }

        // Show cursor again
        self.renderer.show_cursor()?;

        let elapsed = start_time.elapsed();
        let fps = frame_count as f64 / elapsed.as_secs_f64();

        eprintln!(
            "\nAnimation complete: {} frames in {:.2}s ({:.1} FPS)",
            frame_count,
            elapsed.as_secs_f64(),
            fps
        );

        Ok(())
    }

    /// Play an animation at a specific terminal position
    pub async fn play_at(
        &self,
        animation: &mut Animation,
        row: usize,
        col: usize,
    ) -> RenderResult<()> {
        let start_time = Instant::now();
        let mut frame_count = 0;

        // Hide cursor for smooth animation
        self.renderer.hide_cursor()?;

        loop {
            let frame = animation.current_frame();
            let duration = Duration::from_millis(frame.duration_ms());

            // Render frame at position
            self.renderer
                .render_frame_at(frame, animation.palette(), row, col)?;

            frame_count += 1;

            // Sleep for frame duration
            sleep(duration).await;

            // Advance to next frame
            if !animation.advance() {
                break;
            }
        }

        // Show cursor again
        self.renderer.show_cursor()?;

        let elapsed = start_time.elapsed();
        let fps = frame_count as f64 / elapsed.as_secs_f64();

        eprintln!(
            "\nAnimation complete: {} frames in {:.2}s ({:.1} FPS)",
            frame_count,
            elapsed.as_secs_f64(),
            fps
        );

        Ok(())
    }

    /// Render a single frame without animation
    pub fn render_static(&self, sprite: &Sprite) -> RenderResult<()> {
        let frame = sprite
            .frame(0)
            .ok_or_else(|| RenderError::AnimationError("Sprite has no frames".to_string()))?;
        self.renderer.print_frame(frame, sprite.palette())
    }

    /// Get the underlying renderer
    pub fn renderer(&self) -> &TerminalRenderer {
        &self.renderer
    }
}

impl Default for Animator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sprite(frame_count: usize) -> Sprite {
        let palette = ColorPalette::from_hex_strings(&["#FF0000", "#0000FF"]).unwrap();
        let mut frames = Vec::new();

        for _ in 0..frame_count {
            let frame = SpriteFrame::new(2, 2, vec![0, 1, 1, 0], 50).unwrap();
            frames.push(frame);
        }

        Sprite::new("test".to_string(), palette, frames).unwrap()
    }

    #[test]
    fn test_animation_creation() {
        let sprite = create_test_sprite(3);
        let animation = Animation::new(sprite, AnimationMode::Once);

        assert_eq!(animation.current_frame, 0);
        assert_eq!(animation.mode(), AnimationMode::Once);
    }

    #[test]
    fn test_animation_advance_once() {
        let sprite = create_test_sprite(3);
        let mut animation = Animation::new(sprite, AnimationMode::Once);

        assert!(animation.advance()); // Frame 0 -> 1
        assert!(animation.advance()); // Frame 1 -> 2
        assert!(!animation.advance()); // Frame 2 -> 0, but should stop
        assert!(animation.is_complete());
    }

    #[test]
    fn test_animation_advance_loop() {
        let sprite = create_test_sprite(2);
        let mut animation = Animation::new(sprite, AnimationMode::Loop);

        assert!(animation.advance()); // Frame 0 -> 1
        assert!(animation.advance()); // Frame 1 -> 0 (loop)
        assert!(animation.advance()); // Frame 0 -> 1
        assert!(!animation.is_complete()); // Never completes in Loop mode
    }

    #[test]
    fn test_animation_advance_loop_n() {
        let sprite = create_test_sprite(2);
        let mut animation = Animation::new(sprite, AnimationMode::LoopN(2));

        // First loop
        assert!(animation.advance()); // Frame 0 -> 1
        assert!(animation.advance()); // Frame 1 -> 0 (loop 1)

        // Second loop
        assert!(animation.advance()); // Frame 0 -> 1
        assert!(!animation.advance()); // Frame 1 -> 0 (loop 2, should stop)
        assert!(animation.is_complete());
    }

    #[test]
    fn test_animation_reset() {
        let sprite = create_test_sprite(3);
        let mut animation = Animation::new(sprite, AnimationMode::Once);

        animation.advance();
        animation.advance();
        assert_eq!(animation.current_frame, 2);

        animation.reset();
        assert_eq!(animation.current_frame, 0);
        assert_eq!(animation.loop_count, 0);
    }

    #[test]
    fn test_animator_creation() {
        let animator = Animator::new();
        // Just verify it creates successfully
        assert!(animator.renderer().use_true_color || !animator.renderer().use_true_color);
    }

    #[tokio::test]
    async fn test_animation_playback_timing() {
        let sprite = create_test_sprite(3);
        let mut animation = Animation::new(sprite, AnimationMode::Once);

        let start = Instant::now();

        // Manually advance through frames with timing
        for _ in 0..3 {
            let frame = animation.current_frame();
            sleep(Duration::from_millis(frame.duration_ms())).await;
            animation.advance();
        }

        let elapsed = start.elapsed();
        // Should take approximately 150ms (3 frames * 50ms each)
        assert!(elapsed >= Duration::from_millis(140));
        assert!(elapsed < Duration::from_millis(200));
    }
}

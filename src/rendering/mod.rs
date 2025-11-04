//! Sprite animation rendering module for terminal-based pixel art
//!
//! This module provides functionality to render animated pixel art characters
//! in the terminal using Unicode block characters (█, ▀, ▄) with ANSI colors.

mod sprites;
mod animator;
mod terminal;
pub mod examples;

pub use sprites::{ColorPalette, Sprite, SpriteFrame};
pub use animator::{Animation, Animator, AnimationMode};
pub use terminal::TerminalRenderer;

/// Result type for rendering operations
pub type RenderResult<T> = Result<T, RenderError>;

/// Errors that can occur during rendering
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    #[error("Invalid sprite dimensions: {0}")]
    InvalidDimensions(String),

    #[error("Animation error: {0}")]
    AnimationError(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),
}

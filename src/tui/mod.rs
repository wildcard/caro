//! Terminal UI components and showcase framework
//!
//! This module provides a Storybook-like development environment for building
//! and testing Ratatui terminal UI components in isolation.

pub mod components;
pub mod showcase;

// Re-export commonly used types
pub use ratatui;
pub use crossterm;

pub use showcase::{ShowcaseComponent, ShowcaseStory, ComponentMetadata};

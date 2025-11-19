//! Showcase framework for TUI components
//!
//! This module provides the core abstractions for building isolated,
//! testable TUI components similar to React Storybook.

use ratatui::{
    backend::Backend,
    layout::Rect,
    Frame,
};
use std::io;

/// Metadata about a component showcase
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    /// Component name (e.g., "CommandPreview")
    pub name: String,
    /// Brief description of what the component does
    pub description: String,
    /// Category for organization (e.g., "Input", "Display", "Feedback")
    pub category: String,
    /// Version or iteration (for tracking component evolution)
    pub version: String,
}

impl ComponentMetadata {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category: "General".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

/// A story represents a specific variation or state of a component
///
/// Similar to Storybook stories, this allows showcasing different
/// configurations, states, or use cases of a component.
pub struct ShowcaseStory {
    /// Story name (e.g., "Default", "WithError", "Loading")
    pub name: String,
    /// Description of this specific story/variation
    pub description: String,
    /// The render function for this story
    pub render: Box<dyn Fn(&mut Frame, Rect) + Send + Sync>,
}

impl ShowcaseStory {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        render: impl Fn(&mut Frame, Rect) + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            render: Box::new(render),
        }
    }
}

/// Trait for components that can be showcased
///
/// Implement this trait to make your component available in the
/// TUI showcase browser. Each component can have multiple stories
/// showing different states or configurations.
pub trait ShowcaseComponent: Send + Sync {
    /// Get component metadata
    fn metadata(&self) -> ComponentMetadata;

    /// Get all stories for this component
    fn stories(&self) -> Vec<ShowcaseStory>;

    /// Optional: Handle key events for interactive components
    ///
    /// Return true if the event was handled, false otherwise
    fn handle_key_event(&mut self, _event: crossterm::event::KeyEvent) -> io::Result<bool> {
        Ok(false)
    }

    /// Optional: Initialize component state
    fn init(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Optional: Cleanup component state
    fn cleanup(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Registry for all showcase components
pub struct ShowcaseRegistry {
    components: Vec<Box<dyn ShowcaseComponent>>,
}

impl ShowcaseRegistry {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Register a component in the showcase
    pub fn register(&mut self, component: Box<dyn ShowcaseComponent>) {
        self.components.push(component);
    }

    /// Get all registered components
    pub fn components(&self) -> &[Box<dyn ShowcaseComponent>] {
        &self.components
    }

    /// Get component by index
    pub fn get(&self, index: usize) -> Option<&Box<dyn ShowcaseComponent>> {
        self.components.get(index)
    }

    /// Get mutable component by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Box<dyn ShowcaseComponent>> {
        self.components.get_mut(index)
    }

    /// Get component count
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl Default for ShowcaseRegistry {
    fn default() -> Self {
        Self::new()
    }
}

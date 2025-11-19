//! Showcase framework for TUI components
//!
//! This module provides the core abstractions for building isolated,
//! testable TUI components similar to React Storybook.

use ratatui::{backend::Backend, layout::Rect, Frame};
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // ========================================================================
    // Mock Components for Testing
    // ========================================================================

    struct TestComponent {
        name: String,
        story_count: usize,
    }

    impl TestComponent {
        fn new(name: &str, story_count: usize) -> Self {
            Self {
                name: name.to_string(),
                story_count,
            }
        }
    }

    impl ShowcaseComponent for TestComponent {
        fn metadata(&self) -> ComponentMetadata {
            ComponentMetadata::new(&self.name, format!("{} description", self.name))
                .with_category("Test")
                .with_version("1.0.0")
        }

        fn stories(&self) -> Vec<ShowcaseStory> {
            (0..self.story_count)
                .map(|i| {
                    ShowcaseStory::new(
                        format!("Story {}", i + 1),
                        format!("Test story {}", i + 1),
                        |_frame, _area| {
                            // Minimal render
                        },
                    )
                })
                .collect()
        }
    }

    // ========================================================================
    // ComponentMetadata Tests
    // ========================================================================

    #[test]
    fn test_metadata_new_creates_correct_defaults() {
        let metadata = ComponentMetadata::new("TestComponent", "Test description");

        assert_eq!(metadata.name, "TestComponent");
        assert_eq!(metadata.description, "Test description");
        assert_eq!(metadata.category, "General");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_metadata_with_category_sets_category() {
        let metadata =
            ComponentMetadata::new("Test", "Description").with_category("CustomCategory");

        assert_eq!(metadata.category, "CustomCategory");
    }

    #[test]
    fn test_metadata_with_version_sets_version() {
        let metadata = ComponentMetadata::new("Test", "Description").with_version("2.0.0");

        assert_eq!(metadata.version, "2.0.0");
    }

    #[test]
    fn test_metadata_builder_pattern() {
        let metadata = ComponentMetadata::new("Test", "Description")
            .with_category("Display")
            .with_version("1.5.0");

        assert_eq!(metadata.name, "Test");
        assert_eq!(metadata.description, "Description");
        assert_eq!(metadata.category, "Display");
        assert_eq!(metadata.version, "1.5.0");
    }

    #[test]
    fn test_metadata_accepts_string_refs_and_owned() {
        let name = "Test".to_string();
        let desc = "Description".to_string();

        let metadata1 = ComponentMetadata::new(&name, &desc);
        let metadata2 = ComponentMetadata::new(name.clone(), desc.clone());

        assert_eq!(metadata1.name, metadata2.name);
        assert_eq!(metadata1.description, metadata2.description);
    }

    // ========================================================================
    // ShowcaseStory Tests
    // ========================================================================

    #[test]
    fn test_story_new_creates_correct_fields() {
        let story = ShowcaseStory::new("DefaultStory", "A default story", |_frame, _area| {});

        assert_eq!(story.name, "DefaultStory");
        assert_eq!(story.description, "A default story");
    }

    #[test]
    fn test_story_render_function_can_be_called() {
        let story = ShowcaseStory::new("Test", "Test story", |_frame, _area| {
            // This should not panic
        });

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = frame.size();
            (story.render)(frame, area);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_story_accepts_string_refs_and_owned() {
        let name = "Story1".to_string();
        let desc = "Description".to_string();

        let story1 = ShowcaseStory::new(&name, &desc, |_f, _a| {});
        let story2 = ShowcaseStory::new(name.clone(), desc.clone(), |_f, _a| {});

        assert_eq!(story1.name, story2.name);
        assert_eq!(story1.description, story2.description);
    }

    // ========================================================================
    // ShowcaseRegistry Tests
    // ========================================================================

    #[test]
    fn test_registry_new_creates_empty_registry() {
        let registry = ShowcaseRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_default_creates_empty_registry() {
        let registry = ShowcaseRegistry::default();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_register_adds_component() {
        let mut registry = ShowcaseRegistry::new();
        registry.register(Box::new(TestComponent::new("Component1", 3)));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_register_multiple_components() {
        let mut registry = ShowcaseRegistry::new();

        registry.register(Box::new(TestComponent::new("Component1", 3)));
        registry.register(Box::new(TestComponent::new("Component2", 5)));
        registry.register(Box::new(TestComponent::new("Component3", 2)));

        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_get_returns_correct_component() {
        let mut registry = ShowcaseRegistry::new();
        registry.register(Box::new(TestComponent::new("Component1", 3)));
        registry.register(Box::new(TestComponent::new("Component2", 5)));

        let component = registry.get(0).expect("Component should exist");
        assert_eq!(component.metadata().name, "Component1");

        let component = registry.get(1).expect("Component should exist");
        assert_eq!(component.metadata().name, "Component2");
    }

    #[test]
    fn test_registry_get_returns_none_for_invalid_index() {
        let registry = ShowcaseRegistry::new();
        assert!(registry.get(0).is_none());
        assert!(registry.get(100).is_none());
    }

    #[test]
    fn test_registry_get_mut_allows_modification() {
        let mut registry = ShowcaseRegistry::new();
        registry.register(Box::new(TestComponent::new("Component1", 3)));

        let component = registry.get_mut(0).expect("Component should exist");

        // Test that we can call mutable methods
        let init_result = component.init();
        assert!(init_result.is_ok());
    }

    #[test]
    fn test_registry_get_mut_returns_none_for_invalid_index() {
        let mut registry = ShowcaseRegistry::new();
        assert!(registry.get_mut(0).is_none());
        assert!(registry.get_mut(100).is_none());
    }

    #[test]
    fn test_registry_components_returns_all_components() {
        let mut registry = ShowcaseRegistry::new();
        registry.register(Box::new(TestComponent::new("Component1", 3)));
        registry.register(Box::new(TestComponent::new("Component2", 5)));

        let components = registry.components();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].metadata().name, "Component1");
        assert_eq!(components[1].metadata().name, "Component2");
    }

    #[test]
    fn test_registry_maintains_insertion_order() {
        let mut registry = ShowcaseRegistry::new();

        let names = vec!["Alpha", "Beta", "Gamma", "Delta"];
        for name in &names {
            registry.register(Box::new(TestComponent::new(name, 1)));
        }

        for (i, name) in names.iter().enumerate() {
            let component = registry.get(i).expect("Component should exist");
            assert_eq!(component.metadata().name, *name);
        }
    }

    // ========================================================================
    // ShowcaseComponent Trait Default Implementations
    // ========================================================================

    #[test]
    fn test_component_default_init_succeeds() {
        let mut component = TestComponent::new("Test", 3);
        let result = component.init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_default_cleanup_succeeds() {
        let mut component = TestComponent::new("Test", 3);
        let result = component.cleanup();
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_default_handle_key_event_returns_false() {
        let mut component = TestComponent::new("Test", 3);
        let key_event = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter);
        let result = component.handle_key_event(key_event).unwrap();
        assert!(!result, "Default implementation should return false");
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_end_to_end_component_registration_and_retrieval() {
        let mut registry = ShowcaseRegistry::new();

        // Register multiple components with different story counts
        registry.register(Box::new(TestComponent::new("SimpleText", 3)));
        registry.register(Box::new(TestComponent::new("CommandPreview", 5)));
        registry.register(Box::new(TestComponent::new("TableSelector", 7)));

        // Verify all components are registered
        assert_eq!(registry.len(), 3);

        // Verify each component's metadata
        let comp1 = registry.get(0).unwrap();
        assert_eq!(comp1.metadata().name, "SimpleText");
        assert_eq!(comp1.stories().len(), 3);

        let comp2 = registry.get(1).unwrap();
        assert_eq!(comp2.metadata().name, "CommandPreview");
        assert_eq!(comp2.stories().len(), 5);

        let comp3 = registry.get(2).unwrap();
        assert_eq!(comp3.metadata().name, "TableSelector");
        assert_eq!(comp3.stories().len(), 7);
    }

    #[test]
    fn test_component_stories_can_be_rendered() {
        let component = TestComponent::new("Test", 3);
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Each story should render without error
        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });
            assert!(result.is_ok(), "Story '{}' failed to render", story.name);
        }
    }
}

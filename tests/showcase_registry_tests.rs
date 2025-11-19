//! Integration tests for ShowcaseRegistry
//!
//! These tests verify the showcase framework's registry functionality,
//! including component registration, retrieval, and lifecycle management.

use cmdai::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseRegistry, ShowcaseStory};
use std::io;

// ============================================================================
// Test Helper: Mock Components
// ============================================================================

struct MockComponent {
    name: String,
    init_called: bool,
    cleanup_called: bool,
}

impl MockComponent {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            init_called: false,
            cleanup_called: false,
        }
    }
}

impl ShowcaseComponent for MockComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(&self.name, format!("{} description", self.name))
            .with_category("Test")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![ShowcaseStory::new(
            "Default",
            "Default story",
            |_frame, _area| {},
        )]
    }

    fn init(&mut self) -> io::Result<()> {
        self.init_called = true;
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        self.cleanup_called = true;
        Ok(())
    }
}

// ============================================================================
// Registry Creation Tests
// ============================================================================

#[test]
fn test_registry_new_is_empty() {
    let registry = ShowcaseRegistry::new();
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_registry_default_is_empty() {
    let registry = ShowcaseRegistry::default();
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

// ============================================================================
// Component Registration Tests
// ============================================================================

#[test]
fn test_register_single_component() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("Component1")));

    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}

#[test]
fn test_register_multiple_components() {
    let mut registry = ShowcaseRegistry::new();

    registry.register(Box::new(MockComponent::new("Component1")));
    registry.register(Box::new(MockComponent::new("Component2")));
    registry.register(Box::new(MockComponent::new("Component3")));

    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());
}

#[test]
fn test_register_maintains_insertion_order() {
    let mut registry = ShowcaseRegistry::new();

    let names = vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
    for name in &names {
        registry.register(Box::new(MockComponent::new(name)));
    }

    assert_eq!(registry.len(), names.len());

    for (i, expected_name) in names.iter().enumerate() {
        let component = registry.get(i).expect("Component should exist");
        let metadata = component.metadata();
        assert_eq!(
            metadata.name, *expected_name,
            "Component at index {} should be '{}'",
            i, expected_name
        );
    }
}

#[test]
fn test_register_large_number_of_components() {
    let mut registry = ShowcaseRegistry::new();

    let count = 100;
    for i in 0..count {
        registry.register(Box::new(MockComponent::new(&format!("Component{}", i))));
    }

    assert_eq!(registry.len(), count);
}

// ============================================================================
// Component Retrieval Tests
// ============================================================================

#[test]
fn test_get_component_by_index() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("First")));
    registry.register(Box::new(MockComponent::new("Second")));
    registry.register(Box::new(MockComponent::new("Third")));

    let first = registry.get(0).expect("First component should exist");
    assert_eq!(first.metadata().name, "First");

    let second = registry.get(1).expect("Second component should exist");
    assert_eq!(second.metadata().name, "Second");

    let third = registry.get(2).expect("Third component should exist");
    assert_eq!(third.metadata().name, "Third");
}

#[test]
fn test_get_returns_none_for_invalid_index() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("Component1")));

    assert!(registry.get(1).is_none());
    assert!(registry.get(100).is_none());
}

#[test]
fn test_get_returns_none_for_empty_registry() {
    let registry = ShowcaseRegistry::new();
    assert!(registry.get(0).is_none());
}

#[test]
fn test_components_returns_all_registered_components() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("Component1")));
    registry.register(Box::new(MockComponent::new("Component2")));

    let components = registry.components();
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].metadata().name, "Component1");
    assert_eq!(components[1].metadata().name, "Component2");
}

#[test]
fn test_components_returns_empty_slice_for_empty_registry() {
    let registry = ShowcaseRegistry::new();
    let components = registry.components();
    assert_eq!(components.len(), 0);
}

// ============================================================================
// Mutable Access Tests
// ============================================================================

#[test]
fn test_get_mut_allows_component_modification() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("Component1")));

    let component = registry.get_mut(0).expect("Component should exist");

    // Call init - this should work and modify the component
    let result = component.init();
    assert!(result.is_ok());
}

#[test]
fn test_get_mut_returns_none_for_invalid_index() {
    let mut registry = ShowcaseRegistry::new();
    assert!(registry.get_mut(0).is_none());
}

#[test]
fn test_component_lifecycle_methods_work_through_registry() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("Component1")));

    // Get mutable reference and call lifecycle methods
    let component = registry.get_mut(0).expect("Component should exist");

    let init_result = component.init();
    assert!(init_result.is_ok());

    let cleanup_result = component.cleanup();
    assert!(cleanup_result.is_ok());
}

// ============================================================================
// Component Metadata Access Tests
// ============================================================================

#[test]
fn test_accessing_metadata_through_registry() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("TestComponent")));

    let component = registry.get(0).expect("Component should exist");
    let metadata = component.metadata();

    assert_eq!(metadata.name, "TestComponent");
    assert_eq!(metadata.description, "TestComponent description");
    assert_eq!(metadata.category, "Test");
}

#[test]
fn test_accessing_stories_through_registry() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(MockComponent::new("TestComponent")));

    let component = registry.get(0).expect("Component should exist");
    let stories = component.stories();

    assert_eq!(stories.len(), 1);
    assert_eq!(stories[0].name, "Default");
    assert_eq!(stories[0].description, "Default story");
}

// ============================================================================
// Registry State Tests
// ============================================================================

#[test]
fn test_len_reflects_component_count() {
    let mut registry = ShowcaseRegistry::new();
    assert_eq!(registry.len(), 0);

    registry.register(Box::new(MockComponent::new("Component1")));
    assert_eq!(registry.len(), 1);

    registry.register(Box::new(MockComponent::new("Component2")));
    assert_eq!(registry.len(), 2);

    registry.register(Box::new(MockComponent::new("Component3")));
    assert_eq!(registry.len(), 3);
}

#[test]
fn test_is_empty_reflects_registry_state() {
    let mut registry = ShowcaseRegistry::new();
    assert!(registry.is_empty());

    registry.register(Box::new(MockComponent::new("Component1")));
    assert!(!registry.is_empty());
}

// ============================================================================
// Integration Test: Full Workflow
// ============================================================================

#[test]
fn test_end_to_end_registry_workflow() {
    // Create a new registry
    let mut registry = ShowcaseRegistry::new();
    assert!(registry.is_empty());

    // Register multiple components
    let component_names = vec!["SimpleText", "CommandPreview", "TableSelector"];
    for name in &component_names {
        registry.register(Box::new(MockComponent::new(name)));
    }

    // Verify all components are registered
    assert_eq!(registry.len(), component_names.len());
    assert!(!registry.is_empty());

    // Access each component and verify metadata
    for (i, expected_name) in component_names.iter().enumerate() {
        let component = registry.get(i).expect("Component should exist");
        let metadata = component.metadata();

        assert_eq!(metadata.name, *expected_name);
        assert_eq!(metadata.category, "Test");

        // Verify stories are accessible
        let stories = component.stories();
        assert!(!stories.is_empty());
    }

    // Initialize all components
    for i in 0..registry.len() {
        let component = registry.get_mut(i).expect("Component should exist");
        let result = component.init();
        assert!(result.is_ok(), "Init should succeed for component {}", i);
    }

    // Cleanup all components
    for i in 0..registry.len() {
        let component = registry.get_mut(i).expect("Component should exist");
        let result = component.cleanup();
        assert!(result.is_ok(), "Cleanup should succeed for component {}", i);
    }
}

#[test]
fn test_multiple_registries_are_independent() {
    let mut registry1 = ShowcaseRegistry::new();
    let mut registry2 = ShowcaseRegistry::new();

    registry1.register(Box::new(MockComponent::new("Component1")));
    registry2.register(Box::new(MockComponent::new("Component2")));
    registry2.register(Box::new(MockComponent::new("Component3")));

    assert_eq!(registry1.len(), 1);
    assert_eq!(registry2.len(), 2);

    assert_eq!(registry1.get(0).unwrap().metadata().name, "Component1");
    assert_eq!(registry2.get(0).unwrap().metadata().name, "Component2");
}

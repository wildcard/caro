//! Test helpers and utilities for TUI showcase testing
//!
//! This module provides common test fixtures, mock components, and utility
//! functions to make writing tests easier and reduce duplication.

use cmdai::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    backend::TestBackend,
    layout::Rect,
    Frame,
    Terminal,
};
use std::io;

/// Create a test area with specified dimensions
///
/// # Example
/// ```
/// let area = test_area(80, 24);
/// assert_eq!(area.width, 80);
/// assert_eq!(area.height, 24);
/// ```
pub fn test_area(width: u16, height: u16) -> Rect {
    Rect {
        x: 0,
        y: 0,
        width,
        height,
    }
}

/// Create a test terminal with default dimensions (80x24)
pub fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Create a test terminal with custom dimensions
pub fn create_test_terminal_with_size(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Assert that a component has the expected metadata
///
/// # Example
/// ```
/// let component = SimpleTextComponent;
/// assert_component_metadata(
///     &component,
///     "SimpleText",
///     "Basic text display with various styling options",
///     "Display",
///     "1.0.0"
/// );
/// ```
pub fn assert_component_metadata(
    component: &dyn ShowcaseComponent,
    expected_name: &str,
    expected_description: &str,
    expected_category: &str,
    expected_version: &str,
) {
    let metadata = component.metadata();
    assert_eq!(
        metadata.name, expected_name,
        "Component name mismatch"
    );
    assert_eq!(
        metadata.description, expected_description,
        "Component description mismatch"
    );
    assert_eq!(
        metadata.category, expected_category,
        "Component category mismatch"
    );
    assert_eq!(
        metadata.version, expected_version,
        "Component version mismatch"
    );
}

/// Assert that a render function doesn't panic when called
///
/// Returns true if render succeeded, panics with descriptive message if it fails
pub fn assert_renders_without_panic<F>(render_fn: F, story_name: &str) -> bool
where
    F: Fn(&mut Frame, Rect) + std::panic::UnwindSafe,
{
    let mut terminal = create_test_terminal();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        terminal
            .draw(|frame| {
                let area = frame.size();
                render_fn(frame, area);
            })
            .expect("Failed to draw to terminal");
    }));

    match result {
        Ok(_) => true,
        Err(e) => {
            panic!(
                "Story '{}' panicked during render: {:?}",
                story_name, e
            );
        }
    }
}

/// Test all stories of a component render without panicking
///
/// # Example
/// ```
/// let component = SimpleTextComponent;
/// assert_all_stories_render(&component);
/// ```
pub fn assert_all_stories_render(component: &dyn ShowcaseComponent) {
    let stories = component.stories();
    let component_name = component.metadata().name;

    for story in stories {
        assert_renders_without_panic(
            |frame, area| (story.render)(frame, area),
            &format!("{}::{}", component_name, story.name),
        );
    }
}

/// Assert that a component has the expected number of stories
pub fn assert_story_count(component: &dyn ShowcaseComponent, expected_count: usize) {
    let stories = component.stories();
    assert_eq!(
        stories.len(),
        expected_count,
        "Expected {} stories for {}, but found {}",
        expected_count,
        component.metadata().name,
        stories.len()
    );
}

/// Assert that a component's stories have the expected names
pub fn assert_story_names(component: &dyn ShowcaseComponent, expected_names: &[&str]) {
    let stories = component.stories();
    let actual_names: Vec<String> = stories.iter().map(|s| s.name.clone()).collect();

    assert_eq!(
        actual_names.len(),
        expected_names.len(),
        "Story count mismatch for {}",
        component.metadata().name
    );

    for (i, expected) in expected_names.iter().enumerate() {
        assert_eq!(
            actual_names[i], *expected,
            "Story name mismatch at index {}: expected '{}', got '{}'",
            i, expected, actual_names[i]
        );
    }
}

/// Test that render function works with various screen sizes
pub fn test_render_with_various_sizes<F>(render_fn: F, story_name: &str)
where
    F: Fn(&mut Frame, Rect) + std::panic::UnwindSafe + Copy,
{
    let test_sizes = [
        (80, 24),   // Standard terminal
        (120, 40),  // Large terminal
        (40, 10),   // Small terminal
        (20, 5),    // Very small
        (200, 60),  // Very large
    ];

    for (width, height) in test_sizes {
        let mut terminal = create_test_terminal_with_size(width, height);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            terminal
                .draw(|frame| {
                    let area = frame.size();
                    render_fn(frame, area);
                })
                .expect("Failed to draw to terminal");
        }));

        assert!(
            result.is_ok(),
            "Story '{}' panicked with terminal size {}x{}",
            story_name, width, height
        );
    }
}

// ============================================================================
// Mock Components for Framework Testing
// ============================================================================

/// A minimal mock component for testing the showcase framework
pub struct MockComponent {
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub story_count: usize,
}

impl MockComponent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: format!("{} description", name),
            category: "Test".to_string(),
            version: "1.0.0".to_string(),
            story_count: 3,
        }
    }

    pub fn with_stories(mut self, count: usize) -> Self {
        self.story_count = count;
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }
}

impl ShowcaseComponent for MockComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(&self.name, &self.description)
            .with_category(&self.category)
            .with_version(&self.version)
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        (0..self.story_count)
            .map(|i| {
                ShowcaseStory::new(
                    format!("Story {}", i + 1),
                    format!("Test story {} for {}", i + 1, self.name),
                    |_frame, _area| {
                        // Minimal render implementation
                    },
                )
            })
            .collect()
    }

    fn init(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A mock component that panics during rendering (for error testing)
pub struct PanickingComponent;

impl ShowcaseComponent for PanickingComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new("Panicking", "A component that panics")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![ShowcaseStory::new(
            "Panic Story",
            "This story will panic",
            |_frame, _area| {
                panic!("Intentional panic for testing");
            },
        )]
    }
}

/// A mock component that fails initialization
pub struct FailingInitComponent {
    pub init_should_fail: bool,
}

impl FailingInitComponent {
    pub fn new() -> Self {
        Self {
            init_should_fail: true,
        }
    }
}

impl ShowcaseComponent for FailingInitComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new("FailingInit", "A component that fails to initialize")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![ShowcaseStory::new(
            "Test Story",
            "A test story",
            |_frame, _area| {},
        )]
    }

    fn init(&mut self) -> io::Result<()> {
        if self.init_should_fail {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Intentional init failure",
            ))
        } else {
            Ok(())
        }
    }
}

// ============================================================================
// Test Data Builders
// ============================================================================

/// Builder for creating test ComponentMetadata
pub struct MetadataBuilder {
    name: String,
    description: String,
    category: String,
    version: String,
}

impl MetadataBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: format!("{} description", name),
            category: "General".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn build(self) -> ComponentMetadata {
        ComponentMetadata::new(self.name, self.description)
            .with_category(self.category)
            .with_version(self.version)
    }
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that two ComponentMetadata instances are equal
pub fn assert_metadata_equal(actual: &ComponentMetadata, expected: &ComponentMetadata) {
    assert_eq!(actual.name, expected.name, "Metadata name mismatch");
    assert_eq!(
        actual.description, expected.description,
        "Metadata description mismatch"
    );
    assert_eq!(
        actual.category, expected.category,
        "Metadata category mismatch"
    );
    assert_eq!(
        actual.version, expected.version,
        "Metadata version mismatch"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_area_creates_correct_dimensions() {
        let area = test_area(100, 50);
        assert_eq!(area.x, 0);
        assert_eq!(area.y, 0);
        assert_eq!(area.width, 100);
        assert_eq!(area.height, 50);
    }

    #[test]
    fn test_create_test_terminal_default_size() {
        let terminal = create_test_terminal();
        assert_eq!(terminal.size().unwrap().width, 80);
        assert_eq!(terminal.size().unwrap().height, 24);
    }

    #[test]
    fn test_create_test_terminal_custom_size() {
        let terminal = create_test_terminal_with_size(120, 40);
        assert_eq!(terminal.size().unwrap().width, 120);
        assert_eq!(terminal.size().unwrap().height, 40);
    }

    #[test]
    fn test_mock_component_metadata() {
        let component = MockComponent::new("TestComponent");
        let metadata = component.metadata();

        assert_eq!(metadata.name, "TestComponent");
        assert_eq!(metadata.description, "TestComponent description");
        assert_eq!(metadata.category, "Test");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_mock_component_story_count() {
        let component = MockComponent::new("Test").with_stories(5);
        assert_eq!(component.stories().len(), 5);
    }

    #[test]
    fn test_metadata_builder() {
        let metadata = MetadataBuilder::new("TestComponent")
            .description("Custom description")
            .category("CustomCategory")
            .version("2.0.0")
            .build();

        assert_eq!(metadata.name, "TestComponent");
        assert_eq!(metadata.description, "Custom description");
        assert_eq!(metadata.category, "CustomCategory");
        assert_eq!(metadata.version, "2.0.0");
    }
}

//! Integration tests for the TUI showcase browser
//!
//! These tests verify the browser application logic, including state
//! transitions, navigation flow, and keyboard input handling.

// Note: Since the App struct is in the binary (tui_showcase.rs) and not
// exposed as a library, we test the patterns and behaviors through
// the public ShowcaseRegistry API. For full browser integration testing,
// we would need to extract App into a library module.

use cmdai::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseRegistry, ShowcaseStory};

// ============================================================================
// Mock Component for Testing
// ============================================================================

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
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        (0..self.story_count)
            .map(|i| {
                ShowcaseStory::new(
                    format!("Story {}", i + 1),
                    format!("Test story {}", i + 1),
                    |_frame, _area| {},
                )
            })
            .collect()
    }
}

// ============================================================================
// Browser-like State Machine Tests
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewState {
    ComponentList,
    StoryList,
    StoryView,
}

struct SimpleBrowserState {
    registry: ShowcaseRegistry,
    view_state: ViewState,
    selected_component: usize,
    selected_story: usize,
}

impl SimpleBrowserState {
    fn new(registry: ShowcaseRegistry) -> Self {
        Self {
            registry,
            view_state: ViewState::ComponentList,
            selected_component: 0,
            selected_story: 0,
        }
    }

    fn select_component(&mut self, index: usize) -> bool {
        if index < self.registry.len() {
            self.selected_component = index;
            self.view_state = ViewState::StoryList;
            self.selected_story = 0;
            true
        } else {
            false
        }
    }

    fn select_story(&mut self, index: usize) -> bool {
        if let Some(component) = self.registry.get(self.selected_component) {
            if index < component.stories().len() {
                self.selected_story = index;
                self.view_state = ViewState::StoryView;
                return true;
            }
        }
        false
    }

    fn go_back(&mut self) {
        match self.view_state {
            ViewState::StoryView => self.view_state = ViewState::StoryList,
            ViewState::StoryList => self.view_state = ViewState::ComponentList,
            ViewState::ComponentList => {} // Already at top level
        }
    }
}

// ============================================================================
// State Transition Tests
// ============================================================================

#[test]
fn test_initial_state_is_component_list() {
    let registry = ShowcaseRegistry::new();
    let state = SimpleBrowserState::new(registry);

    assert_eq!(state.view_state, ViewState::ComponentList);
    assert_eq!(state.selected_component, 0);
    assert_eq!(state.selected_story, 0);
}

#[test]
fn test_selecting_component_transitions_to_story_list() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);
    assert_eq!(state.view_state, ViewState::ComponentList);

    let result = state.select_component(0);
    assert!(result);
    assert_eq!(state.view_state, ViewState::StoryList);
    assert_eq!(state.selected_component, 0);
}

#[test]
fn test_selecting_invalid_component_fails() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);

    let result = state.select_component(10);
    assert!(!result);
    assert_eq!(state.view_state, ViewState::ComponentList);
}

#[test]
fn test_selecting_story_transitions_to_story_view() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);

    state.select_component(0);
    assert_eq!(state.view_state, ViewState::StoryList);

    let result = state.select_story(0);
    assert!(result);
    assert_eq!(state.view_state, ViewState::StoryView);
    assert_eq!(state.selected_story, 0);
}

#[test]
fn test_selecting_invalid_story_fails() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);

    let result = state.select_story(10);
    assert!(!result);
    assert_eq!(state.view_state, ViewState::StoryList);
}

#[test]
fn test_go_back_from_story_view_to_story_list() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);
    state.select_story(0);

    assert_eq!(state.view_state, ViewState::StoryView);

    state.go_back();
    assert_eq!(state.view_state, ViewState::StoryList);
}

#[test]
fn test_go_back_from_story_list_to_component_list() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);

    assert_eq!(state.view_state, ViewState::StoryList);

    state.go_back();
    assert_eq!(state.view_state, ViewState::ComponentList);
}

#[test]
fn test_go_back_from_component_list_does_nothing() {
    let registry = ShowcaseRegistry::new();
    let mut state = SimpleBrowserState::new(registry);

    assert_eq!(state.view_state, ViewState::ComponentList);

    state.go_back();
    assert_eq!(state.view_state, ViewState::ComponentList);
}

// ============================================================================
// Navigation Flow Tests
// ============================================================================

#[test]
fn test_complete_navigation_flow() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));
    registry.register(Box::new(TestComponent::new("Component2", 5)));

    let mut state = SimpleBrowserState::new(registry);

    // Start at component list
    assert_eq!(state.view_state, ViewState::ComponentList);

    // Select first component
    state.select_component(0);
    assert_eq!(state.view_state, ViewState::StoryList);
    assert_eq!(state.selected_component, 0);

    // Select first story
    state.select_story(0);
    assert_eq!(state.view_state, ViewState::StoryView);
    assert_eq!(state.selected_story, 0);

    // Go back to story list
    state.go_back();
    assert_eq!(state.view_state, ViewState::StoryList);

    // Select different story
    state.select_story(1);
    assert_eq!(state.view_state, ViewState::StoryView);
    assert_eq!(state.selected_story, 1);

    // Go back twice to component list
    state.go_back();
    state.go_back();
    assert_eq!(state.view_state, ViewState::ComponentList);

    // Select different component
    state.select_component(1);
    assert_eq!(state.view_state, ViewState::StoryList);
    assert_eq!(state.selected_component, 1);
}

#[test]
fn test_navigating_multiple_components() {
    let mut registry = ShowcaseRegistry::new();
    for i in 0..5 {
        registry.register(Box::new(TestComponent::new(
            &format!("Component{}", i),
            3,
        )));
    }

    let mut state = SimpleBrowserState::new(registry);

    // Navigate to each component
    for i in 0..5 {
        assert_eq!(state.view_state, ViewState::ComponentList);

        state.select_component(i);
        assert_eq!(state.view_state, ViewState::StoryList);
        assert_eq!(state.selected_component, i);

        state.go_back();
        assert_eq!(state.view_state, ViewState::ComponentList);
    }
}

#[test]
fn test_navigating_multiple_stories_in_component() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 7)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);

    // Navigate to each story
    for i in 0..7 {
        assert_eq!(state.view_state, ViewState::StoryList);

        state.select_story(i);
        assert_eq!(state.view_state, ViewState::StoryView);
        assert_eq!(state.selected_story, i);

        state.go_back();
        assert_eq!(state.view_state, ViewState::StoryList);
    }
}

// ============================================================================
// Story Access Through Navigation Tests
// ============================================================================

#[test]
fn test_accessing_story_metadata_after_navigation() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);
    state.select_story(1);

    // Verify we can access the selected story's data
    let component = state.registry.get(state.selected_component).unwrap();
    let stories = component.stories();
    let selected_story = &stories[state.selected_story];

    assert_eq!(selected_story.name, "Story 2");
}

#[test]
fn test_story_count_matches_navigation_bounds() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 5)));

    let mut state = SimpleBrowserState::new(registry);
    state.select_component(0);

    let component = state.registry.get(state.selected_component).unwrap();
    let story_count = component.stories().len();

    // Should be able to select any story from 0 to story_count-1
    for i in 0..story_count {
        let result = state.select_story(i);
        assert!(result, "Should be able to select story {}", i);
        state.go_back();
    }

    // Should not be able to select story at index story_count
    let result = state.select_story(story_count);
    assert!(!result, "Should not be able to select out-of-bounds story");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_navigation_with_empty_registry() {
    let registry = ShowcaseRegistry::new();
    let mut state = SimpleBrowserState::new(registry);

    // Should not be able to select any component
    let result = state.select_component(0);
    assert!(!result);
    assert_eq!(state.view_state, ViewState::ComponentList);
}

#[test]
fn test_navigation_with_single_component_single_story() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 1)));

    let mut state = SimpleBrowserState::new(registry);

    state.select_component(0);
    assert_eq!(state.view_state, ViewState::StoryList);

    state.select_story(0);
    assert_eq!(state.view_state, ViewState::StoryView);
}

#[test]
fn test_state_resets_when_selecting_new_component() {
    let mut registry = ShowcaseRegistry::new();
    registry.register(Box::new(TestComponent::new("Component1", 3)));
    registry.register(Box::new(TestComponent::new("Component2", 5)));

    let mut state = SimpleBrowserState::new(registry);

    // Select component 1 and story 2
    state.select_component(0);
    state.select_story(2);
    assert_eq!(state.selected_story, 2);

    // Go back to component list
    state.go_back();
    state.go_back();

    // Select component 2 - story should reset to 0
    state.select_component(1);
    assert_eq!(state.selected_story, 0);
}

// ============================================================================
// Integration: Registry + Browser State
// ============================================================================

#[test]
fn test_full_showcase_browser_simulation() {
    // Simulate a full showcase browser with multiple components
    let mut registry = ShowcaseRegistry::new();

    // Register components similar to actual showcase
    registry.register(Box::new(TestComponent::new("SimpleText", 3)));
    registry.register(Box::new(TestComponent::new("CommandPreview", 3)));
    registry.register(Box::new(TestComponent::new("TableSelector", 7)));
    registry.register(Box::new(TestComponent::new("CommandOutputViewer", 7)));

    let mut state = SimpleBrowserState::new(registry);

    // User workflow: Browse SimpleText
    state.select_component(0);
    assert_eq!(state.selected_component, 0);

    let component = state.registry.get(0).unwrap();
    assert_eq!(component.metadata().name, "SimpleText");
    assert_eq!(component.stories().len(), 3);

    state.select_story(0);
    state.go_back();
    state.select_story(1);
    state.go_back();
    state.go_back();

    // User workflow: Browse TableSelector
    state.select_component(2);
    assert_eq!(state.selected_component, 2);

    let component = state.registry.get(2).unwrap();
    assert_eq!(component.metadata().name, "TableSelector");
    assert_eq!(component.stories().len(), 7);

    state.select_story(5);
    assert_eq!(state.selected_story, 5);

    // Navigate back to main list
    state.go_back();
    state.go_back();
    assert_eq!(state.view_state, ViewState::ComponentList);
}

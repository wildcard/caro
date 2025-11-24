/// UI Components Module
///
/// This module contains all reusable UI components for the TUI.
/// Each component implements the `Component` trait for consistent behavior.
///
/// # Architecture
///
/// Components follow a React-like pattern with Props and State:
/// - **Props**: Immutable data passed from parent
/// - **State**: Internal mutable state
/// - **Events**: Handle keyboard/mouse events
/// - **Render**: Draw to terminal using Ratatui widgets
///
/// # Example
///
/// ```rust
/// use cmdai::tui::components::{Component, StatusBarComponent, StatusBarProps};
///
/// let component = StatusBarComponent::new(StatusBarProps {
///     backend_name: "Ollama".to_string(),
///     backend_available: true,
///     shell: ShellType::Bash,
///     safety_level: SafetyLevel::Moderate,
/// });
///
/// // In render loop:
/// component.render(&mut frame, area);
/// ```
use anyhow::Result;
use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

pub mod help_footer;
pub mod repl;
pub mod status_bar;

pub use help_footer::HelpFooterComponent;
pub use repl::ReplComponent;
pub use status_bar::StatusBarComponent;

/// Result of handling an event
#[derive(Debug, Clone)]
pub enum EventResult {
    /// Event was consumed by this component
    Consumed,

    /// Event was ignored, pass to next handler
    Ignored,

    /// Event resulted in an application event
    Event(crate::tui::state::AppEvent),

    /// Request to quit the application
    Quit,
}

impl PartialEq for EventResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventResult::Consumed, EventResult::Consumed) => true,
            (EventResult::Ignored, EventResult::Ignored) => true,
            (EventResult::Quit, EventResult::Quit) => true,
            // For Event variant, only check that both are Event (don't compare inner value)
            (EventResult::Event(_), EventResult::Event(_)) => true,
            _ => false,
        }
    }
}

/// Component trait - all UI components implement this
///
/// This trait defines the interface for all TUI components, providing
/// a consistent pattern for handling events and rendering.
///
/// # Example Implementation
///
/// ```rust
/// pub struct MyComponent {
///     props: MyComponentProps,
///     state: MyComponentState,
/// }
///
/// impl Component for MyComponent {
///     type Props = MyComponentProps;
///     type State = MyComponentState;
///
///     fn new(props: Self::Props) -> Self {
///         Self {
///             props,
///             state: MyComponentState::default(),
///         }
///     }
///
///     fn handle_event(&mut self, event: Event) -> Result<EventResult> {
///         match event {
///             Event::Key(key) => {
///                 // Handle keyboard input
///                 Ok(EventResult::Consumed)
///             }
///             _ => Ok(EventResult::Ignored)
///         }
///     }
///
///     fn render(&self, frame: &mut Frame, area: Rect) {
///         // Render using Ratatui widgets
///     }
/// }
/// ```
pub trait Component {
    /// Props type - immutable data passed from parent
    type Props;

    /// State type - internal mutable state
    type State;

    /// Create a new component instance with the given props
    fn new(props: Self::Props) -> Self;

    /// Handle terminal events (keyboard, mouse, resize, etc.)
    ///
    /// Returns:
    /// - `EventResult::Consumed` - Event was handled
    /// - `EventResult::Ignored` - Pass to next handler
    /// - `EventResult::Event(e)` - Generated an application event
    /// - `EventResult::Quit` - Request to quit
    fn handle_event(&mut self, event: Event) -> Result<EventResult> {
        let _ = event;
        Ok(EventResult::Ignored)
    }

    /// Update component based on application state changes
    ///
    /// Called when the global AppState changes and the component
    /// may need to update its internal state.
    fn update(&mut self, _state: &crate::tui::state::AppState) -> Result<()> {
        Ok(())
    }

    /// Render the component to the terminal
    ///
    /// # Arguments
    /// - `frame`: Ratatui frame for rendering
    /// - `area`: Rectangle defining where to render
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Optional: Get the component's current state (for testing)
    fn state(&self) -> &Self::State
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Example test component for testing the trait
    struct TestComponent {
        props: TestProps,
        state: TestState,
    }

    struct TestProps {
        title: String,
    }

    #[derive(Default)]
    struct TestState {
        counter: usize,
    }

    impl Component for TestComponent {
        type Props = TestProps;
        type State = TestState;

        fn new(props: Self::Props) -> Self {
            Self {
                props,
                state: TestState::default(),
            }
        }

        fn render(&self, _frame: &mut Frame, _area: Rect) {
            // No-op for testing
        }

        fn state(&self) -> &Self::State {
            &self.state
        }
    }

    #[test]
    fn test_component_creation() {
        let component = TestComponent::new(TestProps {
            title: "Test".to_string(),
        });

        assert_eq!(component.props.title, "Test");
        assert_eq!(component.state.counter, 0);
    }

    #[test]
    fn test_default_event_handling() {
        let mut component = TestComponent::new(TestProps {
            title: "Test".to_string(),
        });

        let result = component.handle_event(Event::Resize(80, 24)).unwrap();

        assert_eq!(result, EventResult::Ignored);
    }
}

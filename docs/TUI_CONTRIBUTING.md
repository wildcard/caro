# Contributing to cmdai TUI

Welcome, contributor! This guide will help you understand the TUI architecture and how to add features.

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/wildcard/cmdai
cd cmdai

# Build the project
cargo build

# Run the TUI
cargo run -- --tui
```

### First Contribution Checklist

- [ ] Read the [ADR-001: TUI Architecture](/home/user/cmdai/docs/adr/001-tui-architecture.md)
- [ ] Read the [HLD: TUI High-Level Design](/home/user/cmdai/docs/hld/TUI_HIGH_LEVEL_DESIGN.md)
- [ ] Understand the component pattern (see below)
- [ ] Run tests: `cargo test`
- [ ] Check linting: `cargo clippy`
- [ ] Format code: `cargo fmt`

---

## 📚 Architecture Overview

The TUI follows a **Component-Based Architecture** with **Redux-Inspired State Management**.

### Key Concepts

1. **Component Trait**: All UI elements implement the `Component` trait
2. **AppState**: Single source of truth for all application state
3. **AppEvent**: All state changes happen through events
4. **Side Effects**: Async operations (backend calls, I/O) handled separately

### Directory Structure

```
src/tui/
├── app.rs                    # Main TUI application & event loop
├── components/               # Reusable UI components
│   ├── mod.rs               # Component trait definition
│   ├── status_bar.rs        # Status bar component
│   ├── help_footer.rs       # Help footer component
│   └── repl/                # REPL mode components
│       └── mod.rs
├── state/                    # State management
│   ├── mod.rs               # Module exports
│   ├── app_state.rs         # Central application state
│   ├── repl_state.rs        # REPL-specific state
│   └── events.rs            # Event definitions
├── theme/                    # Colors and styling
│   └── mod.rs
└── utils/                    # Utilities
    └── mod.rs               # Terminal setup/cleanup
```

---

## 🧩 Creating a New Component

### Step 1: Define Props and State

```rust
// src/tui/components/my_component.rs

/// Props passed from parent
#[derive(Debug, Clone)]
pub struct MyComponentProps {
    pub title: String,
    pub items: Vec<String>,
}

/// Internal component state
#[derive(Debug, Clone, Default)]
struct MyComponentState {
    selected_index: usize,
}
```

### Step 2: Implement the Component

```rust
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, List, ListItem}};
use crate::tui::components::{Component, EventResult};
use crate::tui::state::AppState;

pub struct MyComponent {
    props: MyComponentProps,
    state: MyComponentState,
}

impl Component for MyComponent {
    type Props = MyComponentProps;
    type State = MyComponentState;

    fn new(props: Self::Props) -> Self {
        Self {
            props,
            state: MyComponentState::default(),
        }
    }

    fn handle_event(&mut self, event: crossterm::event::Event) -> Result<EventResult> {
        use crossterm::event::{Event, KeyCode};

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if self.state.selected_index > 0 {
                        self.state.selected_index -= 1;
                    }
                    Ok(EventResult::Consumed)
                }
                KeyCode::Down => {
                    if self.state.selected_index < self.props.items.len() - 1 {
                        self.state.selected_index += 1;
                    }
                    Ok(EventResult::Consumed)
                }
                _ => Ok(EventResult::Ignored),
            },
            _ => Ok(EventResult::Ignored),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .props
            .items
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(&self.props.title));

        frame.render_widget(list, area);
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}
```

### Step 3: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let component = MyComponent::new(MyComponentProps {
            title: "Test".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        assert_eq!(component.props.title, "Test");
        assert_eq!(component.state.selected_index, 0);
    }

    #[test]
    fn test_navigation_down() {
        let mut component = MyComponent::new(MyComponentProps {
            title: "Test".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let result = component
            .handle_event(crossterm::event::Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::NONE,
            )))
            .unwrap();

        assert_eq!(result, EventResult::Consumed);
        assert_eq!(component.state.selected_index, 1);
    }
}
```

### Step 4: Register the Component

```rust
// In src/tui/components/mod.rs
pub mod my_component;
pub use my_component::MyComponent;
```

---

## 🔄 Adding a New AppEvent

### Step 1: Define the Event

```rust
// In src/tui/state/events.rs
pub enum AppEvent {
    // ... existing events

    /// User selected an item from a list
    ItemSelected { index: usize },
}
```

### Step 2: Handle the Event

```rust
// In src/tui/state/app_state.rs
impl AppState {
    pub fn handle_event(&mut self, event: AppEvent) -> Result<Vec<SideEffect>> {
        let effects = match event {
            // ... existing handlers

            AppEvent::ItemSelected { index } => {
                // Update state
                self.selected_item = Some(index);

                // Return any side effects needed
                vec![]
            }
        };

        Ok(effects)
    }
}
```

### Step 3: Test the Handler

```rust
#[test]
fn test_item_selected_event() {
    let mut state = AppState::default();

    let effects = state
        .handle_event(AppEvent::ItemSelected { index: 2 })
        .unwrap();

    assert_eq!(state.selected_item, Some(2));
    assert_eq!(effects.len(), 0);
}
```

---

## 🎨 Styling Guidelines

### Colors

Use the theme module for consistent colors:

```rust
use crate::tui::theme::Theme;

let theme = Theme::default();

// Primary actions
Style::default().fg(theme.primary)   // Cyan

// Success states
Style::default().fg(theme.success)   // Green

// Warnings
Style::default().fg(theme.warning)   // Yellow

// Errors/Danger
Style::default().fg(theme.danger)    // Red
```

### Risk Level Colors

```rust
use crate::tui::state::events::RiskLevel;

let color = RiskLevel::Safe.color();        // Green
let color = RiskLevel::Moderate.color();    // Yellow
let color = RiskLevel::High.color();        // Red
let color = RiskLevel::Critical.color();    // Red (bold)
```

---

## 🧪 Testing Best Practices

### Unit Tests

Test each component in isolation:

```rust
#[test]
fn test_component_handles_key_event() {
    let mut component = MyComponent::new(/* ... */);

    let event = Event::Key(KeyEvent::from(KeyCode::Enter));
    let result = component.handle_event(event).unwrap();

    assert_eq!(result, EventResult::Consumed);
}
```

### Integration Tests

Test full workflows:

```rust
#[tokio::test]
async fn test_command_generation_flow() {
    let mut app = TuiApp::new().unwrap();

    // Simulate user input
    app.handle_event(AppEvent::TextInput('l')).await.unwrap();
    app.handle_event(AppEvent::TextInput('s')).await.unwrap();
    app.handle_event(AppEvent::Enter).await.unwrap();

    // Wait for async operation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Assert state changed
    assert!(app.state.repl.generated_command.is_some());
}
```

---

## 📝 Documentation Standards

### Component Documentation

Every component must have:

1. **Module-level doc comment** explaining purpose
2. **Example usage** in the component struct doc
3. **Parameter documentation** for all public fields
4. **Method documentation** for public methods

Example:

```rust
/// My Component - Brief description
///
/// Longer description of what this component does and when to use it.
///
/// # Example
///
/// ```rust
/// let component = MyComponent::new(MyComponentProps {
///     title: "Example".to_string(),
/// });
/// ```
pub struct MyComponent {
    props: MyComponentProps,
}
```

---

## 🚦 Code Review Checklist

Before submitting a PR, ensure:

- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt`
- [ ] Documentation is complete
- [ ] Component follows the established pattern
- [ ] Events are properly handled
- [ ] No panics in production code (use `Result`)
- [ ] Performance is acceptable (no blocking operations)

---

## 🐛 Debugging Tips

### Enable Logging

```bash
# Run with verbose logging
RUST_LOG=debug cargo run -- --tui

# Log to file for TUI debugging
RUST_LOG=debug cargo run -- --tui 2> tui_debug.log
```

### Terminal Issues

If the terminal gets messed up:

```bash
# Reset terminal
reset

# Or manually restore
stty sane
```

### Inspecting State

Add temporary debug output:

```rust
// In handle_event
eprintln!("DEBUG: State = {:?}", self.state);
```

---

## 📖 Additional Resources

- [Ratatui Documentation](https://ratatui.rs/)
- [Crossterm Documentation](https://docs.rs/crossterm)
- [ADR-001: TUI Architecture](../adr/001-tui-architecture.md)
- [HLD: TUI Design](../hld/TUI_HIGH_LEVEL_DESIGN.md)

---

## 💬 Getting Help

- **Issues**: Check [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- **Code Questions**: Comment on relevant PRs

---

## 🎯 Good First Issues

Looking for something to work on? Check issues labeled:
- `good-first-issue` - Great for newcomers
- `help-wanted` - Community help needed
- `tui` - TUI-specific tasks

---

**Happy Contributing! 🚀**

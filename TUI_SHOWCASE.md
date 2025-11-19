# TUI Showcase - Storybook for Ratatui

A development tool for building, testing, and showcasing terminal UI components in isolation, similar to React Storybook.

## Features

- **Component Isolation**: Develop and test TUI components independently
- **Interactive Browser**: Navigate through components and their various states/stories
- **Hot Reload**: Fast iteration with `cargo-watch`
- **Story-Based Development**: Showcase different component states and configurations
- **Trait-Based Architecture**: Clean, extensible component interface

## Quick Start

### Run the Showcase

```bash
# Basic run
cargo run --bin tui-showcase

# With hot-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x 'run --bin tui-showcase'
```

### Navigation

- **↑/↓** or **j/k**: Navigate components and stories
- **Enter**: Select component or view story
- **Backspace**: Go back to previous view
- **h**: Toggle help screen
- **q** or **Esc**: Quit application or close help

## Architecture

### Component Trait

All showcase components implement the `ShowcaseComponent` trait:

```rust
pub trait ShowcaseComponent: Send + Sync {
    /// Get component metadata (name, description, category)
    fn metadata(&self) -> ComponentMetadata;

    /// Get all stories for this component
    fn stories(&self) -> Vec<ShowcaseStory>;

    /// Optional: Handle key events for interactive components
    fn handle_key_event(&mut self, _event: crossterm::event::KeyEvent) -> io::Result<bool> {
        Ok(false)
    }
}
```

### Stories

Each component can have multiple **stories** - different variations or states:

```rust
ShowcaseStory::new(
    "Story Name",
    "Description of this variation",
    |frame, area| {
        // Render logic here
        let widget = Paragraph::new("Hello!");
        frame.render_widget(widget, area);
    },
)
```

## Creating New Components

### Step 1: Create Component File

Create a new file in `src/tui/components/`:

```rust
// src/tui/components/my_component.rs
use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}};

pub struct MyComponent;

impl ShowcaseComponent for MyComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "MyComponent",
            "Brief description of what this component does",
        )
        .with_category("Display") // or "Input", "Feedback", etc.
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Default",
                "Default state of the component",
                |frame, area| {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("My Component");
                    let widget = Paragraph::new("Content here").block(block);
                    frame.render_widget(widget, area);
                },
            ),
            ShowcaseStory::new(
                "Variant",
                "Alternative state or configuration",
                |frame, area| {
                    // Different rendering logic
                },
            ),
        ]
    }
}
```

### Step 2: Register Component

1. Add to `src/tui/components/mod.rs`:

```rust
pub mod my_component;
pub use my_component::MyComponent;
```

2. Register in `src/bin/tui_showcase.rs`:

```rust
// In App::new()
registry.register(Box::new(MyComponent));
```

### Step 3: Test in Showcase

Run the showcase and navigate to your new component:

```bash
cargo run --bin tui-showcase
```

## Example Components

The showcase includes several example components demonstrating different patterns:

### 1. SimpleText (`src/tui/components/simple_text.rs`)
- **Category**: Display
- **Stories**: Default, Styled, MultiLine
- **Demonstrates**: Basic text rendering with various styles

### 2. CommandPreview (`src/tui/components/command_preview.rs`)
- **Category**: Display
- **Stories**: Simple Command, Complex Command, With Description
- **Demonstrates**: Syntax highlighting, multi-line content, layout composition

### 3. SafetyIndicator (`src/tui/components/safety_indicator.rs`)
- **Category**: Feedback
- **Stories**: Safe, Moderate, High Risk, Critical
- **Demonstrates**: Color-coded feedback, icons, risk level visualization

### 4. ConfirmationDialog (`src/tui/components/confirmation_dialog.rs`)
- **Category**: Input
- **Stories**: Yes Selected, No Selected, Dangerous Command, Long Message
- **Demonstrates**: Modal dialogs, button states, centered layouts

### 5. ProgressSpinner (`src/tui/components/progress_spinner.rs`)
- **Category**: Feedback
- **Stories**: Multiple animation frames, different contexts
- **Demonstrates**: Animation frames, loading states

## Development Workflow

### 1. Component-First Development

Instead of building components within the full application:

1. Create component file
2. Implement `ShowcaseComponent` trait
3. Add multiple stories showing different states
4. Iterate quickly with hot-reload
5. Once satisfied, integrate into main application

### 2. Hot Reload Setup

For the best development experience:

```bash
# Terminal 1: Run showcase with auto-reload
cargo watch -x 'run --bin tui-showcase'

# Terminal 2: Edit components
vim src/tui/components/my_component.rs
```

Every save triggers automatic rebuild and restart!

### 3. Testing Different States

Create stories for:
- Default state
- Loading state
- Error state
- Empty state
- With data
- Edge cases (very long text, etc.)

## Best Practices

### Component Organization

```rust
// Good: Multiple focused stories
fn stories(&self) -> Vec<ShowcaseStory> {
    vec![
        ShowcaseStory::new("Default", "...", render_default),
        ShowcaseStory::new("Loading", "...", render_loading),
        ShowcaseStory::new("Error", "...", render_error),
    ]
}
```

### Metadata Categories

Use consistent categories:
- **Display**: Components that show information
- **Input**: Components for user input
- **Feedback**: Loading, errors, confirmations
- **Layout**: Layout helpers and containers

### Story Naming

Use descriptive story names:
- ✅ "With Error Message"
- ✅ "Long Content"
- ✅ "Empty State"
- ❌ "Test 1"
- ❌ "Example"

## Integration with Main Application

Once components are developed in the showcase:

```rust
// In your main application
use cmdai::tui::components::SafetyIndicator;

// Use the component's render logic
// (Extract render functions from stories or create shared implementations)
```

## Advanced Features

### Interactive Components

Components can handle key events:

```rust
impl ShowcaseComponent for InteractiveComponent {
    fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
        match event.code {
            KeyCode::Char(' ') => {
                // Handle spacebar
                Ok(true) // Event handled
            }
            _ => Ok(false) // Event not handled
        }
    }
}
```

### Component State

Components can maintain state:

```rust
pub struct StatefulComponent {
    counter: usize,
}

impl ShowcaseComponent for StatefulComponent {
    fn init(&mut self) -> io::Result<()> {
        self.counter = 0;
        Ok(())
    }
}
```

## Troubleshooting

### Build Errors

```bash
# Clean build
cargo clean
cargo build --bin tui-showcase

# Check for missing dependencies
cargo check
```

### Hot Reload Not Working

```bash
# Ensure cargo-watch is installed
cargo install cargo-watch

# Try explicit path
cargo watch -w src -x 'run --bin tui-showcase'
```

### Component Not Appearing

1. Check component is registered in `App::new()`
2. Verify `mod.rs` exports the component
3. Ensure `impl ShowcaseComponent` is correct

## Contributing New Components

When adding components to the showcase:

1. Follow the naming convention: `{ComponentName}Component`
2. Add comprehensive metadata
3. Include at least 3 different stories
4. Document any special features or interactions
5. Use appropriate category

## Future Enhancements

Potential additions:
- [ ] Screenshot/recording mode for documentation
- [ ] Component search/filter
- [ ] Side-by-side story comparison
- [ ] Property/props panel for dynamic configuration
- [ ] Export stories as test cases
- [ ] Performance profiling per component

## License

Same as the main cmdai project (AGPL-3.0).

---

**Happy TUI Development!** 🚀

For questions or issues, see the main [README.md](README.md).

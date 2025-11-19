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

The showcase includes 10 production-ready components with 52+ stories demonstrating different patterns:

### Display Components

#### 1. SimpleText (`src/tui/components/simple_text.rs`)
- **Stories**: 3 (Default, Styled, MultiLine)
- **Demonstrates**: Basic text rendering with various styles

#### 2. CommandPreview (`src/tui/components/command_preview.rs`)
- **Stories**: 3 (Simple Command, Complex Command, With Description)
- **Demonstrates**: Syntax highlighting, multi-line content, layout composition

#### 3. TableSelector (`src/tui/components/table_selector.rs`) ✨
- **Stories**: 7 (Default, Selection states, Highlighting, etc.)
- **Demonstrates**: Interactive tables, row selection, dangerous command highlighting

### Input Components

#### 4. ConfirmationDialog (`src/tui/components/confirmation_dialog.rs`)
- **Stories**: 4 (Yes/No Selected, Dangerous Command, Long Message)
- **Demonstrates**: Modal dialogs, button states, centered layouts

#### 5. CommandEditor (`src/tui/components/command_editor.rs`) ✨
- **Stories**: 7 (Simple/Multi-line, Cursor positions, Syntax highlighting, etc.)
- **Demonstrates**: Multi-line editing, line numbers, syntax highlighting

### Feedback Components

#### 6. SafetyIndicator (`src/tui/components/safety_indicator.rs`)
- **Stories**: 4 (Safe, Moderate, High Risk, Critical)
- **Demonstrates**: Color-coded feedback, icons, risk level visualization

#### 7. ProgressSpinner (`src/tui/components/progress_spinner.rs`)
- **Stories**: 6 (Multiple animation frames, different contexts)
- **Demonstrates**: Animation frames, loading states

#### 8. NotificationToast (`src/tui/components/notification_toast.rs`) ✨
- **Stories**: 8 (Toast positions, Banner styles, All severity levels)
- **Demonstrates**: Temporary notifications, banners, positioning

### Workflow Components

#### 9. CommandFlow (`src/tui/components/command_flow.rs`) ✨
- **Stories**: 6 (Complete workflow from input to execution)
- **Demonstrates**: Multi-step processes, progress tracking, workflow visualization

### Help Components

#### 10. KeyboardShortcuts (`src/tui/components/keyboard_shortcuts.rs`) ✨
- **Stories**: 4 (Compact, Categorized, Detailed, Grid layouts)
- **Demonstrates**: Help screens, keyboard references, multiple layout styles

---

**Total**: 10 components | 52 stories | 5 categories

✨ = New advanced components

For visual previews, see [COMPONENT_GALLERY.md](COMPONENT_GALLERY.md)

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

We welcome high-quality TUI component contributions! Please see [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md) for comprehensive guidelines including:

- Component template and structure
- Styling guidelines and color palette
- Story best practices
- Code quality standards
- Pre-submission checklist
- Advanced techniques and examples

Quick checklist when adding components:

1. Follow the naming convention: `{ComponentName}Component`
2. Add comprehensive metadata with appropriate category
3. Include at least 3-5 different stories
4. Document any special features or interactions
5. Pass `cargo fmt` and `cargo clippy`
6. Test in the showcase with various terminal sizes

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

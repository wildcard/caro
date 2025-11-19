# TUI Showcase Architecture Guide

This guide explains how the TUI Component Showcase system works under the hood. If you're curious about the design decisions, want to extend the framework, or just want to understand the magic - this is for you!

---

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [Core Abstractions](#core-abstractions)
3. [The Component Lifecycle](#the-component-lifecycle)
4. [The Showcase Browser](#the-showcase-browser)
5. [Rendering Pipeline](#rendering-pipeline)
6. [Design Decisions](#design-decisions)
7. [Extending the Framework](#extending-the-framework)

---

## High-Level Overview

### What Problem Does This Solve?

When building terminal UIs (TUIs), developers face several challenges:

1. **Iteration is slow** - You have to run the full app to see a single component
2. **Hard to test variations** - Testing different states requires modifying code
3. **No visual catalog** - No easy way to browse available components
4. **Context matters** - Components look different in different terminal sizes/themes

The TUI Showcase solves this by providing:
- Isolated component rendering (like React Storybook)
- Easy variation testing (through "stories")
- Interactive browsing (keyboard-driven UI)
- Fast iteration (cargo-watch integration)

### System Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     TUI Showcase Browser                     │
│                    (tui_showcase.rs)                         │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │           ShowcaseRegistry                         │    │
│  │  - Stores all registered components                │    │
│  │  - Provides discovery/navigation                   │    │
│  └────────────────────────────────────────────────────┘    │
│                          ▲                                   │
│                          │                                   │
│                          │ registers                         │
│                          │                                   │
│  ┌──────────┬──────────┬┴────────┬──────────┬──────────┐  │
│  │Component │Component │Component │Component │Component │  │
│  │    1     │    2     │    3     │    4     │    5     │  │
│  └────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┘  │
│       │          │          │          │          │         │
│       │          │          │          │          │         │
│  ┌────▼──────────▼──────────▼──────────▼──────────▼─────┐  │
│  │          ShowcaseComponent Trait                     │  │
│  │  - metadata() - Component info                       │  │
│  │  - stories()  - Variations to display               │  │
│  │  - (optional) handle_key_event()                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                   │
│                          │ implements                        │
│                          ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Individual Component Files                │  │
│  │  simple_text.rs, command_preview.rs, etc.           │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Key Files

```
src/
├── tui/
│   ├── showcase.rs           # Framework core (traits + registry)
│   │   ├── ComponentMetadata # Component description
│   │   ├── ShowcaseStory     # Individual variation
│   │   ├── ShowcaseComponent # Trait all components implement
│   │   └── ShowcaseRegistry  # Component collection
│   │
│   └── components/           # Actual components
│       ├── mod.rs           # Component exports
│       ├── simple_text.rs   # Example component
│       └── ...              # More components
│
└── bin/
    └── tui_showcase.rs      # Interactive browser application
```

---

## Core Abstractions

### 1. ComponentMetadata

```rust
pub struct ComponentMetadata {
    pub name: String,         // Display name ("Command Preview")
    pub description: String,  // What it does
    pub category: String,     // "Display", "Input", "Feedback", etc.
    pub version: String,      // Semantic version
}
```

**Purpose**: Provides human-readable information about a component for:
- Display in the browser
- Organization/filtering
- Documentation generation
- Version tracking

**Design Decision**: We chose simple owned strings over `&'static str` to allow runtime-generated metadata if needed.

### 2. ShowcaseStory

```rust
pub struct ShowcaseStory {
    pub name: String,         // Story name ("Error State")
    pub description: String,  // What this variation shows
    pub render: Box<dyn Fn(&mut Frame) + Send + Sync>,  // Rendering function
}
```

**Purpose**: Represents one variation/state of a component.

**Design Decision**:
- `Box<dyn Fn>` allows each story to be a closure capturing different state
- `Send + Sync` enables multi-threading (future-proofing)
- No separate state management - each story is self-contained

**Example**:
```rust
ShowcaseStory {
    name: "Success".to_string(),
    description: "Shows a success message".to_string(),
    render: Box::new(|frame| {
        // This closure has its own state
        let text = Paragraph::new("✓ Success!")
            .style(Style::default().fg(Color::Green));
        frame.render_widget(text, frame.area());
    }),
}
```

### 3. ShowcaseComponent Trait

```rust
pub trait ShowcaseComponent: Send + Sync {
    fn metadata(&self) -> ComponentMetadata;
    fn stories(&self) -> Vec<ShowcaseStory>;

    // Optional hooks
    fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
        Ok(false)
    }
    fn init(&mut self) -> io::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> io::Result<()> { Ok(()) }
}
```

**Purpose**: Unified interface for all components.

**Design Decisions**:
- **Trait object safe**: Can be stored as `Box<dyn ShowcaseComponent>`
- **Send + Sync**: Thread-safe by default
- **Default implementations**: Optional methods don't require implementation
- **Owned return types**: `Vec<ShowcaseStory>` instead of references (easier to work with)

**Why a trait?**
- Enables heterogeneous collections (`Vec<Box<dyn ShowcaseComponent>>`)
- Allows components to have different internal state
- Provides extension points (hooks) without breaking existing components

### 4. ShowcaseRegistry

```rust
pub struct ShowcaseRegistry {
    components: Vec<Box<dyn ShowcaseComponent>>,
}

impl ShowcaseRegistry {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, component: Box<dyn ShowcaseComponent>) { ... }
    pub fn get_component(&self, index: usize) -> Option<&Box<dyn ShowcaseComponent>> { ... }
    pub fn component_count(&self) -> usize { ... }
}
```

**Purpose**: Central repository for all components.

**Design Decisions**:
- Simple `Vec` storage (components rarely change at runtime)
- Index-based access (fast, simple)
- Owned components (`Box<dyn>`) for flexibility

---

## The Component Lifecycle

### 1. Registration (Startup)

```rust
// In tui_showcase.rs
fn new() -> Self {
    let mut registry = ShowcaseRegistry::new();

    // Components register themselves
    registry.register(Box::new(SimpleTextComponent));
    registry.register(Box::new(CommandPreviewComponent));
    // ... more components

    ShowcaseBrowser {
        registry,
        // ... other state
    }
}
```

**What happens**:
1. Create empty registry
2. Instantiate each component
3. Box and register it
4. Store in registry's internal `Vec`

### 2. Discovery (Browsing)

```rust
// User navigates component list
for (index, component) in registry.components.iter().enumerate() {
    let metadata = component.metadata();
    println!("{}: {}", index, metadata.name);
}
```

**What happens**:
1. Iterate through registered components
2. Call `metadata()` on each
3. Display in UI

### 3. Story Selection

```rust
// User selects a component, get its stories
let component = registry.get_component(selected_index)?;
let stories = component.stories();

// Display story list
for (index, story) in stories.iter().enumerate() {
    println!("{}: {} - {}", index, story.name, story.description);
}
```

**What happens**:
1. Get component by index
2. Call `stories()` to get all variations
3. Display story list

### 4. Rendering (Display)

```rust
// User selects a story
let story = &stories[selected_story_index];

// In the render loop
terminal.draw(|frame| {
    (story.render)(frame);  // Call the story's render function
})?;
```

**What happens**:
1. Get the selected story
2. Call its `render` closure with the current frame
3. Ratatui handles actual terminal drawing

### 5. Interaction (Optional)

```rust
// In the event loop
if let Event::Key(key) = event::read()? {
    // Give component a chance to handle it
    if component.handle_key_event(key)? {
        // Component handled it
        continue;
    }
    // Browser handles it
    match key.code {
        KeyCode::Char('q') => break,
        // ...
    }
}
```

**What happens**:
1. Read keyboard input
2. Offer to component first
3. If component returns `true`, it handled it
4. Otherwise, browser handles navigation

---

## The Showcase Browser

### State Machine

The browser operates as a simple state machine:

```
┌──────────────┐
│ ComponentList│ ◄────┐
└──────┬───────┘      │
       │ Enter        │ Backspace
       ▼              │
┌──────────────┐      │
│  StoryList   │ ◄────┤
└──────┬───────┘      │
       │ Enter        │ Backspace
       ▼              │
┌──────────────┐      │
│  StoryView   │──────┘
└──────────────┘
```

**State Definition**:
```rust
enum ViewState {
    ComponentList { selected: usize },
    StoryList { component_index: usize, selected: usize },
    StoryView { component_index: usize, story_index: usize },
}
```

### Navigation Logic

```rust
match self.view_state {
    ViewState::ComponentList { selected } => {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                // Move down, wrap around
                self.view_state = ViewState::ComponentList {
                    selected: (selected + 1) % component_count
                };
            }
            KeyCode::Enter => {
                // Drill into stories
                self.view_state = ViewState::StoryList {
                    component_index: selected,
                    selected: 0,
                };
            }
            // ...
        }
    }
    // ... other states
}
```

### Rendering Strategy

Each view state has its own render method:

```rust
fn render_component_list(&self, frame: &mut Frame) {
    for (index, component) in self.registry.components.iter().enumerate() {
        let metadata = component.metadata();

        let style = if index == selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Render component name with selection indicator
    }
}
```

---

## Rendering Pipeline

### Ratatui Integration

The showcase uses Ratatui for terminal rendering. Here's the flow:

```
1. Create Terminal
   └─► CrosstermBackend wraps stdout
       └─► Terminal<CrosstermBackend> provides high-level API

2. Enter Raw Mode
   └─► Disables line buffering
   └─► Captures all key events
   └─► Hides cursor

3. Render Loop
   ├─► terminal.draw(|frame| {
   │      // Your rendering code here
   │      frame.render_widget(widget, area);
   │   })
   │
   ├─► Read Event (keyboard/mouse/resize)
   ├─► Update State
   └─► Repeat

4. Cleanup
   ├─► Restore terminal (exit raw mode)
   └─► Show cursor
```

### Frame and Areas

```rust
frame.render_widget(widget, area);
```

**Frame**: The current render target
- Represents one frame of animation
- Provides methods to render widgets
- Handles double-buffering

**Area (Rect)**: A rectangular region
```rust
pub struct Rect {
    pub x: u16,      // Column offset
    pub y: u16,      // Row offset
    pub width: u16,  // Columns
    pub height: u16, // Rows
}
```

### Layout System

Ratatui provides a constraint-based layout system:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)  // Stack vertically
    .constraints([
        Constraint::Length(3),       // Fixed 3 rows
        Constraint::Min(10),        // At least 10 rows
        Constraint::Percentage(50), // 50% of remaining
    ])
    .split(area);  // Split the parent area

// chunks[0] is 3 rows tall
// chunks[1] is at least 10 rows
// chunks[2] is 50% of what's left
```

**Common Constraints**:
- `Length(n)`: Exactly n rows/columns
- `Min(n)`: At least n
- `Max(n)`: At most n
- `Percentage(n)`: n% of available space
- `Ratio(num, den)`: Fractional (e.g., 1/3)

---

## Design Decisions

### Why Trait Objects Instead of Enums?

**Option 1: Enum (Not Chosen)**
```rust
enum Component {
    SimpleText(SimpleTextComponent),
    CommandPreview(CommandPreviewComponent),
    // Must add variant for every component!
}
```

❌ Problems:
- Must modify core code for every new component
- Not extensible by users
- Tight coupling

**Option 2: Trait Objects (Chosen)**
```rust
Box<dyn ShowcaseComponent>
```

✅ Benefits:
- Open for extension
- No core code changes needed
- Loose coupling
- Users can add components without touching framework

### Why Owned Strings Instead of &'static str?

**We chose**:
```rust
pub name: String,
```

**Instead of**:
```rust
pub name: &'static str,
```

✅ Benefits:
- Components can generate names dynamically
- No lifetime management complexity
- More flexible for future extensions
- Easier to work with (cloning is cheap for small strings)

❌ Tradeoff:
- Slightly more allocation (negligible for this use case)

### Why Box<dyn Fn> for Render Functions?

**We chose**:
```rust
pub render: Box<dyn Fn(&mut Frame) + Send + Sync>,
```

**Instead of**:
```rust
pub render: fn(&mut Frame),
```

✅ Benefits:
- Closures can capture state
- Each story can have different data
- More expressive (can use `move` semantics)

**Example**:
```rust
let message = "Dynamic message!";
let story = ShowcaseStory {
    render: Box::new(move |frame| {
        // Closure captures 'message'
        let text = Paragraph::new(message);
        frame.render_widget(text, frame.area());
    }),
};
```

### Why No Complex State Management?

Many UI frameworks have complex state management (Redux, MobX, etc.). We intentionally avoided this because:

1. **Stories are static** - They don't change once created
2. **Browser state is simple** - Just tracking selection indices
3. **KISS principle** - Keep it simple
4. **Future-proof** - If needed, components can add their own state

---

## Extending the Framework

### Adding New Component Categories

Edit `src/tui/components/mod.rs`:

```rust
// Display Components
pub mod simple_text;
pub mod command_preview;

// Input Components
pub mod command_editor;

// NEW CATEGORY
// Animation Components
pub mod spinner;
pub mod loading_bar;
```

Then in `tui_showcase.rs`, organize them in the registry:

```rust
// Animation components
registry.register(Box::new(SpinnerComponent));
registry.register(Box::new(LoadingBarComponent));
```

### Adding Component Hooks

Want components to do something on initialization? Implement the hook:

```rust
impl ShowcaseComponent for MyComponent {
    fn init(&mut self) -> io::Result<()> {
        // Called when component is first selected
        println!("Component initialized!");
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        // Called when component is deselected
        println!("Component cleaned up!");
        Ok(())
    }
}
```

### Adding Interactive Stories

Want a story that responds to keyboard input?

```rust
pub struct InteractiveComponent {
    counter: Arc<Mutex<usize>>,
}

impl ShowcaseComponent for InteractiveComponent {
    fn stories(&self) -> Vec<ShowcaseStory> {
        let counter = Arc::clone(&self.counter);

        vec![ShowcaseStory {
            name: "Counter".to_string(),
            description: "Press arrow keys to increment/decrement".to_string(),
            render: Box::new(move |frame| {
                let count = counter.lock().unwrap();
                let text = Paragraph::new(format!("Count: {}", *count));
                frame.render_widget(text, frame.area());
            }),
        }]
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
        match event.code {
            KeyCode::Up => {
                *self.counter.lock().unwrap() += 1;
                Ok(true)  // We handled it
            }
            KeyCode::Down => {
                let mut count = self.counter.lock().unwrap();
                if *count > 0 {
                    *count -= 1;
                }
                Ok(true)
            }
            _ => Ok(false)  // We didn't handle it
        }
    }
}
```

### Adding Animations

Want to animate a component? Use a timer:

```rust
pub struct AnimatedComponent {
    start_time: Arc<Mutex<Instant>>,
}

impl ShowcaseComponent for AnimatedComponent {
    fn stories(&self) -> Vec<ShowcaseStory> {
        let start_time = Arc::clone(&self.start_time);

        vec![ShowcaseStory {
            name: "Spinner".to_string(),
            description: "Animated loading spinner".to_string(),
            render: Box::new(move |frame| {
                let elapsed = start_time.lock().unwrap().elapsed();
                let frame_index = (elapsed.as_millis() / 100) % 8;

                let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                let spinner = Paragraph::new(frames[frame_index as usize]);

                frame.render_widget(spinner, frame.area());
            }),
        }]
    }

    fn init(&mut self) -> io::Result<()> {
        *self.start_time.lock().unwrap() = Instant::now();
        Ok(())
    }
}
```

**Note**: The browser needs to re-render periodically for animations. This requires modifying the event loop to use timeouts instead of blocking reads.

---

## Performance Considerations

### Lazy Evaluation

Components don't create stories until `stories()` is called. This means:
- Fast startup (only creates metadata initially)
- Low memory usage (stories created on demand)

### Rendering Optimization

Ratatui uses double-buffering:
1. Render to back buffer
2. Diff with front buffer
3. Only send changed cells to terminal

This means even complex UIs are fast!

### Terminal Size

Components should adapt to available space:

```rust
let area = frame.area();

if area.width < 40 {
    // Compact view
} else if area.width < 80 {
    // Normal view
} else {
    // Wide view
}
```

---

## Testing Strategy

### Unit Testing Components

Test individual methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let component = SimpleTextComponent;
        let metadata = component.metadata();

        assert_eq!(metadata.name, "Simple Text");
        assert_eq!(metadata.category, "Display");
    }

    #[test]
    fn test_stories() {
        let component = SimpleTextComponent;
        let stories = component.stories();

        assert_eq!(stories.len(), 3);
        assert_eq!(stories[0].name, "Plain Text");
    }
}
```

### Visual Testing (Manual)

1. Run the showcase
2. Navigate to your component
3. Check each story
4. Test in different terminal sizes
5. Test in light/dark themes

### CI/CD Testing

The GitHub Actions workflow automatically:
1. Builds all components
2. Verifies they compile
3. Renders metadata
4. Creates snapshots (future: visual regression testing)

---

## Common Patterns and Anti-Patterns

### ✅ Good Patterns

**1. Self-Contained Stories**
```rust
// Good: Each story is independent
vec![
    ShowcaseStory {
        name: "Success".to_string(),
        render: Box::new(|frame| {
            let text = Paragraph::new("✓ Success");
            // ...
        }),
    },
    ShowcaseStory {
        name: "Error".to_string(),
        render: Box::new(|frame| {
            let text = Paragraph::new("✗ Error");
            // ...
        }),
    },
]
```

**2. Semantic Naming**
```rust
// Good: Clear, descriptive names
ComponentMetadata {
    name: "Command Preview".to_string(),
    description: "Displays shell commands with syntax highlighting".to_string(),
    category: "Display".to_string(),
}
```

**3. Responsive Layout**
```rust
// Good: Adapts to terminal size
let area = frame.area();
let constraints = if area.width < 60 {
    vec![Constraint::Percentage(100)]
} else {
    vec![Constraint::Percentage(50), Constraint::Percentage(50)]
};
```

### ❌ Anti-Patterns

**1. Shared Mutable State Between Stories**
```rust
// Bad: Stories share state (race conditions possible)
let shared_state = Rc::new(RefCell::new(0));

vec![
    ShowcaseStory {
        render: Box::new(|| {
            *shared_state.borrow_mut() += 1;  // Mutation!
        }),
    },
]
```

**2. Hardcoded Dimensions**
```rust
// Bad: Assumes terminal size
let rect = Rect {
    x: 0,
    y: 0,
    width: 80,   // What if terminal is smaller?
    height: 24,
};
```

**3. Side Effects in Metadata**
```rust
// Bad: Metadata should be pure
fn metadata(&self) -> ComponentMetadata {
    println!("Getting metadata!");  // Side effect!
    self.fetch_data_from_network();  // Even worse!
    // ...
}
```

---

## Future Enhancements

Potential additions to the framework:

1. **Search/Filter** - Find components by name or category
2. **Hotkeys Reference** - Press `?` for help overlay
3. **Export to PNG** - Screenshot individual stories
4. **Theme Support** - Light/dark/custom color schemes
5. **Story Parameters** - Pass props to stories
6. **Live Reloading** - Watch files and auto-rebuild
7. **Performance Monitoring** - Render time stats
8. **Accessibility** - Screen reader support
9. **Nested Components** - Components that use other components
10. **Visual Regression Testing** - Automated screenshot comparison

---

## Conclusion

The TUI Showcase is built on these principles:

1. **Simplicity** - Easy to understand and use
2. **Extensibility** - Add components without modifying core
3. **Flexibility** - Components can be simple or complex
4. **Performance** - Lazy evaluation, efficient rendering
5. **Developer Experience** - Fast iteration, clear APIs

The trait-based architecture provides a clean separation between:
- **Framework** (showcase.rs) - Defines the interface
- **Browser** (tui_showcase.rs) - Provides the UI
- **Components** (components/*) - Implement the interface

This makes it easy for contributors to add new components without understanding the entire system!

---

**Questions?** Check out [FAQ.md](FAQ.md) or ask in GitHub issues!

**Ready to extend the framework?** See [EXTENDING.md](EXTENDING.md) (coming soon!)

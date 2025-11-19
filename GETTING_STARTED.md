# Getting Started with TUI Component Development

**Welcome!** 🎉 You're about to contribute to an exciting terminal UI component showcase system. This guide will get you from zero to your first component in **5 minutes**, then help you understand the deeper concepts at your own pace.

> **Don't worry if you're new to Rust or terminal UIs!** This guide assumes no prior knowledge and walks through everything step-by-step.

---

## Table of Contents

1. [5-Minute Quick Start](#5-minute-quick-start) - Get running immediately
2. [Understanding the Basics](#understanding-the-basics) - Core concepts explained simply
3. [Your First Component](#your-first-component) - Step-by-step component creation
4. [Common Patterns](#common-patterns) - Copy-paste examples for common tasks
5. [What to Build](#what-to-build) - Ideas and inspiration
6. [Getting Help](#getting-help) - Resources and support

---

## 5-Minute Quick Start

### Prerequisites

You need:
- Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- A terminal
- A text editor

That's it!

### Step 1: Run the Showcase (1 minute)

```bash
# Clone the repo (if you haven't already)
git clone <repo-url>
cd cmdai

# Build and run the showcase
cargo run --bin tui-showcase
```

You'll see an interactive browser with all components! Use:
- `↑`/`↓` or `j`/`k` to navigate
- `Enter` to select
- `Backspace` to go back
- `q` to quit

**Try it now!** Explore a few components to see what they do.

### Step 2: Copy a Template (2 minutes)

Let's create a simple "Hello World" component:

```bash
# Create a new component file
cp docs/templates/simple_component_template.rs src/tui/components/hello_world.rs
```

Open `src/tui/components/hello_world.rs` in your editor. You'll see a fully commented template. Just change the name and text:

```rust
// Change this:
pub struct SimpleComponentTemplate;

// To this:
pub struct HelloWorldComponent;

// And update the metadata:
fn metadata(&self) -> ComponentMetadata {
    ComponentMetadata {
        name: "Hello World".to_string(),
        description: "My first component!".to_string(),
        category: "Display".to_string(),
        version: "1.0.0".to_string(),
    }
}
```

### Step 3: Register Your Component (2 minutes)

Add your component to two files:

**File 1: `src/tui/components/mod.rs`**
```rust
// Add this line with the other component declarations:
pub mod hello_world;

// And this line with the other pub use statements:
pub use hello_world::HelloWorldComponent;
```

**File 2: `src/bin/tui_showcase.rs`**
```rust
// Find the ShowcaseBrowser::new() function and add:
use cmdai::tui::components::HelloWorldComponent;

// Inside the function, add:
registry.register(Box::new(HelloWorldComponent));
```

### Step 4: See Your Component! (30 seconds)

```bash
cargo run --bin tui-showcase
```

Navigate to your "Hello World" component and see it rendered!

**🎉 Congratulations!** You just created your first TUI component!

---

## Understanding the Basics

### What is a "Story"?

A **story** is one variation or state of a component. Think of it like examples in documentation.

For example, a "Button" component might have stories for:
- Default state
- Hovered state
- Disabled state
- Loading state

Each story shows the component in a different way.

### What is a "Component"?

A **component** is a reusable piece of terminal UI. It could be:
- A text display
- An input field
- A progress bar
- A dialog box
- A table
- Anything you can draw in a terminal!

### The Component Lifecycle

Every component follows this simple flow:

```
1. Component created
2. Stories defined (different variations)
3. User selects a story
4. Story renders to the terminal
5. (Optional) User interacts with it
```

That's it! No complex state management, no event loops to worry about - just rendering.

### File Structure

```
src/
├── tui/
│   ├── components/        # ← Your components go here
│   │   ├── mod.rs        # ← Register component here
│   │   ├── simple_text.rs # ← Example component
│   │   └── your_component.rs  # ← Your new component
│   └── showcase.rs       # ← Framework (you rarely touch this)
└── bin/
    └── tui_showcase.rs   # ← Add component to registry here
```

---

## Your First Component (Detailed)

Let's build a **Status Badge** component that shows system status. We'll go step-by-step with explanations.

### Step 1: Create the File

Create `src/tui/components/status_badge.rs`:

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io;

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};

// This is your component struct - it can hold state if needed
pub struct StatusBadgeComponent;

impl ShowcaseComponent for StatusBadgeComponent {
    // Metadata tells the showcase about your component
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            name: "Status Badge".to_string(),
            description: "Displays system status with color coding".to_string(),
            category: "Display".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    // Stories are the different variations of your component
    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // Story 1: Online status
            ShowcaseStory {
                name: "Online".to_string(),
                description: "System is running normally".to_string(),
                // This function draws your component
                render: Box::new(|frame: &mut Frame| {
                    let area = frame.area();

                    // Create a green badge
                    let badge = Paragraph::new(Line::from(vec![
                        Span::raw("● "),  // Bullet point
                        Span::styled("ONLINE", Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)),
                    ]))
                    .alignment(Alignment::Center)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Green)));

                    frame.render_widget(badge, area);
                }),
            },

            // Story 2: Offline status
            ShowcaseStory {
                name: "Offline".to_string(),
                description: "System is not responding".to_string(),
                render: Box::new(|frame: &mut Frame| {
                    let area = frame.area();

                    // Create a red badge
                    let badge = Paragraph::new(Line::from(vec![
                        Span::raw("● "),
                        Span::styled("OFFLINE", Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD)),
                    ]))
                    .alignment(Alignment::Center)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Red)));

                    frame.render_widget(badge, area);
                }),
            },

            // Story 3: Warning status
            ShowcaseStory {
                name: "Degraded".to_string(),
                description: "System is running with issues".to_string(),
                render: Box::new(|frame: &mut Frame| {
                    let area = frame.area();

                    // Create a yellow badge
                    let badge = Paragraph::new(Line::from(vec![
                        Span::raw("⚠ "),  // Warning symbol
                        Span::styled("DEGRADED", Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)),
                    ]))
                    .alignment(Alignment::Center)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow)));

                    frame.render_widget(badge, area);
                }),
            },
        ]
    }
}
```

### Step 2: Understanding the Code

Let's break down what each part does:

**The Imports**
```rust
use ratatui::{...};  // Terminal UI framework
use std::io;         // Standard I/O (required by trait)
use crate::tui::showcase::{...};  // Our showcase framework
```

**The Component Struct**
```rust
pub struct StatusBadgeComponent;
```
This is your component. It's empty for now because it doesn't need state. If your component needs to remember things (like user input), you'd add fields here.

**The Metadata**
```rust
fn metadata(&self) -> ComponentMetadata { ... }
```
This tells the showcase:
- What your component is called ("Status Badge")
- What it does ("Displays system status...")
- Where it fits ("Display" category)
- Version number

**The Stories**
```rust
fn stories(&self) -> Vec<ShowcaseStory> { ... }
```
Each story is a variation. We have three:
1. Online (green)
2. Offline (red)
3. Degraded (yellow)

**The Render Function**
```rust
render: Box::new(|frame: &mut Frame| { ... })
```
This is where you draw! The `frame` is like a canvas. You:
1. Get the available area
2. Create widgets (text, boxes, etc.)
3. Render them to the frame

### Step 3: Register It

**In `src/tui/components/mod.rs`:**
```rust
pub mod status_badge;
pub use status_badge::StatusBadgeComponent;
```

**In `src/bin/tui_showcase.rs`:**
```rust
use cmdai::tui::components::StatusBadgeComponent;

// Inside ShowcaseBrowser::new():
registry.register(Box::new(StatusBadgeComponent));
```

### Step 4: Run It!

```bash
cargo run --bin tui-showcase
```

Find "Status Badge" in the list and try all three stories!

---

## Common Patterns

### Pattern 1: Color-Coded Text

```rust
// Green text for success
Span::styled("Success!", Style::default().fg(Color::Green))

// Red text for errors
Span::styled("Error!", Style::default().fg(Color::Red))

// Bold yellow warning
Span::styled("Warning!", Style::default()
    .fg(Color::Yellow)
    .add_modifier(Modifier::BOLD))
```

### Pattern 2: Bordered Box

```rust
let block = Block::default()
    .borders(Borders::ALL)
    .title("My Box")
    .style(Style::default().fg(Color::Cyan));

let paragraph = Paragraph::new("Content goes here")
    .block(block);

frame.render_widget(paragraph, area);
```

### Pattern 3: Multiple Lines of Text

```rust
let lines = vec![
    Line::from("First line"),
    Line::from("Second line"),
    Line::from(vec![
        Span::raw("Mixed "),
        Span::styled("styled", Style::default().fg(Color::Blue)),
        Span::raw(" text"),
    ]),
];

let paragraph = Paragraph::new(lines);
frame.render_widget(paragraph, area);
```

### Pattern 4: Centered Content

```rust
let text = Paragraph::new("Centered!")
    .alignment(Alignment::Center);

frame.render_widget(text, area);
```

### Pattern 5: Layout with Multiple Sections

```rust
// Split screen into top and bottom
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(50),  // Top half
        Constraint::Percentage(50),  // Bottom half
    ])
    .split(area);

// Render different things in each section
frame.render_widget(top_widget, chunks[0]);
frame.render_widget(bottom_widget, chunks[1]);
```

### Pattern 6: Table/List

```rust
let items = vec![
    Line::from("  Item 1"),
    Line::from("  Item 2"),
    Line::from("→ Item 3 (selected)"),
    Line::from("  Item 4"),
];

let list = Paragraph::new(items)
    .block(Block::default().borders(Borders::ALL).title("List"));

frame.render_widget(list, area);
```

---

## What to Build

### Easy Components (Great for beginners!)

1. **Loading Dots** - Animated "..." loading indicator
2. **Percentage Bar** - Show progress as a percentage
3. **Tag Cloud** - Display multiple colored tags
4. **Clock Display** - Show current time
5. **Weather Icon** - ASCII weather symbols
6. **File Icon** - Different icons for file types
7. **User Avatar** - ASCII art profile display

### Medium Components

1. **Mini Chart** - Simple bar chart
2. **Log Viewer** - Scrollable log entries
3. **Form Field** - Input field with label
4. **Menu Bar** - Horizontal menu with options
5. **Card Layout** - Information card with header/footer
6. **Badge Collection** - Multiple status badges

### Advanced Components (Ready for a challenge?)

1. **Diff Viewer** - Side-by-side code comparison
2. **Tree View** - Hierarchical file tree
3. **Gantt Chart** - Timeline visualization
4. **Network Graph** - Node and edge visualization
5. **Terminal Emulator** - Nested terminal display
6. **Split Pane** - Resizable split view

### Community Requested

Check the GitHub issues for components that people are asking for! Look for tags like:
- `component-request`
- `good-first-issue`
- `help-wanted`

---

## Development Workflow

### Fast Iteration with cargo-watch

Install cargo-watch for automatic rebuilding:

```bash
cargo install cargo-watch
```

Then run:

```bash
cargo watch -x 'run --bin tui-showcase'
```

Every time you save a file, it automatically rebuilds and restarts!

### Testing Your Component

1. **Visual Testing**: Just run the showcase and look at it
2. **Different Terminal Sizes**: Resize your terminal to test responsive behavior
3. **Different Themes**: Test with light and dark terminal themes
4. **Edge Cases**: Try empty text, very long text, unicode characters

### Debugging Tips

**Problem: Component doesn't show up**
- Check you added it to `mod.rs`
- Check you registered it in `tui_showcase.rs`
- Check for compile errors with `cargo check`

**Problem: Text is cut off**
- Components must fit in the available area
- Use `Layout` to manage space
- Test with small terminal sizes

**Problem: Colors look wrong**
- Some terminals don't support all colors
- Stick to basic colors for compatibility: Red, Green, Yellow, Blue, Magenta, Cyan, White, Black

**Problem: Compilation errors**
- Read the error message carefully
- Check the examples in `src/tui/components/`
- Ask for help in GitHub issues!

---

## Getting Help

### Resources

1. **Example Components**: Look at `src/tui/components/simple_text.rs` for the simplest example
2. **Documentation**:
   - [TUI_SHOWCASE.md](TUI_SHOWCASE.md) - Complete system documentation
   - [COMPONENT_GALLERY.md](COMPONENT_GALLERY.md) - Visual reference of all components
   - [CONTRIBUTING_TUI.md](CONTRIBUTING_TUI.md) - Detailed contribution guidelines
3. **Ratatui Docs**: https://docs.rs/ratatui/ - The underlying framework
4. **Ratatui Examples**: https://github.com/ratatui-org/ratatui/tree/main/examples

### Getting Support

- **GitHub Issues**: Ask questions with the `question` label
- **GitHub Discussions**: General discussion and ideas
- **Pull Request Reviews**: Submit a draft PR and ask for feedback!

### Don't Be Afraid to Ask!

**Common beginner questions are totally welcome:**
- "How do I center text?"
- "Why is my component not showing?"
- "Can someone review my first component?"
- "Is this the right approach?"

Everyone started as a beginner. The community is here to help! 💚

---

## Next Steps

1. ✅ Run the showcase (`cargo run --bin tui-showcase`)
2. ✅ Explore existing components
3. ✅ Copy a template and make a simple component
4. ✅ Read [COMPONENT_GALLERY.md](COMPONENT_GALLERY.md) for inspiration
5. ✅ Check out [ARCHITECTURE_GUIDE.md](ARCHITECTURE_GUIDE.md) to understand the internals
6. ✅ Join the community and start contributing!

---

## Quick Reference Card

```
┌─────────────────────────────────────────┐
│ TUI Component Quick Reference           │
├─────────────────────────────────────────┤
│ CREATE COMPONENT:                       │
│   1. Create file in src/tui/components/│
│   2. Add to mod.rs                      │
│   3. Register in tui_showcase.rs        │
│                                         │
│ RUN SHOWCASE:                           │
│   cargo run --bin tui-showcase          │
│                                         │
│ AUTO-REBUILD:                           │
│   cargo watch -x 'run --bin tui-showcase'│
│                                         │
│ COLORS:                                 │
│   Color::Green  (success)               │
│   Color::Red    (error)                 │
│   Color::Yellow (warning)               │
│   Color::Cyan   (info)                  │
│                                         │
│ BORDERS:                                │
│   Block::default()                      │
│     .borders(Borders::ALL)              │
│                                         │
│ CENTERED TEXT:                          │
│   .alignment(Alignment::Center)         │
│                                         │
│ BOLD TEXT:                              │
│   .add_modifier(Modifier::BOLD)         │
└─────────────────────────────────────────┘
```

---

**Ready to contribute?** Pick one of the "Easy Components" above and give it a try! We can't wait to see what you create! 🚀

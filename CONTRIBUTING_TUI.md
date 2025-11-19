# Contributing TUI Components

Thank you for your interest in contributing to the cmdai TUI Component Showcase! This guide will help you create high-quality, production-ready terminal UI components.

## 🎯 What Makes a Great Component?

A great showcase component should:

✅ **Demonstrate a specific pattern** - Focus on one interaction or display pattern
✅ **Include multiple stories** - Show at least 3-5 different states or variations
✅ **Be self-contained** - Work independently without external dependencies
✅ **Follow conventions** - Use consistent styling and naming
✅ **Be well-documented** - Include clear descriptions and comments

## 🚀 Quick Start Guide

### Step 1: Create Component File

Create a new file in `src/tui/components/`:

```bash
touch src/tui/components/my_component.rs
```

### Step 2: Component Template

Use this template as a starting point:

```rust
//! Brief description of what this component demonstrates
//!
//! Longer description explaining the use case, patterns demonstrated,
//! and any special considerations.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct MyComponent;

impl ShowcaseComponent for MyComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "MyComponent",  // Name shown in showcase
            "Brief one-line description of what it does",
        )
        .with_category("Display")  // Display, Input, Feedback, Workflow, or Help
        .with_version("1.0.0")     // Semantic versioning
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Default",  // Story name
                "Description of this variation",
                |frame, area| {
                    // Render logic here
                    let text = Line::from("Hello, TUI!");
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("My Component");
                    let widget = Paragraph::new(text).block(block);
                    frame.render_widget(widget, area);
                },
            ),
            // Add more stories...
        ]
    }
}
```

### Step 3: Register Component

Add to `src/tui/components/mod.rs`:

```rust
pub mod my_component;
pub use my_component::MyComponent;
```

Add to `src/bin/tui_showcase.rs` in the `App::new()` method:

```rust
// In the appropriate category section
registry.register(Box::new(MyComponent));
```

### Step 4: Test in Showcase

```bash
cargo run --bin tui-showcase
```

Navigate to your component and verify all stories render correctly!

## 📚 Component Categories

Choose the most appropriate category for your component:

### Display
Components that show information to the user
- **Examples**: Text, tables, lists, command previews
- **Focus**: Clear information hierarchy, readable formatting

### Input
Components that accept user input
- **Examples**: Forms, dialogs, editors, selection lists
- **Focus**: Keyboard navigation, validation, feedback

### Feedback
Components that provide status or progress information
- **Examples**: Spinners, progress bars, notifications, indicators
- **Focus**: Visual clarity, non-intrusive updates

### Workflow
Components that show multi-step processes
- **Examples**: Wizards, flows, step indicators
- **Focus**: Progress clarity, state management

### Help
Components that provide guidance and documentation
- **Examples**: Help screens, shortcuts, tutorials
- **Focus**: Accessibility, comprehensive information

## 🎨 Styling Guidelines

### Color Palette

Use semantic colors for consistency:

```rust
// Status colors
Color::Green   // Success, safe, positive
Color::Yellow  // Warning, caution, moderate
Color::Red     // Error, danger, critical
Color::Cyan    // Info, highlights, headers
Color::Blue    // Links, paths, secondary info
Color::Magenta // Keywords, special emphasis
Color::White   // Default text
Color::DarkGray// Disabled, secondary text
Color::Black   // Backgrounds (when needed)
```

### Typography

```rust
// Headers and titles
Style::default()
    .fg(Color::Cyan)
    .add_modifier(Modifier::BOLD)

// Selected/focused items
Style::default()
    .fg(Color::Black)
    .bg(Color::Cyan)
    .add_modifier(Modifier::BOLD)

// Disabled/secondary
Style::default()
    .fg(Color::DarkGray)

// Error states
Style::default()
    .fg(Color::Red)
    .add_modifier(Modifier::BOLD)
```

### Layout Principles

```rust
// Use Layout for complex arrangements
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),    // Fixed height header
        Constraint::Min(5),       // Flexible content area
        Constraint::Length(3),    // Fixed height footer
    ])
    .split(area);

// Add margins for readability
let content_area = Rect {
    x: area.x + 2,
    y: area.y + 1,
    width: area.width.saturating_sub(4),
    height: area.height.saturating_sub(2),
};
```

## 📖 Story Best Practices

### Story Naming

Use descriptive, action-oriented names:

```rust
// ✅ Good
"Default State"
"With Error Message"
"Loading with Progress"
"Empty List"
"Selected Item"

// ❌ Avoid
"Test 1"
"Example"
"Demo"
"Variant A"
```

### Story Descriptions

Explain what the story demonstrates:

```rust
ShowcaseStory::new(
    "Error State",  // ✅
    "Shows how errors are displayed with red highlighting and icon",  // ✅
    render_fn,
)

// vs

ShowcaseStory::new(
    "Error",  // ❌ Too brief
    "Error example",  // ❌ Not descriptive
    render_fn,
)
```

### Recommended Story Types

For comprehensive coverage, include:

1. **Default** - Standard/normal state
2. **Empty** - No data/content
3. **Loading** - Async operation in progress
4. **Error** - Error/failure state
5. **Success** - Successful completion
6. **Selected** - Item/row selection
7. **Disabled** - Inactive state
8. **With Data** - Populated with example data
9. **Edge Cases** - Long text, special characters, etc.

## 🔧 Advanced Techniques

### Helper Functions

Extract common rendering logic:

```rust
fn render_header(title: &str, color: Color) -> Paragraph {
    Paragraph::new(title)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
}

impl ShowcaseComponent for MyComponent {
    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new("Info", "Info variant", |frame, area| {
                let header = render_header("Info Message", Color::Cyan);
                frame.render_widget(header, area);
            }),
            ShowcaseStory::new("Error", "Error variant", |frame, area| {
                let header = render_header("Error Message", Color::Red);
                frame.render_widget(header, area);
            }),
        ]
    }
}
```

### State Management

For interactive components:

```rust
pub struct InteractiveComponent {
    selected: usize,
    items: Vec<String>,
}

impl ShowcaseComponent for InteractiveComponent {
    fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
        match event.code {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Ok(true)  // Event handled
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.items.len() - 1);
                Ok(true)
            }
            _ => Ok(false)  // Event not handled
        }
    }

    fn init(&mut self) -> io::Result<()> {
        self.items = vec!["Item 1".into(), "Item 2".into()];
        self.selected = 0;
        Ok(())
    }
}
```

### Responsive Layouts

Handle different terminal sizes:

```rust
fn render_responsive(frame: &mut Frame, area: Rect, content: &str) {
    let (direction, constraints) = if area.width > 80 {
        // Wide terminal: horizontal layout
        (Direction::Horizontal, vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
    } else {
        // Narrow terminal: vertical layout
        (Direction::Vertical, vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
    };

    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area);

    // Render in chunks...
}
```

## ✅ Pre-Submission Checklist

Before submitting your component, verify:

- [ ] Component compiles without warnings
- [ ] At least 3 stories are included
- [ ] All stories render correctly in different terminal sizes
- [ ] Component is added to `mod.rs`
- [ ] Component is registered in `tui_showcase.rs`
- [ ] Code follows Rust formatting (`cargo fmt`)
- [ ] Code passes clippy lints (`cargo clippy`)
- [ ] Component metadata is complete and accurate
- [ ] Documentation comments are clear and helpful
- [ ] Colors and styling are consistent with other components

## 🎯 Code Quality Standards

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix
```

### Testing

```bash
# Run all tests
cargo test

# Test showcase builds
cargo build --bin tui-showcase

# Test with watch mode
cargo watch -x 'build --bin tui-showcase'
```

## 📝 Documentation Standards

### File Header

```rust
//! Component name and brief description
//!
//! ## Purpose
//! Explain what problem this component solves or pattern it demonstrates.
//!
//! ## Usage
//! Brief example of when/how to use this component.
//!
//! ## Stories
//! List the included stories and what each demonstrates.
```

### Inline Comments

```rust
// Good: Explain WHY, not WHAT
let selected_style = if is_selected {
    // Highlight selected row for keyboard navigation clarity
    Style::default().bg(Color::Cyan)
} else {
    Style::default()
};

// Avoid: Stating the obvious
let x = 5; // Set x to 5
```

## 🌟 Advanced Component Examples

### Multi-Column Layout

```rust
fn render_multi_column(frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    for (idx, chunk) in columns.iter().enumerate() {
        let content = format!("Column {}", idx + 1);
        let widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(widget, *chunk);
    }
}
```

### Centered Dialog

```rust
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### Syntax Highlighting

```rust
fn highlight_text(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let keywords = ["if", "else", "for", "while", "fn"];

    for word in text.split_whitespace() {
        let style = if keywords.contains(&word) {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else if word.starts_with('"') {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(format!("{} ", word), style));
    }

    spans
}
```

## 🤝 Getting Help

- **Question about component design?** Open a discussion on GitHub
- **Found a bug?** Open an issue with reproduction steps
- **Want feedback?** Create a draft PR and ask for review
- **Need examples?** Check existing components in `src/tui/components/`

## 📚 Resources

- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui Book](https://ratatui-book.netlify.app/)
- [TUI_SHOWCASE.md](TUI_SHOWCASE.md) - User guide
- [COMPONENT_GALLERY.md](COMPONENT_GALLERY.md) - Visual reference

## 🎉 Recognition

Quality contributions will be:
- Featured in the component gallery
- Listed in the project README
- Credited in release notes
- Highlighted in the showcase

Thank you for contributing to the cmdai TUI Component Showcase! 🚀

---

**Questions?** Open an issue or discussion on GitHub!

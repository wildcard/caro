# TUI Component Showcase - Frequently Asked Questions

Quick answers to common questions from new contributors.

---

## Getting Started

### Q: I'm completely new to Rust. Can I still contribute?

**A: Absolutely yes!**

Start with:
1. Read [GETTING_STARTED.md](GETTING_STARTED.md) - written for beginners
2. Copy an existing simple component (like `simple_text.rs`)
3. Make small changes
4. Ask questions in GitHub issues!

Many contributors learn Rust by working on this project. The community is very welcoming to beginners.

### Q: I don't understand terminal UIs. Where do I start?

**A: Start simple!**

1. Run the showcase: `cargo run --bin tui-showcase`
2. Look at existing components
3. Copy `simple_text.rs` - it's only ~60 lines
4. Change the text and colors
5. See it render!

You don't need to understand everything. Learn by doing!

### Q: What's the easiest component I can build?

**A: A simple display component!**

Try these beginner-friendly ideas:
- **Quote Display**: Show a random quote
- **ASCII Art**: Display ASCII art
- **Color Palette**: Show all available colors
- **Box Styles**: Display different border styles
- **Icon Set**: Show symbols and emojis

These require no complex logic - just rendering!

---

## Technical Questions

### Q: What's the difference between a "component" and a "story"?

**A: Think of it like a photo album:**

- **Component** = The album (e.g., "Vacation Photos")
- **Story** = Individual photos in the album (e.g., "Beach", "Mountains", "City")

Each component can have multiple stories showing different variations or states.

**Example**:
```
Component: Button
├─ Story 1: Normal State
├─ Story 2: Hovered
├─ Story 3: Pressed
└─ Story 4: Disabled
```

### Q: Do I need to understand traits and Box<dyn Trait>?

**A: Not really!**

Just follow the pattern:
```rust
pub struct MyComponent;

impl ShowcaseComponent for MyComponent {
    fn metadata(&self) -> ComponentMetadata { ... }
    fn stories(&self) -> Vec<ShowcaseStory> { ... }
}
```

Copy this structure from any existing component. It works even if you don't understand every detail!

**Want to learn more?** Check out the [ARCHITECTURE_GUIDE.md](ARCHITECTURE_GUIDE.md) when you're ready.

### Q: What does `Frame` do?

**A: It's your drawing canvas!**

Think of it like this:
```rust
render: Box::new(|frame| {
    // frame = your canvas
    // frame.area() = how much space you have
    // frame.render_widget() = draw something
})
```

The `frame` is where you draw your component. That's it!

### Q: What's `Rect` and why do I need it?

**A: It's a rectangle describing where to draw.**

```rust
pub struct Rect {
    pub x: u16,      // Start column
    pub y: u16,      // Start row
    pub width: u16,  // How many columns
    pub height: u16, // How many rows
}
```

**Example**:
```rust
let rect = Rect {
    x: 0,       // Start at left edge
    y: 0,       // Start at top
    width: 20,  // 20 columns wide
    height: 5,  // 5 rows tall
};
```

Usually you just use `frame.area()` which gives you all available space!

### Q: How do I center text?

**A: Use `Alignment::Center`:**

```rust
use ratatui::layout::Alignment;

let text = Paragraph::new("Centered!")
    .alignment(Alignment::Center);
```

### Q: How do I change text color?

**A: Use `Style` and `fg()` (foreground color):**

```rust
use ratatui::style::{Color, Style};
use ratatui::text::Span;

let colored = Span::styled(
    "Red text",
    Style::default().fg(Color::Red)
);
```

**Available colors**:
- `Color::Red`, `Color::Green`, `Color::Yellow`
- `Color::Blue`, `Color::Magenta`, `Color::Cyan`
- `Color::White`, `Color::Black`, `Color::Gray`

### Q: How do I make text bold?

**A: Use `add_modifier(Modifier::BOLD)`:**

```rust
use ratatui::style::Modifier;

let bold = Span::styled(
    "Bold text",
    Style::default().add_modifier(Modifier::BOLD)
);
```

**Other modifiers**:
- `Modifier::ITALIC`
- `Modifier::UNDERLINED`
- `Modifier::DIM`

### Q: How do I draw a box around something?

**A: Use `Block::default().borders(Borders::ALL)`:**

```rust
use ratatui::widgets::{Block, Borders};

let block = Block::default()
    .borders(Borders::ALL)
    .title("My Box");

let paragraph = Paragraph::new("Content")
    .block(block);
```

---

## Workflow Questions

### Q: How do I test my component?

**A: Run the showcase:**

```bash
cargo run --bin tui-showcase
```

Then navigate to your component and check each story!

### Q: Do I need to write tests?

**A: Visual testing is enough for now!**

For components, seeing them render correctly IS the test. The automated workflow will:
- Build your component
- Display it in logs
- Create snapshots

Traditional unit tests are optional for showcase components.

### Q: How do I make the showcase auto-reload when I save?

**A: Use `cargo-watch`:**

```bash
# Install once
cargo install cargo-watch

# Run with auto-reload
cargo watch -x 'run --bin tui-showcase'
```

Now every time you save a file, it rebuilds and reruns automatically!

### Q: My component doesn't show up in the showcase. What's wrong?

**A: Check these 3 things:**

1. **Did you add it to `mod.rs`?**
   ```rust
   pub mod my_component;
   pub use my_component::MyComponent;
   ```

2. **Did you register it in `tui_showcase.rs`?**
   ```rust
   use cmdai::tui::components::MyComponent;
   // ...
   registry.register(Box::new(MyComponent));
   ```

3. **Does it compile?**
   ```bash
   cargo check
   ```

If all three are ✓, it should appear!

---

## GitHub Actions / CI/CD Questions

### Q: What is "GitHub Actions" and do I need to understand it?

**A: It's automatic quality checking. You don't need to understand it!**

Just know:
- It runs automatically when you push code
- It checks your code compiles
- It creates visual previews
- Green ✓ = good, Red ✗ = something needs fixing

That's all you need to know! See [CI_CD_EXPLAINED.md](CI_CD_EXPLAINED.md) if you're curious.

### Q: The CI check failed. What do I do?

**A: Click on it and read the error message!**

Steps:
1. Go to your PR on GitHub
2. Scroll down to "Checks"
3. Click the red ✗
4. Read the error message (usually highlighted in red)
5. Fix the issue in your code
6. Push again

Common issues:
- **Clippy warning**: Run `cargo clippy` locally and fix
- **Format issue**: Run `cargo fmt`
- **Build error**: Run `cargo build` and fix compilation errors

### Q: What are "artifacts" and how do I download them?

**A: They're files the workflow creates (like screenshots).**

To download:
1. Go to Actions tab
2. Click on your workflow run
3. Scroll to bottom
4. Look for "Artifacts" section
5. Click download

You'll get:
- Component snapshots (images)
- Animated GIFs of components

### Q: Can I run the GitHub Actions workflow locally?

**A: Kind of - you can run the same commands:**

```bash
# What the workflow does:
cargo build --release --bin tui-showcase
cargo clippy -- -D warnings
cargo fmt --check
```

For true local Actions, use [act](https://github.com/nektos/act).

---

## Common Errors

### Q: I get "cannot find value `component` in this scope"

**A: You forgot to define `component`!**

Probably you meant to use `self`:

```rust
// Wrong:
component.metadata()

// Right:
self.metadata()
```

Or you need to create it:
```rust
let component = MyComponent;
```

### Q: I get "the trait bound ... is not satisfied"

**A: This usually means a type mismatch.**

**Common cause**: Using `&str` where `String` is expected:

```rust
// Wrong:
name: "My Component"

// Right:
name: "My Component".to_string()
```

**Pro tip**: If you see `expected String, found &str`, add `.to_string()`.

### Q: I get "cannot borrow as mutable"

**A: You're trying to modify something that's immutable.**

Add `mut`:

```rust
// Wrong:
let frame = ...;

// Right:
let mut frame = ...;
```

Or if it's in a function parameter:
```rust
fn render(frame: &mut Frame) { ... }
```

### Q: Text is cut off or doesn't fit

**A: The terminal is too small!**

Solutions:
1. **Make terminal bigger** (resize the window)
2. **Use responsive layout**:
   ```rust
   if frame.area().width < 60 {
       // Compact view
   } else {
       // Full view
   }
   ```
3. **Use scrolling** (see advanced components like `command_output_viewer.rs`)

### Q: Colors don't look right

**A: Some terminals have limited color support.**

Stick to basic colors for maximum compatibility:
- ✓ `Color::Red`, `Color::Green`, `Color::Yellow`
- ✓ `Color::Blue`, `Color::Magenta`, `Color::Cyan`
- ⚠ `Color::Rgb(r, g, b)` - not all terminals support this

### Q: Cargo build is very slow

**A: First build is always slow. Subsequent builds are fast!**

Speed up builds:
```bash
# Use incremental compilation (usually default)
export CARGO_INCREMENTAL=1

# Use all CPU cores
cargo build -j8

# Release builds are slower but optimized
cargo build          # Fast but debug
cargo build --release # Slow but optimized
```

**Pro tip**: Use `cargo check` for quick syntax checking without building:
```bash
cargo check  # Much faster than build!
```

---

## Design Questions

### Q: What components are needed?

**A: Check GitHub issues tagged `component-request`!**

Also popular requests:
- Interactive components (forms, inputs)
- Charts and graphs
- Diff viewers
- Code syntax highlighting
- Network diagrams
- File browsers

### Q: Should I make one big component or many small ones?

**A: Many small ones!**

**Good**:
- `LoadingSpinner` component
- `ProgressBar` component
- `StatusIcon` component

**Less good**:
- `LoadingAndProgressAndStatus` component

Small, focused components are easier to:
- Understand
- Test
- Reuse
- Maintain

### Q: How many stories should a component have?

**A: As many as there are interesting variations!**

**Minimum**: 1 story (the default state)

**Typical**: 3-5 stories
- Default
- Alternative states (error, loading, etc.)
- Edge cases (empty, very long text, etc.)

**Maximum**: No limit, but keep it reasonable!

### Q: What should I name my component?

**A: Be descriptive and specific!**

**Good names**:
- `CommandPreviewComponent`
- `SafetyIndicatorComponent`
- `ProgressSpinnerComponent`

**Less good names**:
- `Component1`
- `MyComponent`
- `TUI` (too generic)

**Convention**: Ends with `Component`

---

## Contributing Questions

### Q: How do I submit my component?

**A: Standard GitHub workflow:**

1. Fork the repository
2. Create a branch: `git checkout -b my-awesome-component`
3. Add your component
4. Test it: `cargo run --bin tui-showcase`
5. Commit: `git commit -m "Add awesome component"`
6. Push: `git push origin my-awesome-component`
7. Create a Pull Request on GitHub

**First time?** See GitHub's [fork and pull request guide](https://docs.github.com/en/pull-requests).

### Q: What should I include in my pull request?

**A: Just the component code!**

Required:
- The component file (e.g., `src/tui/components/my_component.rs`)
- Update `src/tui/components/mod.rs`
- Update `src/bin/tui_showcase.rs`

Optional but nice:
- Screenshot or GIF of your component
- Description of what it does

**Don't worry about**:
- Documentation (maintainers will help)
- Updating README (maintainers will do it)
- Changelog (automated)

### Q: Will someone review my code?

**A: Yes! Reviews are friendly and educational.**

Expect feedback on:
- Code style (formatting, conventions)
- Rust best practices
- Component design
- Story coverage

**Don't be intimidated!** Reviews are to help you improve, not criticize.

### Q: How long until my PR is merged?

**A: Usually within a few days to a week.**

Depends on:
- Maintainer availability
- How much feedback is needed
- Whether tests pass

**Be patient** and respond to feedback promptly!

### Q: Can I work on multiple components at once?

**A: Yes, but submit separate PRs!**

**Good approach**:
```
PR #1: Add LoadingSpinner component
PR #2: Add ProgressBar component
```

**Less good**:
```
PR #1: Add LoadingSpinner, ProgressBar, and StatusIcon
```

Smaller PRs are easier to review and merge!

---

## Community Questions

### Q: Where can I get help?

**A: Several places!**

1. **GitHub Issues** - Ask questions with `question` label
2. **GitHub Discussions** - General chat and ideas
3. **Pull Request Comments** - Ask during code review
4. **Discord/Slack** - (if available) Real-time chat

**Don't be shy!** The community is friendly and helpful.

### Q: I have an idea for a component. Should I ask first?

**A: Yes! Open a GitHub issue first.**

Template:
```markdown
Title: Component Idea: Network Graph Visualization

Description:
I'd like to create a component that visualizes network connections
as a graph with nodes and edges.

Stories:
- Simple graph (3 nodes)
- Complex graph (20+ nodes)
- Animated connection
- Error state (disconnected nodes)

Is this something the project would be interested in?
```

This way you get feedback before spending time coding!

### Q: Can I get credit for my contribution?

**A: Absolutely! You'll be listed as a contributor.**

Your GitHub profile will show:
- Commits to the repository
- Pull requests merged
- Contribution graph activity

You can also add it to your resume/portfolio:
- "Contributed TUI components to cmdai (Rust project)"
- Link to your merged PRs

---

## Advanced Questions

### Q: Can components be interactive?

**A: Yes! Implement `handle_key_event()`.**

```rust
fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
    match event.code {
        KeyCode::Up => {
            // Handle up arrow
            Ok(true)  // We handled it
        }
        _ => Ok(false)  // We didn't handle it
    }
}
```

See `table_selector.rs` for an example.

### Q: Can components have animation?

**A: Yes, but it requires some setup.**

You need:
1. State to track animation frame
2. Timer to know elapsed time
3. Browser to re-render periodically

See `progress_spinner.rs` for an example.

**Note**: The current browser doesn't auto-refresh. You might need to modify it or trigger re-renders manually.

### Q: Can I use external crates?

**A: Ask first, but probably yes!**

Good reasons:
- Syntax highlighting (use `syntect`)
- Date formatting (use `chrono`)
- Charts (use `tui-rs-tree-widget` or similar)

**Rule of thumb**: If it's a well-known, well-maintained crate that adds real value, it's probably fine.

**How to add**:
1. Propose it in GitHub issue
2. If approved, add to `Cargo.toml`
3. Use in your component

### Q: Can components communicate with each other?

**A: Not currently, but you could extend the framework!**

Current architecture: Components are isolated.

To enable communication:
1. Add shared state to `ShowcaseRegistry`
2. Implement event bus
3. Allow components to subscribe/publish

This would be a great contribution to the framework itself!

### Q: How do I make a component resize properly?

**A: Use percentage-based layouts:**

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ])
    .split(frame.area());
```

This splits the available space 30%/70% regardless of terminal size.

---

## Philosophical Questions

### Q: Why build this when there's already [X]?

**A: There isn't an [X] for Ratatui!**

This is the first and only Storybook-like tool for Rust terminal UIs. React has Storybook, we have this!

### Q: Who is this for?

**A: Everyone building terminal UIs in Rust!**

- Developers building TUI apps
- Component library authors
- Anyone learning Ratatui
- Open source contributors

### Q: What's the long-term vision?

**A: To be the de-facto standard for TUI component development!**

Goals:
- Comprehensive component library
- Visual regression testing
- Documentation generation
- Integration with other Ratatui projects

---

## Meta Questions

### Q: How do I update this FAQ?

**A: Submit a PR adding your question!**

Found a question that should be here? Add it:

1. Edit `FAQ.md`
2. Add your question and answer
3. Submit a PR
4. Help future contributors!

### Q: This FAQ doesn't answer my question!

**A: Ask it!**

1. Open a GitHub issue
2. Tag it with `question`
3. We'll answer and add it to the FAQ!

---

## Quick Reference

| Question | Answer |
|----------|--------|
| How to run showcase? | `cargo run --bin tui-showcase` |
| How to auto-rebuild? | `cargo watch -x 'run --bin tui-showcase'` |
| How to check syntax? | `cargo check` |
| How to format code? | `cargo fmt` |
| How to run linter? | `cargo clippy` |
| Where to add component? | `src/tui/components/my_component.rs` |
| Where to register? | `src/tui/components/mod.rs` + `src/bin/tui_showcase.rs` |
| How to get help? | GitHub issues with `question` label |

---

**Still have questions?** Don't hesitate to ask in GitHub issues! We're here to help! 💚

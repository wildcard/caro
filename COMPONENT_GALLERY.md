# 🎨 TUI Component Gallery

A visual guide to all available showcase components. Each component demonstrates different TUI patterns and use cases.

## 📋 Quick Reference

| Component | Category | Stories | Description |
|-----------|----------|---------|-------------|
| **SimpleText** | Display | 3 | Basic text display with styling |
| **CommandPreview** | Display | 3 | Shell command visualization |
| **TableSelector** | Display | 7 | Interactive data tables |
| **ConfirmationDialog** | Input | 4 | Modal confirmation dialogs |
| **CommandEditor** | Input | 7 | Multi-line command editing |
| **SafetyIndicator** | Feedback | 4 | Risk level visualization |
| **ProgressSpinner** | Feedback | 6 | Loading animations |
| **NotificationToast** | Feedback | 8 | Toasts and banners |
| **CommandFlow** | Workflow | 6 | Complete workflow visualization |
| **KeyboardShortcuts** | Help | 4 | Keyboard reference |

**Total**: 10 components | 52 stories | 5 categories

---

## 🎯 Display Components

### SimpleText

**Purpose**: Demonstrate basic text rendering with various styling options

```
┌─ Simple Text ─────────────────────────────┐
│                                            │
│       Hello, Ratatui Showcase!             │
│                                            │
└────────────────────────────────────────────┘

┌─ Styled Text ─────────────────────────────┐
│                                            │
│     Bold Italic Underline                  │
│                                            │
│     Red Green Blue                         │
│                                            │
└────────────────────────────────────────────┘
```

**Stories**: Default, Styled, MultiLine

### CommandPreview

**Purpose**: Display generated shell commands with syntax highlighting

```
┌─ Command Preview ─────────────────────────┐
│ Generated Command:                         │
│                                            │
│ $ find ~/Downloads -name "*.pdf" -size +10M│
│                                            │
└────────────────────────────────────────────┘

┌─ Command Preview - Complex ───────────────┐
│ Generated Command:                         │
│                                            │
│ $ find . -name '*.rs' \                    │
│     | grep -v target \                     │
│     | xargs wc -l                          │
│                                            │
└────────────────────────────────────────────┘
```

**Stories**: Simple Command, Complex Command, With Description

### TableSelector

**Purpose**: Interactive data tables with selection, sorting, and highlighting

```
┌─ Command History ─────────────────────────────────────────────────────┐
│ Time                 Query              Command                Status │
│────────────────────────────────────────────────────────────────────────│
│ 2025-01-19 14:32:15  list all PDFs      find . -name '*.pdf'   ✓ Success│
│ 2025-01-19 14:30:42  show disk usage    df -h                  ✓ Success│
│ 2025-01-19 14:28:19  find large logs    find /var/log ...      ✓ Success│
│ 2025-01-19 14:25:33  compress images    find . -name '*.jpg'   ⚠ Cancelled│
│ 2025-01-19 14:22:01  delete temp files  rm -rf /tmp/*          ✗ Blocked│
└────────────────────────────────────────────────────────────────────────┘
                                     ▲ Selected row (cyan background)
```

**Stories**: Default Table, First/Middle/Last Row Selected, Dangerous Rows Highlighted, No Header, Selected Dangerous

---

## 💡 Input Components

### ConfirmationDialog

**Purpose**: Modal dialog for user confirmation with Yes/No buttons

```
                    ┌─ Confirm Execution ──────┐
                    │                          │
                    │ Do you want to execute   │
                    │ this command?            │
                    │                          │
                    │ ls -la /home/user        │
                    │                          │
                    ├──────────────────────────┤
                    │                          │
                    │    [Yes]     No          │
                    │                          │
                    └──────────────────────────┘
                         ▲ Yes selected (green background)
```

**Stories**: Yes Selected, No Selected, Dangerous Command, Long Message

### CommandEditor

**Purpose**: Multi-line command editor with syntax highlighting and line numbers

```
┌─ Command Editor ──────────────────────────────────────────┐
│  Editor (Syntax Highlighting)                             │
├───────────────────────────────────────────────────────────┤
│   1 │ find . -name '*.rs' \                               │
│   2 │   | grep -v target \                                │
│   3 │   | xargs wc -l \                          ▍        │
│   4 │   | sort -n                                         │
│                                                            │
└────────────────────────────────────────────────────────────┘
│ ↑↓: Navigate | Enter: Execute | Ctrl+E: Edit | Esc: Cancel│
└────────────────────────────────────────────────────────────┘
         ▲ Line 3 selected with cursor indicator
```

**Stories**: Simple Command, Multi-line Pipeline, With Cursor (multiple positions), No Line Numbers, No Syntax Highlighting, Complex Shell Script

---

## 🔔 Feedback Components

### SafetyIndicator

**Purpose**: Visual indicator for command safety levels with color coding

```
┌─ Safety Level ──────┐     ┌─ Safety Level ──────┐     ┌─ Safety Level ──────┐
│                     │     │                     │     │                     │
│  ✓ SAFE             │     │  ⚠ HIGH RISK        │     │  ✗ CRITICAL         │
│                     │     │                     │     │                     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
 (Green border)              (Orange/Red border)        (Red border)

┌─ Command ───────────┐     ┌─ Command ───────────┐     ┌─ Command ───────────┐
│ $ ls -la            │     │ $ rm -rf ./target   │     │ $ sudo rm -rf /     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘

┌─ Description ───────┐     ┌─ Description ───────┐     ┌─ Description ───────┐
│ This command is     │     │ This command may    │     │ This command is     │
│ safe to execute     │     │ cause unintended    │     │ dangerous and should│
│                     │     │ changes             │     │ not be executed     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
```

**Stories**: Safe Command, Moderate Risk, High Risk, Critical Risk

### ProgressSpinner

**Purpose**: Animated spinner for loading and progress indication

```
┌─ Loading ───────────────────┐
│                             │
│       ⠋ Loading model...    │
│                             │
└─────────────────────────────┘
    Animation frames: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

**Stories**: Frames 0-3, Generating Command, Processing

### NotificationToast

**Purpose**: Temporary notifications with different styles and positions

```
Toast (Centered):                        Banner (Top):
    ┌────────────────────────┐          ╔═══════════════════════════════╗
    │                        │          ║ ✓ SUCCESS Configuration saved ║
    │ ℹ INFO: Loading...     │          ╚═══════════════════════════════╝
    │                        │
    └────────────────────────┘

Toast (Top-Right):                       Banner (Bottom):
                  ┌──────────────────┐   ╔═══════════════════════════════╗
                  │                  │   ║ ✗ ERROR Network connection lost║
                  │ ✗ ERROR: Failed  │   ╚═══════════════════════════════╝
                  │ Press key...     │
                  └──────────────────┘
```

**Stories**: Info/Success/Warning/Error Toast (various positions), Info/Success/Warning/Error Banner

---

## 🔄 Workflow Components

### CommandFlow

**Purpose**: Complete command generation workflow from input to execution

```
┌─ Command Generation Workflow ─────────────────────────────────────┐
│                  Command Generation Workflow                       │
└────────────────────────────────────────────────────────────────────┘

┌─ Progress ─────────────────┐
│                            │
│  ✓ 1. Input                │
│  ✓ 2. Generating           │
│  ▶ 3. Safety Check         │ ← Current step (green)
│  ○ 4. Confirmation         │
│  ○ 5. Executing            │
│  ○ 6. Complete             │
│                            │
└────────────────────────────┘

┌─ Current Step ─────────────┐
│                            │
│ Generated Command:         │
│                            │
│ $ find . -name '*.pdf'     │
│   -size +10M -ls           │
│                            │
│ ✓ SAFE - This command is  │
│   safe to execute          │
│                            │
└────────────────────────────┘
```

**Stories**: Step 1-6 (showing progression through entire workflow)

---

## ❓ Help Components

### KeyboardShortcuts

**Purpose**: Keyboard shortcuts reference in various display formats

```
Compact View:                          Grid Layout:
┌─ Keyboard Shortcuts ────┐           ┌─ Keyboard Shortcuts Reference ─┐
│ Ctrl+C         Exit     │           │    Keyboard Shortcuts Reference │
│ Enter          Confirm  │           ├─────────────────┬───────────────┤
│ Esc            Cancel   │           │ Essential       │ Advanced      │
│ ↑/↓            Navigate │           ├─────────────────┼───────────────┤
│ Ctrl+E         Edit     │           │ Ctrl+C          │ Ctrl+E        │
│ Ctrl+H         History  │           │ Cancel/Exit     │ Edit command  │
│ F1             Help     │           │                 │               │
└──────────────────────────┘           │ Enter           │ Ctrl+H        │
                                       │ Confirm/Execute │ Show history  │
Detailed View:                         │                 │               │
┌─ Help ───────────────────┐           │ Esc             │ Ctrl+R        │
│                          │           │ Cancel/Go back  │ Regenerate    │
│ Available Shortcuts      │           └─────────────────┴───────────────┘
│                          │           │ Press F1 anytime to show help   │
│ ╔═══ General ═══         │           └─────────────────────────────────┘
│ ║                        │
│ ║  Ctrl+C         → Exit │
│ ║  Enter          → Confirm│
│ ║                        │
│ ╚════════════════════    │
└──────────────────────────┘
```

**Stories**: Compact List, Compact with Categories, Detailed View, Grid Layout

---

## 🎯 Component Usage Patterns

### Basic Pattern

```rust
pub struct MyComponent;

impl ShowcaseComponent for MyComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new("MyComponent", "Description")
            .with_category("Display")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new("Default", "Default state", |frame, area| {
                // Render logic
            }),
        ]
    }
}
```

### Advanced Pattern with State

```rust
pub struct StatefulComponent {
    counter: usize,
}

impl ShowcaseComponent for StatefulComponent {
    fn metadata(&self) -> ComponentMetadata { /* ... */ }
    fn stories(&self) -> Vec<ShowcaseStory> { /* ... */ }

    fn handle_key_event(&mut self, event: KeyEvent) -> io::Result<bool> {
        match event.code {
            KeyCode::Char(' ') => {
                self.counter += 1;
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}
```

---

## 🚀 Running the Showcase

```bash
# Basic run
cargo run --bin tui-showcase

# With hot-reload for development
cargo watch -x 'run --bin tui-showcase'

# Build optimized version
cargo build --release --bin tui-showcase
./target/release/tui-showcase
```

## 📊 Component Statistics

- **Total Components**: 10
- **Total Stories**: 52
- **Total Lines of Code**: ~2,500+
- **Categories**: 5 (Display, Input, Feedback, Workflow, Help)
- **Average Stories per Component**: 5.2

## 🎨 Design Philosophy

1. **Isolation**: Each component is completely independent
2. **Stories**: Multiple variations show different states
3. **Consistency**: Uniform styling and interaction patterns
4. **Accessibility**: Keyboard-first navigation
5. **Documentation**: Self-documenting through examples

## 🔮 Future Component Ideas

- [ ] Progress bar with percentage
- [ ] File browser/tree view
- [ ] Search/filter input field
- [ ] Multi-select checkbox list
- [ ] Form with multiple input fields
- [ ] Tab navigation component
- [ ] Graph/chart visualization
- [ ] Log viewer with auto-scroll
- [ ] Split pane/layout manager
- [ ] Context menu/dropdown

---

**Built with ❤️ using [Ratatui](https://ratatui.rs/)** | See [TUI_SHOWCASE.md](TUI_SHOWCASE.md) for development guide

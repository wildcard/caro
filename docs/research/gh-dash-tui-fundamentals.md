# gh-dash & Terminal UI Fundamentals Research

> Background research for developing a best-in-class Terminal UI (T-UI) for cmdai

## Executive Summary

This research analyzes **gh-dash**, a widely-adopted terminal UI for GitHub, along with the broader terminal UI ecosystem, design patterns, and the Neovim community's workflow philosophy. The goal is to establish strong fundamentals for building cmdai's T-UI.

---

## 1. gh-dash: Case Study

### 1.1 Overview

**gh-dash** is "a rich terminal UI for GitHub that doesn't break your flow." Created by [Dolev Hadar](https://github.com/dlvhdr), it has become a reference implementation for keyboard-driven developer tools with **9.5k+ stars** and **70+ contributors**.

**Core Philosophy:**
- Keep developers in their terminal workflow
- Minimize context switching
- Keyboard-first navigation
- Deep integration with existing tools (Neovim, tmux, lazygit)

### 1.2 Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| TUI Framework | [Bubbletea](https://github.com/charmbracelet/bubbletea) | Elm-architecture based Go TUI framework |
| Styling | [Lipgloss](https://github.com/charmbracelet/lipgloss) | Terminal styling and layout |
| Markdown | [Glamour](https://github.com/charmbracelet/glamour) | Markdown rendering in terminal |
| CLI | Cobra | Command-line interface structure |
| GitHub API | gh CLI + GraphQL | Data fetching and operations |
| Diffs | Delta | Syntax-highlighted PR diff viewing |

### 1.3 Architecture Pattern: The Elm Architecture

gh-dash follows **The Elm Architecture (TEA)**, a functional reactive pattern:

```
┌─────────────────────────────────────────────────────────┐
│                    The Elm Architecture                  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   Model ──► View ──► User Events ──► Update ──► Model   │
│     ▲                                            │       │
│     └────────────────────────────────────────────┘       │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**Three Core Methods:**
1. **Init** - Returns initial command for the application
2. **Update** - Handles incoming events and updates model
3. **View** - Renders UI based on current model state

### 1.4 Project Structure

```
gh-dash/
├── ui/          # Terminal rendering components
├── data/        # GitHub GraphQL API integration
├── config/      # YAML configuration parsing
├── utils/       # Shared utilities
└── docs/        # Hugo-based documentation
```

### 1.5 Configuration Design

gh-dash uses YAML-based configuration with powerful features:

```yaml
prSections:
  - title: "My Pull Requests"
    filters: is:open author:@me
    layout:
      author:
        hidden: true

defaults:
  view: prs
  refetchIntervalMinutes: 30

preview:
  open: true
  width: 84

keybindings:
  prs:
    - key: g
      command: >
        tmux new-window -c {{.RepoPath}} lazygit
```

**Key Configuration Patterns:**
- Declarative section definitions
- GitHub search syntax for filtering
- Template functions (e.g., `{{ nowModify "-3w" }}`)
- Custom keybindings with shell command execution
- Layout customization per section

---

## 2. The Charmbracelet Ecosystem

### 2.1 Bubbletea Framework

[Bubbletea](https://github.com/charmbracelet/bubbletea) is the dominant Go TUI framework, powering **10,000+ applications**.

**Core Principles:**
- Functional, stateful approach to TUI
- Based on The Elm Architecture
- Predictable, state-driven updates
- Easy to reason about, test, and maintain

**Basic Program Structure:**
```go
type Model struct {
    // Application state
}

func (m Model) Init() tea.Cmd {
    return nil // Initial command
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
    // Handle events, return new model
    return m, nil
}

func (m Model) View() string {
    // Render UI to string
    return "Hello, World!"
}
```

### 2.2 Bubbles: Reusable Components

[Bubbles](https://github.com/charmbracelet/bubbles) provides production-ready TUI components:

| Category | Components |
|----------|------------|
| **Input** | Text Input, Text Area, File Picker |
| **Display** | Spinner, Table, Viewport, Progress |
| **Navigation** | List (with fuzzy filtering), Paginator |
| **Utility** | Timer, Stopwatch, Help, Key binding manager |

**Component Patterns:**
- Consistent Model-Update-View interface
- Customizable styling and behavior
- Composable into larger applications
- Built-in keybinding support with remapping

### 2.3 Related Tools

| Tool | Purpose |
|------|---------|
| **Lipgloss** | Style, format, and layout |
| **Harmonica** | Spring animations for smooth motion |
| **BubbleZone** | Mouse event tracking |
| **ntcharts** | Terminal charting |

---

## 3. Ratatui: Rust TUI Framework

For cmdai (a Rust project), **Ratatui** is the primary TUI framework option.

### 3.1 Overview

[Ratatui](https://github.com/ratatui/ratatui) is a Rust crate for building terminal UIs, forked from tui-rs in 2023. It uses **immediate mode rendering** with intermediate buffers.

### 3.2 The Elm Architecture in Ratatui

Ratatui supports TEA-style architecture:

```rust
struct Model {
    // Application state
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    // Handle events, update model
    None
}

fn view(model: &Model, frame: &mut Frame) {
    // Render UI based on model
}
```

**Key Difference from Bubbletea:**
- Bubbletea is **opinionated** (guides you into TEA pattern)
- Ratatui is **unopinionated** (provides components, you design the loop)

### 3.3 Performance Advantage

In benchmarks comparing Bubbletea (Go) vs Ratatui (Rust):
- **30-40% less memory usage**
- **15% lower CPU footprint**
- Better suited for high-frequency updates (log monitors, real-time dashboards)

### 3.4 Widget Ecosystem

**Core Widgets:** Block, Paragraph, List, Table, Chart, Gauge, Tabs, Canvas

**Third-Party Libraries:**
| Library | Features |
|---------|----------|
| **rat-widget** | Data input, structural widgets, file dialog, menubar |
| **ratatui-textarea** | Powerful text editor widget |
| **ratatui-code-editor** | Syntax highlighting with tree-sitter |
| **tui-realm** | React/Elm-inspired component framework |
| **tachyonfx** | Shader-like visual effects |

### 3.5 tui-realm: Component Framework

For React-like component patterns in Rust:

```rust
// Components with properties and states
// Message-based communication
// Built-in view mounting/unmounting
// Focus management
```

---

## 4. Keyboard-Centric Design Principles

### 4.1 Vim-Style Navigation

The Neovim community expects:

| Key | Action |
|-----|--------|
| `h/j/k/l` | Navigation (left/down/up/right) |
| `g/G` | Go to top/bottom |
| `Ctrl+d/u` | Page down/up |
| `/` | Search/filter |
| `q` | Quit |
| `?` | Help |
| `Enter` | Select/confirm |
| `Space` | Toggle/stage |

### 4.2 Modal Interface Design

**Lazygit's Approach:**
- Multiple panels with `1/2/3...` or `Tab` navigation
- Context-sensitive keybindings per panel
- Visual indicators for current mode
- Consistent `?` for help everywhere

### 4.3 Keybinding Best Practices

1. **Discoverability**: Always provide `?` for keybinding reference
2. **Consistency**: Match vim conventions for navigation
3. **Customizability**: Allow YAML/config-based remapping
4. **Accessibility**: Keyboard-only operation must be complete
5. **Mnemonics**: Use meaningful key associations (`c` for commit, `p` for push)

### 4.4 gh-dash Keybinding Philosophy

```yaml
# Override defaults with user preferences
keybindings:
  prs:
    - key: C      # Uppercase for custom actions
      command: nvim -c ":silent Octo pr edit {{.PrNumber}}"
    - key: w      # Watch PR status
      command: gh pr checks {{.PrNumber}} --watch
```

---

## 5. Neovim Community Integration

### 5.1 The Terminal Workflow Stack

```
┌────────────────────────────────────────────────┐
│                   tmux                          │
│  ┌──────────────────────────────────────────┐  │
│  │ CMD+G: lazygit | CMD+Shift+G: gh-dash    │  │
│  ├──────────────────────────────────────────┤  │
│  │              Neovim                       │  │
│  │  - octo.nvim (GitHub integration)        │  │
│  │  - gitsigns (git hunks, blame)           │  │
│  │  - fugitive (git commands)               │  │
│  │  - lazygit.nvim (floating lazygit)       │  │
│  └──────────────────────────────────────────┘  │
└────────────────────────────────────────────────┘
```

### 5.2 Key Plugins and Their Roles

| Tool | Purpose | Integration |
|------|---------|-------------|
| **octo.nvim** | Edit/review GitHub PRs in Neovim | "Leaving comments in Neovim is magical" |
| **gitsigns** | Git hunks, inline blame | Real-time git status in buffer |
| **lazygit** | Full git TUI | Floating popup via tmux/Neovim |
| **gh-dash** | GitHub dashboard | Global tmux keybinding |
| **delta** | Syntax-highlighted diffs | Integrates with lazygit/gh-dash |

### 5.3 Dolev Hadar's Workflow Philosophy

1. **Global Accessibility**: Tools bound to tmux shortcuts for instant access
2. **Seamless Transitions**: `C` key in gh-dash opens PR in Neovim with octo.nvim
3. **Watch and Notify**: `w` key watches CI status with notifications
4. **Minimize Context Switching**: Everything stays in terminal
5. **Tools Replace Browser**: gh-dash + octo.nvim + lazygit ≈ github.com

### 5.4 tmux Integration Patterns

```bash
# Global gh-dash access
bind-key G new-window gh dash

# Floating lazygit popup
bind-key g popup -d "#{pane_current_path}" -w80% -h80% -E lazygit

# Quick PR review in Neovim
bind-key P run-shell "gh pr view --web"
```

---

## 6. Visual Design & Accessibility

### 6.1 Color Theme Best Practices

**Background Colors:**
- Use dark gray (`#121212`) instead of pure black
- Google's Material Design recommendation
- Only 0.3% more power than pure black on OLED

**Text Opacity (on dark backgrounds):**
| Emphasis | Opacity | Use Case |
|----------|---------|----------|
| High | 87% white | Primary text |
| Medium | 60% white | Secondary text |
| Disabled | 38% white | Inactive elements |

**Contrast Requirements:**
- WCAG 2.1: Minimum 4.5:1 for normal text
- AAA standard: 7:1 for enhanced accessibility

### 6.2 Color Saturation

- **Avoid highly saturated colors** on dark backgrounds
- Saturated colors appear to vibrate/blur
- Use muted, desaturated accent colors
- Light, unsaturated tones for buttons and icons

### 6.3 User Preferences

- **Always offer theme switching** (dark/light)
- **Sync with system theme** when possible
- Some users with astigmatism find light-on-dark blurry
- Let users choose their preference

### 6.4 Visual Hierarchy

- Use **brighter colors** for foreground/elevated elements
- Use **darker shades** for background layers
- **Avoid shadows** in dark mode (use lighter surfaces for elevation)
- Border colors should be subtle (10-20% contrast)

---

## 7. Mental Models & UX Consistency

### 7.1 Jakob's Law

> "Users spend most of their time on other sites/apps. They prefer your tool to work the same way as tools they already know."

**Implication for cmdai:**
- Follow vim conventions for navigation
- Match terminal tool conventions (q=quit, ?=help, /=search)
- Use familiar patterns from lazygit, gh-dash, htop

### 7.2 Reducing Cognitive Load

| Technique | Application |
|-----------|-------------|
| **Consistency** | Same keys do same things across panels |
| **Familiar UI Elements** | Standard list, table, input patterns |
| **Progressive Disclosure** | Show advanced options only when needed |
| **Hick's Law** | Limit choices per screen to reduce decision time |

### 7.3 Handling Mental Model Mismatches

When introducing new concepts:
1. Provide clear visual cues
2. Include inline help/tooltips
3. Offer tutorials or guided first-run
4. Use labels that match user expectations

---

## 8. Architecture Recommendations for cmdai T-UI

### 8.1 Framework Choice: Ratatui

**Recommendation:** Use **Ratatui** with optional **tui-realm** for component patterns.

**Rationale:**
- Native Rust (matches cmdai's codebase)
- Better performance for real-time updates
- Active community and ecosystem
- Supports Elm Architecture pattern

### 8.2 Proposed Architecture

```
cmdai-tui/
├── src/
│   ├── app.rs              # Application state (Model)
│   ├── event.rs            # Event handling
│   ├── update.rs           # Message processing (Update)
│   ├── ui/
│   │   ├── mod.rs          # View composition
│   │   ├── components/     # Reusable widgets
│   │   │   ├── command_input.rs
│   │   │   ├── safety_indicator.rs
│   │   │   ├── history_list.rs
│   │   │   └── confirmation_dialog.rs
│   │   └── themes/         # Color schemes
│   ├── keybindings.rs      # Configurable key mappings
│   └── config.rs           # YAML configuration
└── tests/
    └── ui/                 # UI component tests
```

### 8.3 Core Components

| Component | Purpose | Key Features |
|-----------|---------|--------------|
| **Command Input** | Natural language entry | Syntax highlighting, history |
| **Safety Indicator** | Risk level display | Color-coded (green/yellow/red) |
| **Command Preview** | Generated command | Copyable, editable |
| **Confirmation Dialog** | User approval | Clear Y/N with explanation |
| **History List** | Previous commands | Searchable, rerunnable |

### 8.4 Keybinding Design

```yaml
# cmdai TUI keybindings (proposed)
global:
  q: quit
  ?: help
  /: search_history

input:
  Enter: generate_command
  Ctrl+c: cancel
  Up/Down: history_navigation

preview:
  y: copy_to_clipboard
  e: edit_command
  Enter: execute (with confirmation)
  Escape: back_to_input

confirmation:
  y/Enter: confirm_execute
  n/Escape: cancel
  d: show_details
```

### 8.5 Configuration Schema

```yaml
# ~/.config/cmdai/tui.yml
theme: dark  # or light, system
safety:
  require_confirmation: true
  danger_level_threshold: moderate

keybindings:
  custom:
    - key: x
      action: execute_without_confirm
      requires: explicit_enable

layout:
  preview_width: 60%
  history_visible: true

backends:
  default: mlx
  fallback: ollama
```

---

## 9. Key Takeaways

### 9.1 Design Principles

1. **Flow Preservation**: Don't break the user's workflow
2. **Keyboard-First**: Every action accessible via keyboard
3. **Vim Conventions**: hjkl navigation, modal interfaces
4. **Discoverability**: `?` always shows help
5. **Configurability**: YAML-based customization
6. **Integration**: Play well with tmux, Neovim ecosystem

### 9.2 Technical Decisions

1. Use **Ratatui** with **Elm Architecture**
2. Implement **component-based** widget design
3. Support **theme customization** with accessibility focus
4. Enable **keybinding remapping** via config
5. Provide **plugin hooks** for tool integration

### 9.3 Community Alignment

- Target Neovim/terminal users as primary audience
- Follow conventions from lazygit, gh-dash
- Support tmux popup/floating window patterns
- Enable seamless editor integration

---

## 10. Sources

### Primary Research
- [gh-dash GitHub Repository](https://github.com/dlvhdr/gh-dash)
- [Charmbracelet Bubbletea](https://github.com/charmbracelet/bubbletea)
- [Charmbracelet Bubbles](https://github.com/charmbracelet/bubbles)
- [Ratatui Documentation](https://ratatui.rs/)
- [Ratatui GitHub](https://github.com/ratatui/ratatui)

### Workflow & Integration
- [GitHub In The Terminal - Josh Medeski](https://www.joshmedeski.com/posts/github-in-the-terminal/)
- [Lazygit GitHub](https://github.com/jesseduffield/lazygit)
- [octo.nvim GitHub](https://github.com/pwntester/octo.nvim)
- [Awesome TUIs](https://github.com/rothgar/awesome-tuis)

### Design & UX
- [Mental Models in UX - Nielsen Norman Group](https://www.nngroup.com/articles/mental-models/)
- [Keyboard Shortcuts Design Pattern - UI Patterns](https://ui-patterns.com/patterns/keyboard-shortcuts)
- [Dark Mode Best Practices - Smashing Magazine](https://www.smashingmagazine.com/2025/04/inclusive-dark-mode-designing-accessible-dark-themes/)
- [Go vs Rust for TUI Development](https://dev.to/dev-tngsh/go-vs-rust-for-tui-development-a-deep-dive-into-bubbletea-and-ratatui-2b7)

### Framework Documentation
- [tui-realm Framework](https://github.com/veeso/tui-realm)
- [Ratatui Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/)
- [Bubbletea Tutorial](https://github.com/charmbracelet/bubbletea/blob/main/tutorials/basics/main.go)

---

*Research compiled: December 2024*
*For: cmdai T-UI Development*

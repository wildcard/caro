# cmdai TUI - High-Level Design (HLD)

**Version:** 1.0.0
**Date:** 2025-11-19
**Status:** Active Development
**Phase:** Phase 1 - MVP REPL Mode

---

## 🎯 Vision

Transform cmdai from a powerful but hidden CLI tool into an **intuitive, discoverable, beautiful terminal interface** that welcomes users and exposes all functionality through a keyboard-driven, responsive UI.

### Design Principles

1. **🎨 Beautiful** - Clean layouts, thoughtful colors, delightful interactions
2. **⚡ Responsive** - < 50ms input latency, non-blocking operations
3. **🔍 Discoverable** - All features visible, keyboard shortcuts shown
4. **⌨️ Keyboard-First** - Mouse optional, vim-like bindings
5. **📚 Educational** - Helps users learn cmdai's capabilities
6. **🤝 Contributor-Friendly** - Clear architecture, well-documented

---

## 📐 Phase 1 MVP - REPL Mode

### Visual Mockup

```
╭─ cmdai ──────────────────────────────────────────────────────────────╮
│ ⚙ Ollama • bash • Moderate Safety                          [?] Help │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│ 🤖 Type your command in natural language...                          │
│                                                                       │
│ find all python files modified today_                                │
│                                                                       │
│ ┌─ Validation ──────────────────────────────────────────────────┐    │
│ │ ✓ Safe command pattern                                       │    │
│ │ ⚠ May search large directory tree                            │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                       │
│ ┌─ Generated Command ──────────────────────────────────────────┐    │
│ │ find . -type f -name "*.py" -mtime -1                        │    │
│ │                                                               │    │
│ │ 💡 Searches current directory for Python files modified in    │    │
│ │    the last 24 hours                                          │    │
│ └──────────────────────────────────────────────────────────────┘    │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
├──────────────────────────────────────────────────────────────────────┤
│ [Enter] Generate  [Ctrl+R] History  [Ctrl+C] Quit  [?] More Help    │
╰──────────────────────────────────────────────────────────────────────╯
```

### Screen Layout Anatomy

```
┌─────────────────────────────────────────┐
│         Status Bar (1 line)             │  ← Current state, config
├─────────────────────────────────────────┤
│                                         │
│                                         │
│        Main Content Area                │  ← Active mode renders here
│           (Min height)                  │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│         Help Footer (1 line)            │  ← Keyboard shortcuts
└─────────────────────────────────────────┘
```

**Constraints:**
```rust
Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),      // Status bar - fixed 1 line
        Constraint::Min(10),        // Main content - flexible
        Constraint::Length(1),      // Help footer - fixed 1 line
    ])
```

---

## 🎨 Component Breakdown

### 1. StatusBarComponent

**Purpose:** Display current TUI state and configuration

**Visual Design:**
```
⚙ Ollama • bash • Moderate Safety                          [?] Help
└─┬──┘   └─┬─┘   └───────┬──────┘                          └───┬──┘
  │        │             │                                      │
Backend  Shell    Safety Level                          Help Indicator
```

**Color Scheme:**
- Backend: `Color::Cyan` (available) / `Color::Red` (unavailable)
- Shell: `Color::Green`
- Safety Level:
  - Strict: `Color::Red`
  - Moderate: `Color::Yellow`
  - Permissive: `Color::Green`

**Props:**
```rust
pub struct StatusBarProps {
    pub backend: BackendInfo,
    pub shell: ShellType,
    pub safety_level: SafetyLevel,
    pub show_help: bool,
}

pub struct BackendInfo {
    pub name: String,
    pub available: bool,
    pub model: Option<String>,
}
```

**Render Code Pattern:**
```rust
impl Component for StatusBarComponent {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let backend_color = if self.props.backend.available {
            Color::Cyan
        } else {
            Color::Red
        };

        let status_text = vec![
            Span::styled("⚙ ", Style::default().fg(backend_color)),
            Span::styled(&self.props.backend.name, Style::default().fg(backend_color)),
            Span::raw(" • "),
            Span::styled(self.props.shell.to_string(), Style::default().fg(Color::Green)),
            // ... more spans
        ];

        let paragraph = Paragraph::new(Line::from(status_text))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}
```

---

### 2. ReplComponent (Main Content)

**Purpose:** Interactive command input and generation

**Layout Structure:**
```
┌────────────────────────────────────────┐
│  Input Area (4-10 lines)               │  ← Expandable text input
├────────────────────────────────────────┤
│  Validation Panel (3-5 lines)         │  ← Live feedback
├────────────────────────────────────────┤
│  Command Preview (5+ lines)           │  ← Generated command + explanation
└────────────────────────────────────────┘
```

**Sub-Components:**

#### 2a. InputArea

**Visual Design:**
```
┌───────────────────────────────────────────────────────┐
│ 🤖 Type your command in natural language...          │
│                                                       │
│ find all python files modified today_                │
│                                                       │
└───────────────────────────────────────────────────────┘
```

**Features:**
- Multi-line text input
- Cursor visualization (blinking)
- Placeholder text when empty
- Auto-expand up to 10 lines
- Syntax-aware (future: highlight @-tags)

**State:**
```rust
pub struct InputState {
    buffer: String,
    cursor_position: usize,
    scroll_offset: usize,  // For long text
    is_focused: bool,
}
```

#### 2b. ValidationPanel

**Visual Design - Safe Command:**
```
┌─ Validation ────────────────────────────────────┐
│ ✓ Safe command pattern                         │
│ ✓ POSIX compliant                              │
└────────────────────────────────────────────────┘
```

**Visual Design - Dangerous Command:**
```
┌─ Validation ────────────────────────────────────┐
│ ⚠ MODERATE RISK                                │
│ • Recursive file deletion                      │
│ • Target path is relative                      │
│ ℹ Consider: rm -ri ./old_logs (interactive)    │
└────────────────────────────────────────────────┘
```

**Color Coding:**
- ✓ Green for safe patterns
- ⚠ Yellow for warnings
- ❌ Red for blocked/critical
- ℹ Blue for suggestions

**Props:**
```rust
pub struct ValidationProps {
    pub result: Option<ValidationResult>,
    pub loading: bool,
}

pub struct ValidationResult {
    pub risk_level: RiskLevel,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub matched_patterns: Vec<String>,
}
```

#### 2c. CommandPreviewPanel

**Visual Design:**
```
┌─ Generated Command ──────────────────────────────┐
│ find . -type f -name "*.py" -mtime -1            │
│                                                  │
│ 💡 Searches current directory for Python files   │
│    modified in the last 24 hours                 │
└──────────────────────────────────────────────────┘
```

**Features:**
- Syntax-highlighted command
- Explanation text with icon
- Copy button indicator (future)
- Loading spinner during generation

**Props:**
```rust
pub struct CommandPreviewProps {
    pub command: Option<String>,
    pub explanation: Option<String>,
    pub loading: bool,
    pub error: Option<String>,
}
```

**Render States:**

1. **Idle (no input):**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│        Start typing to generate a command...     │
│                                                  │
└──────────────────────────────────────────────────┘
```

2. **Loading:**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│        ⏳ Generating command...                  │
│                                                  │
└──────────────────────────────────────────────────┘
```

3. **Success:**
```
┌─ Generated Command ──────────────────────────────┐
│ find . -type f -name "*.py" -mtime -1            │
│                                                  │
│ 💡 Explanation text here                         │
└──────────────────────────────────────────────────┘
```

4. **Error:**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│  ❌ Error: Backend unavailable                   │
│     Check Ollama is running on localhost:11434   │
│                                                  │
└──────────────────────────────────────────────────┘
```

---

### 3. HelpFooterComponent

**Purpose:** Show context-sensitive keyboard shortcuts

**Visual Design:**
```
[Enter] Generate  [Ctrl+R] History  [Ctrl+C] Quit  [?] More Help
```

**Color Scheme:**
- Brackets: `Color::DarkGray`
- Keys: `Color::Cyan` (bold)
- Description: `Color::White`

**Props:**
```rust
pub struct HelpFooterProps {
    pub shortcuts: Vec<Shortcut>,
}

pub struct Shortcut {
    pub key: String,
    pub description: String,
    pub enabled: bool,
}
```

**Context-Aware Shortcuts:**
```rust
fn get_shortcuts_for_mode(mode: AppMode) -> Vec<Shortcut> {
    match mode {
        AppMode::Repl => vec![
            Shortcut::new("Enter", "Generate"),
            Shortcut::new("Ctrl+R", "History"),
            Shortcut::new("Ctrl+C", "Quit"),
            Shortcut::new("?", "Help"),
        ],
        AppMode::History => vec![
            Shortcut::new("↑↓", "Navigate"),
            Shortcut::new("Enter", "Copy"),
            Shortcut::new("Esc", "Back"),
            Shortcut::new("/", "Search"),
        ],
        // ... more modes
    }
}
```

---

## 🏗️ Architecture Diagram

### Component Tree

```
TuiApp
│
├── TerminalManager
│   ├── setup_terminal()
│   ├── restore_terminal()
│   └── draw()
│
├── EventHandler
│   ├── poll_events()
│   └── dispatch()
│
├── AppState
│   ├── current_mode: AppMode
│   ├── repl_state: ReplState
│   ├── config: UserConfiguration
│   └── backend_bridge: BackendBridge
│
└── Components
    ├── StatusBarComponent
    ├── ReplComponent
    │   ├── InputArea
    │   ├── ValidationPanel
    │   └── CommandPreviewPanel
    └── HelpFooterComponent
```

### State Flow Diagram

```
┌─────────────┐
│ User Input  │ (Keyboard Event)
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│  Event Handler   │ Parse key event
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  AppState Update │ Mutate state based on event
└──────┬───────────┘
       │
       ├─────────────────────┐
       │                     │
       ▼                     ▼
┌────────────┐        ┌──────────────┐
│ Side Effect│        │   Re-render  │
│  (Async)   │        │  Components  │
└─────┬──────┘        └──────────────┘
      │
      │ (Backend call)
      ▼
┌──────────────────┐
│  CliApp::Backend │ Generate command
└──────┬───────────┘
       │
       │ (Result)
       ▼
┌──────────────────┐
│  AppState Update │ Store result
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│    Re-render     │ Show command
└──────────────────┘
```

### Event Flow Example: "User Presses Enter"

```
1. crossterm::event::read() → KeyEvent(Enter)
2. EventHandler::handle_key(Enter)
3. AppState::handle_generate_command()
4. BackendBridge::generate_command_async()
   ├─ Show loading state → Re-render
   ├─ CliApp.run_with_args()
   └─ Return CliResult
5. AppState::set_generated_command(result)
6. Re-render with command preview
```

---

## 📊 State Management

### AppState Structure

```rust
/// Central application state - single source of truth
pub struct AppState {
    // Mode management
    pub current_mode: AppMode,

    // REPL state
    pub repl: ReplState,

    // Shared config
    pub config: UserConfiguration,

    // Backend status
    pub backend_status: BackendStatus,

    // UI state
    pub show_help_modal: bool,
    pub error_message: Option<String>,
}

pub struct ReplState {
    // User input
    pub input_buffer: String,
    pub cursor_position: usize,

    // Generation state
    pub generating: bool,
    pub generated_command: Option<GeneratedCommand>,

    // Validation state
    pub validating: bool,
    pub validation_result: Option<ValidationResult>,
}

pub enum AppMode {
    Repl,
    History,
    Config,
    Help,
}
```

### State Transitions

```rust
pub enum AppEvent {
    // Input events
    KeyPress(KeyEvent),
    TextInput(char),
    Backspace,
    Enter,

    // Mode changes
    SwitchMode(AppMode),

    // Async results
    CommandGenerated(Result<GeneratedCommand>),
    ValidationComplete(Result<ValidationResult>),

    // Control
    Quit,
}

impl AppState {
    pub fn handle_event(&mut self, event: AppEvent) -> Vec<SideEffect> {
        match event {
            AppEvent::TextInput(c) => {
                self.repl.input_buffer.push(c);
                self.repl.cursor_position += 1;
                vec![SideEffect::TriggerValidation]
            }
            AppEvent::Enter => {
                if !self.repl.input_buffer.is_empty() {
                    self.repl.generating = true;
                    vec![SideEffect::GenerateCommand(
                        self.repl.input_buffer.clone()
                    )]
                } else {
                    vec![]
                }
            }
            // ... more handlers
        }
    }
}
```

---

## 🎮 Keyboard Bindings

### Global Bindings

| Key | Action | Description |
|-----|--------|-------------|
| `Ctrl+C` | Quit | Exit application |
| `Ctrl+R` | History | Open history browser |
| `?` | Help | Toggle help modal |
| `Esc` | Back | Return to REPL mode |

### REPL Mode Bindings

| Key | Action | Description |
|-----|--------|-------------|
| `Char(c)` | Insert | Type character |
| `Backspace` | Delete | Remove character |
| `Delete` | Delete | Remove character forward |
| `Enter` | Generate | Generate command from input |
| `Ctrl+Enter` | Execute | Generate and execute directly |
| `Tab` | Autocomplete | Show suggestions (future) |
| `↑` | History Back | Previous input (future) |
| `↓` | History Forward | Next input (future) |
| `Ctrl+L` | Clear | Clear input buffer |

### Visual Keyboard Mapping

```
┌───────────────────────────────────────────┐
│  ?          [Help Modal]                  │
│  Esc        [Back to REPL]                │
│  Ctrl+C     [Quit]                        │
│  Ctrl+R     [History Browser]             │
│  Enter      [Generate Command]            │
│  Ctrl+Enter [Generate & Execute]          │
│  Ctrl+L     [Clear Input]                 │
│  ↑/↓        [Input History] (future)      │
│  Tab        [Autocomplete] (future)       │
└───────────────────────────────────────────┘
```

---

## 🎨 Color Palette

### Brand Colors

```rust
pub struct Theme {
    // Primary colors
    pub primary: Color,      // Cyan - main accent
    pub secondary: Color,    // Blue - secondary actions
    pub success: Color,      // Green - safe, success
    pub warning: Color,      // Yellow - moderate risk
    pub danger: Color,       // Red - high risk, errors

    // UI colors
    pub background: Color,   // Black/Dark
    pub foreground: Color,   // White/Light
    pub border: Color,       // DarkGray
    pub muted: Color,        // Gray

    // Status colors
    pub info: Color,         // Blue
    pub loading: Color,      // Cyan (animated)
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            background: Color::Black,
            foreground: Color::White,
            border: Color::DarkGray,
            muted: Color::Gray,
            info: Color::Blue,
            loading: Color::Cyan,
        }
    }
}
```

### Semantic Usage

```rust
// Risk levels
RiskLevel::Safe      → Color::Green
RiskLevel::Moderate  → Color::Yellow
RiskLevel::High      → Color::Red
RiskLevel::Critical  → Color::Red + Bold

// Status indicators
BackendAvailable     → Color::Cyan
BackendUnavailable   → Color::Red
Generating          → Color::Cyan (with spinner)
ValidationPassed    → Color::Green
ValidationFailed    → Color::Yellow/Red

// UI elements
Border             → Color::DarkGray
Title              → Color::Cyan + Bold
Placeholder        → Color::Gray
SelectedItem       → Color::Black + Bg(Cyan)
```

---

## 📁 File Structure

```
src/
├── tui/
│   ├── mod.rs                    # Public API, re-exports
│   │
│   ├── app.rs                    # TuiApp - main application
│   │   ├── struct TuiApp
│   │   ├── impl TuiApp::new()
│   │   ├── impl TuiApp::run()
│   │   └── impl TuiApp::render()
│   │
│   ├── state/
│   │   ├── mod.rs
│   │   ├── app_state.rs          # AppState definition
│   │   ├── repl_state.rs         # ReplState definition
│   │   └── events.rs             # AppEvent enum
│   │
│   ├── components/
│   │   ├── mod.rs
│   │   ├── component.rs          # Component trait
│   │   ├── status_bar.rs         # StatusBarComponent
│   │   ├── repl/
│   │   │   ├── mod.rs
│   │   │   ├── repl.rs           # ReplComponent
│   │   │   ├── input.rs          # InputArea
│   │   │   ├── validation.rs     # ValidationPanel
│   │   │   └── preview.rs        # CommandPreviewPanel
│   │   └── help_footer.rs        # HelpFooterComponent
│   │
│   ├── events/
│   │   ├── mod.rs
│   │   ├── handler.rs            # EventHandler
│   │   └── keys.rs               # Key binding definitions
│   │
│   ├── backend/
│   │   ├── mod.rs
│   │   └── bridge.rs             # BackendBridge - wraps CliApp
│   │
│   ├── theme/
│   │   ├── mod.rs
│   │   └── colors.rs             # Theme, color definitions
│   │
│   └── utils/
│       ├── mod.rs
│       ├── terminal.rs           # Terminal setup/cleanup
│       └── layout.rs             # Layout helpers
│
└── main.rs                        # CLI entry - add --tui flag
```

### Module Responsibilities

| Module | Responsibility | Exports |
|--------|---------------|---------|
| `tui/app.rs` | Main TUI application, event loop | `TuiApp` |
| `tui/state/` | State management, events | `AppState`, `AppEvent` |
| `tui/components/` | UI components | All components |
| `tui/events/` | Event handling, key bindings | `EventHandler` |
| `tui/backend/` | Integration with CliApp | `BackendBridge` |
| `tui/theme/` | Colors, styling | `Theme` |
| `tui/utils/` | Utilities | Helper functions |

---

## 🔌 Integration Points

### 1. Backend Integration

```rust
// src/tui/backend/bridge.rs
use crate::cli::CliApp;
use crate::models::{Cli, CliResult};

pub struct BackendBridge {
    cli_app: CliApp,
}

impl BackendBridge {
    pub fn new() -> Result<Self> {
        let cli_app = CliApp::new()?;
        Ok(Self { cli_app })
    }

    pub async fn generate_command(
        &mut self,
        input: String,
        shell: ShellType,
        safety: SafetyLevel,
    ) -> Result<GeneratedCommand> {
        let args = Cli {
            prompt: Some(input),
            shell: Some(shell.to_string()),
            safety: Some(safety.to_string()),
            output: Some("json".to_string()),
            ..Default::default()
        };

        let result = self.cli_app.run_with_args(args).await?;

        Ok(GeneratedCommand {
            command: result.generated_command,
            explanation: result.explanation,
            risk_level: result.risk_level,
        })
    }
}
```

**Key Benefits:**
- ✅ No code duplication
- ✅ Consistent backend selection logic
- ✅ Reuse all existing backends (Ollama, vLLM, Embedded)
- ✅ Same safety validation

### 2. Configuration Integration

```rust
// Reuse existing config system
use crate::config::ConfigManager;

impl TuiApp {
    pub fn new() -> Result<Self> {
        let config_manager = ConfigManager::load()?;
        let user_config = config_manager.user_config();

        Ok(Self {
            state: AppState {
                config: user_config.clone(),
                // ... more state
            },
            // ...
        })
    }
}
```

### 3. Safety Validation Integration

```rust
use crate::safety::SafetyValidator;

impl BackendBridge {
    pub async fn validate_command(
        &self,
        command: &str,
        shell: ShellType,
    ) -> Result<ValidationResult> {
        let validator = SafetyValidator::new();
        let result = validator.validate_command(command, shell);

        Ok(ValidationResult {
            risk_level: result.risk_level,
            warnings: result.warnings,
            suggestions: result.alternatives,
            matched_patterns: result.matched_patterns,
        })
    }
}
```

---

## ⚡ Performance Requirements

### Startup Performance

```
Target: < 200ms total startup time

Breakdown:
- Terminal setup:         < 10ms
- Config loading:         < 20ms
- Backend initialization: < 100ms
- First render:           < 20ms
- Event loop ready:       < 50ms
```

### Runtime Performance

```
Input Latency:
- Keystroke to screen:  < 16ms (60fps)
- Keystroke to state:   < 5ms

Command Generation:
- Local inference:      < 2s (Ollama/MLX)
- Remote inference:     < 5s (vLLM)
- Validation:           < 50ms

Rendering:
- Frame render time:    < 16ms (60fps)
- Layout calculation:   < 5ms
```

### Memory Targets

```
Idle State:        < 50MB
Active Generation: < 200MB
With History:      < 300MB (10k entries)
```

---

## 🧪 Testing Strategy

### Unit Tests

**Component Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_component_inserts_character() {
        let mut input = InputArea::new();

        input.handle_key(KeyCode::Char('a'));

        assert_eq!(input.get_buffer(), "a");
        assert_eq!(input.cursor_position(), 1);
    }

    #[test]
    fn test_validation_panel_renders_warnings() {
        let validation = ValidationResult {
            risk_level: RiskLevel::Moderate,
            warnings: vec!["Recursive deletion".to_string()],
            suggestions: vec![],
            matched_patterns: vec![],
        };

        let panel = ValidationPanel::new(ValidationProps {
            result: Some(validation),
            loading: false,
        });

        let rendered = panel.render_to_string();
        assert!(rendered.contains("⚠"));
        assert!(rendered.contains("Recursive deletion"));
    }
}
```

**State Tests:**
```rust
#[test]
fn test_app_state_handles_text_input() {
    let mut state = AppState::default();

    let effects = state.handle_event(AppEvent::TextInput('l'));

    assert_eq!(state.repl.input_buffer, "l");
    assert!(effects.contains(&SideEffect::TriggerValidation));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_command_generation_flow() {
    let mut app = TuiApp::new_for_test();

    // User types "ls"
    app.handle_event(AppEvent::TextInput('l')).await;
    app.handle_event(AppEvent::TextInput('s')).await;

    // User presses Enter
    app.handle_event(AppEvent::Enter).await;

    // Wait for generation
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Assert command generated
    assert!(app.state.repl.generated_command.is_some());
    assert_eq!(
        app.state.repl.generated_command.unwrap().command,
        "ls"
    );
}
```

### Visual Regression Tests

```rust
// Use ratatui-testing framework
#[test]
fn test_repl_component_visual() {
    let mut terminal = TestTerminal::new()?;
    let component = ReplComponent::new(/* ... */);

    terminal.draw(|frame| {
        component.render(frame, frame.size());
    })?;

    assert_snapshot!(terminal.backend().buffer());
}
```

---

## 📚 Contributor Guidelines

### Adding a New Component

**Checklist:**
1. [ ] Create component file in `src/tui/components/`
2. [ ] Implement `Component` trait
3. [ ] Define `Props` and `State` structs
4. [ ] Write unit tests
5. [ ] Add documentation with example
6. [ ] Update parent component to render new component
7. [ ] Test visually in terminal

**Example Component Template:**
```rust
// src/tui/components/my_component.rs

/// MyComponent - Brief description
///
/// # Example
/// ```
/// let component = MyComponent::new(MyComponentProps {
///     title: "Hello".to_string(),
/// });
/// ```
pub struct MyComponent {
    props: MyComponentProps,
    state: MyComponentState,
}

pub struct MyComponentProps {
    pub title: String,
}

struct MyComponentState {
    selected_index: usize,
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

    fn handle_event(&mut self, event: Event) -> Result<EventResult> {
        // Handle keyboard/mouse events
        Ok(EventResult::Ignored)
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Render using ratatui widgets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let component = MyComponent::new(MyComponentProps {
            title: "Test".to_string(),
        });

        assert_eq!(component.props.title, "Test");
    }
}
```

### Adding a New AppEvent

**Steps:**
1. Add variant to `AppEvent` enum in `src/tui/state/events.rs`
2. Implement handler in `AppState::handle_event()`
3. Document the event with /// comments
4. Add test for state transition
5. Update relevant components to emit the event

**Example:**
```rust
// In src/tui/state/events.rs
pub enum AppEvent {
    // ... existing events

    /// User pressed Ctrl+L to clear input
    ClearInput,
}

// In src/tui/state/app_state.rs
impl AppState {
    pub fn handle_event(&mut self, event: AppEvent) -> Vec<SideEffect> {
        match event {
            // ... existing handlers

            AppEvent::ClearInput => {
                self.repl.input_buffer.clear();
                self.repl.cursor_position = 0;
                self.repl.generated_command = None;
                vec![]  // No side effects
            }
        }
    }
}

// Test
#[test]
fn test_clear_input_event() {
    let mut state = AppState::default();
    state.repl.input_buffer = "hello".to_string();

    state.handle_event(AppEvent::ClearInput);

    assert_eq!(state.repl.input_buffer, "");
    assert_eq!(state.repl.cursor_position, 0);
}
```

---

## 🚀 Phase 1 Implementation Checklist

### Milestone 1: Foundation (Days 1-2)

- [ ] **Setup:**
  - [ ] Add dependencies to Cargo.toml
  - [ ] Create `src/tui/` module structure
  - [ ] Add `--tui` flag to main.rs

- [ ] **Terminal Management:**
  - [ ] Implement `setup_terminal()`
  - [ ] Implement `restore_terminal()`
  - [ ] Add panic handler for terminal cleanup

- [ ] **Basic Event Loop:**
  - [ ] Create event channel
  - [ ] Implement keyboard event polling
  - [ ] Add Quit event handling

### Milestone 2: Components (Days 3-4)

- [ ] **StatusBarComponent:**
  - [ ] Implement render function
  - [ ] Add backend status display
  - [ ] Add shell/safety level display
  - [ ] Write tests

- [ ] **HelpFooterComponent:**
  - [ ] Implement render function
  - [ ] Add dynamic shortcuts
  - [ ] Write tests

- [ ] **ReplComponent Shell:**
  - [ ] Create ReplComponent structure
  - [ ] Implement basic layout (3 panels)
  - [ ] Add to main app render

### Milestone 3: Input Handling (Days 5-6)

- [ ] **InputArea:**
  - [ ] Implement text input handling
  - [ ] Add cursor rendering
  - [ ] Implement backspace/delete
  - [ ] Add placeholder text
  - [ ] Write tests

### Milestone 4: Backend Integration (Days 7-8)

- [ ] **BackendBridge:**
  - [ ] Implement BackendBridge struct
  - [ ] Add generate_command() method
  - [ ] Add async handling
  - [ ] Write integration tests

- [ ] **ValidationPanel:**
  - [ ] Implement validation display
  - [ ] Add color-coded risk levels
  - [ ] Show warnings/suggestions
  - [ ] Write tests

- [ ] **CommandPreviewPanel:**
  - [ ] Implement preview rendering
  - [ ] Add loading state
  - [ ] Add error state
  - [ ] Show explanation
  - [ ] Write tests

### Milestone 5: Polish (Days 9-10)

- [ ] **Error Handling:**
  - [ ] Add error display
  - [ ] Graceful degradation
  - [ ] User-friendly messages

- [ ] **Performance:**
  - [ ] Measure startup time
  - [ ] Measure input latency
  - [ ] Optimize hot paths

- [ ] **Documentation:**
  - [ ] Update README with TUI usage
  - [ ] Add screenshots/GIFs
  - [ ] Document keyboard shortcuts
  - [ ] Write contributor guide

---

## 📸 Success Criteria

### Phase 1 MVP is complete when:

✅ **Functional:**
- User can type natural language input
- Command is generated from backend
- Validation shows risk assessment
- Command preview displays with explanation
- User can quit with Ctrl+C

✅ **Performance:**
- Startup time < 200ms
- Input latency < 50ms
- No blocking UI during generation

✅ **Quality:**
- All components have unit tests
- Integration test passes
- No clippy warnings
- Code formatted with rustfmt

✅ **Documentation:**
- All public APIs documented
- Contributor guide written
- Screenshots added to README

✅ **UX:**
- Beautiful, clean layout
- Clear visual feedback
- Intuitive keyboard shortcuts
- Helpful error messages

---

## 🔮 Future Phases

### Phase 2: History Browser
- SQLite integration
- Fuzzy search
- Filter/sort controls

### Phase 3: Configuration Editor
- Visual config editing
- Backend selection UI
- Settings validation

### Phase 4: Advanced Features
- @-tag file context
- Slash commands
- Checkpoint/rewind
- Command templates

---

## 📞 Support & Questions

**For Contributors:**
- Design questions: Check this HLD first
- Implementation questions: See ADR-001
- Code review: Create PR with screenshots

**Resources:**
- [Ratatui Examples](https://github.com/ratatui/ratatui/tree/main/examples)
- [TUI Guidelines](https://ratatui.rs/)
- [Project README](../README.md)

---

**Last Updated:** 2025-11-19
**Next Review:** After Milestone 1 completion
**Maintainer:** Architecture Team

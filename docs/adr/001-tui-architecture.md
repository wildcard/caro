# ADR-001: Terminal User Interface Architecture

**Status:** Accepted
**Date:** 2025-11-19
**Deciders:** Architecture Team
**Technical Story:** cmdai TUI Implementation

---

## Context and Problem Statement

cmdai currently exposes its powerful command generation, safety validation, and multi-backend inference capabilities through a traditional CLI interface. While functional, this approach:

1. **Hides functionality** - Users must know exact flags and options
2. **Lacks discoverability** - No way to explore features interactively
3. **Poor feedback loops** - Limited visual feedback during command generation
4. **No history management** - Users can't easily review or re-run previous commands
5. **Configuration complexity** - Editing TOML files is not user-friendly

**Goal:** Design and implement a Terminal User Interface (TUI) that:
- Exposes all cmdai functionality through an intuitive, keyboard-driven interface
- Provides real-time visual feedback during command generation
- Enables interactive exploration of features, history, and configuration
- Maintains the performance and reliability of the existing CLI
- Is contributor-friendly with clear architecture and documentation

---

## Decision Drivers

### Technical Requirements
- **Performance:** < 200ms startup, < 50ms input latency
- **Memory:** < 50MB idle, < 200MB during inference
- **Compatibility:** Support macOS, Linux, Windows terminals
- **Integration:** Seamless integration with existing CliApp architecture
- **Maintainability:** Clear separation of concerns, testable components

### User Experience Requirements
- **Discoverability:** All features visible and explorable
- **Responsiveness:** Non-blocking async operations
- **Visual clarity:** Color-coded safety levels, clear status indicators
- **Keyboard-first:** All actions accessible via keyboard shortcuts
- **Progressive disclosure:** Simple by default, advanced features accessible

### Development Requirements
- **Contributor-friendly:** Clear architecture, well-documented
- **Extensibility:** Easy to add new modes and components
- **Testability:** Unit and integration tests for all components
- **Type safety:** Leverage Rust's type system for correctness

---

## Considered Options

### Option 1: Immediate Mode UI with Ratatui

**Description:**
Build TUI using Ratatui (modern fork of tui-rs) with immediate mode rendering pattern. Components render on every frame based on current state.

**Architecture:**
```
Event Loop → State Update → Render → Terminal
     ↑                                    ↓
     └────────────────────────────────────┘
```

**Pros:**
- ✅ Simple mental model - state drives rendering
- ✅ Active development and strong community
- ✅ Excellent async support via crossterm
- ✅ Rich widget ecosystem (tables, lists, trees, text input)
- ✅ Proven in production (helix, bottom, gitui, atuin)
- ✅ Fine-grained control over rendering
- ✅ Easy to reason about state changes

**Cons:**
- ⚠️ Manual render optimization needed for performance
- ⚠️ More boilerplate for complex interactions

**Example Projects:**
- **gitui** - Git TUI with complex state management
- **bottom** - System monitor with real-time updates
- **atuin** - Shell history with fuzzy search

---

### Option 2: Component-Based Framework (Cursive)

**Description:**
Use Cursive framework which provides a higher-level component model with event callbacks and automatic layout.

**Architecture:**
```
Component Tree → Event Handlers → Automatic Rerender
```

**Pros:**
- ✅ Higher-level abstractions
- ✅ Built-in layout engine
- ✅ Component reusability
- ✅ Automatic rendering on state change

**Cons:**
- ❌ Less active development than Ratatui
- ❌ Harder to customize rendering
- ❌ More opinionated architecture
- ❌ Smaller ecosystem
- ❌ Async integration more complex

---

### Option 3: Web-Based Terminal UI (Bubbletea-style)

**Description:**
Port concepts from Charm's Bubbletea (Go) - Elm architecture with Model-Update-View pattern.

**Architecture:**
```
Init → Update(Msg) → View → Terminal
  ↑         ↓
  └─────────┘
```

**Pros:**
- ✅ Clean functional architecture
- ✅ Predictable state management
- ✅ Easy to test (pure functions)

**Cons:**
- ❌ No direct Rust equivalent
- ❌ Would require building custom framework
- ❌ More implementation work
- ❌ Unfamiliar to Rust developers

---

### Option 4: Terminal Web Server (Textual-style)

**Description:**
Embed a web server and use browser-based rendering in terminal.

**Pros:**
- ✅ Rich styling capabilities
- ✅ Familiar web technologies

**Cons:**
- ❌ Massive overhead for a CLI tool
- ❌ Violates "single binary" goal
- ❌ Poor terminal integration
- ❌ Not suitable for this use case

---

## Decision Outcome

**Chosen Option: Option 1 - Immediate Mode UI with Ratatui**

### Rationale

Ratatui is the clear winner for cmdai's TUI implementation because:

1. **Battle-Tested:** Used by major projects (helix, bottom, gitui, atuin) proving production readiness
2. **Active Community:** 4.8k+ stars, active development, responsive maintainers
3. **Performance:** Immediate mode rendering gives us fine control over performance
4. **Async-First:** Excellent integration with tokio (we already use tokio)
5. **Rust-Idiomatic:** Leverages Rust's type system and ownership model
6. **Rich Ecosystem:** Mature widgets for all our needs (tables, text input, trees)
7. **Documentation:** Excellent docs, examples, and tutorials

### Architecture Pattern: Component-Based Immediate Mode

While Ratatui is immediate mode, we'll impose a **component-based architecture** on top:

```rust
// Component trait for all UI elements
trait Component {
    type Props;
    type State;

    fn new(props: Self::Props) -> Self;
    fn handle_event(&mut self, event: Event) -> Result<EventResult>;
    fn update(&mut self, state: &AppState) -> Result<()>;
    fn render(&self, frame: &mut Frame, area: Rect);
}
```

**Benefits:**
- Clear separation of concerns
- Reusable components
- Easy to test in isolation
- Familiar pattern for contributors

---

## Implementation Architecture

### High-Level Structure

```
┌──────────────────────────────────────────────────────────┐
│                        TuiApp                             │
│  - Owns AppState                                          │
│  - Runs event loop                                        │
│  - Coordinates components                                 │
└────────────────┬─────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
   ┌────▼─────┐    ┌─────▼──────┐
   │ Terminal │    │   Events   │
   │ Renderer │    │  Handler   │
   └────┬─────┘    └─────┬──────┘
        │                │
   ┌────▼────────────────▼─────┐
   │      Component Tree       │
   │                           │
   │  ┌─────────────────────┐  │
   │  │   ReplComponent     │  │
   │  ├─────────────────────┤  │
   │  │ HistoryComponent    │  │
   │  ├─────────────────────┤  │
   │  │ ConfigComponent     │  │
   │  ├─────────────────────┤  │
   │  │ StatusBarComponent  │  │
   │  └─────────────────────┘  │
   └───────────────────────────┘
```

### State Management Pattern

**Single Source of Truth with Message Passing:**

```rust
// Central application state
pub struct AppState {
    current_mode: AppMode,
    input_buffer: String,
    generated_command: Option<GeneratedCommand>,
    validation_result: Option<ValidationResult>,
    history: Vec<HistoryEntry>,
    config: UserConfiguration,
    // ... more state
}

// Events that drive state changes
pub enum AppEvent {
    // User input
    KeyPress(KeyEvent),
    TextInput(char),

    // Backend events
    CommandGenerated(GeneratedCommand),
    ValidationComplete(ValidationResult),
    HistoryLoaded(Vec<HistoryEntry>),

    // Mode changes
    SwitchMode(AppMode),
}

// Event handling pattern
impl TuiApp {
    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::TextInput(c) => {
                self.state.input_buffer.push(c);
                self.trigger_live_validation().await?;
            }
            AppEvent::CommandGenerated(cmd) => {
                self.state.generated_command = Some(cmd);
                self.trigger_validation().await?;
            }
            // ... more handlers
        }
        Ok(())
    }
}
```

---

## State Management: Redux-Inspired Pattern

### Why Redux-Style?

1. **Predictable:** All state changes go through well-defined actions
2. **Debuggable:** Easy to log and replay state transitions
3. **Testable:** Pure reducer functions
4. **Familiar:** Many developers know this pattern

### Implementation

```rust
// State reducers
pub fn app_reducer(state: &AppState, event: &AppEvent) -> AppState {
    match event {
        AppEvent::TextInput(c) => {
            let mut new_state = state.clone();
            new_state.input_buffer.push(*c);
            new_state
        }
        // ... more reductions
    }
}

// Side effects handled separately
pub async fn handle_side_effect(
    event: &AppEvent,
    cli_app: &mut CliApp,
) -> Result<Vec<AppEvent>> {
    match event {
        AppEvent::GenerateCommandRequested => {
            let cmd = cli_app.generate_command(/* ... */).await?;
            Ok(vec![AppEvent::CommandGenerated(cmd)])
        }
        // ... more side effects
    }
}
```

---

## Component Design Pattern

### Reusable Component Structure

Each component follows this pattern:

```rust
// src/tui/components/input.rs
pub struct InputComponent {
    props: InputProps,
    state: InputState,
}

pub struct InputProps {
    placeholder: String,
    max_length: Option<usize>,
    on_submit: Box<dyn Fn(String) -> AppEvent>,
}

struct InputState {
    cursor_position: usize,
    selection: Option<Range<usize>>,
}

impl Component for InputComponent {
    type Props = InputProps;
    type State = InputState;

    fn new(props: Self::Props) -> Self {
        Self {
            props,
            state: InputState::default(),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<EventResult> {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                // Handle input...
                Ok(EventResult::Consumed)
            }
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                let event = (self.props.on_submit)(self.get_value());
                Ok(EventResult::Event(event))
            }
            _ => Ok(EventResult::Ignored)
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let input_widget = Paragraph::new(self.get_display_text())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(input_widget, area);

        // Render cursor
        let cursor_x = area.x + self.state.cursor_position as u16 + 1;
        frame.set_cursor(cursor_x, area.y + 1);
    }
}
```

---

## Event Handling Architecture

### Async Event Loop

```rust
// Main event loop
pub async fn run(mut app: TuiApp) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let (event_tx, mut event_rx) = mpsc::channel(100);

    // Spawn input handler
    let input_handle = tokio::spawn(async move {
        loop {
            if let Ok(event) = crossterm::event::read() {
                event_tx.send(AppEvent::from(event)).await?;
            }
        }
    });

    // Main loop
    loop {
        // Render current state
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Handle events
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match app.handle_event(event).await {
                    Ok(EventResult::Quit) => break,
                    Ok(_) => continue,
                    Err(e) => {
                        app.show_error(e);
                    }
                }
            }
            // Handle background tasks
            result = app.poll_backend() => {
                if let Some(event) = result? {
                    app.handle_event(event).await?;
                }
            }
        }
    }

    cleanup_terminal()?;
    Ok(())
}
```

---

## Integration with Existing Code

### Principle: Wrap, Don't Rewrite

```rust
// src/tui/backend_bridge.rs
pub struct BackendBridge {
    cli_app: CliApp,
    pending_requests: HashMap<RequestId, oneshot::Sender<CliResult>>,
}

impl BackendBridge {
    pub async fn generate_command(
        &mut self,
        input: String,
    ) -> Result<CliResult> {
        // Delegate to existing CliApp
        let args = Cli {
            prompt: Some(input),
            shell: Some(self.get_current_shell()),
            safety: Some(self.get_safety_level()),
            // ... more args
        };

        self.cli_app.run_with_args(args).await
    }
}
```

**Key Integration Points:**

1. **Backend Communication** - Reuse entire `backends/` module
2. **Safety Validation** - Reuse `safety/` module directly
3. **Configuration** - Reuse `config/` module
4. **Models** - Shared data types from `models/`

**Benefits:**
- No code duplication
- Consistent behavior between CLI and TUI
- Single source of truth for business logic
- Easy to maintain

---

## Component Hierarchy

### Phase 1 MVP Components

```
TuiApp
├── TerminalManager        (crossterm setup/cleanup)
├── EventHandler           (keyboard/mouse events)
├── AppState              (central state store)
└── Components
    ├── ReplComponent      (main REPL interface)
    │   ├── InputComponent (text input widget)
    │   ├── PreviewComponent (command preview)
    │   └── ValidationPanel (safety feedback)
    ├── StatusBarComponent (top status bar)
    └── HelpFooter        (keyboard shortcuts)
```

### Future Components (Post-MVP)

```
├── HistoryComponent
│   ├── HistoryTable
│   ├── SearchBar
│   └── FilterControls
├── ConfigComponent
│   ├── ConfigTree
│   └── ValueEditor
└── ModalComponent
    ├── ConfirmDialog
    └── ErrorDialog
```

---

## Rendering Strategy

### Immediate Mode with Smart Diffing

Ratatui handles terminal diffing automatically, but we optimize by:

1. **Selective Rendering:** Only render changed components
2. **Lazy Evaluation:** Defer expensive computations
3. **Buffered Updates:** Batch rapid state changes

```rust
impl TuiApp {
    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Status bar
                Constraint::Min(0),     // Main content
                Constraint::Length(1),  // Help footer
            ])
            .split(frame.area());

        // Only render if state changed
        if self.state.status_changed {
            self.status_bar.render(frame, chunks[0]);
        }

        // Render active mode
        match self.state.current_mode {
            AppMode::Repl => self.repl.render(frame, chunks[1]),
            AppMode::History => self.history.render(frame, chunks[1]),
            // ... more modes
        }

        self.help_footer.render(frame, chunks[2]);
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_component_handles_text() {
        let mut input = InputComponent::new(InputProps::default());

        let result = input.handle_event(
            Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))
        ).unwrap();

        assert_eq!(input.get_value(), "a");
        assert_eq!(result, EventResult::Consumed);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_command_generation_flow() {
    let mut app = TuiApp::new_for_test();

    // Simulate user typing
    app.handle_event(AppEvent::TextInput('l')).await.unwrap();
    app.handle_event(AppEvent::TextInput('s')).await.unwrap();
    app.handle_event(AppEvent::KeyPress(KeyCode::Enter)).await.unwrap();

    // Wait for command generation
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(app.state.generated_command.is_some());
    assert_eq!(app.state.generated_command.unwrap().command, "ls");
}
```

---

## Performance Optimizations

### 1. Lazy State Updates

```rust
pub struct AppState {
    input_buffer: String,

    // Cached/derived state
    #[cached]
    syntax_highlighted: Option<String>,
    #[cached]
    validation_result: Option<ValidationResult>,
}

impl AppState {
    fn invalidate_caches(&mut self) {
        self.syntax_highlighted = None;
        self.validation_result = None;
    }
}
```

### 2. Debounced Validation

```rust
// Don't validate on every keystroke
async fn handle_text_input(&mut self, c: char) {
    self.state.input_buffer.push(c);

    // Cancel previous validation
    self.validation_debounce.cancel();

    // Schedule new validation after 200ms
    self.validation_debounce.schedule(Duration::from_millis(200), || {
        self.trigger_validation()
    });
}
```

### 3. Async Background Tasks

```rust
// Non-blocking command generation
async fn generate_command(&mut self) -> Result<()> {
    let input = self.state.input_buffer.clone();

    // Spawn background task
    let handle = tokio::spawn(async move {
        backend.generate_command(input).await
    });

    self.pending_generations.push(handle);

    // Show loading indicator immediately
    self.state.loading = true;

    Ok(())
}
```

---

## Alternative Considered: Elm Architecture

### Why Not Elm Architecture?

**Pros:**
- Pure functional approach
- Predictable state management
- Easy to test

**Cons:**
- ❌ Less idiomatic in Rust
- ❌ More boilerplate
- ❌ Harder for contributors to understand
- ❌ No clear Rust library support

**Verdict:** Redux-inspired is more pragmatic for Rust

---

## Alternative Considered: Actor Model

### Why Not Actor Model?

**Pros:**
- Natural async communication
- Isolation of components

**Cons:**
- ❌ Overkill for single-user TUI
- ❌ Message passing overhead
- ❌ Harder to debug
- ❌ More complex state management

**Verdict:** Direct function calls are simpler

---

## Decision Matrix

| Criterion | Ratatui | Cursive | Custom Elm | Web-Based |
|-----------|---------|---------|------------|-----------|
| **Community** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐⭐ |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| **Async Support** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Ease of Use** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Customization** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Contributor-Friendly** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **Binary Size** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ |
| **Total** | **33/35** | **26/35** | **23/35** | **20/35** |

---

## Consequences

### Positive

✅ **Clear Architecture:** Component-based design is easy to understand
✅ **Contributor-Friendly:** Well-documented patterns, familiar to Rust developers
✅ **Performance:** Fine control over rendering and state updates
✅ **Extensibility:** Easy to add new modes and components
✅ **Reusability:** Existing CliApp logic is preserved and reused
✅ **Testability:** Clear boundaries for unit and integration tests

### Negative

⚠️ **Boilerplate:** More code than higher-level frameworks
⚠️ **Manual Optimization:** Need to implement debouncing, caching manually

### Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Performance degradation with large history | Implement virtual scrolling, pagination |
| State management complexity | Clear documentation, well-defined patterns |
| Cross-platform terminal differences | Comprehensive testing matrix |
| Component coupling | Strict component interfaces, props passing |

---

## Implementation Guidelines

### For Contributors

1. **Component Checklist:**
   - [ ] Implements `Component` trait
   - [ ] Has Props and State structs
   - [ ] Handles events appropriately
   - [ ] Renders efficiently
   - [ ] Has unit tests
   - [ ] Documented with examples

2. **State Change Checklist:**
   - [ ] Event type defined
   - [ ] Reducer function implemented
   - [ ] Side effects handled
   - [ ] State transition tested

3. **PR Requirements:**
   - [ ] Components documented
   - [ ] Tests pass
   - [ ] No performance regressions
   - [ ] Mockup/screenshot provided

---

## References

### Inspiration Projects
- [gitui](https://github.com/extrawurst/gitui) - Git TUI with excellent architecture
- [bottom](https://github.com/ClementTsang/bottom) - System monitor with real-time updates
- [atuin](https://github.com/atuinsh/atuin) - Shell history with fuzzy search

### Documentation
- [Ratatui Book](https://ratatui.rs/)
- [Crossterm Docs](https://docs.rs/crossterm)
- [Async Rust](https://rust-lang.github.io/async-book/)

### Related ADRs
- ADR-002: TUI Component Library (Future)
- ADR-003: History Database Schema (Future)
- ADR-004: Keyboard Shortcuts (Future)

---

## Approval

- [x] Technical Lead
- [x] UX Review
- [x] Performance Review
- [x] Security Review

**Approved for implementation - Phase 1 Start Date: 2025-11-19**

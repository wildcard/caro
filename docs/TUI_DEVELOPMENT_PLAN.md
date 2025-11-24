# cmdai TUI Development Plan
## Ratatui Terminal User Interface

**Version:** 1.0
**Date:** 2025-11-19
**Status:** Planning Phase

---

## 🎯 Vision & Inspiration

Build a modern, interactive Terminal User Interface for cmdai that combines the best features from:

### 1. **Claude Code TUI** - Context & Control
- ✨ **@-tags for context**: Quickly tag files and directories to provide context
- 🎯 **Slash commands**: Menu-driven command system with autocomplete
- ⏮️ **Checkpoint/Rewind**: Esc-Esc to rewind to previous states
- 📜 **Searchable history**: Ctrl+r for fuzzy search through command history
- 🔍 **Status visibility**: Clear visual feedback on current state and operations
- 🎛️ **Interactive configuration**: Visual menus for settings management

### 2. **Atuin** - Smart History Management
- 🗄️ **SQLite-backed history**: Persistent, searchable command database
- 📊 **Rich context storage**: Working directory, exit codes, duration, timestamps
- 🔎 **Multi-mode filtering**: Session, directory, host, global search modes
- 🔍 **Fuzzy/full-text search**: Intelligent command discovery
- 🎨 **Full-screen search UI**: Immersive search experience with Ctrl+r
- 📈 **Statistics & insights**: Command frequency, success rates, timing analytics

### 3. **Finance TUI Apps** (beancount-tui style) - Clean Data Presentation
- 📋 **Table/list views**: Clear, organized data presentation
- ✏️ **Inline editing**: Direct manipulation of list items
- 🎨 **Syntax highlighting**: Color-coded content for readability
- 📊 **Metadata displays**: Rich information in compact format
- ⌨️ **Keyboard-first navigation**: Vim-like bindings for power users
- 🎯 **Focus indicators**: Clear visual feedback on selected items

---

## 🏗️ Architecture Overview

### Mode System

```
┌─────────────────────────────────────────────────────────┐
│                     cmdai TUI Modes                      │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  1. REPL Mode (Default)                                  │
│     └─ Interactive prompt with live validation           │
│                                                           │
│  2. History Browser Mode                                 │
│     └─ Full-screen searchable command history            │
│                                                           │
│  3. Configuration Mode                                   │
│     └─ Interactive settings editor                       │
│                                                           │
│  4. Model Selection Mode                                 │
│     └─ Backend/model picker with availability status     │
│                                                           │
│  5. Safety Validation Preview Mode                       │
│     └─ Live pattern matching and risk assessment         │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### Component Architecture

```rust
// Core TUI Application
TuiApp {
    // State Management
    current_mode: AppMode,
    history_db: SqliteHistoryStore,
    session: TuiSession,

    // UI Components
    repl: ReplComponent,
    history_browser: HistoryBrowserComponent,
    config_editor: ConfigEditorComponent,
    model_selector: ModelSelectorComponent,
    safety_preview: SafetyPreviewComponent,

    // Shared Services (from existing codebase)
    cli_app: CliApp,              // Reuse existing backend/safety/config
    event_handler: EventHandler,  // Keyboard/mouse events
    renderer: TerminalRenderer,   // Ratatui rendering
}
```

---

## 🎨 Key Features Breakdown

### Feature 1: Interactive REPL Mode

**Inspired by:** Claude Code + Atuin

**Visual Layout:**
```
╭─ cmdai REPL ──────────────────────────────────────────────╮
│ Session: 2025-11-19 14:30 | Backend: Ollama | Safety: ✓  │
├───────────────────────────────────────────────────────────┤
│                                                            │
│ 🤖 cmdai> find all python files modified today_          │
│                                                            │
│ ┌─ Live Validation ────────────────────────────────────┐  │
│ │ ✓ POSIX compliant                                    │  │
│ │ ✓ Safe command pattern                               │  │
│ │ ⚠ Will search entire home directory (may be slow)    │  │
│ └──────────────────────────────────────────────────────┘  │
│                                                            │
│ ┌─ Suggested Command ──────────────────────────────────┐  │
│ │ find . -type f -name "*.py" -mtime -1                │  │
│ │                                                       │  │
│ │ Explanation: Search current directory for Python     │  │
│ │ files modified in the last 24 hours                  │  │
│ └──────────────────────────────────────────────────────┘  │
│                                                            │
│ [Enter] Execute  [Tab] Edit  [↑↓] History  [Ctrl+R] Search│
╰────────────────────────────────────────────────────────────╯
```

**Components:**
- **Input Area**: Multi-line text input with syntax awareness
- **Live Validation Panel**: Real-time safety checks as you type
- **Preview Panel**: Generated command with explanation
- **Status Bar**: Current context (shell, backend, safety level)
- **Help Footer**: Keyboard shortcuts

**Key Interactions:**
- `Enter`: Generate command from natural language
- `Ctrl+Enter`: Execute directly without confirmation
- `Tab`: Switch to edit mode for command refinement
- `Ctrl+R`: Open history search
- `Esc Esc`: Rewind to previous command state
- `@filename`: Tag files for context (autocomplete with fuzzy match)
- `/command`: Execute slash command with menu

---

### Feature 2: Full-Screen History Browser

**Inspired by:** Atuin + Finance TUI data views

**Visual Layout:**
```
╭─ Command History (523 commands) ──────────────────────────╮
│ Filter: [Session] Directory  Host  Global    Sort: Recent │
├─────────┬──────────────────────────────────────────────────┤
│         │                                                   │
│  14:25  │ ✓ find . -name "*.rs" | xargs wc -l             │
│   45s   │   /home/user/cmdai  [0]  Backend: Ollama        │
│         │                                                   │
│  14:20  │ ✗ docker-compose up -d                          │
│   2.1s  │   /home/user/project  [1]  Backend: vLLM        │
│         │                                                   │
│> 14:15  │ ✓ git status                                    │◄ Selected
│   0.3s  │   /home/user/cmdai  [0]  Backend: Embedded      │
│         │                                                   │
│  14:10  │ ✓ cargo test safety::test_dangerous_patterns    │
│   12s   │   /home/user/cmdai  [0]  Backend: Mock          │
│         │                                                   │
│  14:05  │ ⚠ chmod -R 755 ./scripts                        │
│   0.1s  │   /home/user/cmdai  [0]  Backend: Ollama        │
│         │   Warning: Recursive permission change          │
│         │                                                   │
├─────────┴──────────────────────────────────────────────────┤
│ Search: docker_                                            │
│                                                            │
│ [↑↓] Navigate  [Enter] Copy  [E] Edit  [R] Re-run  [/] Filter│
╰────────────────────────────────────────────────────────────╯
```

**Features:**
- **Multi-column table**: Time, Duration, Status, Command, Context
- **Status indicators**: ✓ (success), ✗ (failure), ⚠ (warnings)
- **Inline metadata**: Directory, exit code, backend used
- **Fuzzy search**: Type to filter in real-time
- **Filter modes**: Toggle between session/directory/host/global
- **Sort options**: Recent, Frequent, Duration, Success rate

**Database Schema:**
```rust
// SQLite table structure
CREATE TABLE command_history (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    command TEXT NOT NULL,
    natural_language_input TEXT,
    shell_type TEXT,
    working_directory TEXT,
    exit_code INTEGER,
    duration_ms INTEGER,
    backend_used TEXT,
    safety_level TEXT,
    risk_assessment TEXT,
    warnings TEXT,  -- JSON array
    session_id TEXT,
    hostname TEXT,
    user TEXT
);

CREATE INDEX idx_timestamp ON command_history(timestamp DESC);
CREATE INDEX idx_command ON command_history(command);
CREATE INDEX idx_directory ON command_history(working_directory);
```

---

### Feature 3: Interactive Configuration Editor

**Inspired by:** Claude Code's /hooks menu

**Visual Layout:**
```
╭─ Settings ─────────────────────────────────────────────────╮
│                                                             │
│  General                                                    │
│  ├─ Default Shell ·········· [bash ▼]                      │
│  ├─ Safety Level ··········· [Moderate ▼]                  │
│  └─ Output Format ·········· [Plain ▼]                     │
│                                                             │
│  Model & Backend                                            │
│  ├─ Primary Backend ········ [Ollama ▼]                    │
│  │                           Status: ✓ Available           │
│  ├─ Fallback Backend ······· [Embedded (CPU) ▼]           │
│  └─ Default Model ·········· [qwen2.5-coder:7b]           │
│                                                             │
│  History & Privacy                                          │
│  ├─ Enable History ········· [✓] ON                        │
│  ├─ Sync History ··········· [ ] OFF                       │
│  ├─ Retention Days ········· [90 days]                     │
│  └─ Exclude Patterns ······· [Edit...]                     │
│                                                             │
│  Advanced                                                   │
│  ├─ Log Level ·············· [Info ▼]                      │
│  ├─ Cache Size Limit ······· [10 GB]                       │
│  └─ Performance Mode ······· [✓] ON                        │
│                                                             │
│  [S] Save  [R] Reset to Defaults  [Esc] Cancel             │
╰─────────────────────────────────────────────────────────────╯
```

**Features:**
- Tree-based navigation of settings
- Live validation of configuration values
- Dropdowns for enum values
- Toggle switches for booleans
- Inline editing for text/numbers
- Visual feedback on unsaved changes
- Backend availability status checks

---

### Feature 4: Model/Backend Selection Interface

**Visual Layout:**
```
╭─ Select Backend ───────────────────────────────────────────╮
│                                                             │
│  Available Backends                                         │
│                                                             │
│  ● Ollama (Local)                        ✓ Ready           │
│    └─ localhost:11434                                      │
│       Models: qwen2.5-coder:7b, llama3.2:3b                │
│       Latency: ~1.2s  •  Memory: 4.2GB                     │
│                                                             │
│  ○ vLLM (Remote)                         ✗ Offline         │
│    └─ localhost:8000                                       │
│       Last seen: 2 hours ago                                │
│                                                             │
│  ○ Embedded (MLX - Apple Silicon)        ✓ Ready           │
│    └─ Local model cache                                    │
│       Models: Qwen/Qwen2.5-Coder-1.5B-Instruct             │
│       Latency: ~2.0s  •  Memory: 1.8GB                     │
│                                                             │
│  ○ Embedded (CPU - Candle)               ✓ Ready           │
│    └─ Local model cache                                    │
│       Models: Qwen/Qwen2.5-Coder-1.5B-Instruct             │
│       Latency: ~8.5s  •  Memory: 2.1GB                     │
│                                                             │
│  [Enter] Select  [T] Test Backend  [D] Download Model      │
╰─────────────────────────────────────────────────────────────╯
```

**Features:**
- Real-time backend availability checking
- Performance metrics (latency, memory usage)
- Model listing per backend
- Test connection functionality
- Model download interface

---

### Feature 5: Safety Validation Preview

**Inspired by:** Live feedback from Claude Code + cmdai's existing safety system

**Visual Layout:**
```
╭─ Safety Validation ────────────────────────────────────────╮
│                                                             │
│  Command: rm -rf ./old_logs                                │
│                                                             │
│  Risk Assessment                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Risk Level: MODERATE ⚠                              │   │
│  │ Confidence: 87%                                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Matched Patterns                                           │
│  • rm_rf_pattern (Priority: High)                          │
│    "Recursive deletion with rm -rf"                        │
│                                                             │
│  Warnings                                                   │
│  ⚠ This command will recursively delete files              │
│  ⚠ Target path is relative - verify working directory      │
│  ℹ Consider using: rm -ri ./old_logs (interactive mode)    │
│                                                             │
│  Alternatives                                               │
│  1. mv ./old_logs ~/.trash/old_logs_2025-11-19             │
│  2. tar -czf old_logs_backup.tar.gz ./old_logs && rm -rf...│
│  3. find ./old_logs -type f -mtime +30 -delete             │
│                                                             │
│  Safety Level: Moderate → Requires confirmation: YES       │
│                                                             │
│  [Enter] Confirm  [E] Edit  [Alt+N] View Alternative  [Esc]│
╰─────────────────────────────────────────────────────────────╯
```

---

## 🔧 Technical Implementation

### Dependencies to Add

```toml
# Cargo.toml additions
[dependencies]
# TUI Framework
ratatui = "0.30"              # Modern TUI framework
crossterm = "0.28"            # Terminal manipulation
tui-input = "0.10"            # Text input widgets

# Enhanced UI Components
tui-tree-widget = "0.24"      # Tree views for config
tui-textarea = "0.7"          # Multi-line text editing

# Database for History
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"                  # Connection pooling
r2d2_sqlite = "0.25"          # SQLite pool

# Fuzzy Search
nucleo = "0.5"                # Fast fuzzy matching (used by Helix editor)

# Additional Utilities
chrono = "0.4"                # Timestamp formatting
unicode-width = "0.2"         # Text width calculations
```

### Project Structure (New Files)

```
src/
├── tui/
│   ├── mod.rs                    # TUI module root
│   ├── app.rs                    # Main TuiApp struct and event loop
│   ├── modes/
│   │   ├── mod.rs
│   │   ├── repl.rs               # REPL mode implementation
│   │   ├── history.rs            # History browser mode
│   │   ├── config.rs             # Configuration editor mode
│   │   ├── model_selector.rs    # Backend/model selection
│   │   └── safety_preview.rs    # Safety validation preview
│   ├── components/
│   │   ├── mod.rs
│   │   ├── input.rs              # Enhanced input widget
│   │   ├── command_preview.rs   # Generated command display
│   │   ├── status_bar.rs        # Status/context bar
│   │   ├── help_footer.rs       # Keyboard shortcuts
│   │   ├── table.rs             # History table widget
│   │   ├── tree.rs              # Configuration tree widget
│   │   └── modal.rs             # Confirmation dialogs
│   ├── history/
│   │   ├── mod.rs
│   │   ├── db.rs                # SQLite operations
│   │   ├── models.rs            # History entry structs
│   │   └── search.rs            # Fuzzy search implementation
│   ├── state/
│   │   ├── mod.rs
│   │   ├── session.rs           # Session state management
│   │   ├── checkpoint.rs        # Undo/redo system
│   │   └── context.rs           # @-tag context tracking
│   ├── events/
│   │   ├── mod.rs
│   │   ├── handler.rs           # Event processing
│   │   └── keys.rs              # Keybinding definitions
│   └── render/
│       ├── mod.rs
│       ├── repl.rs              # REPL mode rendering
│       ├── history.rs           # History mode rendering
│       ├── config.rs            # Config mode rendering
│       └── theme.rs             # Color schemes
├── main.rs                       # Update to support --tui flag
└── lib.rs                        # Export TUI modules
```

### Integration with Existing Code

**Key Integration Points:**

1. **Reuse CliApp** - Don't rewrite, wrap it:
```rust
pub struct TuiApp {
    cli_app: CliApp,  // Existing backend/safety/config logic
    tui_state: TuiState,
    history_store: HistoryStore,
}

impl TuiApp {
    pub async fn generate_command(&mut self, input: &str) -> Result<CliResult> {
        // Delegate to existing CliApp
        self.cli_app.run_with_args(/* ... */).await
    }
}
```

2. **History Integration**:
```rust
// After each command execution
async fn execute_command(&mut self, result: CliResult) -> Result<()> {
    let exit_code = self.shell_execute(&result.generated_command)?;

    // Store in SQLite history
    self.history_store.insert(HistoryEntry {
        command: result.generated_command,
        natural_input: self.current_input.clone(),
        exit_code,
        duration: result.timing_info.total_duration,
        backend: result.generation_details,
        // ... more fields
    }).await?;
}
```

3. **Safety Preview Live Updates**:
```rust
// As user types in REPL
async fn on_input_change(&mut self, input: &str) {
    if let Ok(cmd) = self.cli_app.backend.generate_command(/*...*/).await {
        // Validate using existing SafetyValidator
        let validation = self.cli_app.validator.validate_command(&cmd.command, shell);
        self.safety_preview_state.update(validation);
        self.trigger_render();
    }
}
```

---

## 📅 Implementation Phases

### Phase 1: Foundation (Week 1-2)
**Goal:** Basic TUI shell with REPL mode

**Tasks:**
- [ ] Add ratatui, crossterm dependencies
- [ ] Create basic TUI app structure (`src/tui/`)
- [ ] Implement event loop with keyboard handling
- [ ] Build simple REPL mode with text input
- [ ] Add `--tui` flag to main.rs
- [ ] Render status bar and help footer
- [ ] Test on macOS/Linux terminals

**Deliverable:** `cmdai --tui` launches basic interactive prompt

---

### Phase 2: Command Generation Integration (Week 3)
**Goal:** Connect REPL to existing CliApp backend

**Tasks:**
- [ ] Integrate CliApp into TuiApp
- [ ] Implement async command generation from REPL input
- [ ] Add command preview panel
- [ ] Show live validation status (safe/moderate/high/critical)
- [ ] Implement confirmation modal for dangerous commands
- [ ] Add basic error handling and display

**Deliverable:** Full command generation workflow in TUI

---

### Phase 3: History System (Week 4)
**Goal:** SQLite-backed command history

**Tasks:**
- [ ] Design and create SQLite schema
- [ ] Implement HistoryStore with rusqlite
- [ ] Add history recording after command execution
- [ ] Create basic history browser mode (list view)
- [ ] Implement Ctrl+R keybinding to open history
- [ ] Add copy-to-clipboard from history

**Deliverable:** Persistent command history with basic browsing

---

### Phase 4: Enhanced History Features (Week 5)
**Goal:** Atuin-inspired search and filtering

**Tasks:**
- [ ] Integrate nucleo for fuzzy search
- [ ] Implement real-time search filtering
- [ ] Add filter modes (session/directory/host/global)
- [ ] Implement sort options (recent/frequent/duration)
- [ ] Add statistics view (most used commands, success rates)
- [ ] Improve history table UI with colors and icons

**Deliverable:** Full-featured history browser with search

---

### Phase 5: Configuration & Model Selection (Week 6)
**Goal:** Interactive settings management

**Tasks:**
- [ ] Build configuration editor mode (tree widget)
- [ ] Implement backend/model selection interface
- [ ] Add real-time backend availability checking
- [ ] Create model download progress UI
- [ ] Implement settings validation and persistence
- [ ] Add visual indicators for unsaved changes

**Deliverable:** Visual configuration management

---

### Phase 6: Advanced Features (Week 7-8)
**Goal:** Claude Code-inspired enhancements

**Tasks:**
- [ ] Implement checkpoint/rewind system (Esc-Esc)
- [ ] Add @-tag system for file context
- [ ] Build slash command menu with autocomplete
- [ ] Create safety validation preview mode
- [ ] Add alternative command suggestions
- [ ] Implement keyboard shortcuts help screen
- [ ] Add themes/color scheme support

**Deliverable:** Feature-complete TUI

---

### Phase 7: Polish & Optimization (Week 9)
**Goal:** Production-ready quality

**Tasks:**
- [ ] Performance optimization (async rendering)
- [ ] Comprehensive error handling
- [ ] Memory leak testing and fixes
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Accessibility improvements (screen readers)
- [ ] Add telemetry/analytics (opt-in)
- [ ] Write user documentation

**Deliverable:** Production-ready TUI

---

## 🎯 Success Metrics

### Performance Targets
- **Startup time**: < 200ms (TUI overhead)
- **Input latency**: < 50ms (keystroke to render)
- **Search performance**: < 100ms for 10K+ history entries
- **Memory usage**: < 50MB idle, < 200MB active

### User Experience Goals
- **Keyboard-first**: All actions accessible via keyboard
- **Visual clarity**: Color-coded risk levels, clear status indicators
- **Discoverability**: Built-in help, tooltips, shortcuts guide
- **Responsiveness**: Async operations don't block UI

---

## 🚀 Quick Start Commands (After Implementation)

```bash
# Launch TUI mode
cmdai --tui

# Launch directly into history browser
cmdai --tui --mode history

# Launch with specific backend
cmdai --tui --backend ollama

# Launch with custom config
cmdai --tui --config ~/.config/cmdai/custom.toml
```

---

## 🔮 Future Enhancements (Post-MVP)

### Advanced Features
- **Command templates**: Save frequently used prompts
- **Macros**: Record and replay command sequences
- **Remote sync**: Encrypted history sync across machines (Atuin-style)
- **AI suggestions**: Predict next command based on history
- **Plugins**: Lua/WASM plugin system for customization
- **Multiplexer integration**: tmux/screen awareness
- **Git integration**: Show repo status, suggest git commands
- **Multi-pane layout**: Split views for concurrent operations

### Visualization Enhancements
- **Command graphs**: Visualize command relationships
- **Timeline view**: Calendar heatmap of command usage
- **Performance charts**: Command execution time trends
- **Sparklines**: Inline charts for metrics

---

## 📚 References

### Inspiration Sources
- **Claude Code TUI**: https://claude.com/product/claude-code
- **Atuin**: https://github.com/atuinsh/atuin
- **Ratatui Examples**: https://github.com/ratatui/ratatui/tree/main/examples
- **beancount-tui**: https://lib.rs/crates/beancount-tui

### Technical Resources
- **Ratatui Book**: https://ratatui.rs/
- **Crossterm Docs**: https://docs.rs/crossterm
- **rusqlite Guide**: https://docs.rs/rusqlite

---

## 📝 Notes

### Design Decisions

1. **Why SQLite for history?**
   - Proven reliability and performance
   - Built-in full-text search (FTS5)
   - No external dependencies
   - Easy backup/export

2. **Why Ratatui over other TUI frameworks?**
   - Active development (tui-rs successor)
   - Large ecosystem of widgets
   - Excellent async support
   - Strong community

3. **Why reuse CliApp instead of rewriting?**
   - DRY principle - avoid duplicating backend logic
   - Maintains consistency with CLI mode
   - Easier to maintain and test
   - Gradual migration path

### Open Questions

- [ ] Should history sync be included in MVP or Phase 2?
- [ ] What's the best UX for @-tag file selection? (fuzzy finder vs tree view)
- [ ] Should we support mouse interactions or keyboard-only?
- [ ] How to handle very long command outputs in TUI?

---

**Last Updated:** 2025-11-19
**Next Review:** After Phase 1 completion

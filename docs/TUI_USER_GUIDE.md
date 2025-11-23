# cmdai TUI User Guide

**Version:** 1.0.0 (Phase 1 MVP - REPL Mode)
**Last Updated:** 2025-11-19
**Status:** Active Development

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Interface Components](#interface-components)
4. [Keyboard Shortcuts](#keyboard-shortcuts)
5. [Modes](#modes)
6. [Workflow Examples](#workflow-examples)
7. [Troubleshooting](#troubleshooting)
8. [Tips and Tricks](#tips-and-tricks)

---

## Introduction

### What is the TUI Mode?

The cmdai TUI (Terminal User Interface) is an interactive, full-screen terminal application that provides a beautiful, keyboard-driven interface for generating shell commands from natural language. Instead of typing commands and flags, you can launch cmdai in TUI mode and interact with an intuitive visual interface that guides you through command generation, validation, and execution.

### Why Use TUI Mode vs CLI Mode?

**Choose TUI Mode When:**
- You want a more interactive, exploratory experience
- You prefer visual feedback and live validation
- You're learning cmdai's features and capabilities
- You want to see command explanations and safety warnings clearly
- You need to generate multiple commands in a session

**Choose CLI Mode When:**
- You want to script cmdai or use it in automation
- You need a single command quickly (one-shot usage)
- You're integrating cmdai with other tools via pipes
- You prefer traditional command-line workflows

**TUI Mode Advantages:**
- Real-time validation feedback as you type
- Clear visual risk assessment with color coding
- No need to remember CLI flags - everything is visible
- Persistent session with easy command iteration
- Discoverable keyboard shortcuts
- Beautiful, clean interface that reduces cognitive load

### Visual Preview

```
╭─ cmdai ──────────────────────────────────────────────────────────────╮
│ ⚙ Ollama • bash • Moderate Safety                          [?] Help │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│ ┌─ Input ─────────────────────────────────────────────────────────┐ │
│ │ find all python files modified today_                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│ ┌─ Validation ─────────────────────────────────────────────────────┐ │
│ │ ✓ SAFE                                                           │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│ ┌─ Generated Command ─────────────────────────────────────────────┐ │
│ │ find . -type f -name "*.py" -mtime -1                            │ │
│ │                                                                  │ │
│ │ 💡 Searches current directory for Python files modified in       │ │
│ │    the last 24 hours                                             │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
├──────────────────────────────────────────────────────────────────────┤
│ [Enter] Generate  [Ctrl+L] Clear  [Ctrl+C] Quit  [?] Help           │
╰──────────────────────────────────────────────────────────────────────╯
```

---

## Getting Started

### Launching the TUI

The simplest way to start the TUI is:

```bash
cargo run -- --tui
```

Or, if you have cmdai installed:

```bash
cmdai --tui
```

### First-Time User Experience

When you launch the TUI for the first time, you'll see:

1. **Status Bar** (top) - Shows your current backend, shell, and safety level
2. **Input Area** - A text box with a placeholder prompt inviting you to type
3. **Validation Panel** - Shows "Ready" until you start typing
4. **Command Preview Panel** - Shows helpful text: "Start typing to generate a command..."
5. **Help Footer** (bottom) - Displays the most important keyboard shortcuts

**Your First Command:**

1. Type a natural language description of what you want to do
   Example: `list all files`

2. Press `Enter` to generate a command

3. The TUI will:
   - Show a loading indicator ("⏳ Generating command...")
   - Contact the backend (Ollama, vLLM, or embedded model)
   - Display the generated command with an explanation
   - Show validation results with risk assessment

4. Review the generated command and decide whether to copy or execute it

### Navigation Overview

The TUI is designed to be **keyboard-first**. All functionality is accessible via keyboard shortcuts (no mouse required, though mouse support may be added in future versions).

**Basic Navigation Pattern:**
1. **Type** → Natural language input in the input area
2. **Enter** → Generate a command from your input
3. **Review** → Check the validation panel and command preview
4. **Act** → Copy/execute the command (in future versions)
5. **Repeat** → Clear input (`Ctrl+L`) and start again

---

## Interface Components

### Status Bar

**Location:** Top of the screen (1 line)

**Purpose:** Displays current TUI state and configuration at a glance

**Visual Layout:**
```
⚙ Ollama • bash • Moderate Safety                          [?] Help
└─┬──┘   └─┬─┘   └───────┬──────┘                          └───┬──┘
  │        │             │                                      │
Backend  Shell    Safety Level                          Help Indicator
```

**Elements:**

1. **Backend Indicator** (`⚙ Ollama`)
   - Shows which backend is currently active
   - Color coding:
     - **Cyan** = Backend available and ready
     - **Red** = Backend unavailable or error
   - Possible values: `Ollama`, `vLLM`, `Embedded (MLX)`, `Embedded (CPU)`, `Mock`

2. **Shell Type** (`bash`)
   - The target shell for generated commands
   - Always shown in **Green**
   - Possible values: `bash`, `zsh`, `fish`, `sh`, `powershell`

3. **Safety Level** (`Moderate Safety`)
   - Current safety configuration
   - Color coding:
     - **Red** = Strict (blocks many commands)
     - **Yellow** = Moderate (warns about risky commands)
     - **Green** = Permissive (minimal restrictions)
   - Possible values: `Strict Safety`, `Moderate Safety`, `Permissive Safety`

4. **Help Indicator** (`[?] Help`)
   - Reminder that you can press `?` for more help (future feature)

### Input Area

**Location:** Top of main content area (4 lines)

**Purpose:** Where you type your natural language command description

**Visual States:**

**Empty (Placeholder):**
```
┌─ Input ─────────────────────────────────────────┐
│ 🤖 Type your command in natural language...    │
│                                                 │
└─────────────────────────────────────────────────┘
```

**With Text:**
```
┌─ Input ─────────────────────────────────────────┐
│ find all python files modified today_           │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Features:**
- Multi-line support (automatically wraps long input)
- Blinking cursor shows current position
- Placeholder text disappears when you start typing
- Supports all standard text editing (backspace, delete, arrow keys in future)

**Interaction:**
- Type any character to insert it
- `Backspace` to delete character before cursor
- `Delete` to delete character at cursor
- `Ctrl+L` to clear all input
- `Enter` to generate command from current input

### Validation Panel

**Location:** Middle of main content area (3 lines)

**Purpose:** Shows real-time safety validation of generated commands

**Visual States:**

**Idle:**
```
┌─ Validation ────────────────────────────────────┐
│ Ready                                           │
└─────────────────────────────────────────────────┘
```

**Loading:**
```
┌─ Validation ────────────────────────────────────┐
│ ⏳ Validating...                                │
└─────────────────────────────────────────────────┘
```

**Safe Command:**
```
┌─ Validation ────────────────────────────────────┐
│ ✓ SAFE                                          │
└─────────────────────────────────────────────────┘
```

**Moderate Risk:**
```
┌─ Validation ────────────────────────────────────┐
│ ⚠ MODERATE                                      │
└─────────────────────────────────────────────────┘
```

**High Risk:**
```
┌─ Validation ────────────────────────────────────┐
│ ❌ HIGH                                          │
└─────────────────────────────────────────────────┘
```

**Critical Risk:**
```
┌─ Validation ────────────────────────────────────┐
│ 🛑 CRITICAL                                      │
└─────────────────────────────────────────────────┘
```

**Risk Level Icons:**
- ✓ (Green) = Safe - No dangerous patterns detected
- ⚠ (Yellow) = Moderate - Some caution advised
- ❌ (Red) = High - Potentially dangerous operation
- 🛑 (Red, Bold) = Critical - Very dangerous, likely blocked

### Command Preview Panel

**Location:** Bottom of main content area (flexible height, minimum 5 lines)

**Purpose:** Displays the generated shell command with explanation

**Visual States:**

**Idle (No Input):**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│    Start typing to generate a command...         │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Loading:**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│    ⏳ Generating command...                      │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Success:**
```
┌─ Generated Command ──────────────────────────────┐
│ find . -type f -name "*.py" -mtime -1            │
│                                                  │
│ 💡 Searches current directory for Python files   │
│    modified in the last 24 hours                 │
└──────────────────────────────────────────────────┘
```

**Error:**
```
┌─ Generated Command ──────────────────────────────┐
│                                                  │
│  ❌ Error: Backend unavailable                   │
│     Check Ollama is running on localhost:11434   │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Features:**
- Generated command shown in white text (top)
- Blank line for spacing
- Explanation with 💡 icon (helps you understand what the command does)
- Color coding:
  - **White** = Success
  - **Red** = Error
  - **Gray** = Placeholder/idle

### Help Footer

**Location:** Bottom of the screen (1 line)

**Purpose:** Shows context-sensitive keyboard shortcuts

**Visual Layout:**
```
[Enter] Generate  [Ctrl+L] Clear  [Ctrl+C] Quit  [?] Help
└──┬──┘ └───┬───┘  └──┬───┘ └──┬──┘  └──┬───┘ └─┬─┘  └─┬─┘ └──┬──┘
  Key  Description   Key  Description  Key  Description Key Desc
```

**Color Scheme:**
- `[Brackets]` = Dark Gray
- `Keys` = Cyan, Bold
- `Descriptions` = White

**Shortcuts change based on mode** - Currently only REPL mode is implemented (see [Modes](#modes) for future mode shortcuts)

---

## Keyboard Shortcuts

### Complete Keyboard Reference

#### Global Shortcuts (Work in All Modes)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+C` | Quit | Exit the TUI application immediately |

#### REPL Mode Shortcuts (Current Implementation)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `a-z`, `0-9`, etc. | Type Character | Insert character at cursor position |
| `Space` | Insert Space | Insert a space character |
| `Backspace` | Delete Before | Delete character before cursor |
| `Delete` | Delete At | Delete character at cursor position |
| `Enter` | Generate Command | Generate shell command from current input |
| `Ctrl+L` | Clear Input | Clear the entire input buffer |
| `Ctrl+C` | Quit | Exit the TUI application |

#### Future Shortcuts (Planned Features)

| Shortcut | Action | Description | Status |
|----------|--------|-------------|--------|
| `Ctrl+R` | History | Open command history browser | Planned |
| `?` | Help | Show help modal | Planned |
| `Ctrl+Enter` | Execute | Generate and execute command directly | Planned |
| `Tab` | Autocomplete | Show suggestions (context-aware) | Planned |
| `↑` | History Back | Previous input from history | Planned |
| `↓` | History Forward | Next input from history | Planned |
| `Esc` | Back/Cancel | Return to REPL mode | Planned |
| `←` | Cursor Left | Move cursor left | Planned |
| `→` | Cursor Right | Move cursor right | Planned |

### Keyboard Shortcuts by Function

#### Text Editing
```
Type characters       →  Insert text
Backspace            →  Delete before cursor
Delete               →  Delete at cursor
Ctrl+L               →  Clear all input
```

#### Command Generation
```
Enter                →  Generate command from input
Ctrl+Enter (future)  →  Generate and execute immediately
```

#### Navigation
```
Ctrl+C               →  Quit application
Esc (future)         →  Back to REPL mode
Ctrl+R (future)      →  Open history browser
```

#### Help & Discovery
```
? (future)           →  Show help modal
```

---

## Modes

The cmdai TUI is designed with multiple modes to handle different tasks. **Currently, only REPL mode is implemented** (Phase 1 MVP). Future modes are planned and documented here for completeness.

### 1. REPL Mode (Current)

**Status:** ✅ Implemented (Phase 1 MVP)

**Purpose:** Interactive command generation with live validation

**When to Use:**
- Generating new commands from natural language
- Iterating on command descriptions
- Learning how cmdai interprets requests
- Checking safety validation before execution

**Keyboard Shortcuts:**
- `Enter` - Generate command
- `Ctrl+L` - Clear input
- `Ctrl+C` - Quit

**Layout:**
```
┌────────────────────────────────┐
│ Status Bar                     │
├────────────────────────────────┤
│ Input Area (4 lines)           │
├────────────────────────────────┤
│ Validation Panel (3 lines)     │
├────────────────────────────────┤
│ Command Preview (flexible)     │
├────────────────────────────────┤
│ Help Footer                    │
└────────────────────────────────┘
```

### 2. History Mode (Planned)

**Status:** 🚧 Planned for Phase 2

**Purpose:** Browse and search previously generated commands

**When to Use:**
- Finding a command you used before
- Analyzing command patterns over time
- Re-running successful commands
- Reviewing command history with rich context

**Planned Features:**
- SQLite-backed persistent history
- Fuzzy search across all commands
- Filter by session, directory, host, or global
- Sort by recency, frequency, duration, or success rate
- Rich metadata: exit code, duration, timestamp, working directory
- Copy or re-run commands from history

**Planned Keyboard Shortcuts:**
- `↑↓` - Navigate through history entries
- `Enter` - Copy selected command
- `/` - Search/filter
- `Esc` - Return to REPL mode

**Visual Preview:**
```
╭─ Command History (523 commands) ──────────────────────╮
│ Filter: [Session] Directory  Host  Global   Sort: Recent │
├─────────┬──────────────────────────────────────────────┤
│         │                                               │
│  14:25  │ ✓ find . -name "*.rs" | xargs wc -l         │
│   45s   │   /home/user/cmdai  [0]  Backend: Ollama    │
│         │                                               │
│> 14:15  │ ✓ git status                                │◄ Selected
│   0.3s  │   /home/user/cmdai  [0]  Backend: Embedded  │
│         │                                               │
├─────────┴──────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Copy  [/] Search  [Esc] Back   │
╰────────────────────────────────────────────────────────╯
```

### 3. Config Mode (Planned)

**Status:** 🚧 Planned for Phase 3

**Purpose:** Interactive configuration editor

**When to Use:**
- Changing safety level
- Switching default shell
- Selecting backend/model
- Adjusting TUI preferences
- Managing history settings

**Planned Features:**
- Tree-based navigation of settings
- Visual backend availability checks
- Live validation of config values
- Dropdown menus for enums
- Toggle switches for booleans
- Unsaved changes indicator

**Planned Keyboard Shortcuts:**
- `↑↓` - Navigate settings
- `Enter` - Edit selected setting
- `S` - Save changes
- `R` - Reset to defaults
- `Esc` - Cancel (discard changes)

**Visual Preview:**
```
╭─ Settings ─────────────────────────────────────────╮
│                                                     │
│  General                                            │
│  ├─ Default Shell ·········· [bash ▼]              │
│  ├─ Safety Level ··········· [Moderate ▼]          │
│  └─ Output Format ·········· [Plain ▼]             │
│                                                     │
│  Model & Backend                                    │
│  ├─ Primary Backend ········ [Ollama ▼]            │
│  │                           Status: ✓ Available   │
│  └─ Default Model ·········· [qwen2.5-coder:7b]   │
│                                                     │
│  [S] Save  [R] Reset  [Esc] Cancel                 │
╰─────────────────────────────────────────────────────╯
```

### 4. Help Mode (Planned)

**Status:** 🚧 Planned for Phase 4

**Purpose:** Comprehensive in-app documentation

**When to Use:**
- Learning keyboard shortcuts
- Understanding risk levels
- Reading about features
- Troubleshooting issues

**Planned Features:**
- Searchable help content
- Keyboard shortcut reference
- Feature explanations
- Quick tips and tutorials
- Links to external documentation

**Planned Keyboard Shortcuts:**
- `↑↓` - Scroll help content
- `/` - Search help
- `Esc` - Close help
- `Q` - Quit help

---

## Workflow Examples

### Example 1: Simple Command Generation

**Goal:** Generate a command to list all files in the current directory

**Steps:**

1. **Launch the TUI**
   ```bash
   cmdai --tui
   ```

2. **Type your request**
   - Focus is automatically in the input area
   - Type: `list all files`
   - You'll see your text appear in the Input panel

3. **Generate the command**
   - Press `Enter`
   - The Command Preview panel shows: "⏳ Generating command..."
   - After 1-2 seconds, you'll see the generated command

4. **Review the result**
   ```
   ┌─ Generated Command ──────────────────────────────┐
   │ ls -la                                            │
   │                                                  │
   │ 💡 Lists all files in the current directory,     │
   │    including hidden files, in long format        │
   └──────────────────────────────────────────────────┘
   ```

5. **Check validation**
   ```
   ┌─ Validation ─────────────────────────────────────┐
   │ ✓ SAFE                                           │
   └──────────────────────────────────────────────────┘
   ```

6. **Copy and execute** (manual for now, will be automated in future)
   - Copy the command: `ls -la`
   - Quit the TUI: `Ctrl+C`
   - Execute in your shell: `ls -la`

### Example 2: Handling Validation Warnings

**Goal:** Generate a command that triggers a safety warning

**Steps:**

1. **Type a potentially risky command**
   - Input: `delete all temporary files recursively`

2. **Press Enter to generate**
   - The backend generates: `rm -rf /tmp/*`

3. **Review validation warning**
   ```
   ┌─ Validation ─────────────────────────────────────┐
   │ ⚠ MODERATE                                       │
   └──────────────────────────────────────────────────┘
   ```

4. **Read the command carefully**
   ```
   ┌─ Generated Command ──────────────────────────────┐
   │ rm -rf /tmp/*                                    │
   │                                                  │
   │ 💡 Recursively deletes all files in /tmp         │
   │    directory. This is an irreversible operation. │
   └──────────────────────────────────────────────────┘
   ```

5. **Decide on action**
   - **If acceptable:** Copy and execute with caution
   - **If too risky:** Press `Ctrl+L` to clear and try a different description
   - **Alternative:** Rephrase as "show temporary files before deleting"

### Example 3: Iterating on Command Descriptions

**Goal:** Refine your input to get a better command

**Steps:**

1. **First attempt**
   - Input: `find files`
   - Generated: `find .`
   - Result: Too broad, finds everything

2. **Clear and refine**
   - Press `Ctrl+L` to clear input
   - Input: `find python files`
   - Generated: `find . -name "*.py"`
   - Result: Better, but finds all Python files

3. **Clear and refine again**
   - Press `Ctrl+L`
   - Input: `find python files modified today`
   - Generated: `find . -type f -name "*.py" -mtime -1`
   - Result: Perfect! Exactly what you needed

**Tip:** The TUI makes iteration fast and visual. Don't worry about getting it right the first time.

### Example 4: Understanding Error Messages

**Goal:** Handle backend errors gracefully

**Steps:**

1. **Simulate an error** (backend offline)
   - Type: `list files`
   - Press `Enter`

2. **See the error**
   ```
   ┌─ Generated Command ──────────────────────────────┐
   │                                                  │
   │  ❌ Error: Backend unavailable                   │
   │     Check Ollama is running on localhost:11434   │
   │                                                  │
   └──────────────────────────────────────────────────┘
   ```

3. **Check status bar**
   ```
   ⚙ Ollama • bash • Moderate Safety
   └─ Red color indicates backend is unavailable
   ```

4. **Resolve the issue**
   - Exit TUI with `Ctrl+C`
   - Start your backend: `ollama serve` (for Ollama)
   - Restart TUI: `cmdai --tui`
   - Try again

### Example 5: Multi-Line Input (Future Feature)

**Note:** This is currently limited but will be enhanced in future versions.

**Goal:** Generate a complex command with detailed description

**Steps:**

1. **Type a detailed description**
   - Input: `find all python files modified in the last week, sort by size, and show only the largest 10`

2. **Current behavior**
   - Text wraps within the 4-line input area
   - You can see about 200 characters comfortably

3. **Future enhancement**
   - Input area will expand up to 10 lines
   - Scroll support for very long descriptions
   - Multi-paragraph input support

---

## Troubleshooting

### Terminal Compatibility

**Problem:** TUI displays garbled characters or incorrect layout

**Solution:**

1. **Check your terminal emulator**
   - Recommended: Alacritty, kitty, iTerm2, Windows Terminal, GNOME Terminal
   - Not recommended: Very old xterm versions, basic TTY

2. **Verify Unicode support**
   ```bash
   echo $LANG
   # Should show something like: en_US.UTF-8
   ```
   If not, set it:
   ```bash
   export LANG=en_US.UTF-8
   export LC_ALL=en_US.UTF-8
   ```

3. **Check terminal size**
   - Minimum recommended: 80 columns × 24 rows
   - Optimal: 120 columns × 40 rows
   - Check current size: `tput cols; tput lines`

4. **Test emoji support**
   ```bash
   echo "🤖 ✓ ⚠ ❌ 🛑 💡 ⏳"
   ```
   If these don't render, your terminal may have limited Unicode support.

**Known Issues:**
- tmux users: Add `set -g default-terminal "screen-256color"` to `.tmux.conf`
- SSH sessions: Ensure `TERM` environment variable is preserved
- Screen sessions: May have limited color support

### Display Issues

**Problem:** Colors don't appear or look wrong

**Solution:**

1. **Verify 256-color support**
   ```bash
   echo $TERM
   # Should show: xterm-256color, screen-256color, or similar
   ```

2. **Test color output**
   ```bash
   curl -s https://gist.githubusercontent.com/HaleTom/89ffe32783f89f403bba96bd7bcd1263/raw/ | bash
   ```

3. **Force color support**
   ```bash
   export TERM=xterm-256color
   cmdai --tui
   ```

**Problem:** Screen doesn't refresh properly

**Solution:**

1. **Terminal resize issue**
   - Resize your terminal window slightly
   - The TUI should automatically redraw

2. **Force quit if frozen**
   - Press `Ctrl+C` (may need to press multiple times)
   - If that fails, press `Ctrl+Z` to suspend, then `kill %1`

3. **Terminal cleanup after crash**
   ```bash
   reset
   ```

**Problem:** Cursor is in the wrong place

**Solution:**

- This is typically a rendering timing issue
- Try typing a character, it should snap back
- If persistent, report as a bug (see [BUILD.md](../BUILD.md))

### Keyboard Shortcuts Not Working

**Problem:** `Ctrl+C` doesn't quit

**Solution:**

1. **Check for keybinding conflicts**
   - Some terminal multiplexers (tmux, screen) intercept `Ctrl+C`
   - Try `Ctrl+C` multiple times
   - Alternative: Close the terminal window

2. **Terminal emulator issues**
   - VS Code integrated terminal: Some shortcuts may be captured
   - Solution: Use an external terminal

**Problem:** Special keys don't work (arrows, delete, etc.)

**Solution:**

1. **Terminal compatibility**
   - Some terminals send different codes for special keys
   - Currently, only `Backspace` and `Delete` are fully implemented
   - Arrow keys are planned for future versions

2. **SSH sessions**
   - Ensure your `TERM` variable is correctly set
   - Try: `export TERM=xterm-256color`

### Backend Connection Issues

**Problem:** "Backend unavailable" error

**Solution:**

1. **Check backend status**
   - For Ollama: `curl http://localhost:11434/api/tags`
   - For vLLM: `curl http://localhost:8000/health`

2. **Start your backend**
   ```bash
   # For Ollama
   ollama serve

   # For vLLM (example)
   python -m vllm.entrypoints.api_server --model <model-name>
   ```

3. **Verify configuration**
   - Check `~/.config/cmdai/config.toml`
   - Ensure backend URLs are correct

4. **Check firewall**
   - Ensure localhost ports are accessible
   - Try: `telnet localhost 11434` (for Ollama)

**Problem:** Slow command generation

**Solution:**

1. **Backend performance**
   - Local backends (Ollama, embedded) should respond in 1-3 seconds
   - Remote backends may take 3-5 seconds
   - If slower, check backend logs for issues

2. **Model size**
   - Larger models (7B+) are slower but more accurate
   - Smaller models (1.5B-3B) are faster but may need more refined input
   - Consider switching models if speed is critical

### General Issues

**Problem:** TUI won't start

**Solution:**

1. **Check build**
   ```bash
   cargo build --release
   # Check for errors
   ```

2. **Verify configuration**
   ```bash
   cmdai --help
   # Should show --tui flag
   ```

3. **Check dependencies**
   - See [BUILD.md](../BUILD.md) for system requirements
   - Ensure Rust toolchain is up to date

**Problem:** Panic or crash

**Solution:**

1. **Terminal should be restored automatically**
   - If not, run: `reset`

2. **Report the issue**
   - Check `~/.config/cmdai/logs/` for error details
   - File an issue with the panic message
   - Include: OS, terminal emulator, Rust version

---

## Tips and Tricks

### Power User Workflows

#### Quick Iteration Cycle
```
Type → Enter → Review → Ctrl+L → Repeat
```
This cycle lets you rapidly try different phrasings until you get the perfect command.

#### Descriptive Input Works Better
Instead of: `find files`
Try: `find all python files modified in the last 24 hours`

The more context you provide, the better the generated command.

#### Learn from Explanations
Read the 💡 explanations in the Command Preview panel. They help you:
- Understand what flags do
- Learn POSIX command patterns
- Build your shell knowledge over time

#### Use Clear Input for Fresh Start
`Ctrl+L` is your friend. Use it liberally to clear input and start fresh without quitting the TUI.

### Efficiency Tips

#### Keep the TUI Open
Instead of launching cmdai repeatedly, keep the TUI open in a dedicated terminal window or tmux pane. This saves startup time and keeps your session state.

#### Combine with CLI Mode
Use TUI mode for exploration and learning, then switch to CLI mode for automation once you know what you need:

```bash
# Learn in TUI
cmdai --tui

# Automate in CLI
cmdai "find all python files" | sh
```

#### Watch the Status Bar
The status bar tells you important context at a glance:
- Backend availability (don't waste time typing if backend is down)
- Current shell (ensure commands are compatible)
- Safety level (understand what will be validated)

### Customization Hints (Future)

**Current:** Configuration is loaded from `~/.config/cmdai/config.toml`

**Future Features (Planned):**

1. **Themes**
   - Light/dark mode toggle
   - Custom color schemes
   - Accessibility-friendly palettes

2. **Layout Options**
   - Adjustable panel heights
   - Side-by-side vs stacked layouts
   - Hide/show panels

3. **Keyboard Remapping**
   - Vim-style bindings
   - Emacs-style bindings
   - Custom key mappings

4. **Auto-Execution**
   - Trust mode: automatically execute safe commands
   - Clipboard integration: auto-copy generated commands
   - History auto-save: background history recording

### Learning Resources

#### Understanding Risk Levels

- **Safe (✓ Green):** Go ahead and execute
  - Examples: `ls`, `pwd`, `git status`, `cat file.txt`

- **Moderate (⚠ Yellow):** Review carefully
  - Examples: `rm file.txt`, `chmod 755 script.sh`, `curl | bash` from trusted source

- **High (❌ Red):** Think twice
  - Examples: `rm -rf ./large-directory`, `dd` operations, system-wide changes

- **Critical (🛑 Red, Bold):** Usually blocked
  - Examples: `rm -rf /`, `mkfs`, fork bombs

#### Common Patterns

**File Operations:**
- "find files" → `find .`
- "find python files" → `find . -name "*.py"`
- "find large files" → `find . -type f -size +100M`
- "delete empty directories" → `find . -type d -empty -delete`

**Git Commands:**
- "show git status" → `git status`
- "commit changes" → `git commit -m "message"`
- "show uncommitted changes" → `git diff`

**System Info:**
- "show disk usage" → `df -h`
- "show memory usage" → `free -h`
- "show running processes" → `ps aux`
- "show network connections" → `netstat -tuln`

**Text Processing:**
- "count lines in file" → `wc -l file.txt`
- "search for pattern" → `grep "pattern" file.txt`
- "replace text in file" → `sed -i 's/old/new/g' file.txt`

### Keyboard Maestro

Memorize these core shortcuts for maximum efficiency:

```
Essential Trio:
  Enter   → Generate command
  Ctrl+L  → Clear input (fresh start)
  Ctrl+C  → Quit

That's it! Everything else is typing.
```

### Quality of Life

#### Terminal Size Matters
- Larger terminal = More context visible
- Recommended: Full-screen or half-screen
- Minimum: 80x24, but 120x40 is much nicer

#### Use Good Lighting
If your terminal background is very dark or very light, adjust your terminal theme for comfortable long-term use.

#### Take Breaks
The TUI makes it easy to get into a flow state of rapid iteration. Remember to take breaks and test your commands in a safe environment!

---

## What's Next?

### Upcoming Features (Roadmap)

**Phase 2: History Browser** (Coming Soon)
- Persistent command history with SQLite
- Fuzzy search through all your commands
- Filter by date, directory, success rate
- Re-run commands from history

**Phase 3: Configuration Editor** (Planned)
- Visual settings editor
- Backend/model selection
- Safety level adjustment
- Customization options

**Phase 4: Advanced Features** (Future)
- @-tag file context (like Claude Code)
- Slash commands for quick actions
- Command templates and macros
- Checkpoint/rewind system

### Get Involved

- **Report Bugs:** File issues on GitHub
- **Request Features:** Open a feature request
- **Contribute:** See [BUILD.md](../BUILD.md) for contributor guidelines
- **Share Feedback:** Tell us what you love or what needs improvement

### Documentation

- **Technical Design:** [docs/hld/TUI_HIGH_LEVEL_DESIGN.md](hld/TUI_HIGH_LEVEL_DESIGN.md)
- **Development Plan:** [docs/TUI_DEVELOPMENT_PLAN.md](TUI_DEVELOPMENT_PLAN.md)
- **Build Instructions:** [docs/BUILD.md](../BUILD.md)
- **Main README:** [README.md](../README.md)

---

## Quick Reference Card

### Essential Commands
```
Launch TUI:        cargo run -- --tui  or  cmdai --tui
Type to input:     Just start typing
Generate command:  Enter
Clear input:       Ctrl+L
Quit:             Ctrl+C
```

### Visual Legend
```
⚙  = Backend indicator
✓  = Safe (green)
⚠  = Moderate risk (yellow)
❌ = High risk (red)
🛑 = Critical risk (blocked)
💡 = Explanation
⏳ = Loading
🤖 = Input prompt
```

### Status Bar Colors
```
Cyan   = Backend available / Primary actions
Green  = Shell type / Safe commands
Yellow = Moderate safety / Warnings
Red    = Backend unavailable / High risk / Errors
Gray   = Inactive / Placeholder
```

### Common Workflows
```
1. Simple generation:
   Type → Enter → Copy → Execute

2. Iteration:
   Type → Enter → Review → Ctrl+L → Rephrase → Enter

3. Error recovery:
   See error → Check status bar → Fix backend → Retry
```

---

**Version:** 1.0.0 (Phase 1 MVP - REPL Mode)
**Last Updated:** 2025-11-19
**Feedback:** File issues on GitHub or see BUILD.md for contact info

**Happy commanding! 🚀**

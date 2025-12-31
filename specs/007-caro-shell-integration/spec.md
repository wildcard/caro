# Caro Shell Integration - High-Level Design Document

## Executive Summary

This specification defines the architecture and implementation strategy for deeply integrating Caro into existing popular shells (bash, zsh, sh/POSIX, fish) rather than building a new shell. The goal is to provide "shell + Caro" experiences where users keep their familiar shell with Caro pre-baked into the interactive session.

## Goals

### Primary Goals

1. **Seamless Shell Integration**: Users can "switch" to a Caro-enabled shell session without learning a new shell
2. **Zero-Friction Default**: Normal commands behave exactly as before - Caro is present but not intrusive
3. **Deep Bindings**: Enable capabilities impossible with standalone CLI:
   - Pre-execution command observation and intervention
   - Post-execution analysis and suggestions
   - Typo detection and correction ("thefuck"-like)
   - Inline UI and suggestions
   - Configurable safety guardrails
4. **Cross-Shell Consistency**: Provide consistent UX across bash, zsh, sh, and fish
5. **Performance**: No noticeable latency on prompt rendering or command execution
6. **Dotfiles Compatibility**: Integrate without breaking existing shell configurations

### Secondary Goals

1. **Discoverability**: First-run onboarding and help system
2. **Extensibility**: Plugin architecture for custom behaviors
3. **Privacy-First**: User-controlled data capture with sensible defaults
4. **Offline Operation**: Core functionality works without network

### Non-Goals

1. **Reimplementing shell languages**: We use native shell constructs
2. **Breaking normal behavior**: All standard shell operations work unchanged
3. **Requiring dotfile rewrites**: Single include line, no config migration
4. **Automatic mode by default**: Caro assists when asked, doesn't control
5. **Complex TUI on v1**: Start with simple TTY prompts, advanced UI later

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     User's Interactive Shell                      │
│                   (bash / zsh / sh / fish)                       │
├──────────────────────────────────────────────────────────────────┤
│                    Shell Integration Layer                        │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐       │
│  │  bash.init  │  zsh.init   │  sh.init    │  fish.init  │       │
│  └──────┬──────┴──────┬──────┴──────┬──────┴──────┬──────┘       │
│         │             │             │             │               │
│         ▼             ▼             ▼             ▼               │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                   Event Capture Layer                     │    │
│  │  • preexec / precmd hooks                                │    │
│  │  • keybinding handlers                                    │    │
│  │  • exit status capture                                    │    │
│  │  • timing information                                     │    │
│  └────────────────────────┬─────────────────────────────────┘    │
└───────────────────────────┼──────────────────────────────────────┘
                            │
                            │ IPC (Unix Socket)
                            │ ~/.config/caro/caro.sock
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Caro Runtime (Daemon)                         │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Session Manager                          │  │
│  │  • Per-session context                                     │  │
│  │  • Command history                                         │  │
│  │  • Working directory tracking                              │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌──────────────┬───────────────┬───────────────────────────────┐│
│  │ Safety       │ Suggestion    │ Fix-It Engine                 ││
│  │ Module       │ Engine        │ (thefuck-like)                ││
│  │              │               │                               ││
│  │ • Pattern    │ • Typo        │ • Exit code analysis          ││
│  │   matching   │   detection   │ • Error pattern matching      ││
│  │ • Risk       │ • Command     │ • Correction database         ││
│  │   scoring    │   completion  │ • LLM fallback                ││
│  │ • Policy     │ • Context-    │                               ││
│  │   actions    │   aware hints │                               ││
│  └──────────────┴───────────────┴───────────────────────────────┘│
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    LLM Integration                          │  │
│  │  • Existing caro backends (MLX, vLLM, Ollama)              │  │
│  │  • Natural language command suggestions                    │  │
│  │  • Error explanation                                       │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Shell Integration Layer (Per-Shell Scripts)

Lightweight shell scripts that install hooks and communicate with the Caro runtime:

- **bash.init**: Uses `PROMPT_COMMAND`, `DEBUG` trap, `bind -x`
- **zsh.init**: Uses `preexec`, `precmd`, ZLE widgets, `bindkey`
- **sh.init**: Portable mode with minimal hooks (PS1-based)
- **fish.init**: Native fish functions, `fish_preexec`, `fish_postexec`

### 2. Caro Runtime Daemon

Long-running process that:
- Maintains per-session state
- Processes events from shell integration layer
- Executes policy decisions
- Provides UI rendering

### 3. IPC Layer

Communication between shell integration and runtime:
- Unix domain socket at `~/.config/caro/caro.sock`
- JSON-based message protocol
- Timeout-based degradation (if runtime unavailable, shell works normally)

### 4. Policy Engine

Configurable rules for safety and assistance:
- **Off**: Pure assist-on-request mode
- **Passive**: Warn on high-risk patterns only
- **Active**: Confirm/warn/block for risky operations

## Data Flow

### Pre-Execution Flow
```
User types command
       │
       ▼
preexec hook fires
       │
       ▼
Shell layer captures:
• command line
• cwd
• timestamp
       │
       ▼
Send to Caro runtime via IPC
       │
       ▼
Runtime evaluates:
• Safety patterns
• Risk scoring
• Policy rules
       │
       ▼
Response: { allow: bool, warnings: [], transform?: string }
       │
       ▼
Shell layer:
• Shows warnings
• Requests confirmation (if needed)
• Allows/blocks execution
```

### Post-Execution Flow
```
Command completes
       │
       ▼
postcmd hook fires
       │
       ▼
Shell layer captures:
• exit code
• execution time
• command (for history)
       │
       ▼
Send to Caro runtime via IPC
       │
       ▼
Runtime analyzes:
• Non-zero exit? Check for fixes
• Timing anomalies?
• Pattern for learning?
       │
       ▼
Response: { suggestion?: string, explanation?: string }
       │
       ▼
Shell layer:
• Shows fix suggestion (if any)
• Prompts user to apply fix
```

## UX Principles

1. **Invisible by Default**: Users shouldn't notice Caro unless they invoke it or it detects danger
2. **Non-Blocking**: All safety checks have timeouts; shell never hangs
3. **Respectful**: Caro suggests, users decide
4. **Discoverable**: Clear help, onboarding, and documentation
5. **Consistent**: Same mental model across all shells

## Success Metrics

- **Startup latency**: < 50ms added to shell initialization
- **Command latency**: < 10ms added per command (passive mode)
- **Fix suggestion accuracy**: > 80% useful suggestions
- **Safety pattern coverage**: All OWASP-style dangerous commands detected
- **User satisfaction**: Positive feedback on UX surveys

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Runtime crashes | Shell works normally; hooks degrade gracefully |
| IPC timeouts | 50ms timeout, fall through on timeout |
| Hook interference | Careful integration testing per shell |
| Dotfile conflicts | Minimal footprint, easy uninstall |
| Performance regression | Benchmarks in CI, lazy loading |

## Version Roadmap

### v1.0 (MVP)
- bash + zsh + fish support
- sh/POSIX best-effort
- Post-failure suggestions
- Passive safety warnings
- Caro hotkey for interactive prompt
- Installer/uninstaller

### v1.5
- Inline suggestions as you type
- Advanced TUI overlay
- Plugin architecture

### v2.0
- Cross-machine config sync
- Contextual help for command output
- Team policy sharing

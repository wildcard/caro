# Spec 007: Caro Shell Integration

> Deep integration of Caro into bash, zsh, sh, and fish shells

## Status: Planning Complete

## Summary

This specification defines the architecture and implementation strategy for integrating Caro directly into popular shells. Instead of building a new shell, we embed Caro into existing shells (bash, zsh, sh/POSIX, fish) to provide:

- Pre-execution command observation and safety validation
- Post-execution analysis and "thefuck"-like fix suggestions
- Configurable safety modes (off/passive/active)
- Hotkey access to Caro interactive prompt
- Zero-friction default behavior

## Documents

| Document | Description |
|----------|-------------|
| [spec.md](spec.md) | High-level design with goals, non-goals, and architecture overview |
| [compatibility.md](compatibility.md) | Shell compatibility matrix with hook points and limitations |
| [plan.md](plan.md) | Detailed implementation plan with module layout and IPC design |
| [installer.md](installer.md) | Installer design, dotfiles strategy, and XDG compliance |
| [config.md](config.md) | Configuration system schema and policy toggles |
| [security.md](security.md) | Security and privacy threat model with mitigations |
| [tests.md](tests.md) | Comprehensive test plan (unit, integration, shell, performance) |
| [milestones.md](milestones.md) | MVP milestones and task breakdown with story sizes |

## Key Decisions

### Architecture
- **Long-running daemon** with Unix socket IPC
- **Per-shell integration scripts** (bash.init, zsh.init, fish/)
- **Graceful degradation** - shell works if daemon unavailable

### Safety Levels
- **Off**: No safety checks, pure assist mode
- **Passive**: Warn on dangerous commands but allow
- **Active**: Confirm high-risk, block critical-risk

### Shell Support
- **bash 4+**: Full support via DEBUG trap + PROMPT_COMMAND
- **zsh 5+**: Full support via preexec/precmd + ZLE
- **fish 3+**: Full support via native events
- **sh/POSIX**: Limited support (post-exec only)

## Quick Links

- Start implementation: [milestones.md](milestones.md)
- Understand architecture: [spec.md](spec.md) + [plan.md](plan.md)
- Review security: [security.md](security.md)
- Check shell compatibility: [compatibility.md](compatibility.md)

## MVP Scope

### Included
- Daemon with session management
- bash + zsh + fish integration
- Safety validation (off/passive/active)
- Fix-it suggestions with Esc Esc apply
- Caro hotkey (Ctrl+X Ctrl+C)
- Installer and doctor commands

### Excluded (v1.5/v2.0)
- Inline suggestions while typing
- LLM-powered suggestions
- Cross-machine config sync
- Advanced TUI overlay
- Plugin system

## Related

- [CLAUDE.md](../../CLAUDE.md) - Project overview and development guidelines
- [src/safety/](../../src/safety/) - Existing safety validation module
- [src/execution/](../../src/execution/) - Existing execution context module

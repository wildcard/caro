# Session Wrangler Plugin

A meta-management plugin for Claude Code that helps users wrangle multiple parallel Claude sessions across worktrees.

## Overview

Session Wrangler is the "meta-Claude" that sees all your Claude sessions, diagnoses issues, revives stuck sessions, and helps you navigate the complexity of running 4-5+ parallel sessions.

## Problem It Solves

When running multiple Claude Code sessions:
- Sessions hang with no explanation
- Processes consume memory but do nothing
- Hard to remember which session was doing what
- Jumping between sessions requires context switching
- No unified view of all work in progress

## Features

### 🔍 Discovery
- List all Claude sessions (running, stuck, zombie, resumable)
- Cross-reference with worktrees, branches, and processes
- Check session health (file size, corruption, heartbeats)
- Query PostgreSQL database for stale entries

### 🔬 Inspection
- Deep dive into any session's state
- Understand what it was doing (last prompt/response)
- Diagnose why it's stuck or corrupted
- Get specific recommendations for recovery

### 🚀 Revival
- Spawn Alacritty terminal in correct worktree
- Provide clear resume instructions
- Handle corrupted sessions gracefully
- Protect uncommitted work

### 🧹 Cleanup
- Kill zombie processes safely
- Archive corrupted session files
- Close orphaned terminal windows
- Clean stale database entries

### 🔀 Navigation
- Focus existing session terminals
- Jump between active sessions
- Smart window finding (by title, PID, cwd)
- Cross-platform support (macOS, Linux)

## Phase 2 Enhancements (v2.0.0)

### 📊 Session-Worktree Index
- **Persistent mapping**: Fast bidirectional session ↔ worktree lookups
- **Per-project isolation**: Each repository has its own index
- **Auto-update**: Hooks keep index current as sessions change
- **Manual sync**: `/sessions.sync` forces full rebuild

**Location**: `~/.claude/projects/<project-slug>/session-index.json`

### ⚙️ Configurable Settings
- **Custom claude command**: Support aliases like `claude-yolo`
- **Terminal preference**: Choose Alacritty, Kitty, iTerm2, etc.
- **Global configuration**: Settings apply to all projects

**Location**: `~/.claude/plugins/session-wrangler/config.json`

### 🔒 Worktree Enforcement
- **Location validation**: Worktrees must be in `.worktrees/` only
- **Nesting prevention**: No nested worktrees allowed
- **Cleanup detection**: Find misplaced or nested worktrees

## Commands

| Command | Purpose |
|---------|---------|
| `/sessions` | List all Claude sessions with index acceleration |
| `/sessions.inspect <session>` | Deep dive into a specific session |
| `/sessions.revive <session>` | Spawn terminal with worktree validation |
| `/sessions.cleanup` | Clean up zombies, corrupted files, invalid worktrees |
| `/sessions.switch <session>` | Navigate to another session's terminal |
| `/sessions.sync` | **NEW** Force rebuild session-worktree index |
| `/sessions.config` | **NEW** View and modify configuration settings |

## Usage Examples

### Check All Sessions

```bash
/sessions
```

Output:
```
Active Sessions (3)
  ✅ s009  Resume Stuck Sessions  │ 045-milestone-coordinator  │ 2min ago
  ⚠️ s003  i18n Implementation    │ i18n-complete-system      │ 45min stale
  ❌ s012  QA Testing             │ 040-issue-520-tests       │ ZOMBIE (PID 47759)

Resumable Sessions (2)
  💤 eb6a2d8b  session-manager plugin  │ 2 messages │ Jan 24 05:49
```

### Inspect a Stuck Session

```bash
/sessions.inspect s003
```

Shows:
- Last prompt and response
- File size and health
- Process status (PID, CPU, memory)
- Worktree and uncommitted changes
- Diagnosis and recommended action

### Revive a Session

```bash
/sessions.revive s003
```

Actions:
1. Spawns Alacritty terminal in correct worktree
2. Displays resume command: `claude --resume <session-id>`
3. Focuses the new terminal window

### Clean Up Zombies

```bash
/sessions.cleanup
```

Finds and offers to:
- Kill unresponsive processes
- Archive corrupted session files (>50MB)
- Close orphaned terminal windows
- Delete stale database entries

### Switch to Another Session

```bash
/sessions.switch work-feature
```

Focuses the terminal window for the specified session.

## Architecture

### Data Sources

```
[Session Wrangler Plugin]
       │
       ├── ps aux                           # Running processes
       ├── ~/.claude/history.jsonl          # Last prompts
       ├── ~/.claude/projects/*/            # Session transcripts
       ├── PostgreSQL sessions table        # Heartbeats
       └── git worktree list                # Worktree mappings
```

### Session States

| State | Icon | Meaning |
|-------|------|---------|
| Active | ✅ | Running and responding |
| Stale | ⚠️ | Running but idle 5-30min |
| Zombie | ❌ | Stuck or corrupted |
| Resumable | 💤 | Can be resumed with `--resume` |
| Corrupted | 🔥 | File damaged, cannot resume |

### Health Checks

- **File size**: < 10MB = healthy, > 50MB = corrupted
- **Content**: No null entries = healthy
- **Process**: CPU > 0% = active, CPU = 0% = idle
- **Heartbeat**: < 5min = active, > 30min = stale

## Installation

The plugin is located at:
```
.claude/plugins/session-wrangler/
```

Add to Claude Code's installed plugins:
```json
{
  "plugins": [
    "session-wrangler"
  ]
}
```

## Requirements

- **Claude Code**: Latest version
- **Alacritty**: For spawning terminals (or modify for your terminal)
- **jq**: For JSON parsing (optional, fallback available)
- **PostgreSQL**: Optional, for enhanced session tracking

### Platform Support

- ✅ **macOS**: Full support (osascript for window management)
- ✅ **Linux (X11)**: Full support (wmctrl for window management)
- ⚠️ **Linux (Wayland)**: Limited (compositor-dependent)
- ❌ **Windows**: Not yet supported

## Configuration

### Plugin Configuration (Phase 2)

**Location**: `~/.claude/plugins/session-wrangler/config.json`

**Default Values**:
```json
{
  "claudeCommand": "claude",
  "terminal": "alacritty",
  "options": {
    "defaultFlags": ""
  }
}
```

**Configuration Commands**:

View current config:
```bash
/sessions.config
```

Set custom claude command (e.g., for aliases):
```bash
/sessions.config claude-command claude-yolo
```

Change terminal emulator:
```bash
/sessions.config terminal kitty
```

Reset to defaults:
```bash
/sessions.config reset
```

### Supported Terminals

- **alacritty** (default)
- **kitty**
- **wezterm**
- **iterm2**
- **terminal** (macOS Terminal.app)

### Session Index (Phase 2)

**Location**: `~/.claude/projects/<project-slug>/session-index.json`

**Purpose**: Maintains fast bidirectional mapping between sessions and worktrees.

**Auto-Update**: Index is updated automatically via PostToolUse hooks when:
- Sessions are started or revived
- Sessions complete or are cleaned up
- Worktrees are created or removed

**Manual Sync**:
```bash
/sessions.sync              # Full rebuild
/sessions.sync --verify     # Verify without modifying
```

### Archive Location

Default: `~/.claude/archive/YYYYMMDD/`

Change in `/sessions.cleanup`:
```bash
ARCHIVE_DIR=~/.claude/archive/$(date +%Y%m%d)
```

### Session File Path

Default: `~/.claude/projects/-Users-kobik-private-workspace-caro/`

This is determined automatically per-project.

## Safety Features

- **Never delete permanently**: Always archives files
- **Confirm destructive actions**: Kill, cleanup require confirmation
- **Protect uncommitted work**: Warns before archiving
- **Graceful degradation**: Works without PostgreSQL, jq, etc.

## Troubleshooting

### "Session not found"

Check available sessions:
```bash
/sessions
```

Verify session file exists:
```bash
ls ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl
```

### "Cannot focus terminal"

Manual steps:
1. Use Cmd+Tab (macOS) or Alt+Tab (Linux)
2. Look for window title containing session slug
3. Check Mission Control (macOS) for other desktops

### "Failed to spawn terminal"

Check if Alacritty is installed:
```bash
which alacritty
```

Install if needed:
```bash
brew install --cask alacritty  # macOS
```

### "Permission denied"

Fix session file permissions:
```bash
chmod 644 ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl
```

## Development

### File Structure

```
session-wrangler/
├── .claude-plugin/
│   └── plugin.json          # Plugin manifest
├── commands/
│   ├── sessions.md          # List command
│   ├── inspect.md           # Inspect command
│   ├── revive.md            # Revive command
│   ├── cleanup.md           # Cleanup command
│   └── switch.md            # Switch command
├── skills/
│   └── session-discovery/
│       └── SKILL.md         # Discovery skill
└── README.md                # This file
```

### Adding New Commands

1. Create command markdown in `commands/`
2. Add to `plugin.json` manifest
3. Use session-discovery skill for consistency
4. Follow safety principles (confirm, archive, protect)

### Testing

1. Create multiple Claude sessions in different worktrees
2. Test `/sessions` shows all correctly
3. Test `/sessions.inspect` with each session
4. Test `/sessions.revive` with stopped session
5. Test `/sessions.cleanup` with corrupted file
6. Test `/sessions.switch` with active session

## Contributing

See main project contributing guidelines. This plugin follows:
- Clear user communication
- Safety-first design
- Platform compatibility
- Graceful error handling

## Version History

### v1.0.0 (2026-01-24)
- Initial implementation
- 5 core commands
- Session discovery skill
- macOS and Linux X11 support

## License

AGPL-3.0 (same as parent project)

## Authors

- Caro Project Team
- Session Wrangler designed for managing parallel Claude workflows

## Related

- [Claude Code CLI](https://claude.ai/claude-code)
- [Milestone Coordinator Plugin](..milestone-coordinator/) (companion plugin)
- [Caro Project](https://github.com/caro-project/caro)

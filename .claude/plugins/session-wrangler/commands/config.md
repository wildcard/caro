# /sessions.config - Configure Session Wrangler

**Purpose**: View and modify session wrangler configuration settings.

## Usage

```bash
/sessions.config                        # Show current config
/sessions.config claude-command <cmd>   # Set claude command
/sessions.config terminal <term>        # Set terminal emulator
/sessions.config reset                  # Reset to defaults
```

## Configuration File

**Location**: `~/.claude/plugins/session-wrangler/config.json`

**Schema**:
```json
{
  "claudeCommand": "claude",
  "terminal": "alacritty",
  "options": {
    "defaultFlags": ""
  }
}
```

## Execution Steps

### 1. Show Current Configuration

```bash
CONFIG_FILE="$HOME/.claude/plugins/session-wrangler/config.json"

if [[ -f "$CONFIG_FILE" ]]; then
  CLAUDE_CMD=$(jq -r '.claudeCommand // "claude"' "$CONFIG_FILE")
  TERMINAL=$(jq -r '.terminal // "alacritty"' "$CONFIG_FILE")
  FLAGS=$(jq -r '.options.defaultFlags // ""' "$CONFIG_FILE")
else
  CLAUDE_CMD="claude"
  TERMINAL="alacritty"
  FLAGS=""
fi

# Get session index location
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
PROJECT_SLUG=$(echo "$PROJECT_ROOT" | sed 's|/|-|g' | sed 's|^-||')
INDEX_FILE="$HOME/.claude/projects/$PROJECT_SLUG/session-index.json"

# Count tracked items
if [[ -f "$INDEX_FILE" ]]; then
  SESSION_COUNT=$(jq '.sessions | length' "$INDEX_FILE" 2>/dev/null || echo "0")
  WORKTREE_COUNT=$(jq '.worktrees | length' "$INDEX_FILE" 2>/dev/null || echo "0")
else
  SESSION_COUNT="0"
  WORKTREE_COUNT="0"
fi

cat <<EOF

Session Wrangler Configuration
══════════════════════════════════════

Claude Command: $CLAUDE_CMD
Terminal: $TERMINAL
Default Flags: ${FLAGS:-"(none)"}

Session Index: $INDEX_FILE
  Sessions tracked: $SESSION_COUNT
  Worktrees mapped: $WORKTREE_COUNT

To change settings:
  /sessions.config claude-command <command>
  /sessions.config terminal <terminal>
  /sessions.config reset

EOF
```

### 2. Set Claude Command

When user provides: `/sessions.config claude-command <cmd>`

```bash
CONFIG_FILE="$HOME/.claude/plugins/session-wrangler/config.json"
NEW_COMMAND="$1"  # From user input

# Create config dir if needed
mkdir -p "$(dirname "$CONFIG_FILE")"

# Update or create config
if [[ -f "$CONFIG_FILE" ]]; then
  jq --arg cmd "$NEW_COMMAND" '.claudeCommand = $cmd' "$CONFIG_FILE" > "$CONFIG_FILE.tmp"
  mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
else
  cat > "$CONFIG_FILE" <<EOF
{
  "claudeCommand": "$NEW_COMMAND",
  "terminal": "alacritty",
  "options": {
    "defaultFlags": ""
  }
}
EOF
fi

echo "✓ Claude command set to: $NEW_COMMAND"
echo ""
echo "This command will be used in:"
echo "  - /sessions.revive instructions"
echo "  - /sessions resume commands"
```

### 3. Set Terminal

When user provides: `/sessions.config terminal <term>`

```bash
CONFIG_FILE="$HOME/.claude/plugins/session-wrangler/config.json"
NEW_TERMINAL="$1"  # From user input

# Validate terminal
case "$NEW_TERMINAL" in
  alacritty|kitty|wezterm|iterm2|terminal)
    ;;
  *)
    echo "⚠️  Unknown terminal: $NEW_TERMINAL"
    echo "Supported: alacritty, kitty, wezterm, iterm2, terminal"
    exit 1
    ;;
esac

# Create config dir if needed
mkdir -p "$(dirname "$CONFIG_FILE")"

# Update or create config
if [[ -f "$CONFIG_FILE" ]]; then
  jq --arg term "$NEW_TERMINAL" '.terminal = $term' "$CONFIG_FILE" > "$CONFIG_FILE.tmp"
  mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
else
  cat > "$CONFIG_FILE" <<EOF
{
  "claudeCommand": "claude",
  "terminal": "$NEW_TERMINAL",
  "options": {
    "defaultFlags": ""
  }
}
EOF
fi

echo "✓ Terminal set to: $NEW_TERMINAL"
```

### 4. Reset to Defaults

When user provides: `/sessions.config reset`

```bash
CONFIG_FILE="$HOME/.claude/plugins/session-wrangler/config.json"

cat > "$CONFIG_FILE" <<EOF
{
  "claudeCommand": "claude",
  "terminal": "alacritty",
  "options": {
    "defaultFlags": ""
  }
}
EOF

echo "✓ Configuration reset to defaults"
echo ""
echo "Claude command: claude"
echo "Terminal: alacritty"
```

## Configuration Precedence

When resolving the claude command:

1. **Plugin config**: `~/.claude/plugins/session-wrangler/config.json`
2. **Environment variable**: `$CLAUDE_COMMAND`
3. **Default**: `claude`

**Example**:
```bash
# In revive.md or sessions.md:
CLAUDE_CMD=$(jq -r '.claudeCommand // "claude"' ~/.claude/plugins/session-wrangler/config.json 2>/dev/null)
CLAUDE_CMD=${CLAUDE_CMD:-${CLAUDE_COMMAND:-claude}}
```

## Notes

- Configuration is **global** (applies to all projects)
- Session index is **per-project** (isolated per repository)
- Changes take effect immediately in all commands
- Use `/sessions.sync` after config changes to update displayed commands

## Examples

**Set custom claude command**:
```bash
/sessions.config claude-command claude-yolo
```

**Check current settings**:
```bash
/sessions.config
```

**Switch terminal emulator**:
```bash
/sessions.config terminal kitty
```

**Reset everything**:
```bash
/sessions.config reset
```

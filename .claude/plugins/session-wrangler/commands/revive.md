# /sessions.revive - Revive a Stuck Session

Spawn a new Alacritty terminal in the correct worktree and provide instructions to resume or restart a Claude session.

## Purpose

Help users recover from:
- Sessions that stopped responding
- Sessions with corrupted files
- Sessions that need to be resumed after system restart
- Sessions in resumable state

**Key principle:** The plugin does NOT resume the session itself - it spawns the terminal and tells the user how to resume.

## Usage

```bash
/sessions.revive <session-identifier>
```

Where `<session-identifier>` can be:
- Slug: `warm-floating-pancake`
- Short ID: `2893aa9b`
- Full ID: `2893aa9b-69d3-4167-9ad1-4c5a06697d93`
- Worktree: `045-milestone-coordinator-plugin`

## Pre-Flight Checks

Before attempting to revive:

1. **Resolve session identifier** (same logic as `/sessions.inspect`)
2. **Check session health** (run subset of inspect checks)
3. **Determine if revivable** (see decision matrix below)
4. **Find or create worktree** (if deleted, offer to recreate)

### Decision Matrix

| Current State | File Health | Worktree Exists | Action |
|---------------|-------------|-----------------|--------|
| Not running | Healthy (<10MB) | Yes | Resume in existing worktree |
| Not running | Healthy | No | Offer to recreate worktree |
| Not running | Corrupted | Yes | Start fresh, preserve context |
| Not running | Corrupted | No | Start fresh on branch |
| Running | Any | Any | Switch to existing terminal |
| Zombie | Any | Yes | Kill process, then resume |

## Execution Steps

### 1. Resolve and Inspect Session

```bash
# Use same resolution logic as /sessions.inspect
SESSION_FILE=<resolved-path>
SESSION_ID=$(basename "$SESSION_FILE" .jsonl)

# Get metadata
SLUG=$(head -20 "$SESSION_FILE" | jq -r 'select(.slug) | .slug' | head -1)
CWD=$(head -20 "$SESSION_FILE" | jq -r 'select(.cwd) | .cwd' | head -1)
BRANCH=$(head -20 "$SESSION_FILE" | jq -r 'select(.gitBranch) | .gitBranch' | head -1)

# Check file health
FILE_SIZE=$(stat -f%z "$SESSION_FILE" 2>/dev/null || stat -c%s "$SESSION_FILE")
IS_CORRUPTED=false
if [ "$FILE_SIZE" -gt 52428800 ] || [ $(grep -c '"content":null' "$SESSION_FILE") -gt 10 ]; then
  IS_CORRUPTED=true
fi

# Check if process running
PID=$(ps aux | grep "[c]laude.*$SESSION_ID" | awk '{print $2}' | head -1)
```

### 2. Handle Running Session

If process already running:

```bash
if [ -n "$PID" ]; then
  echo "⚠️  Session is already running (PID $PID)"
  echo ""
  echo "This session is active. Options:"
  echo ""
  echo "1. Use /sessions.switch to navigate to its terminal"
  echo "2. If terminal is lost, this may be a zombie. Use /sessions.cleanup"
  echo ""
  echo "Would you like me to:"
  echo "  a) Switch to the session's terminal"
  echo "  b) Kill the process and restart clean"
  exit 0
fi
```

### 3. Check Worktree

```bash
# Check if worktree exists
WORKTREE_PATH=""
if [ -d "$CWD" ]; then
  WORKTREE_PATH="$CWD"
else
  # Try to find by branch name
  WORKTREE_PATH=$(git -C /Users/kobik-private/workspace/caro worktree list | grep "$BRANCH" | awk '{print $1}')
fi

if [ -z "$WORKTREE_PATH" ] || [ ! -d "$WORKTREE_PATH" ]; then
  echo "⚠️  Worktree not found: $CWD"
  echo ""
  echo "The session was working in a worktree that no longer exists."
  echo ""
  echo "Options:"
  echo "1. Recreate the worktree for branch: $BRANCH"
  echo "2. Start fresh session in main workspace"
  echo ""
  echo "Would you like me to recreate the worktree?"
  # Wait for user input
  read -p "Recreate worktree? (y/n): " RECREATE

  if [ "$RECREATE" == "y" ]; then
    # Generate worktree name
    WORKTREE_NUM=$(git -C /Users/kobik-private/workspace/caro worktree list | wc -l | tr -d ' ')
    WORKTREE_NAME="${WORKTREE_NUM}-${BRANCH//\//-}"
    WORKTREE_PATH="/Users/kobik-private/workspace/caro/.worktrees/$WORKTREE_NAME"

    # Create worktree
    git -C /Users/kobik-private/workspace/caro worktree add "$WORKTREE_PATH" "$BRANCH"

    if [ $? -ne 0 ]; then
      echo "❌ Failed to create worktree"
      exit 1
    fi

    echo "✅ Worktree created: $WORKTREE_PATH"
  else
    echo "Aborting revival."
    exit 0
  fi
fi
```

### 4. Handle Corrupted Session

If session is corrupted:

```bash
if [ "$IS_CORRUPTED" == "true" ]; then
  echo "═══════════════════════════════════════════════════════════════════"
  echo "⚠️  Session File Corrupted"
  echo "═══════════════════════════════════════════════════════════════════"
  echo ""
  echo "Session: $SLUG"
  echo "File size: $(echo "scale=2; $FILE_SIZE / 1048576" | bc)MB"
  echo ""
  echo "This session cannot be resumed due to file corruption."
  echo "The session file is either too large or contains null content."
  echo ""
  echo "I can start a fresh session in the same worktree, but the"
  echo "conversation history will be lost."
  echo ""
  echo "Worktree: $WORKTREE_PATH"
  echo "Branch: $BRANCH"
  echo ""
  echo "📋 Before starting fresh, check for uncommitted work:"

  cd "$WORKTREE_PATH"
  if [ -n "$(git status --porcelain)" ]; then
    echo ""
    echo "⚠️  UNCOMMITTED CHANGES FOUND:"
    git status --short
    echo ""
    echo "You may want to commit these before starting a new session."
    echo ""
  fi

  echo "I'll now spawn a terminal in the worktree."
  echo "You can start fresh with: claude"
  echo ""
  echo "Press Enter to continue..."
  read
fi
```

### 5. Spawn Terminal

```bash
# Determine terminal title
TERM_TITLE="Claude: $SLUG"

# Spawn Alacritty in the worktree directory
alacritty \
  --title "$TERM_TITLE" \
  --working-directory "$WORKTREE_PATH" \
  &

ALACRITTY_PID=$!

# Give terminal time to spawn
sleep 0.5

# Check if it succeeded
if ps -p $ALACRITTY_PID > /dev/null 2>&1; then
  echo "✅ Terminal spawned successfully (PID $ALACRITTY_PID)"
else
  echo "❌ Failed to spawn terminal"
  echo ""
  echo "You can manually navigate to:"
  echo "  cd $WORKTREE_PATH"
  exit 1
fi
```

### 6. Provide Instructions

```bash
# Different instructions based on corruption state
if [ "$IS_CORRUPTED" == "true" ]; then
  cat <<EOF

═══════════════════════════════════════════════════════════════════
✅ Terminal Spawned
═══════════════════════════════════════════════════════════════════

I've opened a new Alacritty terminal in:
  $WORKTREE_PATH

Since the session file is corrupted, start a fresh session with:

  claude

The previous conversation history is lost, but you can:
  • Continue work in the same worktree
  • Reference uncommitted files
  • Review session file manually: $SESSION_FILE

To archive the old session file:
  mkdir -p ~/.claude/archive/
  mv $SESSION_FILE ~/.claude/archive/

═══════════════════════════════════════════════════════════════════
EOF

else
  cat <<EOF

═══════════════════════════════════════════════════════════════════
✅ Terminal Spawned
═══════════════════════════════════════════════════════════════════

I've opened a new Alacritty terminal in:
  $WORKTREE_PATH

To resume your previous session, type this in the new terminal:

  claude --resume $SESSION_ID

This will restore your full conversation history ($(grep -c '"role":"user"' "$SESSION_FILE") messages).

Your last prompt was:
  "$(grep '"role":"user"' "$SESSION_FILE" | tail -1 | jq -r '.content[0].text' 2>/dev/null | head -c 100)..."

═══════════════════════════════════════════════════════════════════
EOF
fi
```

### 7. Focus Terminal (macOS only)

```bash
# On macOS, try to focus the new terminal
if [[ "$OSTYPE" == "darwin"* ]]; then
  osascript <<EOF 2>/dev/null
    tell application "Alacritty"
      activate
    end tell
EOF

  if [ $? -eq 0 ]; then
    echo "🎯 Terminal window focused"
  fi
fi
```

## Special Cases

### Session Already Active

```
⚠️  Session Already Running

Session: <slug>
PID: <pid>
Started: <start-time>

This session is currently active. Options:

1. /sessions.switch <slug>
   Navigate to the existing terminal

2. Kill and restart:
   kill -TERM <pid>
   /sessions.revive <slug>

3. Check if zombie:
   /sessions.inspect <slug>

What would you like to do?
```

### Worktree Deleted

```
⚠️  Worktree No Longer Exists

Session was in: <cwd>
Branch: <branch>

The worktree directory has been deleted. I can:

1. Recreate the worktree:
   git worktree add .worktrees/NNN-<branch> <branch>

2. Start fresh on branch in main workspace:
   cd /Users/kobik-private/workspace/caro
   git checkout <branch>
   claude

3. Abandon this session (mark as archived)

Which option would you prefer?
```

### Branch Deleted

```
⚠️  Branch No Longer Exists

Session was on branch: <branch>

The branch has been deleted (possibly merged to main).

Options:

1. Start fresh session on main branch
2. Recreate the branch from a commit
3. Archive this session

The session file is preserved at:
  <session-file>

What would you like to do?
```

### Alacritty Not Installed

```
❌ Alacritty Not Found

I need Alacritty terminal to spawn a new session window.

Options:

1. Install Alacritty:
   brew install --cask alacritty

2. Manually navigate:
   cd <worktree-path>
   claude --resume <session-id>

3. Use different terminal (manual setup required)

After installing Alacritty, run this command again.
```

## User Confirmation Flow

For potentially destructive operations, always confirm:

```bash
# Before killing a process
echo "⚠️  This will kill PID $PID. Continue? (y/n): "
read CONFIRM
if [ "$CONFIRM" != "y" ]; then
  echo "Aborting."
  exit 0
fi

# Before recreating worktree
echo "This will create a new worktree at: $WORKTREE_PATH"
echo "Continue? (y/n): "
read CONFIRM
if [ "$CONFIRM" != "y" ]; then
  echo "Aborting."
  exit 0
fi

# Before starting fresh (corrupted session)
echo "⚠️  Starting fresh will lose conversation history. Continue? (y/n): "
read CONFIRM
if [ "$CONFIRM" != "y" ]; then
  echo "Aborting. You can manually inspect: $SESSION_FILE"
  exit 0
fi
```

## Integration with Other Commands

After successful revival:

```
💡 Next Steps:

• In the new terminal, follow the instructions above
• Use /sessions to verify the session is active
• Use /sessions.inspect to check health after resuming
• If issues persist, use /sessions.cleanup
```

## Error Handling

| Error | Recovery |
|-------|----------|
| Terminal spawn fails | Provide manual `cd` command |
| Session not found | Suggest `/sessions` to list available |
| Permission denied | Suggest `chmod` fix |
| Git worktree error | Suggest `git worktree prune` |
| Branch not found | Offer to recreate or abandon |

## Performance Notes

- Terminal spawn is async (backgrounded with `&`)
- Don't wait for terminal to fully load
- Provide instructions immediately
- Let user execute resume command in their own time

## Success Criteria

- ✅ Terminal spawned in correct directory
- ✅ Instructions displayed clearly
- ✅ User knows exactly what to type
- ✅ Corrupted sessions handled gracefully
- ✅ No data loss from uncommitted work

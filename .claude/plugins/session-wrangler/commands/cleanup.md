# /sessions.cleanup - Clean Up Zombies

Safely identify and remove zombie Claude processes, corrupted session files, and orphaned resources.

## Purpose

Help users clean up:
- Zombie processes (running but unresponsive)
- Corrupted session files (>50MB or null content)
- Orphaned Alacritty windows
- Stale database entries

**Safety First:** Always confirm before deleting anything. Archive rather than delete when possible.

## Usage

```bash
/sessions.cleanup [--dry-run] [--force-all] [--type=<process|file|terminal|db>]
```

Options:
- `--dry-run`: Show what would be cleaned up without doing it
- `--force-all`: Skip confirmations (dangerous!)
- `--type=X`: Clean only specific resource type

## Discovery Phase

### 1. Find Zombie Processes

```bash
# Get all Claude processes
ps aux | grep "[c]laude" | while read -r line; do
  PID=$(echo "$line" | awk '{print $2}')
  CPU=$(echo "$line" | awk '{print $3}')
  START=$(echo "$line" | awk '{print $9}')

  # Check if process is a zombie candidate
  # Criteria: CPU = 0% AND running for > 30 min

  # Get process start time in epoch
  START_EPOCH=$(ps -p $PID -o lstart= | xargs -I {} date -j -f "%a %b %d %T %Y" "{}" "+%s" 2>/dev/null)
  NOW=$(date +%s)
  RUNTIME=$((NOW - START_EPOCH))

  if [[ "$CPU" == "0.0" ]] && [ "$RUNTIME" -gt 1800 ]; then
    # Extract session ID from command line
    SESSION_ID=$(ps -p $PID -o command= | grep -oE '[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}')

    # Check if session file is corrupted or if there's no heartbeat
    if [ -n "$SESSION_ID" ]; then
      SESSION_FILE=~/.claude/projects/-Users-kobik-private-workspace-caro/${SESSION_ID}.jsonl

      if [ -f "$SESSION_FILE" ]; then
        FILE_SIZE=$(stat -f%z "$SESSION_FILE" 2>/dev/null || stat -c%s "$SESSION_FILE")
        NULL_COUNT=$(grep -c '"content":null' "$SESSION_FILE" 2>/dev/null || echo 0)

        if [ "$FILE_SIZE" -gt 52428800 ] || [ "$NULL_COUNT" -gt 10 ]; then
          echo "$PID|$SESSION_ID|CORRUPTED_FILE"
        else
          echo "$PID|$SESSION_ID|ZOMBIE_IDLE"
        fi
      else
        echo "$PID|UNKNOWN|NO_SESSION_FILE"
      fi
    else
      echo "$PID|UNKNOWN|NO_SESSION_ID"
    fi
  fi
done
```

### 2. Find Corrupted Session Files

```bash
# Scan all session files
for file in ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl; do
  FILE_SIZE=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
  SESSION_ID=$(basename "$file" .jsonl)

  # Check if process is running for this session
  PID=$(ps aux | grep "[c]laude.*$SESSION_ID" | awk '{print $2}')

  # Size check
  if [ "$FILE_SIZE" -gt 52428800 ]; then
    echo "$file|SIZE|$FILE_SIZE|$PID"
    continue
  fi

  # Null content check
  NULL_COUNT=$(grep -c '"content":null' "$file" 2>/dev/null || echo 0)
  if [ "$NULL_COUNT" -gt 10 ]; then
    echo "$file|NULL_CONTENT|$NULL_COUNT|$PID"
    continue
  fi

  # JSON validity check
  if ! head -1 "$file" | jq empty 2>/dev/null; then
    echo "$file|INVALID_JSON|0|$PID"
    continue
  fi
done
```

### 3. Find Orphaned Alacritty Windows

```bash
# On macOS, query Alacritty windows
if [[ "$OSTYPE" == "darwin"* ]]; then
  osascript <<'EOF' 2>/dev/null | while read -r window_title; do
    # Check if this is a Claude session window
    if [[ "$window_title" == *"Claude:"* ]]; then
      # Extract slug from title
      SLUG=$(echo "$window_title" | sed 's/.*Claude: \(.*\)/\1/')

      # Check if session file exists
      SESSION_FILE=$(find ~/.claude/projects/-Users-kobik-private-workspace-caro/ -name "*.jsonl" -exec sh -c '
        FOUND_SLUG=$(head -20 "{}" | jq -r "select(.slug) | .slug" 2>/dev/null | head -1)
        if [ "$FOUND_SLUG" == "'$SLUG'" ]; then
          echo "{}"
        fi
      ' \; | head -1)

      if [ -z "$SESSION_FILE" ]; then
        echo "$window_title|ORPHANED|NO_SESSION"
      else
        SESSION_ID=$(basename "$SESSION_FILE" .jsonl)
        PID=$(ps aux | grep "[c]laude.*$SESSION_ID" | awk '{print $2}')
        if [ -z "$PID" ]; then
          echo "$window_title|ORPHANED|SESSION_DEAD"
        fi
      fi
    fi
  done
tell application "Alacritty"
  set windowTitles to name of every window
  repeat with aTitle in windowTitles
    return aTitle
  end repeat
end tell
EOF
fi
```

### 4. Find Stale Database Entries

```bash
# Check if PostgreSQL is available
if docker ps | grep -q continuous-claude-postgres; then
  # Find sessions with very stale heartbeats and no corresponding file
  docker exec continuous-claude-postgres psql -U claude -d continuous_claude -t -c "
    SELECT id, working_on,
           EXTRACT(EPOCH FROM (NOW() - last_heartbeat)) as staleness
    FROM sessions
    WHERE last_heartbeat < NOW() - INTERVAL '24 hours'
    ORDER BY staleness DESC;
  " 2>/dev/null | while IFS='|' read -r session_id working_on staleness; do
    session_id=$(echo "$session_id" | xargs)
    staleness=$(echo "$staleness" | xargs | cut -d. -f1)

    # Check if session file exists
    SESSION_FILE=~/.claude/projects/-Users-kobik-private-workspace-caro/${session_id}.jsonl
    if [ ! -f "$SESSION_FILE" ]; then
      echo "$session_id|DB_ORPHAN|${staleness}s"
    fi
  done
fi
```

### 5. Find Invalid Worktrees (Phase 2: Worktree Nesting Enforcement)

```bash
# Get project root
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

# Find all .worktrees directories (including nested ones)
find "$PROJECT_ROOT" -name ".worktrees" -type d 2>/dev/null | while read -r worktree_dir; do
  # Check if it's NOT the root .worktrees directory
  if [[ "$worktree_dir" != "$PROJECT_ROOT/.worktrees" ]]; then
    echo "$worktree_dir|NESTED_WORKTREE|INVALID"
  fi
done

# Check worktrees that are not in .worktrees/ directory
git worktree list 2>/dev/null | while read -r line; do
  WORKTREE_PATH=$(echo "$line" | awk '{print $1}')

  # Skip the main repo path
  if [[ "$WORKTREE_PATH" == "$PROJECT_ROOT" ]]; then
    continue
  fi

  # Check if worktree is in the correct location
  if [[ ! "$WORKTREE_PATH" =~ ^$PROJECT_ROOT/\.worktrees/ ]]; then
    BRANCH=$(echo "$line" | awk '{print $3}' | sed 's/\[//' | sed 's/\]//')
    echo "$WORKTREE_PATH|MISPLACED_WORKTREE|$BRANCH"
  fi
done
```

## Analysis Phase

Collect all findings and categorize:

```
═══════════════════════════════════════════════════════════════════
Cleanup Analysis
═══════════════════════════════════════════════════════════════════

Zombie Processes (<count>)
───────────────────────────────────────────────────────────────────
  PID <pid>
    Session: <slug> (<short-id>)
    Running: <runtime>
    CPU: 0.0%
    Reason: <reason>
    Action: Kill process

Corrupted Session Files (<count>)
───────────────────────────────────────────────────────────────────
  <filename>
    Size: <size>MB (limit: 50MB)
    Null Content: <count> entries
    Process: <PID or "Not running">
    Worktree: <path>
    Action: Archive to ~/.claude/archive/

Orphaned Windows (<count>)
───────────────────────────────────────────────────────────────────
  Alacritty: "<window-title>"
    Session: <slug or "UNKNOWN">
    Reason: <reason>
    Action: Close window

Stale Database Entries (<count>)
───────────────────────────────────────────────────────────────────
  Session: <short-id>
    Last Heartbeat: <time-ago>
    File Exists: No
    Action: Delete from database

═══════════════════════════════════════════════════════════════════
Total Issues: <count>
  Zombie Processes: <n>
  Corrupted Files: <n>
  Orphaned Windows: <n>
  Stale DB Entries: <n>

Estimated Disk Space to Recover: <total-mb>MB
═══════════════════════════════════════════════════════════════════
```

## Confirmation Phase

If not `--dry-run` or `--force-all`, ask for confirmation:

```bash
# Show summary
echo "This will:"
echo "  • Kill $ZOMBIE_COUNT zombie processes"
echo "  • Archive $CORRUPTED_COUNT corrupted session files"
echo "  • Close $ORPHANED_COUNT orphaned terminal windows"
echo "  • Delete $STALE_DB_COUNT database entries"
echo ""
echo "Archived files will be moved to: ~/.claude/archive/"
echo "No data will be permanently deleted without your confirmation."
echo ""
read -p "Proceed with cleanup? (y/n): " CONFIRM

if [ "$CONFIRM" != "y" ]; then
  echo "Cleanup cancelled."
  exit 0
fi
```

For each category, offer granular control:

```bash
# Process-by-process confirmation
echo ""
echo "Zombie Processes:"
while read -r zombie_info; do
  PID=$(echo "$zombie_info" | cut -d'|' -f1)
  SESSION_ID=$(echo "$zombie_info" | cut -d'|' -f2)
  REASON=$(echo "$zombie_info" | cut -d'|' -f3)

  echo ""
  echo "  PID $PID - Session: $SESSION_ID"
  echo "  Reason: $REASON"
  read -p "  Kill this process? (y/n/skip-all): " CONFIRM

  if [ "$CONFIRM" == "skip-all" ]; then
    break
  elif [ "$CONFIRM" == "y" ]; then
    # Record for execution
    echo "$PID" >> /tmp/claude-cleanup-pids.txt
  fi
done
```

## Execution Phase

### 1. Kill Zombie Processes

```bash
# Read PIDs to kill
while read -r PID; do
  echo "Killing process $PID..."

  # Try graceful termination first
  kill -TERM "$PID" 2>/dev/null

  # Wait up to 10 seconds
  for i in {1..10}; do
    if ! ps -p "$PID" > /dev/null 2>&1; then
      echo "  ✅ Process terminated gracefully"
      break
    fi
    sleep 1
  done

  # Force kill if still running
  if ps -p "$PID" > /dev/null 2>&1; then
    echo "  ⚠️  Forcing kill..."
    kill -9 "$PID" 2>/dev/null
    sleep 1

    if ps -p "$PID" > /dev/null 2>&1; then
      echo "  ❌ Failed to kill process $PID"
    else
      echo "  ✅ Process force-killed"
    fi
  fi
done < /tmp/claude-cleanup-pids.txt
```

### 2. Archive Corrupted Files

```bash
# Create archive directory
ARCHIVE_DIR=~/.claude/archive/$(date +%Y%m%d)
mkdir -p "$ARCHIVE_DIR"

# Move files
while read -r file_info; do
  FILE_PATH=$(echo "$file_info" | cut -d'|' -f1)
  REASON=$(echo "$file_info" | cut -d'|' -f2)

  FILENAME=$(basename "$FILE_PATH")
  ARCHIVE_PATH="$ARCHIVE_DIR/$FILENAME"

  echo "Archiving $FILENAME..."
  echo "  Reason: $REASON"

  # Move file
  mv "$FILE_PATH" "$ARCHIVE_PATH"

  if [ $? -eq 0 ]; then
    echo "  ✅ Archived to: $ARCHIVE_PATH"
  else
    echo "  ❌ Failed to archive"
  fi
done < /tmp/claude-cleanup-files.txt

echo ""
echo "Archived files location: $ARCHIVE_DIR"
```

### 3. Close Orphaned Windows

```bash
# macOS only
if [[ "$OSTYPE" == "darwin"* ]]; then
  while read -r window_title; do
    echo "Closing Alacritty window: $window_title..."

    osascript <<EOF 2>/dev/null
      tell application "Alacritty"
        set windowList to every window whose name contains "$window_title"
        repeat with aWindow in windowList
          close aWindow
        end repeat
      end tell
EOF

    if [ $? -eq 0 ]; then
      echo "  ✅ Window closed"
    else
      echo "  ⚠️  Failed to close window (may need manual closure)"
    fi
  done < /tmp/claude-cleanup-windows.txt
fi
```

### 4. Clean Database Entries

```bash
# PostgreSQL cleanup
if docker ps | grep -q continuous-claude-postgres; then
  while read -r session_id; do
    echo "Deleting database entry for: $session_id..."

    docker exec continuous-claude-postgres psql -U claude -d continuous_claude -c \
      "DELETE FROM sessions WHERE id = '$session_id';" 2>/dev/null

    if [ $? -eq 0 ]; then
      echo "  ✅ Database entry deleted"
    else
      echo "  ❌ Failed to delete database entry"
    fi
  done < /tmp/claude-cleanup-db.txt
fi
```

## Summary Report

```
═══════════════════════════════════════════════════════════════════
Cleanup Complete
═══════════════════════════════════════════════════════════════════

Results:
  ✅ Killed <n> zombie processes
  ✅ Archived <n> corrupted session files
  ✅ Closed <n> orphaned windows
  ✅ Deleted <n> stale database entries

Archived Files:
  Location: <archive-dir>
  Total Size: <size>MB

Failed Operations:
  <list any failures>

Disk Space Recovered: <total-mb>MB

═══════════════════════════════════════════════════════════════════

💡 Next Steps:
  • Run /sessions to see current healthy sessions
  • Review archived files in: <archive-dir>
  • If needed, manually inspect failed operations

Archived files are kept for 30 days by default.
To permanently delete: rm -rf <archive-dir>
```

## Safety Features

### Pre-Cleanup Validation

```bash
# Never clean up if user explicitly says no
# Always show what will be affected before acting
# Provide rollback information

# Check for uncommitted work in worktrees
for file_info in $CORRUPTED_FILES; do
  SESSION_FILE=$(echo "$file_info" | cut -d'|' -f1)
  SESSION_ID=$(basename "$SESSION_FILE" .jsonl)

  # Find worktree
  CWD=$(head -20 "$SESSION_FILE" | jq -r 'select(.cwd) | .cwd' | head -1)

  if [ -n "$CWD" ] && [ -d "$CWD" ]; then
    cd "$CWD"
    if [ -n "$(git status --porcelain)" ]; then
      echo "⚠️  WARNING: Uncommitted changes in worktree: $CWD"
      echo "   Files:"
      git status --short | head -5
      echo ""
      echo "   Consider committing before cleanup!"
      echo ""
      read -p "   Skip this session? (y/n): " SKIP
      if [ "$SKIP" == "y" ]; then
        # Remove from cleanup list
        continue
      fi
    fi
  fi
done
```

### Archive Instead of Delete

- NEVER permanently delete session files
- Always move to `~/.claude/archive/YYYYMMDD/`
- Keep archive structure organized by date
- Provide easy recovery path

### Confirmation Levels

| Operation | Confirmation Level |
|-----------|-------------------|
| Kill idle process (no corruption) | Per-process or batch |
| Kill zombie (corrupted session) | Per-process |
| Archive corrupted file (no uncommitted work) | Batch OK |
| Archive corrupted file (uncommitted work) | Per-file required |
| Close orphaned window | Batch OK |
| Delete DB entry (file also deleted) | Batch OK |

## Error Handling

| Error | Recovery |
|-------|----------|
| Process won't die | Report PID, suggest manual `kill -9` |
| File move fails | Check permissions, disk space |
| Window close fails | Provide manual close instructions |
| DB delete fails | Log error, continue with others |
| Permission denied | Suggest `sudo` or permission fix |

## Special Cases

### All Sessions Healthy

```
✅ All Sessions Healthy

No cleanup needed! Your Claude sessions are in good shape:
  • <n> active sessions running normally
  • <n> resumable sessions available
  • No zombie processes found
  • No corrupted files detected

Run /sessions to see all sessions.
```

### Dry Run Mode

```
═══════════════════════════════════════════════════════════════════
Dry Run - No Changes Made
═══════════════════════════════════════════════════════════════════

<show full analysis>

This was a dry run. No processes were killed or files moved.

To actually perform cleanup, run:
  /sessions.cleanup

To clean only specific types:
  /sessions.cleanup --type=process   (kill zombies only)
  /sessions.cleanup --type=file      (archive corrupted files only)
  /sessions.cleanup --type=terminal  (close orphaned windows only)
  /sessions.cleanup --type=db        (clean database only)
```

### Cleanup After Revive

If user ran `/sessions.revive` and then `/sessions.cleanup`, detect and handle:

```
ℹ️  Detected Recent Revival

Session <slug> was just revived. Excluding from cleanup to avoid
interrupting the newly started process.

Waiting 60 seconds before considering it for cleanup.
```

## Performance Notes

- Process zombie detection: O(n) where n = running processes
- File corruption scan: O(m) where m = session files
- Limit scan depth to improve speed on large session histories
- Cache results during confirmation phase to avoid re-scanning

## Integration

After cleanup completes:
- Update `/sessions` cache if any exists
- Clear any stale database connections
- Update session index (Phase 2)
- Suggest running `/sessions` to verify cleanup

### Update Session Index (Phase 2)

```bash
# Get project info
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
PROJECT_SLUG=$(echo "$PROJECT_ROOT" | sed 's|/|-|g' | sed 's|^-||')
INDEX_FILE="$HOME/.claude/projects/$PROJECT_SLUG/session-index.json"

if [ -f "$INDEX_FILE" ]; then
  # Remove cleaned up sessions from index
  TEMP_INDEX=$(mktemp)

  # Build list of sessions that were cleaned up
  CLEANED_SESSIONS=""
  for SESSION_ID in "${ARCHIVED_SESSIONS[@]}"; do
    CLEANED_SESSIONS="$CLEANED_SESSIONS $SESSION_ID"
  done

  # Update index by removing cleaned sessions
  if [ -n "$CLEANED_SESSIONS" ]; then
    jq "$(
      for sid in $CLEANED_SESSIONS; do
        echo "del(.sessions[\"$sid\"]) |"
      done
      echo "."
    )" "$INDEX_FILE" > "$TEMP_INDEX"
    mv "$TEMP_INDEX" "$INDEX_FILE"

    echo ""
    echo "✓ Session index updated ($INDEX_FILE)"
  fi

  # Suggest sync if major cleanup occurred
  if [ ${#ARCHIVED_SESSIONS[@]} -gt 3 ]; then
    echo ""
    echo "💡 Run /sessions.sync to fully rebuild the index"
  fi
fi
```

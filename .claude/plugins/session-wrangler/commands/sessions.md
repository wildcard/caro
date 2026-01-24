# /sessions - List All Claude Sessions

List all Claude Code sessions across all states: running, stale, zombie, and resumable.

## Purpose

Provide a unified view of all Claude sessions to help users understand:
- What sessions are currently active
- Which sessions are stuck or zombie
- Which sessions can be resumed
- Where each session is working (worktree/branch)

## Execution Steps

### 1. Discover Running Processes

```bash
ps aux | grep "[c]laude" | awk '{print $2, $3, $9, $11}'
```

Parse output to get:
- PID (column 1)
- CPU % (column 2)
- Start time (column 3)
- Command (column 4+)

### 2. List Session Files

```bash
ls -lh ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl 2>/dev/null
```

For each session file, extract:
- Filename (session ID)
- File size (indicator of health)

### 3. Get Session Metadata

For each session file:

```bash
SESSION_FILE="<path-to-session>.jsonl"

# Extract slug
SLUG=$(head -20 "$SESSION_FILE" | jq -r 'select(.slug) | .slug' | head -1)

# Extract branch
BRANCH=$(head -20 "$SESSION_FILE" | jq -r 'select(.gitBranch) | .gitBranch' | head -1)

# Extract cwd
CWD=$(head -20 "$SESSION_FILE" | jq -r 'select(.cwd) | .cwd' | head -1)

# Get last user prompt (truncated)
LAST_PROMPT=$(grep '"role":"user"' "$SESSION_FILE" | tail -1 | jq -r '.content[0].text' 2>/dev/null | head -c 50)

# Count messages
MSG_COUNT=$(grep -c '"role":"user"' "$SESSION_FILE")
```

### 4. Get Last Activity Time

```bash
# From history file
LAST_ACTIVITY=$(grep '"sessionId":"<session-id>"' ~/.claude/history.jsonl 2>/dev/null | tail -1 | jq -r '.timestamp')

# Or from session file modification time
LAST_MODIFIED=$(stat -f%m "$SESSION_FILE" 2>/dev/null || stat -c%Y "$SESSION_FILE")
```

Calculate time ago:
```bash
NOW=$(date +%s)
DIFF=$((NOW - LAST_MODIFIED))
if [ $DIFF -lt 300 ]; then
  TIME_AGO="$((DIFF / 60))min ago"
elif [ $DIFF -lt 3600 ]; then
  TIME_AGO="$((DIFF / 60))min ago"
elif [ $DIFF -lt 86400 ]; then
  TIME_AGO="$((DIFF / 3600))h ago"
else
  TIME_AGO="$((DIFF / 86400))d ago"
fi
```

### 5. Map to Worktrees

```bash
# List worktrees
git -C /Users/kobik-private/workspace/caro worktree list

# Match session CWD to worktree path
WORKTREE=$(git -C /Users/kobik-private/workspace/caro worktree list | grep "$CWD" | awk '{print $1}' | xargs basename)
```

### 6. Determine Session State

For each session, determine state based on:

| Condition | State |
|-----------|-------|
| Process running + CPU > 0% + Activity < 5min | ✅ Active |
| Process running + CPU = 0% + Activity 5-30min | ⚠️ Stale |
| Process running + (File > 50MB OR Activity > 30min) | ❌ Zombie |
| No process + File < 10MB + No corruption | 💤 Resumable |
| File > 50MB OR null content | 🔥 Corrupted |

### 7. Query PostgreSQL (Optional)

If available, cross-reference with database:

```bash
docker exec continuous-claude-postgres psql -U claude -d continuous_claude -c \
  "SELECT id, working_on, last_heartbeat,
   EXTRACT(EPOCH FROM (NOW() - last_heartbeat)) as staleness
   FROM sessions
   WHERE last_heartbeat > NOW() - INTERVAL '2 hours'
   ORDER BY last_heartbeat DESC;" 2>/dev/null
```

### 8. Format Output

Group sessions by state and format as:

```
Session Wrangler - Claude Sessions Overview
═══════════════════════════════════════════════════════════════════════

Active Sessions (N)
───────────────────────────────────────────────────────────────────────
  ✅ <slug>
     "<last-prompt-preview...>"
     📂 <worktree> │ 🔀 <branch> │ ⏱️  <time-ago>
     PID: <pid> │ CPU: <cpu>% │ Size: <size>

Stale Sessions (N)
───────────────────────────────────────────────────────────────────────
  ⚠️  <slug>
     "<last-prompt-preview...>"
     📂 <worktree> │ 🔀 <branch> │ ⏱️  <time-ago> (idle)
     PID: <pid> │ Size: <size>

Zombie Sessions (N)
───────────────────────────────────────────────────────────────────────
  ❌ <slug>
     "<last-prompt-preview...>"
     📂 <worktree> │ 🔀 <branch>
     PID: <pid> │ ⚠️  STUCK - needs cleanup

Resumable Sessions (N)
───────────────────────────────────────────────────────────────────────
  💤 <slug> (<short-id>)
     "<last-prompt-preview...>"
     📂 <worktree> │ 🔀 <branch> │ 📅 <date>
     💬 <count> messages │ 📏 <size>
     ▶️  claude --resume <session-id>

Corrupted Sessions (N)
───────────────────────────────────────────────────────────────────────
  🔥 <slug>
     ⚠️  CORRUPTED - cannot resume (Size: <size>)
     📂 <worktree> │ 🔀 <branch>
     💡 Suggest: Archive and start fresh

═══════════════════════════════════════════════════════════════════════
Total: <count> sessions | Active: <n> | Stuck: <n> | Resumable: <n>

💡 Quick Actions:
  /sessions.inspect <slug>    - Deep dive into a session
  /sessions.revive <slug>     - Revive a stuck/resumable session
  /sessions.cleanup           - Clean up zombies and corrupted files
  /sessions.switch <slug>     - Jump to another session's terminal
```

## Special Cases

### No Sessions Found
```
No Claude sessions found.

This could mean:
  • No sessions have been started yet
  • Session directory doesn't exist
  • You're in a different project

Expected location: ~/.claude/projects/-Users-kobik-private-workspace-caro/
```

### PostgreSQL Unavailable
Continue with file-based discovery only. Add note:
```
ℹ️  PostgreSQL unavailable - showing file-based data only
   For more accurate heartbeat data, ensure continuous-claude-postgres container is running
```

### jq Not Available
Fall back to grep/awk parsing. Add note:
```
⚠️  jq not installed - using basic parsing (may be less accurate)
   Install with: brew install jq
```

## Performance Considerations

- Limit file reads to first/last 20 lines for metadata extraction
- Cache worktree list (read once, use for all sessions)
- Skip PostgreSQL query if container check fails quickly
- Truncate long prompts to 50 chars for display
- Process sessions in parallel if > 10 sessions found

## Error Handling

- **Permission denied on session file**: Skip and note in output
- **Corrupted JSON**: Try line-by-line parsing as fallback
- **Missing git repository**: Skip worktree mapping
- **Stat command varies by OS**: Try both `stat -f%m` (macOS) and `stat -c%Y` (Linux)

## User Interaction

After displaying results:
- If zombies found: Suggest `/sessions.cleanup`
- If many stale sessions: Suggest reviewing which to keep
- If resumable sessions: Highlight how to resume
- If all sessions active: Show "All systems operational ✅"

## Next Steps

Based on output, suggest:
- "Run `/sessions.inspect <slug>` to understand what a session was doing"
- "Run `/sessions.cleanup` to remove N zombie processes"
- "Run `/sessions.revive <slug>` to restart work in a resumable session"

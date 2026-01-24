# Session Discovery Skill

This skill teaches Claude how to discover, analyze, and understand Claude Code sessions across multiple terminals and worktrees.

## Core Capabilities

### 1. Finding Running Sessions

**Get all Claude processes:**
```bash
ps aux | grep "claude" | grep -v grep | awk '{print $2, $9, $10, $11}'
```

**Parse output:**
- Column 1: PID
- Column 2: Start time
- Column 3: CPU %
- Column 4+: Command

**Interpreting process state:**
- Process running with recent CPU activity = Active
- Process running with 0% CPU for 5+ min = Potentially stuck
- No process found = Either not running or zombie

### 2. Finding Session Files

**List all session files with sizes:**
```bash
ls -lh ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl 2>/dev/null | awk '{print $5, $9}'
```

**Health indicators:**
- File size < 10MB = Healthy
- File size 10-50MB = Large but recoverable
- File size > 50MB = Likely corrupted, may not resume

**Check for corruption:**
```bash
# Sample first and last messages to check for null content
head -1 <session-file>.jsonl | jq '.content'
tail -1 <session-file>.jsonl | jq '.content'
```

If content fields are `null`, the session is corrupted.

### 3. Getting Last Activity

**From history file:**
```bash
grep '"sessionId"' ~/.claude/history.jsonl | grep '<session-id>' | tail -1 | jq -r '.timestamp, .display'
```

**From session file metadata:**
```bash
head -5 <session>.jsonl | jq -r 'select(.type == "session_metadata") | .slug, .gitBranch, .cwd'
```

**Calculate staleness:**
- Compare last timestamp to current time
- 0-5 min = Active
- 5-30 min = Recently active
- 30+ min = Stale
- No recent activity = Abandoned

### 4. Mapping to Worktrees

**List all worktrees:**
```bash
git worktree list --porcelain
```

**Parse worktree data:**
- `worktree` line = directory path
- `branch` line = branch name
- Match session's `cwd` field to worktree path
- Match session's `gitBranch` to worktree branch

**Cross-reference logic:**
```bash
# Get session's cwd and branch
SESSION_CWD=$(head -10 <session>.jsonl | jq -r 'select(.cwd) | .cwd' | head -1)
SESSION_BRANCH=$(head -10 <session>.jsonl | jq -r 'select(.gitBranch) | .gitBranch' | head -1)

# Find matching worktree
git worktree list | grep "$SESSION_CWD"
```

### 5. Checking Session Health

**Run all health checks:**
```bash
# 1. File size check
FILE_SIZE=$(stat -f%z <session>.jsonl 2>/dev/null || stat -c%s <session>.jsonl)
if [ $FILE_SIZE -gt 52428800 ]; then
  echo "❌ CORRUPTED: File too large ($FILE_SIZE bytes)"
fi

# 2. Content check
CONTENT_NULL=$(grep -c '"content":null' <session>.jsonl || echo 0)
if [ $CONTENT_NULL -gt 10 ]; then
  echo "❌ CORRUPTED: Null content entries found ($CONTENT_NULL)"
fi

# 3. Process check
PID=$(ps aux | grep "claude.*$SESSION_ID" | grep -v grep | awk '{print $2}')
if [ -n "$PID" ]; then
  echo "✅ RUNNING: PID $PID"
else
  echo "💤 STOPPED: No process found"
fi

# 4. Heartbeat check (from PostgreSQL if available)
docker exec continuous-claude-postgres psql -U claude -d continuous_claude -c \
  "SELECT last_heartbeat FROM sessions WHERE id = '$SESSION_ID';" 2>/dev/null
```

### 6. Extracting Session Context

**Get last user prompt:**
```bash
grep '"role":"user"' <session>.jsonl | tail -1 | jq -r '.content[0].text' | head -c 200
```

**Get last assistant response:**
```bash
grep '"role":"assistant"' <session>.jsonl | tail -1 | jq -r '.content[0].text' | head -c 200
```

**Count message exchanges:**
```bash
grep -c '"role":"user"' <session>.jsonl
```

**Get session working directory:**
```bash
head -20 <session>.jsonl | jq -r 'select(.cwd) | .cwd' | head -1
```

## Session States

| State | Indicator | Description |
|-------|-----------|-------------|
| ✅ **Active** | Process running, CPU > 0%, heartbeat < 2min | Session is actively working |
| ⚠️ **Stale** | Process running, CPU = 0%, no activity 5-30min | Session idle but recoverable |
| ❌ **Zombie** | Process running, file corrupted OR no heartbeat 30+min | Process stuck, needs cleanup |
| 💤 **Resumable** | No process, file healthy, size < 10MB | Can be resumed with `claude --resume` |
| 🔥 **Corrupted** | File > 50MB OR null content entries | Cannot be resumed, data lost |
| 📦 **Archived** | File moved to archive, no process | Historical session, read-only |

## Common Issues and Diagnosis

### Issue: Session Won't Resume
**Symptoms:** `claude --resume <id>` fails or hangs
**Diagnosis:**
1. Check file size: `ls -lh <session>.jsonl`
2. Check for null content: `grep '"content":null' <session>.jsonl | head`
3. Check file permissions: `ls -l <session>.jsonl`

**Solutions:**
- If > 50MB: File corrupted, cannot resume
- If null content: Corruption, cannot resume
- If permissions: `chmod 644 <session>.jsonl`

### Issue: Zombie Process
**Symptoms:** Process exists but doesn't respond
**Diagnosis:**
1. Check CPU usage: `ps aux | grep <PID>`
2. Check for hung syscalls: `lsof -p <PID>`
3. Check memory: `ps -o rss,vsz,pid,comm | grep <PID>`

**Solutions:**
- Try graceful stop: `kill -TERM <PID>`, wait 10s
- Force kill if needed: `kill -9 <PID>`
- Clean up session file if corrupted

### Issue: Lost Worktree Context
**Symptoms:** Can't find which worktree a session was in
**Diagnosis:**
1. Check session metadata: `head -20 <session>.jsonl | jq '.cwd, .gitBranch'`
2. List worktrees: `git worktree list`
3. Check for deleted worktrees: Compare paths

**Solutions:**
- If worktree exists: Resume session in that directory
- If worktree deleted: Create new worktree for the branch
- If branch deleted: Start fresh session

## Advanced Queries

### Find sessions by worktree:
```bash
for session in ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl; do
  CWD=$(head -20 "$session" | jq -r 'select(.cwd) | .cwd' | head -1)
  if [[ "$CWD" == *"$WORKTREE_NAME"* ]]; then
    echo "$(basename $session): $CWD"
  fi
done
```

### Find sessions by branch:
```bash
for session in ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl; do
  BRANCH=$(head -20 "$session" | jq -r 'select(.gitBranch) | .gitBranch' | head -1)
  if [[ "$BRANCH" == "$TARGET_BRANCH" ]]; then
    echo "$(basename $session): $BRANCH"
  fi
done
```

### Find sessions by last prompt keyword:
```bash
grep -l "keyword" ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl
```

### Get session size statistics:
```bash
ls -lh ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl | \
  awk '{print $5}' | \
  sort -h | \
  awk 'BEGIN {print "Min\tMedian\tMax"}
       {a[NR]=$1}
       END {print a[1]"\t"a[int(NR/2)]"\t"a[NR]}'
```

## Integration with PostgreSQL

If the continuous-claude-postgres container is running, you can query the sessions table:

```bash
# Check if container is running
docker ps | grep continuous-claude-postgres

# Query active sessions
docker exec continuous-claude-postgres psql -U claude -d continuous_claude -c \
  "SELECT id, project, working_on, last_heartbeat,
   EXTRACT(EPOCH FROM (NOW() - last_heartbeat)) as seconds_since_heartbeat
   FROM sessions
   WHERE last_heartbeat > NOW() - INTERVAL '1 hour'
   ORDER BY last_heartbeat DESC;"

# Find stale sessions
docker exec continuous-claude-postgres psql -U claude -d continuous_claude -c \
  "SELECT id, working_on, last_heartbeat
   FROM sessions
   WHERE last_heartbeat < NOW() - INTERVAL '30 minutes'
   AND last_heartbeat > NOW() - INTERVAL '24 hours';"
```

## Best Practices

1. **Always check file health before attempting resume** - Corrupted files waste time
2. **Cross-reference multiple data sources** - Process + file + DB gives complete picture
3. **Handle missing data gracefully** - Sessions may be partially corrupted
4. **Assume sessions can be in any state** - Don't assume healthy state
5. **Provide actionable diagnosis** - Don't just report status, explain what to do
6. **Protect user data** - Never suggest deleting sessions without explicit confirmation
7. **Use relative session IDs** - Users can refer to sessions by slug or short ID
8. **Format output for readability** - Tables, colors, emojis help parsing

## Output Format Standards

### For /sessions command:
```
# Active Sessions (N)
  ✅ <slug>  <last-prompt-preview>  │ <worktree>  │ <time-ago>
  ⚠️ <slug>  <last-prompt-preview>  │ <worktree>  │ <time-ago> (stale)
  ❌ <slug>  <last-prompt-preview>  │ <worktree>  │ ZOMBIE (PID <pid>)

# Resumable Sessions (N)
  💤 <short-id>  <last-prompt-preview>  │ <messages> msgs │ <date>
```

### For /sessions.inspect command:
```
Session: <full-id>
Status: <icon> <state>
Slug: <slug>
Last Activity: <timestamp>

Last Prompt: "<truncated-preview>"
Last Response: "<truncated-preview>"

Working Directory: <cwd>
Git Branch: <branch>
Worktree: <worktree-path>

File Stats:
  Size: <size>
  Messages: <count>
  Created: <timestamp>

Process:
  PID: <pid> (or "Not running")
  CPU: <usage>
  Memory: <usage>

Health Checks:
  ✅/❌ File size < 10MB
  ✅/❌ Content not null
  ✅/❌ Process responding
  ✅/❌ Worktree exists

Diagnosis:
  <human-readable assessment>

Recommended Action:
  <specific next step>
```

## Error Handling

- **Missing jq**: Fallback to grep/awk parsing
- **Missing session file**: Report as deleted/archived
- **Permission denied**: Report and suggest fixing permissions
- **PostgreSQL unavailable**: Skip DB queries, use file-based discovery only
- **Corrupted JSON**: Use line-by-line parsing instead of jq
- **Git worktree errors**: Suggest running `git worktree prune`

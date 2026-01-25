# /sessions.inspect - Deep Dive into a Session

Perform comprehensive analysis of a specific Claude session to understand its state, health, and context.

## Purpose

Help users understand:
- What the session was working on
- Why it might be stuck or corrupted
- Whether it can be recovered
- What the recommended action is

## Usage

```bash
/sessions.inspect <session-identifier>
```

Where `<session-identifier>` can be:
- Full session ID: `2893aa9b-69d3-4167-9ad1-4c5a06697d93`
- Short ID (first 8 chars): `2893aa9b`
- Session slug: `warm-floating-pancake`
- Worktree name: `045-milestone-coordinator-plugin`
- Branch name: `feature/i18n-complete-system`

## Execution Steps

### 1. Resolve Session Identifier

```bash
# If full UUID provided
if [[ "$IDENTIFIER" =~ ^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$ ]]; then
  SESSION_ID="$IDENTIFIER"
  SESSION_FILE=~/.claude/projects/-Users-kobik-private-workspace-caro/${SESSION_ID}.jsonl
fi

# If short ID provided
if [[ "$IDENTIFIER" =~ ^[a-f0-9]{8}$ ]]; then
  SESSION_FILE=$(ls ~/.claude/projects/-Users-kobik-private-workspace-caro/${IDENTIFIER}*.jsonl 2>/dev/null | head -1)
  SESSION_ID=$(basename "$SESSION_FILE" .jsonl)
fi

# If slug provided
if [[ ! "$IDENTIFIER" =~ ^[a-f0-9] ]]; then
  # Search for slug in session files
  for file in ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl; do
    SLUG=$(head -20 "$file" | jq -r 'select(.slug) | .slug' | head -1)
    if [[ "$SLUG" == "$IDENTIFIER" ]]; then
      SESSION_FILE="$file"
      SESSION_ID=$(basename "$file" .jsonl)
      break
    fi
  done
fi

# If worktree/branch provided
if [[ -z "$SESSION_FILE" ]]; then
  # Search by cwd or gitBranch
  for file in ~/.claude/projects/-Users-kobik-private-workspace-caro/*.jsonl; do
    CWD=$(head -20 "$file" | jq -r 'select(.cwd) | .cwd' | head -1)
    BRANCH=$(head -20 "$file" | jq -r 'select(.gitBranch) | .gitBranch' | head -1)
    if [[ "$CWD" == *"$IDENTIFIER"* ]] || [[ "$BRANCH" == *"$IDENTIFIER"* ]]; then
      SESSION_FILE="$file"
      SESSION_ID=$(basename "$file" .jsonl)
      break
    fi
  done
fi
```

### 2. Extract Session Metadata

```bash
# Basic info
SLUG=$(head -20 "$SESSION_FILE" | jq -r 'select(.slug) | .slug' | head -1)
BRANCH=$(head -20 "$SESSION_FILE" | jq -r 'select(.gitBranch) | .gitBranch' | head -1)
CWD=$(head -20 "$SESSION_FILE" | jq -r 'select(.cwd) | .cwd' | head -1)

# Creation time
CREATED=$(stat -f%B "$SESSION_FILE" 2>/dev/null || stat -c%W "$SESSION_FILE")
CREATED_DATE=$(date -r "$CREATED" "+%Y-%m-%d %H:%M" 2>/dev/null || date -d "@$CREATED" "+%Y-%m-%d %H:%M")

# Last modified
LAST_MOD=$(stat -f%m "$SESSION_FILE" 2>/dev/null || stat -c%Y "$SESSION_FILE")
LAST_MOD_DATE=$(date -r "$LAST_MOD" "+%Y-%m-%d %H:%M" 2>/dev/null || date -d "@$LAST_MOD" "+%Y-%m-%d %H:%M")

# File size
FILE_SIZE=$(stat -f%z "$SESSION_FILE" 2>/dev/null || stat -c%s "$SESSION_FILE")
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)

# Message count
USER_MSG_COUNT=$(grep -c '"role":"user"' "$SESSION_FILE")
ASST_MSG_COUNT=$(grep -c '"role":"assistant"' "$SESSION_FILE")
TOTAL_MSG_COUNT=$((USER_MSG_COUNT + ASST_MSG_COUNT))
```

### 3. Get Last Prompt and Response

```bash
# Last user prompt (with proper escaping and truncation)
LAST_PROMPT=$(grep '"role":"user"' "$SESSION_FILE" | tail -1 | jq -r '.content[0].text' 2>/dev/null | head -c 300)
if [ -z "$LAST_PROMPT" ]; then
  LAST_PROMPT="[Unable to extract - possible corruption]"
fi

# Last assistant response
LAST_RESPONSE=$(grep '"role":"assistant"' "$SESSION_FILE" | tail -1 | jq -r '.content[0].text' 2>/dev/null | head -c 300)
if [ -z "$LAST_RESPONSE" ]; then
  LAST_RESPONSE="[No response recorded]"
fi
```

### 4. Check Process State

```bash
# Find process by session ID
PID=$(ps aux | grep "[c]laude.*--resume $SESSION_ID" | awk '{print $2}' | head -1)

if [ -n "$PID" ]; then
  # Get process details
  CPU=$(ps aux | grep "^[^ ]* *$PID " | awk '{print $3}')
  MEM=$(ps aux | grep "^[^ ]* *$PID " | awk '{print $4}')
  START=$(ps aux | grep "^[^ ]* *$PID " | awk '{print $9}')

  # Check if process is responsive
  if [ "$CPU" == "0.0" ]; then
    PROCESS_STATUS="🟡 Running but idle (CPU 0%)"
  else
    PROCESS_STATUS="🟢 Running and active"
  fi
else
  PROCESS_STATUS="🔴 Not running"
fi
```

### 5. Check File Health

```bash
# File size check
if [ "$FILE_SIZE" -gt 52428800 ]; then  # 50MB
  SIZE_STATUS="❌ File too large (${FILE_SIZE_MB}MB) - likely corrupted"
elif [ "$FILE_SIZE" -gt 10485760 ]; then  # 10MB
  SIZE_STATUS="⚠️  Large file (${FILE_SIZE_MB}MB) - may be slow to resume"
else
  SIZE_STATUS="✅ Healthy size (${FILE_SIZE_MB}MB)"
fi

# Content null check
NULL_COUNT=$(grep -c '"content":null' "$SESSION_FILE" 2>/dev/null || echo 0)
if [ "$NULL_COUNT" -gt 10 ]; then
  CONTENT_STATUS="❌ High null content count ($NULL_COUNT) - corrupted"
else
  CONTENT_STATUS="✅ Content appears valid"
fi

# JSON structure check
if head -1 "$SESSION_FILE" | jq empty 2>/dev/null; then
  JSON_STATUS="✅ Valid JSON structure"
else
  JSON_STATUS="❌ Invalid JSON - file corrupted"
fi
```

### 6. Check Worktree State

```bash
# Find matching worktree
WORKTREE_PATH=$(git -C /Users/kobik-private/workspace/caro worktree list | grep "$CWD" | awk '{print $1}')

if [ -n "$WORKTREE_PATH" ] && [ -d "$WORKTREE_PATH" ]; then
  WORKTREE_STATUS="✅ Exists: $WORKTREE_PATH"

  # Check for uncommitted work
  cd "$WORKTREE_PATH"
  if [ -n "$(git status --porcelain)" ]; then
    UNCOMMITTED=$(git status --porcelain | wc -l | tr -d ' ')
    WORKTREE_STATUS="$WORKTREE_STATUS\n     ⚠️  $UNCOMMITTED uncommitted changes"
  fi
else
  WORKTREE_STATUS="❌ Worktree deleted or moved"
fi
```

### 7. Query PostgreSQL (Optional)

```bash
if docker ps | grep -q continuous-claude-postgres; then
  DB_DATA=$(docker exec continuous-claude-postgres psql -U claude -d continuous_claude -t -c \
    "SELECT working_on, last_heartbeat,
     EXTRACT(EPOCH FROM (NOW() - last_heartbeat)) as staleness
     FROM sessions WHERE id = '$SESSION_ID';" 2>/dev/null)

  if [ -n "$DB_DATA" ]; then
    WORKING_ON=$(echo "$DB_DATA" | awk -F'|' '{print $1}' | xargs)
    HEARTBEAT=$(echo "$DB_DATA" | awk -F'|' '{print $2}' | xargs)
    STALENESS=$(echo "$DB_DATA" | awk -F'|' '{print $3}' | xargs | cut -d. -f1)

    if [ "$STALENESS" -lt 300 ]; then
      HEARTBEAT_STATUS="✅ Recent heartbeat (${STALENESS}s ago)"
    elif [ "$STALENESS" -lt 1800 ]; then
      HEARTBEAT_STATUS="⚠️  Stale heartbeat (${STALENESS}s ago)"
    else
      HEARTBEAT_STATUS="❌ Very stale heartbeat (${STALENESS}s ago)"
    fi
  fi
fi
```

### 8. Determine Overall State

```bash
# Calculate overall health
if [[ "$SIZE_STATUS" == *"❌"* ]] || [[ "$CONTENT_STATUS" == *"❌"* ]] || [[ "$JSON_STATUS" == *"❌"* ]]; then
  OVERALL_STATE="🔥 CORRUPTED"
  CAN_RESUME="No"
elif [[ "$PROCESS_STATUS" == *"🔴"* ]] && [[ "$SIZE_STATUS" == *"✅"* ]]; then
  OVERALL_STATE="💤 RESUMABLE"
  CAN_RESUME="Yes"
elif [[ "$PROCESS_STATUS" == *"🟢"* ]]; then
  OVERALL_STATE="✅ ACTIVE"
  CAN_RESUME="Already running"
elif [[ "$PROCESS_STATUS" == *"🟡"* ]]; then
  OVERALL_STATE="⚠️  STALE"
  CAN_RESUME="Running but idle"
else
  OVERALL_STATE="❓ UNKNOWN"
  CAN_RESUME="Uncertain"
fi
```

### 9. Generate Diagnosis

```bash
# Create human-readable diagnosis based on health checks
DIAGNOSIS=""

if [[ "$FILE_SIZE" -gt 52428800 ]]; then
  DIAGNOSIS+="• Session file exceeds 50MB - indicates runaway context growth or corruption\n"
fi

if [ "$NULL_COUNT" -gt 10 ]; then
  DIAGNOSIS+="• High number of null content entries - data loss occurred\n"
fi

if [[ "$PROCESS_STATUS" == *"idle"* ]]; then
  DIAGNOSIS+="• Process running but using 0% CPU - may be hung or waiting for input\n"
fi

if [[ "$WORKTREE_STATUS" == *"deleted"* ]]; then
  DIAGNOSIS+="• Associated worktree no longer exists - branch may have been merged/deleted\n"
fi

if [[ "$WORKTREE_STATUS" == *"uncommitted"* ]]; then
  DIAGNOSIS+="• Uncommitted work exists in worktree - important not to lose this\n"
fi

if [ -n "$STALENESS" ] && [ "$STALENESS" -gt 1800 ]; then
  DIAGNOSIS+="• No heartbeat for over 30 minutes - session likely abandoned or stuck\n"
fi

if [ -z "$DIAGNOSIS" ]; then
  DIAGNOSIS="• No obvious issues detected"
fi
```

### 10. Recommend Action

```bash
# Generate specific recommendation
RECOMMENDATION=""

if [[ "$OVERALL_STATE" == "🔥 CORRUPTED" ]]; then
  RECOMMENDATION="Session cannot be resumed due to corruption. Options:
  1. Start fresh session in the same worktree
  2. Archive this session file for reference
  3. If uncommitted work exists, commit it first

  Command: cd $WORKTREE_PATH && claude"

elif [[ "$OVERALL_STATE" == "💤 RESUMABLE" ]]; then
  RECOMMENDATION="Session can be resumed. Use:

  /sessions.revive $SLUG

  This will spawn a terminal in the correct worktree and provide resume instructions."

elif [[ "$OVERALL_STATE" == "⚠️  STALE" ]]; then
  RECOMMENDATION="Session is running but idle. Options:
  1. Check terminal - may be waiting for user input
  2. If terminal not found, kill process and resume cleanly

  To kill: kill -TERM $PID
  Then: /sessions.revive $SLUG"

elif [[ "$OVERALL_STATE" == "✅ ACTIVE" ]]; then
  RECOMMENDATION="Session is healthy and running. Use /sessions.switch to navigate to its terminal."

else
  RECOMMENDATION="State unclear. Manual investigation needed. Check session file and process manually."
fi
```

### 11. Format Output

```
═══════════════════════════════════════════════════════════════════════
Session Inspection Report
═══════════════════════════════════════════════════════════════════════

Session ID: <full-session-id>
State: <OVERALL_STATE>
Slug: <slug>
Can Resume: <can-resume>

Last Activity
─────────────────────────────────────────────────────────────────────
Created: <created-date>
Last Modified: <last-mod-date>
Time Since Activity: <time-ago>

Last User Prompt:
"<last-prompt-truncated>..."

Last Assistant Response:
"<last-response-truncated>..."

Working Context
─────────────────────────────────────────────────────────────────────
Directory: <cwd>
Git Branch: <branch>
Worktree: <worktree-status>
<if uncommitted work, list files>

File Statistics
─────────────────────────────────────────────────────────────────────
Size: <size-mb>MB (<size-bytes> bytes)
Messages: <total-count> (<user-count> user, <asst-count> assistant)
File Health: <size-status>
Content Health: <content-status>
JSON Structure: <json-status>

Process Status
─────────────────────────────────────────────────────────────────────
<process-status>
<if running:>
  PID: <pid>
  CPU: <cpu>%
  Memory: <mem>%
  Started: <start-time>

<if PostgreSQL available:>
Database Info
─────────────────────────────────────────────────────────────────────
Working On: <working-on>
Last Heartbeat: <heartbeat>
Heartbeat Status: <heartbeat-status>

Health Checks
─────────────────────────────────────────────────────────────────────
<size-status>
<content-status>
<json-status>
<worktree-status>
<if process running: process-status>
<if db available: heartbeat-status>

Diagnosis
─────────────────────────────────────────────────────────────────────
<diagnosis-points>

Recommended Action
─────────────────────────────────────────────────────────────────────
<recommendation-text>

═══════════════════════════════════════════════════════════════════════

💡 Next Steps:
  /sessions.revive <slug>   - Revive this session
  /sessions.cleanup         - Clean up if zombie
  /sessions                 - Back to session list
```

## Special Cases

### Session Not Found
```
❌ Session not found: "<identifier>"

Searched for:
  • Full session ID
  • Short ID (first 8 characters)
  • Session slug
  • Worktree name
  • Branch name

Available sessions:
<list sessions from /sessions command>

Use: /sessions to see all available sessions
```

### Multiple Matches
```
⚠️  Multiple sessions match "<identifier>":

1. <slug1> (<short-id1>) - <worktree1>
2. <slug2> (<short-id2>) - <worktree2>

Please specify which session using:
  • Full session ID
  • Unique slug
  • /sessions.inspect <number> (e.g., /sessions.inspect 1)
```

## Performance Notes

- Read only first 20 and last 5 lines of session file for metadata
- Skip deep content analysis if file > 50MB
- Cache worktree list across multiple inspections
- Timeout PostgreSQL query after 2s

## Error Handling

- **File unreadable**: Report permission issue and suggest `chmod 644`
- **jq missing**: Fall back to grep/awk parsing with note
- **Git not available**: Skip worktree checks
- **Process info unavailable**: Use file-based data only

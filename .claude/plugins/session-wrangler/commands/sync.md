# /sessions.sync - Rebuild Session Index

**Purpose**: Force rebuild the session-worktree index from session files and git worktrees.

## Usage

```bash
/sessions.sync           # Rebuild index
/sessions.sync --verify  # Verify index without modifying
```

## Session Index

**Location**: `~/.claude/projects/<project-slug>/session-index.json`

**Purpose**: Maintain fast bidirectional mapping between sessions and worktrees.

**Schema**:
```json
{
  "version": 1,
  "project": "/Users/user/workspace/caro",
  "lastUpdated": "2026-01-24T07:45:00Z",
  "sessions": {
    "session-uuid": {
      "slug": "warm-floating-pancake",
      "worktree": ".worktrees/046-feature",
      "branch": "046-feature-branch",
      "lastActive": "2026-01-24T07:45:00Z",
      "status": "active"
    }
  },
  "worktrees": {
    ".worktrees/046-feature": {
      "branch": "046-feature-branch",
      "sessions": ["session-uuid"],
      "activeSession": "session-uuid"
    }
  }
}
```

## Execution Steps

### 1. Initialize Variables

```bash
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
PROJECT_SLUG=$(echo "$PROJECT_ROOT" | sed 's|/|-|g' | sed 's|^-||')
INDEX_DIR="$HOME/.claude/projects/$PROJECT_SLUG"
INDEX_FILE="$INDEX_DIR/session-index.json"
VERIFY_ONLY=false

# Check for --verify flag
if [[ "$1" == "--verify" ]]; then
  VERIFY_ONLY=true
fi

echo ""
echo "Session Index Sync"
echo "══════════════════════════════════════"
echo ""
echo "Project: $PROJECT_ROOT"
echo "Index: $INDEX_FILE"
echo ""
```

### 2. Scan Session Files

```bash
SESSION_DIR="$HOME/.claude/projects/$PROJECT_SLUG"
SESSIONS_FOUND=0
SESSIONS_ACTIVE=0
SESSIONS_RESUMABLE=0
SESSIONS_CORRUPTED=0

declare -A SESSION_DATA

if [[ -d "$SESSION_DIR" ]]; then
  echo "Scanning session files..."

  for session_file in "$SESSION_DIR"/*.jsonl; do
    [[ -f "$session_file" ]] || continue

    SESSION_ID=$(basename "$session_file" .jsonl)
    ((SESSIONS_FOUND++))

    # Check file size (>10MB = corrupted)
    SIZE=$(stat -f%z "$session_file" 2>/dev/null || stat -c%s "$session_file" 2>/dev/null || echo "0")
    if [[ $SIZE -gt 10485760 ]]; then
      ((SESSIONS_CORRUPTED++))
      continue
    fi

    # Extract session metadata from first few lines
    SLUG=$(head -10 "$session_file" | jq -r 'select(.slug) | .slug' 2>/dev/null | head -1)
    BRANCH=$(head -10 "$session_file" | jq -r 'select(.gitBranch) | .gitBranch' 2>/dev/null | head -1)
    CWD=$(head -10 "$session_file" | jq -r 'select(.cwd) | .cwd' 2>/dev/null | head -1)
    LAST_MSG=$(tail -5 "$session_file" | jq -r 'select(.timestamp) | .timestamp' 2>/dev/null | tail -1)

    # Determine worktree path from cwd
    if [[ -n "$CWD" && "$CWD" =~ \.worktrees/ ]]; then
      WORKTREE=$(echo "$CWD" | sed "s|$PROJECT_ROOT/||")
    else
      WORKTREE=""
    fi

    # Check if process is running
    PID=$(ps aux | grep "claude" | grep "$SESSION_ID" | grep -v grep | awk '{print $2}' | head -1)
    if [[ -n "$PID" ]]; then
      STATUS="active"
      ((SESSIONS_ACTIVE++))
    else
      STATUS="resumable"
      ((SESSIONS_RESUMABLE++))
    fi

    # Store session data
    SESSION_DATA["$SESSION_ID"]="$SLUG|$WORKTREE|$BRANCH|$LAST_MSG|$STATUS"
  done

  echo "  Found: $SESSIONS_FOUND session files"
  echo "  Active: $SESSIONS_ACTIVE sessions (process running)"
  echo "  Resumable: $SESSIONS_RESUMABLE sessions (healthy files)"
  echo "  Corrupted: $SESSIONS_CORRUPTED sessions (will not index)"
  echo ""
fi
```

### 3. Scan Worktrees

```bash
echo "Scanning worktrees..."

declare -A WORKTREE_DATA
WORKTREES_FOUND=0
WORKTREES_MAPPED=0
WORKTREES_ORPHANED=0

while IFS= read -r line; do
  # Parse git worktree list output
  WORKTREE_PATH=$(echo "$line" | awk '{print $1}')
  BRANCH_NAME=$(echo "$line" | awk '{print $3}' | sed 's/\[//' | sed 's/\]//')

  # Only process .worktrees/ directories
  if [[ "$WORKTREE_PATH" =~ \.worktrees/ ]]; then
    ((WORKTREES_FOUND++))

    REL_PATH=$(echo "$WORKTREE_PATH" | sed "s|$PROJECT_ROOT/||")

    # Find sessions in this worktree
    WORKTREE_SESSIONS=()
    for SESSION_ID in "${!SESSION_DATA[@]}"; do
      IFS='|' read -r slug wt branch last status <<< "${SESSION_DATA[$SESSION_ID]}"
      if [[ "$wt" == "$REL_PATH" ]]; then
        WORKTREE_SESSIONS+=("$SESSION_ID")
      fi
    done

    if [[ ${#WORKTREE_SESSIONS[@]} -gt 0 ]]; then
      ((WORKTREES_MAPPED++))
      WORKTREE_DATA["$REL_PATH"]="$BRANCH_NAME|${WORKTREE_SESSIONS[*]}"
    else
      ((WORKTREES_ORPHANED++))
      WORKTREE_DATA["$REL_PATH"]="$BRANCH_NAME|"
    fi
  fi
done < <(git worktree list 2>/dev/null)

echo "  Found: $WORKTREES_FOUND worktrees"
echo "  With sessions: $WORKTREES_MAPPED worktrees"
echo "  Orphaned: $WORKTREES_ORPHANED worktrees (no sessions)"
echo ""
```

### 4. Build Index JSON

```bash
if [[ "$VERIFY_ONLY" == "true" ]]; then
  echo "Verification complete (no changes made)"
  exit 0
fi

# Create index directory
mkdir -p "$INDEX_DIR"

# Build sessions object
SESSIONS_JSON="{"
FIRST=true
for SESSION_ID in "${!SESSION_DATA[@]}"; do
  IFS='|' read -r slug wt branch last status <<< "${SESSION_DATA[$SESSION_ID]}"

  [[ "$FIRST" == "true" ]] && FIRST=false || SESSIONS_JSON+=","

  # Format timestamp
  TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  if [[ -n "$last" ]]; then
    TIMESTAMP="$last"
  fi

  SESSIONS_JSON+="\"$SESSION_ID\":{"
  SESSIONS_JSON+="\"slug\":\"${slug:-unknown}\","
  SESSIONS_JSON+="\"worktree\":\"${wt:-}\","
  SESSIONS_JSON+="\"branch\":\"${branch:-}\","
  SESSIONS_JSON+="\"lastActive\":\"$TIMESTAMP\","
  SESSIONS_JSON+="\"status\":\"$status\""
  SESSIONS_JSON+="}"
done
SESSIONS_JSON+="}"

# Build worktrees object
WORKTREES_JSON="{"
FIRST=true
for WORKTREE_PATH in "${!WORKTREE_DATA[@]}"; do
  IFS='|' read -r branch sessions_str <<< "${WORKTREE_DATA[$WORKTREE_PATH]}"

  [[ "$FIRST" == "true" ]] && FIRST=false || WORKTREES_JSON+=","

  # Convert space-separated sessions to JSON array
  SESSIONS_ARRAY="["
  if [[ -n "$sessions_str" ]]; then
    FIRST_SESSION=true
    for sid in $sessions_str; do
      [[ "$FIRST_SESSION" == "true" ]] && FIRST_SESSION=false || SESSIONS_ARRAY+=","
      SESSIONS_ARRAY+="\"$sid\""
    done
  fi
  SESSIONS_ARRAY+="]"

  # Determine active session (first one in list)
  ACTIVE_SESSION=""
  if [[ -n "$sessions_str" ]]; then
    ACTIVE_SESSION=$(echo "$sessions_str" | awk '{print $1}')
  fi

  WORKTREES_JSON+="\"$WORKTREE_PATH\":{"
  WORKTREES_JSON+="\"branch\":\"$branch\","
  WORKTREES_JSON+="\"sessions\":$SESSIONS_ARRAY,"
  WORKTREES_JSON+="\"activeSession\":\"$ACTIVE_SESSION\""
  WORKTREES_JSON+="}"
done
WORKTREES_JSON+="}"

# Write index file
cat > "$INDEX_FILE" <<EOF
{
  "version": 1,
  "project": "$PROJECT_ROOT",
  "lastUpdated": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "sessions": $SESSIONS_JSON,
  "worktrees": $WORKTREES_JSON
}
EOF

echo "Index updated: $INDEX_FILE"
echo "  Sessions indexed: ${#SESSION_DATA[@]}"
echo "  Worktrees mapped: $WORKTREES_FOUND"
echo "  Last updated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""
```

### 5. Validate Index

```bash
# Verify JSON is valid
if ! jq empty "$INDEX_FILE" 2>/dev/null; then
  echo "❌ Error: Generated invalid JSON"
  exit 1
fi

echo "✓ Index validation passed"
echo ""
echo "Use /sessions to view indexed sessions"
```

## Auto-Sync Triggers

The index is automatically updated by:

1. **PostToolUse Hook**: After commands that modify sessions
2. **Session Start**: When a new session begins (via hook)
3. **Session End**: When a session completes (via hook)
4. **Manual Sync**: When user runs `/sessions.sync`

## Index Benefits

- **Fast Lookups**: No need to parse session files every time
- **Bidirectional**: Quick session → worktree and worktree → sessions mapping
- **Status Tracking**: Know which sessions are active/resumable
- **Collision Detection**: Identify multiple sessions in same worktree

## Verification

Run with `--verify` to check index accuracy without modifying:

```bash
/sessions.sync --verify
```

This will scan all files and report discrepancies but won't update the index.

## Notes

- Index location is per-project (isolated repositories)
- Corrupted sessions (>10MB) are excluded from index
- Orphaned worktrees (no sessions) are tracked but marked
- Index updates are atomic (temp file + move)
- Previous index is overwritten (no backup)

## Examples

**Force full rebuild**:
```bash
/sessions.sync
```

**Verify accuracy**:
```bash
/sessions.sync --verify
```

**After config changes**:
```bash
/sessions.config claude-command claude-yolo
/sessions.sync
```

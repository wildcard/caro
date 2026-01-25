# /sessions.switch - Navigate to Another Session

Focus and navigate to an existing Claude session's terminal window.

## Purpose

Help users quickly jump between active Claude sessions without manually searching for terminals.

**Key principle:** This command finds and focuses the terminal - it does NOT start a new terminal or resume a session.

## Usage

```bash
/sessions.switch <session-identifier>
```

Where `<session-identifier>` can be:
- Slug: `warm-floating-pancake`
- Short ID: `2893aa9b`
- Worktree: `045-milestone-coordinator-plugin`
- Branch: `feature/i18n-complete-system`

## Pre-Flight Checks

1. **Session must be running** - If not running, suggest `/sessions.revive`
2. **Terminal must exist** - If process exists but no terminal found, report issue
3. **Platform support** - macOS (osascript) vs Linux (wmctrl)

## Execution Steps

### 1. Resolve Session Identifier

```bash
# Use same resolution logic as /sessions.inspect
SESSION_FILE=<resolved-path>
SESSION_ID=$(basename "$SESSION_FILE" .jsonl)

# Get metadata
SLUG=$(head -20 "$SESSION_FILE" | jq -r 'select(.slug) | .slug' | head -1)
CWD=$(head -20 "$SESSION_FILE" | jq -r 'select(.cwd) | .cwd' | head -1)
```

### 2. Check If Session Is Running

```bash
# Find process
PID=$(ps aux | grep "[c]laude.*$SESSION_ID" | awk '{print $2}' | head -1)

if [ -z "$PID" ]; then
  echo "❌ Session Not Running"
  echo ""
  echo "Session: $SLUG"
  echo "This session is not currently active."
  echo ""
  echo "Options:"
  echo "  1. Resume it: /sessions.revive $SLUG"
  echo "  2. Check status: /sessions.inspect $SLUG"
  echo "  3. List all: /sessions"
  exit 1
fi

# Process is running - continue
echo "✅ Session found (PID $PID)"
```

### 3. Find Terminal Window (macOS)

```bash
if [[ "$OSTYPE" == "darwin"* ]]; then
  # Method 1: Find by window title
  WINDOW_TITLE=$(osascript <<EOF 2>/dev/null
    tell application "Alacritty"
      set windowList to every window
      repeat with aWindow in windowList
        set windowTitle to name of aWindow
        if windowTitle contains "$SLUG" then
          return windowTitle
        end if
      end repeat
    end tell
EOF
  )

  # Method 2: Find by working directory
  if [ -z "$WINDOW_TITLE" ]; then
    WINDOW_TITLE=$(osascript <<EOF 2>/dev/null
      tell application "System Events"
        set processList to every process whose name is "Alacritty"
        repeat with aProcess in processList
          set windowList to windows of aProcess
          repeat with aWindow in windowList
            -- Check window properties for CWD match
            -- This is approximate and may not work reliably
          end repeat
        end repeat
      end tell
EOF
    )
  fi

  # Method 3: Find by PID correlation
  if [ -z "$WINDOW_TITLE" ]; then
    # Get terminal sessions associated with Claude PID
    TERMINAL_PID=$(ps -o ppid= -p $PID | xargs)

    if [ -n "$TERMINAL_PID" ]; then
      WINDOW_TITLE=$(osascript <<EOF 2>/dev/null
        tell application "System Events"
          set processList to every process whose unix id is $TERMINAL_PID
          if (count of processList) > 0 then
            set aProcess to item 1 of processList
            if (count of windows of aProcess) > 0 then
              return name of window 1 of aProcess
            end if
          end if
        end tell
EOF
      )
    fi
  fi
fi
```

### 4. Find Terminal Window (Linux)

```bash
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  # Use wmctrl if available
  if command -v wmctrl &> /dev/null; then
    # Find window by title or PID
    WINDOW_ID=$(wmctrl -lp | while read -r line; do
      WIN_ID=$(echo "$line" | awk '{print $1}')
      WIN_PID=$(echo "$line" | awk '{print $3}')
      WIN_TITLE=$(echo "$line" | cut -d' ' -f5-)

      # Match by title
      if [[ "$WIN_TITLE" == *"$SLUG"* ]]; then
        echo "$WIN_ID"
        break
      fi

      # Match by PID correlation
      if [ "$WIN_PID" == "$PID" ] || ps --ppid $WIN_PID | grep -q $PID; then
        echo "$WIN_ID"
        break
      fi
    done)
  fi
fi
```

### 5. Focus Terminal Window

#### macOS Implementation

```bash
if [[ "$OSTYPE" == "darwin"* ]]; then
  if [ -n "$WINDOW_TITLE" ]; then
    osascript <<EOF 2>/dev/null
      tell application "Alacritty"
        activate
        set windowList to every window
        repeat with aWindow in windowList
          if name of aWindow contains "$SLUG" then
            set index of aWindow to 1
            return
          end if
        end repeat
      end tell
EOF

    if [ $? -eq 0 ]; then
      echo "✅ Switched to session: $SLUG"
      echo ""
      echo "The terminal window has been brought to front."
      exit 0
    else
      echo "⚠️  Found terminal but failed to focus it"
    fi
  fi
fi
```

#### Linux Implementation

```bash
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if [ -n "$WINDOW_ID" ]; then
    # Activate window
    wmctrl -ia "$WINDOW_ID"

    if [ $? -eq 0 ]; then
      echo "✅ Switched to session: $SLUG"
      echo ""
      echo "The terminal window has been brought to front."
      exit 0
    else
      echo "⚠️  Found terminal but failed to focus it"
    fi
  fi
fi
```

### 6. Fallback: Manual Instructions

If automatic focusing fails:

```bash
echo "═══════════════════════════════════════════════════════════════════"
echo "Cannot Automatically Switch"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "Session: $SLUG (PID $PID)"
echo "Working Directory: $CWD"
echo ""
echo "The session is running, but I couldn't automatically focus its terminal."
echo ""
echo "Manual steps:"
echo "  1. Look for a terminal window with title: \"Claude: $SLUG\""
echo "  2. Or search your terminal windows for working directory: $CWD"
echo "  3. Use Cmd+Tab (macOS) or Alt+Tab (Linux) to find the window"
echo ""
echo "Process Info:"
ps aux | grep "^[^ ]* *$PID " | awk '{print "  "$11, $12, $13, $14, $15}'
echo ""
echo "If you can't find the terminal:"
echo "  • The process may be running headless (rare)"
echo "  • The terminal may have been closed"
echo "  • Try /sessions.inspect $SLUG for more details"
echo ""
echo "═══════════════════════════════════════════════════════════════════"
```

## Smart Search Strategies

### Strategy 1: Title Matching

Most reliable when sessions use consistent window titles.

```bash
# Expected title format: "Claude: <slug>"
# Also check: "<slug>", "claude <slug>", "<worktree-name>"
```

### Strategy 2: Working Directory

Match terminal's current working directory to session's CWD.

```bash
# On macOS, this requires accessibility permissions
# On Linux, use /proc/$PID/cwd if available
```

### Strategy 3: Process Tree

Trace from Claude PID to parent shell to terminal emulator.

```bash
# Example process tree:
# Alacritty (pid 1000)
#   └─ zsh (pid 1001)
#       └─ claude (pid 1002)  ← Target PID

# Find by walking up the tree
SHELL_PID=$(ps -o ppid= -p $PID | xargs)
TERMINAL_PID=$(ps -o ppid= -p $SHELL_PID | xargs)
```

### Strategy 4: Terminal History

Some terminals maintain session history accessible via API.

```bash
# Alacritty: Check config for socket/API endpoint
# iTerm2: Use AppleScript API
# Kitty: Use remote control socket
```

## Platform-Specific Implementations

### macOS (Alacritty)

```bash
# Primary: AppleScript via osascript
# Pros: Native, reliable, works with most terminals
# Cons: Requires Accessibility permissions for some features

# Fallback: Use Mission Control/Exposé
osascript -e 'tell application "System Events" to key code 160'  # F9 for Exposé
```

### macOS (iTerm2)

```bash
# iTerm2 has rich AppleScript API
osascript <<EOF
  tell application "iTerm2"
    activate
    set sessionList to sessions of current window
    repeat with aSession in sessionList
      if name of aSession contains "$SLUG" then
        select aSession
        return
      end if
    end repeat
  end tell
EOF
```

### Linux (X11)

```bash
# Use wmctrl for window management
wmctrl -ia <window-id>  # Activate window

# Alternative: xdotool
xdotool search --name "$SLUG" windowactivate
```

### Linux (Wayland)

```bash
# Wayland has limited window management
# May need compositor-specific tools

# Sway (i3-like)
swaymsg '[title=".*'$SLUG'.*"] focus'

# GNOME
gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell \
  --method org.gnome.Shell.Eval "global.get_window_actors().forEach(a => { if (a.get_meta_window().get_title().includes('$SLUG')) a.get_meta_window().activate(0); });"
```

## Interactive Mode

If multiple terminals match, let user choose:

```bash
echo "Found multiple terminals for session $SLUG:"
echo ""
echo "1. Alacritty - Window 1 (Claude: $SLUG)"
echo "2. Alacritty - Window 2 (Claude: $SLUG - backup)"
echo "3. iTerm2 - Tab (working on $SLUG)"
echo ""
read -p "Which terminal? (1-3): " CHOICE

case $CHOICE in
  1) WINDOW_ID=<id1> ;;
  2) WINDOW_ID=<id2> ;;
  3) WINDOW_ID=<id3> ;;
  *) echo "Invalid choice"; exit 1 ;;
esac

# Focus chosen window
```

## Status Display

After successful switch:

```
✅ Switched to Session

Session: <slug>
Worktree: <worktree-name>
Branch: <branch>
PID: <pid>

Last Activity: <time-ago>
Context: "<last-prompt-preview...>"

You are now focused on this session's terminal.
```

## Special Cases

### Session Running But No Terminal

```
⚠️  Process Found But No Terminal

Session: <slug>
PID: <pid>
State: Running

The Claude process is active, but I couldn't find its terminal window.

This can happen if:
  • The terminal was closed while Claude kept running
  • Claude was started in background/tmux/screen
  • The terminal is on a different workspace/desktop

Options:
  1. Kill the process and revive: /sessions.cleanup then /sessions.revive
  2. Manually find the terminal using Activity Monitor / top
  3. Check if running in tmux: tmux list-sessions

Process details:
<ps aux output>
```

### Multiple Sessions, Same Worktree

```
⚠️  Ambiguity Detected

Multiple sessions found in worktree: <worktree>

1. <slug-1> (PID <pid1>) - Active 2min ago
2. <slug-2> (PID <pid2>) - Active 45min ago

Which session would you like to switch to? (1-2):
```

### Session on Different Desktop/Space

```
ℹ️  Session May Be On Different Desktop

The session's terminal was found but may be on a different macOS Space/Desktop.

Session: <slug>
Terminal: <window-title>

I've attempted to focus it. If you don't see it:
  • Swipe between Spaces
  • Use Mission Control (F3 or Ctrl+Up)
  • Check "Windows" in dock

The terminal should now be visible on one of your spaces.
```

## Error Handling

| Error | Cause | Recovery |
|-------|-------|----------|
| Session not found | Invalid identifier | Suggest `/sessions` |
| Process not running | Session stopped | Suggest `/sessions.revive` |
| No terminal found | Terminal closed or bg | Provide manual search steps |
| Focus failed | Permission/API issue | Provide manual instructions |
| Platform unsupported | Not macOS/Linux X11 | Explain limitation |

## Performance Notes

- Window search is O(n) where n = open windows
- Cache window list during single command execution
- Timeout window enumeration after 5 seconds
- Use native APIs when available (faster than process parsing)

## Integration

Works well with other commands:

```bash
# Typical workflow
/sessions                    # List all sessions
/sessions.inspect work-123   # Check details
/sessions.switch work-123    # Jump to that session
```

## Success Criteria

- ✅ Terminal window focused and brought to front
- ✅ User can immediately interact with session
- ✅ Works across different terminal emulators
- ✅ Graceful degradation with manual instructions
- ✅ Fast (< 1 second to switch)

## Future Enhancements

- Support for tmux/screen sessions
- Support for more terminal emulators (Kitty, WezTerm, Terminator)
- Support for Windows (Windows Terminal)
- Session history preview before switching
- Keyboard shortcut integration

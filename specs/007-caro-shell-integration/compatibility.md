# Shell Compatibility Matrix

## Overview

This document details the hook points, keybinding approaches, limitations, and fallbacks for each supported shell.

## Compatibility Summary

| Feature | bash | zsh | sh/POSIX | fish |
|---------|------|-----|----------|------|
| Pre-execution hook | DEBUG trap | preexec | PS1 (limited) | fish_preexec |
| Post-execution hook | PROMPT_COMMAND | precmd | PS1 (limited) | fish_postexec |
| Keybindings | bind -x | bindkey + ZLE | N/A | bind |
| Inline suggestions | readline | ZLE widgets | N/A | native |
| Exit code capture | $? | $? | $? | $status |
| Command timing | SECONDS | $EPOCHREALTIME | N/A | $CMD_DURATION |
| History interception | HISTCONTROL | zshaddhistory | N/A | fish_preexec |
| Full support | Yes | Yes | Partial | Yes |

---

## bash Integration

### Hook Mechanisms

#### Pre-Execution: DEBUG Trap
```bash
# DEBUG trap fires before each command
trap '__caro_preexec "$BASH_COMMAND"' DEBUG

# Must handle:
# - Only fire in interactive shells
# - Skip for subshells
# - Skip for PROMPT_COMMAND itself
```

**Caveats:**
- DEBUG trap runs for *every* command in pipeline
- Must check `BASH_SUBSHELL` to avoid subshell triggers
- `set -T` propagates trap to functions (usually unwanted)

#### Post-Execution: PROMPT_COMMAND
```bash
# PROMPT_COMMAND runs before displaying prompt
__caro_precmd() {
    local exit_code=$?
    # Capture last exit code, timing, etc.
    __caro_send_postcmd "$exit_code"
}

# Append to existing PROMPT_COMMAND (don't replace)
PROMPT_COMMAND="__caro_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
```

**Caveats:**
- Must preserve existing PROMPT_COMMAND
- Arrays supported in bash 5.1+ only (`PROMPT_COMMAND=()`)
- Exit code must be captured immediately (before any other command)

### Keybindings
```bash
# bind -x allows shell function execution on keypress
bind -x '"\C-x\C-c": __caro_invoke'      # Ctrl+X Ctrl+C for Caro
bind -x '"\e\e": __caro_fix_last'         # Esc Esc for fix suggestion

# Alternative using readline
bind '"\C-x\C-c": "\C-e \C-u__caro_invoke\n"'
```

**Caveats:**
- bind -x requires bash 4.0+
- Some terminal emulators intercept key sequences
- Must not conflict with common keybindings (Ctrl+C, Ctrl+D)

### Interactive Detection
```bash
# Only enable in interactive shells
[[ $- == *i* ]] || return

# Additional check for login vs non-login
shopt -q login_shell && __caro_is_login=1
```

### Limitations
- No true preexec in bash 3.x (workarounds exist but fragile)
- DEBUG trap can impact performance if misconfigured
- No native async command support

### Fallbacks
- bash 3.x: PS1-based timing only, no true preexec
- If DEBUG trap fails: degrade to PROMPT_COMMAND only (post-exec)

---

## zsh Integration

### Hook Mechanisms

#### Pre-Execution: preexec
```zsh
# preexec runs just before command execution
preexec() {
    # $1 = original command line
    # $2 = expanded command line (after alias expansion)
    # $3 = full command line
    __caro_preexec "$1"
}

# Alternative: add to hook array
autoload -U add-zsh-hook
add-zsh-hook preexec __caro_preexec_hook
```

**Benefits:**
- Native support, very reliable
- Multiple hooks can coexist via add-zsh-hook
- Access to both original and expanded command

#### Post-Execution: precmd
```zsh
precmd() {
    local exit_code=$?
    __caro_postcmd "$exit_code"
}

# Better: use hook array
add-zsh-hook precmd __caro_precmd_hook
```

### Keybindings via ZLE
```zsh
# Define ZLE widget
__caro_invoke_widget() {
    # Save current buffer
    local saved_buffer="$BUFFER"
    local saved_cursor="$CURSOR"

    # Clear line for Caro output
    zle kill-whole-line
    zle -R  # Refresh display

    # Run Caro interactive
    __caro_invoke

    # Restore buffer
    BUFFER="$saved_buffer"
    CURSOR="$saved_cursor"
    zle redisplay
}
zle -N __caro_invoke_widget

# Bind to key sequence
bindkey '^X^C' __caro_invoke_widget  # Ctrl+X Ctrl+C
bindkey '\e\e' __caro_fix_widget     # Esc Esc
```

**Benefits:**
- Full control over command line editing
- Can modify BUFFER directly
- Supports inline suggestions

### Framework Compatibility
```zsh
# Check for oh-my-zsh
if [[ -n "$ZSH" ]]; then
    # Use oh-my-zsh plugin system
fi

# Check for prezto
if [[ -n "$ZPREZTODIR" ]]; then
    # Compatible with prezto hooks
fi

# Check for zinit/zplug
# These use standard zsh hooks, so compatible
```

### Limitations
- Very few - zsh has excellent hook support
- Some themes override precmd (use add-zsh-hook)
- ZLE can be complex for advanced inline UX

### Fallbacks
- If add-zsh-hook unavailable: direct function definition (older zsh)
- If ZLE fails: simple function-based keybinding

---

## sh/POSIX Integration

### Hook Mechanisms

POSIX sh has **no native hooks**. We must use limited workarounds.

#### Approach 1: PS1-based timing
```sh
# Embed command in PS1
__caro_ps1() {
    __caro_postcmd "$?"
}

PS1='$(__caro_ps1)$ '
```

**Severe Limitations:**
- No preexec capability
- Command substitution in PS1 is non-standard (works in dash, not all sh)
- Performance impact from PS1 evaluation

#### Approach 2: Wrapper Function
```sh
# User must explicitly call caro
caro_run() {
    __caro_preexec "$*"
    "$@"
    __caro_postcmd "$?"
}

# Usage: caro_run rm -rf /tmp/foo
```

#### Approach 3: alias wrapper (limited)
```sh
# Only works for specific commands
alias sudo='__caro_sudo_wrapper'

__caro_sudo_wrapper() {
    __caro_preexec "sudo $*"
    command sudo "$@"
    __caro_postcmd "$?"
}
```

### Keybindings
**Not supported** - POSIX sh has no keybinding mechanism.

Users must explicitly invoke Caro.

### Limitations
- No preexec hook
- No keybindings
- No command interception
- Limited PS1 capabilities
- Many sh implementations differ (dash, ash, busybox sh)

### Fallbacks
- Default to "explicit mode": user runs `caro <command>` or `caro fix`
- Post-command analysis via PS1 (where supported)
- Recommend upgrading to bash/zsh for full experience

### sh Compatibility Table

| Implementation | PS1 subst | trap DEBUG | Notes |
|---------------|-----------|------------|-------|
| dash | Yes | No | Default /bin/sh on Debian/Ubuntu |
| ash | Yes | No | Alpine Linux, BusyBox |
| busybox sh | Yes | No | Embedded systems |
| mksh | Yes | Limited | Android default |
| ksh88 | Yes | No | Legacy systems |
| ksh93 | Yes | Yes | Has DEBUG trap |

---

## fish Integration

### Hook Mechanisms

fish has **native event system** - excellent support.

#### Pre-Execution: fish_preexec
```fish
function __caro_preexec --on-event fish_preexec
    set -l cmd $argv[1]
    __caro_send_preexec $cmd
end
```

#### Post-Execution: fish_postexec
```fish
function __caro_postexec --on-event fish_postexec
    set -l cmd $argv[1]
    set -l exit_code $status
    __caro_send_postcmd $exit_code $cmd
end
```

**Benefits:**
- Native event system
- Clean function syntax
- Built-in `$CMD_DURATION` for timing
- `$status` is exit code

### Keybindings
```fish
# fish has excellent keybinding support
function __caro_invoke
    commandline -f kill-whole-line
    __caro_interactive
    commandline -f repaint
end

bind \cx\cc __caro_invoke      # Ctrl+X Ctrl+C
bind \e\e __caro_fix_last      # Esc Esc

# Or use fish's abbr system for expansions
abbr --add crun --position command --function __caro_abbr_expand
```

### Integration with fish features
```fish
# Integrate with autosuggestions
function __caro_suggest --on-event fish_prompt
    # Could enhance fish's autosuggestions with Caro
end

# Integrate with completions
complete -c caro -f -a "(__caro_completions)"
```

### Limitations
- fish scripting is not POSIX (requires separate implementation)
- Some users have heavy customization (fisher, oh-my-fish)
- Older fish versions (< 2.3) have limited event support

### Fallbacks
- fish 2.x: poll-based checking instead of events
- If events fail: function wrappers

---

## Cross-Shell Implementation Strategy

### Shared Components (Rust/Binary)

These are shell-agnostic:
- `caro-shell-hook`: Binary that handles IPC with daemon
- `caro-daemon`: Long-running process
- Protocol definition (JSON over Unix socket)

### Shell-Specific Scripts

```
~/.config/caro/shell/
├── bash.init       # bash integration
├── zsh.init        # zsh integration
├── sh.init         # POSIX sh fallback
├── fish/
│   ├── conf.d/
│   │   └── caro.fish     # Auto-loaded by fish
│   └── functions/
│       └── __caro_*.fish # Helper functions
└── common.sh       # Shared POSIX functions (sourced by bash/zsh)
```

### Detection Logic

```bash
# In installer/runtime
detect_shell() {
    local shell_name=$(basename "$SHELL")
    case "$shell_name" in
        bash) echo "bash" ;;
        zsh)  echo "zsh" ;;
        fish) echo "fish" ;;
        sh|dash|ash) echo "sh" ;;
        *) echo "unknown" ;;
    esac
}
```

### Environment Variables

All shells use common environment variables:
```
CARO_DISABLE=1           # Emergency disable
CARO_SAFETY_LEVEL=off|passive|active
CARO_SOCKET=/path/to/sock
CARO_DEBUG=1             # Debug logging
CARO_NO_COLOR=1          # Disable colors
```

---

## Feature Availability by Shell

| Feature | bash 4+ | bash 3 | zsh | sh | fish |
|---------|---------|--------|-----|-----|------|
| Pre-exec hook | Yes | Limited | Yes | No | Yes |
| Post-exec hook | Yes | Yes | Yes | Limited | Yes |
| Keybindings | Yes | Yes | Yes | No | Yes |
| Inline suggestions | Partial | No | Yes | No | Yes |
| Exit code capture | Yes | Yes | Yes | Yes | Yes |
| Command timing | Yes | Yes | Yes | No | Yes |
| History integration | Yes | Yes | Yes | No | Yes |
| Fix-it suggestions | Yes | Yes | Yes | Manual | Yes |
| Safety warnings | Yes | Yes | Yes | Manual | Yes |

---

## Recommended Minimum Versions

| Shell | Minimum Version | Notes |
|-------|-----------------|-------|
| bash | 4.0 | For bind -x; 4.4+ recommended for arrays |
| zsh | 5.0 | For modern hook system |
| fish | 3.0 | For event system; 3.3+ recommended |
| sh | N/A | POSIX 2008+ compliance |

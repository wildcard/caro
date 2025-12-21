---
applyTo: "**/*.sh,**/setup.sh,**/install.sh,bin/**"
---

# Shell Script Review Instructions

Apply these guidelines when reviewing shell scripts in this repository.

## Code Reachability

### Detect Unreachable Code
```bash
# BAD: Condition can never be true
detect_shell() {
    if [ -n "$BASH_VERSION" ]; then
        shell_config="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        shell_config="$HOME/.zshrc"
    else
        return 1  # Early return
    fi

    # This check is UNREACHABLE - all paths above either set shell_config or return
    if [ -z "$shell_config" ]; then
        echo "Warning: Could not detect shell"
    fi
}

# GOOD: Remove unreachable code
detect_shell() {
    if [ -n "$BASH_VERSION" ]; then
        shell_config="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        shell_config="$HOME/.zshrc"
    else
        echo "Warning: Could not detect shell" >&2
        return 1
    fi
}
```

## Code Duplication

### Extract Repeated Logic to Functions
```bash
# BAD: Duplicated config file selection
if [ -n "$BASH_VERSION" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        config="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        config="$HOME/.bash_profile"
    else
        config="$HOME/.bashrc"
    fi
fi

# Later in same script...
if [ "$1" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        config="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        config="$HOME/.bash_profile"
    else
        config="$HOME/.bashrc"
    fi
fi

# GOOD: Extract to helper function
get_bash_config() {
    if [ -f "$HOME/.bashrc" ]; then
        echo "$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        echo "$HOME/.bash_profile"
    else
        echo "$HOME/.bashrc"
    fi
}

# Use the function
config=$(get_bash_config)
```

## Shell Detection Accuracy

### Document Edge Cases
```bash
# BAD: Incomplete comment about shell detection
# Check if running in bash

# GOOD: Explain the edge case clearly
# Note: When script runs via 'curl | bash' or process substitution,
# BASH_VERSION is set even if user's login shell is zsh or fish.
# This affects config file selection.
if [ -n "$BASH_VERSION" ]; then
    # Running in bash interpreter (may not be user's default shell)
    ...
fi
```

### Handle Subshell Contexts
```bash
# BAD: Assumes BASH_VERSION means user's shell is bash
if [ -n "$BASH_VERSION" ]; then
    echo "Adding to ~/.bashrc"
    echo 'alias caro="caro"' >> ~/.bashrc
fi

# GOOD: Check user's actual login shell
user_shell=$(basename "$SHELL")
case "$user_shell" in
    bash)
        config="$HOME/.bashrc"
        ;;
    zsh)
        config="$HOME/.zshrc"
        ;;
    fish)
        config="$HOME/.config/fish/config.fish"
        ;;
    *)
        echo "Unknown shell: $user_shell" >&2
        exit 1
        ;;
esac
```

## Error Handling

### Check Command Existence
```bash
# BAD: Assumes command exists
cargo install caro

# GOOD: Check first
if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found. Please install Rust first." >&2
    echo "Visit: https://rustup.rs" >&2
    exit 1
fi
cargo install caro
```

### Use set -e Carefully
```bash
#!/bin/bash
set -e  # Exit on error

# BAD: No cleanup on failure
download_file
extract_file
install_binary

# GOOD: Trap for cleanup
cleanup() {
    rm -f "$tmp_file"
}
trap cleanup EXIT

download_file
extract_file
install_binary
```

## Quoting and Variables

### Always Quote Variables
```bash
# BAD: Unquoted variables break on spaces
cd $HOME/my directory
rm $file

# GOOD: Quote all variables
cd "$HOME/my directory"
rm "$file"
```

### Use Braces for Clarity
```bash
# BAD: Ambiguous variable boundary
echo $filename_backup

# GOOD: Clear variable boundary
echo "${filename}_backup"
```

## Portability

### POSIX Compatibility
```bash
# BAD: Bash-specific syntax
if [[ $var =~ regex ]]; then
    array+=("element")
fi

# GOOD: POSIX-compatible
if echo "$var" | grep -qE 'regex'; then
    # Use separate variable for array simulation
    elements="$elements element"
fi
```

### Avoid Bashisms When Possible
```bash
# BAD: Bash-only features
source ./script.sh
echo $((x++))
local var="value"  # local is not POSIX but widely supported

# GOOD: POSIX alternatives
. ./script.sh
x=$((x + 1))
var="value"  # Or keep 'local' if targeting bash/zsh only
```

## Security

### Validate User Input
```bash
# BAD: Direct use of user input
rm -rf "$1"

# GOOD: Validate input
if [ -z "$1" ]; then
    echo "Error: Path required" >&2
    exit 1
fi
if [ ! -e "$1" ]; then
    echo "Error: Path does not exist: $1" >&2
    exit 1
fi
# Additional safety: don't allow root or home deletion
case "$1" in
    /|/home|/home/*)
        echo "Error: Refusing to delete protected path" >&2
        exit 1
        ;;
esac
rm -rf "$1"
```

### Secure Temporary Files
```bash
# BAD: Predictable temp file
tmp=/tmp/caro_install

# GOOD: Secure temp file
tmp=$(mktemp) || exit 1
```

## Documentation

### Script Header
```bash
#!/bin/bash
#
# caro-setup.sh - Install and configure caro CLI tool
#
# Usage:
#   ./setup.sh [options]
#
# Options:
#   -h, --help     Show this help message
#   -v, --version  Show version
#
# Requirements:
#   - Rust toolchain (rustup.rs)
#   - curl or wget
#
```

### Explain Non-Obvious Logic
```bash
# We use $SHELL (login shell) instead of $BASH_VERSION because
# the script might be running via 'curl | bash' which sets
# BASH_VERSION even when user's actual shell is zsh or fish.
user_shell=$(basename "$SHELL")
```

## Testing Considerations

### Exit Codes
```bash
# Define meaningful exit codes
EXIT_SUCCESS=0
EXIT_MISSING_DEPENDENCY=1
EXIT_PERMISSION_DENIED=2
EXIT_NETWORK_ERROR=3

# Use consistently
if ! command -v cargo >/dev/null 2>&1; then
    exit $EXIT_MISSING_DEPENDENCY
fi
```

### Idempotency
```bash
# BAD: Appends alias every time script runs
echo 'alias caro="caro"' >> ~/.bashrc

# GOOD: Check if already configured
if ! grep -q 'alias caro=' ~/.bashrc 2>/dev/null; then
    echo 'alias caro="caro"' >> ~/.bashrc
fi
```

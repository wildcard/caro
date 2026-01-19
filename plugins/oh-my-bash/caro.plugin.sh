# caro - Oh My Bash Plugin
# Natural Language → Shell Commands
# https://github.com/wildcard/caro

# Add caro to PATH if not already present
CARO_BIN="${CARO_INSTALL_DIR:-$HOME/.local/bin}"
if [[ ":$PATH:" != *":$CARO_BIN:"* ]]; then
    export PATH="$CARO_BIN:$PATH"
fi

# Also check cargo bin
if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]] && [[ -d "$HOME/.cargo/bin" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Verify caro is available
if ! command -v caro &>/dev/null; then
    echo "[caro] Warning: caro not found in PATH. Install with:"
    echo "  bash <(curl -sSfL https://setup.caro.sh)"
    return
fi

# Load completions
if command -v caro &>/dev/null; then
    eval "$(caro --completion bash 2>/dev/null)" || true
fi

# Aliases
alias c='caro'
alias cx='caro -x'      # Execute directly
alias ce='caro -e'      # Explain command
alias cs='caro -s'      # Safe mode (strict)

# Quick execute with confirmation
crun() {
    local cmd
    cmd=$(caro "$@") || return $?
    echo "Execute: $cmd"
    read -p "Run? [y/N] " reply
    if [[ "$reply" =~ ^[Yy]$ ]]; then
        eval "$cmd"
    fi
}

# History integration - convert last command to natural language explanation
cexplain() {
    local last_cmd="${1:-$(fc -ln -1 2>/dev/null || history 1 | sed 's/^[ ]*[0-9]*[ ]*//')}"
    caro -e "$last_cmd"
}

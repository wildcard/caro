# caro - Oh My Zsh Plugin
# Natural Language → Shell Commands
# https://github.com/wildcard/caro

# Add caro to PATH if not already present
typeset -U path  # Ensure unique entries
path=(
    "${CARO_INSTALL_DIR:-$HOME/.local/bin}"
    "$HOME/.cargo/bin"
    $path
)
export PATH

# Verify caro is available
if (( ! $+commands[caro] )); then
    echo "[caro] Warning: caro not found in PATH. Install with:"
    echo "  bash <(curl -sSfL https://setup.caro.sh)"
    return
fi

# Load completions
if (( $+commands[caro] )); then
    eval "$(caro --completion zsh 2>/dev/null)" || true
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
    echo -n "Run? [y/N] "
    read -r reply
    if [[ "$reply" =~ ^[Yy]$ ]]; then
        eval "$cmd"
    fi
}

# History integration - convert last command to natural language explanation
cexplain() {
    local last_cmd="${1:-$(fc -ln -1)}"
    caro -e "$last_cmd"
}

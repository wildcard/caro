//! Shell integration scripts that wire a `?` keybinding to `caro ai` (Atuin-AI-style).
//!
//! Each supported shell gets a snippet that:
//! 1. Leaves `?` alone when the prompt already has characters (globs etc.).
//! 2. On an empty prompt, invokes `caro ai` and replaces the buffer with the
//!    command the user accepts.
//!
//! The caller may opt out of the `?` binding with `disable_ai = true`, in which
//! case only the plain `caro` wrapper is emitted.

/// Shells we emit integration snippets for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedShell {
    Bash,
    Zsh,
    Fish,
}

impl SupportedShell {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }
}

/// Emit the full shell-init script for `shell`.
///
/// When `disable_ai` is true or [`crate::models::AiConfig::enabled`] is false,
/// the `?` keybinding block is omitted.
pub fn render(shell: SupportedShell, ai_enabled: bool, disable_ai: bool) -> String {
    let base = match shell {
        SupportedShell::Bash => BASH_BASE,
        SupportedShell::Zsh => ZSH_BASE,
        SupportedShell::Fish => FISH_BASE,
    };

    let bind = if ai_enabled && !disable_ai {
        match shell {
            SupportedShell::Bash => BASH_AI_BIND,
            SupportedShell::Zsh => ZSH_AI_BIND,
            SupportedShell::Fish => FISH_AI_BIND,
        }
    } else {
        ""
    };

    format!("{}{}", base, bind)
}

const BASH_BASE: &str = r#"# caro shell integration (bash) — add to ~/.bashrc: eval "$(caro shell-init bash)"

_caro_wrapper() {
    local output exit_code tmpfile
    tmpfile=$(mktemp)
    CARO_WRAPPER=1 command caro "$@" > "$tmpfile"
    exit_code=$?
    output=$(cat "$tmpfile")
    rm -f "$tmpfile"

    if [[ $exit_code -eq 201 ]]; then
        read -e -i "$output" -p "" edited_cmd
        [[ -n "$edited_cmd" ]] && eval "$edited_cmd"
    else
        [[ -n "$output" ]] && echo "$output"
    fi
    return $exit_code
}
alias caro=_caro_wrapper
"#;

const BASH_AI_BIND: &str = r#"
_caro_ai_widget() {
    if [[ -n "${READLINE_LINE}" ]]; then
        READLINE_LINE="${READLINE_LINE:0:$READLINE_POINT}?${READLINE_LINE:$READLINE_POINT}"
        READLINE_POINT=$((READLINE_POINT + 1))
        return
    fi
    local out
    out=$(command caro ai --continue-session < /dev/tty)
    [[ -z "$out" ]] && return
    READLINE_LINE="$out"
    READLINE_POINT=${#out}
}
bind -x '"?": _caro_ai_widget' 2>/dev/null || true
"#;

const ZSH_BASE: &str = r#"# caro shell integration (zsh) — add to ~/.zshrc: eval "$(caro shell-init zsh)"

caro() {
    local output exit_code tmpfile
    tmpfile=$(mktemp)
    CARO_WRAPPER=1 command caro "$@" > "$tmpfile"
    exit_code=$?
    output=$(cat "$tmpfile")
    rm -f "$tmpfile"

    if [[ $exit_code -eq 201 ]]; then
        print -z "$output"
    else
        [[ -n "$output" ]] && echo "$output"
    fi
    return $exit_code
}
"#;

const ZSH_AI_BIND: &str = r#"
_caro_ai_widget() {
    if [[ -n "$BUFFER" ]]; then
        zle self-insert
        return
    fi
    local out
    out=$(command caro ai --continue-session </dev/tty)
    [[ -z "$out" ]] && { zle redisplay; return; }
    BUFFER="$out"
    CURSOR=${#BUFFER}
    zle redisplay
}
zle -N _caro_ai_widget
bindkey '?' _caro_ai_widget
"#;

const FISH_BASE: &str = r#"# caro shell integration (fish) — add to ~/.config/fish/config.fish:
#   caro shell-init fish | source

function caro
    set -l tmpfile (mktemp)
    set -x CARO_WRAPPER 1
    command caro $argv > $tmpfile
    set -l exit_code $status
    set -l output (cat $tmpfile)
    rm -f $tmpfile
    set -e CARO_WRAPPER

    if test $exit_code -eq 201
        commandline -r -- "$output"
    else
        test -n "$output"; and echo "$output"
    end
    return $exit_code
end
"#;

const FISH_AI_BIND: &str = r#"
function __caro_ai_widget
    set -l buf (commandline)
    if test -n "$buf"
        commandline -i '?'
        return
    end
    set -l out (command caro ai --continue-session)
    test -z "$out"; and return
    commandline -r -- $out
end
bind \? __caro_ai_widget
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_snapshot_with_ai_contains_bind() {
        let s = render(SupportedShell::Bash, true, false);
        assert!(s.contains("bind -x '\"?\":"));
        assert!(s.contains("_caro_ai_widget"));
        assert!(s.contains("caro ai --continue-session"));
    }

    #[test]
    fn bash_disable_ai_omits_bind() {
        let s = render(SupportedShell::Bash, true, true);
        assert!(!s.contains("_caro_ai_widget"));
        assert!(s.contains("_caro_wrapper"));
    }

    #[test]
    fn ai_disabled_in_config_omits_bind() {
        let s = render(SupportedShell::Bash, false, false);
        assert!(!s.contains("_caro_ai_widget"));
    }

    #[test]
    fn zsh_snapshot_uses_zle_widget() {
        let s = render(SupportedShell::Zsh, true, false);
        assert!(s.contains("zle -N _caro_ai_widget"));
        assert!(s.contains("bindkey '?' _caro_ai_widget"));
        // Fall-through guard for non-empty prompts.
        assert!(s.contains("zle self-insert"));
    }

    #[test]
    fn fish_snapshot_uses_bind_question() {
        let s = render(SupportedShell::Fish, true, false);
        assert!(s.contains("function __caro_ai_widget"));
        assert!(s.contains("bind \\? __caro_ai_widget"));
        assert!(s.contains("commandline -i '?'"));
    }

    #[test]
    fn parse_returns_supported_shells() {
        assert_eq!(SupportedShell::parse("bash"), Some(SupportedShell::Bash));
        assert_eq!(SupportedShell::parse("ZSH"), Some(SupportedShell::Zsh));
        assert_eq!(SupportedShell::parse("Fish"), Some(SupportedShell::Fish));
        assert_eq!(SupportedShell::parse("ksh"), None);
    }
}

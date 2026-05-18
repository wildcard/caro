//! In-process editable command prompt.
//!
//! Inspired by simonw/llm-cmd: pre-fill a readline buffer with the LLM-
//! generated command so the user can tweak it before execution. Used by the
//! `--edit` / `-e` flag when no shell-integration wrapper (`CARO_WRAPPER`)
//! is active. When the wrapper *is* active, `main.rs` instead emits the
//! command on stdout and exits 201, letting the wrapper push it into the
//! user's real shell buffer (better UX than this fallback).

use rustyline::{error::ReadlineError, DefaultEditor};
use std::io;

/// Outcome of the edit prompt loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOutcome {
    /// User accepted (possibly edited) command for execution.
    Execute(String),
    /// User cancelled — Ctrl+C, Ctrl+D, or submitted an empty buffer.
    Cancelled,
}

/// Open an in-process editable prompt pre-filled with `command`.
///
/// `Enter` submits the (possibly edited) command. `Ctrl+C` / `Ctrl+D` /
/// submitting an empty buffer cancels.
///
/// Multi-line generated commands are collapsed into a single editable line
/// (rustyline's default editor is single-line). Users who need multi-line
/// editing should install the shell wrapper (`eval "$(caro init zsh)"`),
/// which delegates to the host shell's full editor.
pub fn prompt_for_edit(command: &str) -> io::Result<EditOutcome> {
    let initial = collapse_newlines(command.trim());

    let mut rl = DefaultEditor::new()
        .map_err(|e| io::Error::other(format!("rustyline init failed: {e}")))?;

    match rl.readline_with_initial("❯ ", (&initial, "")) {
        Ok(line) => {
            let edited = line.trim().to_string();
            if edited.is_empty() {
                Ok(EditOutcome::Cancelled)
            } else {
                Ok(EditOutcome::Execute(edited))
            }
        }
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok(EditOutcome::Cancelled),
        Err(e) => Err(io::Error::other(format!("readline error: {e}"))),
    }
}

fn collapse_newlines(s: &str) -> String {
    // Flatten any whitespace run (including newlines + leading indentation)
    // into a single space so the command fits on one editable line. Heredoc
    // bodies and for-loop indentation get collapsed — acceptable for v1;
    // users who need multi-line fidelity should install the shell wrapper.
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_equality() {
        assert_eq!(
            EditOutcome::Execute("ls".into()),
            EditOutcome::Execute("ls".into())
        );
        assert_ne!(EditOutcome::Execute("ls".into()), EditOutcome::Cancelled);
    }

    #[test]
    fn collapse_newlines_joins_multiline_command() {
        let input = "for f in *.py; do\n  echo $f\ndone";
        assert_eq!(collapse_newlines(input), "for f in *.py; do echo $f done");
    }

    #[test]
    fn collapse_newlines_preserves_single_line() {
        let input = "ls -la /tmp";
        assert_eq!(collapse_newlines(input), "ls -la /tmp");
    }

    #[test]
    fn collapse_newlines_dedups_whitespace_runs() {
        let input = "a\n\n\tb";
        assert_eq!(collapse_newlines(input), "a b");
    }

    // The interactive rustyline loop itself is not unit-testable without a
    // PTY harness; the integration path is exercised manually per the plan's
    // verification list (smoke tests #1–#6).
}

//! Minimal system prompt variant for the embedded backend.
//!
//! Modeled on Simon Willison's llm-cmd prompt design:
//! one short instruction + one input/output example pair + a shell-specific line.
//!
//! Hypothesis: small models (≤3B parameters) may follow a terse prompt more
//! reliably than a long one because they have a harder time attending to all
//! instructions in a long context window.
//!
//! # Status: evaluated, NOT recommended as default (kept as opt-in experiment)
//!
//! Evaluated 2026-05-16 against `data/evals/default.yaml` (11 cases) using
//! Qwen2.5-Coder-1.5B-Instruct (Q4_K_M, locally cached). Strict exact-string
//! matcher.
//!
//! # Eval Results
//!
//! | Variant | Pass rate     | Wall-clock | Notes |
//! |---------|---------------|------------|-------|
//! | default | 5/11 = 45.5%  | 60.0s      | Production prompt |
//! | minimal | 2/11 = 18.2%  | 33.9s      | This variant |
//!
//! Delta: **-27.3 pp** accuracy, **+43% faster**.
//!
//! Failure modes (minimal):
//! 1. Semantic — model picks `git log --since='yesterday'` instead of `find`
//!    for "files modified today/yesterday" (loses non-git-tracked files).
//! 2. Refusal — model returns `echo 'Unable to generate command'`.
//! 3. Strict-matcher friction — semantically-correct outputs like
//!    `find . -name '*.py' -mtime -7` fail because expected has `-type f`
//!    and double-quoted glob. These would pass a fuzzy matcher.
//!
//! # Decision
//!
//! **Discarded as default.** The accuracy regression (27 pp) far exceeds
//! the 5 pp threshold set in the plan rubric, and the latency win is
//! consumed by retry/rework cost when commands are wrong.
//!
//! **Kept as opt-in** via `--prompt-style minimal` for two reasons:
//!   1. Documents the experiment so the same hypothesis isn't re-run blindly.
//!   2. Useful as a baseline when iterating on prompt engineering — future
//!      experiments can A/B against this minimal variant.
//!
//! Plausible follow-ups (out of scope here):
//!   - Re-run against a 3B+ model (hypothesis was about ≤3B; 1.5B is below).
//!   - Re-run with a fuzzy/semantic eval matcher to disentangle (1) and (3).
//!   - Test prompt variants between full and minimal (e.g., drop only the
//!     Docker/K8s tables, keep the BSD-compat + mtime rules).
//!
//! Plan: `~/.claude/plans/any-novel-ideas-we-floofy-rainbow.md` — Idea #3.

use crate::models::CommandRequest;

/// Build the minimal system prompt for shell command generation.
///
/// This is the llm-cmd-style variant: one line of instruction, one
/// example, one platform note. No Docker/K8s/mtime tables, no
/// numbered rule lists.
pub fn build_minimal_prompt(request: &CommandRequest) -> String {
    let shell = &request.shell;

    // Base minimal prompt — mirrors llm-cmd's philosophy: "no yapping"
    let base = format!(
        r#"Return only the shell command to run. No explanation, no markdown, no fenced code blocks. Respond with ONLY valid JSON: {{"cmd": "your_command_here"}}

Shell: {shell}. Use BSD-compatible flags (macOS). Use "." as the starting path, not "/".

Example:
User: undo last git commit
Assistant: {{"cmd": "git reset --soft HEAD~1"}}

Request: {input}"#,
        shell = shell,
        input = request.input,
    );

    // Append any extra context the caller provided
    if let Some(context) = &request.context {
        format!("{}\n\n{}", base, context)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShellType;

    #[test]
    fn test_minimal_prompt_contains_input() {
        let request = CommandRequest::new("list files modified today", ShellType::Bash);
        let prompt = build_minimal_prompt(&request);
        assert!(prompt.contains("list files modified today"));
    }

    #[test]
    fn test_minimal_prompt_contains_shell() {
        let request = CommandRequest::new("list files", ShellType::Zsh);
        let prompt = build_minimal_prompt(&request);
        assert!(prompt.contains("zsh"));
    }

    #[test]
    fn test_minimal_prompt_no_yapping_keywords() {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let prompt = build_minimal_prompt(&request);
        // Should not contain verbose rule lists
        assert!(!prompt.contains("CRITICAL RULES"));
        assert!(!prompt.contains("KUBERNETES"));
        assert!(!prompt.contains("DOCKER COMMANDS"));
    }

    #[test]
    fn test_minimal_prompt_includes_json_format() {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let prompt = build_minimal_prompt(&request);
        assert!(prompt.contains(r#"{"cmd": "your_command_here"}"#));
    }

    #[test]
    fn test_minimal_prompt_with_context() {
        let request =
            CommandRequest::new("list files", ShellType::Bash).with_context("cwd: /tmp");
        let prompt = build_minimal_prompt(&request);
        assert!(prompt.contains("cwd: /tmp"));
    }

    #[test]
    fn test_minimal_prompt_bsd_note() {
        let request = CommandRequest::new("list files", ShellType::Bash);
        let prompt = build_minimal_prompt(&request);
        assert!(prompt.contains("BSD"));
    }
}

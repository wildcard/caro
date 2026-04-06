//! Scope-aware command evaluation
//!
//! Inspired by Claude Code's auto mode which blocks actions that "escalate beyond
//! the task scope." This module classifies the intent of a natural language prompt
//! and the scope of a generated shell command, then detects mismatches.

use serde::{Deserialize, Serialize};

/// Classification of a command's operational scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommandScope {
    /// Read-only operations (ls, cat, grep, find, etc.)
    ReadOnly,
    /// Write operations that create or modify (cp, mv, mkdir, touch, etc.)
    Write,
    /// Destructive operations that remove or overwrite (rm, truncate, format, etc.)
    Destructive,
}

impl std::fmt::Display for CommandScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read-only"),
            Self::Write => write!(f, "write"),
            Self::Destructive => write!(f, "destructive"),
        }
    }
}

/// Classify the scope of a shell command using keyword heuristics.
///
/// This is intentionally conservative — if we can't determine scope, we assume Write
/// rather than ReadOnly, to avoid false negatives.
pub fn classify_command_scope(command: &str) -> CommandScope {
    let lower = command.to_lowercase();

    // Extract the base command (first token, ignoring env vars and sudo)
    let base_cmd = lower
        .split(|c: char| c.is_whitespace() || c == '|' || c == ';' || c == '&')
        .filter(|t| !t.is_empty())
        .find(|t| {
            !t.starts_with("sudo")
                && !t.contains('=')
                && !t.starts_with("env")
                && !t.starts_with("time")
        })
        .unwrap_or("");

    // Destructive commands
    const DESTRUCTIVE_CMDS: &[&str] = &[
        "rm", "rmdir", "shred", "truncate", "dd", "mkfs", "fdisk", "wipefs",
    ];
    if DESTRUCTIVE_CMDS.iter().any(|c| base_cmd == *c || base_cmd.ends_with(&format!("/{}", c))) {
        return CommandScope::Destructive;
    }

    // Check for destructive flags/patterns anywhere in command
    if lower.contains("--delete") || lower.contains("--remove") || lower.contains("> /dev/null") {
        return CommandScope::Destructive;
    }

    // Read-only commands (single-word)
    const READONLY_CMDS: &[&str] = &[
        "ls", "cat", "head", "tail", "less", "more", "grep", "rg", "find", "fd", "which",
        "whereis", "type", "file", "stat", "wc", "diff", "cmp", "md5sum", "sha256sum", "du",
        "df", "free", "top", "htop", "ps", "uptime", "uname", "whoami", "id", "hostname", "date",
        "cal", "echo", "printf", "pwd", "env", "printenv", "tree", "bat", "exa", "eza", "lsof",
        "dig", "nslookup", "ping", "traceroute", "curl", "wget",
    ];
    // Multi-word read-only commands (checked against the full command)
    const READONLY_PREFIXES: &[&str] = &[
        "git log", "git status", "git diff", "git show", "git branch", "git remote",
        "git tag", "git stash list",
    ];
    if READONLY_PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return CommandScope::ReadOnly;
    }
    if READONLY_CMDS.iter().any(|c| base_cmd == *c || base_cmd.ends_with(&format!("/{}", c))) {
        // Check for output redirection which makes it a write
        if lower.contains('>') && !lower.contains("> /dev/null") {
            return CommandScope::Write;
        }
        return CommandScope::ReadOnly;
    }

    // Default to Write for unknown commands
    CommandScope::Write
}

/// Classify the intent of a natural language prompt using keyword heuristics.
///
/// This is the "expected scope" — what the user likely wants to happen.
pub fn classify_prompt_intent(prompt: &str) -> CommandScope {
    let lower = prompt.to_lowercase();

    // Destructive intent keywords
    const DESTRUCTIVE_KEYWORDS: &[&str] = &[
        "delete",
        "remove",
        "destroy",
        "wipe",
        "erase",
        "purge",
        "clean up",
        "cleanup",
        "uninstall",
        "drop",
        "truncate",
        "format disk",
        "format the",
    ];
    if DESTRUCTIVE_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return CommandScope::Destructive;
    }

    // Read-only intent keywords
    const READONLY_KEYWORDS: &[&str] = &[
        "list",
        "show",
        "display",
        "find",
        "search",
        "look for",
        "check",
        "view",
        "read",
        "print",
        "count",
        "how many",
        "what is",
        "what are",
        "where is",
        "which",
        "status",
        "info",
        "size of",
        "disk usage",
        "memory usage",
    ];
    if READONLY_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return CommandScope::ReadOnly;
    }

    // Write intent keywords
    const WRITE_KEYWORDS: &[&str] = &[
        "create",
        "make",
        "add",
        "write",
        "copy",
        "move",
        "rename",
        "install",
        "build",
        "compile",
        "run",
        "start",
        "stop",
        "restart",
        "update",
        "change",
        "modify",
        "set",
        "configure",
    ];
    if WRITE_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return CommandScope::Write;
    }

    // Default: assume read-only for ambiguous prompts (conservative for user safety)
    CommandScope::ReadOnly
}

/// Check if a command scope exceeds the expected prompt intent.
///
/// Returns a human-readable warning if scope escalation is detected, or None if the
/// command scope is within the expected intent.
pub fn check_scope_escalation(prompt: &str, command: &str) -> Option<String> {
    let intent = classify_prompt_intent(prompt);
    let scope = classify_command_scope(command);

    if scope > intent {
        Some(format!(
            "Scope escalation: prompt intent is {} but command is {} \
             (prompt: \"{}\", command: \"{}\")",
            intent,
            scope,
            truncate_str(prompt, 50),
            truncate_str(command, 50),
        ))
    } else {
        None
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_command_scope_readonly() {
        assert_eq!(classify_command_scope("ls -la"), CommandScope::ReadOnly);
        assert_eq!(classify_command_scope("cat README.md"), CommandScope::ReadOnly);
        assert_eq!(classify_command_scope("grep pattern file.txt"), CommandScope::ReadOnly);
        assert_eq!(classify_command_scope("find . -name '*.rs'"), CommandScope::ReadOnly);
        assert_eq!(classify_command_scope("git status"), CommandScope::ReadOnly);
        assert_eq!(classify_command_scope("pwd"), CommandScope::ReadOnly);
    }

    #[test]
    fn test_classify_command_scope_write() {
        assert_eq!(classify_command_scope("cp file1.txt file2.txt"), CommandScope::Write);
        assert_eq!(classify_command_scope("mv old.txt new.txt"), CommandScope::Write);
        assert_eq!(classify_command_scope("mkdir new_dir"), CommandScope::Write);
        assert_eq!(classify_command_scope("touch new_file"), CommandScope::Write);
        // Read command with redirection becomes write
        assert_eq!(classify_command_scope("echo hello > file.txt"), CommandScope::Write);
    }

    #[test]
    fn test_classify_command_scope_destructive() {
        assert_eq!(classify_command_scope("rm -rf ./build"), CommandScope::Destructive);
        assert_eq!(classify_command_scope("rm file.txt"), CommandScope::Destructive);
        assert_eq!(classify_command_scope("shred secret.txt"), CommandScope::Destructive);
        assert_eq!(classify_command_scope("dd if=/dev/zero of=/dev/sda"), CommandScope::Destructive);
    }

    #[test]
    fn test_classify_prompt_intent_readonly() {
        assert_eq!(classify_prompt_intent("list files in current directory"), CommandScope::ReadOnly);
        assert_eq!(classify_prompt_intent("show me the git status"), CommandScope::ReadOnly);
        assert_eq!(classify_prompt_intent("find all rust files"), CommandScope::ReadOnly);
        assert_eq!(classify_prompt_intent("how many lines in main.rs"), CommandScope::ReadOnly);
        assert_eq!(classify_prompt_intent("what is the disk usage"), CommandScope::ReadOnly);
    }

    #[test]
    fn test_classify_prompt_intent_write() {
        assert_eq!(classify_prompt_intent("create a new directory called build"), CommandScope::Write);
        assert_eq!(classify_prompt_intent("copy the config file"), CommandScope::Write);
        assert_eq!(classify_prompt_intent("install the dependencies"), CommandScope::Write);
        assert_eq!(classify_prompt_intent("build the project"), CommandScope::Write);
    }

    #[test]
    fn test_classify_prompt_intent_destructive() {
        assert_eq!(classify_prompt_intent("delete the build directory"), CommandScope::Destructive);
        assert_eq!(classify_prompt_intent("remove all temp files"), CommandScope::Destructive);
        assert_eq!(classify_prompt_intent("clean up old logs"), CommandScope::Destructive);
    }

    #[test]
    fn test_scope_escalation_detected() {
        // User asks to list files, but command deletes them
        let warning = check_scope_escalation("list files", "rm -rf .");
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Scope escalation"));

        // User asks to show something, but command writes
        let warning = check_scope_escalation("show me the config", "echo 'new' > config.toml");
        assert!(warning.is_some());
    }

    #[test]
    fn test_scope_escalation_not_detected() {
        // User asks to delete, command deletes — no escalation
        assert!(check_scope_escalation("delete the build dir", "rm -rf ./build").is_none());

        // User asks to list, command lists — no escalation
        assert!(check_scope_escalation("list files", "ls -la").is_none());

        // User asks to create, command creates — no escalation
        assert!(check_scope_escalation("create a directory", "mkdir foo").is_none());
    }
}

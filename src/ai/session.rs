//! Session and turn types for the AI conversational loop.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Who produced a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User natural-language input.
    User,
    /// Assistant-generated command or answer.
    Assistant,
}

/// A single turn in an AI session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    pub role: Role,
    /// Free-form content (user prompt, or assistant rationale).
    pub content: String,
    /// When this is an assistant turn that generated a command, the command text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Backend-reported confidence score (0.0-1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Human-readable risk level tag (LOW/MEDIUM/HIGH/CRITICAL) after safety validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    pub ts: DateTime<Utc>,
}

impl Turn {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            command: None,
            confidence: None,
            risk: None,
            ts: Utc::now(),
        }
    }

    pub fn assistant(command: impl Into<String>, rationale: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: rationale.into(),
            command: Some(command.into()),
            confidence: None,
            risk: None,
            ts: Utc::now(),
        }
    }
}

/// A conversational session with the AI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiSession {
    /// Monotonic id within the session file.
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
    /// Shell name captured at session start, e.g. "bash".
    pub shell: String,
    /// Optional CWD snapshot; only recorded when `ai.opening.send_cwd` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub turns: Vec<Turn>,
}

impl AiSession {
    pub fn new(id: u64, shell: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            last_at: now,
            shell: shell.into(),
            cwd: None,
            turns: Vec::new(),
        }
    }

    /// Append a turn and bump `last_at`.
    pub fn push(&mut self, turn: Turn) {
        self.last_at = turn.ts;
        self.turns.push(turn);
    }

    /// The most recent assistant-generated command, if any.
    pub fn last_command(&self) -> Option<&str> {
        self.turns
            .iter()
            .rev()
            .find(|t| t.role == Role::Assistant)
            .and_then(|t| t.command.as_deref())
    }

    /// True when the session was last touched within `minutes` minutes of `now`.
    pub fn is_recent(&self, minutes: u32, now: DateTime<Utc>) -> bool {
        let elapsed = now.signed_duration_since(self.last_at);
        elapsed.num_seconds() >= 0 && elapsed.num_seconds() <= i64::from(minutes) * 60
    }

    /// Render the prior turns as a short "Previous turns" block for the LLM.
    ///
    /// Bounded to the last `max_turns` entries so the prompt stays small.
    pub fn render_history(&self, max_turns: usize) -> String {
        if self.turns.is_empty() {
            return String::new();
        }
        let start = self.turns.len().saturating_sub(max_turns);
        let mut s = String::from("Previous turns:\n");
        for t in &self.turns[start..] {
            match t.role {
                Role::User => s.push_str(&format!("  User: {}\n", t.content)),
                Role::Assistant => {
                    if let Some(cmd) = &t.command {
                        s.push_str(&format!("  Assistant: `{}`\n", cmd));
                    } else {
                        s.push_str(&format!("  Assistant: {}\n", t.content));
                    }
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn session_is_recent_within_window() {
        let mut s = AiSession::new(1, "bash");
        s.last_at = Utc::now() - Duration::minutes(30);
        assert!(s.is_recent(60, Utc::now()));
    }

    #[test]
    fn session_not_recent_past_window() {
        let mut s = AiSession::new(1, "bash");
        s.last_at = Utc::now() - Duration::minutes(61);
        assert!(!s.is_recent(60, Utc::now()));
    }

    #[test]
    fn last_command_returns_most_recent_assistant_command() {
        let mut s = AiSession::new(1, "bash");
        s.push(Turn::user("list files"));
        s.push(Turn::assistant("ls -la", "lists files"));
        s.push(Turn::user("now sort by size"));
        s.push(Turn::assistant("ls -laS", "sorted by size"));
        assert_eq!(s.last_command(), Some("ls -laS"));
    }

    #[test]
    fn render_history_truncates_to_max_turns() {
        let mut s = AiSession::new(1, "bash");
        for i in 0..10 {
            s.push(Turn::user(format!("q{}", i)));
            s.push(Turn::assistant(format!("c{}", i), ""));
        }
        let rendered = s.render_history(4);
        assert!(rendered.contains("q8"));
        assert!(rendered.contains("c9"));
        assert!(!rendered.contains("q0"));
    }
}

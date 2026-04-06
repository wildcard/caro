//! Session memory for safety decisions
//!
//! Remembers user safety approvals within a session to reduce prompt fatigue.
//! Inspired by Claude Code's auto mode where "approving resets denial counters."
//!
//! This module tracks:
//! - Commands that were approved by the user (pattern-based matching)
//! - Denial counts for fallback escalation
//! - Session-scoped state (cleared when the process exits)

use std::collections::HashSet;
use std::sync::{Arc, OnceLock, RwLock};

/// Global session memory instance (process-scoped)
static SESSION_MEMORY: OnceLock<Arc<SafetySessionMemory>> = OnceLock::new();

/// Get or create the global session memory
pub fn session_memory() -> Arc<SafetySessionMemory> {
    SESSION_MEMORY
        .get_or_init(|| Arc::new(SafetySessionMemory::new()))
        .clone()
}

/// Session-scoped memory for safety decisions.
///
/// Tracks approved command patterns so that repeated similar commands
/// don't require re-confirmation within the same session.
#[derive(Debug)]
pub struct SafetySessionMemory {
    /// Command patterns that the user has approved
    approved_patterns: RwLock<HashSet<String>>,
    /// Count of consecutive denials (for fallback escalation)
    consecutive_denials: RwLock<u32>,
    /// Total denials in this session
    total_denials: RwLock<u32>,
}

/// Thresholds for fallback escalation (matching Claude Code's auto mode)
const MAX_CONSECUTIVE_DENIALS: u32 = 3;
const MAX_TOTAL_DENIALS: u32 = 20;

impl SafetySessionMemory {
    /// Create a new empty session memory
    pub fn new() -> Self {
        Self {
            approved_patterns: RwLock::new(HashSet::new()),
            consecutive_denials: RwLock::new(0),
            total_denials: RwLock::new(0),
        }
    }

    /// Record that the user approved a command.
    ///
    /// Extracts the command's "pattern" (base command + key flags) and stores it.
    /// Future similar commands will be auto-approved within this session.
    pub fn record_approval(&self, command: &str) {
        let pattern = Self::extract_pattern(command);
        if let Ok(mut approved) = self.approved_patterns.write() {
            approved.insert(pattern);
        }
        // Reset consecutive denial counter on approval
        if let Ok(mut denials) = self.consecutive_denials.write() {
            *denials = 0;
        }
    }

    /// Record a denial (user rejected or command was blocked)
    pub fn record_denial(&self) {
        if let Ok(mut consecutive) = self.consecutive_denials.write() {
            *consecutive += 1;
        }
        if let Ok(mut total) = self.total_denials.write() {
            *total += 1;
        }
    }

    /// Check if a command matches a previously approved pattern
    pub fn is_pre_approved(&self, command: &str) -> bool {
        let pattern = Self::extract_pattern(command);
        self.approved_patterns
            .read()
            .map(|approved| approved.contains(&pattern))
            .unwrap_or(false)
    }

    /// Check if we should fall back to manual prompting due to excessive denials.
    ///
    /// Mirrors Claude Code's auto mode: falls back after 3 consecutive or 20 total blocks.
    pub fn should_fallback(&self) -> bool {
        let consecutive = self
            .consecutive_denials
            .read()
            .map(|c| *c)
            .unwrap_or(0);
        let total = self.total_denials.read().map(|t| *t).unwrap_or(0);
        consecutive >= MAX_CONSECUTIVE_DENIALS || total >= MAX_TOTAL_DENIALS
    }

    /// Get the number of approved patterns in this session
    pub fn approved_count(&self) -> usize {
        self.approved_patterns
            .read()
            .map(|a| a.len())
            .unwrap_or(0)
    }

    /// Extract a command pattern for matching.
    ///
    /// Normalizes the command by keeping only the base command and key structural
    /// elements, removing specific arguments like file paths and numbers.
    /// This allows "rm -rf ./build" to match "rm -rf ./dist" in the same session.
    fn extract_pattern(command: &str) -> String {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        if tokens.is_empty() {
            return String::new();
        }

        let mut pattern_parts = Vec::new();

        // Keep the base command
        pattern_parts.push(tokens[0].to_string());

        // Keep flags (tokens starting with -)
        for token in tokens.iter().skip(1) {
            if token.starts_with('-') {
                pattern_parts.push(token.to_string());
            }
        }

        pattern_parts.join(" ")
    }
}

impl Default for SafetySessionMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_and_pre_approval() {
        let memory = SafetySessionMemory::new();

        assert!(!memory.is_pre_approved("rm -rf ./build"));

        memory.record_approval("rm -rf ./build");

        // Same command pattern should be pre-approved
        assert!(memory.is_pre_approved("rm -rf ./dist"));
        // Different base command should not
        assert!(!memory.is_pre_approved("ls -la"));
    }

    #[test]
    fn test_pattern_extraction() {
        // Same command with different paths → same pattern
        assert_eq!(
            SafetySessionMemory::extract_pattern("rm -rf ./build"),
            SafetySessionMemory::extract_pattern("rm -rf ./dist")
        );

        // Different flags → different patterns
        assert_ne!(
            SafetySessionMemory::extract_pattern("rm -rf ./build"),
            SafetySessionMemory::extract_pattern("rm -f ./build")
        );
    }

    #[test]
    fn test_denial_tracking() {
        let memory = SafetySessionMemory::new();

        assert!(!memory.should_fallback());

        // 3 consecutive denials should trigger fallback
        memory.record_denial();
        memory.record_denial();
        assert!(!memory.should_fallback());
        memory.record_denial();
        assert!(memory.should_fallback());
    }

    #[test]
    fn test_approval_resets_consecutive_denials() {
        let memory = SafetySessionMemory::new();

        memory.record_denial();
        memory.record_denial();
        assert!(!memory.should_fallback());

        // Approval resets consecutive counter
        memory.record_approval("ls -la");
        memory.record_denial();
        memory.record_denial();
        assert!(!memory.should_fallback()); // Still only 2 consecutive
    }

    #[test]
    fn test_total_denial_fallback() {
        let memory = SafetySessionMemory::new();

        // Simulate 20 denials with approvals in between (never hitting 3 consecutive)
        for _ in 0..10 {
            memory.record_denial();
            memory.record_denial();
            memory.record_approval("ok");
        }

        assert!(memory.should_fallback()); // 20 total denials
    }

    #[test]
    fn test_approved_count() {
        let memory = SafetySessionMemory::new();

        assert_eq!(memory.approved_count(), 0);
        memory.record_approval("rm -rf ./build");
        assert_eq!(memory.approved_count(), 1);
        memory.record_approval("git push origin main");
        assert_eq!(memory.approved_count(), 2);
        // Same pattern doesn't increase count
        memory.record_approval("rm -rf ./dist");
        assert_eq!(memory.approved_count(), 2);
    }
}

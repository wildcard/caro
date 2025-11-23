//! Feature extraction for ML-based risk prediction
//!
//! This module extracts features from shell commands for use in risk prediction.
//! Features are designed to capture lexical, semantic, and contextual aspects of commands.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Privilege level indicators in commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivilegeLevel {
    /// Normal user operations
    User,
    /// Elevated privileges (sudo, su, etc.)
    Elevated,
    /// Root/Administrator operations
    Root,
}

/// Target scope of command operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetScope {
    /// Single file operation
    SingleFile,
    /// Multiple files in current directory
    LocalFiles,
    /// Recursive directory operations
    Recursive,
    /// System-wide paths (/usr, /bin, /etc)
    System,
    /// Root filesystem
    Root,
    /// Network operations
    Network,
}

/// Extracted features from a command for ML model input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFeatures {
    // Lexical features
    pub tokens: Vec<String>,
    pub token_count: usize,
    pub command_length: usize,
    pub flags: HashMap<String, bool>,
    pub has_pipe: bool,
    pub has_redirect: bool,
    pub has_background: bool,
    pub has_logic_ops: bool, // &&, ||, ;

    // Semantic features
    pub destructive_score: f32,       // 0.0-1.0
    pub privilege_level: PrivilegeLevel,
    pub target_scope: TargetScope,
    pub is_system_command: bool,
    pub is_network_command: bool,
    pub is_disk_command: bool,

    // Pattern-based features
    pub has_recursive_flag: bool,
    pub has_force_flag: bool,
    pub has_wildcard: bool,
    pub has_root_path: bool,
    pub has_system_path: bool,

    // Historical features (placeholder for future ML integration)
    pub similarity_to_dangerous: f32, // 0.0-1.0
}

impl CommandFeatures {
    /// Extract features from a shell command
    pub fn extract(command: &str) -> Self {
        let tokens = tokenize(command);
        let token_count = tokens.len();
        let command_length = command.len();

        // Extract flags
        let flags = extract_flags(&tokens);

        // Detect operators
        let has_pipe = command.contains('|') && !is_in_quotes(command, '|');
        let has_redirect = (command.contains('>') || command.contains('<'))
            && !is_in_quotes(command, '>')
            && !is_in_quotes(command, '<');
        let has_background = command.contains('&') && !is_in_quotes(command, '&');
        let has_logic_ops = (command.contains("&&") || command.contains("||") || command.contains(';'))
            && !is_in_quotes(command, ';');

        // Calculate semantic features
        let destructive_score = calculate_destructive_score(&tokens, command);
        let privilege_level = detect_privilege_level(&tokens);
        let target_scope = detect_target_scope(&tokens, command);
        let is_system_command = is_system_command(&tokens);
        let is_network_command = is_network_command(&tokens);
        let is_disk_command = is_disk_command(&tokens);

        // Pattern-based features
        let has_recursive_flag = flags.contains_key("-r")
            || flags.contains_key("-R")
            || flags.contains_key("--recursive");
        let has_force_flag = flags.contains_key("-f")
            || flags.contains_key("-F")
            || flags.contains_key("--force");
        let has_wildcard = command.contains('*') || command.contains('?');
        let has_root_path = command.contains(" /") || command.starts_with('/');
        let has_system_path = command.contains("/usr")
            || command.contains("/bin")
            || command.contains("/etc")
            || command.contains("/sys");

        // Historical features (stub for future ML)
        let similarity_to_dangerous = 0.0;

        Self {
            tokens,
            token_count,
            command_length,
            flags,
            has_pipe,
            has_redirect,
            has_background,
            has_logic_ops,
            destructive_score,
            privilege_level,
            target_scope,
            is_system_command,
            is_network_command,
            is_disk_command,
            has_recursive_flag,
            has_force_flag,
            has_wildcard,
            has_root_path,
            has_system_path,
            similarity_to_dangerous,
        }
    }

    /// Convert features to a vector for ML model input
    /// Returns a 30-dimensional feature vector
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            // Lexical features (5)
            self.token_count as f32,
            self.command_length as f32,
            if self.has_pipe { 1.0 } else { 0.0 },
            if self.has_redirect { 1.0 } else { 0.0 },
            if self.has_logic_ops { 1.0 } else { 0.0 },

            // Semantic features (8)
            self.destructive_score,
            match self.privilege_level {
                PrivilegeLevel::User => 0.0,
                PrivilegeLevel::Elevated => 0.5,
                PrivilegeLevel::Root => 1.0,
            },
            match self.target_scope {
                TargetScope::SingleFile => 0.0,
                TargetScope::LocalFiles => 0.2,
                TargetScope::Recursive => 0.5,
                TargetScope::System => 0.8,
                TargetScope::Root => 1.0,
                TargetScope::Network => 0.6,
            },
            if self.is_system_command { 1.0 } else { 0.0 },
            if self.is_network_command { 1.0 } else { 0.0 },
            if self.is_disk_command { 1.0 } else { 0.0 },
            if self.has_background { 1.0 } else { 0.0 },
            if self.has_wildcard { 1.0 } else { 0.0 },

            // Pattern features (7)
            if self.has_recursive_flag { 1.0 } else { 0.0 },
            if self.has_force_flag { 1.0 } else { 0.0 },
            if self.has_root_path { 1.0 } else { 0.0 },
            if self.has_system_path { 1.0 } else { 0.0 },
            self.flags.len() as f32,
            if self.flags.contains_key("-rf") ||
               (self.has_recursive_flag && self.has_force_flag) { 1.0 } else { 0.0 },
            if self.tokens.first().map(|s| s.as_str()) == Some("rm") &&
               self.has_recursive_flag &&
               self.has_force_flag { 1.0 } else { 0.0 },

            // Historical features (10 - reserved for future ML)
            self.similarity_to_dangerous,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]
    }
}

/// Tokenize command into words (space-separated)
fn tokenize(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Extract flags from tokens (anything starting with - or --)
fn extract_flags(tokens: &[String]) -> HashMap<String, bool> {
    let mut flags = HashMap::new();

    for token in tokens {
        if token.starts_with("--") {
            flags.insert(token.clone(), true);
        } else if token.starts_with('-') && token.len() > 1 {
            // Handle combined flags like -rf
            if token.len() == 2 {
                flags.insert(token.clone(), true);
            } else {
                // Split combined flags: -rf becomes -r and -f
                for ch in token.chars().skip(1) {
                    flags.insert(format!("-{}", ch), true);
                }
                // Also store the combined form
                flags.insert(token.clone(), true);
            }
        }
    }

    flags
}

/// Check if a character appears in quotes
fn is_in_quotes(command: &str, _ch: char) -> bool {
    // Simple heuristic: check if more quotes before than after
    // This is a simplified version - full parser would be more complex
    let single_quotes = command.matches('\'').count();
    let double_quotes = command.matches('"').count();

    // If odd number of quotes, might be inside quotes
    (single_quotes % 2 == 1) || (double_quotes % 2 == 1)
}

/// Calculate destructive score based on command patterns
fn calculate_destructive_score(tokens: &[String], command: &str) -> f32 {
    let first_token = tokens.first().map(|s| s.as_str()).unwrap_or("");

    let mut score: f32 = 0.0;

    // Destructive commands
    match first_token {
        "rm" => score += 0.7,
        "dd" => score += 0.9,
        "mkfs" => score += 1.0,
        "fdisk" => score += 0.8,
        "format" => score += 1.0,
        "del" | "erase" => score += 0.6,
        "truncate" => score += 0.5,
        "shred" => score += 0.8,
        _ => {}
    }

    // Modifying commands
    if matches!(first_token, "chmod" | "chown" | "chgrp") {
        score += 0.3;
    }

    // Check for destructive keywords in command
    if command.contains("drop") && command.contains("database") {
        score += 0.9;
    }
    if command.contains("truncate") && command.contains("table") {
        score += 0.7;
    }

    // Increase score for force flags
    if command.contains("-f") || command.contains("--force") {
        score += 0.2;
    }

    // Increase score for recursive operations
    if command.contains("-r") || command.contains("-R") || command.contains("--recursive") {
        score += 0.2;
    }

    score.min(1.0)
}

/// Detect privilege level from tokens
fn detect_privilege_level(tokens: &[String]) -> PrivilegeLevel {
    let first = tokens.first().map(|s| s.as_str()).unwrap_or("");

    match first {
        "sudo" | "doas" => PrivilegeLevel::Elevated,
        "su" => PrivilegeLevel::Root,
        _ => PrivilegeLevel::User,
    }
}

/// Detect target scope from command
fn detect_target_scope(tokens: &[String], command: &str) -> TargetScope {
    // Check for root path
    if command.contains(" /") || (command.starts_with('/') && !command.starts_with("/home")) {
        return TargetScope::Root;
    }

    // Check for system paths
    if command.contains("/usr") || command.contains("/bin") ||
       command.contains("/etc") || command.contains("/sys") {
        return TargetScope::System;
    }

    // Check for network operations
    if command.contains("://") || tokens.iter().any(|t| t.contains("http")) {
        return TargetScope::Network;
    }

    // Check for recursive flag
    if command.contains("-r") || command.contains("-R") || command.contains("--recursive") {
        return TargetScope::Recursive;
    }

    // Check for wildcards
    if command.contains('*') || command.contains('?') {
        return TargetScope::LocalFiles;
    }

    TargetScope::SingleFile
}

/// Check if command is a system administration command
fn is_system_command(tokens: &[String]) -> bool {
    let first = tokens.first().map(|s| s.as_str()).unwrap_or("");
    matches!(
        first,
        "systemctl" | "service" | "init" | "reboot" | "shutdown" |
        "halt" | "poweroff" | "mount" | "umount" | "fsck" |
        "mkfs" | "fdisk" | "parted" | "lvm" | "mdadm"
    )
}

/// Check if command is a network command
fn is_network_command(tokens: &[String]) -> bool {
    let first = tokens.first().map(|s| s.as_str()).unwrap_or("");
    matches!(
        first,
        "curl" | "wget" | "nc" | "netcat" | "ssh" | "scp" |
        "ftp" | "telnet" | "ping" | "traceroute" | "nmap" |
        "iptables" | "firewall-cmd" | "ufw"
    )
}

/// Check if command is a disk/filesystem command
fn is_disk_command(tokens: &[String]) -> bool {
    let first = tokens.first().map(|s| s.as_str()).unwrap_or("");
    matches!(
        first,
        "dd" | "mkfs" | "fdisk" | "parted" | "lvm" | "mdadm" |
        "mount" | "umount" | "fsck" | "e2fsck"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_command() {
        let features = CommandFeatures::extract("ls -la");
        assert_eq!(features.token_count, 2);
        assert!(features.flags.contains_key("-l"));
        assert!(features.flags.contains_key("-a"));
        assert_eq!(features.destructive_score, 0.0);
    }

    #[test]
    fn test_extract_destructive_command() {
        let features = CommandFeatures::extract("rm -rf /tmp/test");
        assert_eq!(features.token_count, 3);
        assert!(features.has_recursive_flag);
        assert!(features.has_force_flag);
        assert!(features.destructive_score > 0.5);
    }

    #[test]
    fn test_extract_sudo_command() {
        let features = CommandFeatures::extract("sudo apt-get install package");
        assert_eq!(features.privilege_level, PrivilegeLevel::Elevated);
    }

    #[test]
    fn test_extract_system_path() {
        let features = CommandFeatures::extract("rm /usr/bin/something");
        assert!(features.has_system_path);
        assert_eq!(features.target_scope, TargetScope::System);
    }

    #[test]
    fn test_to_vector() {
        let features = CommandFeatures::extract("ls -la");
        let vector = features.to_vector();
        assert_eq!(vector.len(), 30);
    }

    #[test]
    fn test_combined_flags() {
        let features = CommandFeatures::extract("rm -rf test");
        assert!(features.flags.contains_key("-r"));
        assert!(features.flags.contains_key("-f"));
        assert!(features.flags.contains_key("-rf"));
    }

    #[test]
    fn test_pipe_detection() {
        let features = CommandFeatures::extract("cat file | grep pattern");
        assert!(features.has_pipe);
    }

    #[test]
    fn test_redirect_detection() {
        let features = CommandFeatures::extract("echo hello > file.txt");
        assert!(features.has_redirect);
    }
}

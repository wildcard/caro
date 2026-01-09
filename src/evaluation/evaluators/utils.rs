//! Utility functions for command evaluation
//!
//! Provides common functionality used across evaluators:
//! - Command equivalence checking
//! - Pattern matching
//! - POSIX compliance validation

use regex::Regex;
use std::collections::HashSet;

/// Checks if two shell commands are functionally equivalent
///
/// This function performs basic normalization and comparison to determine
/// if two commands would produce the same result.
///
/// # Examples
///
/// ```
/// use caro::evaluation::evaluators::utils::command_equivalence;
///
/// assert!(command_equivalence("ls -la", "ls -l -a"));
/// assert!(command_equivalence("find .", "find . -type f"));  // Both find files
/// ```
///
/// # Limitations
///
/// - Does not execute commands or perform semantic analysis
/// - May produce false negatives for complex equivalent commands
/// - Start with simple normalization, iterate based on failures
pub fn command_equivalence(cmd1: &str, cmd2: &str) -> bool {
    // Normalize whitespace
    let norm1 = normalize_whitespace(cmd1);
    let norm2 = normalize_whitespace(cmd2);

    // Exact match after normalization
    if norm1 == norm2 {
        return true;
    }

    // Parse into tokens for flag-order-independent comparison
    let tokens1 = tokenize_command(&norm1);
    let tokens2 = tokenize_command(&norm2);

    // Check if base command is the same
    if tokens1.first() != tokens2.first() {
        return false;
    }

    // For simple cases, check if flags are equivalent (order-independent)
    if are_flags_equivalent(&tokens1, &tokens2) {
        return true;
    }

    // TODO: Add more sophisticated equivalence checks as needed:
    // - Implicit defaults (e.g., "find ." vs "find . -type f")
    // - Flag synonyms (e.g., "-a" vs "--all")
    // - Command aliases (e.g., "ll" vs "ls -l")

    false
}

/// Checks if a command matches a regex pattern
///
/// # Arguments
///
/// * `cmd` - The command string to match
/// * `pattern` - The regex pattern to match against
///
/// # Returns
///
/// `true` if the command matches the pattern, `false` otherwise
///
/// # Examples
///
/// ```
/// use caro::evaluation::evaluators::utils::matches_pattern;
///
/// assert!(matches_pattern("find . -name '*.py'", r"find.*\.py"));
/// assert!(matches_pattern("grep -rn 'TODO'", r"grep.*-rn"));
/// ```
pub fn matches_pattern(cmd: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(re) => re.is_match(cmd),
        Err(_) => false, // Invalid regex = no match
    }
}

/// Checks command for POSIX compliance issues
///
/// Returns a list of potential POSIX violations found in the command.
/// Empty vector means no obvious violations detected.
///
/// # Examples
///
/// ```
/// use caro::evaluation::evaluators::utils::check_posix_compliance;
///
/// let violations = check_posix_compliance("find . -mtime -1");
/// assert!(!violations.is_empty());  // -mtime -1 is GNU extension
///
/// let violations = check_posix_compliance("find . -mtime 0");
/// assert!(violations.is_empty());  // -mtime 0 is POSIX compliant
/// ```
pub fn check_posix_compliance(cmd: &str) -> Vec<String> {
    let mut violations = Vec::new();

    // Check for bash-specific features
    if cmd.contains("[[") || cmd.contains("]]") {
        violations.push("Bash-specific [[ test operator (use [ instead)".to_string());
    }

    if cmd.contains("&>>") || cmd.contains("|&") {
        violations.push("Bash-specific redirection operators".to_string());
    }

    // Check for GNU-specific flags
    if cmd.contains("--") {
        violations.push("GNU-style long options (--flag) may not be POSIX".to_string());
    }

    // Check for common GNU extensions
    if let Some(gnu_violation) = check_gnu_extensions(cmd) {
        violations.push(gnu_violation);
    }

    // Check for process substitution
    if cmd.contains("<(") || cmd.contains(">(") {
        violations.push("Process substitution <() is not POSIX".to_string());
    }

    // Check for here-strings
    if cmd.contains("<<<") {
        violations.push("Here-strings (<<<) are not POSIX".to_string());
    }

    violations
}

// Helper functions

fn normalize_whitespace(cmd: &str) -> String {
    cmd.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize_command(cmd: &str) -> Vec<String> {
    // Simple tokenization - split by whitespace
    // TODO: Handle quotes and escaping properly
    cmd.split_whitespace().map(|s| s.to_string()).collect()
}

fn are_flags_equivalent(tokens1: &[String], tokens2: &[String]) -> bool {
    // Extract flags (tokens starting with -)
    let flags1: HashSet<_> = tokens1
        .iter()
        .filter(|t| t.starts_with('-') && !t.starts_with("--"))
        .collect();
    let flags2: HashSet<_> = tokens2
        .iter()
        .filter(|t| t.starts_with('-') && !t.starts_with("--"))
        .collect();

    // Check if flag sets are equivalent
    if flags1 != flags2 {
        // Try expanding combined flags (e.g., -la vs -l -a)
        let expanded1 = expand_combined_flags(&flags1);
        let expanded2 = expand_combined_flags(&flags2);
        if expanded1 != expanded2 {
            return false;
        }
    }

    // Check if arguments (non-flag tokens) are the same
    let args1: Vec<_> = tokens1
        .iter()
        .filter(|t| !t.starts_with('-'))
        .skip(1) // Skip command name
        .collect();
    let args2: Vec<_> = tokens2
        .iter()
        .filter(|t| !t.starts_with('-'))
        .skip(1)
        .collect();

    args1 == args2
}

fn expand_combined_flags(flags: &HashSet<&String>) -> HashSet<char> {
    let mut expanded = HashSet::new();
    for flag in flags {
        if flag.len() > 2 && !flag.starts_with("--") {
            // Combined flags like -la
            for ch in flag.chars().skip(1) {
                expanded.insert(ch);
            }
        } else if flag.len() == 2 {
            // Single flag like -l
            if let Some(ch) = flag.chars().nth(1) {
                expanded.insert(ch);
            }
        }
    }
    expanded
}

fn check_gnu_extensions(cmd: &str) -> Option<String> {
    // Check for find command GNU extensions
    if cmd.contains("find") {
        if let Some(violation) = check_find_gnu_extensions(cmd) {
            return Some(violation);
        }
    }

    // Check for stat command differences
    if cmd.contains("stat -c") {
        return Some("stat -c is GNU (use stat -f on BSD)".to_string());
    }
    if cmd.contains("stat -f") && !cmd.contains("stat -f %") {
        return Some("stat -f on BSD requires format (use stat -c on GNU)".to_string());
    }

    // Check for date command GNU extensions
    if cmd.contains("date") && cmd.contains("-d") {
        return Some("date -d is GNU extension (not POSIX)".to_string());
    }

    None
}

fn check_find_gnu_extensions(cmd: &str) -> Option<String> {
    // Check for -mtime with negative values (GNU extension)
    if Regex::new(r"-mtime\s+-\d+").unwrap().is_match(cmd) {
        return Some("find -mtime -N is GNU extension (use -mtime N for POSIX)".to_string());
    }

    // Check for -regex (GNU extension)
    if cmd.contains("-regex") {
        return Some("find -regex is GNU extension (use -name with wildcards)".to_string());
    }

    // Check for -printf (GNU extension)
    if cmd.contains("-printf") {
        return Some("find -printf is GNU extension (use -print or -exec)".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_equivalence_exact_match() {
        assert!(command_equivalence("ls -la", "ls -la"));
        assert!(command_equivalence("find . -name '*.py'", "find . -name '*.py'"));
    }

    #[test]
    fn test_command_equivalence_whitespace() {
        assert!(command_equivalence("ls  -la", "ls -la"));
        assert!(command_equivalence("ls\t-la", "ls -la"));
    }

    #[test]
    fn test_command_equivalence_flag_order() {
        assert!(command_equivalence("ls -la", "ls -al"));
        assert!(command_equivalence("ls -l -a", "ls -a -l"));
    }

    #[test]
    fn test_command_equivalence_different_commands() {
        assert!(!command_equivalence("ls", "find"));
        assert!(!command_equivalence("ls -la", "ls -l"));
    }

    #[test]
    fn test_matches_pattern_basic() {
        assert!(matches_pattern("find . -name '*.py'", r"find.*\.py"));
        assert!(matches_pattern("grep -rn 'TODO'", r"grep.*-rn"));
        assert!(!matches_pattern("ls -la", r"find.*"));
    }

    #[test]
    fn test_matches_pattern_invalid_regex() {
        assert!(!matches_pattern("ls -la", r"[invalid("));
    }

    #[test]
    fn test_check_posix_compliance_bash_features() {
        let violations = check_posix_compliance("[[ -f file ]]");
        assert!(!violations.is_empty());
        assert!(violations[0].contains("Bash-specific"));
    }

    #[test]
    fn test_check_posix_compliance_gnu_long_options() {
        let violations = check_posix_compliance("ls --all --human-readable");
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_check_posix_compliance_find_mtime() {
        let violations = check_posix_compliance("find . -mtime -1");
        assert!(!violations.is_empty());
        assert!(violations[0].contains("-mtime -N"));

        let violations = check_posix_compliance("find . -mtime 0");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_posix_compliance_process_substitution() {
        let violations = check_posix_compliance("diff <(ls dir1) <(ls dir2)");
        assert!(!violations.is_empty());
        assert!(violations[0].contains("Process substitution"));
    }

    #[test]
    fn test_check_posix_compliance_clean_command() {
        let violations = check_posix_compliance("find . -type f -name '*.txt'");
        assert!(violations.is_empty());
    }
}

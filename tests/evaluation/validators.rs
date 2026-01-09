//! Command validation (safety, POSIX compliance)

use caro::safety::{SafetyConfig, SafetyValidator};
use caro::models::{RiskLevel, ShellType};
use regex::Regex;

/// Normalize a shell command for semantic comparison
///
/// Applies these transformations:
/// 1. Collapse whitespace (multiple spaces/tabs → single space)
/// 2. Sort consolidated flags (e.g., -la → -al)
/// 3. Trim leading/trailing whitespace
pub fn normalize_command(cmd: &str) -> String {
    let mut normalized = cmd.to_string();

    // Step 1: Collapse whitespace
    normalized = collapse_whitespace(&normalized);

    // Step 2: Sort flags (e.g., "ls -l -a" → "ls -al")
    normalized = sort_flags(&normalized);

    // Step 3: Trim
    normalized.trim().to_string()
}

fn collapse_whitespace(cmd: &str) -> String {
    cmd.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn sort_flags(cmd: &str) -> String {
    // Parse command into tokens
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    let mut result = Vec::new();
    let mut pending_flags = Vec::new();  // Accumulate consecutive single-character flags

    for token in tokens {
        if token.starts_with('-') && !token.starts_with("--") && token.len() == 2 {
            // Single-character flag (e.g., "-l")
            pending_flags.push(token.chars().nth(1).unwrap());
        } else {
            // Flush any accumulated single-character flags
            if !pending_flags.is_empty() {
                pending_flags.sort_unstable();
                result.push(format!("-{}", pending_flags.iter().collect::<String>()));
                pending_flags.clear();
            }

            // Handle current token
            if token.starts_with('-') && !token.starts_with("--") && token.len() > 2 {
                // Multi-character consolidated flag (e.g., "-la")
                let mut chars: Vec<char> = token.chars().skip(1).collect();
                chars.sort_unstable();
                result.push(format!("-{}", chars.iter().collect::<String>()));
            } else {
                // Not a flag (command name, argument, etc.)
                result.push(token.to_string());
            }
        }
    }

    // Flush any remaining accumulated flags at end of command
    if !pending_flags.is_empty() {
        pending_flags.sort_unstable();
        result.push(format!("-{}", pending_flags.iter().collect::<String>()));
    }

    result.join(" ")
}

/// Compare two commands for semantic equivalence
///
/// Returns true if commands are semantically equivalent after normalization.
///
/// # Examples
/// ```
/// use evaluation::validators::commands_match;
/// assert!(commands_match("ls -la", "ls -l -a"));
/// assert!(commands_match("grep  'error'  logs", "grep 'error' logs"));
/// assert!(!commands_match("ls -la", "ls -lh"));
/// ```
pub fn commands_match(expected: &str, actual: &str) -> bool {
    let normalized_expected = normalize_command(expected);
    let normalized_actual = normalize_command(actual);

    normalized_expected == normalized_actual
}

/// Validate command safety using existing caro safety module
///
/// Returns true if command is safe, false if dangerous.
///
/// # Examples
/// ```
/// assert_eq!(validate_safety("ls -la"), true);
/// assert_eq!(validate_safety("rm -rf /"), false);
/// assert_eq!(validate_safety("dd if=/dev/zero of=/dev/sda"), false);
/// ```
pub async fn validate_safety(command: &str) -> bool {
    // Create validator with moderate safety config
    let validator = SafetyValidator::new(SafetyConfig::moderate()).unwrap();

    // Validate command (default to Bash shell type)
    match validator.validate_command(command, ShellType::Bash).await {
        Ok(result) => result.allowed,  // Safe if allowed = true
        Err(_) => false,              // Treat validation errors as unsafe
    }
}

/// Check if command uses only POSIX-compliant syntax
///
/// Returns true if command is POSIX-compliant, false if it uses
/// bash/zsh-specific features.
///
/// # Examples
/// ```
/// assert_eq!(is_posix_compliant("[ -f file.txt ]"), true);
/// assert_eq!(is_posix_compliant("[[ -f file.txt ]]"), false);  // Bash [[
/// assert_eq!(is_posix_compliant("arr=(1 2 3)"), false);        // Bash arrays
/// assert_eq!(is_posix_compliant("ls **/"), false);             // Zsh globstar
/// ```
pub fn is_posix_compliant(command: &str) -> bool {
    // Bash-specific patterns
    let bash_patterns = [
        r"\[\[",                    // [[ test construct
        r"\bfunction\b",            // function keyword
        r"\{[0-9]+\.\.[0-9]+\}",   // Brace expansion {1..10}
        r"<\(",                     // Process substitution <()
        r"\$\(\(",                  // Arithmetic expansion $(())
    ];

    // Zsh-specific patterns
    let zsh_patterns = [
        r"\*\*/",                   // Recursive globstar **/
        r"=\(",                     // Process substitution =()
    ];

    // Check bash patterns
    for pattern in &bash_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(command) {
                return false;  // Found shell-specific syntax
            }
        }
    }

    // Check zsh patterns
    for pattern in &zsh_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(command) {
                return false;  // Found shell-specific syntax
            }
        }
    }

    true  // POSIX compliant
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            normalize_command("ls  -la   /tmp"),
            "ls -al /tmp"
        );
        assert_eq!(
            normalize_command("  grep\t'error'\t\tlogs  "),
            "grep 'error' logs"
        );
    }

    #[test]
    fn test_normalize_flags() {
        assert_eq!(normalize_command("ls -la"), "ls -al");
        assert_eq!(normalize_command("ls -l -a"), "ls -al");
        assert_eq!(normalize_command("tar -czf"), "tar -cfz");
    }

    #[test]
    fn test_long_flags_unchanged() {
        // Long flags should not be sorted
        assert_eq!(
            normalize_command("ls --all --long"),
            "ls --all --long"
        );
    }

    #[test]
    fn test_commands_match_equivalent() {
        assert!(commands_match("ls -la", "ls -l -a"));
        assert!(commands_match("ls -la", "ls  -al"));
        assert!(commands_match("grep 'error' logs", "grep  'error'  logs"));
    }

    #[test]
    fn test_commands_match_different() {
        assert!(!commands_match("ls -la", "ls -lh"));
        assert!(!commands_match("grep 'error' logs", "grep 'warning' logs"));
        assert!(!commands_match("ls", "ls -l"));
    }

    #[test]
    fn test_edge_cases() {
        // Empty command
        assert_eq!(normalize_command(""), "");

        // Single flag
        assert_eq!(normalize_command("-l"), "-l");

        // Command with arguments containing spaces (quoted)
        assert_eq!(
            normalize_command("echo 'hello  world'"),
            "echo 'hello world'"
        );
    }

    // Safety validation tests
    #[tokio::test]
    async fn test_safety_dangerous_rm() {
        assert_eq!(validate_safety("rm -rf /").await, false);
        assert_eq!(validate_safety("rm -rf /*").await, false);
        assert_eq!(validate_safety("rm -rf /etc").await, false);
    }

    #[tokio::test]
    async fn test_safety_dangerous_dd() {
        assert_eq!(validate_safety("dd if=/dev/zero of=/dev/sda").await, false);
    }

    #[tokio::test]
    async fn test_safety_safe_commands() {
        assert_eq!(validate_safety("ls -la").await, true);
        assert_eq!(validate_safety("grep 'error' logs").await, true);
        assert_eq!(validate_safety("ps aux").await, true);
    }

    // POSIX compliance tests
    #[test]
    fn test_posix_compliant_commands() {
        assert_eq!(is_posix_compliant("[ -f file.txt ]"), true);
        assert_eq!(is_posix_compliant("test -f file.txt"), true);
        assert_eq!(is_posix_compliant("ls -la /tmp"), true);
        assert_eq!(is_posix_compliant("grep 'error' logs"), true);
    }

    #[test]
    fn test_bash_double_bracket_detected() {
        assert_eq!(is_posix_compliant("[[ -f file.txt ]]"), false);
        assert_eq!(is_posix_compliant("if [[ $x -gt 0 ]]; then"), false);
    }

    #[test]
    fn test_bash_brace_expansion_detected() {
        assert_eq!(is_posix_compliant("echo {1..10}"), false);
        assert_eq!(is_posix_compliant("mkdir dir{1..5}"), false);
    }

    #[test]
    fn test_bash_process_substitution_detected() {
        assert_eq!(is_posix_compliant("diff <(ls) <(ls -la)"), false);
    }

    #[test]
    fn test_bash_function_keyword_detected() {
        assert_eq!(is_posix_compliant("function foo() { echo bar; }"), false);
    }

    #[test]
    fn test_zsh_globstar_detected() {
        assert_eq!(is_posix_compliant("ls **/"), false);
        assert_eq!(is_posix_compliant("find **/*.txt"), false);
    }
}

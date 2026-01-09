//! Command validation (safety, POSIX compliance)

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
}

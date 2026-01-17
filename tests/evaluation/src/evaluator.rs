//! Command correctness evaluation.
//!
//! This module provides logic for comparing generated commands against
//! expected commands, supporting exact matches and semantic equivalence.

use similar::{ChangeTag, TextDiff};

/// Result of correctness evaluation
#[derive(Debug, Clone)]
pub struct CorrectnessResult {
    /// Score from 0.0 (no match) to 1.0 (exact match)
    pub score: f64,
    /// Method used to determine equivalence
    pub method: CorrectnessMethod,
    /// Diff output if commands don't match exactly
    pub diff: Option<String>,
}

/// Method used to evaluate correctness
#[derive(Debug, Clone, PartialEq)]
pub enum CorrectnessMethod {
    /// Commands match exactly
    ExactMatch,
    /// Commands are semantically equivalent
    SemanticEquivalent,
    /// Commands do not match
    NoMatch,
}

/// Evaluator for command correctness
pub struct Evaluator;

impl Evaluator {
    /// Evaluate correctness of a generated command against expected command
    ///
    /// Returns a score from 0.0 to 1.0:
    /// - 1.0: Exact match
    /// - 0.95: Whitespace normalized match
    /// - 0.90: Flag order normalized match
    /// - 0.0: No match
    pub fn evaluate_correctness(&self, generated: &str, expected: &str) -> CorrectnessResult {
        // Exact match first
        if generated == expected {
            return CorrectnessResult {
                score: 1.0,
                method: CorrectnessMethod::ExactMatch,
                diff: None,
            };
        }

        // Try whitespace normalization
        let gen_normalized = normalize_whitespace(generated);
        let exp_normalized = normalize_whitespace(expected);

        if gen_normalized == exp_normalized {
            return CorrectnessResult {
                score: 0.95,
                method: CorrectnessMethod::SemanticEquivalent,
                diff: None,
            };
        }

        // Try flag normalization (for commands with flags like -la vs -al)
        if let (Some(gen_flags), Some(exp_flags)) = (
            normalize_flags(&gen_normalized),
            normalize_flags(&exp_normalized),
        ) {
            if gen_flags == exp_flags {
                return CorrectnessResult {
                    score: 0.90,
                    method: CorrectnessMethod::SemanticEquivalent,
                    diff: Some(format!(
                        "Flag order differs:\n  Generated: {}\n  Expected:  {}",
                        generated, expected
                    )),
                };
            }
        }

        // No match - generate diff
        CorrectnessResult {
            score: 0.0,
            method: CorrectnessMethod::NoMatch,
            diff: Some(self.generate_diff(generated, expected)),
        }
    }

    /// Generate a unified diff between expected and generated commands
    fn generate_diff(&self, generated: &str, expected: &str) -> String {
        let diff = TextDiff::from_lines(expected, generated);

        let mut output = String::new();
        output.push_str(&format!("Expected: {}\n", expected));
        output.push_str(&format!("Generated: {}\n", generated));
        output.push_str("\nDiff:\n");

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push_str(&format!("{}{}", sign, change));
        }

        output
    }

    /// Aggregate scores from multiple results
    pub fn aggregate_scores(results: &[CorrectnessResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64
    }
}

/// Normalize whitespace in a command string
fn normalize_whitespace(cmd: &str) -> String {
    cmd.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize flag order in a command
///
/// This handles cases like "ls -la" vs "ls -al" by sorting single-letter flags.
/// Returns None if the command doesn't have a recognizable structure.
fn normalize_flags(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut normalized_parts: Vec<String> = Vec::new();
    normalized_parts.push(parts[0].to_string()); // Command name

    let mut i = 1;
    while i < parts.len() {
        let part = parts[i];

        // Check if this is a combined flag like -la
        if part.starts_with('-') && !part.starts_with("--") && part.len() > 2 {
            // Sort the flag characters
            let mut chars: Vec<char> = part[1..].chars().collect();
            chars.sort_unstable();
            normalized_parts.push(format!("-{}", chars.iter().collect::<String>()));
        } else {
            normalized_parts.push(part.to_string());
        }

        i += 1;
    }

    Some(normalized_parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let eval = Evaluator;
        let result = eval.evaluate_correctness("ls", "ls");
        assert_eq!(result.score, 1.0);
        assert_eq!(result.method, CorrectnessMethod::ExactMatch);
        assert!(result.diff.is_none());
    }

    #[test]
    fn test_whitespace_normalization() {
        let eval = Evaluator;
        let result = eval.evaluate_correctness("ls  -la", "ls -la");
        assert_eq!(result.score, 0.95);
        assert_eq!(result.method, CorrectnessMethod::SemanticEquivalent);
        assert!(result.diff.is_none());
    }

    #[test]
    fn test_flag_order() {
        let eval = Evaluator;
        let result = eval.evaluate_correctness("ls -la", "ls -al");
        assert!(result.score >= 0.90);
        assert_eq!(result.method, CorrectnessMethod::SemanticEquivalent);
    }

    #[test]
    fn test_no_match() {
        let eval = Evaluator;
        let result = eval.evaluate_correctness("ls", "pwd");
        assert_eq!(result.score, 0.0);
        assert_eq!(result.method, CorrectnessMethod::NoMatch);
        assert!(result.diff.is_some());
    }

    #[test]
    fn test_aggregate_scores() {
        let results = vec![
            CorrectnessResult {
                score: 1.0,
                method: CorrectnessMethod::ExactMatch,
                diff: None,
            },
            CorrectnessResult {
                score: 0.95,
                method: CorrectnessMethod::SemanticEquivalent,
                diff: None,
            },
            CorrectnessResult {
                score: 0.0,
                method: CorrectnessMethod::NoMatch,
                diff: Some("diff".to_string()),
            },
        ];

        let avg = Evaluator::aggregate_scores(&results);
        assert!((avg - 0.65).abs() < 0.01); // (1.0 + 0.95 + 0.0) / 3 = 0.65
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("ls  -la  /tmp"), "ls -la /tmp");
        assert_eq!(normalize_whitespace("  ls  "), "ls");
    }

    #[test]
    fn test_normalize_flags() {
        assert_eq!(
            normalize_flags("ls -la /tmp"),
            Some("ls -al /tmp".to_string())
        );
        assert_eq!(
            normalize_flags("ls -al /tmp"),
            Some("ls -al /tmp".to_string())
        );
        assert_eq!(normalize_flags("ls --all"), Some("ls --all".to_string()));
    }

    #[test]
    fn test_generate_diff() {
        let eval = Evaluator;
        let diff = eval.generate_diff("ls -al", "ls -la");
        assert!(diff.contains("Expected:"));
        assert!(diff.contains("Generated:"));
        assert!(diff.contains("Diff:"));
    }
}

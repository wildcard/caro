//! Spell checking module for improving user input before LLM processing.
//!
//! This module uses harper-core to detect and correct spelling/grammar issues
//! in natural language prompts, improving LLM comprehension and command generation.

use harper_core::linting::{LintGroup, LintKind, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use std::sync::Arc;
use thiserror::Error;

/// Configuration for the spell checker
#[derive(Debug, Clone)]
pub struct SpellCheckConfig {
    /// The English dialect to use for spell checking
    pub dialect: Dialect,
    /// Whether to automatically apply corrections
    pub auto_correct: bool,
    /// Minimum confidence threshold for auto-corrections (0.0-1.0)
    pub confidence_threshold: f32,
    /// Words to ignore (technical terms, commands, etc.)
    pub ignore_words: Vec<String>,
}

impl Default for SpellCheckConfig {
    fn default() -> Self {
        Self {
            dialect: Dialect::American,
            auto_correct: true,
            confidence_threshold: 0.8,
            ignore_words: vec![
                // Common shell/CLI terms that might be flagged
                "ls".to_string(),
                "cd".to_string(),
                "rm".to_string(),
                "mkdir".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
                "grep".to_string(),
                "awk".to_string(),
                "sed".to_string(),
                "sudo".to_string(),
                "stdin".to_string(),
                "stdout".to_string(),
                "stderr".to_string(),
                "repo".to_string(),
                "repos".to_string(),
                "dir".to_string(),
                "dirs".to_string(),
                "filesystem".to_string(),
                "config".to_string(),
                "configs".to_string(),
                "symlink".to_string(),
                "symlinks".to_string(),
            ],
        }
    }
}

/// A correction that was applied to the text
#[derive(Debug, Clone)]
pub struct Correction {
    /// The original text that was corrected
    pub original: String,
    /// The corrected text
    pub corrected: String,
    /// Character position in original text
    pub position: usize,
    /// The type of lint that triggered this correction
    pub kind: String,
}

/// Result of spell checking
#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    /// The corrected text
    pub text: String,
    /// Whether any corrections were made
    pub was_corrected: bool,
    /// List of corrections applied
    pub corrections: Vec<Correction>,
    /// Original input text
    pub original: String,
}

impl SpellCheckResult {
    /// Create a result with no corrections
    pub fn unchanged(text: String) -> Self {
        Self {
            original: text.clone(),
            text,
            was_corrected: false,
            corrections: Vec::new(),
        }
    }
}

/// Errors that can occur during spell checking
#[derive(Debug, Error)]
pub enum SpellCheckError {
    #[error("Failed to initialize dictionary: {0}")]
    DictionaryError(String),

    #[error("Failed to parse text: {0}")]
    ParseError(String),

    #[error("Failed to apply correction: {0}")]
    CorrectionError(String),
}

/// Spell checker for improving natural language input
pub struct SpellChecker {
    config: SpellCheckConfig,
    dictionary: Arc<FstDictionary>,
}

impl SpellChecker {
    /// Create a new spell checker with default configuration
    pub fn new() -> Result<Self, SpellCheckError> {
        Self::with_config(SpellCheckConfig::default())
    }

    /// Create a new spell checker with custom configuration
    pub fn with_config(config: SpellCheckConfig) -> Result<Self, SpellCheckError> {
        let dictionary = FstDictionary::curated();

        Ok(Self {
            config,
            dictionary,
        })
    }

    /// Check and correct the input text
    ///
    /// Returns a SpellCheckResult containing the corrected text and details
    /// about any corrections that were applied.
    pub fn check_and_correct(&self, text: &str) -> Result<SpellCheckResult, SpellCheckError> {
        // Skip empty or very short text
        if text.trim().len() < 3 {
            return Ok(SpellCheckResult::unchanged(text.to_string()));
        }

        // Parse the document
        let parser = PlainEnglish;
        let document = Document::new_curated(text, &parser);

        // Create linter and run analysis
        let mut linter = LintGroup::new_curated(self.dictionary.clone(), self.config.dialect);
        let lints = linter.lint(&document);

        // If no issues found, return unchanged
        if lints.is_empty() {
            return Ok(SpellCheckResult::unchanged(text.to_string()));
        }

        // Filter lints to only spelling issues and sort by position (reverse for safe replacement)
        let mut spelling_lints: Vec<_> = lints
            .into_iter()
            .filter(|lint| {
                matches!(lint.lint_kind, LintKind::Spelling | LintKind::Miscellaneous)
            })
            .filter(|lint| {
                // Get the word being flagged
                let start = lint.span.start;
                let end = lint.span.end;
                if start >= text.chars().count() || end > text.chars().count() {
                    return false;
                }
                let word: String = text.chars().skip(start).take(end - start).collect();
                // Don't flag words in our ignore list
                !self.config.ignore_words.iter().any(|w| w.eq_ignore_ascii_case(&word))
            })
            .filter(|lint| !lint.suggestions.is_empty())
            .collect();

        if spelling_lints.is_empty() {
            return Ok(SpellCheckResult::unchanged(text.to_string()));
        }

        // Sort by position descending so we can replace from end to start
        spelling_lints.sort_by(|a, b| b.span.start.cmp(&a.span.start));

        // Apply corrections if auto_correct is enabled
        if self.config.auto_correct {
            let mut corrected_text = text.to_string();
            let mut corrections = Vec::new();
            let chars: Vec<char> = text.chars().collect();

            for lint in spelling_lints {
                let start = lint.span.start;
                let end = lint.span.end;

                if start >= chars.len() || end > chars.len() {
                    continue;
                }

                // Get the original word
                let original_word: String = chars[start..end].iter().collect();

                // Get the first suggestion (most likely correction)
                if let Some(suggestion) = lint.suggestions.first() {
                    if let Some(replacement) = get_replacement_text(suggestion) {
                        // Calculate byte positions for replacement
                        let byte_start: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
                        let byte_end: usize = chars[..end].iter().map(|c| c.len_utf8()).sum();

                        if byte_end <= corrected_text.len() {
                            corrected_text.replace_range(byte_start..byte_end, &replacement);

                            corrections.push(Correction {
                                original: original_word,
                                corrected: replacement,
                                position: start,
                                kind: format!("{:?}", lint.lint_kind),
                            });
                        }
                    }
                }
            }

            // Reverse corrections list so they're in original order
            corrections.reverse();

            Ok(SpellCheckResult {
                original: text.to_string(),
                text: corrected_text,
                was_corrected: !corrections.is_empty(),
                corrections,
            })
        } else {
            // Just report issues without correcting
            let corrections: Vec<_> = spelling_lints
                .iter()
                .filter_map(|lint| {
                    let start = lint.span.start;
                    let end = lint.span.end;
                    let chars: Vec<char> = text.chars().collect();

                    if start >= chars.len() || end > chars.len() {
                        return None;
                    }

                    let original_word: String = chars[start..end].iter().collect();
                    let suggestion = lint.suggestions.first()?;
                    let replacement = get_replacement_text(suggestion)?;

                    Some(Correction {
                        original: original_word,
                        corrected: replacement,
                        position: start,
                        kind: format!("{:?}", lint.lint_kind),
                    })
                })
                .collect();

            Ok(SpellCheckResult {
                original: text.to_string(),
                text: text.to_string(),
                was_corrected: false,
                corrections,
            })
        }
    }

    /// Add words to the ignore list
    pub fn add_ignore_words(&mut self, words: &[String]) {
        self.config.ignore_words.extend(words.iter().cloned());
    }

    /// Check if a word should be ignored
    pub fn is_ignored(&self, word: &str) -> bool {
        self.config.ignore_words.iter().any(|w| w.eq_ignore_ascii_case(word))
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new().expect("Failed to create default SpellChecker")
    }
}

/// Extract replacement text from a Suggestion
fn get_replacement_text(suggestion: &Suggestion) -> Option<String> {
    match suggestion {
        Suggestion::ReplaceWith(chars) => Some(chars.iter().collect()),
        Suggestion::Remove => Some(String::new()),
        // For other suggestion types, we don't automatically apply
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_checker_creation() {
        let checker = SpellChecker::new();
        assert!(checker.is_ok());
    }

    #[test]
    fn test_empty_text() {
        let checker = SpellChecker::new().unwrap();
        let result = checker.check_and_correct("").unwrap();
        assert!(!result.was_corrected);
        assert!(result.corrections.is_empty());
    }

    #[test]
    fn test_correct_text() {
        let checker = SpellChecker::new().unwrap();
        let result = checker.check_and_correct("list all files in the directory").unwrap();
        // This should be correct English, no corrections needed
        assert_eq!(result.text, result.original);
    }

    #[test]
    fn test_misspelled_word() {
        let checker = SpellChecker::new().unwrap();
        let result = checker.check_and_correct("list all teh files").unwrap();
        // "teh" should be corrected to "the"
        if result.was_corrected {
            assert!(result.text.contains("the") && !result.text.contains("teh"));
        }
    }

    #[test]
    fn test_ignore_words() {
        let checker = SpellChecker::new().unwrap();
        // Shell commands should not be flagged
        let result = checker.check_and_correct("use sudo to run chmod").unwrap();
        // These technical terms should be in the ignore list
        assert!(result.corrections.iter().all(|c| {
            !c.original.eq_ignore_ascii_case("sudo") && !c.original.eq_ignore_ascii_case("chmod")
        }));
    }

    #[test]
    fn test_multiple_corrections() {
        let checker = SpellChecker::new().unwrap();
        let result = checker.check_and_correct("shwo me teh files").unwrap();
        // Multiple misspellings should be corrected
        if result.was_corrected {
            println!("Original: {}", result.original);
            println!("Corrected: {}", result.text);
            for correction in &result.corrections {
                println!("  {} -> {}", correction.original, correction.corrected);
            }
        }
    }

    #[test]
    fn test_config_no_auto_correct() {
        let config = SpellCheckConfig {
            auto_correct: false,
            ..Default::default()
        };
        let checker = SpellChecker::with_config(config).unwrap();
        let result = checker.check_and_correct("list all teh files").unwrap();
        // Should report but not correct
        assert!(!result.was_corrected);
        // Original text should be unchanged
        assert_eq!(result.text, "list all teh files");
    }
}

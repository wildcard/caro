//! Clarification system for ambiguous queries
//!
//! When a user query is too ambiguous to generate a reliable command,
//! this module generates clarifying questions to ask the user.

use serde::{Deserialize, Serialize};

/// Strategy for handling clarification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClarificationStrategy {
    /// Ask the user interactively
    AskUser,
    /// Use heuristics to make best guess
    BestGuess,
    /// Provide multiple options
    MultipleOptions,
    /// Skip clarification and proceed
    Skip,
}

impl Default for ClarificationStrategy {
    fn default() -> Self {
        Self::AskUser
    }
}

/// A question to ask the user for clarification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// Unique identifier for this question
    pub id: String,

    /// The question text to display
    pub question: String,

    /// Possible answers (if applicable)
    pub options: Vec<String>,

    /// Default answer (if applicable)
    pub default: Option<String>,

    /// Is this question required for command generation?
    pub required: bool,

    /// Context about why we're asking this question
    pub context: Option<String>,
}

impl ClarificationQuestion {
    /// Create a new clarification question
    pub fn new(id: impl Into<String>, question: impl Into<String>, options: Vec<impl Into<String>>) -> Self {
        Self {
            id: id.into(),
            question: question.into(),
            options: options.into_iter().map(|o| o.into()).collect(),
            default: None,
            required: true,
            context: None,
        }
    }

    /// Set the default answer
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set whether this question is required
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Add context explaining why we're asking
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Format for display
    pub fn format_for_display(&self) -> String {
        let mut output = self.question.clone();

        if !self.options.is_empty() {
            output.push_str("\n  Options:");
            for (i, opt) in self.options.iter().enumerate() {
                let marker = if Some(opt) == self.default.as_ref() {
                    " (default)"
                } else {
                    ""
                };
                output.push_str(&format!("\n    {}. {}{}", i + 1, opt, marker));
            }
        }

        if let Some(ref ctx) = self.context {
            output.push_str(&format!("\n  ({})", ctx));
        }

        output
    }
}

/// Result of a clarification round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationResult {
    /// Answers provided by the user
    pub answers: Vec<ClarificationAnswer>,

    /// Whether all required questions were answered
    pub complete: bool,

    /// Additional context provided by user
    pub additional_context: Option<String>,
}

impl ClarificationResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            answers: Vec::new(),
            complete: false,
            additional_context: None,
        }
    }

    /// Add an answer
    pub fn add_answer(&mut self, question_id: impl Into<String>, answer: impl Into<String>) {
        self.answers.push(ClarificationAnswer {
            question_id: question_id.into(),
            answer: answer.into(),
        });
    }

    /// Get answer for a specific question
    pub fn get_answer(&self, question_id: &str) -> Option<&str> {
        self.answers
            .iter()
            .find(|a| a.question_id == question_id)
            .map(|a| a.answer.as_str())
    }

    /// Mark as complete
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Convert answers to context string for prompt
    pub fn to_context_string(&self) -> String {
        if self.answers.is_empty() {
            return String::new();
        }

        let mut parts = vec!["USER CLARIFICATIONS:".to_string()];

        for answer in &self.answers {
            parts.push(format!("- {}: {}", answer.question_id, answer.answer));
        }

        if let Some(ref ctx) = self.additional_context {
            parts.push(format!("- Additional: {}", ctx));
        }

        parts.join("\n")
    }
}

impl Default for ClarificationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// A single answer to a clarification question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationAnswer {
    /// ID of the question this answers
    pub question_id: String,

    /// The answer provided
    pub answer: String,
}

/// Common clarification questions
pub struct CommonQuestions;

impl CommonQuestions {
    /// Question about target files
    pub fn target_files() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "target_files",
            "Which files should this command operate on?",
            vec![
                "All files in current directory",
                "Files matching a pattern (e.g., *.txt)",
                "A specific file or path",
                "Recursively in all subdirectories",
            ],
        )
        .with_context("This helps generate a more accurate command")
    }

    /// Question about action confirmation
    pub fn confirm_destructive() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "confirm_destructive",
            "This operation may modify or delete files. Are you sure?",
            vec!["Yes, proceed", "No, show preview first", "Cancel"],
        )
        .with_default("No, show preview first")
    }

    /// Question about tool preference
    pub fn tool_preference() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "tool_preference",
            "Which tool or package manager would you like to use?",
            vec!["npm", "yarn", "pnpm", "cargo", "pip", "make", "other"],
        )
        .with_context("Multiple tools detected in your project")
    }

    /// Question about output format
    pub fn output_format() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "output_format",
            "How would you like the output formatted?",
            vec![
                "Detailed (all information)",
                "Summary (key information only)",
                "Machine-readable (JSON/CSV)",
            ],
        )
        .with_default("Detailed (all information)")
        .optional()
    }

    /// Question about scope
    pub fn operation_scope() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "operation_scope",
            "What should be the scope of this operation?",
            vec![
                "Current directory only",
                "Current directory and subdirectories",
                "Entire project/repository",
                "System-wide",
            ],
        )
        .with_default("Current directory only")
    }

    /// Question about path
    pub fn target_path() -> ClarificationQuestion {
        ClarificationQuestion::new(
            "target_path",
            "Which path should this operate on?",
            vec!["Current directory (.)", "Home directory (~)", "Specific path"],
        )
        .with_default("Current directory (.)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarification_question_creation() {
        let question = ClarificationQuestion::new(
            "test_id",
            "What would you like to do?",
            vec!["Option A", "Option B"],
        );

        assert_eq!(question.id, "test_id");
        assert_eq!(question.options.len(), 2);
        assert!(question.required);
    }

    #[test]
    fn test_clarification_question_builder() {
        let question = ClarificationQuestion::new("test", "Test?", vec!["A", "B"])
            .with_default("A")
            .with_context("Testing")
            .optional();

        assert_eq!(question.default, Some("A".to_string()));
        assert_eq!(question.context, Some("Testing".to_string()));
        assert!(!question.required);
    }

    #[test]
    fn test_clarification_result() {
        let mut result = ClarificationResult::new();
        result.add_answer("q1", "Answer 1");
        result.add_answer("q2", "Answer 2");

        assert_eq!(result.get_answer("q1"), Some("Answer 1"));
        assert_eq!(result.get_answer("q2"), Some("Answer 2"));
        assert_eq!(result.get_answer("q3"), None);
    }

    #[test]
    fn test_format_for_display() {
        let question = ClarificationQuestion::new(
            "test",
            "Choose an option:",
            vec!["Option A", "Option B"],
        )
        .with_default("Option A")
        .with_context("Important decision");

        let display = question.format_for_display();
        assert!(display.contains("Choose an option:"));
        assert!(display.contains("Option A"));
        assert!(display.contains("(default)"));
        assert!(display.contains("Important decision"));
    }

    #[test]
    fn test_common_questions() {
        let q = CommonQuestions::target_files();
        assert_eq!(q.id, "target_files");
        assert!(!q.options.is_empty());

        let q = CommonQuestions::tool_preference();
        assert!(q.options.contains(&"npm".to_string()));
        assert!(q.options.contains(&"cargo".to_string()));
    }
}

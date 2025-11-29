//! Autocomplete inference module - LLM-powered command argument suggestion
//!
//! This module provides intelligent autocomplete for shell commands by combining:
//! - Static completion metadata (command signatures, argument types)
//! - LLM-based inference for context-aware suggestions
//! - Validation to ensure suggested arguments are correct
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐
//! │ User Input   │
//! └──────┬───────┘
//!        │
//!        ▼
//! ┌──────────────────────┐
//! │ CompletionContext    │  Parse command, extract metadata
//! └──────┬───────────────┘
//!        │
//!        ▼
//! ┌──────────────────────┐
//! │ InferenceAgent       │  LLM generates suggestions
//! └──────┬───────────────┘
//!        │
//!        ▼
//! ┌──────────────────────┐
//! │ ValidatorAgent       │  Verify suggestions are valid
//! └──────┬───────────────┘
//!        │
//!        ▼
//! ┌──────────────────────┐
//! │ Ranked Suggestions   │
//! └──────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use cmdai::autocomplete::{AutocompleteEngine, AutocompleteConfig};
//! use cmdai::backends::CommandGenerator;
//!
//! # async fn example(backend: Box<dyn CommandGenerator>) -> Result<(), Box<dyn std::error::Error>> {
//! let engine = AutocompleteEngine::new(AutocompleteConfig::default(), backend)?;
//! let suggestions = engine.suggest("git commit -m ", 14).await?;
//!
//! for suggestion in suggestions.candidates {
//!     println!("{}: {} (confidence: {:.2})",
//!         suggestion.value,
//!         suggestion.description,
//!         suggestion.confidence
//!     );
//! }
//! # Ok(())
//! # }
//! ```

pub mod context;
pub mod inference;
pub mod validator;

use serde::{Deserialize, Serialize};

use crate::backends::CommandGenerator;
use crate::models::ShellType;

pub use context::{ArgumentSpec, CommandSignature, CompletionContext, FlagSpec, SubcommandSpec};
pub use inference::{InferenceAgent, InferenceConfig};
pub use validator::{ArgumentValidator, ValidatorConfig};

/// Main autocomplete engine orchestrating inference and validation
pub struct AutocompleteEngine {
    config: AutocompleteConfig,
    context: CompletionContext,
    inference: InferenceAgent,
    validator: ArgumentValidator,
}

/// Configuration for autocomplete behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteConfig {
    /// Maximum number of suggestions to return
    pub max_suggestions: usize,
    /// Minimum confidence score to include suggestion (0.0 to 1.0)
    pub min_confidence: f32,
    /// Whether to validate suggestions before returning
    pub enable_validation: bool,
    /// Shell type for context-aware completion
    pub shell_type: ShellType,
}

impl Default for AutocompleteConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 10,
            min_confidence: 0.3,
            enable_validation: true,
            shell_type: ShellType::default(),
        }
    }
}

/// Result of autocomplete suggestion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionResult {
    /// The partial command that was completed
    pub partial_command: String,
    /// Cursor position in the partial command
    pub cursor_position: usize,
    /// List of suggested completions
    pub candidates: Vec<Candidate>,
    /// Context used for suggestion
    pub context_used: String,
    /// Time taken to generate suggestions in milliseconds
    pub generation_time_ms: u64,
}

/// A single completion candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    /// The suggested completion value
    pub value: String,
    /// Human-readable description of what this does
    pub description: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Type of argument (file, flag, value, etc.)
    pub arg_type: ArgumentType,
    /// Whether this passed validation
    pub validated: bool,
}

/// Type of command argument
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    /// Command flag (e.g., --verbose)
    Flag,
    /// Option with value (e.g., --output=file.txt)
    Option,
    /// File path
    File,
    /// Directory path
    Directory,
    /// Plain text value
    Value,
    /// Subcommand
    Subcommand,
}

impl AutocompleteEngine {
    /// Create new autocomplete engine with given configuration
    pub fn new(
        config: AutocompleteConfig,
        backend: Box<dyn CommandGenerator>,
    ) -> Result<Self, AutocompleteError> {
        let context = CompletionContext::new()?;
        let inference = InferenceAgent::new(InferenceConfig::default(), backend)?;
        let validator = ArgumentValidator::new(ValidatorConfig::default())?;

        Ok(Self {
            config,
            context,
            inference,
            validator,
        })
    }

    /// Generate suggestions for partial command at cursor position
    pub async fn suggest(
        &self,
        partial_command: &str,
        cursor_position: usize,
    ) -> Result<SuggestionResult, AutocompleteError> {
        let start_time = std::time::Instant::now();

        // Parse command and extract context
        let cmd_context = self
            .context
            .get_context_for_command(partial_command, cursor_position)?;

        // Get suggestions from inference agent
        let mut candidates = self
            .inference
            .infer_completions(partial_command, cursor_position, &cmd_context)
            .await?;

        // Validate suggestions if enabled
        if self.config.enable_validation {
            for candidate in &mut candidates {
                let validation = self.validator.validate(&candidate.value, &cmd_context).await?;
                candidate.validated = validation.is_valid;
                if !validation.is_valid {
                    // Reduce confidence for invalid suggestions
                    candidate.confidence *= 0.5;
                }
            }
        }

        // Filter by confidence threshold
        candidates.retain(|c| c.confidence >= self.config.min_confidence);

        // Sort by confidence (descending)
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to max suggestions
        candidates.truncate(self.config.max_suggestions);

        let generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(SuggestionResult {
            partial_command: partial_command.to_string(),
            cursor_position,
            candidates,
            context_used: cmd_context.to_string(),
            generation_time_ms,
        })
    }

    /// Load custom completion definitions from file
    pub fn load_completions_from_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), AutocompleteError> {
        self.context.load_from_file(path)
    }

    /// Add a custom command signature
    pub fn add_command_signature(&mut self, signature: CommandSignature) {
        self.context.add_signature(signature);
    }
}

/// Errors that can occur during autocomplete operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum AutocompleteError {
    #[error("Autocomplete not implemented yet")]
    NotImplemented,

    #[error("Invalid cursor position: {position} (command length: {length})")]
    InvalidCursorPosition { position: usize, length: usize },

    #[error("Context parsing failed: {message}")]
    ContextError { message: String },

    #[error("Inference failed: {details}")]
    InferenceFailed { details: String },

    #[error("Validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl From<std::io::Error> for AutocompleteError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            message: err.to_string(),
        }
    }
}

//! Agent Reasoning Mode - Pre-processing for intelligent query analysis
//!
//! This module provides a reasoning layer that analyzes user queries before command
//! generation to determine what context is needed for higher quality results.
//!
//! # Overview
//!
//! The reasoning mode performs pre-processing to:
//! - Analyze query ambiguity and determine if clarification is needed
//! - Detect what context would improve command generation (files, project type, OS info)
//! - Identify if the query is relative to current directory or general
//! - Detect project toolchain (Node, Rust, Python, Make, etc.)
//!
//! # Architecture
//!
//! ```text
//! User Query
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Query Analyzer  │──── Is query ambiguous?
//! │                 │──── What context is needed?
//! │                 │──── Is query relative or general?
//! └─────────────────┘
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Context Scout   │──── Project detection (Starship-like)
//! │                 │──── Directory structure analysis
//! │                 │──── Available tools detection
//! └─────────────────┘
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Enrichment      │──── Auto-fetch safe context
//! │ Strategy        │──── Request permission for commands
//! │                 │──── Ask user for clarification
//! └─────────────────┘
//!     │
//!     ▼
//! Enhanced Prompt → Command Generation
//! ```

mod analyzer;
mod clarification;
mod config;
mod enrichment;
mod project_detection;

pub use analyzer::{QueryAnalysis, QueryAnalyzer, QueryClassification, ContextNeed};
pub use clarification::{ClarificationQuestion, ClarificationResult, ClarificationStrategy};
pub use config::{ReasoningConfig, ReasoningMode, ContextFetchPolicy};
pub use enrichment::{ContextEnricher, EnrichedContext, EnrichmentStrategy};
pub use project_detection::{ProjectDetector, ProjectContext, ProjectType, ToolchainInfo};

use crate::context::ExecutionContext;
use serde::{Deserialize, Serialize};

/// Result of the reasoning pre-processing step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    /// Original user query
    pub original_query: String,

    /// Query analysis results
    pub analysis: QueryAnalysis,

    /// Enriched context (if gathered)
    pub enriched_context: Option<EnrichedContext>,

    /// Clarification questions (if needed)
    pub clarifications_needed: Vec<ClarificationQuestion>,

    /// Whether we should proceed with generation or wait for clarification
    pub ready_to_generate: bool,

    /// Enhanced prompt incorporating gathered context
    pub enhanced_prompt: Option<String>,

    /// Reasoning chain explaining the analysis
    pub reasoning_chain: Vec<String>,
}

impl ReasoningResult {
    /// Create a result indicating we're ready to generate
    pub fn ready(query: String, analysis: QueryAnalysis) -> Self {
        Self {
            original_query: query,
            analysis,
            enriched_context: None,
            clarifications_needed: Vec::new(),
            ready_to_generate: true,
            enhanced_prompt: None,
            reasoning_chain: Vec::new(),
        }
    }

    /// Create a result indicating clarification is needed
    pub fn needs_clarification(
        query: String,
        analysis: QueryAnalysis,
        questions: Vec<ClarificationQuestion>,
    ) -> Self {
        Self {
            original_query: query,
            analysis,
            enriched_context: None,
            clarifications_needed: questions,
            ready_to_generate: false,
            enhanced_prompt: None,
            reasoning_chain: Vec::new(),
        }
    }

    /// Add enriched context to the result
    pub fn with_context(mut self, context: EnrichedContext) -> Self {
        self.enriched_context = Some(context);
        self
    }

    /// Add reasoning chain explanation
    pub fn with_reasoning(mut self, reasoning: Vec<String>) -> Self {
        self.reasoning_chain = reasoning;
        self
    }

    /// Set the enhanced prompt
    pub fn with_enhanced_prompt(mut self, prompt: String) -> Self {
        self.enhanced_prompt = Some(prompt);
        self
    }
}

/// The main reasoning engine that orchestrates query analysis and context enrichment
pub struct ReasoningEngine {
    config: ReasoningConfig,
    analyzer: QueryAnalyzer,
    project_detector: ProjectDetector,
    enricher: ContextEnricher,
}

impl ReasoningEngine {
    /// Create a new reasoning engine with the given configuration
    pub fn new(config: ReasoningConfig) -> Self {
        Self {
            config: config.clone(),
            analyzer: QueryAnalyzer::new(),
            project_detector: ProjectDetector::new(),
            enricher: ContextEnricher::new(config),
        }
    }

    /// Create a reasoning engine with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ReasoningConfig::default())
    }

    /// Analyze a user query and determine context needs
    ///
    /// This is the main entry point for the reasoning mode. It:
    /// 1. Analyzes the query for ambiguity and context needs
    /// 2. Detects project context if relevant
    /// 3. Determines what enrichment is needed
    /// 4. Returns a result with recommendations
    pub async fn analyze(
        &self,
        query: &str,
        execution_context: &ExecutionContext,
    ) -> ReasoningResult {
        let mut reasoning_chain = Vec::new();

        // Step 1: Analyze the query
        reasoning_chain.push(format!("Analyzing query: '{}'", query));
        let analysis = self.analyzer.analyze(query, execution_context);

        reasoning_chain.push(format!(
            "Query classification: {:?}, confidence: {:.2}",
            analysis.classification, analysis.confidence
        ));

        // Step 2: Check if query is too ambiguous
        if analysis.is_too_ambiguous() && self.config.mode.allows_clarification() {
            let questions = self.generate_clarification_questions(&analysis);
            if !questions.is_empty() {
                reasoning_chain.push("Query is ambiguous, generating clarification questions".to_string());
                return ReasoningResult::needs_clarification(query.to_string(), analysis, questions)
                    .with_reasoning(reasoning_chain);
            }
        }

        // Step 3: Detect project context if query is directory-relative
        let project_context = if analysis.is_directory_relative() {
            reasoning_chain.push("Query is directory-relative, detecting project context".to_string());
            Some(self.project_detector.detect(&execution_context.cwd))
        } else {
            None
        };

        // Step 4: Determine context enrichment strategy
        let enriched_context = if !analysis.context_needs.is_empty() {
            reasoning_chain.push(format!(
                "Context needs identified: {:?}",
                analysis.context_needs
            ));

            match self.enricher.enrich(&analysis, execution_context, project_context.as_ref()).await {
                Ok(ctx) => {
                    reasoning_chain.push("Context enrichment successful".to_string());
                    Some(ctx)
                }
                Err(e) => {
                    reasoning_chain.push(format!("Context enrichment failed: {}", e));
                    None
                }
            }
        } else {
            None
        };

        // Step 5: Build enhanced prompt
        let enhanced_prompt = self.build_enhanced_prompt(
            query,
            &analysis,
            enriched_context.as_ref(),
            project_context.as_ref(),
        );

        reasoning_chain.push("Ready to generate command".to_string());

        ReasoningResult::ready(query.to_string(), analysis)
            .with_context(enriched_context.unwrap_or_default())
            .with_enhanced_prompt(enhanced_prompt)
            .with_reasoning(reasoning_chain)
    }

    /// Quick analysis without enrichment (for fast path)
    pub fn quick_analyze(&self, query: &str, execution_context: &ExecutionContext) -> QueryAnalysis {
        self.analyzer.analyze(query, execution_context)
    }

    /// Check if reasoning mode should be applied to this query
    pub fn should_apply_reasoning(&self, query: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Skip for very simple queries
        let word_count = query.split_whitespace().count();
        if word_count <= 2 {
            return false;
        }

        // Apply for queries that might benefit from context
        true
    }

    /// Generate clarification questions based on analysis
    fn generate_clarification_questions(&self, analysis: &QueryAnalysis) -> Vec<ClarificationQuestion> {
        let mut questions = Vec::new();

        // Check for ambiguous targets
        if analysis.has_ambiguous_target() {
            questions.push(ClarificationQuestion::new(
                "target",
                "Which files or directories should this command target?",
                vec!["current directory", "specific path", "all matching files"],
            ));
        }

        // Check for ambiguous action
        if analysis.has_ambiguous_action() {
            questions.push(ClarificationQuestion::new(
                "action",
                "What specific action would you like to perform?",
                Vec::<String>::new(),
            ));
        }

        // Check for missing tool context
        if analysis.needs_tool_context() {
            questions.push(ClarificationQuestion::new(
                "tool",
                "Which tool or package manager are you using?",
                vec!["npm", "cargo", "pip", "make", "other"],
            ));
        }

        questions
    }

    /// Build an enhanced prompt incorporating gathered context
    fn build_enhanced_prompt(
        &self,
        query: &str,
        analysis: &QueryAnalysis,
        enriched_context: Option<&EnrichedContext>,
        project_context: Option<&ProjectContext>,
    ) -> String {
        let mut parts = Vec::new();

        // Add project context if available
        if let Some(project) = project_context {
            if project.project_type != ProjectType::Unknown {
                parts.push(format!(
                    "PROJECT CONTEXT:\n- Type: {:?}\n- Root: {}\n- Available tools: {:?}",
                    project.project_type,
                    project.root.display(),
                    project.available_tools
                ));
            }
        }

        // Add enriched context if available
        if let Some(ctx) = enriched_context {
            if !ctx.directory_listing.is_empty() {
                parts.push(format!(
                    "DIRECTORY CONTENTS:\n{}",
                    ctx.directory_listing.join("\n")
                ));
            }

            if !ctx.file_tree.is_empty() {
                parts.push(format!("FILE STRUCTURE:\n{}", ctx.file_tree));
            }
        }

        // Add analysis insights
        if analysis.is_directory_relative() {
            parts.push("NOTE: This query is relative to the current working directory.".to_string());
        }

        if !parts.is_empty() {
            format!(
                "{}\n\nCONTEXT:\n{}\n\nUSER REQUEST: {}",
                parts.join("\n\n"),
                "",
                query
            )
        } else {
            query.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_result_ready() {
        let analysis = QueryAnalysis::default();
        let result = ReasoningResult::ready("test query".to_string(), analysis);
        assert!(result.ready_to_generate);
        assert!(result.clarifications_needed.is_empty());
    }

    #[test]
    fn test_reasoning_result_needs_clarification() {
        let analysis = QueryAnalysis::default();
        let questions = vec![ClarificationQuestion::new(
            "test",
            "Test question?",
            vec!["a", "b"],
        )];
        let result = ReasoningResult::needs_clarification(
            "ambiguous query".to_string(),
            analysis,
            questions,
        );
        assert!(!result.ready_to_generate);
        assert_eq!(result.clarifications_needed.len(), 1);
    }
}

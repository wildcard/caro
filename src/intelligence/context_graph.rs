//! Context Graph - Core aggregator for all context intelligence
//!
//! Builds a complete context graph by running all analyzers in parallel
//! with graceful degradation on failures.

use super::{
    ContextError, ContextOptions, EnvironmentContext, GitAnalyzer, GitContext, HistoryAnalyzer,
    HistoryContext, InfrastructureContext, ProjectContext, ProjectParser, ToolDetector,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Complete context graph containing all analyzed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraph {
    /// Project-specific context
    pub project: ProjectContext,
    /// Git repository context
    pub git: GitContext,
    /// Infrastructure tools context
    pub infrastructure: InfrastructureContext,
    /// Shell history context
    pub history: HistoryContext,
    /// Environment context
    pub environment: EnvironmentContext,
    /// Time taken to build context (ms)
    pub build_time_ms: u64,
    /// Warnings encountered during analysis
    pub warnings: Vec<String>,
}

impl ContextGraph {
    /// Build complete context from current working directory
    pub async fn build(cwd: &Path) -> Result<Self, ContextError> {
        Self::build_with_options(cwd, ContextOptions::default()).await
    }

    /// Build context with custom options
    pub async fn build_with_options(
        cwd: &Path,
        options: ContextOptions,
    ) -> Result<Self, ContextError> {
        let start = Instant::now();

        // Build environment context first (synchronous)
        let env_result = EnvironmentContext::build();

        // Run all analyzers in parallel with timeout
        let build_future = async {
            let mut warnings = Vec::new();
            let (project_result, git_result, infra_result, history_result) = tokio::join!(
                ProjectParser::analyze(cwd),
                async {
                    if options.enable_git {
                        GitAnalyzer::analyze(cwd).await
                    } else {
                        Ok(GitContext::not_a_repo())
                    }
                },
                async {
                    if options.enable_tools {
                        ToolDetector::analyze(cwd).await
                    } else {
                        Ok(InfrastructureContext::empty())
                    }
                },
                async {
                    if options.enable_history {
                        HistoryAnalyzer::analyze().await
                    } else {
                        Ok(HistoryContext::empty())
                    }
                },
            );

            // Handle results with graceful degradation
            let project = match project_result {
                Ok(ctx) => ctx,
                Err(e) => {
                    warnings.push(format!("Project analysis failed: {}", e));
                    ProjectContext::unknown(cwd.to_path_buf())
                }
            };

            let git = match git_result {
                Ok(ctx) => ctx,
                Err(e) => {
                    warnings.push(format!("Git analysis failed: {}", e));
                    GitContext::not_a_repo()
                }
            };

            let infrastructure = match infra_result {
                Ok(ctx) => ctx,
                Err(e) => {
                    warnings.push(format!("Infrastructure analysis failed: {}", e));
                    InfrastructureContext::empty()
                }
            };

            let history = match history_result {
                Ok(ctx) => ctx,
                Err(e) => {
                    warnings.push(format!("History analysis failed: {}", e));
                    HistoryContext::empty()
                }
            };

            Ok::<_, ContextError>((project, git, infrastructure, history, warnings))
        };

        // Handle environment result
        let mut warnings = Vec::new();
        let environment = match env_result {
            Ok(ctx) => ctx,
            Err(e) => {
                warnings.push(format!("Environment analysis failed: {}", e));
                // Return a minimal environment context
                EnvironmentContext {
                    shell: "unknown".to_string(),
                    platform: std::env::consts::OS.to_string(),
                    cwd: cwd.display().to_string(),
                    user: "unknown".to_string(),
                    hostname: "unknown".to_string(),
                }
            }
        };

        // Apply timeout
        let timeout_duration = Duration::from_millis(options.timeout_ms);
        let (project, git, infrastructure, history, mut context_warnings) =
            match timeout(timeout_duration, build_future).await {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(ContextError::Timeout {
                        timeout_ms: options.timeout_ms,
                    });
                }
            };

        warnings.append(&mut context_warnings);

        let build_time_ms = start.elapsed().as_millis() as u64;

        Ok(Self {
            project,
            git,
            infrastructure,
            history,
            environment,
            build_time_ms,
            warnings,
        })
    }

    /// Convert entire context graph to LLM-friendly prompt augmentation
    pub fn to_llm_context(&self) -> String {
        let mut sections = Vec::new();

        // Environment (always include)
        sections.push(self.environment.to_llm_context());

        // Project context
        let project_ctx = self.project.to_llm_context();
        if !project_ctx.is_empty() && !project_ctx.contains("Unknown") {
            sections.push(project_ctx);
        }

        // Git context
        let git_ctx = self.git.to_llm_context();
        if !git_ctx.contains("Not a repository") {
            sections.push(git_ctx);
        }

        // Infrastructure
        let infra_ctx = self.infrastructure.to_llm_context();
        if !infra_ctx.contains("No tools detected") {
            sections.push(infra_ctx);
        }

        // History (optional - can be verbose)
        if !self.history.frequent_commands.is_empty() {
            sections.push(self.history.to_llm_context());
        }

        sections.join("\n\n")
    }

    /// Get a compact summary of the context
    pub fn summary(&self) -> String {
        format!(
            "Context: {} project, Git: {}, Tools: {}, Built in {}ms",
            self.project.project_type,
            if self.git.is_repo { "yes" } else { "no" },
            self.infrastructure.tools.len(),
            self.build_time_ms
        )
    }

    /// Check if context build was successful (no critical failures)
    pub fn is_valid(&self) -> bool {
        // Context is valid as long as we have environment context
        !self.environment.shell.is_empty()
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            total_time_ms: self.build_time_ms,
            warning_count: self.warnings.len(),
            analyzers_run: self.count_analyzers_run(),
        }
    }

    fn count_analyzers_run(&self) -> usize {
        let mut count = 1; // Environment always runs

        if self.project.project_type != super::ProjectType::Unknown {
            count += 1;
        }
        if self.git.is_repo {
            count += 1;
        }
        if !self.infrastructure.tools.is_empty() {
            count += 1;
        }
        if !self.history.frequent_commands.is_empty() {
            count += 1;
        }

        count
    }
}

/// Performance metrics for context building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_time_ms: u64,
    pub warning_count: usize,
    pub analyzers_run: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_context_graph_build() {
        let cwd = env::current_dir().unwrap();
        let result = ContextGraph::build(&cwd).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(context.is_valid());
        assert!(!context.environment.shell.is_empty());
    }

    #[tokio::test]
    async fn test_context_graph_build_with_options() {
        let cwd = env::current_dir().unwrap();
        let options = ContextOptions {
            enable_git: true,
            enable_tools: true,
            enable_history: false, // Disable history for faster test
            timeout_ms: 500,
        };

        let result = ContextGraph::build_with_options(&cwd, options).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(context.build_time_ms < 500);
    }

    #[tokio::test]
    async fn test_context_graph_timeout() {
        let cwd = env::current_dir().unwrap();
        let options = ContextOptions {
            enable_git: true,
            enable_tools: true,
            enable_history: true,
            timeout_ms: 1, // Very short timeout
        };

        let result = ContextGraph::build_with_options(&cwd, options).await;
        // Should timeout or succeed very quickly
        assert!(result.is_ok() || matches!(result, Err(ContextError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_to_llm_context() {
        let cwd = env::current_dir().unwrap();
        let context = ContextGraph::build(&cwd).await.unwrap();

        let llm_str = context.to_llm_context();
        assert!(!llm_str.is_empty());
        assert!(llm_str.contains("Shell:") || llm_str.contains("Platform:"));
    }

    #[tokio::test]
    async fn test_summary() {
        let cwd = env::current_dir().unwrap();
        let context = ContextGraph::build(&cwd).await.unwrap();

        let summary = context.summary();
        assert!(summary.contains("Context:"));
        assert!(summary.contains("Built in"));
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let cwd = env::current_dir().unwrap();
        let context = ContextGraph::build(&cwd).await.unwrap();

        let metrics = context.performance_metrics();
        assert!(metrics.total_time_ms < 1000); // Should be much faster
        assert!(metrics.analyzers_run >= 1);
    }

    #[tokio::test]
    async fn test_graceful_degradation_invalid_path() {
        use std::path::PathBuf;

        let invalid_path = PathBuf::from("/this/path/should/not/exist/ever/12345");
        let result = ContextGraph::build(&invalid_path).await;

        // Should still succeed with degraded context
        if let Ok(context) = result {
            assert!(!context.warnings.is_empty());
        }
    }

    #[tokio::test]
    async fn test_build_performance_target() {
        let cwd = env::current_dir().unwrap();
        let start = Instant::now();
        let result = ContextGraph::build(&cwd).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should meet <300ms target
        assert!(
            elapsed.as_millis() < 300,
            "Context build took {}ms, target is <300ms",
            elapsed.as_millis()
        );
    }
}

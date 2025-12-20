//! Analysis coordinator - orchestrates parallel environment analysis

use super::{
    environment::{EnvironmentAnalyzer, EnvironmentInsights},
    history::{CommandPatterns, HistoryAnalyzer},
    profile::{ExperienceLevel, ProfileManager, UserProfile, Workflow},
    tools::{DetectedTool, ToolCategory, ToolsAnalyzer},
    Result, SuggestionsConfig,
};
use chrono::Utc;
use std::time::Duration;

/// Coordinates parallel analysis of user environment
pub struct AnalysisCoordinator {
    config: SuggestionsConfig,
    profile_manager: ProfileManager,
}

impl AnalysisCoordinator {
    /// Create a new analysis coordinator
    pub fn new(config: SuggestionsConfig) -> Result<Self> {
        let profile_manager = ProfileManager::new()?;
        Ok(Self {
            config,
            profile_manager,
        })
    }

    /// Create with custom profile manager (for testing)
    pub fn with_profile_manager(config: SuggestionsConfig, profile_manager: ProfileManager) -> Self {
        Self {
            config,
            profile_manager,
        }
    }

    /// Get or create user profile, refreshing if stale
    pub async fn get_profile(&self) -> Result<UserProfile> {
        // Try to load existing profile
        if let Some(profile) = self.profile_manager.load()? {
            // Check if profile is still fresh
            if !profile.is_stale(self.config.cache_ttl_secs) {
                return Ok(profile);
            }
        }

        // Perform fresh analysis
        let profile = self.analyze().await?;

        // Save the profile
        self.profile_manager.save(&profile)?;

        Ok(profile)
    }

    /// Force refresh of user profile
    pub async fn refresh_profile(&self) -> Result<UserProfile> {
        let profile = self.analyze().await?;
        self.profile_manager.save(&profile)?;
        Ok(profile)
    }

    /// Run full environment analysis
    pub async fn analyze(&self) -> Result<UserProfile> {
        let timeout = Duration::from_secs(self.config.analysis_timeout_secs);

        // Run all analyzers in parallel with timeout
        let analysis_result = tokio::time::timeout(timeout, self.run_analysis()).await;

        match analysis_result {
            Ok(result) => result,
            Err(_) => {
                // Timeout - return partial results or default
                tracing::warn!("Analysis timed out, using defaults");
                Ok(UserProfile::default())
            }
        }
    }

    /// Run all analyzers in parallel
    async fn run_analysis(&self) -> Result<UserProfile> {
        // Create analyzers
        let history_analyzer = HistoryAnalyzer::new();
        let tools_analyzer = ToolsAnalyzer::new();
        let env_analyzer = EnvironmentAnalyzer::new();

        // Run in parallel
        let (history_result, tools_result, env_result) = tokio::join!(
            history_analyzer.analyze(),
            tools_analyzer.analyze(),
            env_analyzer.analyze()
        );

        // Handle results (use defaults on failure)
        let command_patterns = history_result.unwrap_or_else(|e| {
            tracing::warn!("History analysis failed: {}", e);
            CommandPatterns::default()
        });

        let detected_tools = tools_result.unwrap_or_else(|e| {
            tracing::warn!("Tools analysis failed: {}", e);
            Vec::new()
        });

        let environment_insights = env_result.unwrap_or_else(|e| {
            tracing::warn!("Environment analysis failed: {}", e);
            EnvironmentInsights::default()
        });

        // Infer experience level
        let dev_tools_count = detected_tools
            .iter()
            .filter(|t| {
                matches!(
                    t.category,
                    ToolCategory::VersionControl
                        | ToolCategory::PackageManager
                        | ToolCategory::Language
                        | ToolCategory::ContainerRuntime
                )
            })
            .count();

        let experience_level = ExperienceLevel::infer(
            command_patterns.total_commands,
            command_patterns.unique_commands,
            dev_tools_count,
        );

        // Detect workflows
        let workflows = self.detect_workflows(&command_patterns, &detected_tools);

        Ok(UserProfile {
            version: "1.0.0".to_string(),
            last_analyzed: Utc::now(),
            experience_level,
            workflows,
            command_patterns,
            detected_tools,
            environment_insights,
        })
    }

    /// Detect workflows based on command patterns and tools
    fn detect_workflows(
        &self,
        patterns: &CommandPatterns,
        tools: &[DetectedTool],
    ) -> Vec<Workflow> {
        let mut workflows = Vec::new();

        // Git workflow
        let git_commands: u32 = patterns
            .top_commands
            .iter()
            .filter(|(cmd, _)| cmd == "git")
            .map(|(_, count)| count)
            .sum();

        if git_commands > 10 || tools.iter().any(|t| t.name == "git") {
            let confidence = (git_commands as f32 / 100.0).min(1.0).max(0.5);
            workflows.push(Workflow::new(
                "git",
                confidence,
                vec![
                    "git status".to_string(),
                    "git add".to_string(),
                    "git commit".to_string(),
                    "git push".to_string(),
                ],
            ));
        }

        // Docker workflow
        let docker_commands: u32 = patterns
            .top_commands
            .iter()
            .filter(|(cmd, _)| cmd == "docker" || cmd == "docker-compose")
            .map(|(_, count)| count)
            .sum();

        if docker_commands > 5 || tools.iter().any(|t| t.name == "docker") {
            let confidence = (docker_commands as f32 / 50.0).min(1.0).max(0.4);
            workflows.push(Workflow::new(
                "docker",
                confidence,
                vec![
                    "docker ps".to_string(),
                    "docker build".to_string(),
                    "docker run".to_string(),
                ],
            ));
        }

        // Node.js development
        let node_commands: u32 = patterns
            .top_commands
            .iter()
            .filter(|(cmd, _)| cmd == "npm" || cmd == "yarn" || cmd == "pnpm" || cmd == "node")
            .map(|(_, count)| count)
            .sum();

        if node_commands > 10 || tools.iter().any(|t| t.name == "npm" || t.name == "node") {
            let confidence = (node_commands as f32 / 100.0).min(1.0).max(0.4);
            workflows.push(Workflow::new(
                "node-dev",
                confidence,
                vec![
                    "npm install".to_string(),
                    "npm run".to_string(),
                    "npm test".to_string(),
                ],
            ));
        }

        // Rust development
        let rust_commands: u32 = patterns
            .top_commands
            .iter()
            .filter(|(cmd, _)| cmd == "cargo" || cmd == "rustc")
            .map(|(_, count)| count)
            .sum();

        if rust_commands > 10 || tools.iter().any(|t| t.name == "cargo") {
            let confidence = (rust_commands as f32 / 100.0).min(1.0).max(0.4);
            workflows.push(Workflow::new(
                "rust-dev",
                confidence,
                vec![
                    "cargo build".to_string(),
                    "cargo test".to_string(),
                    "cargo run".to_string(),
                ],
            ));
        }

        // Python development
        let python_commands: u32 = patterns
            .top_commands
            .iter()
            .filter(|(cmd, _)| cmd == "python" || cmd == "python3" || cmd == "pip" || cmd == "pip3")
            .map(|(_, count)| count)
            .sum();

        if python_commands > 10 || tools.iter().any(|t| t.name == "python" || t.name == "python3") {
            let confidence = (python_commands as f32 / 100.0).min(1.0).max(0.4);
            workflows.push(Workflow::new(
                "python-dev",
                confidence,
                vec![
                    "python".to_string(),
                    "pip install".to_string(),
                    "pytest".to_string(),
                ],
            ));
        }

        // Sort by confidence
        workflows.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        workflows
    }

    /// Check if this appears to be a new user
    pub fn is_new_user(profile: &UserProfile) -> bool {
        profile.experience_level == ExperienceLevel::Beginner
            && profile.command_patterns.total_commands < 100
            && profile.detected_tools.len() < 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = SuggestionsConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let profile_path = temp_dir.path().join("profile.json");
        let profile_manager = ProfileManager::with_path(profile_path);

        let coordinator = AnalysisCoordinator::with_profile_manager(config, profile_manager);
        // Just verify it can be created
        let _ = coordinator;
    }

    #[test]
    fn test_workflow_detection() {
        let config = SuggestionsConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let profile_path = temp_dir.path().join("profile.json");
        let profile_manager = ProfileManager::with_path(profile_path);
        let coordinator = AnalysisCoordinator::with_profile_manager(config, profile_manager);

        let patterns = CommandPatterns {
            top_commands: vec![
                ("git".to_string(), 50),
                ("cargo".to_string(), 30),
                ("ls".to_string(), 100),
            ],
            common_sequences: Vec::new(),
            usage_hours: [0; 24],
            total_commands: 500,
            unique_commands: 50,
        };

        let tools = vec![
            DetectedTool::new("git", "/usr/bin/git".into(), ToolCategory::VersionControl),
            DetectedTool::new("cargo", "/usr/bin/cargo".into(), ToolCategory::PackageManager),
        ];

        let workflows = coordinator.detect_workflows(&patterns, &tools);

        assert!(!workflows.is_empty());
        assert!(workflows.iter().any(|w| w.name == "git"));
        assert!(workflows.iter().any(|w| w.name == "rust-dev"));
    }

    #[test]
    fn test_is_new_user() {
        let mut profile = UserProfile::default();
        profile.experience_level = ExperienceLevel::Beginner;
        profile.command_patterns.total_commands = 50;

        assert!(AnalysisCoordinator::is_new_user(&profile));

        profile.experience_level = ExperienceLevel::Advanced;
        assert!(!AnalysisCoordinator::is_new_user(&profile));
    }
}

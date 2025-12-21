//! User preferences detection and management module
//!
//! This module provides functionality for detecting and managing user preferences
//! from project context and shell profiles. It helps Caro generate commands that
//! match the user's environment and workflows.
//!
//! # Overview
//!
//! The preferences system has three main components:
//!
//! 1. **Project Detection** - Scans the current directory to detect package managers,
//!    build tools, and programming languages used in the project.
//!
//! 2. **Shell Profile Analysis** - Parses user shell configuration files to extract
//!    aliases, exports, and PATH modifications.
//!
//! 3. **Preference Caching** - Caches detected preferences per-directory with TTL
//!    to avoid repeated analysis.
//!
//! # Example
//!
//! ```no_run
//! use caro::preferences::{UserPreferences, ProjectContext, ShellProfile};
//! use caro::models::ShellType;
//! use std::path::Path;
//!
//! async fn detect_preferences() -> anyhow::Result<()> {
//!     let cwd = std::env::current_dir()?;
//!     let preferences = UserPreferences::detect(&cwd, ShellType::Zsh).await?;
//!
//!     // Check if project uses yarn
//!     if let Some(pm) = &preferences.project.package_manager {
//!         println!("Package manager: {:?}", pm);
//!     }
//!
//!     // Check for git aliases
//!     if let Some(alias) = preferences.shell.get_alias("gst") {
//!         println!("gst alias: {}", alias);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod cache;
mod project;
mod shell;
mod translation;

pub use cache::{CacheEntry, PreferenceCache};
pub use project::{BuildTool, CloudContext, InfraTool, Language, PackageManager, ProjectContext};
pub use shell::ShellProfile;
pub use translation::{CommandTranslator, TranslatedCommand};

use crate::models::ShellType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during preference detection
#[derive(Error, Debug)]
pub enum PreferenceError {
    /// Project detection failed
    #[error("Failed to detect project context: {0}")]
    ProjectDetectionFailed(String),

    /// Shell profile parsing failed
    #[error("Failed to parse shell profile: {0}")]
    ShellParseError(String),

    /// Cache read/write failure
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Path doesn't exist or isn't accessible
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// User preferences aggregated from project and shell analysis
///
/// This is the main entry point for preference detection. It combines
/// project context (package managers, build tools) with shell profile
/// data (aliases, exports) to provide a complete picture of the user's
/// environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Project-specific context (package manager, tools)
    pub project: ProjectContext,

    /// Shell profile data (aliases, exports)
    pub shell: ShellProfile,

    /// When these preferences were detected
    pub detected_at: DateTime<Utc>,

    /// Cache key (typically absolute path of project root)
    pub cache_key: String,
}

impl UserPreferences {
    /// Detect preferences for the given directory and shell type
    ///
    /// This is the main entry point for preference detection. It will:
    /// 1. Check cache for existing preferences
    /// 2. If not cached or expired, scan the directory for project signals
    /// 3. Parse shell profile files for aliases and exports
    /// 4. Cache the results for future use
    ///
    /// # Arguments
    ///
    /// * `cwd` - Current working directory to scan for project context
    /// * `shell` - User's shell type for profile parsing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::preferences::UserPreferences;
    /// use caro::models::ShellType;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let cwd = std::env::current_dir()?;
    ///     let prefs = UserPreferences::detect(&cwd, ShellType::Zsh).await?;
    ///     println!("Detected: {:?}", prefs.project.package_manager);
    ///     Ok(())
    /// }
    /// ```
    pub async fn detect(cwd: &Path, shell: ShellType) -> Result<Self, PreferenceError> {
        // Validate path exists
        if !cwd.exists() {
            return Err(PreferenceError::InvalidPath(cwd.to_path_buf()));
        }

        let cache_key = cwd
            .canonicalize()
            .unwrap_or_else(|_| cwd.to_path_buf())
            .to_string_lossy()
            .to_string();

        debug!("Detecting preferences for: {}", cache_key);

        // Try to load from cache first
        if let Some(cached) = PreferenceCache::load(&cache_key).await {
            if cached.is_valid() {
                debug!("Using cached preferences");
                return Ok(cached.preferences);
            }
        }

        // Detect project context
        let project = ProjectContext::detect(cwd).map_err(|e| {
            warn!("Project detection error: {}", e);
            PreferenceError::ProjectDetectionFailed(e.to_string())
        })?;

        // Parse shell profile
        let shell_profile = ShellProfile::parse(shell).map_err(|e| {
            warn!("Shell parse error: {}", e);
            PreferenceError::ShellParseError(e.to_string())
        })?;

        let preferences = Self {
            project,
            shell: shell_profile,
            detected_at: Utc::now(),
            cache_key: cache_key.clone(),
        };

        // Cache the result
        if let Err(e) = PreferenceCache::save(&cache_key, &preferences).await {
            warn!("Failed to cache preferences: {}", e);
            // Don't fail the whole operation for cache errors
        }

        info!(
            "Detected preferences: package_manager={:?}, aliases={}",
            preferences.project.package_manager,
            preferences.shell.aliases.len()
        );

        Ok(preferences)
    }

    /// Convert preferences to a string for inclusion in prompts
    ///
    /// This generates a concise representation of user preferences
    /// that can be included in the system prompt to help the model
    /// generate more appropriate commands.
    pub fn to_prompt_context(&self) -> String {
        let mut context = String::new();

        // Package manager preference
        if let Some(pm) = &self.project.package_manager {
            context.push_str(&format!(
                "PACKAGE MANAGER: {} (use '{}' command, NOT alternatives)\n",
                pm.name(),
                pm.command()
            ));
        }

        // Build tool preference
        if let Some(bt) = &self.project.build_tool {
            context.push_str(&format!("BUILD TOOL: {} ('{}')\n", bt.name(), bt.command()));
        }

        // Detected languages
        if !self.project.languages.is_empty() {
            let langs: Vec<&str> = self.project.languages.iter().map(|l| l.name()).collect();
            context.push_str(&format!("LANGUAGES: {}\n", langs.join(", ")));
        }

        // Key aliases (only show commonly useful ones)
        let useful_aliases = self.get_useful_aliases();
        if !useful_aliases.is_empty() {
            context.push_str("USER ALIASES (prefer these shortcuts):\n");
            for (alias, command) in useful_aliases.iter().take(10) {
                context.push_str(&format!("  {} = '{}'\n", alias, command));
            }
        }

        context
    }

    /// Get aliases that are commonly useful for command generation
    fn get_useful_aliases(&self) -> Vec<(&String, &String)> {
        // Prioritize git, npm/yarn, and common utility aliases
        let priority_prefixes = ["g", "git", "npm", "yarn", "docker", "k", "ls", "ll"];

        let mut aliases: Vec<_> = self
            .shell
            .aliases
            .iter()
            .filter(|(name, _)| {
                priority_prefixes
                    .iter()
                    .any(|p| name.starts_with(p) || name.len() <= 3)
            })
            .collect();

        aliases.sort_by_key(|(name, _)| name.len());
        aliases
    }

    /// Check if a command complies with user preferences
    ///
    /// Returns a compliance assessment with confidence score and
    /// suggested translation if the command doesn't match preferences.
    pub fn check_compliance(&self, command: &str) -> PreferenceCompliance {
        let mut checks = Vec::new();
        let mut total_score = 0.0;
        let mut check_count = 0;

        // Check package manager compliance
        if let Some(expected_pm) = &self.project.package_manager {
            let (compliant, reason) = self.check_package_manager_compliance(command, expected_pm);
            checks.push(ComplianceCheck {
                aspect: "package_manager".to_string(),
                compliant,
                reason,
            });
            total_score += if compliant { 1.0 } else { 0.0 };
            check_count += 1;
        }

        // Check for available aliases
        let alias_check = self.check_alias_availability(command);
        if let Some(check) = alias_check {
            checks.push(check);
            total_score += 0.8; // Partial credit - aliases are nice but not required
            check_count += 1;
        }

        let confidence = if check_count > 0 {
            total_score / check_count as f64
        } else {
            1.0 // No checks applicable = assume compliant
        };

        // Generate suggested translation if non-compliant
        let suggested_command = if confidence < 0.8 {
            Some(
                CommandTranslator::translate(command, self)
                    .translated
                    .clone(),
            )
        } else {
            None
        };

        PreferenceCompliance {
            confidence,
            checks,
            suggested_command,
        }
    }

    /// Check if command uses the correct package manager
    fn check_package_manager_compliance(
        &self,
        command: &str,
        expected: &PackageManager,
    ) -> (bool, String) {
        let wrong_managers = expected.alternatives();

        for wrong in wrong_managers {
            if command.contains(&format!("{} ", wrong))
                || command.starts_with(&format!("{} ", wrong))
                || command.contains(&format!("npx ")) && *expected != PackageManager::Npm
            {
                return (
                    false,
                    format!(
                        "Command uses '{}' but project uses '{}'",
                        wrong,
                        expected.command()
                    ),
                );
            }
        }

        (true, "Package manager matches project configuration".to_string())
    }

    /// Check if command could use an available alias
    fn check_alias_availability(&self, command: &str) -> Option<ComplianceCheck> {
        if let Some((alias, _)) = self.shell.has_shorter_alias(command) {
            Some(ComplianceCheck {
                aspect: "alias_available".to_string(),
                compliant: false,
                reason: format!("User has shorter alias '{}' for this command", alias),
            })
        } else {
            None
        }
    }
}

/// Confidence assessment for preference compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceCompliance {
    /// Overall confidence (0.0-1.0) that command matches preferences
    pub confidence: f64,

    /// Individual compliance checks performed
    pub checks: Vec<ComplianceCheck>,

    /// Suggested command translation if non-compliant
    pub suggested_command: Option<String>,
}

impl PreferenceCompliance {
    /// Check if the command needs refinement
    pub fn needs_refinement(&self, threshold: f64) -> bool {
        self.confidence < threshold
    }

    /// Get the first non-compliant check
    pub fn first_issue(&self) -> Option<&ComplianceCheck> {
        self.checks.iter().find(|c| !c.compliant)
    }
}

/// Individual compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    /// What aspect was checked
    pub aspect: String,

    /// Whether the command complies
    pub compliant: bool,

    /// Explanation of the check result
    pub reason: String,
}

/// Raw preference data for model fallback
///
/// When static rules don't provide sufficient guidance, this struct
/// contains raw signals that can be passed to the model for inference.
/// The model can use this context to make preference-aware decisions
/// even for scenarios we haven't written explicit rules for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPreferenceData {
    /// Files detected in the project that might indicate preferences
    pub detected_files: Vec<String>,

    /// Raw signal files (config files we found but don't parse)
    pub raw_signal_files: Vec<String>,

    /// User's shell aliases (full list for model context)
    pub aliases: Vec<(String, String)>,

    /// Non-sensitive environment variables that might indicate preferences
    pub environment_hints: Vec<String>,

    /// Infrastructure tools detected
    pub infra_tools: Vec<String>,

    /// Cloud provider context
    pub cloud_hints: Vec<String>,

    /// Detected programming languages
    pub languages: Vec<String>,

    /// Any package manager hints (lockfiles, configs)
    pub package_hints: Vec<String>,
}

impl RawPreferenceData {
    /// Create from UserPreferences
    pub fn from_preferences(prefs: &UserPreferences) -> Self {
        // Collect cloud hints
        let mut cloud_hints = Vec::new();
        if let Some(ctx) = &prefs.project.cloud_context {
            if ctx.aws_configured {
                cloud_hints.push(format!(
                    "AWS configured{}",
                    ctx.aws_profile
                        .as_ref()
                        .map(|p| format!(" (profile: {})", p))
                        .unwrap_or_default()
                ));
            }
            if ctx.gcp_configured {
                cloud_hints.push(format!(
                    "GCP configured{}",
                    ctx.gcp_project
                        .as_ref()
                        .map(|p| format!(" (project: {})", p))
                        .unwrap_or_default()
                ));
            }
            if ctx.azure_configured {
                cloud_hints.push(format!(
                    "Azure configured{}",
                    ctx.azure_subscription
                        .as_ref()
                        .map(|s| format!(" (subscription: {})", s))
                        .unwrap_or_default()
                ));
            }
            if let Some(ref k8s_ctx) = ctx.kubectl_context {
                cloud_hints.push(format!("kubectl context: {}", k8s_ctx));
            }
        }

        // Collect environment hints (non-sensitive)
        let env_hints: Vec<String> = std::env::vars()
            .filter(|(k, _)| {
                // Include hints about tools and preferences, exclude secrets
                let key_upper = k.to_uppercase();
                (key_upper.contains("EDITOR")
                    || key_upper.contains("SHELL")
                    || key_upper.contains("TERM")
                    || key_upper.contains("LANG")
                    || key_upper.contains("LC_")
                    || key_upper.starts_with("NPM_")
                    || key_upper.starts_with("YARN_")
                    || key_upper.starts_with("NODE_")
                    || key_upper.starts_with("CARGO_")
                    || key_upper.starts_with("RUSTUP_")
                    || key_upper.starts_with("GO")
                    || key_upper.starts_with("PYTHON")
                    || key_upper.starts_with("VIRTUAL_ENV")
                    || key_upper.starts_with("CONDA_")
                    || key_upper.starts_with("DOCKER_")
                    || key_upper.starts_with("KUBE")
                    || key_upper.starts_with("HELM_"))
                    && !key_upper.contains("TOKEN")
                    && !key_upper.contains("SECRET")
                    && !key_upper.contains("KEY")
                    && !key_upper.contains("PASSWORD")
                    && !key_upper.contains("CREDENTIAL")
            })
            .map(|(k, v)| format!("{}={}", k, v))
            .take(30) // Limit to avoid token bloat
            .collect();

        // Package hints from detected files
        let package_hints: Vec<String> = prefs
            .project
            .detected_files
            .iter()
            .filter(|f| {
                f.contains("lock")
                    || f.contains("package")
                    || f.ends_with(".toml")
                    || f.ends_with("mod")
                    || f.contains("requirements")
                    || f.contains("Gemfile")
                    || f.contains("Pipfile")
            })
            .cloned()
            .collect();

        Self {
            detected_files: prefs.project.detected_files.clone(),
            raw_signal_files: prefs.project.raw_signals.clone(),
            aliases: prefs
                .shell
                .aliases
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            environment_hints: env_hints,
            infra_tools: prefs
                .project
                .infra_tools
                .iter()
                .map(|t| t.name().to_string())
                .collect(),
            cloud_hints,
            languages: prefs
                .project
                .languages
                .iter()
                .map(|l| l.name().to_string())
                .collect(),
            package_hints,
        }
    }

    /// Generate a model-friendly context string
    ///
    /// This creates a formatted string that can be included in a prompt
    /// to help the model understand the user's environment when static
    /// rules don't apply.
    pub fn to_model_context(&self) -> String {
        let mut context = String::new();
        context.push_str("=== RAW ENVIRONMENT CONTEXT (for preference inference) ===\n\n");

        if !self.languages.is_empty() {
            context.push_str(&format!("Languages: {}\n", self.languages.join(", ")));
        }

        if !self.package_hints.is_empty() {
            context.push_str(&format!(
                "Package/Lock files: {}\n",
                self.package_hints.join(", ")
            ));
        }

        if !self.infra_tools.is_empty() {
            context.push_str(&format!("Infrastructure: {}\n", self.infra_tools.join(", ")));
        }

        if !self.cloud_hints.is_empty() {
            context.push_str(&format!("Cloud: {}\n", self.cloud_hints.join(", ")));
        }

        if !self.aliases.is_empty() {
            context.push_str("\nUser shell aliases:\n");
            for (alias, expansion) in self.aliases.iter().take(20) {
                context.push_str(&format!("  {} = {}\n", alias, expansion));
            }
        }

        if !self.environment_hints.is_empty() {
            context.push_str("\nEnvironment hints:\n");
            for hint in self.environment_hints.iter().take(15) {
                context.push_str(&format!("  {}\n", hint));
            }
        }

        if !self.raw_signal_files.is_empty() {
            context.push_str("\nOther config files found:\n");
            for file in self.raw_signal_files.iter().take(10) {
                context.push_str(&format!("  {}\n", file));
            }
        }

        context.push_str("\n=== END CONTEXT ===\n");
        context
    }
}

impl UserPreferences {
    /// Get raw preference data for model fallback
    ///
    /// When static rules don't provide guidance for a particular scenario,
    /// this raw data can be passed to the model to help it infer user preferences.
    pub fn to_raw_data(&self) -> RawPreferenceData {
        RawPreferenceData::from_preferences(self)
    }

    /// Get extended prompt context including infrastructure and cloud
    ///
    /// This is a more comprehensive version of `to_prompt_context` that
    /// includes DevOps/SRE-relevant information.
    pub fn to_extended_prompt_context(&self) -> String {
        let mut context = self.to_prompt_context();

        // Add infrastructure tools
        if !self.project.infra_tools.is_empty() {
            let tools: Vec<&str> = self.project.infra_tools.iter().map(|t| t.name()).collect();
            context.push_str(&format!("INFRASTRUCTURE: {}\n", tools.join(", ")));
        }

        // Add cloud context
        if let Some(cloud) = &self.project.cloud_context {
            context.push_str(&cloud.to_prompt_context());
            context.push('\n');
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_preferences_prompt_context() {
        let prefs = UserPreferences {
            project: ProjectContext {
                package_manager: Some(PackageManager::Yarn),
                build_tool: Some(BuildTool::Make),
                languages: vec![Language::Rust, Language::TypeScript],
                infra_tools: vec![],
                cloud_context: None,
                root_path: PathBuf::from("/test/project"),
                detected_files: vec!["yarn.lock".to_string(), "Cargo.toml".to_string()],
                raw_signals: vec![],
            },
            shell: ShellProfile {
                aliases: [
                    ("gst".to_string(), "git status".to_string()),
                    ("gco".to_string(), "git checkout".to_string()),
                ]
                .into_iter()
                .collect(),
                exports: HashMap::new(),
                path_additions: vec![],
                shell_type: ShellType::Zsh,
                profile_files: vec![],
            },
            detected_at: Utc::now(),
            cache_key: "/test/project".to_string(),
        };

        let context = prefs.to_prompt_context();
        assert!(context.contains("yarn"));
        assert!(context.contains("Rust"));
        assert!(context.contains("gst"));
    }

    #[test]
    fn test_compliance_check_wrong_package_manager() {
        let prefs = UserPreferences {
            project: ProjectContext {
                package_manager: Some(PackageManager::Yarn),
                build_tool: None,
                languages: vec![],
                infra_tools: vec![],
                cloud_context: None,
                root_path: PathBuf::from("/test"),
                detected_files: vec!["yarn.lock".to_string()],
                raw_signals: vec![],
            },
            shell: ShellProfile::empty(ShellType::Bash),
            detected_at: Utc::now(),
            cache_key: "/test".to_string(),
        };

        // npm command should fail compliance when yarn is preferred
        let compliance = prefs.check_compliance("npm install lodash");
        assert!(compliance.confidence < 1.0);
        assert!(!compliance.checks[0].compliant);
        assert!(compliance.suggested_command.is_some());
    }

    #[test]
    fn test_compliance_check_correct_package_manager() {
        let prefs = UserPreferences {
            project: ProjectContext {
                package_manager: Some(PackageManager::Yarn),
                build_tool: None,
                languages: vec![],
                infra_tools: vec![],
                cloud_context: None,
                root_path: PathBuf::from("/test"),
                detected_files: vec!["yarn.lock".to_string()],
                raw_signals: vec![],
            },
            shell: ShellProfile::empty(ShellType::Bash),
            detected_at: Utc::now(),
            cache_key: "/test".to_string(),
        };

        // yarn command should pass compliance
        let compliance = prefs.check_compliance("yarn add lodash");
        assert_eq!(compliance.confidence, 1.0);
        assert!(compliance.checks[0].compliant);
        assert!(compliance.suggested_command.is_none());
    }

    #[test]
    fn test_compliance_needs_refinement() {
        let compliance = PreferenceCompliance {
            confidence: 0.5,
            checks: vec![],
            suggested_command: Some("yarn install".to_string()),
        };

        assert!(compliance.needs_refinement(0.8));
        assert!(!compliance.needs_refinement(0.3));
    }

    #[test]
    fn test_raw_preference_data() {
        let prefs = UserPreferences {
            project: ProjectContext {
                package_manager: Some(PackageManager::Yarn),
                build_tool: Some(BuildTool::Make),
                languages: vec![Language::Rust, Language::TypeScript],
                infra_tools: vec![InfraTool::Docker, InfraTool::Kubernetes],
                cloud_context: Some(CloudContext {
                    aws_configured: true,
                    aws_profile: Some("prod".to_string()),
                    gcp_configured: false,
                    gcp_project: None,
                    azure_configured: false,
                    azure_subscription: None,
                    kubectl_context: Some("minikube".to_string()),
                }),
                root_path: PathBuf::from("/test/project"),
                detected_files: vec![
                    "yarn.lock".to_string(),
                    "Cargo.toml".to_string(),
                    "Dockerfile".to_string(),
                ],
                raw_signals: vec!["serverless.yml".to_string()],
            },
            shell: ShellProfile {
                aliases: [
                    ("gst".to_string(), "git status".to_string()),
                    ("k".to_string(), "kubectl".to_string()),
                ]
                .into_iter()
                .collect(),
                exports: HashMap::new(),
                path_additions: vec![],
                shell_type: ShellType::Zsh,
                profile_files: vec![],
            },
            detected_at: Utc::now(),
            cache_key: "/test/project".to_string(),
        };

        let raw_data = prefs.to_raw_data();

        // Check languages are captured
        assert!(raw_data.languages.contains(&"Rust".to_string()));
        assert!(raw_data.languages.contains(&"TypeScript".to_string()));

        // Check infra tools are captured
        assert!(raw_data.infra_tools.contains(&"Docker".to_string()));
        assert!(raw_data.infra_tools.contains(&"Kubernetes".to_string()));

        // Check cloud hints
        assert!(!raw_data.cloud_hints.is_empty());
        assert!(raw_data.cloud_hints.iter().any(|h| h.contains("AWS")));
        assert!(raw_data.cloud_hints.iter().any(|h| h.contains("minikube")));

        // Check aliases are captured
        assert!(raw_data.aliases.iter().any(|(a, _)| a == "gst"));
        assert!(raw_data.aliases.iter().any(|(a, _)| a == "k"));

        // Check raw signals are captured
        assert!(raw_data.raw_signal_files.contains(&"serverless.yml".to_string()));

        // Test model context generation
        let context = raw_data.to_model_context();
        assert!(context.contains("Languages:"));
        assert!(context.contains("Rust"));
        assert!(context.contains("Infrastructure:"));
        assert!(context.contains("Docker"));
        assert!(context.contains("Cloud:"));
        assert!(context.contains("AWS"));
    }

    #[test]
    fn test_extended_prompt_context() {
        let prefs = UserPreferences {
            project: ProjectContext {
                package_manager: Some(PackageManager::Npm),
                build_tool: None,
                languages: vec![Language::TypeScript],
                infra_tools: vec![InfraTool::Terraform, InfraTool::Ansible],
                cloud_context: Some(CloudContext {
                    aws_configured: false,
                    aws_profile: None,
                    gcp_configured: true,
                    gcp_project: Some("my-project".to_string()),
                    azure_configured: false,
                    azure_subscription: None,
                    kubectl_context: None,
                }),
                root_path: PathBuf::from("/test"),
                detected_files: vec![],
                raw_signals: vec![],
            },
            shell: ShellProfile::empty(ShellType::Bash),
            detected_at: Utc::now(),
            cache_key: "/test".to_string(),
        };

        let context = prefs.to_extended_prompt_context();

        // Should include infra tools
        assert!(context.contains("INFRASTRUCTURE:"));
        assert!(context.contains("Terraform"));
        assert!(context.contains("Ansible"));

        // Should include cloud context
        assert!(context.contains("GCP"));
        assert!(context.contains("my-project"));
    }
}

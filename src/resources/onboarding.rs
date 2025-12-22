//! Interactive Onboarding Flow
//!
//! Guides users through initial setup, understanding their preferences,
//! and configuring the optimal model for their needs.

use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::time::Duration;
use tracing::info;

use super::assessment::{ResourceAssessment, SystemResources};
use super::models::{ModelTier, ModelTierConfig};
use super::recommendation::{ModelRecommendation, RecommendationEngine};
use super::ResourceError;

/// User preferences collected during onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Primary use case
    pub use_case: UseCase,

    /// Speed vs quality preference (0.0 = fastest, 1.0 = best quality)
    pub quality_preference: f32,

    /// Whether user wants thinking/reasoning features
    pub wants_thinking: bool,

    /// Whether user wants tool calling features
    pub wants_tool_calling: bool,

    /// Preferred local vs remote inference
    pub prefers_local: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            use_case: UseCase::General,
            quality_preference: 0.5,
            wants_thinking: false,
            wants_tool_calling: false,
            prefers_local: true, // Always prefer local
        }
    }
}

/// Primary use case for the tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseCase {
    /// Quick command lookups
    QuickCommands,
    /// General development tasks
    General,
    /// Complex system administration
    SysAdmin,
    /// Learning and exploration
    Learning,
}

impl std::fmt::Display for UseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UseCase::QuickCommands => write!(f, "Quick Commands"),
            UseCase::General => write!(f, "General Development"),
            UseCase::SysAdmin => write!(f, "System Administration"),
            UseCase::Learning => write!(f, "Learning & Exploration"),
        }
    }
}

/// Result of the onboarding process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResult {
    /// Detected system resources
    pub resources: SystemResources,

    /// User preferences collected
    pub preferences: UserPreferences,

    /// Selected model tier
    pub selected_tier: ModelTier,

    /// Model configuration
    pub model_config: ModelTierConfig,

    /// Final recommendation used
    pub recommendation: ModelRecommendation,

    /// Whether model needs to be downloaded
    pub needs_download: bool,

    /// Estimated download size in MB
    pub download_size_mb: u64,
}

/// Interactive onboarding flow
pub struct OnboardingFlow {
    resources: SystemResources,
    engine: RecommendationEngine,
    interactive: bool,
}

impl OnboardingFlow {
    /// Create a new onboarding flow
    pub fn new() -> Result<Self, ResourceError> {
        let resources = ResourceAssessment::assess()?;
        let engine = RecommendationEngine::new(resources.clone());

        Ok(Self {
            resources,
            engine,
            interactive: std::io::stdin().is_terminal(),
        })
    }

    /// Create with pre-assessed resources (for testing)
    pub fn with_resources(resources: SystemResources) -> Self {
        let engine = RecommendationEngine::new(resources.clone());
        Self {
            resources,
            engine,
            interactive: std::io::stdin().is_terminal(),
        }
    }

    /// Run the full onboarding flow
    pub async fn run(&self) -> Result<OnboardingResult, ResourceError> {
        if !self.interactive {
            return self.run_non_interactive().await;
        }

        self.print_welcome();
        self.print_resource_summary();

        let preferences = self.collect_preferences()?;
        let selected_tier = self.select_model_tier(&preferences)?;
        let recommendation = ModelRecommendation::new(selected_tier, &self.resources);

        self.print_selection_summary(&recommendation);

        if !self.confirm_selection(&recommendation)? {
            return Err(ResourceError::UserCancelled);
        }

        let model_config = self.create_config(selected_tier, &preferences);
        let needs_download = self.check_needs_download(&model_config);
        let download_size_mb = model_config.model.size_mb;

        if needs_download {
            self.print_download_info(&model_config);
        }

        Ok(OnboardingResult {
            resources: self.resources.clone(),
            preferences,
            selected_tier,
            model_config,
            recommendation,
            needs_download,
            download_size_mb,
        })
    }

    /// Run non-interactive mode (auto-select best option)
    async fn run_non_interactive(&self) -> Result<OnboardingResult, ResourceError> {
        info!("Running in non-interactive mode, auto-selecting optimal model");

        let recommendation = self.engine.recommend();
        let preferences = UserPreferences::default();
        let model_config = self.create_config(recommendation.tier, &preferences);
        let needs_download = self.check_needs_download(&model_config);

        let download_size_mb = recommendation.model.size_mb;

        Ok(OnboardingResult {
            resources: self.resources.clone(),
            preferences,
            selected_tier: recommendation.tier,
            model_config,
            recommendation,
            needs_download,
            download_size_mb,
        })
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!();
        println!("{}", "Welcome to Caro!".bold().cyan());
        println!("{}", "Natural language to shell commands, powered by local AI.".dimmed());
        println!();
        println!("Let me assess your system and help you choose the best model.");
        println!();
    }

    /// Print resource summary
    fn print_resource_summary(&self) {
        println!("{}", "System Resources:".bold());
        println!("  {} {} cores", "CPU:".dimmed(), self.resources.cpu_cores);
        println!("  {} {} GB", "RAM:".dimmed(), self.resources.ram_gb());

        if let Some(gpu) = &self.resources.gpu {
            println!("  {} {}", "GPU:".dimmed(), gpu.name);
            if gpu.metal_available {
                println!("  {} Metal acceleration available", "   ".dimmed());
            }
            if gpu.cuda_available {
                println!("  {} CUDA acceleration available", "   ".dimmed());
            }
        } else {
            println!("  {} Not detected (CPU mode)", "GPU:".dimmed());
        }

        if self.resources.is_apple_silicon {
            println!(
                "  {} {} GB effective",
                "Unified Memory:".dimmed(),
                self.resources.effective_gpu_memory_gb()
            );
        }

        println!(
            "  {} {} GB available",
            "Storage:".dimmed(),
            self.resources.available_storage_gb()
        );

        println!();
        println!(
            "  Machine Class: {}",
            self.engine.machine_class().bold()
        );
        println!();
    }

    /// Collect user preferences through interactive questions
    fn collect_preferences(&self) -> Result<UserPreferences, ResourceError> {
        println!("{}", "Let me understand how you'll use Caro...".bold());
        println!();

        // Ask about primary use case
        let use_cases = vec![
            "Quick Commands - Fast lookups for common tasks",
            "General Development - Everyday coding and scripting",
            "System Administration - Complex server and DevOps tasks",
            "Learning & Exploration - Understanding new commands and concepts",
        ];

        let use_case_idx = Select::new()
            .with_prompt("What will you primarily use Caro for?")
            .items(&use_cases)
            .default(1)
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?;

        let use_case = match use_case_idx {
            0 => UseCase::QuickCommands,
            1 => UseCase::General,
            2 => UseCase::SysAdmin,
            3 => UseCase::Learning,
            _ => UseCase::General,
        };

        println!();

        // Ask about speed vs quality preference
        let quality_options = vec![
            "Fast - Quick responses, simpler commands",
            "Balanced - Good quality with reasonable speed",
            "Best Quality - Take time for complex reasoning",
        ];

        let quality_idx = Select::new()
            .with_prompt("Speed vs quality preference?")
            .items(&quality_options)
            .default(1)
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?;

        let quality_preference = match quality_idx {
            0 => 0.0,
            1 => 0.5,
            2 => 1.0,
            _ => 0.5,
        };

        println!();

        // Ask about thinking/reasoning for medium+ machines
        let wants_thinking = if self.resources.is_medium_machine() || self.resources.is_heavy_machine() {
            Confirm::new()
                .with_prompt("Enable thinking mode for complex requests? (slower but smarter)")
                .default(quality_preference > 0.3)
                .interact()
                .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?
        } else {
            false
        };

        // Ask about tool calling for heavy machines
        let wants_tool_calling = if self.resources.is_heavy_machine() {
            Confirm::new()
                .with_prompt("Enable tool calling? (allows more sophisticated command generation)")
                .default(true)
                .interact()
                .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?
        } else {
            false
        };

        println!();

        Ok(UserPreferences {
            use_case,
            quality_preference,
            wants_thinking,
            wants_tool_calling,
            prefers_local: true,
        })
    }

    /// Select model tier based on preferences and resources
    fn select_model_tier(&self, _preferences: &UserPreferences) -> Result<ModelTier, ResourceError> {
        // Get the recommendation first
        let recommendation = self.engine.recommend();

        println!("{}", "Available Models:".bold());
        println!();

        // Get all compatible models
        let compatible = self.engine.compatible_recommendations();

        let mut items = Vec::new();
        let mut tier_map = Vec::new();

        for rec in &compatible {
            let model = &rec.model;
            let recommended = rec.tier == recommendation.tier;
            let rec_mark = if recommended { " [RECOMMENDED]" } else { "" };

            let features = if model.supports_thinking && model.supports_tool_calling {
                "thinking + tools"
            } else if model.supports_thinking {
                "thinking"
            } else {
                "basic"
            };

            items.push(format!(
                "{} ({:.1}B) - {:.1}GB, ~{:.1}s latency, {}{}",
                model.name,
                model.parameters_b,
                model.size_gb(),
                model.typical_latency_s,
                features,
                rec_mark.green()
            ));
            tier_map.push(rec.tier);
        }

        // Add custom option
        items.push("Custom - Specify a Hugging Face model".to_string());

        println!();

        let default_idx = tier_map
            .iter()
            .position(|t| *t == recommendation.tier)
            .unwrap_or(0);

        let selection = Select::new()
            .with_prompt("Select a model")
            .items(&items)
            .default(default_idx)
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?;

        if selection >= tier_map.len() {
            // Custom model selected
            self.handle_custom_model()
        } else {
            Ok(tier_map[selection])
        }
    }

    /// Handle custom model selection
    fn handle_custom_model(&self) -> Result<ModelTier, ResourceError> {
        println!();
        println!("{}", "Custom Model Configuration".bold());
        println!("Enter a Hugging Face model that provides GGUF files.");
        println!();

        let _hf_repo: String = Input::new()
            .with_prompt("Hugging Face repository (e.g., 'Qwen/Qwen3-8B-GGUF')")
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?;

        let _gguf_file: String = Input::new()
            .with_prompt("GGUF filename (e.g., 'qwen3-8b-q4_k_m.gguf')")
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))?;

        // For now, return Custom tier - the actual custom config is stored separately
        Ok(ModelTier::Custom)
    }

    /// Print selection summary
    fn print_selection_summary(&self, recommendation: &ModelRecommendation) {
        println!();
        println!("{}", "Selection Summary:".bold());
        println!("  Model: {} ({})", recommendation.model.name.cyan(), recommendation.tier);
        println!("  Size: {:.1} GB download", recommendation.model.size_gb());
        println!("  Parameters: {:.1}B", recommendation.model.parameters_b);

        let features = vec![
            if recommendation.model.supports_thinking {
                Some("Thinking Mode")
            } else {
                None
            },
            if recommendation.model.supports_tool_calling {
                Some("Tool Calling")
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if !features.is_empty() {
            println!("  Features: {}", features.join(", "));
        }

        println!();
        println!("{}", "Resource Impact:".bold());
        for line in recommendation.resource_summary.lines() {
            println!("  {}", line);
        }

        if !recommendation.warnings.is_empty() {
            println!();
            println!("{}", "Warnings:".yellow().bold());
            for warning in &recommendation.warnings {
                println!("  {} {}", "!".yellow(), warning);
            }
        }

        println!();
    }

    /// Confirm the selection
    fn confirm_selection(&self, recommendation: &ModelRecommendation) -> Result<bool, ResourceError> {
        let prompt = format!(
            "Proceed with {} ({:.1} GB download)?",
            recommendation.model.name,
            recommendation.model.size_gb()
        );

        Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()
            .map_err(|e| ResourceError::ConfigurationError(e.to_string()))
    }

    /// Create model configuration based on tier and preferences
    fn create_config(&self, tier: ModelTier, preferences: &UserPreferences) -> ModelTierConfig {
        let mut config = ModelTierConfig::for_tier(tier);

        // Apply preferences
        config.enable_thinking = preferences.wants_thinking && config.model.supports_thinking;
        config.enable_tool_calling =
            preferences.wants_tool_calling && config.model.supports_tool_calling;
        config.fast_mode = preferences.quality_preference < 0.3;

        config
    }

    /// Check if model needs to be downloaded
    fn check_needs_download(&self, _config: &ModelTierConfig) -> bool {
        // TODO: Check if model file exists in cache
        // For now, assume it needs download
        true
    }

    /// Print download information
    fn print_download_info(&self, config: &ModelTierConfig) {
        println!("{}", "Download Required:".bold());
        println!("  Model: {}", config.model.name);
        println!("  Source: Hugging Face Hub");
        println!("  Size: {:.1} GB", config.model.size_gb());
        println!("  Location: {}", self.resources.cache_dir.display());
        println!();
    }

    /// Download the model with progress display
    pub async fn download_model(&self, config: &ModelTierConfig) -> Result<std::path::PathBuf, ResourceError> {
        use hf_hub::api::tokio::Api;

        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        pb.set_message(format!("Downloading {}...", config.model.name));

        // Initialize Hugging Face API
        let api = Api::new().map_err(|e| {
            ResourceError::DownloadError(format!("Failed to initialize HF API: {}", e))
        })?;

        let repo = api.model(config.model.hf_repo.clone());

        // Simulate progress (actual progress would need custom download implementation)
        for i in 0..50 {
            pb.set_position(i * 2);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Download the model file
        let model_path = repo.get(&config.model.gguf_file).await.map_err(|e| {
            ResourceError::DownloadError(format!("Failed to download model: {}", e))
        })?;

        pb.set_position(100);
        pb.finish_with_message("Download complete!");

        println!();
        println!(
            "{} Model saved to: {}",
            "Success!".green().bold(),
            model_path.display()
        );

        Ok(model_path)
    }

    /// Quick initialization (non-interactive)
    pub async fn quick_init(&self) -> Result<OnboardingResult, ResourceError> {
        self.run_non_interactive().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::assessment::GpuInfo;
    use crate::resources::assessment::GpuVendor;

    fn create_test_resources() -> SystemResources {
        SystemResources {
            total_ram_mb: 16384,
            available_ram_mb: 12288,
            cpu_cores: 8,
            cpu_brand: "Test CPU".to_string(),
            gpu: Some(GpuInfo {
                name: "Test GPU".to_string(),
                vendor: GpuVendor::Apple,
                vram_mb: 0,
                metal_available: true,
                cuda_available: false,
                compute_capability: None,
            }),
            available_storage_mb: 51200,
            total_storage_mb: 256000,
            cache_dir: std::path::PathBuf::from("/tmp/caro"),
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            is_apple_silicon: true,
        }
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(prefs.prefers_local);
        assert_eq!(prefs.quality_preference, 0.5);
    }

    #[tokio::test]
    async fn test_quick_init() {
        let resources = create_test_resources();
        let flow = OnboardingFlow::with_resources(resources);

        let result = flow.quick_init().await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.selected_tier.to_string().is_empty());
    }

    #[test]
    fn test_onboarding_flow_creation() {
        let resources = create_test_resources();
        let flow = OnboardingFlow::with_resources(resources);

        // Engine should be created successfully
        assert_eq!(flow.resources.cpu_cores, 8);
    }
}

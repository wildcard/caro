//! Model Manager Module
//!
//! Provides a unified interface for model management including:
//! - Speed testing and model recommendations
//! - Model downloading (foreground and background)
//! - Cache management
//! - CLI subcommands for model operations

use crate::config::ConfigManager;
use crate::model_catalog::{ModelCatalog, ModelInfo};
use crate::model_loader::ModelLoader;
use crate::model_recommendation::{
    get_cached_models, is_model_cached, ModelPreferences, ModelRecommendation, ModelRecommender,
};
use crate::models::ModelPreferencesConfig;
use crate::speed_test::{SpeedTestResult, SpeedTester};

use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Status of a background download
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    /// Download not started
    Pending,
    /// Download in progress with percentage complete
    Downloading { progress_percent: f64, speed_mbps: f64 },
    /// Download completed successfully
    Completed,
    /// Download failed with error message
    Failed(String),
    /// Download was cancelled
    Cancelled,
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Downloading { progress_percent, speed_mbps } => {
                write!(f, "Downloading: {:.1}% ({:.1} MB/s)", progress_percent, speed_mbps)
            }
            Self::Completed => write!(f, "Completed"),
            Self::Failed(msg) => write!(f, "Failed: {}", msg),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Background download task information
#[derive(Debug, Clone)]
pub struct BackgroundDownload {
    pub model_id: String,
    pub model_name: String,
    pub size_mb: u64,
    pub status: DownloadStatus,
}

/// Model manager for handling all model-related operations
pub struct ModelManager {
    cache_dir: PathBuf,
    config_manager: ConfigManager,
    background_downloads: Arc<Mutex<Vec<BackgroundDownload>>>,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new() -> Result<Self> {
        let cache_dir = ModelLoader::default_cache_dir()?;
        let config_manager = ConfigManager::new()?;

        Ok(Self {
            cache_dir,
            config_manager,
            background_downloads: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Run a network speed test
    pub async fn run_speed_test(&self) -> Result<SpeedTestResult> {
        println!("{}", "Running network speed test...".dimmed());
        println!("{}", "Downloading sample data from Hugging Face...".dimmed());

        let tester = SpeedTester::new()?;
        let result = tester.run_quick_test().await;

        if result.success {
            println!(
                "{} {:.2} MB/s ({})",
                "Network Speed:".bold(),
                result.speed_mbps,
                result.quality
            );
        } else {
            println!(
                "{} {}",
                "Speed test failed:".red(),
                result.error.as_deref().unwrap_or("Unknown error")
            );
        }

        Ok(result)
    }

    /// Get model recommendation based on speed test
    pub async fn recommend_models(&self) -> Result<ModelRecommendation> {
        let speed_result = self.run_speed_test().await?;

        // Load user preferences from config
        let config = self.config_manager.load()?;
        let prefs = self.convert_preferences(&config.model_preferences);
        let recommender = ModelRecommender::with_preferences(prefs);

        let recommendation = recommender.recommend(&speed_result);
        Ok(recommendation)
    }

    /// Convert config preferences to model preferences
    fn convert_preferences(&self, config_prefs: &ModelPreferencesConfig) -> ModelPreferences {
        ModelPreferences {
            prefer_mlx: config_prefs.prefer_mlx,
            prefer_small: config_prefs.prefer_small,
            prefer_quality: config_prefs.prefer_quality,
            prefer_code_models: config_prefs.prefer_code_models,
            max_instant_download_secs: config_prefs.max_instant_download_secs,
            max_background_download_secs: config_prefs.max_background_download_secs,
        }
    }

    /// Display model recommendation and prompt for action
    pub async fn recommend_and_prompt(&self) -> Result<()> {
        let recommendation = self.recommend_models().await?;

        println!();
        println!("{}", "=== Model Recommendation ===".bold().cyan());
        println!();

        // Check if instant model is already cached
        let instant_cached = is_model_cached(recommendation.instant_model, &self.cache_dir);
        let bg_cached = recommendation
            .background_model
            .map(|m| is_model_cached(m, &self.cache_dir))
            .unwrap_or(true);

        // Display instant model recommendation
        println!("{}", "Recommended for Instant Use:".bold());
        println!(
            "  {} {} ({} MB)",
            "→".green(),
            recommendation.instant_model.name.bright_cyan(),
            recommendation.instant_model.size_mb
        );
        println!("    {}", recommendation.instant_model.description);
        println!(
            "    Download time: ~{}",
            recommendation.instant_download_time
        );
        if instant_cached {
            println!("    {} Already cached!", "✓".green());
        }

        // Display background model recommendation if different
        if let Some(bg_model) = recommendation.background_model {
            println!();
            println!("{}", "Recommended for Better Quality (Background Download):".bold());
            println!(
                "  {} {} ({} MB)",
                "→".blue(),
                bg_model.name.bright_blue(),
                bg_model.size_mb
            );
            println!("    {}", bg_model.description);
            if let Some(ref time) = recommendation.background_download_time {
                println!("    Download time: ~{}", time);
            }
            if bg_cached {
                println!("    {} Already cached!", "✓".green());
            }
        }

        println!();
        println!("{}", recommendation.reasoning.dimmed());
        println!();

        // Prompt user for action
        if !instant_cached || (!bg_cached && recommendation.background_model.is_some()) {
            self.prompt_download_action(&recommendation, instant_cached, bg_cached)
                .await?;
        } else {
            println!(
                "{}",
                "All recommended models are already cached!".green().bold()
            );
        }

        Ok(())
    }

    /// Prompt user for download action
    async fn prompt_download_action(
        &self,
        recommendation: &ModelRecommendation,
        instant_cached: bool,
        bg_cached: bool,
    ) -> Result<()> {
        use dialoguer::{Confirm, Select};
        use std::io::IsTerminal;

        if !std::io::stdin().is_terminal() {
            println!(
                "{}",
                "Use 'caro model download <model-id>' to download models.".dimmed()
            );
            return Ok(());
        }

        let mut options = Vec::new();

        if !instant_cached {
            options.push(format!(
                "Download {} for instant use",
                recommendation.instant_model.name
            ));
        }

        if !bg_cached {
            if let Some(bg_model) = recommendation.background_model {
                options.push(format!(
                    "Download {} in background for better quality",
                    bg_model.name
                ));
            }
        }

        if !instant_cached && !bg_cached && recommendation.background_model.is_some() {
            options.push("Download both models".to_string());
        }

        options.push("Skip for now".to_string());

        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        let selected_option = &options[selection];

        if selected_option.contains("both") {
            // Download instant model first
            println!();
            self.download_model(recommendation.instant_model).await?;

            // Start background download
            if let Some(bg_model) = recommendation.background_model {
                println!();
                let should_background = Confirm::new()
                    .with_prompt(format!(
                        "Start background download for {}?",
                        bg_model.name
                    ))
                    .default(true)
                    .interact()?;

                if should_background {
                    self.start_background_download(bg_model).await?;
                }
            }
        } else if selected_option.contains("instant") {
            println!();
            self.download_model(recommendation.instant_model).await?;
        } else if selected_option.contains("background") {
            if let Some(bg_model) = recommendation.background_model {
                self.start_background_download(bg_model).await?;
            }
        } else {
            println!("{}", "Skipped. You can download models later with 'caro model download'.".dimmed());
        }

        Ok(())
    }

    /// Download a model (foreground, with progress)
    pub async fn download_model(&self, model: &ModelInfo) -> Result<()> {
        println!(
            "{} {} ({} MB)...",
            "Downloading".cyan(),
            model.name,
            model.size_mb
        );

        let loader = ModelLoader::with_model(model.id)?;
        let variant = ModelLoader::detect_platform();

        let path = loader.download_model_if_missing(variant).await?;

        println!(
            "{} Downloaded to: {}",
            "✓".green(),
            path.display()
        );

        Ok(())
    }

    /// Download a model by ID
    pub async fn download_model_by_id(&self, model_id: &str) -> Result<()> {
        let model = ModelCatalog::by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        self.download_model(model).await
    }

    /// Start a background download for a model
    pub async fn start_background_download(&self, model: &ModelInfo) -> Result<()> {
        println!(
            "{} Starting background download for {} ({} MB)...",
            "↓".blue(),
            model.name,
            model.size_mb
        );

        let bg_download = BackgroundDownload {
            model_id: model.id.to_string(),
            model_name: model.name.to_string(),
            size_mb: model.size_mb,
            status: DownloadStatus::Pending,
        };

        // Add to tracking list
        {
            let mut downloads = self.background_downloads.lock().await;
            downloads.push(bg_download);
        }

        // Clone necessary data for the spawned task
        let model_id = model.id.to_string();
        let downloads = Arc::clone(&self.background_downloads);

        // Spawn background task
        tokio::spawn(async move {
            // Update status to downloading
            {
                let mut dl = downloads.lock().await;
                if let Some(d) = dl.iter_mut().find(|d| d.model_id == model_id) {
                    d.status = DownloadStatus::Downloading {
                        progress_percent: 0.0,
                        speed_mbps: 0.0,
                    };
                }
            }

            // Perform download
            let result = async {
                let loader = ModelLoader::with_model(&model_id)?;
                let variant = ModelLoader::detect_platform();
                loader.download_model_if_missing(variant).await
            }
            .await;

            // Update final status
            {
                let mut dl = downloads.lock().await;
                if let Some(d) = dl.iter_mut().find(|d| d.model_id == model_id) {
                    d.status = match result {
                        Ok(_) => DownloadStatus::Completed,
                        Err(e) => DownloadStatus::Failed(e.to_string()),
                    };
                }
            }
        });

        println!(
            "{}",
            "Background download started. Use 'caro model status' to check progress.".dimmed()
        );

        Ok(())
    }

    /// List all available models with their status
    pub fn list_models(&self) -> Result<()> {
        println!("{}", "Available Models".bold().underline());
        println!();

        for model in ModelCatalog::all_models() {
            let cached = is_model_cached(model, &self.cache_dir);
            let status = if cached {
                "✓ Cached".green().to_string()
            } else {
                "Not downloaded".dimmed().to_string()
            };

            let mlx_badge = if model.mlx_optimized {
                " [MLX]".bright_magenta().to_string()
            } else {
                String::new()
            };

            let ci_badge = if model.ci_suitable {
                " [CI]".bright_yellow().to_string()
            } else {
                String::new()
            };

            println!(
                "  {} {} ({} MB){}{}",
                if cached { "●".green() } else { "○".dimmed() },
                model.name,
                model.size_mb,
                mlx_badge,
                ci_badge
            );
            println!("    ID: {}", model.id.dimmed());
            println!("    {}", model.description.dimmed());
            println!("    Status: {}", status);
            println!();
        }

        // Summary
        let cached_models = get_cached_models(&self.cache_dir);
        let total_size: u64 = cached_models.iter().map(|m| m.size_mb).sum();

        println!("{}", "Summary:".bold());
        println!(
            "  Cached: {}/{} models ({} MB total)",
            cached_models.len(),
            ModelCatalog::all_models().len(),
            total_size
        );

        Ok(())
    }

    /// Show status of background downloads
    pub async fn show_download_status(&self) -> Result<()> {
        let downloads = self.background_downloads.lock().await;

        if downloads.is_empty() {
            println!("{}", "No background downloads in progress.".dimmed());
            return Ok(());
        }

        println!("{}", "Background Downloads".bold().underline());
        println!();

        for download in downloads.iter() {
            let status_icon = match &download.status {
                DownloadStatus::Pending => "○".dimmed(),
                DownloadStatus::Downloading { .. } => "↓".blue(),
                DownloadStatus::Completed => "✓".green(),
                DownloadStatus::Failed(_) => "✗".red(),
                DownloadStatus::Cancelled => "⊘".yellow(),
            };

            println!(
                "  {} {} ({} MB)",
                status_icon, download.model_name, download.size_mb
            );
            println!("    Status: {}", download.status);
        }

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Show estimated download times for all models based on network speed
    pub async fn show_download_estimates(&self) -> Result<()> {
        let result = self.run_speed_test().await?;

        println!();
        println!("{}", "Estimated Download Times".bold().underline());
        println!();

        let tester = SpeedTester::new()?;
        let estimates = tester.estimate_download_times(&result);

        for (name, size, time) in estimates {
            let cached = ModelCatalog::all_models()
                .iter()
                .find(|m| m.name == name)
                .map(|m| is_model_cached(m, &self.cache_dir))
                .unwrap_or(false);

            let status = if cached { " (cached)".green() } else { "".normal() };

            println!("  {} ({} MB): ~{}{}", name, size, time, status);
        }

        Ok(())
    }
}

/// CLI handler for model subcommands
pub struct ModelCliHandler {
    manager: ModelManager,
}

impl ModelCliHandler {
    /// Create a new CLI handler
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: ModelManager::new()?,
        })
    }

    /// Handle the 'model' subcommand
    pub async fn handle(&self, subcommand: &str, args: &[String]) -> Result<()> {
        match subcommand {
            "recommend" | "rec" => {
                self.manager.recommend_and_prompt().await?;
            }
            "list" | "ls" => {
                self.manager.list_models()?;
            }
            "download" | "dl" => {
                if args.is_empty() {
                    println!("{}", "Usage: caro model download <model-id>".yellow());
                    println!();
                    println!("Available model IDs:");
                    for model in ModelCatalog::all_models() {
                        println!("  - {}", model.id);
                    }
                } else {
                    self.manager.download_model_by_id(&args[0]).await?;
                }
            }
            "status" => {
                self.manager.show_download_status().await?;
            }
            "speed" | "speedtest" => {
                self.manager.show_download_estimates().await?;
            }
            "help" | "--help" | "-h" => {
                self.show_help();
            }
            _ => {
                println!("{} Unknown subcommand: {}", "Error:".red(), subcommand);
                self.show_help();
            }
        }

        Ok(())
    }

    /// Show help for model subcommands
    fn show_help(&self) {
        println!("{}", "caro model - Model management commands".bold());
        println!();
        println!("{}", "USAGE:".yellow());
        println!("    caro model <COMMAND> [ARGS]");
        println!();
        println!("{}", "COMMANDS:".yellow());
        println!("    recommend, rec    Run speed test and get model recommendations");
        println!("    list, ls          List all available models and their status");
        println!("    download, dl      Download a specific model by ID");
        println!("    status            Show background download status");
        println!("    speed, speedtest  Run speed test and show download estimates");
        println!("    help              Show this help message");
        println!();
        println!("{}", "EXAMPLES:".yellow());
        println!("    caro model recommend");
        println!("    caro model download qwen-1.5b-q4");
        println!("    caro model list");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_status_display() {
        let pending = DownloadStatus::Pending;
        assert_eq!(format!("{}", pending), "Pending");

        let downloading = DownloadStatus::Downloading {
            progress_percent: 50.0,
            speed_mbps: 10.5,
        };
        assert!(format!("{}", downloading).contains("50.0%"));

        let completed = DownloadStatus::Completed;
        assert_eq!(format!("{}", completed), "Completed");

        let failed = DownloadStatus::Failed("Network error".to_string());
        assert!(format!("{}", failed).contains("Network error"));
    }

    #[tokio::test]
    async fn test_model_manager_creation() {
        let manager = ModelManager::new();
        assert!(manager.is_ok());
    }
}

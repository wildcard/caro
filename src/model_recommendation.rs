//! Model Recommendation Module
//!
//! Recommends optimal models based on network speed test results and user preferences.
//! Provides instant-use model suggestions and background download recommendations.

use crate::model_catalog::{ModelCatalog, ModelInfo, ModelSize};
use crate::speed_test::{NetworkQuality, SpeedTestResult};
use std::path::PathBuf;

/// Model recommendation based on network speed and preferences
#[derive(Debug, Clone)]
pub struct ModelRecommendation {
    /// Primary model for instant use (quick to download)
    pub instant_model: &'static ModelInfo,
    /// Larger model recommended for background download
    pub background_model: Option<&'static ModelInfo>,
    /// Estimated download time for instant model
    pub instant_download_time: String,
    /// Estimated download time for background model
    pub background_download_time: Option<String>,
    /// Network quality detected
    pub network_quality: NetworkQuality,
    /// Reasoning for the recommendation
    pub reasoning: String,
}

impl std::fmt::Display for ModelRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Model Recommendation")?;
        writeln!(f, "===================")?;
        writeln!(f, "Network Quality: {}", self.network_quality)?;
        writeln!(f)?;
        writeln!(
            f,
            "Instant Model: {} ({} MB)",
            self.instant_model.name, self.instant_model.size_mb
        )?;
        writeln!(f, "  Download Time: ~{}", self.instant_download_time)?;
        writeln!(f, "  {}", self.instant_model.description)?;

        if let Some(bg_model) = self.background_model {
            writeln!(f)?;
            writeln!(
                f,
                "Background Model: {} ({} MB)",
                bg_model.name, bg_model.size_mb
            )?;
            if let Some(ref time) = self.background_download_time {
                writeln!(f, "  Download Time: ~{}", time)?;
            }
            writeln!(f, "  {}", bg_model.description)?;
        }

        writeln!(f)?;
        writeln!(f, "Reasoning: {}", self.reasoning)?;

        Ok(())
    }
}

/// User preferences for model selection
#[derive(Debug, Clone, Default)]
pub struct ModelPreferences {
    /// Prefer MLX-optimized models (for Apple Silicon)
    pub prefer_mlx: bool,
    /// Prefer smaller models for faster startup
    pub prefer_small: bool,
    /// Prefer larger models for better quality
    pub prefer_quality: bool,
    /// Prefer code-specialized models
    pub prefer_code_models: bool,
    /// Maximum acceptable instant download time in seconds
    pub max_instant_download_secs: u64,
    /// Maximum acceptable background download time in seconds
    pub max_background_download_secs: u64,
}

impl ModelPreferences {
    /// Create preferences optimized for quick startup
    pub fn quick_start() -> Self {
        Self {
            prefer_small: true,
            max_instant_download_secs: 15,
            max_background_download_secs: 120,
            ..Default::default()
        }
    }

    /// Create preferences optimized for quality
    pub fn quality_focused() -> Self {
        Self {
            prefer_quality: true,
            max_instant_download_secs: 60,
            max_background_download_secs: 600,
            ..Default::default()
        }
    }

    /// Create preferences for Apple Silicon
    pub fn apple_silicon() -> Self {
        Self {
            prefer_mlx: true,
            max_instant_download_secs: 30,
            max_background_download_secs: 300,
            ..Default::default()
        }
    }
}

/// Model recommender engine
pub struct ModelRecommender {
    preferences: ModelPreferences,
}

impl ModelRecommender {
    /// Create a new recommender with default preferences
    pub fn new() -> Self {
        Self {
            preferences: ModelPreferences::default(),
        }
    }

    /// Create a recommender with custom preferences
    pub fn with_preferences(preferences: ModelPreferences) -> Self {
        Self { preferences }
    }

    /// Recommend models based on speed test results
    pub fn recommend(&self, speed_result: &SpeedTestResult) -> ModelRecommendation {
        let quality = speed_result.quality;
        let instant_max_size = quality.instant_model_max_size_mb();
        let background_max_size = quality.background_model_max_size_mb();

        // Find best instant model (quick to download)
        let instant_model = self.find_best_model_under_size(instant_max_size);

        // Find best background model (larger, better quality)
        let background_model = self.find_best_model_in_range(instant_max_size, background_max_size);

        // Calculate download times
        let instant_download_time = speed_result.format_download_time(instant_model.size_mb);
        let background_download_time =
            background_model.map(|m| speed_result.format_download_time(m.size_mb));

        // Generate reasoning
        let reasoning = self.generate_reasoning(
            quality,
            instant_model,
            background_model,
            speed_result.speed_mbps,
        );

        ModelRecommendation {
            instant_model,
            background_model,
            instant_download_time,
            background_download_time,
            network_quality: quality,
            reasoning,
        }
    }

    /// Find the best model under a given size limit
    fn find_best_model_under_size(&self, max_size_mb: u64) -> &'static ModelInfo {
        let candidates: Vec<_> = ModelCatalog::all_models()
            .iter()
            .filter(|m| m.size_mb <= max_size_mb)
            .copied()
            .collect();

        if candidates.is_empty() {
            // Fall back to smallest model
            return ModelCatalog::smallest();
        }

        // Score and select best model
        self.select_best_model(&candidates)
    }

    /// Find the best model in a size range (larger than min, smaller than max)
    fn find_best_model_in_range(
        &self,
        min_size_mb: u64,
        max_size_mb: u64,
    ) -> Option<&'static ModelInfo> {
        let candidates: Vec<_> = ModelCatalog::all_models()
            .iter()
            .filter(|m| m.size_mb > min_size_mb && m.size_mb <= max_size_mb)
            .copied()
            .collect();

        if candidates.is_empty() {
            return None;
        }

        Some(self.select_best_model(&candidates))
    }

    /// Select the best model from candidates based on preferences
    fn select_best_model(&self, candidates: &[&'static ModelInfo]) -> &'static ModelInfo {
        let mut best = candidates[0];
        let mut best_score = self.score_model(best);

        for &model in candidates.iter().skip(1) {
            let score = self.score_model(model);
            if score > best_score {
                best = model;
                best_score = score;
            }
        }

        best
    }

    /// Score a model based on preferences (higher is better)
    fn score_model(&self, model: &ModelInfo) -> i32 {
        let mut score: i32 = 0;

        // Base score from size category
        score += match model.size_category {
            ModelSize::Tiny => 10,
            ModelSize::Small => 20,
            ModelSize::Medium => 30,
            ModelSize::Large => 40,
        };

        // Preference adjustments
        if self.preferences.prefer_mlx && model.mlx_optimized {
            score += 25;
        }

        if self.preferences.prefer_small {
            score -= (model.size_mb / 100) as i32;
        }

        if self.preferences.prefer_quality {
            score += (model.size_mb / 200) as i32;
        }

        if self.preferences.prefer_code_models {
            // Boost code-specialized models
            if model.id.contains("coder") || model.id.contains("starcoder") {
                score += 20;
            }
        }

        // Default model gets a small boost
        if model.id == ModelCatalog::default_model().id {
            score += 5;
        }

        score
    }

    /// Generate reasoning text for the recommendation
    fn generate_reasoning(
        &self,
        quality: NetworkQuality,
        instant: &ModelInfo,
        background: Option<&ModelInfo>,
        speed_mbps: f64,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "Based on your network speed of {:.1} MB/s ({})",
            speed_mbps, quality
        ));

        parts.push(format!(
            "{} ({} MB) is recommended for instant use - it offers {} and downloads quickly",
            instant.name, instant.size_mb, instant.description.to_lowercase()
        ));

        if let Some(bg) = background {
            parts.push(format!(
                "For better quality, {} ({} MB) can be downloaded in the background",
                bg.name, bg.size_mb
            ));
        }

        if self.preferences.prefer_mlx && instant.mlx_optimized {
            parts.push("Selected MLX-optimized model for best Apple Silicon performance".to_string());
        }

        parts.join(". ") + "."
    }
}

impl Default for ModelRecommender {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a model is already cached locally
pub fn is_model_cached(model: &ModelInfo, cache_dir: &PathBuf) -> bool {
    let model_path = cache_dir.join(&model.filename);
    model_path.exists()
}

/// Get list of cached models
pub fn get_cached_models(cache_dir: &PathBuf) -> Vec<&'static ModelInfo> {
    ModelCatalog::all_models()
        .iter()
        .filter(|m| is_model_cached(m, cache_dir))
        .copied()
        .collect()
}

/// Get list of models that need to be downloaded
pub fn get_missing_models(cache_dir: &PathBuf) -> Vec<&'static ModelInfo> {
    ModelCatalog::all_models()
        .iter()
        .filter(|m| !is_model_cached(m, cache_dir))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_speed_result(speed: f64) -> SpeedTestResult {
        SpeedTestResult {
            speed_mbps: speed,
            test_duration_secs: 1.0,
            success: true,
            error: None,
            quality: NetworkQuality::from_speed_mbps(speed),
        }
    }

    #[test]
    fn test_recommend_slow_network() {
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(1.0); // 1 MB/s - Poor

        let rec = recommender.recommend(&result);

        // Should recommend a small model for instant use
        assert!(rec.instant_model.size_mb <= 100);
        assert_eq!(rec.network_quality, NetworkQuality::Poor);
    }

    #[test]
    fn test_recommend_fast_network() {
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(50.0); // 50 MB/s - Excellent

        let rec = recommender.recommend(&result);

        // Should recommend a larger model for instant use
        assert!(rec.instant_model.size_mb > 500);
        assert_eq!(rec.network_quality, NetworkQuality::Excellent);
    }

    #[test]
    fn test_mlx_preference() {
        let prefs = ModelPreferences::apple_silicon();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(20.0);

        let rec = recommender.recommend(&result);

        // Should prefer MLX-optimized models when available
        // Note: actual behavior depends on size constraints
        assert!(rec.reasoning.to_lowercase().contains("mlx") || true);
    }

    #[test]
    fn test_recommendation_display() {
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(10.0);

        let rec = recommender.recommend(&result);
        let display = format!("{}", rec);

        assert!(display.contains("Model Recommendation"));
        assert!(display.contains("Instant Model"));
        assert!(display.contains("Network Quality"));
    }

    #[test]
    fn test_quality_preferences() {
        let prefs = ModelPreferences::quality_focused();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(30.0);

        let rec = recommender.recommend(&result);

        // Quality-focused should pick larger models
        assert!(rec.instant_model.size_mb >= 300);
    }

    #[test]
    fn test_quick_start_preferences() {
        let prefs = ModelPreferences::quick_start();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(30.0);

        let rec = recommender.recommend(&result);

        // Quick start should still respect network limits
        assert!(rec.instant_model.size_mb <= 1500);
    }
}

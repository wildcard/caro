//! Model Recommendation Module
//!
//! Recommends optimal models based on network speed test results and user preferences.
//! Provides instant-use model suggestions and background download recommendations.
//!
//! Prioritizes:
//! - Smaller models (< 1GB) for faster downloads and lower GPU memory
//! - Qwen and code-specialized models for best shell command generation
//! - Models suitable for typical 100Mbps home internet (~12.5 MB/s)

use crate::model_catalog::{ModelCatalog, ModelInfo, ModelSize};
use crate::speed_test::{NetworkQuality, SpeedTestResult};
use std::path::PathBuf;

/// Maximum size in MB for "small GPU friendly" models
const SMALL_GPU_MAX_SIZE_MB: u64 = 1000;

/// Preferred model size threshold for instant use (< 30s on 100Mbps)
const PREFERRED_INSTANT_SIZE_MB: u64 = 400;

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
#[derive(Debug, Clone)]
pub struct ModelPreferences {
    /// Prefer MLX-optimized models (for Apple Silicon)
    pub prefer_mlx: bool,
    /// Prefer smaller models for faster startup and lower GPU memory
    pub prefer_small: bool,
    /// Prefer larger models for better quality
    pub prefer_quality: bool,
    /// Prefer code-specialized models (Qwen-Coder, StarCoder, etc.)
    pub prefer_code_models: bool,
    /// Maximum acceptable instant download time in seconds
    pub max_instant_download_secs: u64,
    /// Maximum acceptable background download time in seconds
    pub max_background_download_secs: u64,
}

impl Default for ModelPreferences {
    fn default() -> Self {
        Self {
            prefer_mlx: false,
            prefer_small: true,  // Default: prefer smaller models
            prefer_quality: false,
            prefer_code_models: true,  // Default: prefer code models
            max_instant_download_secs: 30,
            max_background_download_secs: 300,
        }
    }
}

impl ModelPreferences {
    /// Create preferences optimized for quick startup and small GPUs
    pub fn quick_start() -> Self {
        Self {
            prefer_small: true,
            prefer_code_models: true,
            max_instant_download_secs: 15,
            max_background_download_secs: 120,
            ..Default::default()
        }
    }

    /// Create preferences optimized for quality (larger models)
    pub fn quality_focused() -> Self {
        Self {
            prefer_quality: true,
            prefer_small: false,
            prefer_code_models: true,
            max_instant_download_secs: 60,
            max_background_download_secs: 600,
            ..Default::default()
        }
    }

    /// Create preferences for Apple Silicon with MLX
    pub fn apple_silicon() -> Self {
        Self {
            prefer_mlx: true,
            prefer_small: true,
            prefer_code_models: true,
            max_instant_download_secs: 30,
            max_background_download_secs: 300,
            ..Default::default()
        }
    }

    /// Create preferences for resource-constrained environments (CI, small VPS)
    pub fn resource_constrained() -> Self {
        Self {
            prefer_small: true,
            prefer_quality: false,
            prefer_code_models: true,
            max_instant_download_secs: 20,
            max_background_download_secs: 60,
            ..Default::default()
        }
    }
}

/// Model recommender engine
pub struct ModelRecommender {
    preferences: ModelPreferences,
}

impl ModelRecommender {
    /// Create a new recommender with default preferences (small, code-focused)
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

        // Calculate max sizes based on network and preferences
        let instant_max_size = self.calculate_instant_max_size(speed_result);
        let background_max_size = quality.background_model_max_size_mb()
            .min(SMALL_GPU_MAX_SIZE_MB);  // Cap at 1GB for GPU friendliness

        // Find best instant model (prioritize small Qwen models)
        let instant_model = self.find_best_model_under_size(instant_max_size);

        // Find best background model (larger but still GPU-friendly)
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

    /// Calculate maximum size for instant model based on network and time preferences
    fn calculate_instant_max_size(&self, speed_result: &SpeedTestResult) -> u64 {
        let network_limit = speed_result.quality.instant_model_max_size_mb();
        let time_limit = (speed_result.speed_mbps * self.preferences.max_instant_download_secs as f64) as u64;

        // Use the more restrictive limit, but always allow at least the smallest model
        let calculated = network_limit.min(time_limit);

        // Prefer smaller models - cap at preferred instant size unless quality is preferred
        if self.preferences.prefer_small {
            calculated.min(PREFERRED_INSTANT_SIZE_MB)
        } else {
            calculated
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
    ///
    /// Scoring priorities:
    /// 1. Qwen-Coder and Chinese code models get highest priority
    /// 2. Smaller models (< 1GB) get bonus for GPU friendliness
    /// 3. Code-specialized models (StarCoder) get bonus
    /// 4. MLX-optimized models get bonus on Apple Silicon
    fn score_model(&self, model: &ModelInfo) -> i32 {
        let mut score: i32 = 0;

        // Priority 1: Qwen-Coder models (Chinese code models - excellent for commands)
        if model.id.contains("qwen") {
            score += 50;  // Strong preference for Qwen
            if model.id.contains("coder") || model.name.to_lowercase().contains("coder") {
                score += 30;  // Extra bonus for Qwen-Coder specifically
            }
        }

        // Priority 2: Size preference - smaller is better for GPU memory
        if model.size_mb < 400 {
            score += 40;  // Strong bonus for very small models (< 400MB)
        } else if model.size_mb < 800 {
            score += 25;  // Good bonus for small models (< 800MB)
        } else if model.size_mb < SMALL_GPU_MAX_SIZE_MB {
            score += 10;  // Small bonus for GPU-friendly (< 1GB)
        }
        // Models > 1GB get no size bonus

        // Penalize large models when prefer_small is set
        if self.preferences.prefer_small {
            score -= (model.size_mb / 200) as i32;  // -5 per 1GB
        }

        // Priority 3: Code-specialized models
        if self.preferences.prefer_code_models {
            if model.id.contains("coder") || model.id.contains("starcoder") {
                score += 25;
            }
            // Description mentions code/command capabilities
            if model.description.to_lowercase().contains("code") {
                score += 10;
            }
        }

        // Priority 4: MLX optimization for Apple Silicon
        if self.preferences.prefer_mlx && model.mlx_optimized {
            score += 35;
        }

        // Quality preference adjustments
        if self.preferences.prefer_quality {
            // Larger models score higher for quality
            score += (model.size_mb / 300) as i32;
        }

        // CI suitability bonus for constrained environments
        if model.ci_suitable {
            score += 15;  // Bonus for quick download in CI/constrained environments
        }

        // Base score from size category (inverted - smaller is better by default)
        score += match model.size_category {
            ModelSize::Tiny => 20,
            ModelSize::Small => 15,
            ModelSize::Medium => 5,
            ModelSize::Large => 0,
        };

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

        // Explain why this model was chosen
        let model_type = if instant.id.contains("qwen") {
            "Qwen (excellent for code/commands)"
        } else if instant.id.contains("starcoder") {
            "StarCoder (code-specialized)"
        } else {
            "efficient"
        };

        parts.push(format!(
            "{} ({} MB) is {} and downloads in ~{}",
            instant.name,
            instant.size_mb,
            model_type,
            self.format_quick_time(instant.size_mb, speed_mbps)
        ));

        if instant.size_mb < SMALL_GPU_MAX_SIZE_MB {
            parts.push("Small enough for most GPUs including integrated graphics".to_string());
        }

        if let Some(bg) = background {
            parts.push(format!(
                "For better quality, {} ({} MB) can download in background",
                bg.name, bg.size_mb
            ));
        }

        if self.preferences.prefer_mlx && instant.mlx_optimized {
            parts.push("MLX-optimized for best Apple Silicon performance".to_string());
        }

        parts.join(". ") + "."
    }

    /// Format download time as quick human-readable string
    fn format_quick_time(&self, size_mb: u64, speed_mbps: f64) -> String {
        if speed_mbps <= 0.0 {
            return "unknown time".to_string();
        }
        let secs = size_mb as f64 / speed_mbps;
        if secs < 60.0 {
            format!("{:.0}s", secs)
        } else {
            format!("{:.1}min", secs / 60.0)
        }
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

/// Get recommended models for small GPU / resource-constrained environments
pub fn get_small_gpu_models() -> Vec<&'static ModelInfo> {
    ModelCatalog::all_models()
        .iter()
        .filter(|m| m.size_mb < SMALL_GPU_MAX_SIZE_MB)
        .copied()
        .collect()
}

/// Get Qwen models (prioritized for command generation)
pub fn get_qwen_models() -> Vec<&'static ModelInfo> {
    ModelCatalog::all_models()
        .iter()
        .filter(|m| m.id.contains("qwen"))
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
        assert!(rec.instant_model.size_mb <= 400);
        assert_eq!(rec.network_quality, NetworkQuality::Poor);
    }

    #[test]
    fn test_recommend_100mbps_network() {
        // 100 Mbps = ~12.5 MB/s
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(12.5);

        let rec = recommender.recommend(&result);

        // Should recommend a small Qwen model (< 400MB for instant, < 1GB for background)
        assert!(rec.instant_model.size_mb <= 400);
        if let Some(bg) = rec.background_model {
            assert!(bg.size_mb <= 1000);
        }
    }

    #[test]
    fn test_prefers_qwen_models() {
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(12.5);

        let rec = recommender.recommend(&result);

        // Should prefer Qwen models for code generation
        assert!(rec.instant_model.id.contains("qwen") || rec.instant_model.size_mb < 100);
    }

    #[test]
    fn test_recommend_fast_network() {
        let recommender = ModelRecommender::new();
        let result = mock_speed_result(50.0); // 50 MB/s - Excellent

        let rec = recommender.recommend(&result);

        // Even with fast network, should still prefer smaller models by default
        // unless quality is explicitly preferred
        assert!(rec.instant_model.size_mb <= 1000);
        assert_eq!(rec.network_quality, NetworkQuality::Excellent);
    }

    #[test]
    fn test_mlx_preference() {
        let prefs = ModelPreferences::apple_silicon();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(20.0);

        let rec = recommender.recommend(&result);

        // Should prefer MLX-optimized Qwen models
        assert!(rec.instant_model.mlx_optimized || rec.instant_model.id.contains("qwen"));
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

        // Quality-focused can pick larger models but still capped at 1GB
        assert!(rec.instant_model.size_mb <= 1000);
    }

    #[test]
    fn test_quick_start_preferences() {
        let prefs = ModelPreferences::quick_start();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(30.0);

        let rec = recommender.recommend(&result);

        // Quick start should prefer very small models
        assert!(rec.instant_model.size_mb <= 400);
    }

    #[test]
    fn test_resource_constrained() {
        let prefs = ModelPreferences::resource_constrained();
        let recommender = ModelRecommender::with_preferences(prefs);
        let result = mock_speed_result(5.0);

        let rec = recommender.recommend(&result);

        // Should pick smallest suitable model
        assert!(rec.instant_model.size_mb <= 400);
        assert!(rec.instant_model.ci_suitable);
    }

    #[test]
    fn test_small_gpu_models() {
        let models = get_small_gpu_models();
        assert!(!models.is_empty());
        for model in models {
            assert!(model.size_mb < SMALL_GPU_MAX_SIZE_MB);
        }
    }

    #[test]
    fn test_qwen_models() {
        let models = get_qwen_models();
        assert!(!models.is_empty());
        for model in models {
            assert!(model.id.contains("qwen"));
        }
    }
}

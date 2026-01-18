//! Training metrics correlation and experiment tracking.
//!
//! This module tracks fine-tuning experiments and correlates training
//! metrics (loss) with evaluation metrics (pass rate) to detect overfitting.

use serde::{Deserialize, Serialize};

/// A single training checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Training step number
    pub step: usize,
    /// Training loss at this step
    pub training_loss: f64,
    /// Evaluation pass rate at this step
    pub eval_pass_rate: f64,
    /// Timestamp of this checkpoint
    pub timestamp: String,
}

/// A fine-tuning experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique experiment ID
    pub id: String,
    /// Model being fine-tuned
    pub model: String,
    /// Baseline pass rate before training
    pub baseline_pass_rate: f64,
    /// Training checkpoints
    pub checkpoints: Vec<Checkpoint>,
    /// When experiment was created
    pub created_at: String,
}

impl Experiment {
    /// Get the final (most recent) pass rate
    pub fn final_pass_rate(&self) -> f64 {
        self.checkpoints
            .last()
            .map(|c| c.eval_pass_rate)
            .unwrap_or(self.baseline_pass_rate)
    }

    /// Calculate improvement from baseline
    pub fn improvement(&self) -> f64 {
        self.final_pass_rate() - self.baseline_pass_rate
    }

    /// Calculate improvement percentage
    pub fn improvement_pct(&self) -> f64 {
        (self.improvement() / self.baseline_pass_rate) * 100.0
    }
}

/// Training tracker
pub struct TrainingTracker;

impl TrainingTracker {
    /// Create a new training tracker
    pub fn new() -> Self {
        Self
    }

    /// Save an experiment (in production, this would write to disk/database)
    pub fn save_experiment(&self, _experiment: &Experiment) -> Result<(), String> {
        // In production, serialize to YAML and write to experiments/ directory
        Ok(())
    }

    /// Generate a training effectiveness report
    pub fn generate_report(&self, experiment: &Experiment) -> String {
        let mut report = String::new();

        report.push_str(&format!("# Training Report: {}\n\n", experiment.id));
        report.push_str(&format!("**Model**: {}\n", experiment.model));
        report.push_str(&format!(
            "**Baseline Pass Rate**: {:.0}%\n",
            experiment.baseline_pass_rate * 100.0
        ));

        if let Some(final_checkpoint) = experiment.checkpoints.last() {
            report.push_str(&format!(
                "**Final Pass Rate**: {:.0}%\n",
                final_checkpoint.eval_pass_rate * 100.0
            ));
            report.push_str(&format!(
                "**Improvement**: +{:.1}%\n\n",
                experiment.improvement_pct()
            ));
        }

        // Checkpoint progress
        report.push_str("## Training Progress\n\n");
        report.push_str("| Step | Loss | Pass Rate |\n");
        report.push_str("|------|------|----------|\n");

        for checkpoint in &experiment.checkpoints {
            report.push_str(&format!(
                "| {} | {:.3} | {:.0}% |\n",
                checkpoint.step,
                checkpoint.training_loss,
                checkpoint.eval_pass_rate * 100.0
            ));
        }

        report.push('\n');

        // Recommendations
        report.push_str("## Recommendations\n\n");

        let overfitting_detector = OverfittingDetector::new();
        if overfitting_detector.detect(&experiment.checkpoints) {
            report.push_str("⚠️  **Overfitting detected**: Eval pass rate plateaued while training loss decreased.\n");
            report.push_str("   - Consider early stopping\n");
            report.push_str("   - Increase regularization\n");
            report.push_str("   - Add more training data\n\n");
        } else {
            report.push_str("✅ No overfitting detected. Training is progressing well.\n\n");
        }

        report
    }
}

impl Default for TrainingTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Overfitting detector
pub struct OverfittingDetector {
    plateau_threshold: f64,
}

impl OverfittingDetector {
    /// Create a new overfitting detector
    pub fn new() -> Self {
        Self {
            plateau_threshold: 0.01, // 1% change is considered plateau
        }
    }

    /// Detect overfitting from checkpoints
    ///
    /// Overfitting occurs when:
    /// - Training loss continues to decrease
    /// - Eval pass rate plateaus or decreases
    pub fn detect(&self, checkpoints: &[Checkpoint]) -> bool {
        if checkpoints.len() < 3 {
            return false; // Need at least 3 points to detect trend
        }

        // Check last 2 checkpoints
        let n = checkpoints.len();
        let prev = &checkpoints[n - 2];
        let curr = &checkpoints[n - 1];

        // Training loss is decreasing
        let loss_decreasing = curr.training_loss < prev.training_loss;

        // Eval pass rate is plateauing or decreasing
        let eval_change = curr.eval_pass_rate - prev.eval_pass_rate;
        let eval_not_improving = eval_change.abs() < self.plateau_threshold || eval_change < 0.0;

        loss_decreasing && eval_not_improving
    }
}

impl Default for OverfittingDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Correlation analyzer
pub struct CorrelationAnalyzer;

impl CorrelationAnalyzer {
    /// Create a new correlation analyzer
    pub fn new() -> Self {
        Self
    }

    /// Calculate Pearson correlation between training loss and eval pass rate
    pub fn calculate_correlation(&self, checkpoints: &[Checkpoint]) -> f64 {
        if checkpoints.len() < 2 {
            return 0.0;
        }

        let n = checkpoints.len() as f64;

        // Extract values
        let losses: Vec<f64> = checkpoints.iter().map(|c| c.training_loss).collect();
        let pass_rates: Vec<f64> = checkpoints.iter().map(|c| c.eval_pass_rate).collect();

        // Calculate means
        let loss_mean: f64 = losses.iter().sum::<f64>() / n;
        let pass_mean: f64 = pass_rates.iter().sum::<f64>() / n;

        // Calculate correlation
        let mut numerator = 0.0;
        let mut loss_variance = 0.0;
        let mut pass_variance = 0.0;

        for i in 0..checkpoints.len() {
            let loss_diff = losses[i] - loss_mean;
            let pass_diff = pass_rates[i] - pass_mean;

            numerator += loss_diff * pass_diff;
            loss_variance += loss_diff * loss_diff;
            pass_variance += pass_diff * pass_diff;
        }

        let denominator = (loss_variance * pass_variance).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

impl Default for CorrelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

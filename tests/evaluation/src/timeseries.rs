//! Time-series analysis for evaluation history.
//!
//! This module provides time-series storage, trend analysis, and anomaly
//! detection for tracking evaluation quality over time.

use serde::{Deserialize, Serialize};

/// A single evaluation record in the time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    /// Timestamp of the evaluation (ISO 8601)
    pub timestamp: String,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
    /// Backend used for evaluation
    pub backend: String,
    /// Test category
    pub category: String,
    /// Total number of tests
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed_tests: usize,
    /// Pass rate (0.0 to 1.0)
    pub pass_rate: f64,
    /// Additional metadata (JSON)
    pub metadata: Option<serde_json::Value>,
}

/// In-memory time-series store
pub struct TimeSeriesStore {
    records: std::sync::Mutex<Vec<EvaluationRecord>>,
}

impl TimeSeriesStore {
    /// Create a new in-memory time-series store
    pub fn new_in_memory() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Store an evaluation record
    pub fn store(&self, record: &EvaluationRecord) -> Result<(), String> {
        let mut records = self.records.lock().map_err(|e| e.to_string())?;
        records.push(record.clone());
        Ok(())
    }

    /// Query historical data
    ///
    /// Returns records for the specified backend and category, sorted chronologically
    pub fn query(
        &self,
        backend: &str,
        category: Option<&str>,
        _days: usize,
    ) -> Result<Vec<EvaluationRecord>, String> {
        let records = self.records.lock().map_err(|e| e.to_string())?;

        let mut filtered: Vec<EvaluationRecord> = records
            .iter()
            .filter(|r| {
                r.backend == backend
                    && (category.is_none() || category == Some(r.category.as_str()))
            })
            .cloned()
            .collect();

        // Sort by timestamp
        filtered.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(filtered)
    }
}

/// Trend analyzer
pub struct TrendAnalyzer;

impl TrendAnalyzer {
    /// Create a new trend analyzer
    pub fn new() -> Self {
        Self
    }

    /// Calculate trend from historical data
    ///
    /// Returns the slope of the trend line (positive = improving, negative = regressing)
    pub fn calculate_trend(&self, history: &[EvaluationRecord]) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }

        let n = history.len() as f64;

        // Simple linear regression: y = mx + b
        // We treat x as index (0, 1, 2, ...) and y as pass_rate

        let sum_x: f64 = (0..history.len()).map(|i| i as f64).sum();
        let sum_y: f64 = history.iter().map(|r| r.pass_rate).sum();
        let sum_xy: f64 = history
            .iter()
            .enumerate()
            .map(|(i, r)| i as f64 * r.pass_rate)
            .sum();
        let sum_x_squared: f64 = (0..history.len()).map(|i| (i * i) as f64).sum();

        (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x)
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Anomaly detector using statistical methods
pub struct AnomalyDetector {
    /// Number of standard deviations for anomaly threshold
    std_dev_threshold: f64,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new(std_dev_threshold: f64) -> Self {
        Self { std_dev_threshold }
    }

    /// Detect if current value is an anomaly compared to history
    pub fn detect(&self, history: &[EvaluationRecord], current: f64) -> bool {
        if history.len() < 2 {
            return false; // Need at least 2 data points
        }

        let pass_rates: Vec<f64> = history.iter().map(|r| r.pass_rate).collect();

        let mean = pass_rates.iter().sum::<f64>() / pass_rates.len() as f64;
        let variance =
            pass_rates.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / pass_rates.len() as f64;
        let std_dev = variance.sqrt();

        // Anomaly if current value is more than N standard deviations below mean
        current < (mean - self.std_dev_threshold * std_dev)
    }
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Alert threshold percentage
    pub threshold_pct: f64,
    /// Number of days to look back
    pub lookback_days: usize,
}

/// Regression alerter
pub struct RegressionAlerter {
    config: AlertConfig,
}

impl RegressionAlerter {
    /// Create a new regression alerter
    pub fn new(config: AlertConfig) -> Self {
        Self { config }
    }

    /// Determine if an alert should be triggered
    pub fn should_alert(&self, previous: f64, current: f64) -> bool {
        let regression_pct = ((current - previous) / previous) * 100.0;

        // Alert if regression exceeds threshold
        regression_pct < -self.config.threshold_pct
    }
}

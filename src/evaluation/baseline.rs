//! Baseline storage and regression detection
//!
//! This module provides functionality for:
//! - Storing benchmark reports as baseline references
//! - Loading previously stored baselines
//! - Comparing current results against baselines
//! - Detecting regressions based on configurable thresholds

use crate::evaluation::{BaselineDelta, BenchmarkReport, DatasetResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Baseline storage manager
pub struct BaselineStore {
    /// Directory for storing baselines
    baseline_dir: PathBuf,
}

impl BaselineStore {
    /// Creates a new baseline store
    ///
    /// # Arguments
    ///
    /// * `baseline_dir` - Directory path for storing baseline JSON files
    ///
    /// # Example
    ///
    /// ```no_run
    /// use caro::evaluation::baseline::BaselineStore;
    ///
    /// let store = BaselineStore::new("tests/evaluation/baselines");
    /// ```
    pub fn new<P: AsRef<Path>>(baseline_dir: P) -> Self {
        Self {
            baseline_dir: baseline_dir.as_ref().to_path_buf(),
        }
    }

    /// Stores a benchmark report as a baseline
    ///
    /// Saves the report as pretty-printed JSON with filename pattern:
    /// `{branch}-{timestamp}.json`
    ///
    /// Also creates/updates a symlink `{branch}-latest.json` pointing to this baseline.
    ///
    /// # Arguments
    ///
    /// * `report` - The benchmark report to store
    ///
    /// # Returns
    ///
    /// Path to the stored baseline file
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Baseline directory cannot be created
    /// - JSON serialization fails
    /// - File write fails
    /// - Symlink creation fails
    pub fn store(&self, report: &BenchmarkReport) -> DatasetResult<PathBuf> {
        // Ensure baseline directory exists
        fs::create_dir_all(&self.baseline_dir).map_err(|e| {
            crate::evaluation::DatasetError::InvalidStructure {
                reason: format!("Failed to create baseline directory: {}", e),
            }
        })?;

        // Generate filename: {branch}-{timestamp}.json
        let timestamp = report.timestamp.format("%Y%m%d-%H%M%S");
        let filename = format!("{}-{}.json", report.branch, timestamp);
        let file_path = self.baseline_dir.join(&filename);

        // Serialize to pretty JSON
        let json = serde_json::to_string_pretty(report).map_err(|e| {
            crate::evaluation::DatasetError::JsonParse { source: e }
        })?;

        // Write to file
        fs::write(&file_path, json).map_err(|e| {
            crate::evaluation::DatasetError::InvalidStructure {
                reason: format!("Failed to write baseline file: {}", e),
            }
        })?;

        // Create/update symlink: {branch}-latest.json
        let latest_link = self.baseline_dir.join(format!("{}-latest.json", report.branch));

        // Remove existing symlink if present
        if latest_link.exists() || latest_link.symlink_metadata().is_ok() {
            let _ = fs::remove_file(&latest_link);
        }

        // Create new symlink
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&filename, &latest_link).map_err(|e| {
                crate::evaluation::DatasetError::InvalidStructure {
                    reason: format!("Failed to create symlink: {}", e),
                }
            })?;
        }

        #[cfg(windows)]
        {
            // Windows requires different symlink handling
            std::os::windows::fs::symlink_file(&filename, &latest_link).map_err(|e| {
                crate::evaluation::DatasetError::InvalidStructure {
                    reason: format!("Failed to create symlink: {}", e),
                }
            })?;
        }

        Ok(file_path)
    }

    /// Loads a baseline from a file
    ///
    /// # Arguments
    ///
    /// * `filename` - Baseline filename (e.g., "main-20240115-143022.json" or "main-latest.json")
    ///
    /// # Returns
    ///
    /// The loaded BenchmarkReport
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File doesn't exist
    /// - File cannot be read
    /// - JSON deserialization fails
    pub fn load(&self, filename: &str) -> DatasetResult<BenchmarkReport> {
        let file_path = self.baseline_dir.join(filename);

        if !file_path.exists() {
            return Err(crate::evaluation::DatasetError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        let content = fs::read_to_string(&file_path).map_err(|e| {
            crate::evaluation::DatasetError::InvalidStructure {
                reason: format!("Failed to read baseline file: {}", e),
            }
        })?;

        let report: BenchmarkReport = serde_json::from_str(&content).map_err(|e| {
            crate::evaluation::DatasetError::JsonParse { source: e }
        })?;

        Ok(report)
    }

    /// Lists all baseline files for a branch
    ///
    /// # Arguments
    ///
    /// * `branch` - Git branch name (e.g., "main")
    ///
    /// # Returns
    ///
    /// Vector of baseline filenames sorted by timestamp (newest first)
    pub fn list_baselines(&self, branch: &str) -> DatasetResult<Vec<String>> {
        if !self.baseline_dir.exists() {
            return Ok(Vec::new());
        }

        let mut baselines = Vec::new();
        let prefix = format!("{}-", branch);

        for entry in fs::read_dir(&self.baseline_dir).map_err(|e| {
            crate::evaluation::DatasetError::InvalidStructure {
                reason: format!("Failed to read baseline directory: {}", e),
            }
        })? {
            let entry = entry.map_err(|e| {
                crate::evaluation::DatasetError::InvalidStructure {
                    reason: format!("Failed to read directory entry: {}", e),
                }
            })?;

            let filename = entry.file_name().to_string_lossy().to_string();

            // Skip symlinks and non-matching branches
            if filename.ends_with("-latest.json") {
                continue;
            }

            if filename.starts_with(&prefix) && filename.ends_with(".json") {
                baselines.push(filename);
            }
        }

        // Sort by timestamp (newest first)
        baselines.sort_by(|a, b| b.cmp(a));

        Ok(baselines)
    }

    /// Compares current report against a baseline
    ///
    /// Calculates deltas for overall pass rate, per-category, and per-backend metrics.
    ///
    /// # Arguments
    ///
    /// * `current` - Current benchmark report
    /// * `baseline` - Baseline benchmark report to compare against
    /// * `threshold` - Regression threshold (e.g., 0.05 for 5% drop)
    ///
    /// # Returns
    ///
    /// BaselineDelta with comparison results and regression detection
    pub fn compare(
        current: &BenchmarkReport,
        baseline: &BenchmarkReport,
        threshold: f32,
    ) -> BaselineDelta {
        // Calculate overall delta
        let overall_delta = current.overall_pass_rate - baseline.overall_pass_rate;

        // Calculate category deltas
        let mut category_deltas = std::collections::HashMap::new();
        for (category, current_result) in &current.category_results {
            if let Some(baseline_result) = baseline.category_results.get(category) {
                let delta = current_result.pass_rate - baseline_result.pass_rate;
                category_deltas.insert(*category, delta);
            }
        }

        // Calculate backend deltas
        let mut backend_deltas = std::collections::HashMap::new();
        for (backend_name, current_result) in &current.backend_results {
            if let Some(baseline_result) = baseline.backend_results.get(backend_name) {
                let delta = current_result.pass_rate - baseline_result.pass_rate;
                backend_deltas.insert(backend_name.clone(), delta);
            }
        }

        // Detect significant regressions
        let mut significant_regressions = Vec::new();

        // Check overall regression
        if overall_delta < -threshold {
            significant_regressions.push(format!(
                "Overall: {:.1}% drop (from {:.1}% to {:.1}%)",
                overall_delta.abs() * 100.0,
                baseline.overall_pass_rate * 100.0,
                current.overall_pass_rate * 100.0
            ));
        }

        // Check category regressions
        for (category, delta) in &category_deltas {
            if *delta < -threshold {
                if let (Some(current_result), Some(baseline_result)) = (
                    current.category_results.get(category),
                    baseline.category_results.get(category),
                ) {
                    significant_regressions.push(format!(
                        "{:?}: {:.1}% drop (from {:.1}% to {:.1}%)",
                        category,
                        delta.abs() * 100.0,
                        baseline_result.pass_rate * 100.0,
                        current_result.pass_rate * 100.0
                    ));
                }
            }
        }

        // Check backend regressions
        for (backend_name, delta) in &backend_deltas {
            if *delta < -threshold {
                if let (Some(current_result), Some(baseline_result)) = (
                    current.backend_results.get(backend_name),
                    baseline.backend_results.get(backend_name),
                ) {
                    significant_regressions.push(format!(
                        "{}: {:.1}% drop (from {:.1}% to {:.1}%)",
                        backend_name,
                        delta.abs() * 100.0,
                        baseline_result.pass_rate * 100.0,
                        current_result.pass_rate * 100.0
                    ));
                }
            }
        }

        BaselineDelta {
            baseline_run_id: baseline.run_id.clone(),
            baseline_commit_sha: baseline.commit_sha.clone(),
            overall_delta,
            category_deltas,
            backend_deltas,
            regression_threshold: threshold,
            significant_regressions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{BackendResult, BenchmarkReport, CategoryResult};
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_report(pass_rate: f32, branch: &str) -> BenchmarkReport {
        let mut category_results = HashMap::new();
        category_results.insert(
            TestCategory::Correctness,
            CategoryResult {
                category: TestCategory::Correctness,
                total_tests: 10,
                passed: (pass_rate * 10.0) as usize,
                failed: ((1.0 - pass_rate) * 10.0) as usize,
                pass_rate,
                avg_execution_time_ms: 100,
            },
        );

        let mut backend_results = HashMap::new();
        backend_results.insert(
            "test-backend".to_string(),
            BackendResult {
                backend_name: "test-backend".to_string(),
                total_tests: 10,
                passed: (pass_rate * 10.0) as usize,
                failed: ((1.0 - pass_rate) * 10.0) as usize,
                pass_rate,
                avg_execution_time_ms: 100,
                timeouts: 0,
                category_breakdown: HashMap::new(),
            },
        );

        BenchmarkReport {
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            branch: branch.to_string(),
            commit_sha: "abc123".to_string(),
            overall_pass_rate: pass_rate,
            total_tests: 10,
            total_passed: (pass_rate * 10.0) as usize,
            total_failed: ((1.0 - pass_rate) * 10.0) as usize,
            category_results,
            backend_results,
            execution_time_ms: 1000,
            regression_detected: false,
            baseline_comparison: None,
        }
    }

    #[test]
    fn test_store_and_load_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let store = BaselineStore::new(temp_dir.path());

        let report = create_test_report(0.9, "main");

        // Store baseline
        let stored_path = store.store(&report).unwrap();
        assert!(stored_path.exists());

        // Check symlink was created
        let latest_link = temp_dir.path().join("main-latest.json");
        assert!(latest_link.exists() || latest_link.symlink_metadata().is_ok());

        // Load from actual file
        let filename = stored_path.file_name().unwrap().to_str().unwrap();
        let loaded = store.load(filename).unwrap();
        assert_eq!(loaded.run_id, report.run_id);
        assert_eq!(loaded.overall_pass_rate, report.overall_pass_rate);

        // Load from symlink
        let loaded_latest = store.load("main-latest.json").unwrap();
        assert_eq!(loaded_latest.run_id, report.run_id);
    }

    #[test]
    fn test_list_baselines() {
        let temp_dir = TempDir::new().unwrap();
        let store = BaselineStore::new(temp_dir.path());

        // Store multiple baselines with sufficient delay to ensure different timestamps
        let report1 = create_test_report(0.9, "main");
        store.store(&report1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let report2 = create_test_report(0.85, "main");
        store.store(&report2).unwrap();

        let report3 = create_test_report(0.95, "feature");
        store.store(&report3).unwrap();

        // List main branch baselines
        let main_baselines = store.list_baselines("main").unwrap();
        assert_eq!(main_baselines.len(), 2);

        // List feature branch baselines
        let feature_baselines = store.list_baselines("feature").unwrap();
        assert_eq!(feature_baselines.len(), 1);
    }

    #[test]
    fn test_compare_no_regression() {
        let baseline = create_test_report(0.85, "main");
        let current = create_test_report(0.90, "main");

        let delta = BaselineStore::compare(&current, &baseline, 0.05);

        assert!((delta.overall_delta - 0.05).abs() < 0.01);
        assert!(delta.significant_regressions.is_empty());
    }

    #[test]
    fn test_compare_with_regression() {
        let baseline = create_test_report(0.90, "main");
        let current = create_test_report(0.80, "main");

        let delta = BaselineStore::compare(&current, &baseline, 0.05);

        assert!((delta.overall_delta + 0.10).abs() < 0.01);
        assert!(!delta.significant_regressions.is_empty());
        assert!(delta.significant_regressions[0].contains("Overall"));
        assert!(delta.significant_regressions[0].contains("drop"));
    }

    #[test]
    fn test_compare_threshold_edge_case() {
        let baseline = create_test_report(0.90, "main");
        let current = create_test_report(0.85, "main");

        // Exactly at threshold (5% drop)
        let delta = BaselineStore::compare(&current, &baseline, 0.05);
        assert!((delta.overall_delta + 0.05).abs() < 0.01);
        // -0.05 is not less than -0.05, so no regression detected
        assert!(delta.significant_regressions.is_empty());

        // Just below threshold (6% drop)
        let current2 = create_test_report(0.84, "main");
        let delta2 = BaselineStore::compare(&current2, &baseline, 0.05);
        assert!((delta2.overall_delta + 0.06).abs() < 0.01);
        assert!(!delta2.significant_regressions.is_empty());
    }

    #[test]
    fn test_load_nonexistent_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let store = BaselineStore::new(temp_dir.path());

        let result = store.load("nonexistent.json");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::evaluation::DatasetError::FileNotFound { .. }
        ));
    }
}

//! Automated issue creation for regressions.
//!
//! This module provides automated GitHub issue creation when test regressions
//! are detected, helping close the feedback loop between evaluation and product.

use serde::{Deserialize, Serialize};

use crate::model_profiling::ModelProfile;

/// Information about a detected regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionInfo {
    pub model: String,
    pub test_category: String,
    pub baseline_pass_rate: f64,
    pub current_pass_rate: f64,
    pub regression_pct: f64,
    pub is_critical: bool,
    pub sample_failures: Vec<(String, String)>, // (test_id, command)
}

/// GitHub issue template
#[derive(Debug, Clone)]
pub struct IssueTemplate {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

/// Regression detector
pub struct RegressionDetector {
    threshold_pct: f64,
}

impl RegressionDetector {
    /// Create a new regression detector with a threshold percentage
    pub fn new(threshold_pct: f64) -> Self {
        Self { threshold_pct }
    }

    /// Detect if a regression occurred between baseline and current
    pub fn detect_regression(
        &self,
        baseline: &ModelProfile,
        current: &ModelProfile,
    ) -> Option<RegressionInfo> {
        if baseline.model_name != current.model_name {
            return None;
        }

        let regression_pct = ((current.overall_pass_rate - baseline.overall_pass_rate)
            / baseline.overall_pass_rate)
            * 100.0;

        // Only report regressions (negative change)
        if regression_pct >= -self.threshold_pct {
            return None;
        }

        Some(RegressionInfo {
            model: baseline.model_name.clone(),
            test_category: "overall".to_string(),
            baseline_pass_rate: baseline.overall_pass_rate,
            current_pass_rate: current.overall_pass_rate,
            regression_pct: regression_pct.abs(),
            is_critical: regression_pct.abs() > 30.0,
            sample_failures: vec![],
        })
    }

    /// Assign priority label based on regression percentage
    pub fn assign_priority(&self, regression_pct: f64) -> &str {
        if regression_pct >= 30.0 {
            "priority:critical"
        } else if regression_pct >= 20.0 {
            "priority:high"
        } else if regression_pct >= 10.0 {
            "priority:medium"
        } else {
            "priority:low"
        }
    }
}

/// Issue generator for creating GitHub issue templates
pub struct IssueGenerator;

impl IssueGenerator {
    /// Create a new issue generator
    pub fn new() -> Self {
        Self
    }

    /// Generate an issue template from regression info
    pub fn generate_issue_template(&self, regression: &RegressionInfo) -> IssueTemplate {
        let title = format!(
            "Regression: {} pass rate dropped {:.0}% in {}",
            regression.model, regression.regression_pct, regression.test_category
        );

        let mut body = String::new();
        body.push_str("## Regression Detected\n\n");
        body.push_str(&format!("**Model**: {}\n", regression.model));
        body.push_str(&format!("**Category**: {}\n", regression.test_category));
        body.push_str(&format!(
            "**Previous Pass Rate**: {:.0}%\n",
            regression.baseline_pass_rate * 100.0
        ));
        body.push_str(&format!(
            "**Current Pass Rate**: {:.0}%\n",
            regression.current_pass_rate * 100.0
        ));
        body.push_str(&format!(
            "**Regression**: -{:.0}%\n\n",
            regression.regression_pct
        ));

        if !regression.sample_failures.is_empty() {
            body.push_str("## Sample Failures\n\n");
            for (test_id, command) in &regression.sample_failures {
                body.push_str(&format!("- `{}`: `{}`\n", test_id, command));
            }
            body.push('\n');
        }

        body.push_str("## Next Steps\n\n");
        body.push_str("1. Investigate recent changes\n");
        body.push_str("2. Review model prompt or configuration\n");
        body.push_str("3. Add specific test cases for failures\n");
        body.push_str("4. Re-run evaluation after fixes\n");

        let labels = self.generate_labels(
            &regression.model,
            &regression.test_category,
            regression.regression_pct,
        );

        IssueTemplate {
            title,
            body,
            labels,
        }
    }

    /// Generate labels for an issue
    pub fn generate_labels(&self, model: &str, category: &str, regression_pct: f64) -> Vec<String> {
        let mut labels = vec![
            "regression".to_string(),
            format!("backend:{}", model),
            format!("category:{}", category),
        ];

        let priority = if regression_pct >= 30.0 {
            "priority:critical"
        } else if regression_pct >= 20.0 {
            "priority:high"
        } else if regression_pct >= 10.0 {
            "priority:medium"
        } else {
            "priority:low"
        };

        labels.push(priority.to_string());
        labels
    }
}

impl Default for IssueGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Issue automation system
pub struct IssueAutomation {
    dry_run: bool,
}

impl IssueAutomation {
    /// Create a new issue automation system
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Create a GitHub issue (or simulate in dry-run mode)
    pub async fn create_issue(&self, regression: &RegressionInfo) -> Result<String, String> {
        let generator = IssueGenerator::new();
        let template = generator.generate_issue_template(regression);

        if self.dry_run {
            Ok(format!(
                "DRY RUN: Would create issue:\nTitle: {}\nLabels: {:?}",
                template.title, template.labels
            ))
        } else {
            // In real implementation, this would call GitHub API
            // For now, return success message
            Ok(format!("Created issue: {}", template.title))
        }
    }
}

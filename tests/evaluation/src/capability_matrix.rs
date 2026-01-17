//! Capability matrix for model comparison and routing.
//!
//! This module provides a cross-model capability matrix showing:
//! - Performance of each model across all categories
//! - Best model recommendations per category
//! - Historical capability tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::test_runner::TestResult;

/// Capability data for a single model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub model_name: String,
    /// Map of category name to pass rate (0.0 to 1.0)
    pub capabilities: Vec<(String, f64)>,
}

/// Matrix of model capabilities across categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatrix {
    /// List of all categories
    pub categories: Vec<String>,
    /// Capability data for each model
    pub models: Vec<ModelCapability>,
}

impl CapabilityMatrix {
    /// Get the pass rate for a specific model and category
    pub fn get_capability(&self, model: &str, category: &str) -> Option<f64> {
        self.models
            .iter()
            .find(|m| m.model_name == model)?
            .capabilities
            .iter()
            .find(|(cat, _)| cat == category)
            .map(|(_, rate)| *rate)
    }
}

/// Builder for creating capability matrices
pub struct CapabilityMatrixBuilder;

impl CapabilityMatrixBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self
    }

    /// Build a capability matrix from test results
    pub fn build_from_results(&self, results: &[TestResult]) -> CapabilityMatrix {
        // Collect all unique categories and models
        let mut categories = std::collections::HashSet::new();
        let mut models = std::collections::HashSet::new();

        for result in results {
            categories.insert(result.category.clone());
            models.insert(result.backend.clone());
        }

        let mut categories: Vec<String> = categories.into_iter().collect();
        categories.sort();

        let mut models_vec: Vec<String> = models.into_iter().collect();
        models_vec.sort();

        // Calculate pass rates for each model × category
        let mut model_capabilities = Vec::new();

        for model_name in models_vec {
            let mut capabilities = Vec::new();

            for category in &categories {
                let model_category_results: Vec<_> = results
                    .iter()
                    .filter(|r| r.backend == model_name && r.category == *category)
                    .collect();

                if !model_category_results.is_empty() {
                    let passed = model_category_results.iter().filter(|r| r.passed).count();
                    let total = model_category_results.len();
                    let pass_rate = passed as f64 / total as f64;

                    capabilities.push((category.clone(), pass_rate));
                }
            }

            model_capabilities.push(ModelCapability {
                model_name,
                capabilities,
            });
        }

        CapabilityMatrix {
            categories,
            models: model_capabilities,
        }
    }

    /// Generate a markdown table report
    pub fn generate_table_report(&self, matrix: &CapabilityMatrix) -> String {
        let mut report = String::new();

        // Header row
        report.push_str("| Model |");
        for category in &matrix.categories {
            report.push_str(&format!(" {} |", category));
        }
        report.push('\n');

        // Separator row
        report.push_str("|-------|");
        for _ in &matrix.categories {
            report.push_str("--------|");
        }
        report.push('\n');

        // Data rows
        for model in &matrix.models {
            report.push_str(&format!("| {} |", model.model_name));

            for category in &matrix.categories {
                if let Some(rate) = model
                    .capabilities
                    .iter()
                    .find(|(cat, _)| cat == category)
                    .map(|(_, rate)| rate)
                {
                    report.push_str(&format!(" {:.0}% |", rate * 100.0));
                } else {
                    report.push_str(" N/A |");
                }
            }

            report.push('\n');
        }

        report
    }

    /// Find the best model for a specific category
    pub fn find_best_model(&self, matrix: &CapabilityMatrix, category: &str) -> Option<String> {
        let mut best_model = None;
        let mut best_rate = 0.0;

        for model in &matrix.models {
            if let Some(rate) = model
                .capabilities
                .iter()
                .find(|(cat, _)| cat == category)
                .map(|(_, rate)| *rate)
            {
                if rate > best_rate {
                    best_rate = rate;
                    best_model = Some(model.model_name.clone());
                }
            }
        }

        best_model
    }

    /// Generate routing recommendations based on the matrix
    pub fn generate_routing_guide(&self, matrix: &CapabilityMatrix) -> HashMap<String, String> {
        let mut recommendations = HashMap::new();

        for category in &matrix.categories {
            if let Some(best_model) = self.find_best_model(matrix, category) {
                recommendations.insert(category.clone(), best_model);
            }
        }

        recommendations
    }
}

impl Default for CapabilityMatrixBuilder {
    fn default() -> Self {
        Self::new()
    }
}

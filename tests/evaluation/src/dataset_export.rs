//! Training dataset export for fine-tuning.
//!
//! This module exports evaluation results in various formats suitable
//! for fine-tuning: OpenAI JSONL, preference pairs (RLHF), etc.

use serde_json::json;

use crate::test_runner::TestResult;

/// Export format for training data
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// OpenAI fine-tuning format (JSONL with messages)
    OpenAI,
    /// Preference pairs for RLHF (DPO/PPO)
    PreferencePairs,
}

/// Filter options for export
#[derive(Debug, Clone)]
pub struct FilterOptions {
    /// Only export passing tests
    pub only_passing: bool,
    /// Only export failing tests
    pub only_failing: bool,
    /// Minimum confidence threshold (if available)
    pub min_confidence: Option<f64>,
}

/// Dataset exporter
pub struct DatasetExporter {
    system_prompt: String,
}

impl DatasetExporter {
    /// Create a new dataset exporter
    pub fn new() -> Self {
        Self {
            system_prompt: "You are a helpful assistant that generates shell commands from natural language descriptions.".to_string(),
        }
    }

    /// Export test results to the specified format
    pub fn export(
        &self,
        results: &[TestResult],
        format: ExportFormat,
        filter: Option<FilterOptions>,
    ) -> Result<String, String> {
        // Apply filters
        let filtered = self.filter_results(results, filter);

        match format {
            ExportFormat::OpenAI => self.export_openai(&filtered),
            ExportFormat::PreferencePairs => self.export_preference_pairs(&filtered),
        }
    }

    /// Filter results based on options
    fn filter_results(
        &self,
        results: &[TestResult],
        filter: Option<FilterOptions>,
    ) -> Vec<TestResult> {
        let Some(filter) = filter else {
            return results.to_vec();
        };

        results
            .iter()
            .filter(|r| {
                if filter.only_passing && !r.passed {
                    return false;
                }
                if filter.only_failing && r.passed {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Export to OpenAI fine-tuning format (JSONL)
    fn export_openai(&self, results: &[TestResult]) -> Result<String, String> {
        let mut lines = Vec::new();

        for result in results {
            // Infer user prompt from test_id (e.g., "test_001" -> "find files")
            let user_prompt = self.infer_user_prompt(&result.test_id);

            // Always use expected_output as the correct answer for training
            let command = &result.expected_output;

            let entry = json!({
                "messages": [
                    {
                        "role": "system",
                        "content": self.system_prompt
                    },
                    {
                        "role": "user",
                        "content": user_prompt
                    },
                    {
                        "role": "assistant",
                        "content": command
                    }
                ]
            });

            lines.push(serde_json::to_string(&entry).map_err(|e| e.to_string())?);
        }

        Ok(lines.join("\n"))
    }

    /// Export as preference pairs for RLHF
    fn export_preference_pairs(&self, results: &[TestResult]) -> Result<String, String> {
        let mut lines = Vec::new();

        // Group by test_id to find passing/failing pairs
        let mut test_groups: std::collections::HashMap<String, Vec<&TestResult>> =
            std::collections::HashMap::new();

        for result in results {
            test_groups
                .entry(result.test_id.clone())
                .or_default()
                .push(result);
        }

        // Create preference pairs from groups that have both pass and fail
        for (test_id, group) in test_groups {
            let passing = group.iter().find(|r| r.passed);
            let failing = group.iter().find(|r| !r.passed);

            if let (Some(pass), Some(fail)) = (passing, failing) {
                let user_prompt = self.infer_user_prompt(&test_id);

                let entry = json!({
                    "prompt": user_prompt,
                    "chosen": pass.expected_output,
                    "rejected": fail.actual_output
                });

                lines.push(serde_json::to_string(&entry).map_err(|e| e.to_string())?);
            }
        }

        Ok(lines.join("\n"))
    }

    /// Infer user prompt from test ID
    fn infer_user_prompt(&self, test_id: &str) -> String {
        // Simplified prompt inference - in production, this would come from test metadata
        format!("Generate command for test case {}", test_id)
    }

    /// Validate OpenAI format JSON
    pub fn validate_openai_format(&self, json_str: &str) -> bool {
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) else {
            return false;
        };

        // Must have "messages" field
        let Some(messages) = parsed.get("messages") else {
            return false;
        };

        // Messages must be an array
        let Some(messages_array) = messages.as_array() else {
            return false;
        };

        // Must have at least 2 messages (system + user, or user + assistant)
        if messages_array.len() < 2 {
            return false;
        }

        // Each message must have "role" and "content"
        for msg in messages_array {
            if msg.get("role").is_none() || msg.get("content").is_none() {
                return false;
            }
        }

        true
    }
}

impl Default for DatasetExporter {
    fn default() -> Self {
        Self::new()
    }
}

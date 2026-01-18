//! Token usage tracking and cost analysis.

use std::collections::HashMap;
use std::sync::Mutex;

/// Token usage for a single evaluation
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub backend: String,
    pub test_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl TokenUsage {
    pub fn calculate_cost(&self) -> f64 {
        match self.backend.as_str() {
            "smollm" | "mlx" | "qwen" | "ollama" | "static_matcher" => 0.0,
            "openai" => {
                let input_cost = self.input_tokens as f64 * 0.01 / 1000.0;
                let output_cost = self.output_tokens as f64 * 0.03 / 1000.0;
                input_cost + output_cost
            }
            _ => 0.0,
        }
    }

    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

pub struct TokenTracker {
    usages: Mutex<Vec<TokenUsage>>,
}

impl TokenTracker {
    pub fn new() -> Self {
        Self {
            usages: Mutex::new(Vec::new()),
        }
    }

    pub fn track(&self, usage: &TokenUsage) {
        let mut usages = self.usages.lock().unwrap();
        usages.push(usage.clone());
    }

    pub fn get_total_tokens(&self, backend: &str) -> u32 {
        let usages = self.usages.lock().unwrap();
        usages
            .iter()
            .filter(|u| u.backend == backend)
            .map(|u| u.total_tokens())
            .sum()
    }
}

impl Default for TokenTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CostAnalyzer;

impl CostAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(&self, usages: &[TokenUsage]) -> String {
        let mut report = String::new();
        report.push_str("# Token Usage & Cost Analysis\n\n");

        let mut backend_stats: HashMap<String, (usize, u32, f64)> = HashMap::new();

        for usage in usages {
            let stats = backend_stats
                .entry(usage.backend.clone())
                .or_insert((0, 0, 0.0));
            stats.0 += 1;
            stats.1 += usage.total_tokens();
            stats.2 += usage.calculate_cost();
        }

        report.push_str("| Backend | Tests | Tokens | Cost |\n");
        report.push_str("|---------|-------|--------|------|\n");

        for (backend, (count, tokens, cost)) in backend_stats {
            report.push_str(&format!(
                "| {} | {} | {} | ${:.2} |\n",
                backend, count, tokens, cost
            ));
        }

        report
    }
}

impl Default for CostAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TokenAnalyzer;

impl TokenAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn find_heavy_tests(&self, usages: &[TokenUsage], threshold: u32) -> Vec<TokenUsage> {
        usages
            .iter()
            .filter(|u| u.total_tokens() >= threshold)
            .cloned()
            .collect()
    }
}

impl Default for TokenAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PromptOptimizer;

impl PromptOptimizer {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize(&self, prompt: &str) -> String {
        let mut optimized = prompt.split_whitespace().collect::<Vec<_>>().join(" ");

        let fillers = ["please", "kindly", "just", "simply"];
        for filler in &fillers {
            optimized = optimized.replace(&format!("{} ", filler), "");
        }

        optimized = optimized.replace("in the current directory", "here");
        optimized = optimized.replace("and all subdirectories", "recursively");
        optimized = optimized.replace("that have the", "with");
        optimized = optimized.replace("and then display them", "");

        while optimized.contains("  ") {
            optimized = optimized.replace("  ", " ");
        }

        optimized.trim().to_string()
    }
}

impl Default for PromptOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

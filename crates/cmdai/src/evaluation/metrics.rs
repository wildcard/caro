// Accuracy metrics and scoring for command evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::models::ShellType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyScore {
    pub exact_match: bool,
    pub semantic_match: bool,
    pub functional_score: f64,
    pub safety_score: f64,
    pub posix_compliance: bool,
    pub overall_score: f64,
}

impl AccuracyScore {
    /// Calculate overall score from individual components
    pub fn calculate(exact_match: bool, semantic_match: bool, functional_score: f64, safety_score: f64, posix_compliance: bool) -> Self {
        // Weighted scoring:
        // - Exact match: 40% weight
        // - Semantic match: 30% weight 
        // - Functional correctness: 20% weight
        // - Safety: 10% weight (but critical - can veto)
        // - POSIX compliance: bonus points
        
        let mut overall_score = 0.0;
        
        if exact_match {
            overall_score += 0.4;
        } else if semantic_match {
            overall_score += 0.3;
        }
        
        overall_score += functional_score * 0.2;
        overall_score += safety_score * 0.1;
        
        if posix_compliance {
            overall_score += 0.05; // Bonus
        }
        
        // Safety veto: if safety score is too low, cap overall score
        if safety_score < 0.5 {
            overall_score = overall_score.min(0.3);
        }
        
        AccuracyScore {
            exact_match,
            semantic_match,
            functional_score,
            safety_score,
            posix_compliance,
            overall_score: overall_score.min(1.0),
        }
    }
    
    /// Check if this is considered a passing score
    pub fn is_passing(&self) -> bool {
        self.overall_score >= 0.7 && self.safety_score >= 0.8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub inference_time: Duration,
    pub memory_usage_mb: f64,
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_case_id: String,
    pub generated_command: String,
    pub accuracy_score: AccuracyScore,
    pub performance_metrics: PerformanceMetrics,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub overall_accuracy: f64,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub category_scores: HashMap<String, f64>,
    pub shell_scores: HashMap<ShellType, f64>,
    pub difficulty_scores: HashMap<String, f64>,
    pub avg_inference_time: Duration,
    pub test_results: Vec<TestCaseResult>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            overall_accuracy: 0.0,
            total_cases: 0,
            passed_cases: 0,
            failed_cases: 0,
            category_scores: HashMap::new(),
            shell_scores: HashMap::new(),
            difficulty_scores: HashMap::new(),
            avg_inference_time: Duration::from_secs(0),
            test_results: Vec::new(),
        }
    }
    
    /// Calculate final metrics from individual test results
    pub fn finalize(&mut self, test_cases: &[crate::evaluation::TestCase]) {
        self.total_cases = self.test_results.len();
        
        // Calculate overall accuracy
        let total_score: f64 = self.test_results
            .iter()
            .map(|r| r.accuracy_score.overall_score)
            .sum();
        self.overall_accuracy = if self.total_cases > 0 {
            total_score / self.total_cases as f64
        } else {
            0.0
        };
        
        // Count passed/failed
        self.passed_cases = self.test_results
            .iter()
            .filter(|r| r.accuracy_score.is_passing())
            .count();
        self.failed_cases = self.total_cases - self.passed_cases;
        
        // Calculate category scores
        for test_case in test_cases {
            let results: Vec<_> = self.test_results
                .iter()
                .filter(|r| r.test_case_id == test_case.id)
                .collect();
            
            if !results.is_empty() {
                let avg_score = results
                    .iter()
                    .map(|r| r.accuracy_score.overall_score)
                    .sum::<f64>() / results.len() as f64;
                self.category_scores.insert(test_case.category.clone(), avg_score);
                
                // Update shell scores
                let shell_entry = self.shell_scores.entry(test_case.shell).or_insert(0.0);
                *shell_entry = (*shell_entry + avg_score) / 2.0; // Running average
                
                // Update difficulty scores
                let difficulty_key = format!("{:?}", test_case.difficulty);
                let difficulty_entry = self.difficulty_scores.entry(difficulty_key).or_insert(0.0);
                *difficulty_entry = (*difficulty_entry + avg_score) / 2.0; // Running average
            }
        }
        
        // Calculate average inference time
        if !self.test_results.is_empty() {
            let total_time: Duration = self.test_results
                .iter()
                .map(|r| r.performance_metrics.inference_time)
                .sum();
            self.avg_inference_time = total_time / self.test_results.len() as u32;
        }
    }
    
    /// Generate a summary report
    pub fn summary(&self) -> String {
        format!(
            "Evaluation Summary:\n\
            Overall Accuracy: {:.2}%\n\
            Passed: {}/{} ({:.1}%)\n\
            Failed: {}/{} ({:.1}%)\n\
            Average Inference Time: {:.2}s\n",
            self.overall_accuracy * 100.0,
            self.passed_cases,
            self.total_cases,
            if self.total_cases > 0 { self.passed_cases as f64 / self.total_cases as f64 * 100.0 } else { 0.0 },
            self.failed_cases,
            self.total_cases,
            if self.total_cases > 0 { self.failed_cases as f64 / self.total_cases as f64 * 100.0 } else { 0.0 },
            self.avg_inference_time.as_secs_f64()
        )
    }
    
    /// Get detailed breakdown by category
    pub fn category_breakdown(&self) -> String {
        let mut output = String::from("Category Breakdown:\n");
        for (category, score) in &self.category_scores {
            output.push_str(&format!("  {}: {:.2}%\n", category, score * 100.0));
        }
        output
    }
    
    /// Get failed test cases for review
    pub fn failed_tests(&self) -> Vec<&TestCaseResult> {
        self.test_results
            .iter()
            .filter(|r| !r.accuracy_score.is_passing())
            .collect()
    }
}
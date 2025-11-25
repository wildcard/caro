// Evaluation runner for batch testing and reporting

use std::path::Path;
use anyhow::Result;
use colored::Colorize;

use crate::evaluation::{TestDataset, EvaluationEngine, EvaluationResult, TestCase};
use crate::models::ShellType;

pub struct EvaluationRunner {
    engine: EvaluationEngine,
}

impl EvaluationRunner {
    /// Create a new evaluation runner with default configuration
    pub async fn new() -> Result<Self> {
        let engine = EvaluationEngine::new().await?;
        Ok(EvaluationRunner { engine })
    }
    
    /// Create evaluation runner with specific shell
    pub async fn with_shell(shell: ShellType) -> Result<Self> {
        let engine = EvaluationEngine::with_shell(shell).await?;
        Ok(EvaluationRunner { engine })
    }
    
    /// Run evaluation on a dataset from file
    pub async fn run_evaluation_from_file<P: AsRef<Path>>(&self, dataset_path: P) -> Result<EvaluationResult> {
        let dataset = TestDataset::load_from_yaml(dataset_path)?;
        self.run_evaluation(&dataset).await
    }
    
    /// Run evaluation on a dataset from directory
    pub async fn run_evaluation_from_directory<P: AsRef<Path>>(&self, dataset_dir: P) -> Result<EvaluationResult> {
        let dataset = TestDataset::load_from_directory(dataset_dir)?;
        self.run_evaluation(&dataset).await
    }
    
    /// Run evaluation on a dataset
    pub async fn run_evaluation(&self, dataset: &TestDataset) -> Result<EvaluationResult> {
        let mut result = EvaluationResult::new();
        
        println!("🚀 Starting evaluation with {} test cases", dataset.test_cases.len());
        
        for (i, test_case) in dataset.test_cases.iter().enumerate() {
            print!("Testing {}/{}: {} ... ", i + 1, dataset.test_cases.len(), test_case.id);
            
            let test_result = self.engine.evaluate_test_case(test_case).await;
            
            // Print result status
            if test_result.accuracy_score.is_passing() {
                println!("{}", "✅ PASS".green());
            } else {
                println!("{}", "❌ FAIL".red());
                if let Some(ref error) = test_result.error {
                    if let Some(ref error) = test_result.error {
                        println!("    Error: {}", error.red());
                    }
                } else {
                    println!("    Expected: {:?}", test_case.expected_commands);
                    println!("    Got: {}", test_result.generated_command);
                    println!("    Score: {:.2}", test_result.accuracy_score.overall_score);
                }
            }
            
            result.test_results.push(test_result);
        }
        
        // Finalize results
        result.finalize(&dataset.test_cases);
        
        println!("\n{}", "📊 Evaluation Complete".bold());
        println!("{}", result.summary());
        
        Ok(result)
    }
    
    /// Run quick evaluation on specific test cases
    pub async fn run_quick_test(&self, test_cases: Vec<(&str, &str)>) -> Result<()> {
        println!("🔍 Running quick test on {} cases", test_cases.len());
        
        for (i, (input, expected)) in test_cases.iter().enumerate() {
            print!("Test {}: '{}' ... ", i + 1, input);
            
            let test_case = TestCase {
                id: format!("quick_test_{}", i + 1),
                category: "quick_test".to_string(),
                subcategory: "manual".to_string(),
                shell: ShellType::Bash,
                difficulty: crate::evaluation::DifficultyLevel::Basic,
                input: input.to_string(),
                expected_commands: vec![expected.to_string()],
                explanation: "Quick test case".to_string(),
                tags: vec!["quick".to_string()],
                safety_level: crate::evaluation::SafetyLevel::Safe,
            };
            
            let result = self.engine.evaluate_test_case(&test_case).await;
            
            if result.accuracy_score.exact_match {
                println!("{}", "✅ EXACT MATCH".green());
            } else if result.accuracy_score.semantic_match {
                println!("{}", "✅ SEMANTIC MATCH".yellow());
            } else {
                println!("{}", "❌ NO MATCH".red());
                println!("    Expected: {}", expected);
                println!("    Got: {}", result.generated_command);
                println!("    Score: {:.2}", result.accuracy_score.overall_score);
            }
        }
        
        Ok(())
    }
    
    /// Generate detailed report
    pub fn generate_report(&self, result: &EvaluationResult) -> String {
        let mut report = String::new();
        
        report.push_str("# Command Accuracy Evaluation Report\n\n");
        
        // Summary
        report.push_str("## Summary\n");
        report.push_str(&result.summary());
        report.push('\n');
        
        // Category breakdown
        report.push_str("## Category Breakdown\n");
        report.push_str(&result.category_breakdown());
        report.push('\n');
        
        // Shell performance
        report.push_str("## Shell Performance\n");
        for (shell, score) in &result.shell_scores {
            report.push_str(&format!("- {:?}: {:.2}%\n", shell, score * 100.0));
        }
        report.push('\n');
        
        // Failed tests
        let failed_tests = result.failed_tests();
        if !failed_tests.is_empty() {
            report.push_str(&format!("## Failed Tests ({})\n", failed_tests.len()));
            for test in failed_tests {
                report.push_str(&format!(
                    "### {}\n- Generated: `{}`\n- Score: {:.2}\n- Error: {:?}\n\n",
                    test.test_case_id,
                    test.generated_command,
                    test.accuracy_score.overall_score,
                    test.error
                ));
            }
        }
        
        report
    }
    
    /// Save evaluation result to file
    pub fn save_result<P: AsRef<Path>>(&self, result: &EvaluationResult, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(result)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
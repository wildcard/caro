//! Test case definitions and implementations
//!
//! Provides a standardized interface for test cases and a comprehensive
//! library of scenarios for validating cmdai safety and performance.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::models::{RiskLevel, ShellType};
use crate::safety::advanced::{AdvancedSafetyValidator, AdvancedValidationResult, UserFeedback};

/// Categories of test cases for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    BasicSafety,
    DangerousCommands,
    EdgeCases,
    AdaptiveLearning,
    PerformanceBenchmarks,
    IntegrationTests,
    CustomUser,
}

impl TestCategory {
    pub fn description(&self) -> &'static str {
        match self {
            TestCategory::BasicSafety => "Commands that should pass safety validation",
            TestCategory::DangerousCommands => "Commands that should be blocked or flagged",
            TestCategory::EdgeCases => "Boundary conditions and unusual patterns",
            TestCategory::AdaptiveLearning => "User feedback and learning scenarios",
            TestCategory::PerformanceBenchmarks => "Speed and reliability validation",
            TestCategory::IntegrationTests => "End-to-end workflow testing",
            TestCategory::CustomUser => "User-defined test scenarios",
        }
    }

    pub fn all_categories() -> Vec<TestCategory> {
        vec![
            TestCategory::BasicSafety,
            TestCategory::DangerousCommands,
            TestCategory::EdgeCases,
            TestCategory::AdaptiveLearning,
            TestCategory::PerformanceBenchmarks,
            TestCategory::IntegrationTests,
            TestCategory::CustomUser,
        ]
    }
}

/// Expected outcome of a test case
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestOutcome {
    ShouldPass {
        max_risk_level: RiskLevel,
        should_require_confirmation: bool,
    },
    ShouldBlock {
        expected_risk_level: RiskLevel,
        expected_patterns: Vec<String>,
    },
    ShouldLearn {
        feedback: UserFeedback,
        expected_recommendation_change: String,
    },
    PerformanceTarget {
        max_response_time_ms: u64,
        min_confidence: f64,
    },
}

/// Result of executing a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub category: TestCategory,
    pub command: String,
    pub shell: ShellType,
    pub execution_time: Duration,
    pub validation_result: AdvancedValidationResult,
    pub expected_outcome: TestOutcome,
    pub passed: bool,
    pub failure_reason: Option<String>,
    pub performance_metrics: HashMap<String, f64>,
    pub logs: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for test cases that can be executed
#[async_trait]
pub trait TestCase: Send + Sync {
    /// Unique identifier for the test case
    fn name(&self) -> &str;
    
    /// Category this test belongs to
    fn category(&self) -> TestCategory;
    
    /// Human-readable description of what this test validates
    fn description(&self) -> &str;
    
    /// Difficulty level (1=easy, 5=complex)
    fn difficulty(&self) -> u8;
    
    /// Command to test
    fn command(&self) -> &str;
    
    /// Shell type for execution
    fn shell(&self) -> ShellType;
    
    /// Expected outcome of the test
    fn expected_outcome(&self) -> &TestOutcome;
    
    /// Execute the test case against the validator
    async fn execute(&self, validator: &AdvancedSafetyValidator) -> TestResult;
    
    /// Validate the result matches expectations
    fn validate_result(&self, result: &AdvancedValidationResult) -> (bool, Option<String>);
    
    /// Setup required before test execution (optional)
    async fn setup(&self, _validator: &AdvancedSafetyValidator) -> Result<(), String> {
        Ok(())
    }
    
    /// Cleanup after test execution (optional)
    async fn cleanup(&self, _validator: &AdvancedSafetyValidator) -> Result<(), String> {
        Ok(())
    }
}

/// Basic implementation of a test case
#[derive(Debug, Clone)]
pub struct BasicTestCase {
    pub name: String,
    pub category: TestCategory,
    pub description: String,
    pub difficulty: u8,
    pub command: String,
    pub shell: ShellType,
    pub expected_outcome: TestOutcome,
}

#[async_trait]
impl TestCase for BasicTestCase {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn category(&self) -> TestCategory {
        self.category.clone()
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn difficulty(&self) -> u8 {
        self.difficulty
    }
    
    fn command(&self) -> &str {
        &self.command
    }
    
    fn shell(&self) -> ShellType {
        self.shell
    }
    
    fn expected_outcome(&self) -> &TestOutcome {
        &self.expected_outcome
    }
    
    async fn execute(&self, validator: &AdvancedSafetyValidator) -> TestResult {
        let start_time = Instant::now();
        let mut logs = Vec::new();
        
        logs.push(format!("Starting test: {}", self.name()));
        logs.push(format!("Command: '{}'", self.command()));
        logs.push(format!("Shell: {:?}", self.shell()));
        
        // Execute validation
        let validation_result = match validator.analyze_command(self.command(), self.shell(), None).await {
            Ok(result) => {
                logs.push(format!("Validation completed successfully"));
                logs.push(format!("Threat level: {:?}", result.threat_level));
                logs.push(format!("Recommendations: {:?}", result.recommendations));
                result
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                logs.push(error_msg.clone());
                
                // Create a default result for error cases
                AdvancedValidationResult {
                    basic_result: crate::safety::ValidationResult {
                        allowed: false,
                        risk_level: RiskLevel::Critical,
                        explanation: error_msg,
                        warnings: vec![],
                        matched_patterns: vec![],
                        confidence_score: 0.0,
                    },
                    threat_level: crate::safety::advanced::ThreatLevel::Critical,
                    behavioral_patterns: vec![],
                    contextual_warnings: vec![],
                    behavioral_warnings: vec![],
                    ml_scores: HashMap::new(),
                    recommendations: vec![],
                    requires_monitoring: false,
                    analysis_time_ms: 0,
                }
            }
        };
        
        let execution_time = start_time.elapsed();
        let (passed, failure_reason) = self.validate_result(&validation_result);
        
        logs.push(format!("Test completed in {:?}", execution_time));
        logs.push(format!("Result: {}", if passed { "PASSED" } else { "FAILED" }));
        
        if let Some(ref reason) = failure_reason {
            logs.push(format!("Failure reason: {}", reason));
        }
        
        let mut performance_metrics = HashMap::new();
        performance_metrics.insert("execution_time_ms".to_string(), execution_time.as_millis() as f64);
        performance_metrics.insert("analysis_time_ms".to_string(), validation_result.analysis_time_ms as f64);
        performance_metrics.insert("confidence_score".to_string(), validation_result.basic_result.confidence_score as f64);
        
        TestResult {
            test_name: self.name().to_string(),
            category: self.category(),
            command: self.command().to_string(),
            shell: self.shell(),
            execution_time,
            validation_result,
            expected_outcome: self.expected_outcome().clone(),
            passed,
            failure_reason,
            performance_metrics,
            logs,
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn validate_result(&self, result: &AdvancedValidationResult) -> (bool, Option<String>) {
        match &self.expected_outcome {
            TestOutcome::ShouldPass { max_risk_level, should_require_confirmation } => {
                if !result.basic_result.allowed {
                    return (false, Some("Command was blocked but should have passed".to_string()));
                }
                
                if result.basic_result.risk_level > *max_risk_level {
                    return (false, Some(format!(
                        "Risk level {:?} exceeds maximum {:?}",
                        result.basic_result.risk_level, max_risk_level
                    )));
                }
                
                let requires_confirmation = result.requires_monitoring || 
                    result.recommendations.iter().any(|r| r.to_lowercase().contains("confirmation"));
                
                if requires_confirmation != *should_require_confirmation {
                    return (false, Some(format!(
                        "Confirmation requirement mismatch: expected {}, got {}",
                        should_require_confirmation, requires_confirmation
                    )));
                }
                
                (true, None)
            }
            
            TestOutcome::ShouldBlock { expected_risk_level, expected_patterns } => {
                if result.basic_result.allowed {
                    return (false, Some("Command was allowed but should have been blocked".to_string()));
                }
                
                if result.basic_result.risk_level < *expected_risk_level {
                    return (false, Some(format!(
                        "Risk level {:?} is lower than expected {:?}",
                        result.basic_result.risk_level, expected_risk_level
                    )));
                }
                
                for pattern in expected_patterns {
                    if !result.basic_result.matched_patterns.iter().any(|p| p.contains(pattern)) {
                        return (false, Some(format!(
                            "Expected pattern '{}' not found in matched patterns",
                            pattern
                        )));
                    }
                }
                
                (true, None)
            }
            
            TestOutcome::ShouldLearn { feedback: _, expected_recommendation_change } => {
                // For learning tests, we check if the expected recommendation is present
                let has_expected_recommendation = result.recommendations.iter()
                    .any(|r| r.to_lowercase().contains(&expected_recommendation_change.to_lowercase()));
                
                if !has_expected_recommendation {
                    return (false, Some(format!(
                        "Expected recommendation change '{}' not found",
                        expected_recommendation_change
                    )));
                }
                
                (true, None)
            }
            
            TestOutcome::PerformanceTarget { max_response_time_ms, min_confidence } => {
                if result.analysis_time_ms > *max_response_time_ms {
                    return (false, Some(format!(
                        "Analysis time {}ms exceeds target {}ms",
                        result.analysis_time_ms, max_response_time_ms
                    )));
                }
                
                if result.basic_result.confidence_score < *min_confidence as f32 {
                    return (false, Some(format!(
                        "Confidence score {} below minimum {}",
                        result.basic_result.confidence_score, min_confidence
                    )));
                }
                
                (true, None)
            }
        }
    }
}

/// Test case builder for fluent construction
pub struct TestCaseBuilder {
    name: String,
    category: TestCategory,
    description: String,
    difficulty: u8,
    command: String,
    shell: ShellType,
    expected_outcome: Option<TestOutcome>,
}

impl TestCaseBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            category: TestCategory::BasicSafety,
            description: String::new(),
            difficulty: 1,
            command: String::new(),
            shell: ShellType::Bash,
            expected_outcome: None,
        }
    }
    
    pub fn category(mut self, category: TestCategory) -> Self {
        self.category = category;
        self
    }
    
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty;
        self
    }
    
    pub fn command(mut self, command: &str) -> Self {
        self.command = command.to_string();
        self
    }
    
    pub fn shell(mut self, shell: ShellType) -> Self {
        self.shell = shell;
        self
    }
    
    pub fn should_pass(mut self, max_risk_level: RiskLevel, require_confirmation: bool) -> Self {
        self.expected_outcome = Some(TestOutcome::ShouldPass {
            max_risk_level,
            should_require_confirmation: require_confirmation,
        });
        self
    }
    
    pub fn should_block(mut self, expected_risk_level: RiskLevel, patterns: Vec<String>) -> Self {
        self.expected_outcome = Some(TestOutcome::ShouldBlock {
            expected_risk_level,
            expected_patterns: patterns,
        });
        self
    }
    
    pub fn should_learn(mut self, feedback: UserFeedback, expected_change: &str) -> Self {
        self.expected_outcome = Some(TestOutcome::ShouldLearn {
            feedback,
            expected_recommendation_change: expected_change.to_string(),
        });
        self
    }
    
    pub fn performance_target(mut self, max_time_ms: u64, min_confidence: f64) -> Self {
        self.expected_outcome = Some(TestOutcome::PerformanceTarget {
            max_response_time_ms: max_time_ms,
            min_confidence,
        });
        self
    }
    
    pub fn build(self) -> Result<BasicTestCase, String> {
        let expected_outcome = self.expected_outcome
            .ok_or("Expected outcome must be specified")?;
        
        if self.command.is_empty() {
            return Err("Command cannot be empty".to_string());
        }
        
        if self.description.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        
        Ok(BasicTestCase {
            name: self.name,
            category: self.category,
            description: self.description,
            difficulty: self.difficulty,
            command: self.command,
            shell: self.shell,
            expected_outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_builder() {
        let test_case = TestCaseBuilder::new("test_ls")
            .category(TestCategory::BasicSafety)
            .description("Test basic file listing")
            .command("ls -la")
            .should_pass(RiskLevel::Low, false)
            .build()
            .unwrap();

        assert_eq!(test_case.name(), "test_ls");
        assert_eq!(test_case.category(), TestCategory::BasicSafety);
        assert_eq!(test_case.command(), "ls -la");
    }

    #[test]
    fn test_category_descriptions() {
        for category in TestCategory::all_categories() {
            assert!(!category.description().is_empty());
        }
    }
}
//! Log analysis and pattern detection for test execution
//!
//! Provides intelligent analysis of test execution logs, performance bottlenecks,
//! and safety decision patterns to help users understand system behavior.

use colored::{ColoredString, Colorize};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::testing::cases::{TestResult, TestCategory};
use crate::testing::metrics::TestMetrics;

/// Log analysis results with structured insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysis {
    /// Key events detected in logs
    pub events: Vec<LogEvent>,
    /// Performance bottlenecks identified
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Safety decision patterns
    pub safety_patterns: Vec<SafetyPattern>,
    /// Warnings and recommendations
    pub warnings: Vec<String>,
    /// Overall health score (0-100)
    pub health_score: f64,
}

/// Important events extracted from logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub category: EventCategory,
    pub message: String,
    pub context: HashMap<String, String>,
}

/// Log levels for event classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Categories of events for analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventCategory {
    TestExecution,
    SafetyValidation,
    PerformanceMetric,
    ErrorCondition,
    UserInteraction,
    SystemResource,
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub location: String,
    pub severity: BottleneckSeverity,
    pub impact_description: String,
    pub suggested_fix: String,
    pub affected_tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Safety decision pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyPattern {
    pub pattern_type: SafetyPatternType,
    pub frequency: usize,
    pub accuracy: f64,
    pub examples: Vec<String>,
    pub recommended_tuning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SafetyPatternType {
    CorrectBlock,      // Dangerous command correctly blocked
    CorrectAllow,      // Safe command correctly allowed
    FalsePositive,     // Safe command incorrectly blocked
    FalseNegative,     // Dangerous command incorrectly allowed
    InconsistentRisk,  // Same command different risk levels
}

/// Main log analyzer for processing test execution data
pub struct LogAnalyzer {
    patterns: Vec<LogPatternMatcher>,
    performance_thresholds: PerformanceThresholds,
}

struct LogPatternMatcher {
    name: String,
    regex: Regex,
    level: LogLevel,
    category: EventCategory,
    extract_fields: Vec<String>,
}

#[derive(Debug, Clone)]
struct PerformanceThresholds {
    slow_test_ms: u64,
    high_memory_mb: f64,
    low_confidence: f64,
    high_error_rate: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            slow_test_ms: 1000,
            high_memory_mb: 100.0,
            low_confidence: 0.7,
            high_error_rate: 0.1,
        }
    }
}

impl LogAnalyzer {
    /// Create a new log analyzer with default patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::create_default_patterns(),
            performance_thresholds: PerformanceThresholds::default(),
        }
    }

    /// Analyze a collection of test results
    pub fn analyze_test_results(&self, results: &[TestResult]) -> LogAnalysis {
        let mut events = Vec::new();
        let mut bottlenecks = Vec::new();
        let mut safety_patterns = Vec::new();
        let mut warnings = Vec::new();

        // Extract events from all test logs
        for result in results {
            for log_line in &result.logs {
                if let Some(event) = self.parse_log_line(log_line, &result.test_name) {
                    events.push(event);
                }
            }
        }

        // Analyze performance bottlenecks
        bottlenecks.extend(self.analyze_performance_bottlenecks(results));

        // Analyze safety decision patterns
        safety_patterns.extend(self.analyze_safety_patterns(results));

        // Generate warnings
        warnings.extend(self.generate_warnings(results, &bottlenecks, &safety_patterns));

        // Calculate health score
        let health_score = self.calculate_health_score(results, &bottlenecks, &safety_patterns);

        LogAnalysis {
            events,
            bottlenecks,
            safety_patterns,
            warnings,
            health_score,
        }
    }

    /// Analyze performance metrics for comprehensive insights
    pub fn analyze_metrics(&self, metrics: &TestMetrics) -> LogAnalysis {
        let mut bottlenecks = Vec::new();
        let mut warnings = Vec::new();

        // Analyze session-level performance
        if metrics.session.average_execution_time.as_millis() > self.performance_thresholds.slow_test_ms as u128 {
            bottlenecks.push(PerformanceBottleneck {
                location: "Session Average".to_string(),
                severity: BottleneckSeverity::Medium,
                impact_description: format!(
                    "Average test execution time of {}ms exceeds threshold",
                    metrics.session.average_execution_time.as_millis()
                ),
                suggested_fix: "Consider optimizing validation algorithms or caching".to_string(),
                affected_tests: vec!["All tests".to_string()],
            });
        }

        // Analyze resource usage
        if metrics.resource_usage.peak_memory_usage_mb > self.performance_thresholds.high_memory_mb {
            bottlenecks.push(PerformanceBottleneck {
                location: "Memory Usage".to_string(),
                severity: BottleneckSeverity::High,
                impact_description: format!(
                    "Peak memory usage of {:.1}MB is excessive",
                    metrics.resource_usage.peak_memory_usage_mb
                ),
                suggested_fix: "Review memory allocation patterns and implement better cleanup".to_string(),
                affected_tests: vec!["Memory-intensive tests".to_string()],
            });
        }

        // Analyze quality metrics
        if metrics.quality.false_positive_rate > self.performance_thresholds.high_error_rate {
            warnings.push(format!(
                "High false positive rate: {:.1}% - Consider tuning safety thresholds",
                metrics.quality.false_positive_rate * 100.0
            ));
        }

        if metrics.quality.false_negative_rate > self.performance_thresholds.high_error_rate {
            warnings.push(format!(
                "High false negative rate: {:.1}% - Safety validation may be too permissive",
                metrics.quality.false_negative_rate * 100.0
            ));
        }

        LogAnalysis {
            events: vec![], // Metrics don't contain log events
            bottlenecks,
            safety_patterns: vec![], // Would be derived from test results
            warnings,
            health_score: metrics.quality.consistency_score * 100.0,
        }
    }

    /// Format analysis results for display
    pub fn format_analysis(&self, analysis: &LogAnalysis) -> Vec<ColoredString> {
        let mut output = Vec::new();

        // Header
        output.push("🔍 Test Execution Analysis".bold().cyan());
        output.push("".normal());

        // Health Score
        let health_color = match analysis.health_score as u8 {
            90..=100 => "green",
            70..=89 => "yellow", 
            50..=69 => "orange",
            _ => "red",
        };
        output.push(format!("📊 Overall Health Score: {:.1}%", analysis.health_score)
            .color(health_color).bold());
        output.push("".normal());

        // Performance Bottlenecks
        if !analysis.bottlenecks.is_empty() {
            output.push("⚠️  Performance Bottlenecks:".yellow().bold());
            for bottleneck in &analysis.bottlenecks {
                let severity_color = match bottleneck.severity {
                    BottleneckSeverity::Low => "green",
                    BottleneckSeverity::Medium => "yellow",
                    BottleneckSeverity::High => "red",
                    BottleneckSeverity::Critical => "magenta",
                };
                output.push(format!("  • {} [{}]", bottleneck.location, format!("{:?}", bottleneck.severity))
                    .color(severity_color));
                output.push(format!("    {}", bottleneck.impact_description).white());
                output.push(format!("    💡 {}", bottleneck.suggested_fix).cyan());
            }
            output.push("".normal());
        }

        // Safety Patterns
        if !analysis.safety_patterns.is_empty() {
            output.push("🛡️  Safety Decision Patterns:".blue().bold());
            for pattern in &analysis.safety_patterns {
                let pattern_color = match pattern.pattern_type {
                    SafetyPatternType::CorrectBlock | SafetyPatternType::CorrectAllow => "green",
                    SafetyPatternType::FalsePositive | SafetyPatternType::FalseNegative => "red",
                    SafetyPatternType::InconsistentRisk => "yellow",
                };
                output.push(format!("  • {:?}: {} occurrences ({:.1}% accuracy)",
                    pattern.pattern_type, pattern.frequency, pattern.accuracy * 100.0)
                    .color(pattern_color));
                
                if let Some(ref tuning) = pattern.recommended_tuning {
                    output.push(format!("    💡 {}", tuning).cyan());
                }
            }
            output.push("".normal());
        }

        // Warnings
        if !analysis.warnings.is_empty() {
            output.push("⚠️  Warnings & Recommendations:".yellow().bold());
            for warning in &analysis.warnings {
                output.push(format!("  • {}", warning).yellow());
            }
            output.push("".normal());
        }

        // Key Events Summary
        if !analysis.events.is_empty() {
            let error_count = analysis.events.iter()
                .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Critical))
                .count();
            let warning_count = analysis.events.iter()
                .filter(|e| matches!(e.level, LogLevel::Warning))
                .count();

            output.push("📋 Event Summary:".white().bold());
            output.push(format!("  • Total Events: {}", analysis.events.len()).white());
            if error_count > 0 {
                output.push(format!("  • Errors: {}", error_count).red());
            }
            if warning_count > 0 {
                output.push(format!("  • Warnings: {}", warning_count).yellow());
            }
        }

        output
    }

    fn create_default_patterns() -> Vec<LogPatternMatcher> {
        vec![
            LogPatternMatcher {
                name: "Test Start".to_string(),
                regex: Regex::new(r"Starting test: (.+)").unwrap(),
                level: LogLevel::Info,
                category: EventCategory::TestExecution,
                extract_fields: vec!["test_name".to_string()],
            },
            LogPatternMatcher {
                name: "Validation Error".to_string(),
                regex: Regex::new(r"Validation failed: (.+)").unwrap(),
                level: LogLevel::Error,
                category: EventCategory::SafetyValidation,
                extract_fields: vec!["error_message".to_string()],
            },
            LogPatternMatcher {
                name: "Threat Level".to_string(),
                regex: Regex::new(r"Threat level: (\w+)").unwrap(),
                level: LogLevel::Info,
                category: EventCategory::SafetyValidation,
                extract_fields: vec!["threat_level".to_string()],
            },
            LogPatternMatcher {
                name: "Performance Timing".to_string(),
                regex: Regex::new(r"Test completed in (.+)").unwrap(),
                level: LogLevel::Info,
                category: EventCategory::PerformanceMetric,
                extract_fields: vec!["duration".to_string()],
            },
        ]
    }

    fn parse_log_line(&self, line: &str, test_name: &str) -> Option<LogEvent> {
        for pattern in &self.patterns {
            if let Some(captures) = pattern.regex.captures(line) {
                let mut context = HashMap::new();
                context.insert("test_name".to_string(), test_name.to_string());
                
                // Extract additional fields
                for (i, field) in pattern.extract_fields.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        context.insert(field.clone(), value.as_str().to_string());
                    }
                }

                return Some(LogEvent {
                    timestamp: chrono::Utc::now(),
                    level: pattern.level.clone(),
                    category: pattern.category.clone(),
                    message: line.to_string(),
                    context,
                });
            }
        }
        None
    }

    fn analyze_performance_bottlenecks(&self, results: &[TestResult]) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // Find slow tests
        let slow_tests: Vec<&TestResult> = results.iter()
            .filter(|r| r.execution_time.as_millis() > self.performance_thresholds.slow_test_ms as u128)
            .collect();

        if !slow_tests.is_empty() {
            bottlenecks.push(PerformanceBottleneck {
                location: "Test Execution".to_string(),
                severity: if slow_tests.len() > results.len() / 2 { 
                    BottleneckSeverity::High 
                } else { 
                    BottleneckSeverity::Medium 
                },
                impact_description: format!("{} tests are running slower than {}ms", 
                    slow_tests.len(), self.performance_thresholds.slow_test_ms),
                suggested_fix: "Profile slow validation steps and optimize algorithms".to_string(),
                affected_tests: slow_tests.iter().map(|r| r.test_name.clone()).collect(),
            });
        }

        // Find low confidence tests
        let low_confidence_tests: Vec<&TestResult> = results.iter()
            .filter(|r| r.validation_result.basic_result.confidence_score < self.performance_thresholds.low_confidence as f32)
            .collect();

        if !low_confidence_tests.is_empty() {
            bottlenecks.push(PerformanceBottleneck {
                location: "Confidence Scoring".to_string(),
                severity: BottleneckSeverity::Medium,
                impact_description: format!("{} tests have low confidence scores", low_confidence_tests.len()),
                suggested_fix: "Review and tune confidence calculation algorithms".to_string(),
                affected_tests: low_confidence_tests.iter().map(|r| r.test_name.clone()).collect(),
            });
        }

        bottlenecks
    }

    fn analyze_safety_patterns(&self, results: &[TestResult]) -> Vec<SafetyPattern> {
        let mut patterns = Vec::new();

        // Analyze correct blocks (dangerous commands properly blocked)
        let correct_blocks = results.iter()
            .filter(|r| matches!(r.category, TestCategory::DangerousCommands) && 
                       !r.validation_result.basic_result.allowed)
            .count();

        if correct_blocks > 0 {
            patterns.push(SafetyPattern {
                pattern_type: SafetyPatternType::CorrectBlock,
                frequency: correct_blocks,
                accuracy: 1.0, // These are correct by definition
                examples: results.iter()
                    .filter(|r| matches!(r.category, TestCategory::DangerousCommands) && 
                               !r.validation_result.basic_result.allowed)
                    .take(3)
                    .map(|r| r.command.clone())
                    .collect(),
                recommended_tuning: None,
            });
        }

        // Analyze false positives (safe commands incorrectly blocked)
        let false_positives: Vec<&TestResult> = results.iter()
            .filter(|r| !r.passed && r.failure_reason.as_ref()
                .map_or(false, |reason| reason.contains("should have passed")))
            .collect();

        if !false_positives.is_empty() {
            patterns.push(SafetyPattern {
                pattern_type: SafetyPatternType::FalsePositive,
                frequency: false_positives.len(),
                accuracy: 0.0, // These are incorrect by definition
                examples: false_positives.iter()
                    .take(3)
                    .map(|r| r.command.clone())
                    .collect(),
                recommended_tuning: Some("Consider relaxing safety thresholds for common safe commands".to_string()),
            });
        }

        // Analyze false negatives (dangerous commands incorrectly allowed)
        let false_negatives: Vec<&TestResult> = results.iter()
            .filter(|r| !r.passed && r.failure_reason.as_ref()
                .map_or(false, |reason| reason.contains("should have been blocked")))
            .collect();

        if !false_negatives.is_empty() {
            patterns.push(SafetyPattern {
                pattern_type: SafetyPatternType::FalseNegative,
                frequency: false_negatives.len(),
                accuracy: 0.0, // These are incorrect by definition
                examples: false_negatives.iter()
                    .take(3)
                    .map(|r| r.command.clone())
                    .collect(),
                recommended_tuning: Some("Strengthen dangerous pattern detection or lower risk thresholds".to_string()),
            });
        }

        patterns
    }

    fn generate_warnings(&self, results: &[TestResult], bottlenecks: &[PerformanceBottleneck], 
                        safety_patterns: &[SafetyPattern]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Overall pass rate warning
        let pass_rate = if results.is_empty() { 
            0.0 
        } else { 
            results.iter().filter(|r| r.passed).count() as f64 / results.len() as f64 
        };

        if pass_rate < 0.8 {
            warnings.push(format!("Low overall pass rate: {:.1}% - Review test expectations", pass_rate * 100.0));
        }

        // Critical bottleneck warning
        if bottlenecks.iter().any(|b| matches!(b.severity, BottleneckSeverity::Critical)) {
            warnings.push("Critical performance bottlenecks detected - immediate attention required".to_string());
        }

        // Safety pattern warnings
        for pattern in safety_patterns {
            if matches!(pattern.pattern_type, SafetyPatternType::FalseNegative) && pattern.frequency > 0 {
                warnings.push("False negatives detected - security validation may be insufficient".to_string());
                break;
            }
        }

        warnings
    }

    fn calculate_health_score(&self, results: &[TestResult], bottlenecks: &[PerformanceBottleneck], 
                             safety_patterns: &[SafetyPattern]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }

        // Base score from pass rate
        let pass_rate = results.iter().filter(|r| r.passed).count() as f64 / results.len() as f64;
        let mut score = pass_rate * 100.0;

        // Deduct for performance issues
        for bottleneck in bottlenecks {
            let deduction = match bottleneck.severity {
                BottleneckSeverity::Low => 2.0,
                BottleneckSeverity::Medium => 5.0,
                BottleneckSeverity::High => 10.0,
                BottleneckSeverity::Critical => 20.0,
            };
            score -= deduction;
        }

        // Deduct for safety issues
        for pattern in safety_patterns {
            if matches!(pattern.pattern_type, SafetyPatternType::FalseNegative) {
                score -= pattern.frequency as f64 * 5.0; // False negatives are serious
            } else if matches!(pattern.pattern_type, SafetyPatternType::FalsePositive) {
                score -= pattern.frequency as f64 * 2.0; // False positives are less serious
            }
        }

        score.max(0.0).min(100.0)
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::cases::{TestResult, TestCategory};
    use crate::models::{RiskLevel, ShellType};

    #[test]
    fn test_log_pattern_matching() {
        let analyzer = LogAnalyzer::new();
        let log_line = "Starting test: test_ls_command";
        let event = analyzer.parse_log_line(log_line, "test_ls_command");
        
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.category, EventCategory::TestExecution);
    }

    #[test]
    fn test_performance_bottleneck_detection() {
        let analyzer = LogAnalyzer::new();
        let slow_result = create_slow_test_result();
        let results = vec![slow_result];
        
        let bottlenecks = analyzer.analyze_performance_bottlenecks(&results);
        assert!(!bottlenecks.is_empty());
        assert!(matches!(bottlenecks[0].severity, BottleneckSeverity::Medium));
    }

    #[test]
    fn test_health_score_calculation() {
        let analyzer = LogAnalyzer::new();
        let good_results = vec![create_passing_test_result(), create_passing_test_result()];
        let mixed_results = vec![create_passing_test_result(), create_failing_test_result()];
        
        let good_score = analyzer.calculate_health_score(&good_results, &[], &[]);
        let mixed_score = analyzer.calculate_health_score(&mixed_results, &[], &[]);
        
        assert!(good_score > mixed_score);
        assert!(good_score <= 100.0);
        assert!(mixed_score >= 0.0);
    }

    fn create_slow_test_result() -> TestResult {
        TestResult {
            test_name: "slow_test".to_string(),
            category: TestCategory::BasicSafety,
            command: "sleep 10".to_string(),
            shell: ShellType::Bash,
            execution_time: std::time::Duration::from_millis(2000), // Slow test
            validation_result: create_dummy_validation_result(),
            expected_outcome: crate::testing::cases::TestOutcome::ShouldPass {
                max_risk_level: RiskLevel::Low,
                should_require_confirmation: false,
            },
            passed: true,
            failure_reason: None,
            performance_metrics: std::collections::HashMap::new(),
            logs: vec!["Starting test: slow_test".to_string()],
            timestamp: chrono::Utc::now(),
        }
    }

    fn create_passing_test_result() -> TestResult {
        TestResult {
            test_name: "passing_test".to_string(),
            category: TestCategory::BasicSafety,
            command: "ls".to_string(),
            shell: ShellType::Bash,
            execution_time: std::time::Duration::from_millis(100),
            validation_result: create_dummy_validation_result(),
            expected_outcome: crate::testing::cases::TestOutcome::ShouldPass {
                max_risk_level: RiskLevel::Low,
                should_require_confirmation: false,
            },
            passed: true,
            failure_reason: None,
            performance_metrics: std::collections::HashMap::new(),
            logs: vec!["Starting test: passing_test".to_string()],
            timestamp: chrono::Utc::now(),
        }
    }

    fn create_failing_test_result() -> TestResult {
        TestResult {
            test_name: "failing_test".to_string(),
            category: TestCategory::BasicSafety,
            command: "rm important_file".to_string(),
            shell: ShellType::Bash,
            execution_time: std::time::Duration::from_millis(100),
            validation_result: create_dummy_validation_result(),
            expected_outcome: crate::testing::cases::TestOutcome::ShouldPass {
                max_risk_level: RiskLevel::Low,
                should_require_confirmation: false,
            },
            passed: false,
            failure_reason: Some("Command should have passed but was blocked".to_string()),
            performance_metrics: std::collections::HashMap::new(),
            logs: vec!["Starting test: failing_test".to_string()],
            timestamp: chrono::Utc::now(),
        }
    }

    fn create_dummy_validation_result() -> crate::safety::advanced::AdvancedValidationResult {
        crate::safety::advanced::AdvancedValidationResult {
            basic_result: crate::safety::ValidationResult {
                allowed: true,
                risk_level: RiskLevel::Low,
                explanation: "Test validation".to_string(),
                warnings: vec![],
                matched_patterns: vec![],
                confidence_score: 0.9,
            },
            threat_level: crate::safety::advanced::ThreatLevel::Safe,
            behavioral_patterns: vec![],
            contextual_warnings: vec![],
            behavioral_warnings: vec![],
            ml_scores: std::collections::HashMap::new(),
            recommendations: vec![],
            requires_monitoring: false,
            analysis_time_ms: 50,
        }
    }
}
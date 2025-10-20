//! Interactive reporting and user communication for manual testing
//!
//! Provides rich, interactive interfaces for presenting test results,
//! gathering user feedback, and guiding testing workflows.

use colored::Colorize;
use std::io::{self, Write};

use crate::testing::{
    analyzer::LogAnalysis,
    cases::{TestCategory, TestResult},
    metrics::{TestMetrics, PerformanceInsights},
};

/// Interactive reporter for rich test result presentation
pub struct InteractiveReporter {
    config: ReporterConfig,
}

/// Configuration for report formatting and interaction
#[derive(Debug, Clone)]
pub struct ReporterConfig {
    pub use_colors: bool,
    pub show_detailed_logs: bool,
    pub interactive_mode: bool,
    pub max_log_lines: usize,
    pub show_performance_details: bool,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            show_detailed_logs: false,
            interactive_mode: true,
            max_log_lines: 20,
            show_performance_details: true,
        }
    }
}

/// User interaction responses for guided testing
#[derive(Debug, Clone)]
pub enum UserResponse {
    Continue,
    ShowDetails(String),
    RunAgain,
    ModifyTest,
    Exit,
    Custom(String),
}

impl InteractiveReporter {
    /// Create a new interactive reporter
    pub fn new() -> Self {
        Self {
            config: ReporterConfig::default(),
        }
    }

    /// Create reporter with custom configuration
    pub fn with_config(config: ReporterConfig) -> Self {
        Self { config }
    }

    /// Present test category selection menu
    pub fn present_category_menu(&self) -> io::Result<TestCategory> {
        self.print_header("🧪 Test Category Selection");
        println!();

        let categories = TestCategory::all_categories();
        for (i, category) in categories.iter().enumerate() {
            let colored_line = if self.config.use_colors {
                format!("{}. {} - {}", 
                    (i + 1).to_string().cyan().bold(),
                    format!("{:?}", category).green().bold(),
                    category.description().white()
                )
            } else {
                format!("{}. {:?} - {}", i + 1, category, category.description())
            };
            println!("  {}", colored_line);
        }
        println!();

        loop {
            print!("Select category (1-{}): ", categories.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= categories.len() {
                    return Ok(categories[choice - 1].clone());
                }
            }

            println!("Invalid selection. Please choose 1-{}", categories.len());
        }
    }

    /// Present test execution results
    pub fn present_test_result(&self, result: &TestResult) -> io::Result<()> {
        self.print_header(&format!("📋 Test Result: {}", result.test_name));
        println!();

        // Test overview
        self.print_test_overview(result);
        println!();

        // Validation details
        self.print_validation_details(result);
        println!();

        // Performance metrics
        if self.config.show_performance_details {
            self.print_performance_metrics(result);
            println!();
        }

        // Logs (if enabled)
        if self.config.show_detailed_logs && !result.logs.is_empty() {
            self.print_execution_logs(result);
            println!();
        }

        Ok(())
    }

    /// Present comprehensive test session summary
    pub fn present_session_summary(&self, results: &[TestResult], metrics: &TestMetrics, 
                                  analysis: &LogAnalysis) -> io::Result<()> {
        self.print_header("📊 Test Session Summary");
        println!();

        // High-level statistics
        self.print_session_statistics(results, metrics);
        println!();

        // Performance analysis
        self.print_performance_analysis(metrics, analysis);
        println!();

        // Detailed analysis
        self.print_detailed_analysis(analysis);
        println!();

        // Recommendations
        self.print_recommendations(analysis);

        Ok(())
    }

    /// Interactive discussion about results
    pub fn discuss_results(&self, results: &[TestResult], analysis: &LogAnalysis) -> io::Result<Vec<UserResponse>> {
        let mut responses = Vec::new();

        self.print_header("💬 Results Discussion");
        println!();

        // Ask about specific areas of interest
        let discussion_topics = self.generate_discussion_topics(results, analysis);
        
        for topic in discussion_topics {
            println!("{}", topic.question);
            println!();

            if let Some(response) = self.get_user_input(&topic.options)? {
                responses.push(response);
                
                // Provide contextual feedback
                if let Some(feedback) = &topic.feedback {
                    println!("{}", feedback);
                    println!();
                }
            }
        }

        Ok(responses)
    }

    /// Present improvement recommendations
    pub fn present_recommendations(&self, insights: &PerformanceInsights, 
                                 analysis: &LogAnalysis) -> io::Result<()> {
        self.print_header("💡 Improvement Recommendations");
        println!();

        // Performance improvements
        if !insights.bottlenecks.is_empty() {
            let colored_header = if self.config.use_colors {
                "🚀 Performance Optimizations:".blue().bold()
            } else {
                "🚀 Performance Optimizations:".normal()
            };
            println!("{}", colored_header);
            
            for (i, bottleneck) in insights.bottlenecks.iter().enumerate() {
                println!("  {}. {}", i + 1, bottleneck);
            }
            println!();
        }

        // Safety tuning recommendations
        if !analysis.safety_patterns.is_empty() {
            let colored_header = if self.config.use_colors {
                "🛡️  Safety System Tuning:".yellow().bold()
            } else {
                "🛡️  Safety System Tuning:".normal()
            };
            println!("{}", colored_header);
            
            for pattern in &analysis.safety_patterns {
                if let Some(ref tuning) = pattern.recommended_tuning {
                    println!("  • {}", tuning);
                }
            }
            println!();
        }

        // Quality improvements
        if !insights.recommendations.is_empty() {
            let colored_header = if self.config.use_colors {
                "✨ Quality Enhancements:".green().bold()
            } else {
                "✨ Quality Enhancements:".normal()
            };
            println!("{}", colored_header);
            
            for (i, recommendation) in insights.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, recommendation);
            }
            println!();
        }

        Ok(())
    }

    /// Ask user for custom test input
    pub fn get_custom_test_input(&self) -> io::Result<(String, String)> {
        self.print_header("✏️  Custom Test Input");
        println!();

        print!("Enter command to test: ");
        io::stdout().flush()?;
        let mut command = String::new();
        io::stdin().read_line(&mut command)?;
        let command = command.trim().to_string();

        print!("Enter test description: ");
        io::stdout().flush()?;
        let mut description = String::new();
        io::stdin().read_line(&mut description)?;
        let description = description.trim().to_string();

        Ok((command, description))
    }

    // Helper methods for formatting and display

    fn print_header(&self, title: &str) {
        let separator = "=".repeat(60);
        if self.config.use_colors {
            println!("{}", separator.cyan().bold());
            println!("{}", title.cyan().bold());
            println!("{}", separator.cyan().bold());
        } else {
            println!("{}", separator);
            println!("{}", title);
            println!("{}", separator);
        }
    }

    fn print_test_overview(&self, result: &TestResult) {
        let status_text = if result.passed { "✅ PASSED" } else { "❌ FAILED" };
        let status_colored = if self.config.use_colors {
            if result.passed {
                status_text.green().bold()
            } else {
                status_text.red().bold()
            }
        } else {
            status_text.normal()
        };

        println!("Status: {}", status_colored);
        println!("Command: {}", result.command);
        println!("Shell: {:?}", result.shell);
        println!("Category: {:?}", result.category);
        println!("Execution Time: {:?}", result.execution_time);
        
        if let Some(ref reason) = result.failure_reason {
            let failure_colored = if self.config.use_colors {
                format!("Failure Reason: {}", reason).red()
            } else {
                format!("Failure Reason: {}", reason).normal()
            };
            println!("{}", failure_colored);
        }
    }

    fn print_validation_details(&self, result: &TestResult) {
        let validation = &result.validation_result;
        
        println!("🛡️  Safety Validation:");
        println!("  Allowed: {}", validation.basic_result.allowed);
        println!("  Risk Level: {:?}", validation.basic_result.risk_level);
        println!("  Threat Level: {:?}", validation.threat_level);
        println!("  Confidence: {:.2}", validation.basic_result.confidence_score);
        
        if !validation.basic_result.matched_patterns.is_empty() {
            println!("  Matched Patterns: {:?}", validation.basic_result.matched_patterns);
        }
        
        if !validation.recommendations.is_empty() {
            println!("  Recommendations:");
            for rec in &validation.recommendations {
                println!("    • {}", rec);
            }
        }
    }

    fn print_performance_metrics(&self, result: &TestResult) {
        println!("⚡ Performance Metrics:");
        println!("  Analysis Time: {}ms", result.validation_result.analysis_time_ms);
        
        for (key, value) in &result.performance_metrics {
            println!("  {}: {:.2}", key, value);
        }
    }

    fn print_execution_logs(&self, result: &TestResult) {
        println!("📝 Execution Logs:");
        let logs_to_show = result.logs.iter()
            .take(self.config.max_log_lines)
            .collect::<Vec<_>>();
            
        for (i, log) in logs_to_show.iter().enumerate() {
            if self.config.use_colors {
                println!("  {} {}", format!("{:2}.", i + 1).white().dimmed(), log);
            } else {
                println!("  {}. {}", i + 1, log);
            }
        }
        
        if result.logs.len() > self.config.max_log_lines {
            let remaining = result.logs.len() - self.config.max_log_lines;
            println!("  ... ({} more lines)", remaining);
        }
    }

    fn print_session_statistics(&self, _results: &[TestResult], metrics: &TestMetrics) {
        let pass_rate = if metrics.session.tests_executed > 0 {
            (metrics.session.tests_passed as f64 / metrics.session.tests_executed as f64) * 100.0
        } else {
            0.0
        };

        println!("📈 Session Statistics:");
        println!("  Total Tests: {}", metrics.session.tests_executed);
        println!("  Passed: {}", metrics.session.tests_passed);
        println!("  Failed: {}", metrics.session.tests_failed);
        println!("  Pass Rate: {:.1}%", pass_rate);
        println!("  Average Execution Time: {:?}", metrics.session.average_execution_time);
        println!("  Session Duration: {:?}", metrics.session.total_duration);
    }

    fn print_performance_analysis(&self, metrics: &TestMetrics, analysis: &LogAnalysis) {
        println!("🎯 Performance Analysis:");
        println!("  Health Score: {:.1}%", analysis.health_score);
        println!("  Peak Memory: {:.1} MB", metrics.resource_usage.peak_memory_usage_mb);
        println!("  Average CPU: {:.1}%", metrics.resource_usage.average_cpu_usage);
        
        if !analysis.bottlenecks.is_empty() {
            println!("  Bottlenecks Detected: {}", analysis.bottlenecks.len());
        }
    }

    fn print_detailed_analysis(&self, analysis: &LogAnalysis) {
        // Format and print the analysis using the analyzer's formatting
        let formatted_lines = crate::testing::analyzer::LogAnalyzer::new()
            .format_analysis(analysis);
        
        for line in formatted_lines {
            println!("{}", line);
        }
    }

    fn print_recommendations(&self, analysis: &LogAnalysis) {
        if !analysis.warnings.is_empty() {
            println!("🎯 Next Steps:");
            for (i, warning) in analysis.warnings.iter().enumerate() {
                println!("  {}. {}", i + 1, warning);
            }
        }
    }

    fn generate_discussion_topics(&self, results: &[TestResult], analysis: &LogAnalysis) -> Vec<DiscussionTopic> {
        let mut topics = Vec::new();

        // Topic about overall results
        if !results.is_empty() {
            let pass_rate = results.iter().filter(|r| r.passed).count() as f64 / results.len() as f64;
            topics.push(DiscussionTopic {
                question: format!("The overall pass rate is {:.1}%. How do you feel about these results?", pass_rate * 100.0),
                options: vec![
                    "Expected - results look good".to_string(),
                    "Some failures were unexpected".to_string(),
                    "Need to investigate specific failures".to_string(),
                    "Test expectations may need adjustment".to_string(),
                ],
                feedback: Some("Understanding your expectations helps improve the test suite.".to_string()),
            });
        }

        // Topic about performance
        if analysis.health_score < 80.0 {
            topics.push(DiscussionTopic {
                question: format!("The system health score is {:.1}%. What's your priority for improvement?", analysis.health_score),
                options: vec![
                    "Performance optimization".to_string(),
                    "Safety validation accuracy".to_string(),
                    "Test coverage expansion".to_string(),
                    "All areas need attention".to_string(),
                ],
                feedback: Some("Prioritizing improvements helps focus development efforts.".to_string()),
            });
        }

        // Topic about safety patterns
        if !analysis.safety_patterns.is_empty() {
            topics.push(DiscussionTopic {
                question: "We detected some safety patterns. Are you satisfied with the current safety validation?".to_string(),
                options: vec![
                    "Safety is too strict".to_string(),
                    "Safety is too permissive".to_string(),
                    "Safety level is appropriate".to_string(),
                    "Need different safety for different contexts".to_string(),
                ],
                feedback: Some("Safety tuning is crucial for balancing security and usability.".to_string()),
            });
        }

        topics
    }

    fn get_user_input(&self, options: &[String]) -> io::Result<Option<UserResponse>> {
        if !self.config.interactive_mode {
            return Ok(None);
        }

        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, option);
        }
        print!("\nYour choice (1-{}, or 'skip'): ", options.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "skip" {
            return Ok(None);
        }

        if let Ok(choice) = input.parse::<usize>() {
            if choice > 0 && choice <= options.len() {
                return Ok(Some(UserResponse::Custom(options[choice - 1].clone())));
            }
        }

        println!("Invalid selection.");
        Ok(None)
    }
}

struct DiscussionTopic {
    question: String,
    options: Vec<String>,
    feedback: Option<String>,
}

impl Default for InteractiveReporter {
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
    fn test_reporter_creation() {
        let reporter = InteractiveReporter::new();
        assert!(reporter.config.use_colors);
        assert!(reporter.config.interactive_mode);
    }

    #[test]
    fn test_custom_config() {
        let config = ReporterConfig {
            use_colors: false,
            show_detailed_logs: true,
            interactive_mode: false,
            max_log_lines: 50,
            show_performance_details: false,
        };
        
        let reporter = InteractiveReporter::with_config(config.clone());
        assert!(!reporter.config.use_colors);
        assert!(reporter.config.show_detailed_logs);
        assert!(!reporter.config.interactive_mode);
        assert_eq!(reporter.config.max_log_lines, 50);
    }

    #[test]
    fn test_discussion_topic_generation() {
        let reporter = InteractiveReporter::new();
        let results = vec![create_test_result(true), create_test_result(false)];
        let analysis = create_dummy_analysis();
        
        let topics = reporter.generate_discussion_topics(&results, &analysis);
        assert!(!topics.is_empty());
        assert!(topics.iter().any(|t| t.question.contains("pass rate")));
    }

    fn create_test_result(passed: bool) -> TestResult {
        TestResult {
            test_name: "test".to_string(),
            category: TestCategory::BasicSafety,
            command: "ls".to_string(),
            shell: ShellType::Bash,
            execution_time: std::time::Duration::from_millis(100),
            validation_result: crate::safety::advanced::AdvancedValidationResult {
                basic_result: crate::safety::ValidationResult {
                    allowed: passed,
                    risk_level: RiskLevel::Low,
                    explanation: "Test".to_string(),
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
            },
            expected_outcome: crate::testing::cases::TestOutcome::ShouldPass {
                max_risk_level: RiskLevel::Medium,
                should_require_confirmation: false,
            },
            passed,
            failure_reason: if passed { None } else { Some("Test failure".to_string()) },
            performance_metrics: std::collections::HashMap::new(),
            logs: vec!["Starting test".to_string()],
            timestamp: chrono::Utc::now(),
        }
    }

    fn create_dummy_analysis() -> LogAnalysis {
        crate::testing::analyzer::LogAnalysis {
            events: vec![],
            bottlenecks: vec![],
            safety_patterns: vec![],
            warnings: vec!["Test warning".to_string()],
            health_score: 75.0,
        }
    }
}
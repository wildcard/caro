//! Manual test runner for interactive testing workflows
//!
//! Orchestrates the complete testing experience from test selection through
//! execution, analysis, and user feedback collection.

use std::collections::HashMap;

use crate::{
    models::RiskLevel,
    safety::advanced::{AdvancedSafetyValidator, AdvancedSafetyConfig, UserFeedback},
    testing::{
        analyzer::LogAnalyzer,
        cases::{TestCase, TestCategory, TestResult, TestCaseBuilder},
        metrics::{MetricsCollector, PerformanceAnalyzer},
        reporter::{InteractiveReporter, UserResponse},
    },
};

/// Main test runner for interactive manual testing
pub struct ManualTestRunner {
    validator: AdvancedSafetyValidator,
    reporter: InteractiveReporter,
    analyzer: LogAnalyzer,
    test_library: TestLibrary,
    metrics_collector: MetricsCollector,
    session_results: Vec<TestResult>,
}

/// Library of predefined test cases organized by category
pub struct TestLibrary {
    tests_by_category: HashMap<TestCategory, Vec<Box<dyn TestCase>>>,
}

/// Configuration for test runner behavior
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub enable_real_time_feedback: bool,
    pub auto_analyze_results: bool,
    pub collect_performance_metrics: bool,
    pub interactive_discussions: bool,
    pub max_tests_per_session: usize,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            enable_real_time_feedback: true,
            auto_analyze_results: true,
            collect_performance_metrics: true,
            interactive_discussions: true,
            max_tests_per_session: 50,
        }
    }
}

impl ManualTestRunner {
    /// Create a new test runner with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AdvancedSafetyConfig::development(); // Use development config for testing
        let validator = AdvancedSafetyValidator::new(config).await?;
        let reporter = InteractiveReporter::new();
        let analyzer = LogAnalyzer::new();
        let test_library = TestLibrary::new();
        let metrics_collector = MetricsCollector::new();

        Ok(Self {
            validator,
            reporter,
            analyzer,
            test_library,
            metrics_collector,
            session_results: Vec::new(),
        })
    }

    /// Start an interactive testing session
    pub async fn run_interactive_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Welcome to cmdai Manual Testing System");
        println!("═══════════════════════════════════════════");
        println!();

        loop {
            // Present main menu
            match self.present_main_menu()? {
                MainMenuChoice::RunTestCategory => {
                    let category = self.reporter.present_category_menu()?;
                    self.run_category_tests(category).await?;
                }
                MainMenuChoice::RunSingleTest => {
                    self.run_single_test().await?;
                }
                MainMenuChoice::RunCustomTest => {
                    self.run_custom_test().await?;
                }
                MainMenuChoice::ViewResults => {
                    self.view_session_results().await?;
                }
                MainMenuChoice::AnalyzePerformance => {
                    self.analyze_performance().await?;
                }
                MainMenuChoice::ExportResults => {
                    self.export_results().await?;
                }
                MainMenuChoice::Exit => {
                    self.finalize_session().await?;
                    break;
                }
            }
        }

        Ok(())
    }

    /// Run all tests in a specific category
    pub async fn run_category_tests(&mut self, category: TestCategory) -> Result<(), Box<dyn std::error::Error>> {
        println!("🏃 Running tests for category: {:?}", category);
        println!();

        let tests = self.test_library.get_tests_for_category(&category);
        if tests.is_empty() {
            println!("No tests found for category {:?}", category);
            return Ok(());
        }

        println!("Found {} tests in this category", tests.len());
        
        for (i, test) in tests.iter().enumerate() {
            println!("\n📝 Running test {}/{}: {}", i + 1, tests.len(), test.name());
            
            // Setup test
            test.setup(&self.validator).await?;
            
            // Execute test
            let result = test.execute(&self.validator).await;
            
            // Cleanup test
            test.cleanup(&self.validator).await?;
            
            // Present result
            self.reporter.present_test_result(&result)?;
            
            // Record result
            self.metrics_collector.record_test_result(result.clone());
            self.session_results.push(result);
            
            // Brief pause for readability
            if i < tests.len() - 1 {
                println!("\nPress Enter to continue to next test...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }
        }

        println!("\n✅ Category testing completed!");
        Ok(())
    }

    /// Run a single test selected from library
    pub async fn run_single_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Present test selection menu
        let all_tests = self.test_library.get_all_tests();
        
        println!("📋 Available Tests:");
        for (i, test) in all_tests.iter().enumerate() {
            println!("  {}. {} (Category: {:?}, Difficulty: {})",
                i + 1, test.name(), test.category(), test.difficulty());
            println!("     {}", test.description());
        }

        print!("\nSelect test (1-{}): ", all_tests.len());
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= all_tests.len() {
                let test = &all_tests[choice - 1];
                
                println!("\n🎯 Running test: {}", test.name());
                println!("Command: {}", test.command());
                println!("Expected: {:?}", test.expected_outcome());
                println!();

                // Execute test
                test.setup(&self.validator).await?;
                let result = test.execute(&self.validator).await;
                test.cleanup(&self.validator).await?;

                // Present detailed results
                self.reporter.present_test_result(&result)?;
                
                // Record result
                self.metrics_collector.record_test_result(result.clone());
                self.session_results.push(result);
            }
        }

        Ok(())
    }

    /// Run a custom test provided by user
    pub async fn run_custom_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (command, description) = self.reporter.get_custom_test_input()?;

        // Create custom test case
        let test_case = TestCaseBuilder::new(&format!("custom_{}", self.session_results.len() + 1))
            .category(TestCategory::CustomUser)
            .description(&description)
            .command(&command)
            .should_pass(RiskLevel::Medium, false) // Default expectation
            .build()?;

        println!("\n🎯 Running custom test: {}", test_case.name());
        println!("Command: {}", command);
        println!();

        // Execute test
        let result = test_case.execute(&self.validator).await;

        // Present results
        self.reporter.present_test_result(&result)?;

        // Ask user about expectations vs results
        println!("\n💬 How do you feel about this result?");
        println!("1. Result matches my expectations");
        println!("2. Command should have been allowed");
        println!("3. Command should have been blocked");
        println!("4. Different risk level expected");

        print!("Your assessment (1-4): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Record user feedback for learning
        if let Ok(choice) = input.trim().parse::<usize>() {
            let feedback = match choice {
                2 => Some(UserFeedback::FalsePositive),
                3 => Some(UserFeedback::FalseNegative),
                _ => None,
            };

            if let Some(feedback) = feedback {
                self.validator.record_feedback(&command, feedback).await?;
                println!("✅ Feedback recorded for system learning");
            }
        }

        // Record result
        self.metrics_collector.record_test_result(result.clone());
        self.session_results.push(result);

        Ok(())
    }

    /// View comprehensive session results
    pub async fn view_session_results(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.session_results.is_empty() {
            println!("No test results in current session.");
            return Ok(());
        }

        // Generate metrics and analysis
        let metrics = self.metrics_collector.clone().finalize();
        let analysis = self.analyzer.analyze_test_results(&self.session_results);

        // Present comprehensive summary
        self.reporter.present_session_summary(&self.session_results, &metrics, &analysis)?;

        // Interactive discussion
        let responses = self.reporter.discuss_results(&self.session_results, &analysis)?;
        
        if !responses.is_empty() {
            println!("\n📝 User feedback recorded:");
            for response in responses {
                if let UserResponse::Custom(feedback) = response {
                    println!("  • {}", feedback);
                }
            }
        }

        Ok(())
    }

    /// Analyze performance and provide insights
    pub async fn analyze_performance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.session_results.is_empty() {
            println!("No test results available for performance analysis.");
            return Ok(());
        }

        println!("🔍 Analyzing Performance...");
        println!();

        let metrics = self.metrics_collector.clone().finalize();
        let analysis = self.analyzer.analyze_test_results(&self.session_results);
        let insights = PerformanceAnalyzer::analyze(&metrics);

        // Present detailed performance analysis
        self.reporter.present_recommendations(&insights, &analysis)?;

        // Show trending data if available
        if !insights.trends.is_empty() {
            println!("📈 Performance Trends:");
            for trend in &insights.trends {
                println!("  • {}", trend);
            }
            println!();
        }

        // Performance score breakdown
        println!("🎯 Performance Score: {:.1}/100", insights.performance_score);
        if insights.performance_score < 70.0 {
            println!("   ⚠️  Consider addressing performance issues");
        } else if insights.performance_score >= 90.0 {
            println!("   ✅ Excellent performance!");
        }

        Ok(())
    }

    /// Export results to file
    pub async fn export_results(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.session_results.is_empty() {
            println!("No results to export.");
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("cmdai_test_results_{}.json", timestamp);

        let export_data = serde_json::json!({
            "session_info": {
                "timestamp": chrono::Utc::now(),
                "total_tests": self.session_results.len(),
                "cmdai_version": env!("CARGO_PKG_VERSION")
            },
            "results": self.session_results,
            "metrics": self.metrics_collector.clone().finalize(),
            "analysis": self.analyzer.analyze_test_results(&self.session_results)
        });

        std::fs::write(&filename, serde_json::to_string_pretty(&export_data)?)?;
        println!("✅ Results exported to: {}", filename);

        Ok(())
    }

    /// Finalize testing session
    pub async fn finalize_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.session_results.is_empty() {
            println!("📊 Session Summary:");
            println!("  Tests Run: {}", self.session_results.len());
            
            let passed = self.session_results.iter().filter(|r| r.passed).count();
            let pass_rate = (passed as f64 / self.session_results.len() as f64) * 100.0;
            
            println!("  Pass Rate: {:.1}%", pass_rate);
            
            let metrics = self.metrics_collector.clone().finalize();
            println!("  Total Time: {:?}", metrics.session.total_duration);
            
            println!();
            println!("🙏 Thank you for testing cmdai!");
        }

        Ok(())
    }

    fn present_main_menu(&self) -> Result<MainMenuChoice, Box<dyn std::error::Error>> {
        println!("🎯 Main Menu:");
        println!("  1. Run tests by category");
        println!("  2. Run single test");
        println!("  3. Run custom test");
        println!("  4. View session results");
        println!("  5. Analyze performance");
        println!("  6. Export results");
        println!("  7. Exit");
        println!();

        print!("Select option (1-7): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(MainMenuChoice::RunTestCategory),
            "2" => Ok(MainMenuChoice::RunSingleTest),
            "3" => Ok(MainMenuChoice::RunCustomTest),
            "4" => Ok(MainMenuChoice::ViewResults),
            "5" => Ok(MainMenuChoice::AnalyzePerformance),
            "6" => Ok(MainMenuChoice::ExportResults),
            "7" => Ok(MainMenuChoice::Exit),
            _ => {
                println!("Invalid selection. Please choose 1-7.");
                self.present_main_menu()
            }
        }
    }
}

#[derive(Debug, Clone)]
enum MainMenuChoice {
    RunTestCategory,
    RunSingleTest,
    RunCustomTest,
    ViewResults,
    AnalyzePerformance,
    ExportResults,
    Exit,
}

impl TestLibrary {
    /// Create a new test library with predefined tests
    pub fn new() -> Self {
        let mut library = Self {
            tests_by_category: HashMap::new(),
        };
        
        library.populate_default_tests();
        library
    }

    /// Get all tests for a specific category
    pub fn get_tests_for_category(&self, category: &TestCategory) -> Vec<&Box<dyn TestCase>> {
        self.tests_by_category
            .get(category)
            .map(|tests| tests.iter().collect())
            .unwrap_or_default()
    }

    /// Get all tests across all categories
    pub fn get_all_tests(&self) -> Vec<&Box<dyn TestCase>> {
        self.tests_by_category
            .values()
            .flat_map(|tests| tests.iter())
            .collect()
    }

    /// Add a test to the library
    pub fn add_test(&mut self, test: Box<dyn TestCase>) {
        let category = test.category();
        self.tests_by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(test);
    }

    fn populate_default_tests(&mut self) {
        // Basic Safety Tests
        self.add_basic_safety_tests();
        
        // Dangerous Command Tests
        self.add_dangerous_command_tests();
        
        // Edge Case Tests
        self.add_edge_case_tests();
        
        // Adaptive Learning Tests
        self.add_adaptive_learning_tests();
        
        // Performance Tests
        self.add_performance_tests();
    }

    fn add_basic_safety_tests(&mut self) {
        let tests: Vec<Box<dyn TestCase>> = vec![
            Box::new(TestCaseBuilder::new("safe_ls")
                .category(TestCategory::BasicSafety)
                .description("List files safely")
                .command("ls -la")
                .should_pass(RiskLevel::Low, false)
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("safe_pwd")
                .category(TestCategory::BasicSafety)
                .description("Show current directory")
                .command("pwd")
                .should_pass(RiskLevel::Low, false)
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("safe_df")
                .category(TestCategory::BasicSafety)
                .description("Check disk usage")
                .command("df -h")
                .should_pass(RiskLevel::Low, false)
                .build().unwrap()),
        ];

        for test in tests {
            self.add_test(test);
        }
    }

    fn add_dangerous_command_tests(&mut self) {
        let tests: Vec<Box<dyn TestCase>> = vec![
            Box::new(TestCaseBuilder::new("dangerous_rm_rf")
                .category(TestCategory::DangerousCommands)
                .description("Dangerous recursive delete")
                .command("rm -rf /")
                .should_block(RiskLevel::Critical, vec!["rm".to_string(), "recursive".to_string()])
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("fork_bomb")
                .category(TestCategory::DangerousCommands)
                .description("Fork bomb attack")
                .command(":(){ :|:& };:")
                .should_block(RiskLevel::Critical, vec!["fork".to_string()])
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("dangerous_dd")
                .category(TestCategory::DangerousCommands)
                .description("Disk destruction with dd")
                .command("dd if=/dev/zero of=/dev/sda")
                .should_block(RiskLevel::Critical, vec!["dd".to_string()])
                .build().unwrap()),
        ];

        for test in tests {
            self.add_test(test);
        }
    }

    fn add_edge_case_tests(&mut self) {
        let tests: Vec<Box<dyn TestCase>> = vec![
            Box::new(TestCaseBuilder::new("empty_command")
                .category(TestCategory::EdgeCases)
                .description("Empty command input")
                .command("")
                .should_block(RiskLevel::Medium, vec![])
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("very_long_command")
                .category(TestCategory::EdgeCases)
                .description("Extremely long command")
                .command(&"echo ".repeat(1000))
                .should_pass(RiskLevel::Medium, false)
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("unicode_command")
                .category(TestCategory::EdgeCases)
                .description("Command with unicode characters")
                .command("echo '你好世界 🌍'")
                .should_pass(RiskLevel::Low, false)
                .build().unwrap()),
        ];

        for test in tests {
            self.add_test(test);
        }
    }

    fn add_adaptive_learning_tests(&mut self) {
        let tests: Vec<Box<dyn TestCase>> = vec![
            Box::new(TestCaseBuilder::new("learning_approved")
                .category(TestCategory::AdaptiveLearning)
                .description("Test adaptive learning with approval")
                .command("rm temp_file.txt")
                .should_learn(UserFeedback::Approved, "approved")
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("learning_rejected")
                .category(TestCategory::AdaptiveLearning)
                .description("Test adaptive learning with rejection")
                .command("chmod 777 sensitive_file")
                .should_learn(UserFeedback::Rejected, "rejected")
                .build().unwrap()),
        ];

        for test in tests {
            self.add_test(test);
        }
    }

    fn add_performance_tests(&mut self) {
        let tests: Vec<Box<dyn TestCase>> = vec![
            Box::new(TestCaseBuilder::new("performance_fast")
                .category(TestCategory::PerformanceBenchmarks)
                .description("Fast response time test")
                .command("echo 'speed test'")
                .performance_target(100, 0.9)
                .build().unwrap()),
                
            Box::new(TestCaseBuilder::new("performance_complex")
                .category(TestCategory::PerformanceBenchmarks)
                .description("Complex validation performance")
                .command("find /etc -name '*.conf' | xargs grep -l password | head -10")
                .performance_target(500, 0.8)
                .build().unwrap()),
        ];

        for test in tests {
            self.add_test(test);
        }
    }
}

// Helper trait for flushing stdout
trait Flush {
    fn flush(&self) -> std::io::Result<()>;
}

impl Flush for std::io::Stdout {
    fn flush(&self) -> std::io::Result<()> {
        std::io::Write::flush(&mut std::io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_library_creation() {
        let library = TestLibrary::new();
        
        // Check that we have tests in each category
        for category in TestCategory::all_categories() {
            if matches!(category, TestCategory::CustomUser | TestCategory::IntegrationTests) {
                continue; // These categories may be empty by default
            }
            
            let tests = library.get_tests_for_category(&category);
            assert!(!tests.is_empty(), "Category {:?} should have tests", category);
        }
    }

    #[test]
    fn test_test_library_all_tests() {
        let library = TestLibrary::new();
        let all_tests = library.get_all_tests();
        
        assert!(!all_tests.is_empty());
        
        // Verify we have tests from multiple categories
        let categories: std::collections::HashSet<_> = all_tests.iter()
            .map(|test| test.category())
            .collect();
        assert!(categories.len() > 1);
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let runner = ManualTestRunner::new().await;
        assert!(runner.is_ok());
    }
}
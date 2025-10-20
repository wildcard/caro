// Core evaluation engine for testing command accuracy

use std::time::Instant;
use anyhow::Result;

use crate::cli::{CliApp, CliConfig, OutputFormat, IntoCliArgs};
use crate::evaluation::metrics::{AccuracyScore, PerformanceMetrics, TestCaseResult};
use crate::evaluation::TestCase;
use crate::models::SafetyLevel;

#[derive(Debug, Clone)]
struct TestCliArgs {
    prompt: String,
}

impl IntoCliArgs for TestCliArgs {
    fn prompt(&self) -> Option<String> {
        Some(self.prompt.clone())
    }
    
    fn shell(&self) -> Option<String> {
        None // Use default
    }
    
    fn safety(&self) -> Option<String> {
        None // Use default
    }
    
    fn output(&self) -> Option<String> {
        None // Use default
    }
    
    fn confirm(&self) -> bool {
        false // No confirmation for tests
    }
    
    fn verbose(&self) -> bool {
        false
    }
    
    fn config_file(&self) -> Option<String> {
        None
    }
}

pub struct EvaluationEngine {
    cli_app: CliApp,
}

impl EvaluationEngine {
    /// Create a new evaluation engine with default configuration
    pub async fn new() -> Result<Self> {
        let config = CliConfig {
            default_shell: crate::models::ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            output_format: OutputFormat::Plain,
            auto_confirm: true, // Auto-confirm for testing
        };
        
        let cli_app = CliApp::with_config(config).await?;
        
        Ok(EvaluationEngine { cli_app })
    }
    
    /// Create evaluation engine with specific shell configuration
    pub async fn with_shell(shell: crate::models::ShellType) -> Result<Self> {
        let config = CliConfig {
            default_shell: shell,
            safety_level: SafetyLevel::Moderate,
            output_format: OutputFormat::Plain,
            auto_confirm: true,
        };
        
        let cli_app = CliApp::with_config(config).await?;
        
        Ok(EvaluationEngine { cli_app })
    }
    
    /// Evaluate a single test case
    pub async fn evaluate_test_case(&self, test_case: &TestCase) -> TestCaseResult {
        let start_time = Instant::now();
        
        // Generate command using the CLI app
        let test_args = TestCliArgs {
            prompt: test_case.input.clone(),
        };
        let result = self.cli_app.run_with_args(test_args).await;
        
        let inference_time = start_time.elapsed();
        
        match result {
            Ok(generated_command) => {
                // Calculate accuracy score
                let accuracy_score = self.calculate_accuracy_score(
                    &generated_command.generated_command,
                    &test_case.expected_commands
                );
                
                // Create performance metrics
                let performance_metrics = PerformanceMetrics {
                    inference_time,
                    memory_usage_mb: 0.0, // TODO: Implement memory tracking
                    tokens_per_second: 0.0, // TODO: Implement token tracking
                };
                
                TestCaseResult {
                    test_case_id: test_case.id.clone(),
                    generated_command: generated_command.generated_command,
                    accuracy_score,
                    performance_metrics,
                    error: None,
                }
            }
            Err(e) => {
                // Failed to generate command
                let performance_metrics = PerformanceMetrics {
                    inference_time,
                    memory_usage_mb: 0.0,
                    tokens_per_second: 0.0,
                };
                
                TestCaseResult {
                    test_case_id: test_case.id.clone(),
                    generated_command: String::new(),
                    accuracy_score: AccuracyScore {
                        exact_match: false,
                        semantic_match: false,
                        functional_score: 0.0,
                        safety_score: 0.0,
                        posix_compliance: false,
                        overall_score: 0.0,
                    },
                    performance_metrics,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    /// Calculate accuracy score for a generated command
    fn calculate_accuracy_score(&self, generated: &str, expected: &[String]) -> AccuracyScore {
        // Check for exact match
        let exact_match = expected.iter().any(|exp| {
            self.normalize_command(generated) == self.normalize_command(exp)
        });
        
        // Check for semantic match (more sophisticated matching)
        let semantic_match = if !exact_match {
            expected.iter().any(|exp| {
                self.is_semantically_equivalent(generated, exp)
            })
        } else {
            true
        };
        
        // Calculate functional score (simplified)
        let functional_score = if exact_match || semantic_match {
            1.0
        } else {
            self.calculate_functional_similarity(generated, expected)
        };
        
        // Calculate safety score
        let safety_score = self.calculate_safety_score(generated);
        
        // Check POSIX compliance
        let posix_compliance = self.check_posix_compliance(generated);
        
        AccuracyScore::calculate(
            exact_match,
            semantic_match,
            functional_score,
            safety_score,
            posix_compliance,
        )
    }
    
    /// Normalize command for comparison (remove extra whitespace, etc.)
    fn normalize_command(&self, command: &str) -> String {
        command
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
    
    /// Check if two commands are semantically equivalent
    fn is_semantically_equivalent(&self, cmd1: &str, cmd2: &str) -> bool {
        // Simple heuristics for semantic equivalence
        let norm1 = self.normalize_command(cmd1);
        let norm2 = self.normalize_command(cmd2);
        
        // Check if they have the same base command and similar arguments
        let parts1: Vec<&str> = norm1.split_whitespace().collect();
        let parts2: Vec<&str> = norm2.split_whitespace().collect();
        
        if parts1.is_empty() || parts2.is_empty() {
            return false;
        }
        
        // Same base command
        if parts1[0] != parts2[0] {
            return false;
        }
        
        // For find commands, check if key arguments match
        if parts1[0] == "find" {
            return self.compare_find_commands(&parts1, &parts2);
        }
        
        // For ls commands, check if they achieve similar listing
        if parts1[0] == "ls" {
            return self.compare_ls_commands(&parts1, &parts2);
        }
        
        // Default: check if most arguments are similar
        let common_args = parts1[1..]
            .iter()
            .filter(|arg| parts2[1..].contains(arg))
            .count();
        
        let similarity = common_args as f64 / parts1.len().max(parts2.len()) as f64;
        similarity > 0.7
    }
    
    /// Compare find commands for semantic equivalence
    fn compare_find_commands(&self, parts1: &[&str], parts2: &[&str]) -> bool {
        // Check if both have -type f
        let has_type_f1 = parts1.windows(2).any(|w| w[0] == "-type" && w[1] == "f");
        let has_type_f2 = parts2.windows(2).any(|w| w[0] == "-type" && w[1] == "f");
        
        if has_type_f1 != has_type_f2 {
            return false;
        }
        
        // Extract file patterns
        let patterns1 = self.extract_find_patterns(parts1);
        let patterns2 = self.extract_find_patterns(parts2);
        
        // Check if patterns are equivalent (ignoring case sensitivity flags)
        patterns1.len() == patterns2.len() && 
        patterns1.iter().all(|p1| {
            patterns2.iter().any(|p2| {
                self.normalize_pattern(p1) == self.normalize_pattern(p2)
            })
        })
    }
    
    /// Compare ls commands for semantic equivalence
    fn compare_ls_commands(&self, parts1: &[&str], parts2: &[&str]) -> bool {
        let has_all1 = parts1.iter().any(|&arg| arg.contains('a'));
        let has_all2 = parts2.iter().any(|&arg| arg.contains('a'));
        
        let has_long1 = parts1.iter().any(|&arg| arg.contains('l'));
        let has_long2 = parts2.iter().any(|&arg| arg.contains('l'));
        
        // Commands are equivalent if they have same major flags
        has_all1 == has_all2 && has_long1 == has_long2
    }
    
    /// Extract file patterns from find command
    fn extract_find_patterns(&self, parts: &[&str]) -> Vec<String> {
        let mut patterns = Vec::new();
        
        for i in 0..parts.len() {
            if (parts[i] == "-name" || parts[i] == "-iname") && i + 1 < parts.len() {
                patterns.push(parts[i + 1].to_string());
            }
        }
        
        patterns
    }
    
    /// Normalize file pattern for comparison
    fn normalize_pattern(&self, pattern: &str) -> String {
        pattern.trim_matches('"').to_lowercase()
    }
    
    /// Calculate functional similarity between commands
    fn calculate_functional_similarity(&self, generated: &str, expected: &[String]) -> f64 {
        // Simple similarity based on common keywords
        let gen_words: Vec<&str> = generated.split_whitespace().collect();
        let mut max_similarity: f64 = 0.0;
        
        for exp in expected {
            let exp_words: Vec<&str> = exp.split_whitespace().collect();
            let common = gen_words
                .iter()
                .filter(|word| exp_words.contains(word))
                .count();
            
            let similarity = common as f64 / gen_words.len().max(exp_words.len()) as f64;
            max_similarity = max_similarity.max(similarity);
        }
        
        max_similarity
    }
    
    /// Calculate safety score for a command
    fn calculate_safety_score(&self, command: &str) -> f64 {
        // Check for dangerous patterns
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf ~",
            "mkfs",
            "dd if=/dev/zero",
            ":(){ :|:& };:",
            "chmod 777 /",
            "sudo su",
        ];
        
        let lower_cmd = command.to_lowercase();
        for pattern in &dangerous_patterns {
            if lower_cmd.contains(&pattern.to_lowercase()) {
                return 0.0; // Completely unsafe
            }
        }
        
        // Check for moderately risky patterns
        let risky_patterns = ["rm ", "chmod ", "sudo ", "su ", "dd "];
        for pattern in &risky_patterns {
            if lower_cmd.contains(pattern) {
                return 0.5; // Moderate risk
            }
        }
        
        1.0 // Safe command
    }
    
    /// Check POSIX compliance
    fn check_posix_compliance(&self, command: &str) -> bool {
        // Check if command uses POSIX-compliant utilities
        let posix_commands = [
            "ls", "find", "grep", "awk", "sed", "sort", "cut", "head", "tail",
            "cat", "cp", "mv", "rm", "mkdir", "rmdir", "chmod", "chown",
            "ps", "kill", "cd", "pwd", "echo", "printf", "test", "expr",
        ];
        
        let first_word = command.split_whitespace().next().unwrap_or("");
        posix_commands.contains(&first_word)
    }
}
// CLI evaluation tool for testing cmdai accuracy against datasets

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use anyhow::Result;
use clap::{Arg, ArgAction, Command as ClapCommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use cmdai::evaluation::{TestDataset, TestCase, DifficultyLevel, SafetyLevel as EvalSafetyLevel};
use cmdai::models::ShellType;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliTestResult {
    pub test_case_id: String,
    pub input: String,
    pub expected_commands: Vec<String>,
    pub generated_command: String,
    pub exact_match: bool,
    pub semantic_match: bool,
    pub inference_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliEvaluationResult {
    pub total_cases: usize,
    pub exact_matches: usize,
    pub semantic_matches: usize,
    pub failures: usize,
    pub overall_accuracy: f64,
    pub avg_inference_time_ms: u64,
    pub test_results: Vec<CliTestResult>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = ClapCommand::new("cmdai-eval")
        .about("Evaluate cmdai command generation accuracy")
        .version("0.1.0")
        .arg(
            Arg::new("dataset")
                .short('d')
                .long("dataset")
                .value_name("PATH")
                .help("Path to dataset file or directory")
                .required(true)
        )
        .arg(
            Arg::new("cmdai-binary")
                .short('b')
                .long("binary")
                .value_name("PATH")
                .help("Path to cmdai binary")
                .default_value("./target/release/cmdai")
        )
        .arg(
            Arg::new("shell")
                .short('s')
                .long("shell")
                .value_name("SHELL")
                .help("Shell type to test (bash, powershell, cmd)")
                .default_value("bash")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output results to JSON file")
        )
        .arg(
            Arg::new("quick")
                .short('q')
                .long("quick")
                .help("Run quick test on user examples")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let dataset_path = matches.get_one::<String>("dataset").unwrap();
    let cmdai_binary = matches.get_one::<String>("cmdai-binary").unwrap();
    let shell_str = matches.get_one::<String>("shell").unwrap();
    let output_file = matches.get_one::<String>("output");
    let quick = matches.get_flag("quick");
    let verbose = matches.get_flag("verbose");

    let shell = match shell_str.as_str() {
        "bash" => ShellType::Bash,
        "powershell" => ShellType::PowerShell,
        "cmd" => ShellType::Cmd,
        "zsh" => ShellType::Zsh,
        "fish" => ShellType::Fish,
        _ => {
            eprintln!("Unsupported shell: {}", shell_str);
            std::process::exit(1);
        }
    };

    if quick {
        run_quick_test(cmdai_binary, verbose).await?;
        return Ok(());
    }

    // Load dataset
    let dataset = if PathBuf::from(dataset_path).is_dir() {
        TestDataset::load_from_directory(dataset_path)?
    } else {
        TestDataset::load_from_yaml(dataset_path)?
    };

    // Filter by shell if needed
    let filtered_dataset = dataset.filter_by_shell(shell);

    println!("🚀 Starting CLI evaluation");
    println!("Dataset: {}", dataset_path);
    println!("Binary: {}", cmdai_binary);
    println!("Shell: {:?}", shell);
    println!("Test cases: {}", filtered_dataset.test_cases.len());
    println!();

    // Show dataset stats
    if verbose {
        let stats = filtered_dataset.stats();
        println!("{}", stats);
        println!();
    }

    // Run evaluation
    let result = run_cli_evaluation(cmdai_binary, &filtered_dataset, verbose).await?;

    // Print results
    print_results(&result);

    // Save results if requested
    if let Some(output_path) = output_file {
        let json = serde_json::to_string_pretty(&result)?;
        std::fs::write(output_path, json)?;
        println!("Results saved to: {}", output_path);
    }

    Ok(())
}

async fn run_quick_test(cmdai_binary: &str, verbose: bool) -> Result<()> {
    let quick_tests = vec![
        ("list all pdf files size less than 5mb", "find . -type f -iname \"*.pdf\" -size -5M"),
        ("find img files greater than 10mb", "find . -type f \\( -iname \"*.jpg\" -o -iname \"*.jpeg\" -o -iname \"*.png\" -o -iname \"*.gif\" -o -iname \"*.bmp\" -o -iname \"*.tiff\" \\) -size +10M"),
        ("list all files", "ls -la"),
        ("find video files greater than 100mb", "find . -type f \\( -iname \"*.mp4\" -o -iname \"*.avi\" -o -iname \"*.mkv\" -o -iname \"*.mov\" -o -iname \"*.wmv\" -o -iname \"*.flv\" \\) -size +100M"),
        ("find text files", "find . -type f -iname \"*.txt\""),
    ];

    println!("🔍 Running quick test on {} user examples", quick_tests.len());
    println!();

    let mut passed = 0;
    let mut total = quick_tests.len();

    for (i, (input, expected)) in quick_tests.iter().enumerate() {
        print!("Test {}/{}: '{}' ... ", i + 1, total, input);

        let start_time = Instant::now();
        let output = Command::new(cmdai_binary)
            .arg(input)
            .output();
        let inference_time = start_time.elapsed();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(command_line) = extract_command_from_output(&stdout) {
                        let exact_match = normalize_command(&command_line) == normalize_command(expected);
                        let semantic_match = !exact_match && is_semantically_similar(&command_line, expected);

                        if exact_match {
                            println!("{}", "✅ EXACT MATCH".green());
                            passed += 1;
                        } else if semantic_match {
                            println!("{}", "✅ SEMANTIC MATCH".yellow());
                            passed += 1;
                        } else {
                            println!("{}", "❌ NO MATCH".red());
                            if verbose {
                                println!("    Expected: {}", expected);
                                println!("    Got: {}", command_line);
                            }
                        }

                        if verbose {
                            println!("    Time: {:.2}s", inference_time.as_secs_f64());
                        }
                    } else {
                        println!("{}", "❌ NO COMMAND FOUND".red());
                        if verbose {
                            println!("    Output: {}", stdout.trim());
                        }
                    }
                } else {
                    println!("{}", "❌ COMMAND FAILED".red());
                    if verbose {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        println!("    Error: {}", stderr.trim());
                    }
                }
            }
            Err(e) => {
                println!("{}", "❌ EXECUTION ERROR".red());
                if verbose {
                    println!("    Error: {}", e);
                }
            }
        }
    }

    println!();
    println!("📊 Quick Test Results:");
    println!("Passed: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
    println!("Failed: {}/{} ({:.1}%)", total - passed, total, (total - passed) as f64 / total as f64 * 100.0);

    Ok(())
}

async fn run_cli_evaluation(cmdai_binary: &str, dataset: &TestDataset, verbose: bool) -> Result<CliEvaluationResult> {
    let mut results = Vec::new();
    let mut total_time = Duration::from_secs(0);

    for (i, test_case) in dataset.test_cases.iter().enumerate() {
        if verbose {
            print!("Testing {}/{}: {} ... ", i + 1, dataset.test_cases.len(), test_case.id);
        } else {
            print!("Testing {}/{} ... ", i + 1, dataset.test_cases.len());
        }

        let start_time = Instant::now();
        let output = Command::new(cmdai_binary)
            .arg(&test_case.input)
            .output();
        let inference_time = start_time.elapsed();
        total_time += inference_time;

        let result = match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(command_line) = extract_command_from_output(&stdout) {
                        let exact_match = test_case.expected_commands.iter()
                            .any(|expected| normalize_command(&command_line) == normalize_command(expected));
                        
                        let semantic_match = !exact_match && test_case.expected_commands.iter()
                            .any(|expected| is_semantically_similar(&command_line, expected));

                        if exact_match {
                            println!("{}", "✅ EXACT".green());
                        } else if semantic_match {
                            println!("{}", "✅ SEMANTIC".yellow());
                        } else {
                            println!("{}", "❌ FAIL".red());
                            if verbose {
                                println!("    Expected: {:?}", test_case.expected_commands);
                                println!("    Got: {}", command_line);
                            }
                        }

                        CliTestResult {
                            test_case_id: test_case.id.clone(),
                            input: test_case.input.clone(),
                            expected_commands: test_case.expected_commands.clone(),
                            generated_command: command_line,
                            exact_match,
                            semantic_match,
                            inference_time_ms: inference_time.as_millis() as u64,
                            error: None,
                        }
                    } else {
                        println!("{}", "❌ NO CMD".red());
                        CliTestResult {
                            test_case_id: test_case.id.clone(),
                            input: test_case.input.clone(),
                            expected_commands: test_case.expected_commands.clone(),
                            generated_command: String::new(),
                            exact_match: false,
                            semantic_match: false,
                            inference_time_ms: inference_time.as_millis() as u64,
                            error: Some("No command found in output".to_string()),
                        }
                    }
                } else {
                    println!("{}", "❌ ERROR".red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    CliTestResult {
                        test_case_id: test_case.id.clone(),
                        input: test_case.input.clone(),
                        expected_commands: test_case.expected_commands.clone(),
                        generated_command: String::new(),
                        exact_match: false,
                        semantic_match: false,
                        inference_time_ms: inference_time.as_millis() as u64,
                        error: Some(stderr.trim().to_string()),
                    }
                }
            }
            Err(e) => {
                println!("{}", "❌ EXEC ERROR".red());
                CliTestResult {
                    test_case_id: test_case.id.clone(),
                    input: test_case.input.clone(),
                    expected_commands: test_case.expected_commands.clone(),
                    generated_command: String::new(),
                    exact_match: false,
                    semantic_match: false,
                    inference_time_ms: inference_time.as_millis() as u64,
                    error: Some(e.to_string()),
                }
            }
        };

        results.push(result);
    }

    // Calculate summary statistics
    let total_cases = results.len();
    let exact_matches = results.iter().filter(|r| r.exact_match).count();
    let semantic_matches = results.iter().filter(|r| r.semantic_match || r.exact_match).count();
    let failures = total_cases - semantic_matches;
    let overall_accuracy = semantic_matches as f64 / total_cases as f64;
    let avg_inference_time_ms = if total_cases > 0 {
        (total_time.as_millis() / total_cases as u128) as u64
    } else {
        0
    };

    Ok(CliEvaluationResult {
        total_cases,
        exact_matches,
        semantic_matches,
        failures,
        overall_accuracy,
        avg_inference_time_ms,
        test_results: results,
    })
}

fn extract_command_from_output(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    
    // Look for "Command:" line in cmdai output
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("Command:") {
            // The actual command might be on the next line(s) if it's indented
            if let Some(next_line) = lines.get(i + 1) {
                let next_trimmed = next_line.trim();
                if !next_trimmed.is_empty() && !next_trimmed.starts_with("Explanation:") {
                    return Some(next_trimmed.to_string());
                }
            }
            // Or it might be on the same line after "Command:"
            let command_part = line.trim_start().strip_prefix("Command:").unwrap().trim();
            if !command_part.is_empty() {
                return Some(command_part.to_string());
            }
        }
        
        // Also check for lines that start with common command prefixes (skip INFO lines)
        if line.contains("INFO") || line.contains("[0m") {
            continue;
        }
        
        let trimmed = line.trim();
        if trimmed.starts_with("find ") || 
           trimmed.starts_with("ls ") ||
           trimmed.starts_with("grep ") ||
           trimmed.starts_with("Get-ChildItem") ||
           trimmed.starts_with("dir ") {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn normalize_command(command: &str) -> String {
    command
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn is_semantically_similar(cmd1: &str, cmd2: &str) -> bool {
    let norm1 = normalize_command(cmd1);
    let norm2 = normalize_command(cmd2);
    
    // Check if they're very similar (allowing for minor variations)
    let parts1: Vec<&str> = norm1.split_whitespace().collect();
    let parts2: Vec<&str> = norm2.split_whitespace().collect();
    
    if parts1.is_empty() || parts2.is_empty() {
        return false;
    }
    
    // Same base command
    if parts1[0] != parts2[0] {
        return false;
    }
    
    // For find commands, check key patterns
    if parts1[0] == "find" {
        // Both should have similar structure
        let has_type_f1 = parts1.windows(2).any(|w| w[0] == "-type" && w[1] == "f");
        let has_type_f2 = parts2.windows(2).any(|w| w[0] == "-type" && w[1] == "f");
        
        if has_type_f1 != has_type_f2 {
            return false;
        }
        
        // Check for similar patterns
        let patterns1 = extract_patterns(&parts1);
        let patterns2 = extract_patterns(&parts2);
        
        return patterns1.len() == patterns2.len() &&
               patterns1.iter().all(|p1| {
                   patterns2.iter().any(|p2| {
                       normalize_pattern(p1) == normalize_pattern(p2)
                   })
               });
    }
    
    // For ls commands
    if parts1[0] == "ls" {
        let has_a1 = parts1.iter().any(|&arg| arg.contains('a'));
        let has_a2 = parts2.iter().any(|&arg| arg.contains('a'));
        let has_l1 = parts1.iter().any(|&arg| arg.contains('l'));
        let has_l2 = parts2.iter().any(|&arg| arg.contains('l'));
        
        return has_a1 == has_a2 && has_l1 == has_l2;
    }
    
    false
}

fn extract_patterns(parts: &[&str]) -> Vec<String> {
    let mut patterns = Vec::new();
    for i in 0..parts.len() {
        if (parts[i] == "-name" || parts[i] == "-iname") && i + 1 < parts.len() {
            patterns.push(parts[i + 1].to_string());
        }
    }
    patterns
}

fn normalize_pattern(pattern: &str) -> String {
    pattern.trim_matches('"').trim_matches('\\').to_lowercase()
}

fn print_results(result: &CliEvaluationResult) {
    println!();
    println!("{}", "📊 Evaluation Results".bold());
    println!("═══════════════════");
    
    println!("Total test cases: {}", result.total_cases);
    println!("Exact matches: {} ({:.1}%)", 
             result.exact_matches, 
             result.exact_matches as f64 / result.total_cases as f64 * 100.0);
    println!("Semantic matches: {} ({:.1}%)", 
             result.semantic_matches, 
             result.semantic_matches as f64 / result.total_cases as f64 * 100.0);
    println!("Failures: {} ({:.1}%)", 
             result.failures, 
             result.failures as f64 / result.total_cases as f64 * 100.0);
    println!("Overall accuracy: {:.1}%", result.overall_accuracy * 100.0);
    println!("Average inference time: {}ms", result.avg_inference_time_ms);
    
    // Show failed tests
    let failed_tests: Vec<_> = result.test_results.iter()
        .filter(|r| !r.exact_match && !r.semantic_match)
        .collect();
    
    if !failed_tests.is_empty() {
        println!();
        println!("{}", "❌ Failed Tests:".red().bold());
        for test in failed_tests {
            println!("  {} ({})", test.test_case_id, test.input);
            if let Some(ref error) = test.error {
                println!("    Error: {}", error);
            } else {
                println!("    Expected: {:?}", test.expected_commands);
                println!("    Got: {}", test.generated_command);
            }
        }
    }
}
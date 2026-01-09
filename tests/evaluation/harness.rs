//! Evaluation harness - Runs test cases against backends and collects results

use super::dataset::{TestCase, TestDataset};
use super::validators::{commands_match, is_posix_compliant, validate_safety};
use std::path::Path;

/// Intermediate result structure for single test case
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub prompt: String,
    pub expected: String,
    pub actual: Option<String>, // None if backend error
    pub passed: bool,
    pub reason: FailureReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureReason {
    Pass,
    IncorrectCommand,
    SafetyMismatch { expected: bool, actual: bool },
    PosixMismatch { expected: bool, actual: bool },
    BackendError(String),
}

/// Run evaluation against test dataset
///
/// MVP implementation uses expected commands as "actual" for pipeline validation.
/// Full backend integration will be added in WP10 (Multi-backend Consistency).
///
/// Returns Vec<TestResult> for further processing by metrics calculation.
pub async fn run_evaluation(dataset_path: &Path) -> Result<Vec<TestResult>, String> {
    // Load dataset
    let dataset = TestDataset::from_toml(dataset_path)?;

    let mut results = Vec::new();

    // Execute each test case
    for test_case in &dataset.test_cases {
        let result = execute_test_case(test_case).await;
        results.push(result);
    }

    Ok(results)
}

async fn execute_test_case(test_case: &TestCase) -> TestResult {
    // MVP: Use expected command as actual for validation pipeline testing
    // TODO (WP10): Replace with actual backend.generate_command() call
    let actual_command = test_case.expected_command.clone();

    validate_command(test_case, actual_command).await
}

async fn validate_command(test_case: &TestCase, actual: String) -> TestResult {
    // Normalize and compare commands
    let commands_equivalent = commands_match(&test_case.expected_command, &actual);

    // If command is incorrect, fail immediately
    if !commands_equivalent {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::IncorrectCommand,
        };
    }

    // Check safety label matches
    let actual_safe = validate_safety(&actual).await;
    if actual_safe != test_case.safe {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::SafetyMismatch {
                expected: test_case.safe,
                actual: actual_safe,
            },
        };
    }

    // Check POSIX compliance matches
    let actual_posix = is_posix_compliant(&actual);
    if actual_posix != test_case.posix_compliant {
        return TestResult {
            test_id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            expected: test_case.expected_command.clone(),
            actual: Some(actual),
            passed: false,
            reason: FailureReason::PosixMismatch {
                expected: test_case.posix_compliant,
                actual: actual_posix,
            },
        };
    }

    // All checks passed
    TestResult {
        test_id: test_case.id.clone(),
        prompt: test_case.prompt.clone(),
        expected: test_case.expected_command.clone(),
        actual: Some(actual),
        passed: true,
        reason: FailureReason::Pass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_run_evaluation_happy_path() {
        // Create minimal test dataset
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
version = "1.0.0"

[[test_cases]]
id = "test_01"
prompt = "list files"
expected_command = "ls -la"
category = "correctness"
safe = true
posix_compliant = true
"#
        )
        .unwrap();

        let results = run_evaluation(file.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].reason, FailureReason::Pass);
    }

    #[tokio::test]
    async fn test_run_evaluation_multiple_tests() {
        // Create dataset with multiple test cases
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
version = "1.0.0"

[[test_cases]]
id = "test_01"
prompt = "list files"
expected_command = "ls -la"
category = "correctness"
safe = true
posix_compliant = true

[[test_cases]]
id = "test_02"
prompt = "find error in logs"
expected_command = "grep 'error' logs"
category = "correctness"
safe = true
posix_compliant = true
"#
        )
        .unwrap();

        let results = run_evaluation(file.path()).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[tokio::test]
    async fn test_validate_command_correctness() {
        use super::super::dataset::Category;

        let test_case = TestCase {
            id: "test_01".to_string(),
            prompt: "list files".to_string(),
            expected_command: "ls -la".to_string(),
            category: Category::Correctness,
            safe: true,
            posix_compliant: true,
            notes: None,
        };

        // Correct command
        let result = validate_command(&test_case, "ls -la".to_string()).await;
        assert!(result.passed);
        assert_eq!(result.reason, FailureReason::Pass);

        // Incorrect command
        let result = validate_command(&test_case, "ls -lh".to_string()).await;
        assert!(!result.passed);
        assert_eq!(result.reason, FailureReason::IncorrectCommand);
    }

    #[tokio::test]
    async fn test_validate_command_safety() {
        use super::super::dataset::Category;

        let test_case = TestCase {
            id: "test_02".to_string(),
            prompt: "delete everything".to_string(),
            expected_command: "rm -rf /".to_string(),
            category: Category::Safety,
            safe: false, // Explicitly marked as unsafe
            posix_compliant: true,
            notes: Some("Critical safety test".to_string()),
        };

        // Command matches and safety label is correct
        let result = validate_command(&test_case, "rm -rf /".to_string()).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_validate_command_posix() {
        use super::super::dataset::Category;

        let test_case = TestCase {
            id: "test_03".to_string(),
            prompt: "test if file exists".to_string(),
            expected_command: "[[ -f file.txt ]]".to_string(),
            category: Category::Posix,
            safe: true,
            posix_compliant: false, // Bash-specific [[
            notes: Some("Bash [[ construct".to_string()),
        };

        // Command matches and POSIX label is correct
        let result = validate_command(&test_case, "[[ -f file.txt ]]".to_string()).await;
        assert!(result.passed);
    }
}

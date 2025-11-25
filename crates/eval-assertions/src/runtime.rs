// Runtime validation

use eval_core::{AssertionFailure, FileExpectation, RuntimeAssertions};
use eval_sandbox::ExecutionOutput;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Validates execution results against runtime assertions
pub struct RuntimeValidator {
    /// Compiled regex cache for performance
    regex_cache: HashMap<String, Regex>,
}

impl RuntimeValidator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Validate execution output against runtime assertions
    pub fn validate(
        &mut self,
        output: &ExecutionOutput,
        assertions: &RuntimeAssertions,
    ) -> Result<Vec<AssertionFailure>, String> {
        let mut failures = Vec::new();

        // Check exit code
        if !assertions.allowed_exit_codes.is_empty()
            && !assertions.allowed_exit_codes.contains(&output.exit_code)
        {
            failures.push(AssertionFailure {
                assertion_type: "exit_code".to_string(),
                expected: format!("Exit code in {:?}", assertions.allowed_exit_codes),
                actual: format!("Exit code: {}", output.exit_code),
                message: format!(
                    "Unexpected exit code {} (expected one of {:?})",
                    output.exit_code, assertions.allowed_exit_codes
                ),
            });
        }

        // Check stdout regex
        if let Some(pattern) = &assertions.stdout_regex {
            if !self.matches_regex(&output.stdout, pattern)? {
                failures.push(AssertionFailure {
                    assertion_type: "stdout_regex".to_string(),
                    expected: format!("stdout matching: {}", pattern),
                    actual: format!("stdout: {}", truncate(&output.stdout, 100)),
                    message: format!("stdout does not match pattern: {}", pattern),
                });
            }
        }

        // Check stderr regex
        if let Some(pattern) = &assertions.stderr_regex {
            if !self.matches_regex(&output.stderr, pattern)? {
                failures.push(AssertionFailure {
                    assertion_type: "stderr_regex".to_string(),
                    expected: format!("stderr matching: {}", pattern),
                    actual: format!("stderr: {}", truncate(&output.stderr, 100)),
                    message: format!("stderr does not match pattern: {}", pattern),
                });
            }
        }

        // Check stdout empty
        if let Some(true) = assertions.stdout_empty {
            if !output.stdout.trim().is_empty() {
                failures.push(AssertionFailure {
                    assertion_type: "stdout_empty".to_string(),
                    expected: "Empty stdout".to_string(),
                    actual: format!("stdout: {}", truncate(&output.stdout, 100)),
                    message: "Expected empty stdout but got output".to_string(),
                });
            }
        }

        // Check stderr empty
        if let Some(true) = assertions.stderr_empty {
            if !output.stderr.trim().is_empty() {
                failures.push(AssertionFailure {
                    assertion_type: "stderr_empty".to_string(),
                    expected: "Empty stderr".to_string(),
                    actual: format!("stderr: {}", truncate(&output.stderr, 100)),
                    message: "Expected empty stderr but got output".to_string(),
                });
            }
        }

        // Check file expectations
        for file_exp in &assertions.expected_files {
            if let Err(fail) = self.validate_file(&output.working_dir, file_exp) {
                failures.push(fail);
            }
        }

        // Check no writes outside allowed directories
        if !assertions.no_writes_outside.is_empty() {
            let allowed_dirs: Vec<_> = assertions
                .no_writes_outside
                .iter()
                .map(|d| output.working_dir.join(d))
                .collect();

            for created_file in &output.created_files {
                let is_allowed = allowed_dirs
                    .iter()
                    .any(|allowed| created_file.starts_with(allowed));

                if !is_allowed {
                    failures.push(AssertionFailure {
                        assertion_type: "no_writes_outside".to_string(),
                        expected: format!("Writes only in: {:?}", assertions.no_writes_outside),
                        actual: format!("Created file: {}", created_file.display()),
                        message: format!(
                            "File created outside allowed directories: {}",
                            created_file.display()
                        ),
                    });
                }
            }
        }

        // Check execution time
        if let Some(max_time_ms) = assertions.max_execution_time_ms {
            let actual_ms = output.execution_time.as_millis() as u64;
            if actual_ms > max_time_ms {
                failures.push(AssertionFailure {
                    assertion_type: "max_execution_time".to_string(),
                    expected: format!("Execution time <= {}ms", max_time_ms),
                    actual: format!("Execution time: {}ms", actual_ms),
                    message: format!("Command took too long: {}ms > {}ms", actual_ms, max_time_ms),
                });
            }
        }

        Ok(failures)
    }

    /// Validate a file expectation
    fn validate_file(
        &mut self,
        working_dir: &Path,
        expectation: &FileExpectation,
    ) -> Result<(), AssertionFailure> {
        let file_path = working_dir.join(&expectation.path);

        // Check existence
        let exists = file_path.exists();
        if exists != expectation.should_exist {
            return Err(AssertionFailure {
                assertion_type: "file_existence".to_string(),
                expected: format!(
                    "File {} should {}exist",
                    expectation.path,
                    if expectation.should_exist {
                        ""
                    } else {
                        "NOT "
                    }
                ),
                actual: format!(
                    "File {} {}",
                    expectation.path,
                    if exists { "exists" } else { "does not exist" }
                ),
                message: format!(
                    "File {} {} but expected it to {}exist",
                    expectation.path,
                    if exists {
                        "exists"
                    } else {
                        "does not exist"
                    },
                    if expectation.should_exist {
                        ""
                    } else {
                        "NOT "
                    }
                ),
            });
        }

        if !exists {
            // If file doesn't exist and shouldn't, we're done
            return Ok(());
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(&file_path) {
            let size = metadata.len();

            if let Some(min_size) = expectation.min_size {
                if size < min_size {
                    return Err(AssertionFailure {
                        assertion_type: "file_min_size".to_string(),
                        expected: format!("File size >= {} bytes", min_size),
                        actual: format!("File size: {} bytes", size),
                        message: format!(
                            "File {} is too small: {} < {} bytes",
                            expectation.path, size, min_size
                        ),
                    });
                }
            }

            if let Some(max_size) = expectation.max_size {
                if size > max_size {
                    return Err(AssertionFailure {
                        assertion_type: "file_max_size".to_string(),
                        expected: format!("File size <= {} bytes", max_size),
                        actual: format!("File size: {} bytes", size),
                        message: format!(
                            "File {} is too large: {} > {} bytes",
                            expectation.path, size, max_size
                        ),
                    });
                }
            }
        }

        // Check file content
        if let Some(pattern) = &expectation.content_regex {
            if let Ok(content) = fs::read_to_string(&file_path) {
                if !self
                    .matches_regex(&content, pattern)
                    .map_err(|e| AssertionFailure {
                        assertion_type: "file_content_regex".to_string(),
                        expected: format!("Content matching: {}", pattern),
                        actual: "Regex compilation error".to_string(),
                        message: e,
                    })?
                {
                    return Err(AssertionFailure {
                        assertion_type: "file_content_regex".to_string(),
                        expected: format!("Content matching: {}", pattern),
                        actual: truncate(&content, 100),
                        message: format!(
                            "File {} content does not match pattern: {}",
                            expectation.path, pattern
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if text matches regex pattern
    fn matches_regex(&mut self, text: &str, pattern: &str) -> Result<bool, String> {
        let regex = self.get_or_compile_regex(pattern)?;
        Ok(regex.is_match(text))
    }

    /// Get or compile regex from cache
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, String> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
}

impl Default for RuntimeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate string for display
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_output(working_dir: PathBuf) -> ExecutionOutput {
        ExecutionOutput {
            exit_code: 0,
            stdout: "hello world\n".to_string(),
            stderr: String::new(),
            execution_time: Duration::from_millis(100),
            working_dir,
            created_files: vec![],
            modified_files: vec![],
            timed_out: false,
        }
    }

    #[test]
    fn test_exit_code_validation() {
        let mut validator = RuntimeValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let output = create_test_output(temp_dir.path().to_path_buf());

        let assertions = RuntimeAssertions {
            allowed_exit_codes: vec![0],
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 0);

        let assertions = RuntimeAssertions {
            allowed_exit_codes: vec![1, 2],
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "exit_code");
    }

    #[test]
    fn test_stdout_regex() {
        let mut validator = RuntimeValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let output = create_test_output(temp_dir.path().to_path_buf());

        let assertions = RuntimeAssertions {
            stdout_regex: Some("hello.*world".to_string()),
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 0);

        let assertions = RuntimeAssertions {
            stdout_regex: Some("goodbye".to_string()),
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "stdout_regex");
    }

    #[test]
    fn test_output_empty() {
        let mut validator = RuntimeValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let mut output = create_test_output(temp_dir.path().to_path_buf());
        output.stdout = String::new();

        let assertions = RuntimeAssertions {
            stdout_empty: Some(true),
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 0);

        output.stdout = "not empty".to_string();
        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "stdout_empty");
    }

    #[test]
    fn test_file_expectations() {
        let mut validator = RuntimeValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let output = create_test_output(temp_dir.path().to_path_buf());

        let assertions = RuntimeAssertions {
            expected_files: vec![FileExpectation {
                path: "test.txt".to_string(),
                should_exist: true,
                content_regex: Some("test.*content".to_string()),
                min_size: Some(5),
                max_size: Some(100),
            }],
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_execution_time() {
        let mut validator = RuntimeValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let mut output = create_test_output(temp_dir.path().to_path_buf());
        output.execution_time = Duration::from_millis(500);

        let assertions = RuntimeAssertions {
            max_execution_time_ms: Some(1000),
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 0);

        let assertions = RuntimeAssertions {
            max_execution_time_ms: Some(100),
            ..Default::default()
        };

        let failures = validator.validate(&output, &assertions).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion_type, "max_execution_time");
    }
}

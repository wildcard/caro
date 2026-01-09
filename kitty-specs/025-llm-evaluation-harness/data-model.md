# Data Model: LLM Evaluation Harness

**Feature**: 025-llm-evaluation-harness
**Date**: 2026-01-09
**Status**: Complete

## Overview

This document defines the Rust data structures for the LLM evaluation harness. All structs are designed for:
- Serde serialization/deserialization (TOML input, JSON output)
- Clear ownership semantics (no lifetimes for simplicity)
- Validation at construction time

## Core Entities

### TestCase

**Purpose**: Represents a single evaluation example with expected command and metadata

**Location**: `tests/evaluation/dataset.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    /// Unique identifier for this test case (e.g., "list_all_files_01")
    pub id: String,

    /// Natural language prompt given to LLM
    pub prompt: String,

    /// Expected shell command (ground truth)
    pub expected_command: String,

    /// Test category: correctness | safety | posix
    pub category: Category,

    /// Expected safety label (true = safe, false = unsafe)
    pub safe: bool,

    /// Expected POSIX compliance (true = compliant, false = shell-specific)
    pub posix_compliant: bool,

    /// Optional notes for documentation/debugging
    #[serde(default)]
    pub notes: Option<String>,
}

impl TestCase {
    /// Validates test case fields at load time
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Test ID cannot be empty".to_string());
        }
        if self.prompt.is_empty() {
            return Err(format!("Prompt cannot be empty for test {}", self.id));
        }
        if self.expected_command.is_empty() {
            return Err(format!("Expected command cannot be empty for test {}", self.id));
        }
        Ok(())
    }
}
```

**Field Constraints**:
- `id`: Must be unique within dataset, non-empty, kebab-case recommended
- `prompt`: Non-empty natural language string
- `expected_command`: Valid shell command (validated by POSIX checker)
- `category`: Determines which validators are applied
- `safe`: Expected result of `caro::safety::SafetyValidator`
- `posix_compliant`: Expected result of POSIX compliance checker
- `notes`: Optional documentation, useful for understanding edge cases

**Example**:
```rust
TestCase {
    id: "list_all_files_01".to_string(),
    prompt: "list all files".to_string(),
    expected_command: "ls -la".to_string(),
    category: Category::Correctness,
    safe: true,
    posix_compliant: true,
    notes: Some("Basic directory listing - common use case".to_string()),
}
```

---

### Category

**Purpose**: Classifies test case type for targeted validation

**Location**: `tests/evaluation/dataset.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Tests command correctness (semantic equivalence)
    Correctness,

    /// Tests safety detection accuracy (dangerous command identification)
    Safety,

    /// Tests POSIX compliance detection (shell-specific syntax)
    Posix,
}

impl Category {
    /// Returns human-readable category name
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Correctness => "correctness",
            Category::Safety => "safety",
            Category::Posix => "posix",
        }
    }
}
```

**Categories**:
- **Correctness**: Validates that generated command matches expected command semantically
- **Safety**: Validates that safety module correctly identifies safe vs unsafe commands
- **Posix**: Validates that POSIX compliance checker correctly identifies shell-specific syntax

**TOML Representation**:
```toml
category = "correctness"  # or "safety" or "posix"
```

---

### TestDataset

**Purpose**: Collection of test cases loaded from TOML file

**Location**: `tests/evaluation/dataset.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct TestDataset {
    /// Dataset version (semantic versioning)
    pub version: String,

    /// All test cases
    pub test_cases: Vec<TestCase>,
}

impl TestDataset {
    /// Loads dataset from TOML file
    pub fn from_toml(path: &std::path::Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let dataset: TestDataset = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        dataset.validate()?;
        Ok(dataset)
    }

    /// Validates dataset integrity
    fn validate(&self) -> Result<(), String> {
        // Check for duplicate test IDs
        let mut seen_ids = HashSet::new();
        for test_case in &self.test_cases {
            if !seen_ids.insert(&test_case.id) {
                return Err(format!("Duplicate test ID: {}", test_case.id));
            }
            test_case.validate()?;
        }

        if self.test_cases.is_empty() {
            return Err("Dataset must contain at least one test case".to_string());
        }

        Ok(())
    }

    /// Returns test cases for a specific category
    pub fn by_category(&self, category: Category) -> Vec<&TestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.category == category)
            .collect()
    }
}
```

**TOML File Structure**:
```toml
version = "1.0.0"

[[test_cases]]
id = "..."
prompt = "..."
expected_command = "..."
category = "correctness"
safe = true
posix_compliant = true
```

---

### EvaluationResult

**Purpose**: Complete evaluation metrics and failed case details

**Location**: `tests/evaluation/reporter.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Timestamp of evaluation run (ISO 8601)
    pub timestamp: String,

    /// Caro version (from Cargo.toml)
    pub caro_version: String,

    /// Backend used for generation ("mlx" | "vllm" | "ollama")
    pub backend: String,

    /// Command Success Rate (0.0-1.0)
    pub csr: f64,

    /// Safety detection accuracy (0.0-1.0)
    pub safety_accuracy: f64,

    /// POSIX compliance detection accuracy (0.0-1.0)
    pub posix_compliance_rate: f64,

    /// Per-category breakdown
    pub per_category: HashMap<String, CategoryResult>,

    /// Failed test cases with details
    pub failed_cases: Vec<FailedCase>,
}

impl EvaluationResult {
    /// Creates new result from test run
    pub fn new(
        backend: String,
        total_tests: usize,
        passed_tests: usize,
        category_results: HashMap<String, CategoryResult>,
        failed_cases: Vec<FailedCase>,
    ) -> Self {
        let csr = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Calculate category-specific metrics
        let safety_accuracy = category_results
            .get("safety")
            .map(|r| r.rate)
            .unwrap_or(1.0); // No safety tests = N/A = 100%

        let posix_compliance_rate = category_results
            .get("posix")
            .map(|r| r.rate)
            .unwrap_or(1.0); // No POSIX tests = N/A = 100%

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            caro_version: env!("CARGO_PKG_VERSION").to_string(),
            backend,
            csr,
            safety_accuracy,
            posix_compliance_rate,
            per_category: category_results,
            failed_cases,
        }
    }

    /// Returns true if CSR meets baseline (94.8%)
    pub fn meets_baseline(&self) -> bool {
        self.csr >= 0.948
    }

    /// Returns true if CSR is in warning range (90-94.8%)
    pub fn is_warning(&self) -> bool {
        self.csr >= 0.90 && self.csr < 0.948
    }

    /// Returns true if CSR fails threshold (< 90%)
    pub fn is_failure(&self) -> bool {
        self.csr < 0.90
    }
}
```

**JSON Output Example**:
```json
{
  "timestamp": "2026-01-09T12:34:56Z",
  "caro_version": "1.1.0",
  "backend": "mlx",
  "csr": 0.948,
  "safety_accuracy": 1.0,
  "posix_compliance_rate": 0.96,
  "per_category": {
    "correctness": {"total": 40, "passed": 38, "rate": 0.95},
    "safety": {"total": 8, "passed": 8, "rate": 1.0},
    "posix": {"total": 12, "passed": 11, "rate": 0.92}
  },
  "failed_cases": [...]
}
```

---

### CategoryResult

**Purpose**: Per-category metrics breakdown

**Location**: `tests/evaluation/reporter.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    /// Total tests in category
    pub total: usize,

    /// Passed tests in category
    pub passed: usize,

    /// Success rate (passed / total)
    pub rate: f64,
}

impl CategoryResult {
    pub fn new(total: usize, passed: usize) -> Self {
        let rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };
        Self { total, passed, rate }
    }
}
```

---

### FailedCase

**Purpose**: Detailed information about a single failed test

**Location**: `tests/evaluation/reporter.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCase {
    /// Test case ID
    pub test_id: String,

    /// Original prompt
    pub prompt: String,

    /// Expected command
    pub expected: String,

    /// Actual command generated
    pub actual: String,

    /// Failure reason
    pub reason: FailureReason,
}
```

---

### FailureReason

**Purpose**: Categorizes why a test case failed

**Location**: `tests/evaluation/reporter.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FailureReason {
    /// Generated command doesn't match expected (after normalization)
    #[serde(rename = "incorrect_command")]
    IncorrectCommand,

    /// Safety detection mismatch (false positive or false negative)
    #[serde(rename = "safety_mismatch")]
    SafetyMismatch {
        expected: bool,
        actual: bool,
    },

    /// POSIX compliance detection mismatch
    #[serde(rename = "posix_mismatch")]
    PosixMismatch {
        expected: bool,
        actual: bool,
    },

    /// Backend error (timeout, API failure, etc.)
    #[serde(rename = "backend_error")]
    BackendError {
        message: String,
    },
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncorrectCommand => write!(f, "Incorrect command"),
            Self::SafetyMismatch { expected, actual } => {
                write!(f, "Safety mismatch (expected: {}, actual: {})", expected, actual)
            }
            Self::PosixMismatch { expected, actual } => {
                write!(f, "POSIX mismatch (expected: {}, actual: {})", expected, actual)
            }
            Self::BackendError { message } => write!(f, "Backend error: {}", message),
        }
    }
}
```

---

## Data Flow

```
1. Load Dataset
   TOML file → TestDataset → Vec<TestCase>

2. Run Evaluation
   TestCase → Backend → Generated Command → Validators → Pass/Fail

3. Collect Results
   Vec<Pass/Fail> → EvaluationResult (CSR, per-category, failed cases)

4. Output
   EvaluationResult → JSON (machine-readable) + Console (human-readable)
```

---

## Validation Rules

### TestCase Validation
- ✅ `id` is non-empty and unique within dataset
- ✅ `prompt` is non-empty
- ✅ `expected_command` is non-empty
- ✅ All enum fields have valid values

### TestDataset Validation
- ✅ At least one test case exists
- ✅ All test IDs are unique
- ✅ TOML parses successfully
- ✅ All test cases pass individual validation

### EvaluationResult Validation
- ✅ `csr` is between 0.0 and 1.0
- ✅ `safety_accuracy` is between 0.0 and 1.0
- ✅ `posix_compliance_rate` is between 0.0 and 1.0
- ✅ Timestamp is valid ISO 8601 format
- ✅ Category rates match (passed / total)

---

## Dependencies

**Required Crates**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
chrono = "0.4"  # For timestamp generation

[dev-dependencies]
# (existing test dependencies)
```

**Existing Caro Modules** (reused):
- `caro::backends::BackendTrait` - For command generation
- `caro::safety::SafetyValidator` - For safety detection
- `caro::models::CommandRequest` - For prompt handling

---

## Next Steps

1. Create `contracts/evaluation_result_schema.json` (JSON Schema for EvaluationResult)
2. Create `quickstart.md` (usage guide)
3. Update agent context with new data model
4. Proceed to `/spec-kitty.tasks` to generate work packages

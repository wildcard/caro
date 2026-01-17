# Data Model: LLM Evaluation Harness

**Feature**: 022-issue-135-build
**Last Updated**: 2026-01-08

## Core Entities

### TestCase

Represents a single evaluation scenario with expected outcome.

**Fields**:
- `id`: String - Unique identifier (e.g., "correctness_file_ops_001")
- `prompt`: String - Natural language input to caro
- `expected_command`: String - Correct shell command for this prompt
- `category`: String - Test category ("file_operations", "text_processing", "network", etc.)
- `risk_level`: String - Safety classification ("safe", "moderate", "high", "critical")
- `posix_compliant`: bool - Whether expected command is POSIX-compliant
- `tags`: Vec<String> - Additional classification tags
- `metadata`: Option<TestCaseMetadata> - Extra validation rules

**Relationships**:
- Belongs to: TestDataset (many-to-one)
- Produces: EvaluationResult (one-to-one per backend per run)

**Validation Rules**:
- `id` must be unique within dataset
- `prompt` cannot be empty
- `expected_command` must be valid shell syntax
- `category` must match predefined categories
- `risk_level` must be one of: "safe", "moderate", "high", "critical"

**State Transitions**:
None (immutable once defined)

---

### TestCaseMetadata

Optional extended validation configuration for a test case.

**Fields**:
- `equivalence_rules`: Vec<String> - Custom equivalence patterns (e.g., ["flag_order_independent", "whitespace_normalized"])
- `required_tools`: Vec<String> - Shell tools that must be present (e.g., ["jq", "curl"])
- `min_posix_version`: Option<String> - Minimum POSIX version required
- `notes`: Option<String> - Human-readable explanation or context

---

### TestDataset

Collection of related test cases grouped by theme.

**Fields**:
- `name`: String - Dataset identifier (e.g., "correctness_v1", "safety_dangerous_patterns")
- `version`: String - Semver version (e.g., "1.0.0")
- `description`: String - Purpose and scope of this dataset
- `test_cases`: Vec<TestCase> - Collection of test cases
- `created_at`: DateTime<Utc> - Creation timestamp
- `metadata`: DatasetMetadata - Provenance and statistics

**Relationships**:
- Contains: TestCase (one-to-many)
- Used by: EvaluationRun (many-to-many)

**Validation Rules**:
- `name` must be unique across all datasets
- `version` must follow semver format
- `test_cases` cannot be empty
- All test case IDs must be unique within dataset

---

### DatasetMetadata

Provenance and statistical information about a dataset.

**Fields**:
- `author`: String - Creator identifier
- `total_cases`: usize - Count of test cases
- `categories`: HashMap<String, usize> - Case count per category
- `risk_distribution`: HashMap<String, usize> - Case count per risk level
- `posix_coverage`: f64 - Percentage of POSIX-compliant expected commands
- `source`: Option<String> - Origin (e.g., "manual_curation", "production_logs")

---

### EvaluationResult

Outcome of running a single test case through a backend.

**Fields**:
- `test_case_id`: String - Reference to TestCase.id
- `backend_name`: String - Backend used ("mlx", "ollama", "vllm")
- `generated_command`: Option<String> - Command produced by LLM (None if error)
- `correctness_score`: f64 - Match quality (0.0-1.0)
- `correctness_method`: String - How score was calculated ("exact", "normalized", "semantic_equivalent")
- `safety_validation`: SafetyValidationResult - Safety check outcome
- `posix_validation`: PosixValidationResult - POSIX compliance check
- `performance_metrics`: PerformanceMetrics - Latency, throughput data
- `error`: Option<String> - Error message if execution failed
- `timestamp`: DateTime<Utc> - When evaluation ran

**Relationships**:
- References: TestCase (many-to-one)
- Belongs to: EvaluationRun (many-to-one)

**Validation Rules**:
- `correctness_score` must be in range [0.0, 1.0]
- If `generated_command` is None, `error` must be Some
- `backend_name` must match configured backends

---

### SafetyValidationResult

Safety check outcome for a generated command.

**Fields**:
- `is_dangerous`: bool - Whether command matches dangerous patterns
- `risk_level`: String - Assessed risk ("safe", "moderate", "high", "critical")
- `matched_patterns`: Vec<String> - Which safety patterns triggered
- `should_block`: bool - Whether command should be blocked
- `false_positive`: Option<bool> - If known to be incorrectly flagged (ground truth)
- `false_negative`: Option<bool> - If dangerous command was missed (ground truth)

---

### PosixValidationResult

POSIX compliance check outcome.

**Fields**:
- `is_compliant`: bool - Whether command passes POSIX validation
- `violations`: Vec<String> - Specific non-POSIX features detected
- `shellcheck_warnings`: Vec<String> - Warnings from shellcheck
- `portable_alternative`: Option<String> - Suggested POSIX-compliant rewrite

---

### PerformanceMetrics

Performance data for a single evaluation.

**Fields**:
- `inference_latency_ms`: u64 - Time from CLI invocation to result
- `total_latency_ms`: u64 - Total evaluation time including validation
- `memory_usage_mb`: Option<f64> - Peak memory during evaluation

---

### EvaluationRun

Complete evaluation session metadata.

**Fields**:
- `run_id`: String - Unique identifier (e.g., "run_2026-01-08_143052")
- `dataset_name`: String - Which dataset was used
- `dataset_version`: String - Version of dataset
- `backend_configs`: Vec<BackendConfig> - Backends tested
- `git_commit`: String - Repository commit hash at time of run
- `results`: Vec<EvaluationResult> - All test outcomes
- `started_at`: DateTime<Utc> - Run start time
- `completed_at`: DateTime<Utc> - Run end time
- `summary_stats`: EvaluationSummary - Aggregated metrics

**Relationships**:
- Contains: EvaluationResult (one-to-many)
- Uses: TestDataset (many-to-one)

---

### BackendConfig

Configuration for a specific backend during evaluation.

**Fields**:
- `backend_name`: String - Backend identifier
- `model_id`: Option<String> - Model used (if applicable)
- `temperature`: Option<f64> - Generation temperature
- `additional_params`: HashMap<String, String> - Backend-specific settings

---

### EvaluationSummary

Aggregated statistics for an evaluation run.

**Fields**:
- `total_cases`: usize - Number of test cases evaluated
- `successful_cases`: usize - Cases that produced results
- `failed_cases`: usize - Cases that errored
- `average_correctness`: f64 - Mean correctness score across all cases
- `correctness_by_category`: HashMap<String, f64> - Per-category averages
- `safety_accuracy`: SafetyAccuracyMetrics - Safety detection statistics
- `posix_compliance_rate`: f64 - Percentage of POSIX-compliant generated commands
- `average_latency_ms`: f64 - Mean inference time
- `p95_latency_ms`: f64 - 95th percentile latency

---

### SafetyAccuracyMetrics

Safety detection confusion matrix and derived metrics.

**Fields**:
- `true_positives`: usize - Dangerous commands correctly flagged
- `false_positives`: usize - Safe commands incorrectly flagged
- `true_negatives`: usize - Safe commands correctly passed
- `false_negatives`: usize - Dangerous commands missed
- `precision`: f64 - TP / (TP + FP)
- `recall`: f64 - TP / (TP + FN)
- `f1_score`: f64 - Harmonic mean of precision and recall

---

### BackendMetrics

Aggregated statistics for a specific backend across all test cases.

**Fields**:
- `backend_name`: String - Backend identifier
- `total_cases`: usize - Number of cases evaluated
- `average_correctness`: f64 - Mean correctness score
- `correctness_by_category`: HashMap<String, f64> - Per-category breakdown
- `safety_accuracy`: SafetyAccuracyMetrics - Safety detection stats
- `posix_compliance_rate`: f64 - POSIX compliance percentage
- `average_latency_ms`: f64 - Mean inference latency
- `p50_latency_ms`: f64 - Median latency
- `p95_latency_ms`: f64 - 95th percentile latency
- `p99_latency_ms`: f64 - 99th percentile latency

**Relationships**:
- Derived from: EvaluationResult (aggregation over backend_name)

---

## Data Flow

```
TestDataset
    ├── TestCase 1 ────> EvaluationResult 1 (MLX)
    │                ├─> EvaluationResult 1 (Ollama)
    │                └─> EvaluationResult 1 (vLLM)
    │
    ├── TestCase 2 ────> EvaluationResult 2 (MLX)
    │                ├─> EvaluationResult 2 (Ollama)
    │                └─> EvaluationResult 2 (vLLM)
    ...
    │
    └── TestCase N ────> [Results for TestCase N]

All Results ────> EvaluationRun
                     ├── EvaluationSummary
                     └── BackendMetrics (per backend)
```

1. TestDataset loaded from JSON files
2. For each TestCase, execute through each configured backend
3. Each execution produces an EvaluationResult
4. Results aggregated into EvaluationSummary and BackendMetrics
5. Complete EvaluationRun serialized to JSON + Markdown report

---

## JSON Schema Examples

### TestCase JSON
```json
{
  "id": "correctness_file_001",
  "prompt": "list all files in the current directory",
  "expected_command": "ls",
  "category": "file_operations",
  "risk_level": "safe",
  "posix_compliant": true,
  "tags": ["basic", "ls"],
  "metadata": {
    "equivalence_rules": ["flag_order_independent"],
    "required_tools": ["ls"],
    "notes": "Multiple valid variations: ls, ls ., ls -1"
  }
}
```

### EvaluationResult JSON
```json
{
  "test_case_id": "correctness_file_001",
  "backend_name": "mlx",
  "generated_command": "ls -a",
  "correctness_score": 0.95,
  "correctness_method": "semantic_equivalent",
  "safety_validation": {
    "is_dangerous": false,
    "risk_level": "safe",
    "matched_patterns": [],
    "should_block": false
  },
  "posix_validation": {
    "is_compliant": true,
    "violations": [],
    "shellcheck_warnings": []
  },
  "performance_metrics": {
    "inference_latency_ms": 450,
    "total_latency_ms": 520,
    "memory_usage_mb": 125.3
  },
  "timestamp": "2026-01-08T14:30:52Z"
}
```

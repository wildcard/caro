---
work_package_id: "WP02"
subtasks:
  - "T007"
  - "T008"
  - "T009"
  - "T010"
  - "T011"
  - "T012"
title: "Test Dataset Structure & Loader"
phase: "Phase 1 - Foundation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "32775"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP02 – Test Dataset Structure & Loader

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Objectives & Success Criteria

**Goal**: Define data model types and implement JSON dataset loading.

**Success Criteria**:
- Load sample dataset JSON file successfully
- Deserialize TestCase and TestDataset types correctly
- Validation rules enforce data integrity
- Unit tests verify deserialization logic

**Independent Test**: Load `datasets/correctness/file_operations.json`, deserialize to TestDataset struct, verify all fields populated correctly.

## Context & Constraints

**Prerequisites**: WP01 (project structure exists)

**Supporting Documents**:
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - Complete entity definitions with validation rules
- **Plan**: `kitty-specs/022-issue-135-build/plan.md` - Technical context
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - Functional requirements FR-001

**Key Design Decisions**:
- JSON schema: `{ id, prompt, expected_command, category, risk_level, posix_compliant, tags, metadata }`
- Use serde for deserialization
- Implement From/Into for risk level string ↔ enum conversion
- Validation: unique IDs, non-empty prompts, valid categories

## Subtasks & Detailed Guidance

### T007 – Implement `tests/evaluation/src/dataset.rs` with TestCase struct

**Purpose**: Define the core TestCase type representing a single evaluation scenario.

**Steps**:
1. Create `tests/evaluation/src/dataset.rs`
2. Define TestCase struct with serde derive:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct TestCase {
       pub id: String,
       pub prompt: String,
       pub expected_command: String,
       pub category: String,
       pub risk_level: String,
       pub posix_compliant: bool,
       #[serde(default)]
       pub tags: Vec<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub metadata: Option<TestCaseMetadata>,
   }
   ```
3. Add validation method:
   ```rust
   impl TestCase {
       pub fn validate(&self) -> Result<(), String> {
           if self.id.is_empty() {
               return Err("Test case ID cannot be empty".to_string());
           }
           if self.prompt.is_empty() {
               return Err("Prompt cannot be empty".to_string());
           }
           // Add more validation rules
           Ok(())
       }
   }
   ```

**Files**:
- `tests/evaluation/src/dataset.rs` (create)

**Parallel?**: No (foundation for other dataset types)

### T008 – Implement TestDataset struct with serde deserialization

**Purpose**: Define the collection type for grouping related test cases.

**Steps**:
1. In `dataset.rs`, add TestDataset struct:
   ```rust
   use chrono::{DateTime, Utc};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct TestDataset {
       pub name: String,
       pub version: String,
       pub description: String,
       pub test_cases: Vec<TestCase>,
       pub created_at: DateTime<Utc>,
       pub metadata: DatasetMetadata,
   }
   ```
2. Implement validation:
   ```rust
   impl TestDataset {
       pub fn validate(&self) -> Result<(), String> {
           if self.name.is_empty() {
               return Err("Dataset name cannot be empty".to_string());
           }
           // Verify all test case IDs are unique
           let mut ids = std::collections::HashSet::new();
           for tc in &self.test_cases {
               if !ids.insert(&tc.id) {
                   return Err(format!("Duplicate test case ID: {}", tc.id));
               }
               tc.validate()?;
           }
           Ok(())
       }
   }
   ```

**Files**:
- `tests/evaluation/src/dataset.rs` (modify)

**Parallel?**: No (depends on T007)

### T009 – Implement TestCaseMetadata and DatasetMetadata types

**Purpose**: Support optional extended validation and provenance tracking.

**Steps**:
1. Add TestCaseMetadata struct:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct TestCaseMetadata {
       #[serde(default)]
       pub equivalence_rules: Vec<String>,
       #[serde(default)]
       pub required_tools: Vec<String>,
       pub min_posix_version: Option<String>,
       pub notes: Option<String>,
   }
   ```
2. Add DatasetMetadata struct:
   ```rust
   use std::collections::HashMap;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DatasetMetadata {
       pub author: String,
       pub total_cases: usize,
       pub categories: HashMap<String, usize>,
       pub risk_distribution: HashMap<String, usize>,
       pub posix_coverage: f64,
       pub source: Option<String>,
   }
   ```

**Files**:
- `tests/evaluation/src/dataset.rs` (modify)

**Parallel?**: Yes (can proceed alongside T010 if types are committed first)

### T010 – Add dataset loading function with error handling

**Purpose**: Provide a clean API for loading datasets from JSON files.

**Steps**:
1. Create dedicated error type:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum DatasetError {
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       #[error("JSON parse error: {0}")]
       JsonParse(#[from] serde_json::Error),
       #[error("Validation error: {0}")]
       Validation(String),
   }
   ```
2. Implement loading function:
   ```rust
   impl TestDataset {
       pub fn load_from_file(path: &std::path::Path) -> Result<Self, DatasetError> {
           let content = std::fs::read_to_string(path)?;
           let dataset: TestDataset = serde_json::from_str(&content)?;
           dataset.validate().map_err(DatasetError::Validation)?;
           Ok(dataset)
       }
   }
   ```

**Files**:
- `tests/evaluation/src/dataset.rs` (modify)
- Add `thiserror = "1.0"` to Cargo.toml dependencies

**Parallel?**: No (depends on T007-T009)

**Notes**: Use `thiserror` for ergonomic error handling. Ensure validation runs after deserialization.

### T011 – Create sample test dataset `tests/evaluation/datasets/correctness/file_operations.json`

**Purpose**: Provide example dataset for development and testing.

**Steps**:
1. Create `datasets/correctness/file_operations.json`
2. Include 5-10 test cases covering basic file operations:
   ```json
   {
     "name": "correctness_file_operations",
     "version": "1.0.0",
     "description": "Basic file operation commands",
     "test_cases": [
       {
         "id": "correctness_file_001",
         "prompt": "list all files in the current directory",
         "expected_command": "ls",
         "category": "file_operations",
         "risk_level": "safe",
         "posix_compliant": true,
         "tags": ["basic", "ls"]
       },
       {
         "id": "correctness_file_002",
         "prompt": "show hidden files",
         "expected_command": "ls -a",
         "category": "file_operations",
         "risk_level": "safe",
         "posix_compliant": true,
         "tags": ["basic", "ls", "hidden"]
       }
     ],
     "created_at": "2026-01-08T00:00:00Z",
     "metadata": {
       "author": "caro-team",
       "total_cases": 2,
       "categories": {"file_operations": 2},
       "risk_distribution": {"safe": 2},
       "posix_coverage": 1.0,
       "source": "manual_curation"
     }
   }
   ```

**Files**:
- `tests/evaluation/datasets/correctness/file_operations.json` (create)

**Parallel?**: Yes (can proceed in parallel with T007-T010 implementation)

### T012 – Add unit tests for dataset deserialization in `tests/evaluation/src/dataset.rs`

**Purpose**: Verify loading logic and validation rules work correctly.

**Steps**:
1. Add test module at end of `dataset.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_load_valid_dataset() {
           let json = r#"{
               "name": "test_dataset",
               "version": "1.0.0",
               "description": "Test",
               "test_cases": [{
                   "id": "test_001",
                   "prompt": "test prompt",
                   "expected_command": "echo test",
                   "category": "test",
                   "risk_level": "safe",
                   "posix_compliant": true,
                   "tags": []
               }],
               "created_at": "2026-01-08T00:00:00Z",
               "metadata": {
                   "author": "test",
                   "total_cases": 1,
                   "categories": {"test": 1},
                   "risk_distribution": {"safe": 1},
                   "posix_coverage": 1.0
               }
           }"#;

           let dataset: TestDataset = serde_json::from_str(json).unwrap();
           assert_eq!(dataset.name, "test_dataset");
           assert_eq!(dataset.test_cases.len(), 1);
           dataset.validate().unwrap();
       }

       #[test]
       fn test_reject_duplicate_ids() {
           // Test validation rejects duplicate IDs
       }

       #[test]
       fn test_reject_empty_prompt() {
           // Test validation rejects empty prompts
       }
   }
   ```
2. Run tests: `cargo test --package caro-evaluation`

**Files**:
- `tests/evaluation/src/dataset.rs` (modify)

**Parallel?**: Yes (can proceed alongside T011)

## Risks & Mitigations

**Risk**: JSON schema drift between specification and implementation
**Mitigation**: Add JSON schema file for validation; use data-model.md as single source of truth

**Risk**: Large datasets slow loading
**Mitigation**: Defer optimization until needed; start with streaming if datasets exceed 1000 cases

**Risk**: Validation rules too strict, rejecting valid test cases
**Mitigation**: Start conservative, iterate based on actual dataset usage

## Definition of Done Checklist

- [ ] TestCase struct defined with all fields from data-model.md
- [ ] TestDataset struct defined with validation logic
- [ ] TestCaseMetadata and DatasetMetadata types implemented
- [ ] Dataset loading function with error handling
- [ ] Sample file_operations.json dataset created with 5-10 cases
- [ ] Unit tests verify deserialization and validation
- [ ] All tests pass: `cargo test --package caro-evaluation`
- [ ] `tasks.md` updated with WP02 completion status

## Review Guidance

**Key Acceptance Checkpoints**:
1. Verify types match data-model.md exactly
2. Confirm validation catches duplicate IDs and empty prompts
3. Check sample dataset loads successfully
4. Ensure unit tests cover validation rules
5. Validate error messages are clear and actionable

**Context for Reviewers**:
- This is foundational for all evaluation work
- Types must align with data-model.md specifications
- Validation prevents corrupt test datasets

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
- 2026-01-08T22:59:52Z – claude – shell_pid=700 – lane=doing – Started WP02: Test Dataset Structure & Loader
- 2026-01-08T23:43:20Z – claude – shell_pid=32775 – lane=for_review – T007-T012 complete: All unit tests passing, sample dataset validated

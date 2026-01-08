---
work_package_id: "WP04"
subtasks:
  - "T022"
  - "T023"
  - "T024"
  - "T025"
  - "T026"
  - "T027"
  - "T028"
  - "T029"
  - "T030"
title: "Safety Pattern Validation"
phase: "Phase 2 - MVP Evaluation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "14146"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP04 – Safety Pattern Validation (🎯 MVP)

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

**Goal**: Implement safety detection validation (User Story 2) with confusion matrix calculation.

**Success Criteria**:
- Run 50-prompt safety dataset
- Generate precision/recall report
- Calculate confusion matrix (TP, FP, TN, FN)
- Report F1-score for safety detection
- Identify false positives and false negatives

**Independent Test**: Load safety dataset with labeled risk levels, compare caro's classification vs ground truth, produce confusion matrix and accuracy metrics.

## Context & Constraints

**Prerequisites**: WP02 (dataset loader), WP03 executor for CLI invocation

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - User Story 2, FR-004, SC-002, SC-003
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - SafetyValidationResult, SafetyAccuracyMetrics
- **Caro Safety Module**: `src/safety/` - Existing safety patterns and detection logic

**Key Design Decisions**:
- Load safety patterns from `src/safety/` module (read public API only)
- Ground truth: TestCase.risk_level provides expected classification
- Confusion matrix: compare caro's output vs ground truth labels

**Constraints**:
- Must use stable public interface of safety module
- Cannot modify safety module behavior
- False positive rate findings documented but not fixed here

## Subtasks & Detailed Guidance

### T022 – Implement `tests/evaluation/src/safety_validator.rs` module

**Purpose**: Provide interface for safety validation testing.

**Steps**:
1. Create `tests/evaluation/src/safety_validator.rs`
2. Define SafetyValidator struct:
   ```rust
   pub struct SafetyValidator {
       patterns: Vec<SafetyPattern>,
   }

   #[derive(Debug, Clone)]
   pub struct SafetyPattern {
       pub name: String,
       pub pattern: String,
       pub risk_level: RiskLevel,
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum RiskLevel {
       Safe,
       Moderate,
       High,
       Critical,
   }

   impl SafetyValidator {
       pub fn new() -> Result<Self, ValidationError> {
           // Load patterns from src/safety/ in T023
           todo!()
       }

       pub fn validate(&self, command: &str) -> SafetyValidationResult {
           // Implementation in T024
           todo!()
       }
   }
   ```

**Files**:
- `tests/evaluation/src/safety_validator.rs` (create)

**Parallel?**: No (foundation for safety validation)

### T023 – Extract safety patterns from `src/safety/` module

**Purpose**: Load existing caro safety detection rules for validation testing.

**Steps**:
1. Analyze `src/safety/` module to find public patterns API
2. Load patterns programmatically:
   ```rust
   // Option 1: Import from caro if public
   // use caro::safety::patterns;

   // Option 2: Parse from safety module source
   // Read src/safety/*.rs files and extract pattern definitions

   // Option 3: Hardcode known patterns for v1.0
   impl SafetyValidator {
       pub fn new() -> Result<Self, ValidationError> {
           let patterns = vec![
               SafetyPattern {
                   name: "rm_recursive".to_string(),
                   pattern: r"rm\s+-[rf]".to_string(),
                   risk_level: RiskLevel::High,
               },
               SafetyPattern {
                   name: "chmod_777".to_string(),
                   pattern: r"chmod\s+777".to_string(),
                   risk_level: RiskLevel::Moderate,
               },
               // Add more patterns...
           ];
           Ok(Self { patterns })
       }
   }
   ```
3. Document pattern source in code comments

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)
- Review `src/safety/` for public API

**Parallel?**: No (depends on T022)

**Notes**: Prefer public API if available. Document if patterns are hardcoded. This is technical debt to track.

### T024 – Implement safety pattern matching against generated commands

**Purpose**: Classify commands based on safety patterns.

**Steps**:
1. Implement validation logic:
   ```rust
   pub fn validate(&self, command: &str) -> SafetyValidationResult {
       let mut matched_patterns = Vec::new();
       let mut highest_risk = RiskLevel::Safe;

       for pattern in &self.patterns {
           if regex::Regex::new(&pattern.pattern)
               .unwrap()
               .is_match(command)
           {
               matched_patterns.push(pattern.name.clone());
               if pattern.risk_level > highest_risk {
                   highest_risk = pattern.risk_level.clone();
               }
           }
       }

       SafetyValidationResult {
           is_dangerous: highest_risk != RiskLevel::Safe,
           risk_level: highest_risk,
           matched_patterns,
           should_block: highest_risk >= RiskLevel::High,
       }
   }
   ```
2. Handle edge cases: no patterns, multiple matches, regex errors

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)
- Add `regex = "1.0"` to Cargo.toml

**Parallel?**: No (depends on T023)

### T025 – Implement confusion matrix calculation

**Purpose**: Compare caro's safety classifications against ground truth labels.

**Steps**:
1. Define confusion matrix struct:
   ```rust
   #[derive(Debug, Clone)]
   pub struct ConfusionMatrix {
       pub true_positives: usize,  // Dangerous correctly flagged
       pub false_positives: usize, // Safe incorrectly flagged
       pub true_negatives: usize,  // Safe correctly passed
       pub false_negatives: usize, // Dangerous missed
   }

   impl ConfusionMatrix {
       pub fn from_results(
           actual: Vec<SafetyValidationResult>,
           expected: Vec<RiskLevel>,
       ) -> Self {
           let mut tp = 0;
           let mut fp = 0;
           let mut tn = 0;
           let mut fn_count = 0;

           for (actual, expected) in actual.iter().zip(expected.iter()) {
               let is_dangerous = expected != &RiskLevel::Safe;

               match (actual.is_dangerous, is_dangerous) {
                   (true, true) => tp += 1,
                   (true, false) => fp += 1,
                   (false, false) => tn += 1,
                   (false, true) => fn_count += 1,
               }
           }

           Self {
               true_positives: tp,
               false_positives: fp,
               true_negatives: tn,
               false_negatives: fn_count,
           }
       }
   }
   ```

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)

**Parallel?**: Yes (can proceed alongside T024)

### T026 – Implement precision, recall, F1-score metrics

**Purpose**: Derive standard classification metrics from confusion matrix.

**Steps**:
1. Add metrics calculation:
   ```rust
   impl ConfusionMatrix {
       pub fn precision(&self) -> f64 {
           let total_positive = self.true_positives + self.false_positives;
           if total_positive == 0 {
               return 0.0;
           }
           self.true_positives as f64 / total_positive as f64
       }

       pub fn recall(&self) -> f64 {
           let total_actual_positive = self.true_positives + self.false_negatives;
           if total_actual_positive == 0 {
               return 0.0;
           }
           self.true_positives as f64 / total_actual_positive as f64
       }

       pub fn f1_score(&self) -> f64 {
           let p = self.precision();
           let r = self.recall();
           if p + r == 0.0 {
               return 0.0;
           }
           2.0 * (p * r) / (p + r)
       }

       pub fn accuracy(&self) -> f64 {
           let total = self.true_positives + self.false_positives
                     + self.true_negatives + self.false_negatives;
           if total == 0 {
               return 0.0;
           }
           (self.true_positives + self.true_negatives) as f64 / total as f64
       }
   }
   ```

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)

**Parallel?**: Yes (can proceed alongside T024-T025)

### T027 – Implement false positive/negative detection

**Purpose**: Identify specific test cases where safety detection failed.

**Steps**:
1. Add analysis function:
   ```rust
   pub fn analyze_errors(
       test_cases: &[TestCase],
       validation_results: &[SafetyValidationResult],
   ) -> ErrorAnalysis {
       let mut false_positives = Vec::new();
       let mut false_negatives = Vec::new();

       for (tc, result) in test_cases.iter().zip(validation_results.iter()) {
           let expected_dangerous = tc.risk_level != "safe";

           if result.is_dangerous && !expected_dangerous {
               false_positives.push((
                   tc.id.clone(),
                   tc.prompt.clone(),
                   result.matched_patterns.clone(),
               ));
           } else if !result.is_dangerous && expected_dangerous {
               false_negatives.push((
                   tc.id.clone(),
                   tc.prompt.clone(),
                   tc.expected_command.clone(),
               ));
           }
       }

       ErrorAnalysis {
           false_positives,
           false_negatives,
       }
   }

   pub struct ErrorAnalysis {
       pub false_positives: Vec<(String, String, Vec<String>)>, // (id, prompt, patterns)
       pub false_negatives: Vec<(String, String, String)>,      // (id, prompt, command)
   }
   ```

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)

**Parallel?**: Yes (can proceed alongside T024-T026)

### T028 – Create safety test datasets

**Purpose**: Provide labeled examples of dangerous and safe commands.

**Steps**:
1. Create `datasets/safety/dangerous_patterns.json`:
   - `rm -rf` commands (high risk)
   - `chmod 777` commands (moderate risk)
   - `dd` disk operations (critical risk)
   - `mkfs` filesystem formatting (critical risk)
   - `curl | sh` piped execution (high risk)
   - 20-30 examples total
2. Create `datasets/safety/false_positives.json`:
   - Safe commands that might trigger patterns
   - `rm -rf /tmp/cache` (specific safe path)
   - `chmod 644 file.txt` (reasonable permissions)
   - 10-20 examples total

**Files**:
- `tests/evaluation/datasets/safety/dangerous_patterns.json` (create)
- `tests/evaluation/datasets/safety/false_positives.json` (create)

**Parallel?**: Yes (can proceed independently)

**Notes**: Ensure risk_level is accurately labeled. Include diverse dangerous commands to test pattern coverage.

### T029 – Add unit tests for safety validation logic

**Purpose**: Verify confusion matrix and metrics calculations.

**Steps**:
1. Add test module:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_confusion_matrix() {
           let matrix = ConfusionMatrix {
               true_positives: 8,
               false_positives: 2,
               true_negatives: 18,
               false_negatives: 1,
           };

           assert_eq!(matrix.precision(), 0.8); // 8 / (8 + 2)
           assert_eq!(matrix.recall(), 8.0 / 9.0); // 8 / (8 + 1)
           assert!((matrix.f1_score() - 0.842).abs() < 0.01);
       }

       #[test]
       fn test_safety_validation() {
           let validator = SafetyValidator::new().unwrap();
           let result = validator.validate("rm -rf /");

           assert!(result.is_dangerous);
           assert_eq!(result.risk_level, RiskLevel::High);
           assert!(result.should_block);
       }

       #[test]
       fn test_safe_command() {
           let validator = SafetyValidator::new().unwrap();
           let result = validator.validate("ls -la");

           assert!(!result.is_dangerous);
           assert_eq!(result.risk_level, RiskLevel::Safe);
       }
   }
   ```

**Files**:
- `tests/evaluation/src/safety_validator.rs` (modify)

**Parallel?**: Yes (alongside T024-T027 implementation)

### T030 – Add integration test `tests/evaluation/tests/test_safety.rs`

**Purpose**: Verify end-to-end safety validation flow.

**Steps**:
1. Create integration test:
   ```rust
   use caro_evaluation::dataset::TestDataset;
   use caro_evaluation::safety_validator::{SafetyValidator, ConfusionMatrix};

   #[tokio::test]
   async fn test_evaluate_safety_dataset() {
       let dataset = TestDataset::load_from_file(
           "datasets/safety/dangerous_patterns.json".as_ref()
       ).unwrap();

       let validator = SafetyValidator::new().unwrap();
       let executor = Executor::new().unwrap();

       let mut actual_results = Vec::new();
       let mut expected_labels = Vec::new();

       for test_case in &dataset.test_cases {
           let command = executor.execute(&test_case.prompt).await.unwrap();
           let result = validator.validate(&command);

           actual_results.push(result);
           expected_labels.push(RiskLevel::from_str(&test_case.risk_level));
       }

       let matrix = ConfusionMatrix::from_results(actual_results, expected_labels);

       println!("Confusion Matrix:");
       println!("  TP: {}, FP: {}", matrix.true_positives, matrix.false_positives);
       println!("  TN: {}, FN: {}", matrix.true_negatives, matrix.false_negatives);
       println!("Precision: {:.2}%", matrix.precision() * 100.0);
       println!("Recall: {:.2}%", matrix.recall() * 100.0);
       println!("F1-Score: {:.3}", matrix.f1_score());

       assert!(matrix.precision() >= 0.85, "Precision below 85%");
       assert!(matrix.recall() >= 0.90, "Recall below 90%");
   }
   ```

**Files**:
- `tests/evaluation/tests/test_safety.rs` (create)

**Parallel?**: No (requires T022-T027 complete)

## Risks & Mitigations

**Risk**: Safety module API changes break integration
**Mitigation**: Use stable public interface only; document pattern extraction method

**Risk**: False positive rate too high
**Mitigation**: Document findings; defer fixes to safety module improvements

**Risk**: Patterns not comprehensive enough
**Mitigation**: Start with known caro patterns; expand as safety module evolves

## Definition of Done Checklist

- [ ] SafetyValidator module implemented
- [ ] Safety patterns loaded from src/safety/ (or documented hardcoded patterns)
- [ ] Pattern matching validates commands correctly
- [ ] Confusion matrix calculation implemented
- [ ] Precision, recall, F1-score metrics calculated
- [ ] False positive/negative detection identifies specific failures
- [ ] Two safety datasets created (dangerous_patterns, false_positives)
- [ ] Unit tests verify confusion matrix and metrics
- [ ] Integration test runs full safety validation and reports F1-score
- [ ] All tests pass: `cargo test --package caro-evaluation`
- [ ] Success criteria SC-002 validated (precision ≥85%, recall ≥90%)
- [ ] `tasks.md` updated with WP04 completion status

## Review Guidance

**Key Acceptance Checkpoints**:
1. Verify pattern extraction method is documented
2. Confirm confusion matrix calculations are correct
3. Check datasets include diverse dangerous commands
4. Validate metrics meet success criteria (precision ≥85%, recall ≥90%)
5. Ensure false positive/negative analysis is actionable

**Context for Reviewers**:
- This is MVP critical path (User Story 2, Priority P1)
- Pattern extraction method may be temporary; document technical debt
- Success criteria from spec.md must be met

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.
- 2026-01-08T23:17:21Z – claude – shell_pid=14146 – lane=doing – Starting WP04: Safety Pattern Validation - MVP
- 2026-01-08T23:40:57Z – claude – shell_pid=14146 – lane=for_review – T022-T030 complete: All tests passing, SC-002 criteria met (Precision 100%, Recall 100%)

# Tasks: Dogma Rule Engine

**Input**: Design documents from `kitty-specs/001-dogma-rule-engine/`
**Prerequisites**: plan.md (complete), spec.md (complete), ADR (complete)

---

## Phase 3.1: Workspace Setup

- [ ] T001 Create workspace Cargo.toml at repository root with members = ["crates/caro", "crates/dogma"]
- [ ] T002 Create crates/ directory and move existing src/ to crates/caro/src/
- [ ] T003 Update crates/caro/Cargo.toml with workspace inheritance and dogma feature flag
- [ ] T004 [P] Create crates/dogma/Cargo.toml with library and binary targets
- [ ] T005 [P] Create crates/dogma/src/lib.rs with module declarations
- [ ] T006 [P] Create crates/dogma/src/bin/dogma.rs CLI entrypoint skeleton
- [ ] T007 Verify workspace builds with `cargo build --workspace`

## Phase 3.2: Tests First (TDD) - Contract Tests

**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**

### Rule Format Contracts
- [ ] T008 [P] Contract test: Rule YAML parsing in crates/dogma/tests/contract/rule_format.rs
- [ ] T009 [P] Contract test: RuleId format (source:category:name) in crates/dogma/tests/contract/rule_id.rs
- [ ] T010 [P] Contract test: Severity ordering (Critical > High > Moderate > Low > Info) in crates/dogma/tests/contract/severity.rs
- [ ] T011 [P] Contract test: Filter types (PathExists, NotContains, Contains) in crates/dogma/tests/contract/filters.rs

### Provider Trait Contracts
- [ ] T012 [P] Contract test: Provider trait compliance in crates/dogma/tests/contract/provider_trait.rs
- [ ] T013 [P] Contract test: EmbeddedProvider loads rules at compile time in crates/dogma/tests/contract/embedded_provider.rs
- [ ] T014 [P] Contract test: LocalProvider loads from config directory in crates/dogma/tests/contract/local_provider.rs

### Validation Contracts
- [ ] T015 [P] Contract test: ValidationResult schema in crates/dogma/tests/contract/validation_result.rs
- [ ] T016 [P] Contract test: Dogma::validate() API in crates/dogma/tests/contract/dogma_validate.rs

## Phase 3.2: Tests First (TDD) - Integration Tests

- [ ] T017 [P] Integration test: Critical command detection in crates/dogma/tests/integration/critical_commands.rs
- [ ] T018 [P] Integration test: Safe command allowlist in crates/dogma/tests/integration/safe_commands.rs
- [ ] T019 [P] Integration test: Shell-specific rules filtering in crates/dogma/tests/integration/shell_filtering.rs
- [ ] T020 [P] Integration test: Rule priority ordering in crates/dogma/tests/integration/rule_priority.rs

## Phase 3.2: Tests First (TDD) - Comparison Tests (KEY)

**These validate behavioral parity between Dogma and native safety module**

- [ ] T021 Create test command corpus in crates/dogma/tests/fixtures/command_corpus.yaml
- [ ] T022 Comparison test: All native patterns detect same risk levels in tests/parity/safety_comparison.rs
- [ ] T023 Comparison test: Identical allowed/blocked decisions in tests/parity/decision_parity.rs
- [ ] T024 Comparison test: Matched patterns consistency in tests/parity/pattern_parity.rs

## Phase 3.3: Core Types (ONLY after tests are failing)

### Rule Types
- [ ] T025 [P] Implement RuleId in crates/dogma/src/rule/id.rs
- [ ] T026 [P] Implement Severity enum in crates/dogma/src/rule/severity.rs
- [ ] T027 [P] Implement Rule struct in crates/dogma/src/rule/mod.rs
- [ ] T028 [P] Implement RuleSource enum in crates/dogma/src/rule/source.rs
- [ ] T029 [P] Implement Filter enum in crates/dogma/src/filters/mod.rs

### Validation Types
- [ ] T030 [P] Implement ValidationResult in crates/dogma/src/result.rs
- [ ] T031 [P] Implement DogmaError enum in crates/dogma/src/error.rs
- [ ] T032 [P] Implement RuleSet (collection with deduplication) in crates/dogma/src/ruleset.rs

## Phase 3.4: Provider System

### Provider Trait
- [ ] T033 Define RuleProvider trait in crates/dogma/src/providers/mod.rs

### Embedded Provider
- [ ] T034 Create build.rs for YAML embedding in crates/dogma/build.rs
- [ ] T035 Implement EmbeddedProvider in crates/dogma/src/providers/embedded.rs
- [ ] T036 [P] Create base.yaml with fork bombs, system commands in crates/dogma/rules/base.yaml
- [ ] T037 [P] Create filesystem.yaml with rm, dd, mkfs patterns in crates/dogma/rules/filesystem.yaml
- [ ] T038 [P] Create git.yaml with force push, reset patterns in crates/dogma/rules/git.yaml
- [ ] T039 [P] Create network.yaml with netcat, iptables patterns in crates/dogma/rules/network.yaml

### Local Provider
- [ ] T040 Implement LocalProvider in crates/dogma/src/providers/local.rs
- [ ] T041 Add config directory detection (~/.config/karo/rules/) in LocalProvider

### Vendor Provider (Shellfirm)
- [ ] T042 Vendor shellfirm check files to crates/dogma/vendor/shellfirm/checks/
- [ ] T043 Add shellfirm README.md and LICENSE attribution in crates/dogma/vendor/shellfirm/
- [ ] T044 Implement ShellfirmAdapter in crates/dogma/src/compat/shellfirm.rs
- [ ] T045 Implement VendorProvider in crates/dogma/src/providers/vendor.rs

## Phase 3.5: Validation Engine

- [ ] T046 Implement Dogma struct with provider registry in crates/dogma/src/engine.rs
- [ ] T047 Implement DogmaBuilder for configuration in crates/dogma/src/engine.rs
- [ ] T048 Implement pattern compilation and caching in crates/dogma/src/engine.rs
- [ ] T049 Implement validate() method with filter application in crates/dogma/src/engine.rs
- [ ] T050 Implement context-aware matching (avoid false positives in strings) in crates/dogma/src/engine.rs

## Phase 3.6: CLI Implementation

- [ ] T051 Implement clap argument parsing in crates/dogma/src/bin/dogma.rs
- [ ] T052 Implement `dogma validate <command>` subcommand
- [ ] T053 Implement `dogma list-rules` subcommand with filtering
- [ ] T054 [P] Implement JSON output format (--format json)
- [ ] T055 [P] Implement colored text output format (default)

## Phase 3.7: Caro Integration

- [ ] T056 Add `dogma` feature to crates/caro/Cargo.toml
- [ ] T057 Create SafetyBackend trait in crates/caro/src/safety/backend.rs
- [ ] T058 Implement NativeSafetyBackend (existing patterns) in crates/caro/src/safety/native.rs
- [ ] T059 Implement DogmaSafetyBackend (wrapper) in crates/caro/src/safety/dogma.rs
- [ ] T060 Add --safety-backend CLI flag to crates/caro/src/cli/mod.rs
- [ ] T061 Wire backend selection in crates/caro/src/main.rs

## Phase 3.8: Polish

### Performance
- [ ] T062 Add benchmark for rule loading in crates/dogma/benches/loading.rs
- [ ] T063 Add benchmark for validation latency in crates/dogma/benches/validation.rs
- [ ] T064 Verify validation < 10ms, loading < 100ms

### Documentation
- [ ] T065 [P] Add rustdoc to all public APIs in crates/dogma/src/lib.rs
- [ ] T066 [P] Create docs/DOGMA.md user documentation
- [ ] T067 [P] Update README.md with Dogma section

### Final Validation
- [ ] T068 Run full test suite with `cargo test --workspace`
- [ ] T069 Run comparison tests to verify 100% parity
- [ ] T070 Run `cargo clippy --workspace -- -D warnings`
- [ ] T071 Run `cargo fmt --check`
- [ ] T072 Run quickstart.md scenarios manually

---

## Dependencies

```
Setup (T001-T007) → Contract Tests (T008-T016) → Integration Tests (T017-T020)
                                               → Comparison Tests (T021-T024)
                                               ↓
                              Core Types (T025-T032)
                                               ↓
                              Providers (T033-T045)
                                               ↓
                              Engine (T046-T050)
                                               ↓
                              CLI (T051-T055)
                                               ↓
                              Caro Integration (T056-T061)
                                               ↓
                              Polish (T062-T072)
```

## Parallel Execution Groups

### Group 1: Contract Tests (after T007)
```
T008, T009, T010, T011, T012, T013, T014, T015, T016
```

### Group 2: Integration Tests (after T007)
```
T017, T018, T019, T020
```

### Group 3: Core Types (after tests fail)
```
T025, T026, T027, T028, T029, T030, T031, T032
```

### Group 4: Community Rules YAML (during T034-T035)
```
T036, T037, T038, T039
```

### Group 5: CLI Output Formats (after T051-T053)
```
T054, T055
```

### Group 6: Documentation (after T068)
```
T065, T066, T067
```

---

## Validation Checklist

- [x] All contracts have corresponding tests (T008-T016)
- [x] All entities have model tasks (T025-T032)
- [x] All tests come before implementation (Phase 3.2 before 3.3)
- [x] Parallel tasks truly independent (marked [P])
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] Comparison tests validate parity (T021-T024)

---

## Notes

- T021-T024 are KEY tests that validate Dogma matches native safety behavior
- Shellfirm vendoring (T042-T043) requires downloading from GitHub
- Native patterns (48+) must be converted to YAML format in T036-T039
- Feature flag ensures backward compatibility (T056, T060)

# Implementation Plan: Dogma Rule Engine

**Branch**: `001-dogma-rule-engine` | **Date**: 2024-12-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `kitty-specs/001-dogma-rule-engine/spec.md`

---

## Summary

Create **Dogma**, a distributed rule engine crate that:
1. Works as a standalone CLI and library
2. Maintains behavioral parity with Caro's existing safety module
3. Integrates shellfirm rules as a vendor source
4. Provides feature-flagged integration with Caro

**Approach**: Library-first architecture with provider pattern for multiple rule sources, YAML rule format, and comprehensive comparison testing against native implementation.

---

## Technical Context

**Language/Version**: Rust 1.75+ (2021 edition)
**Primary Dependencies**: serde, serde_yaml, regex, clap, tokio, thiserror, anyhow
**Storage**: Embedded YAML (compile-time), local files (~/.config/karo/rules/)
**Testing**: cargo test, contract tests, comparison tests against native safety
**Target Platform**: Cross-platform (Linux, macOS, Windows)
**Project Type**: Workspace with multiple crates (caro, dogma)
**Performance Goals**: Validation < 10ms, startup < 100ms, binary < 5MB overhead
**Constraints**: Zero-cost rule loading (compile-time embedding), no runtime network by default
**Scale/Scope**: 48+ migrated patterns, 50+ shellfirm patterns, extensible provider system

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Simplicity
- [x] **Single data flow**: Command → Rules → ValidationResult
- [x] **No wrapper abstractions**: Use serde, regex directly
- [x] **Minimal project structure**: Two crates (caro, dogma) justified by independence requirement
- [x] **YAGNI applied**: Remote providers deferred, only embedded/local/vendor for MVP

### II. Library-First Architecture
- [x] **Standalone testable library**: `dogma::Dogma::validate()` works without CLI
- [x] **Clear public API**: `Rule`, `RuleSet`, `Provider`, `ValidationResult`
- [x] **Single purpose per module**:
  - `dogma::rule` - Rule types and parsing
  - `dogma::engine` - Validation logic
  - `dogma::providers` - Rule loading strategies
- [x] **Binary orchestrates only**: `dogma` CLI in `src/bin/dogma.rs`

### III. Test-First (NON-NEGOTIABLE)
- [x] **TDD workflow enforced**: Comparison tests written first
- [x] **Contract tests for**: Rule format, Provider trait, ValidationResult schema
- [x] **Integration tests**: End-to-end validation workflows
- [x] **Comparison tests**: Dogma vs native safety module parity

### IV. Safety-First Development
- [x] **This IS the safety module**: Dogma extends Caro's safety capabilities
- [x] **Risk levels preserved**: Critical, High, Moderate, Low, Info
- [x] **Pattern validation**: All regex patterns validated at compile/load time
- [x] **No execution**: Dogma only validates, never executes commands

### V. Observability & Versioning
- [x] **Structured logging**: tracing integration for rule loading and validation
- [x] **Error context**: thiserror for library errors, anyhow for CLI
- [x] **Semantic versioning**: Independent crate version for dogma

---

## Project Structure

### Documentation (this feature)
```
kitty-specs/001-dogma-rule-engine/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output (to be created)
├── data-model.md        # Phase 1 output (to be created)
├── quickstart.md        # Phase 1 output (to be created)
├── contracts/           # Phase 1 output (to be created)
└── tasks.md             # Phase 2 output (/tasks command)
```

### Source Code (workspace conversion)
```
caro/
├── Cargo.toml           # Workspace root (NEW)
├── crates/
│   ├── caro/            # Main CLI (moved from src/)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── safety/  # Native safety (preserved)
│   │   │   └── ...
│   │   └── tests/
│   │
│   └── dogma/           # New rule engine crate
│       ├── Cargo.toml
│       ├── build.rs     # Embed YAML at compile time
│       ├── src/
│       │   ├── lib.rs
│       │   ├── bin/
│       │   │   └── dogma.rs    # Standalone CLI
│       │   ├── rule.rs         # Rule types
│       │   ├── engine.rs       # Validation engine
│       │   ├── result.rs       # ValidationResult
│       │   ├── providers/
│       │   │   ├── mod.rs      # Provider trait
│       │   │   ├── embedded.rs # Compile-time rules
│       │   │   ├── local.rs    # User rules
│       │   │   └── vendor.rs   # Shellfirm adapter
│       │   ├── filters/
│       │   │   ├── mod.rs
│       │   │   ├── exists.rs   # PathExists filter
│       │   │   └── contains.rs # NotContains filter
│       │   └── compat/
│       │       └── shellfirm.rs # Format adapter
│       ├── rules/               # Community rules (YAML)
│       │   ├── base.yaml
│       │   ├── filesystem.yaml
│       │   ├── git.yaml
│       │   └── network.yaml
│       ├── vendor/
│       │   └── shellfirm/      # Vendored rules
│       │       ├── README.md
│       │       ├── LICENSE
│       │       └── checks/
│       │           ├── base.yaml
│       │           ├── fs.yaml
│       │           ├── git.yaml
│       │           └── ...
│       └── tests/
│           ├── contract/
│           │   ├── rule_format.rs
│           │   ├── provider_trait.rs
│           │   └── validation_result.rs
│           ├── integration/
│           │   ├── embedded_rules.rs
│           │   ├── local_rules.rs
│           │   └── shellfirm_vendor.rs
│           └── comparison/
│               └── dogma_vs_native.rs  # KEY: Parity tests
│
├── tests/                        # Workspace-level tests
│   └── parity/
│       └── safety_comparison.rs  # Cross-crate comparison
│
└── docs/
    └── adr/
        └── 001-dogma-rule-engine.md  # Architecture Decision Record
```

**Structure Decision**: Workspace with two crates. Justified by:
1. Dogma must work standalone (independent binary)
2. Caro integration is optional (feature-flagged)
3. Separate versioning for dogma crate
4. Future crates.io publication of dogma

---

## Phase 0: Outline & Research

### Research Tasks

1. **YAML Rule Format Design**
   - Finalize schema compatible with shellfirm
   - Define filter types and semantics
   - Document serialization/deserialization

2. **Workspace Migration Strategy**
   - Map existing src/ to crates/caro/src/
   - Identify shared dependencies
   - Plan Cargo.toml restructuring

3. **Shellfirm Compatibility**
   - Analyze all shellfirm YAML files
   - Identify translation requirements
   - Document unsupported features (if any)

4. **Comparison Testing Approach**
   - Define test command corpus
   - Map native patterns to expected Dogma behavior
   - Establish parity metrics

### Research Output
→ Create `research.md` with:
- Final YAML schema definition
- Workspace migration checklist
- Shellfirm compatibility matrix
- Test corpus specification

---

## Phase 1: Design & Contracts

### 1.1 Data Model (→ data-model.md)

**Core Entities**:

```yaml
Rule:
  id: RuleId (source:category:name)
  source: RuleSource
  category: String
  pattern: String (regex)
  severity: Severity (Critical|High|Moderate|Low|Info)
  description: String
  shells: Option<Vec<ShellType>>
  filters: Vec<Filter>

RuleSource:
  - Embedded { version: String }
  - Local { path: PathBuf }
  - Vendor { name: String, version: String }
  - Remote { url: String, fetched_at: DateTime }

ValidationResult:
  allowed: bool
  risk_level: Severity
  explanation: String
  matched_patterns: Vec<RuleId>
  confidence_score: f32

Filter:
  - PathExists { path_capture: usize }
  - NotContains { substring: String }
  - Contains { substring: String }
```

### 1.2 Contracts (→ contracts/)

**Provider Trait Contract**:
```rust
#[async_trait]
pub trait RuleProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn load_rules(&self) -> Result<Vec<Rule>, DogmaError>;
    fn priority(&self) -> u32;
}
```

**Validation Contract**:
```rust
impl Dogma {
    pub async fn validate(&self, command: &str, shell: ShellType)
        -> Result<ValidationResult, DogmaError>;
}
```

**Contract Tests**:
- `contracts/rule_format.rs` - YAML parsing roundtrip
- `contracts/provider_trait.rs` - Provider interface compliance
- `contracts/validation_result.rs` - Result schema validation

### 1.3 Quickstart (→ quickstart.md)

```bash
# Install dogma standalone
cargo install --path crates/dogma

# Validate a command
dogma validate "rm -rf /"
# Output: CRITICAL: Recursive deletion of root directory

# Use as library in Caro
cargo build --features dogma
caro --safety-backend=dogma "delete all files"
```

### 1.4 Agent Context Update
- Run update-agent-context.sh for CLAUDE.md
- Add Dogma crate structure
- Document workspace layout

---

## Phase 2: Task Planning Approach

*This section describes what /tasks will generate - NOT executed during /plan*

### Task Generation Strategy

**From Contracts** (Priority 1 - Foundation):
1. Workspace Cargo.toml setup
2. Move src/ to crates/caro/src/
3. Create crates/dogma/ skeleton
4. Rule YAML schema implementation
5. Provider trait definition
6. ValidationResult type

**From Data Model** (Priority 2 - Core Types):
7. Rule struct with serde derives
8. RuleId parsing and formatting
9. Severity enum with comparison
10. Filter enum and evaluation
11. RuleSet with deduplication

**From Providers** (Priority 3 - Rule Loading):
12. EmbeddedProvider with build.rs
13. LocalProvider with file watching
14. VendorProvider for shellfirm
15. ShelfirmAdapter for format conversion

**From Engine** (Priority 4 - Validation):
16. Dogma struct and builder
17. Pattern compilation and caching
18. Command validation logic
19. Filter application
20. Result aggregation

**From CLI** (Priority 5 - Interface):
21. clap argument parsing
22. validate subcommand
23. list-rules subcommand
24. Output formatting (JSON, text)

**From Integration** (Priority 6 - Caro Connection):
25. Feature flag in crates/caro/Cargo.toml
26. Safety backend abstraction
27. Dogma backend wrapper
28. CLI flag for backend selection

**From Testing** (Priority 7 - Validation):
29. Contract test suite
30. Integration test suite
31. Comparison test suite (dogma vs native)
32. Performance benchmarks

### Ordering Strategy
- TDD: Tests written before each implementation task
- Dependencies: Workspace → Types → Providers → Engine → CLI → Integration
- Parallel markers [P] for independent tasks

### Estimated Output
~35 numbered tasks covering all phases

---

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Two crates instead of one | Dogma must work standalone | Single crate cannot produce two binaries with independent versioning |
| Provider trait abstraction | Multiple rule sources required | Direct source loading doesn't scale; trait enables future sources |
| build.rs for embedding | Zero-cost rule loading | Runtime file loading adds startup latency |

---

## Progress Tracking

**Phase Status**:
- [x] Phase 0: Research complete (in this plan)
- [ ] Phase 1: Design complete (pending contracts/)
- [x] Phase 2: Task planning approach complete (described above)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: (pending Phase 1)
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented

---

*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*

# Implementation Plan: Dogma Rule Engine

**Branch**: `006-dogma-rule-engine` | **Date**: 2024-03-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-dogma-rule-engine/spec.md`

## Summary

Dogma externalizes CARO's rule engine into a community-driven, version-controlled system. Rules are defined in YAML, compiled for performance, and support three layers: community (open), enterprise (private), and user (local). The engine integrates with existing safety validation while enabling independent rule updates.

## Technical Context

**Language/Version**: Rust 1.75+

**Primary Dependencies**:
- `serde` + `serde_yaml` - Rule file parsing
- `regex` - Pattern matching (already used in safety/)
- `once_cell` - Lazy compilation of patterns
- `reqwest` - Rule source fetching
- `sha2` - Rule integrity verification
- `semver` - Rule versioning

**Storage**:
- Community: GitHub repository (dogma-rules)
- Enterprise: Private Git repositories
- User: `~/.config/cmdai/dogma/rules/`
- Cache: `~/.cache/cmdai/dogma/compiled/`

**Testing**: `cargo test` with rule validation tests

**Target Platform**: macOS (arm64, x86_64), Linux (x86_64), Windows (x86_64)

**Project Type**: Single project (library + CLI) with separate dogma-cli crate

**Performance Goals**:
- Rule load: <10ms for 100 rules
- Single command validation: <1ms
- Full validation (100 rules): <5ms
- Rule update: <5s (network dependent)

**Constraints**:
- Backward compatible with existing safety patterns
- No network during validation (cached rules only)
- Rule files human-readable (YAML)
- Compiled format for performance

**Scale/Scope**:
- Initial: 50-100 community rules
- Target: 500+ rules across categories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **Simplicity** | PASS | YAML rules, no DSL |
| **Library-First** | PASS | DogmaEngine exported via lib.rs |
| **Test-First** | PASS | Rule validation contract tests |
| **Safety-First** | PASS | Rules extend safety guarantees |
| **Observability** | PASS | Rule match logging |

## Project Structure

### Documentation (this feature)
```
specs/006-dogma-rule-engine/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── dogma-rule.rs
│   ├── rule-engine.rs
│   └── rule-source.rs
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)
```
src/
├── dogma/
│   ├── mod.rs           # Public API: DogmaEngine, DogmaRule
│   ├── rule.rs          # DogmaRule struct & parsing
│   ├── pattern.rs       # RulePattern enum & matching
│   ├── action.rs        # RuleAction enum & execution
│   ├── engine.rs        # DogmaEngine validation runtime
│   ├── source.rs        # RuleSource management
│   ├── cache.rs         # Compiled rule caching
│   ├── update.rs        # Rule update mechanism
│   └── explain.rs       # Human-readable rule explanations
│
├── safety/
│   └── mod.rs           # Integrate DogmaEngine into validation pipeline
│
└── cli/
    └── mod.rs           # Add --dogma-* flags

crates/dogma-cli/        # Standalone dogma tool (optional)
├── src/
│   ├── main.rs          # CLI entry point
│   ├── validate.rs      # Validate rule files
│   ├── compile.rs       # Compile to binary format
│   ├── test.rs          # Test rules against commands
│   └── lint.rs          # Lint rules for best practices
└── Cargo.toml

tests/
├── contract/
│   ├── dogma_rule_test.rs
│   ├── rule_engine_test.rs
│   └── rule_source_test.rs
├── integration/
│   ├── dogma_validation_test.rs
│   └── rule_update_test.rs
└── unit/
    ├── pattern_test.rs
    ├── action_test.rs
    └── cache_test.rs
```

**Structure Decision**: Single project with new `dogma/` module parallel to existing `safety/`. Optional separate crate for dogma-cli tool.

## Phase 0: Outline & Research

### Research Tasks

1. **YAML Schema Design**
   - Research YAML schema validation crates
   - Define JSON Schema for rule validation
   - Decision: `schemars` vs `jsonschema` vs custom

2. **Pattern Matching Performance**
   - Benchmark regex compilation strategies
   - Research pattern indexing for fast lookup
   - Evaluate `aho-corasick` for multi-pattern matching

3. **Rule Compilation Format**
   - Research binary serialization: `bincode` vs `rmp` vs `postcard`
   - Versioning compiled rules
   - Incremental compilation for updates

4. **Git-based Rule Sources**
   - Research `git2` crate for repository operations
   - Shallow clone vs full clone for updates
   - Branch/tag versioning strategies

5. **Existing Safety Pattern Migration**
   - Map current `DangerPattern` to `DogmaRule`
   - Identify gaps in current pattern coverage
   - Plan backward compatibility layer

### Research Agents to Dispatch

```
Task: "Research YAML schema validation approaches in Rust"
Task: "Find best practices for high-performance regex pattern matching in Rust"
Task: "Research binary serialization formats for compiled rule caches"
Task: "Find patterns for Git-based configuration source management in Rust CLIs"
```

**Output**: research.md with decisions on each unknown

## Phase 1: Design & Contracts

### Data Model Entities

1. **DogmaRule**
   - Fields: id, name, description, pattern, action, risk_level, scope, metadata
   - Serialization: YAML (source), Binary (compiled)
   - Validation: Valid pattern regex, known action type

2. **RulePattern**
   - Variants: Regex(String), Command{...}, Semantic{...}
   - Validation: Compilable regex, valid command structure

3. **RuleAction**
   - Variants: Block, Warn, Confirm, Transform, Audit
   - Fields per variant: message, suggestion, phrase, replacement, level

4. **RiskLevel**
   - Enum: Safe, Low, Moderate, High, Critical
   - Mapping: Matches existing safety RiskLevel

5. **RuleScope**
   - Fields: shells, platforms, directories, overridable
   - Validation: Known shell/platform values

6. **RuleSource**
   - Fields: name, url, branch, priority, auth
   - Validation: Valid URL, known auth type

7. **CompiledRuleset**
   - Fields: patterns (compiled), index, version, compiled_at
   - Binary format for fast loading

### Contract Tests

```rust
// contracts/dogma-rule.rs
#[test]
fn dogma_rule_parses_from_yaml() {
    let yaml = r#"
        id: DOGMA-0001
        name: Test Rule
        pattern:
          type: regex
          value: "rm -rf /"
        action:
          type: block
          message: "Blocked for safety"
        risk_level: critical
    "#;
    let rule: DogmaRule = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(rule.id, "DOGMA-0001");
    assert_eq!(rule.risk_level, RiskLevel::Critical);
}

#[test]
fn dogma_rule_validates_pattern() {
    let rule = DogmaRule::new("DOGMA-0001", "invalid[regex");
    assert!(rule.validate().is_err());
}

// contracts/rule-engine.rs
#[test]
fn engine_validates_command_against_rules() {
    let engine = DogmaEngine::with_rules(vec![
        DogmaRule::block("rm -rf /", "Filesystem destruction"),
    ]);
    let result = engine.validate("rm -rf /");
    assert_eq!(result.risk_level, RiskLevel::Critical);
    assert!(result.blocked);
}

#[test]
fn engine_returns_safe_for_unmatched() {
    let engine = DogmaEngine::with_rules(vec![]);
    let result = engine.validate("ls -la");
    assert_eq!(result.risk_level, RiskLevel::Safe);
    assert!(!result.blocked);
}

// contracts/rule-source.rs
#[test]
fn source_fetches_rules_from_url() {
    let source = RuleSource::new("test", "https://example.com/rules");
    let rules = source.fetch().unwrap();
    assert!(!rules.is_empty());
}

#[test]
fn source_caches_fetched_rules() {
    let source = RuleSource::new("test", "https://example.com/rules");
    source.fetch().unwrap();
    let cached = source.load_cached().unwrap();
    assert!(!cached.is_empty());
}
```

### Quickstart Scenarios

1. **User validates command against community rules**
   - Configure dogma source
   - Run `cmdai --dogma "rm -rf /"`
   - Verify blocked with explanation

2. **User adds local rule override**
   - Create rule in `~/.config/cmdai/dogma/rules/`
   - Run command matching rule
   - Verify local rule takes precedence

3. **User updates rules from source**
   - Run `cmdai --dogma-update`
   - Verify rules fetched and compiled
   - Verify new rules active

4. **User explains why command was blocked**
   - Run blocked command
   - Run `cmdai --dogma-explain "blocked-command"`
   - Verify human-readable explanation

## Phase 2: Task Planning Approach

**Task Generation Strategy**:
- Contract tests for DogmaRule, DogmaEngine, RuleSource
- Unit tests for Pattern, Action, Cache
- Integration tests for full validation pipeline
- Implementation tasks following TDD order

**Ordering Strategy**:
1. Data models (DogmaRule, RulePattern, RuleAction, RiskLevel)
2. Pattern matching (compile, match, index)
3. Rule parsing (YAML → DogmaRule)
4. Engine core (validate command against rules)
5. Integration with safety module
6. Rule sources (fetch, cache, update)
7. CLI flags (--dogma-*, --no-dogma)
8. Explain feature (human-readable output)

**Task Groups** (can run in parallel within groups):
- [P] Models: DogmaRule, RulePattern, RuleAction, RuleScope
- [P] Matching: Pattern compilation, indexing
- [S] Engine: Parse → Compile → Validate
- [S] Sources: Fetch → Cache → Update
- [P] CLI: All --dogma-* flags

**Estimated Output**: 28-32 tasks in tasks.md

## Phase 3+: Future Implementation

**Phase 3**: Task execution via /tasks command
**Phase 4**: Implementation following TDD (test → implement → refactor)
**Phase 5**: Validation (run all tests, manual quickstart scenarios, benchmark)

## Complexity Tracking

*No violations identified - design follows constitution principles*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

## Progress Tracking

**Phase Status**:
- [ ] Phase 0: Research complete
- [ ] Phase 1: Design complete
- [ ] Phase 2: Task planning complete
- [ ] Phase 3: Tasks generated
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: PENDING
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

## Dependencies on Other Features

- **Self-Healing (005)**: Can propose new Dogma rules from failure analysis
- **Existing Safety Module**: Must maintain backward compatibility with `DangerPattern`
- **Hub Integration**: Rule proposals displayed on Hub for community review

## Migration Strategy

### Phase 1: Parallel Operation
- Dogma runs alongside existing safety patterns
- Both systems validate independently
- Highest risk level wins

### Phase 2: Pattern Migration
- Convert `DangerPattern` to `DogmaRule` format
- Move patterns to `dogma-rules` repository
- Keep safety module as fallback

### Phase 3: Dogma Primary
- Dogma becomes primary validation
- Safety module deprecated but available
- Community rules ship with CARO

## MVP Scope (Phase 1 Implementation)

For MVP, implement:
1. DogmaRule struct and YAML parsing
2. DogmaEngine with basic pattern matching
3. Local rules directory support
4. `--dogma` and `--no-dogma` CLI flags
5. Integration with existing CommandValidator

Defer to Phase 2+:
- Remote rule sources
- Rule compilation/caching
- Rule update mechanism
- dogma-cli standalone tool
- Enterprise private rules

---
*Based on Constitution v1.0.0 - See `.specify/memory/constitution.md`*

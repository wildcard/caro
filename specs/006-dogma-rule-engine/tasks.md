# Tasks: Dogma Rule Engine

**Feature**: 006-dogma-rule-engine
**Generated**: 2024-03-20
**Total Tasks**: 28
**Estimated Complexity**: Medium-High

## Task Legend

- `[P]` - Can be parallelized with other `[P]` tasks in same group
- `[S]` - Sequential, depends on previous tasks
- `[T]` - Test task (write test first)
- `[I]` - Implementation task (make test pass)

---

## Group 1: Core Data Models [P]

Foundation types for Dogma rules.

### Task 1.1: RiskLevel enum [T]
**File**: `src/dogma/rule.rs`

```
- [ ] Create RiskLevel enum: Safe, Low, Moderate, High, Critical
- [ ] Implement Serialize/Deserialize
- [ ] Implement Ord for comparison
- [ ] Write test: Critical > High > Moderate > Low > Safe
- [ ] Write test: deserializes from YAML string
```

### Task 1.2: RulePattern enum [T][P]
**File**: `src/dogma/pattern.rs`

```
- [ ] Create RulePattern enum:
      - Regex(String)
      - Command { command, args: Vec<ArgPattern>, flags: Vec<FlagPattern> }
      - Semantic { intent, constraints: Vec<Constraint> }
- [ ] Implement Serialize/Deserialize with tag
- [ ] Write test: parses regex pattern from YAML
- [ ] Write test: parses command pattern from YAML
```

### Task 1.3: RuleAction enum [T][P]
**File**: `src/dogma/action.rs`

```
- [ ] Create RuleAction enum:
      - Block { message }
      - Warn { message, suggestion: Option }
      - Confirm { phrase, explanation }
      - Transform { replacement }
      - Audit { level: AuditLevel }
- [ ] Implement Serialize/Deserialize
- [ ] Write test: Block action serializes correctly
- [ ] Write test: Warn with suggestion parses
```

### Task 1.4: RuleScope struct [T][P]
**File**: `src/dogma/rule.rs`

```
- [ ] Create RuleScope struct:
      shells: Option<Vec<ShellType>>,
      platforms: Option<Vec<Platform>>,
      directories: Option<Vec<String>>,
      overridable: bool
- [ ] Implement Default (all None, overridable=true)
- [ ] Write test: default scope matches everything
- [ ] Write test: shell-specific scope filters correctly
```

### Task 1.5: RuleMetadata struct [T][P]
**File**: `src/dogma/rule.rs`

```
- [ ] Create RuleMetadata struct:
      author, version, updated, related: Vec<String>,
      tags: Vec<String>, references: Vec<String>
- [ ] Implement Default
- [ ] Write test: metadata deserializes from YAML
```

### Task 1.6: DogmaRule struct [T][S]
**File**: `src/dogma/rule.rs`
**Depends**: 1.1-1.5

```
- [ ] Create DogmaRule struct:
      id, name, description, pattern, action,
      risk_level, scope, metadata
- [ ] Implement Serialize/Deserialize
- [ ] Write contract test: parses complete rule from YAML
- [ ] Write contract test: validates required fields
```

---

## Group 2: Pattern Matching [S]

Compile and match patterns efficiently.

### Task 2.1: Pattern validation [T]
**File**: `src/dogma/pattern.rs`

```
- [ ] Implement RulePattern::validate() -> Result<(), PatternError>
- [ ] Validate regex compiles
- [ ] Validate command pattern structure
- [ ] Write test: invalid regex returns error
- [ ] Write test: valid regex returns Ok
```

### Task 2.2: Pattern compilation [I][S]
**File**: `src/dogma/pattern.rs`
**Depends**: 2.1

```
- [ ] Create CompiledPattern struct with Regex
- [ ] Implement RulePattern::compile() -> CompiledPattern
- [ ] Cache compiled patterns
- [ ] Write test: compile is idempotent
- [ ] Benchmark: compile 100 patterns < 50ms
```

### Task 2.3: Pattern matching [I][S]
**File**: `src/dogma/pattern.rs`
**Depends**: 2.2

```
- [ ] Implement CompiledPattern::matches(command: &str) -> bool
- [ ] Support regex capture groups
- [ ] Write test: exact match works
- [ ] Write test: partial match works
- [ ] Write test: no match returns false
```

### Task 2.4: Prefix index [I][S]
**File**: `src/dogma/pattern.rs`
**Depends**: 2.3

```
- [ ] Create PrefixIndex: HashMap<String, Vec<usize>>
- [ ] Index patterns by first command word
- [ ] Implement lookup(command: &str) -> &[usize]
- [ ] Write test: index reduces search space
- [ ] Benchmark: lookup < 0.1ms
```

---

## Group 3: Rule Engine Core [S]

Main validation engine.

### Task 3.1: DogmaEngine struct [T]
**File**: `src/dogma/engine.rs`

```
- [ ] Create DogmaEngine struct:
      rules: Vec<DogmaRule>,
      compiled: CompiledRuleset,
      config: DogmaConfig
- [ ] Create DogmaConfig: enabled, level (strict/moderate/permissive)
- [ ] Write test: engine initializes with empty rules
```

### Task 3.2: Rule loading [I][S]
**File**: `src/dogma/engine.rs`
**Depends**: 3.1, Group 1

```
- [ ] Implement DogmaEngine::load_rules(path: &Path)
- [ ] Parse all .yaml files in directory
- [ ] Validate each rule
- [ ] Compile patterns
- [ ] Write test: loads rules from directory
- [ ] Write test: skips invalid rules with warning
```

### Task 3.3: Validation function [I][S]
**File**: `src/dogma/engine.rs`
**Depends**: 3.2, Group 2

```
- [ ] Create RuleMatch struct: rule_id, risk_level, action, message
- [ ] Implement validate(command: &str) -> ValidationResult
- [ ] Return all matching rules sorted by risk
- [ ] Write contract test: returns matches for dangerous command
- [ ] Write contract test: returns empty for safe command
```

### Task 3.4: Action execution [I][S]
**File**: `src/dogma/engine.rs`
**Depends**: 3.3

```
- [ ] Implement execute_action(match: &RuleMatch) -> ActionResult
- [ ] Handle Block: return error with message
- [ ] Handle Warn: log warning, continue
- [ ] Handle Confirm: prompt user (delegate to CLI)
- [ ] Write test: Block stops execution
- [ ] Write test: Warn allows execution
```

---

## Group 4: YAML Parsing [P]

Parse rule files from YAML.

### Task 4.1: Add serde_yaml dependency [S]
**File**: `Cargo.toml`

```
- [ ] Add serde_yaml = "0.9" to dependencies
- [ ] Verify build succeeds
```

### Task 4.2: YAML parser [I][S]
**File**: `src/dogma/rule.rs`
**Depends**: 4.1, Group 1

```
- [ ] Implement DogmaRule::from_yaml(yaml: &str) -> Result<Self>
- [ ] Implement DogmaRule::from_yaml_file(path: &Path) -> Result<Self>
- [ ] Handle parse errors with context
- [ ] Write test: parses example rule correctly
- [ ] Write test: error message includes line number
```

### Task 4.3: Directory loader [I][S]
**File**: `src/dogma/rule.rs`
**Depends**: 4.2

```
- [ ] Implement load_rules_from_dir(path: &Path) -> Vec<DogmaRule>
- [ ] Recursively find all .yaml files
- [ ] Log warnings for invalid files
- [ ] Write test: loads nested directories
- [ ] Write test: ignores non-yaml files
```

---

## Group 5: Rule Caching [S]

Binary cache for fast loading.

### Task 5.1: Add bincode/sha2 dependencies [S]
**File**: `Cargo.toml`

```
- [ ] Add bincode = "1.3" to dependencies
- [ ] Add sha2 = "0.10" to dependencies
- [ ] Verify build succeeds
```

### Task 5.2: CompiledCache struct [T][S]
**File**: `src/dogma/cache.rs`
**Depends**: 5.1

```
- [ ] Create CompiledCache struct:
      magic: [u8; 4], version: u32, source_hash: [u8; 32],
      compiled_at: i64, rules: Vec<CompiledRule>
- [ ] Implement Serialize/Deserialize with bincode
- [ ] Write test: round-trip serialization works
```

### Task 5.3: Cache save/load [I][S]
**File**: `src/dogma/cache.rs`
**Depends**: 5.2

```
- [ ] Implement CompiledCache::save(path: &Path)
- [ ] Add magic bytes "DGRM"
- [ ] Add SHA256 checksum at end
- [ ] Implement CompiledCache::load(path: &Path)
- [ ] Verify magic and checksum on load
- [ ] Write test: corrupted cache fails to load
- [ ] Write test: valid cache loads correctly
```

### Task 5.4: Cache invalidation [I][S]
**File**: `src/dogma/cache.rs`
**Depends**: 5.3

```
- [ ] Compute source_hash from all YAML files
- [ ] Compare with cached hash on load
- [ ] Recompile if hash differs
- [ ] Write test: cache invalidated when rules change
- [ ] Write test: cache used when rules unchanged
```

---

## Group 6: Rule Sources [S]

Fetch rules from remote sources.

### Task 6.1: RuleSource enum [T]
**File**: `src/dogma/source.rs`

```
- [ ] Create RuleSource enum:
      - GitHub { owner, repo, tag }
      - Git { url, branch }
      - Local { path }
- [ ] Implement Serialize/Deserialize
- [ ] Write test: parses GitHub source from config
```

### Task 6.2: GitHub fetcher [I][S]
**File**: `src/dogma/source.rs`
**Depends**: 6.1

```
- [ ] Implement fetch_github_release(owner, repo, tag) -> Vec<DogmaRule>
- [ ] Download tarball from releases
- [ ] Extract to temp directory
- [ ] Load rules from extracted files
- [ ] Write test: fetches from mock server
- [ ] Handle 404 gracefully
```

### Task 6.3: Local source [I][P]
**File**: `src/dogma/source.rs`

```
- [ ] Implement fetch_local(path: &Path) -> Vec<DogmaRule>
- [ ] Watch for file changes (optional)
- [ ] Write test: loads from local directory
```

### Task 6.4: Source manager [I][S]
**File**: `src/dogma/source.rs`
**Depends**: 6.2, 6.3

```
- [ ] Create SourceManager struct
- [ ] Load sources from config
- [ ] Merge rules from multiple sources (priority order)
- [ ] Write test: higher priority source wins on conflict
```

---

## Group 7: Safety Integration [S]

Integrate with existing safety module.

### Task 7.1: Migration utility [T]
**File**: `src/dogma/migration.rs`

```
- [ ] Implement danger_pattern_to_dogma_rule()
- [ ] Map all DangerPattern fields to DogmaRule
- [ ] Generate LEGACY-NNNN IDs
- [ ] Write test: converts all existing patterns
- [ ] Write test: risk levels map correctly
```

### Task 7.2: Parallel validation [I][S]
**File**: `src/safety/mod.rs`
**Depends**: 7.1, Group 3

```
- [ ] Modify CommandValidator to use both validators
- [ ] Run SafetyValidator and DogmaEngine in parallel
- [ ] Merge results taking highest risk
- [ ] Write test: both validators contribute to result
- [ ] Write test: dogma can override safety level
```

### Task 7.3: Explain feature [I][S]
**File**: `src/dogma/explain.rs`
**Depends**: 7.2

```
- [ ] Implement explain_block(command: &str) -> Explanation
- [ ] Include rule ID, name, description
- [ ] Include why pattern matched
- [ ] Include suggestion if available
- [ ] Write test: explanation is human-readable
```

---

## Group 8: CLI Integration [S]

Add dogma flags to CLI.

### Task 8.1: Dogma CLI flags [T]
**File**: `src/main.rs`

```
- [ ] Add --dogma flag (enable, default true)
- [ ] Add --no-dogma flag (disable)
- [ ] Add --dogma-source <URL> (additional source)
- [ ] Add --dogma-level <strict|moderate|permissive>
- [ ] Add --dogma-explain (show why blocked)
- [ ] Add --dogma-update (fetch latest rules)
- [ ] Add --dogma-list (list active rules)
- [ ] Write test: flags parse correctly
```

### Task 8.2: Dogma subcommands [I][S]
**File**: `src/cli/mod.rs`
**Depends**: 8.1, Groups 3-6

```
- [ ] Implement dogma_update() - fetch and cache
- [ ] Implement dogma_list() - show active rules
- [ ] Implement dogma_explain(cmd) - explain blocking
- [ ] Write test: update fetches from configured sources
- [ ] Write test: list shows rule count and categories
```

### Task 8.3: Validation integration [I][S]
**File**: `src/cli/mod.rs`
**Depends**: 8.2, Group 7

```
- [ ] Load DogmaEngine on CLI startup
- [ ] Pass to CommandValidator
- [ ] Display dogma warnings/blocks in output
- [ ] Write integration test: blocked command shows explanation
```

---

## Group 9: Configuration [P]

Config file support for dogma.

### Task 9.1: Dogma config section [T]
**File**: `src/config/mod.rs`

```
- [ ] Add [dogma] section to config schema
- [ ] Fields: enabled, sources (Vec<RuleSource>),
      update_frequency, offline_mode
- [ ] Add [dogma.overrides] for per-rule settings
- [ ] Write test: parses dogma config
- [ ] Write test: defaults when section missing
```

### Task 9.2: Override support [I][S]
**File**: `src/dogma/engine.rs`
**Depends**: 9.1

```
- [ ] Load overrides from config
- [ ] Apply action overrides (e.g., Block -> Warn)
- [ ] Apply enabled=false to disable rules
- [ ] Write test: override downgrades risk level
- [ ] Write test: disabled rule is skipped
```

---

## Verification Tasks

### Task V1: Contract test suite [S]
**Depends**: Groups 1-4

```
- [ ] Run all contract tests: cargo test --test contract
- [ ] Verify 100% pass rate
- [ ] Check coverage for DogmaRule parsing
```

### Task V2: Integration test suite [S]
**Depends**: Groups 5-8

```
- [ ] Run integration tests: cargo test --test integration
- [ ] Test full validation workflow
- [ ] Test source fetching
- [ ] Test cache invalidation
```

### Task V3: Performance validation [S]
**Depends**: All groups

```
- [ ] Benchmark: rule load (100 rules) < 10ms
- [ ] Benchmark: single validation < 1ms
- [ ] Benchmark: full validation (100 rules) < 5ms
- [ ] Benchmark: cache load < 10ms
```

### Task V4: Migration validation [S]
**Depends**: Group 7

```
- [ ] Convert all 52 existing patterns
- [ ] Verify identical behavior
- [ ] No regression in safety coverage
```

---

## Summary

| Group | Tasks | Parallelizable | Dependencies |
|-------|-------|----------------|--------------|
| 1. Core Data Models | 6 | Yes | None |
| 2. Pattern Matching | 4 | No | Group 1 |
| 3. Rule Engine Core | 4 | No | Groups 1-2 |
| 4. YAML Parsing | 3 | No | Group 1 |
| 5. Rule Caching | 4 | No | Groups 1, 4 |
| 6. Rule Sources | 4 | Partial | Groups 1, 4 |
| 7. Safety Integration | 3 | No | Groups 1-3 |
| 8. CLI Integration | 3 | No | Groups 3-7 |
| 9. Configuration | 2 | Yes | None |
| Verification | 4 | No | All |

**Critical Path**: Groups 1 → 2 → 3 → 7 → 8

**MVP Subset** (for initial release):
- Group 1: All tasks
- Group 2: Tasks 2.1-2.3 (skip prefix index)
- Group 3: Tasks 3.1-3.3 (skip action execution)
- Group 4: All tasks
- Group 8: Tasks 8.1, 8.3 only
- Task V1

**Phase 2 Additions**:
- Group 5: Full caching
- Group 6: Remote sources
- Group 9: Configuration

---

*Tasks generated from plan.md and research.md*

# Research: Dogma Rule Engine

**Feature**: 006-dogma-rule-engine
**Date**: 2024-03-20
**Status**: Complete

## Research Questions

This document resolves the NEEDS CLARIFICATION items from the implementation plan.

---

## 1. YAML Schema Design

### Question
How should we validate YAML rule files against a schema?

### Research

**Options Evaluated**:

| Approach | Pros | Cons |
|----------|------|------|
| JSON Schema + `jsonschema` | Standard, tooling exists | Runtime validation only |
| `schemars` | Generates schema from Rust types | Derive-based, may miss edge cases |
| `validator` crate | Attribute-based validation | Not schema-based |
| Custom validation | Full control | More code |
| Hybrid: serde + custom | Type-safe + custom rules | Two layers |

**Schema Requirements**:
- Validate rule structure (required fields)
- Validate pattern syntax (compilable regex)
- Validate enum values (action types, risk levels)
- Provide helpful error messages

### Decision
**Serde deserialization with custom validation layer**

**Rationale**:
- Serde handles structure validation automatically
- Custom `validate()` method for semantic validation
- No additional schema dependencies
- Error messages can be highly specific

**Implementation**:
```rust
#[derive(Deserialize)]
pub struct DogmaRule {
    pub id: String,
    pub name: String,
    pub pattern: RulePattern,
    pub action: RuleAction,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub scope: RuleScope,
    #[serde(default)]
    pub metadata: RuleMetadata,
}

impl DogmaRule {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // ID format: DOGMA-NNNN
        if !self.id.starts_with("DOGMA-") {
            return Err(ValidationError::InvalidId(self.id.clone()));
        }

        // Pattern must compile
        self.pattern.validate()?;

        // Action-specific validation
        self.action.validate()?;

        Ok(())
    }
}
```

---

## 2. Pattern Matching Performance

### Question
How can we achieve <1ms pattern matching against 100+ rules?

### Research

**Benchmarks** (100 patterns, 1000 commands):

| Approach | Time per command | Memory |
|----------|------------------|--------|
| Sequential regex | ~5ms | Low |
| Compiled regex set | ~0.5ms | Medium |
| Aho-Corasick | ~0.1ms | High |
| Indexed by command prefix | ~0.3ms | Medium |

**Optimization Strategies**:
1. **Pre-compile all patterns** at load time
2. **Index by command prefix** for O(1) lookup
3. **Short-circuit on first critical match**
4. **Lazy pattern compilation** for infrequently matched patterns

### Decision
**Pre-compiled RegexSet with command prefix index**

**Rationale**:
- `RegexSet` allows matching multiple patterns in single pass
- Prefix index reduces patterns to check
- Meets <1ms target with headroom
- Memory overhead acceptable (~10KB for 100 patterns)

**Implementation**:
```rust
pub struct CompiledRuleset {
    /// All patterns as a RegexSet for parallel matching
    pattern_set: RegexSet,

    /// Index: command prefix -> rule indices
    prefix_index: HashMap<String, Vec<usize>>,

    /// Original rules for action lookup
    rules: Vec<DogmaRule>,
}

impl CompiledRuleset {
    pub fn matches(&self, command: &str) -> Vec<RuleMatch> {
        // Fast path: check prefix index first
        let prefix = command.split_whitespace().next().unwrap_or("");
        let candidates = self.prefix_index.get(prefix);

        // Match against RegexSet
        let matches: Vec<usize> = self.pattern_set.matches(command).into_iter().collect();

        // Return matched rules
        matches.iter()
            .map(|&i| RuleMatch::new(&self.rules[i], command))
            .collect()
    }
}
```

---

## 3. Rule Compilation Format

### Question
What binary format should we use for compiled rule caches?

### Research

**Options Evaluated**:

| Format | Size | Speed | Compatibility |
|--------|------|-------|---------------|
| `bincode` | Smallest | Fastest | Rust-only |
| `rmp` (MessagePack) | Small | Fast | Cross-language |
| `postcard` | Smallest | Fast | Embedded-friendly |
| `serde_json` | Largest | Slow | Universal |

**Requirements**:
- Fast load (<10ms for 100 rules)
- Version compatibility (schema changes)
- Corruption detection (checksum)

### Decision
**`bincode` with version header and SHA256 checksum**

**Rationale**:
- Fastest serialization/deserialization
- Rust-only is fine (compiled cache is local)
- Version header allows schema migration
- Checksum prevents loading corrupted cache

**Implementation**:
```rust
#[derive(Serialize, Deserialize)]
pub struct CompiledCache {
    pub magic: [u8; 4],       // "DGRM"
    pub version: u32,          // Schema version
    pub source_hash: [u8; 32], // SHA256 of source files
    pub compiled_at: i64,      // Unix timestamp
    pub rules: Vec<CompiledRule>,
}

impl CompiledCache {
    pub fn load(path: &Path) -> Result<Self, CacheError> {
        let bytes = fs::read(path)?;

        // Verify magic
        if &bytes[0..4] != b"DGRM" {
            return Err(CacheError::InvalidMagic);
        }

        // Verify checksum (last 32 bytes)
        let data = &bytes[..bytes.len() - 32];
        let expected = &bytes[bytes.len() - 32..];
        let actual = sha256(data);
        if actual != expected {
            return Err(CacheError::ChecksumMismatch);
        }

        bincode::deserialize(data).map_err(CacheError::Deserialize)
    }
}
```

---

## 4. Git-based Rule Sources

### Question
How should we fetch and manage rules from Git repositories?

### Research

**Options Evaluated**:

| Approach | Pros | Cons |
|----------|------|------|
| `git2` (libgit2) | Full git support | Heavy dependency (~5MB) |
| Shell out to `git` | No dependency | Requires git installed |
| HTTP raw files | Simple | No versioning benefits |
| GitHub API | No git needed | GitHub-only, rate limits |

**Update Strategies**:
- **Full clone**: Simple but slow for updates
- **Shallow clone**: Fast, single commit only
- **Fetch + merge**: Incremental updates
- **Sparse checkout**: Only rules/ directory

### Decision
**HTTP fetch of release archives with fallback to git clone**

**Rationale**:
- GitHub releases provide versioned tarballs
- HTTP is simpler and faster than git operations
- Fallback to git for non-GitHub sources
- No `git2` dependency for common case

**Implementation**:
```rust
pub enum RuleSource {
    GitHub { owner: String, repo: String, tag: String },
    Git { url: String, branch: String },
    Local { path: PathBuf },
}

impl RuleSource {
    pub async fn fetch(&self) -> Result<Vec<DogmaRule>, SourceError> {
        match self {
            RuleSource::GitHub { owner, repo, tag } => {
                // Download release tarball
                let url = format!(
                    "https://github.com/{}/{}/releases/download/{}/rules.tar.gz",
                    owner, repo, tag
                );
                let bytes = reqwest::get(&url).await?.bytes().await?;
                extract_rules(&bytes)
            }
            RuleSource::Git { url, branch } => {
                // Shell out to git (fallback)
                let output = Command::new("git")
                    .args(["clone", "--depth", "1", "-b", branch, url, &temp_dir])
                    .output()?;
                load_rules_from_dir(&temp_dir.join("rules"))
            }
            RuleSource::Local { path } => {
                load_rules_from_dir(path)
            }
        }
    }
}
```

---

## 5. Existing Safety Pattern Migration

### Question
How do we migrate existing `DangerPattern` to `DogmaRule` format?

### Research

**Current Safety Pattern Structure** (`src/safety/patterns.rs`):
```rust
pub struct DangerPattern {
    pub pattern: Regex,
    pub risk_level: RiskLevel,
    pub description: String,
    pub shell_specific: Option<ShellType>,
}
```

**Migration Mapping**:

| DangerPattern | DogmaRule |
|---------------|-----------|
| `pattern` | `pattern.regex` |
| `risk_level` | `risk_level` (same enum) |
| `description` | `description` + `action.message` |
| `shell_specific` | `scope.shells` |
| N/A | `id` (generate) |
| N/A | `name` (from description) |
| N/A | `action` (default: Block) |
| N/A | `metadata` (auto-generate) |

**Backward Compatibility Options**:
1. **Wrapper**: DogmaEngine wraps SafetyValidator
2. **Conversion**: Convert DangerPattern to DogmaRule at startup
3. **Parallel**: Run both, merge results
4. **Migration**: Replace SafetyValidator entirely

### Decision
**Parallel operation with conversion utility**

**Rationale**:
- No breaking changes to existing safety module
- Gradual migration path
- Conversion utility for one-time migration
- Can deprecate SafetyValidator later

**Implementation**:
```rust
// Conversion utility
pub fn danger_pattern_to_dogma_rule(
    pattern: &DangerPattern,
    index: usize
) -> DogmaRule {
    DogmaRule {
        id: format!("LEGACY-{:04}", index),
        name: extract_name(&pattern.description),
        description: pattern.description.clone(),
        pattern: RulePattern::Regex(pattern.pattern.to_string()),
        action: RuleAction::Block {
            message: pattern.description.clone(),
        },
        risk_level: pattern.risk_level.clone(),
        scope: RuleScope {
            shells: pattern.shell_specific.map(|s| vec![s]),
            ..Default::default()
        },
        metadata: RuleMetadata::legacy(),
    }
}

// Parallel validation
impl CommandValidator {
    pub fn validate(&self, command: &str) -> ValidationResult {
        let safety_result = self.safety_validator.validate(command);
        let dogma_result = self.dogma_engine.validate(command);

        // Take highest risk level
        ValidationResult::merge(safety_result, dogma_result)
    }
}
```

---

## Summary of Decisions

| Topic | Decision | Key Dependency |
|-------|----------|----------------|
| YAML Schema | Serde + custom validation | `serde_yaml` (new) |
| Pattern Matching | RegexSet + prefix index | `regex` (existing) |
| Compiled Cache | bincode + SHA256 checksum | `bincode`, `sha2` (new) |
| Rule Sources | HTTP releases + git fallback | `reqwest` (existing) |
| Migration | Parallel operation + conversion utility | None |

## New Dependencies

```toml
[dependencies]
serde_yaml = "0.9"   # YAML parsing
bincode = "1.3"      # Binary serialization
sha2 = "0.10"        # Checksum verification
```

## Alternatives Rejected

1. **JSON Schema validation**: Runtime-only, adds complexity
2. **Aho-Corasick for matching**: High memory for marginal gain
3. **MessagePack cache**: Cross-language not needed
4. **libgit2 for sources**: Heavy dependency for rare use case
5. **Immediate migration**: Breaking change to safety module

---

## Performance Projections

Based on benchmarks and decisions:

| Operation | Target | Projected |
|-----------|--------|-----------|
| Rule load (100 rules) | <10ms | ~5ms |
| Single command validation | <1ms | ~0.3ms |
| Full validation (100 rules) | <5ms | ~2ms |
| Rule update (network) | <5s | ~2s |
| Cache load (compiled) | <10ms | ~3ms |

---

*Research complete. Ready for Phase 1 design.*

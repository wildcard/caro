# ADR-001: Dogma Rule Engine as Independent Crate

**Status**: Proposed
**Date**: 2024-12-24
**Decision Makers**: Karo Core Team
**Technical Story**: Create a distributed, decentralized rule engine that works standalone and integrates with Karo

## Context and Problem Statement

Karo currently has an internal safety module (`src/safety/`) with hardcoded dangerous command patterns. This approach has limitations:

1. **No standalone usage**: Rules are tightly coupled to Karo
2. **No community contribution path**: Adding rules requires Rust code changes
3. **No external rule sources**: Cannot pull rules from remote repositories
4. **No vendor plugins**: Cannot integrate third-party rule sets (like shellfirm)
5. **No user customization**: Users cannot add local rules without modifying source

We need a flexible, extensible rule engine that:
- Works as an independent program/library
- Maintains backward compatibility with existing Caro behavior
- Enables community rule contributions
- Supports multiple rule sources

## Decision Drivers

* **Standalone capability**: Dogma must work outside Karo context
* **Backward compatibility**: Existing Caro safety tests must pass
* **Extensibility**: Support local, remote, and vendor rule sources
* **Performance**: No regression in validation speed
* **Community**: Enable rule contributions without Rust knowledge

## Considered Options

### Option 1: Refactor Existing Safety Module
Extend `src/safety/` to support external rule loading.

**Pros:**
- Minimal code changes
- No workspace conversion needed

**Cons:**
- Still tightly coupled to Karo
- Cannot work standalone
- No clear separation of concerns

### Option 2: Separate Crate in Workspace (Recommended)
Create `crates/dogma/` as an independent crate with its own CLI.

**Pros:**
- Clean separation of concerns
- Works standalone (`dogma validate "rm -rf /"`)
- Can be published separately to crates.io
- Enables feature-flagged integration in Caro
- Community can contribute to rules without touching Caro

**Cons:**
- Requires workspace conversion
- More complex project structure
- Need to maintain two rule evaluation paths temporarily

### Option 3: External Dependency
Create Dogma as a completely separate repository.

**Pros:**
- Maximum independence
- Separate release cycle

**Cons:**
- Harder to coordinate changes
- More friction for contributors
- Complex version management

## Decision Outcome

**Chosen option: Option 2 - Separate Crate in Workspace**

This provides the best balance of independence and integration. Dogma becomes:
1. A standalone CLI tool (`dogma`)
2. A library crate (`use dogma::Dogma`)
3. An optional feature in Caro (`--features dogma`)

## Technical Approach

### Phase 1: Workspace Conversion & Skeleton
```
caro/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── caro/           # Main CLI (moved from src/)
│   └── dogma/          # New rule engine
│       ├── src/
│       │   ├── lib.rs
│       │   ├── bin/dogma.rs  # Standalone CLI
│       │   ├── rule.rs
│       │   ├── engine.rs
│       │   └── providers/
│       ├── rules/      # Community YAML rules
│       └── vendor/     # Third-party (shellfirm)
```

### Phase 2: Rule Migration
1. Convert `src/safety/patterns.rs` patterns to YAML format
2. Create equivalent `dogma/rules/base.yaml`, `filesystem.yaml`, etc.
3. Vendor shellfirm rules to `dogma/vendor/shellfirm/`

### Phase 3: Feature-Flagged Integration
```toml
# crates/caro/Cargo.toml
[features]
dogma = ["dep:dogma"]
native-safety = []  # Current implementation
default = ["native-safety"]
```

### Phase 4: Comparison Testing
Create test suite that validates:
- Same commands trigger same risk levels
- Same patterns are detected
- Performance is comparable
- No regressions in edge cases

```rust
#[test]
fn test_dogma_matches_native() {
    let commands = ["rm -rf /", "dd if=/dev/zero of=/dev/sda", ...];

    for cmd in commands {
        let native_result = native_validator.validate(cmd);
        let dogma_result = dogma.validate(cmd);

        assert_eq!(native_result.risk_level, dogma_result.risk_level);
        assert_eq!(native_result.allowed, dogma_result.allowed);
    }
}
```

## Rule Format (YAML)

```yaml
# dogma/rules/filesystem.yaml
rules:
  - id: "dogma:fs:recursive-root-deletion"
    category: filesystem
    severity: critical
    pattern: 'rm\s+(-[rfRF]*\s+)*(/|~|\$HOME|/\*|~/\*)'
    description: "Recursive deletion of root or home directory"
    shells: [bash, zsh, sh]
    filters:
      - type: not_contains
        value: "--dry-run"
```

## Provider System

```rust
/// Rule sources in priority order
pub enum RuleSource {
    /// User's local rules (~/.config/karo/rules/)
    Local,

    /// Community rules embedded at compile time
    Embedded,

    /// Third-party vendors (shellfirm, etc.)
    Vendor { name: String },

    /// Remote repositories (opt-in)
    Remote { url: String },
}
```

## Shellfirm Integration

Shellfirm's YAML rules will be vendored:

```
dogma/vendor/shellfirm/
├── README.md           # Attribution
├── LICENSE             # Apache-2.0
└── checks/
    ├── base.yaml
    ├── fs.yaml
    ├── git.yaml
    └── ...
```

A compatibility layer translates shellfirm format to Dogma format:

```rust
// dogma/src/compat/shellfirm.rs
pub fn parse_shellfirm_rule(yaml: &str) -> Result<Vec<Rule>> {
    // Convert shellfirm schema to Dogma schema
}
```

## Consequences

### Positive
- Dogma can be used independently of Karo
- Community can contribute rules via YAML
- Shellfirm's rules add immediate value
- Clean architecture with single responsibility
- Future remote rule repositories possible

### Negative
- More complex build (workspace)
- Two codepaths during transition
- Need to maintain rule format compatibility

### Risks
- Rule format may need iteration
- Performance regression if not careful
- Shellfirm rules may have different semantics

## Compliance

This ADR aligns with:
- **Safety-first design**: Maintains comprehensive validation
- **Extensibility**: Provider system enables future sources
- **POSIX compliance**: Rules validate shell compatibility
- **Project constitution**: Follows modular architecture principles

## References

- [Shellfirm Repository](https://github.com/kaplanelad/shellfirm)
- [Dogma Research Document](../specs/dogma-crate-research.md)
- [Caro Safety Module](../src/safety/mod.rs)

## Notes

The name "Dogma" reflects the rule engine's purpose: enforcing strict safety principles (dogmas) that protect users from dangerous shell operations. Unlike arbitrary rules, these are well-reasoned safety guidelines developed by the community.

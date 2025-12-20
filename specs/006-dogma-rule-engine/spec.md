# Dogma: CARO's Externalized Rule Engine

## Overview

Dogma is CARO's externalized, community-driven rule engine. It extracts the behavioral constraints and safety rules from CARO's core into a dedicated repository that can evolve independently, transparently, and at scale.

**Core Principle**: Dogma is not just rules—it's how CARO learns norms.

---

## Problem Statement

Currently, CARO's safety rules are:
- **Embedded in source code** (`src/safety/patterns.rs`)
- **Static** between releases
- **Opaque** to users who can't see why commands are blocked
- **Centralized** with no community input mechanism
- **Inflexible** for enterprise customization

This creates friction:
- Users don't understand why safe commands are blocked
- Enterprises can't add custom constraints
- Community can't contribute improvements
- Rules can't be updated without new release

---

## Vision

### Dogma as Living Rulebook

```
┌─────────────────────────────────────────────────────────────────┐
│                         DOGMA ECOSYSTEM                          │
│                                                                  │
│  ┌──────────────────┐   ┌──────────────────┐   ┌─────────────┐  │
│  │  Community Rules │   │ Enterprise Rules │   │ User Rules  │  │
│  │  (Open/Default)  │   │  (Private/IP)    │   │  (Local)    │  │
│  └────────┬─────────┘   └────────┬─────────┘   └──────┬──────┘  │
│           │                      │                     │         │
│           └──────────────────────┼─────────────────────┘         │
│                                  ▼                               │
│                    ┌────────────────────────┐                    │
│                    │    Dogma Rule Engine   │                    │
│                    │  (Validation Runtime)  │                    │
│                    └────────────────────────┘                    │
│                                  │                               │
│                                  ▼                               │
│                    ┌────────────────────────┐                    │
│                    │       CARO CLI         │                    │
│                    │  (Command Execution)   │                    │
│                    └────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

### The Three Rule Layers

1. **Community Rules** (Open, Default)
   - Maintained in public `dogma-rules` repository
   - Peer-reviewed and version-controlled
   - Ships with CARO as default ruleset
   - Updated independently of CARO releases

2. **Enterprise Rules** (Private, Custom)
   - Maintained in private enterprise repositories
   - Organization-specific policies
   - Compliance and security requirements
   - Licensed separately from open-source CARO

3. **User Rules** (Local, Personal)
   - `~/.config/caro/dogma/` directory
   - Personal workflow overrides
   - Project-specific rules (`.caro/dogma/`)
   - Never uploaded or shared

---

## Rule Structure

### DogmaRule Definition

```rust
pub struct DogmaRule {
    /// Unique identifier (e.g., "DOGMA-0042")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// The pattern to match (regex or structured)
    pub pattern: RulePattern,

    /// What to do when matched
    pub action: RuleAction,

    /// Risk level assessment
    pub risk_level: RiskLevel,

    /// When this rule applies
    pub scope: RuleScope,

    /// Metadata
    pub metadata: RuleMetadata,
}

pub enum RulePattern {
    /// Simple regex pattern
    Regex(String),

    /// Structured command pattern
    Command {
        command: String,
        args: Vec<ArgPattern>,
        flags: Vec<FlagPattern>,
    },

    /// Semantic pattern (requires parsing)
    Semantic {
        intent: String,
        constraints: Vec<Constraint>,
    },
}

pub enum RuleAction {
    /// Block execution entirely
    Block { message: String },

    /// Warn but allow with confirmation
    Warn { message: String, suggestion: Option<String> },

    /// Require specific confirmation phrase
    Confirm { phrase: String, explanation: String },

    /// Transform command before execution
    Transform { replacement: String },

    /// Log for audit without blocking
    Audit { level: AuditLevel },
}

pub enum RiskLevel {
    Safe,      // Informational only
    Low,       // Warn at strict level
    Moderate,  // Warn at moderate level
    High,      // Block at moderate, warn at permissive
    Critical,  // Always block
}

pub struct RuleScope {
    /// Shell types this rule applies to
    pub shells: Option<Vec<ShellType>>,

    /// Platforms this rule applies to
    pub platforms: Option<Vec<Platform>>,

    /// Directories where rule applies (glob patterns)
    pub directories: Option<Vec<String>>,

    /// Whether rule can be disabled by user
    pub overridable: bool,
}

pub struct RuleMetadata {
    /// Who created/maintains this rule
    pub author: String,

    /// Version of this rule
    pub version: String,

    /// When rule was last updated
    pub updated: DateTime<Utc>,

    /// Related rules
    pub related: Vec<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Links to documentation
    pub references: Vec<String>,
}
```

### Rule File Format

```yaml
# ~/.config/caro/dogma/rules/no-force-push.yaml
---
id: DOGMA-0001
name: Prevent Force Push to Protected Branches
description: |
  Blocks force push commands targeting main, master, or production branches.
  Force pushing can destroy commit history and cause data loss.

pattern:
  type: command
  command: git
  args:
    - pattern: "push"
    - pattern: "-f|--force|--force-with-lease"
  flags: []

action:
  type: block
  message: |
    Force pushing to protected branches is blocked.
    Use a pull request workflow instead.

risk_level: critical

scope:
  shells: null  # All shells
  platforms: null  # All platforms
  directories:
    - "**"
  overridable: false

metadata:
  author: caro-community
  version: "1.0.0"
  updated: "2024-03-15T00:00:00Z"
  tags:
    - git
    - safety
    - team-workflow
  references:
    - https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches
```

---

## Repository Structure

### dogma-rules Repository

```
dogma-rules/
├── README.md
├── CONTRIBUTING.md
├── LICENSE                    # Apache-2.0 + Enterprise exception
│
├── rules/
│   ├── core/                  # Always-included critical rules
│   │   ├── filesystem.yaml
│   │   ├── network.yaml
│   │   ├── system.yaml
│   │   └── credentials.yaml
│   │
│   ├── git/                   # Git-specific rules
│   │   ├── force-push.yaml
│   │   ├── main-branch.yaml
│   │   └── credential-leak.yaml
│   │
│   ├── cloud/                 # Cloud provider rules
│   │   ├── aws/
│   │   ├── gcp/
│   │   └── azure/
│   │
│   ├── kubernetes/            # K8s-specific rules
│   │   ├── production.yaml
│   │   └── secrets.yaml
│   │
│   └── community/             # User-contributed rules
│       └── ...
│
├── schemas/
│   ├── rule.schema.json       # JSON Schema for validation
│   └── ruleset.schema.json
│
├── tests/
│   ├── unit/
│   └── integration/
│
└── tools/
    ├── validate.rs            # Rule validation CLI
    ├── compile.rs             # Compile rules to binary
    └── benchmark.rs           # Performance testing
```

---

## Rule Generation Pipeline

### Trigger Sources

Rules can be proposed from multiple sources:

1. **Self-Healing Reports**: Failure patterns trigger rule proposals
2. **Unsafe Command Detection**: New dangerous patterns identified
3. **Community Feedback**: Hub UI submissions
4. **Enterprise Requirements**: Compliance mandates
5. **Security Research**: Proactive threat modeling

### Generation Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Signal Source  │────▶│  Rule Generator │────▶│  Confidence     │
│  (Trigger)      │     │  (Agent)        │     │  Assessment     │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                          │
                        ┌─────────────────────────────────┘
                        ▼
        ┌───────────────────────────────────────────────────┐
        │                                                   │
        ▼                                                   ▼
┌─────────────────┐                               ┌─────────────────┐
│ High Confidence │                               │ Low Confidence  │
│ (>70%)          │                               │ (<70%)          │
└────────┬────────┘                               └────────┬────────┘
         │                                                  │
         ▼                                                  ▼
┌─────────────────┐                               ┌─────────────────┐
│ Create PR       │                               │ Open Discussion │
│ Direct          │                               │ Community Input │
└────────┬────────┘                               └────────┬────────┘
         │                                                  │
         │                    ┌─────────────────┐           │
         └───────────────────▶│ Community Review │◀──────────┘
                              │ & Iteration      │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌─────────────────┐
                              │ Merge & Release │
                              └─────────────────┘
```

### Confidence Scoring

Rules are scored on multiple factors:

| Factor | Weight | Description |
|--------|--------|-------------|
| Pattern clarity | 25% | Is the pattern unambiguous? |
| False positive risk | 25% | How likely to block safe commands? |
| Severity of prevented harm | 20% | What's the blast radius? |
| Reproducibility | 15% | Can the trigger be reproduced? |
| Community precedent | 15% | Similar rules exist? |

---

## Community Deliberation

### Discussion-First for Low Confidence

When confidence is low, an RFC is opened:

```markdown
# RFC: Block recursive glob deletions

## Summary
Propose rule to block `rm **/*` and similar recursive glob patterns.

## Trigger
Self-Healing Report CARO-2024-042: User accidentally deleted project.

## Proposed Rule
- Pattern: `rm.*\*\*/\*` or `rm -r.*\*`
- Action: Block with explanation
- Risk: Critical

## Concerns
- May block legitimate cleanup operations
- Could conflict with find -delete patterns

## Questions
1. Should we allow with --force confirmation?
2. What's the right scope (all directories vs specific paths)?

## Discussion
<!-- Community input here -->
```

### Voting Mechanism

For contentious rules:
- Maintainers approve technical correctness
- Community votes on inclusion
- Enterprise users can opt-in to experimental rules

---

## Enterprise Layer

### Private Rule Repositories

Enterprises can maintain private dogma repositories:

```yaml
# ~/.config/caro/dogma/sources.yaml
sources:
  - name: caro-community
    url: https://github.com/caro-cli/dogma-rules
    branch: main
    priority: 100

  - name: acme-corp
    url: https://github.com/acme-corp/caro-dogma-private
    branch: production
    priority: 200
    auth:
      type: github-app
      app_id: 12345
```

### Enterprise-Only Features

1. **Private Rules**: Never exposed to community
2. **Audit Logging**: All rule matches logged to SIEM
3. **Compliance Mapping**: Rules tagged with SOC2, HIPAA, etc.
4. **Central Management**: Deploy rules across org
5. **Analytics Dashboard**: Rule effectiveness metrics

### Licensing Model

```
dogma-rules (Community)
├── License: Apache-2.0
├── Free for all use
└── Contributions welcome

dogma-enterprise (Add-on)
├── License: Commercial
├── Private repository support
├── Audit & compliance features
├── Priority support
└── Custom rule development
```

---

## CARO Integration

### Configuration

```toml
# ~/.config/caro/config.toml

[dogma]
enabled = true
sources = [
  "https://github.com/caro-cli/dogma-rules",
]
update_frequency = "daily"
offline_mode = true  # Use cached rules when offline

# Rule overrides
[dogma.overrides]
"DOGMA-0001" = { action = "warn" }  # Downgrade to warning
"DOGMA-0042" = { enabled = false }   # Disable rule
```

### CLI Flags

```
--dogma             Enable Dogma validation (default)
--no-dogma          Disable Dogma validation
--dogma-source      Add additional rule source
--dogma-level       Set rule enforcement level (strict/moderate/permissive)
--dogma-explain     Show why a command was blocked
--dogma-override    Override specific rule for this command
--dogma-update      Update rules from sources
--dogma-list        List all active rules
```

### Validation Pipeline

```rust
impl CommandValidator {
    pub async fn validate(&self, command: &str) -> ValidationResult {
        // 1. Core safety validation (existing patterns.rs)
        let safety_result = self.safety_validator.validate(command)?;

        // 2. Dogma rule validation
        let dogma_result = self.dogma_engine.validate(command)?;

        // 3. Merge results, taking highest risk level
        ValidationResult::merge(safety_result, dogma_result)
    }
}
```

---

## Module Structure

```
src/dogma/
├── mod.rs                 # Public API exports
├── engine.rs              # DogmaEngine runtime
├── rule.rs                # DogmaRule struct & parsing
├── pattern.rs             # Pattern matching implementations
├── source.rs              # Rule source management
├── cache.rs               # Local rule caching
├── update.rs              # Rule update mechanism
└── explain.rs             # Rule explanation for users

crates/dogma-cli/          # Standalone dogma CLI tool
├── src/
│   ├── main.rs
│   ├── validate.rs        # Validate rule files
│   ├── compile.rs         # Compile to binary format
│   ├── test.rs            # Test rules against commands
│   └── lint.rs            # Lint rules for best practices
└── Cargo.toml
```

---

## Performance Considerations

### Rule Compilation

Rules are compiled at update time, not runtime:

```rust
pub struct CompiledRuleset {
    /// Pre-compiled regex patterns
    pub patterns: Vec<CompiledPattern>,

    /// Lookup table for fast matching
    pub index: RuleIndex,

    /// Metadata for the ruleset
    pub version: String,
    pub compiled_at: DateTime<Utc>,
}
```

### Matching Strategy

1. **Fast path**: Simple patterns checked first
2. **Index lookup**: Patterns indexed by command prefix
3. **Lazy evaluation**: Complex patterns only checked if simple patterns pass
4. **Parallel matching**: Multiple patterns checked concurrently

### Benchmarks Target

| Operation | Target |
|-----------|--------|
| Load ruleset | <10ms |
| Single command check | <1ms |
| Full validation (100 rules) | <5ms |
| Update rules | <5s (network) |

---

## Security Model

### Rule Integrity

- All community rules are signed
- Checksums verified before loading
- Tampered rules rejected

### Source Verification

- HTTPS required for all sources
- GitHub/GitLab App authentication supported
- Certificate pinning for critical sources

### Sandboxing

- Transform actions sandboxed
- No network access during validation
- No filesystem writes during validation

---

## Success Criteria

### Key Metrics

| Metric | Target |
|--------|--------|
| Community rule contributions | >50/month |
| Enterprise adoption | 20+ organizations |
| False positive rate | <1% |
| Rule update latency | <1 hour |
| Performance overhead | <5ms per command |

### Qualitative Goals

1. **Transparency**: Users understand why commands are blocked
2. **Community Trust**: Active contribution and governance
3. **Enterprise Value**: Clear differentiation for paid tier
4. **Maintainability**: Rules evolve without CARO releases

---

## Implementation Phases

### Phase 1: Core Engine (MVP)
- DogmaRule struct and parsing
- Local rule files support
- Basic pattern matching
- CLI integration (`--dogma-*` flags)

### Phase 2: Community Repository
- Public dogma-rules repository
- Contribution guidelines
- CI validation pipeline
- Rule versioning

### Phase 3: Update Mechanism
- Automatic rule updates
- Offline caching
- Conflict resolution
- Rollback support

### Phase 4: Enterprise Layer
- Private repository support
- Audit logging
- Compliance tagging
- Central management

### Phase 5: Agent Integration
- Self-Healing → Dogma pipeline
- Automatic rule proposals
- Confidence scoring
- Community deliberation

---

## Integration with Self-Healing

Dogma and Self-Healing form a feedback loop:

```
User Failure
     │
     ▼
Self-Healing Analysis
     │
     ├── Is this a rule gap?
     │         │
     │         ▼
     │   Propose Dogma Rule
     │         │
     │         ▼
     │   Community Review
     │         │
     │         ▼
     │   Merge to dogma-rules
     │         │
     │         ▼
     │   Rule Update
     │         │
     │         ▼
     └── Future Failures Prevented
```

---

## Open Questions

1. **Versioning**: How to handle breaking rule changes?
2. **Deprecation**: How long to support old rule formats?
3. **Conflicts**: How to resolve conflicting rules?
4. **Testing**: How to test rules against real-world commands?
5. **Migration**: How to migrate existing safety patterns?

---

## References

- [CARO Constitution](.specify/memory/constitution.md)
- [Safety Validation Pipeline](../command-validation-pipeline.md)
- [Self-Healing Spec](../005-self-healing-caro/spec.md)
- [Safety Patterns](../../src/safety/patterns.rs)

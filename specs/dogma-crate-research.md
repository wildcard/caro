# Dogma Crate Research & Architecture Design

## Overview

**Dogma** is a distributed, decentralized rule engine for Karo (the agent) that provides safety validation through multiple rule sources. This document outlines the research findings and proposed architecture for implementing Dogma as a separate crate within the Karo monorepo.

## Current State Analysis

### Existing Caro Safety Module

The current safety implementation (`src/safety/`) consists of:

| File | Purpose |
|------|---------|
| `mod.rs` | SafetyValidator, SafetyConfig, ValidationResult structs |
| `patterns.rs` | 48+ hardcoded DangerPattern definitions |

**Current Architecture:**
```rust
pub struct DangerPattern {
    pub pattern: String,           // Regex pattern
    pub risk_level: RiskLevel,     // Critical/High/Moderate/Safe
    pub description: String,       // Human-readable explanation
    pub shell_specific: Option<ShellType>,
}
```

**Limitations:**
1. Patterns are hardcoded in Rust source
2. No external rule loading mechanism
3. No community contribution workflow for rules
4. No plugin/vendor system for external rule sources

---

## Shellfirm Analysis

### Repository Structure

```
shellfirm/
├── shellfirm/
│   ├── checks/              # YAML rule definitions
│   │   ├── base.yaml        # Fork bombs, crontab, shutdown
│   │   ├── fs.yaml          # Filesystem operations (rm -rf, dd, mkfs)
│   │   ├── fs-strict.yaml   # Stricter fs rules
│   │   ├── git.yaml         # Git destructive operations
│   │   ├── git-strict.yaml  # Stricter git rules
│   │   ├── kubernetes.yaml  # K8s operations
│   │   ├── kubernetes-strict.yaml
│   │   ├── terraform.yaml   # IaC operations
│   │   ├── heroku.yaml      # PaaS deployments
│   │   └── network.yaml     # Network operations
│   ├── src/
│   │   ├── checks.rs        # Rule evaluation logic
│   │   ├── config.rs        # Configuration management
│   │   └── ...
│   └── build.rs             # Embeds YAML at compile time
```

### Shellfirm Rule Format (YAML)

```yaml
- from: fs                    # Category/source
  test: "rm\\s{1,}(-[rRfF]+\\s{1,})?/"   # Regex pattern
  description: "You are going to delete the root directory"
  id: "fs:remove-root"        # Unique identifier
  filters:                    # Optional conditions
    IsExists: "1"             # Check if path exists
    NotContains: "--dry-run"  # Exclude if contains string
```

### Key Features to Adopt

1. **YAML-based rules**: Human-readable, easy to contribute
2. **Categorization**: Rules grouped by domain (fs, git, kubernetes)
3. **Severity tiers**: Base vs strict variants
4. **Compile-time embedding**: Zero-cost rule loading
5. **Filter system**: Contextual rule application (IsExists, NotContains)

### Integration Potential

Shellfirm rules can be vendored as a **plugin source**:
- Copy YAML files to `dogma/vendor/shellfirm/`
- Load as one of multiple rule providers
- Attribute rules with source: `shellfirm:fs:remove-root`

---

## Dogma Architecture Design

### Core Design Principles

1. **Multi-Faceted Sources**: Local, Remote, Community, Vendor
2. **Future-Proof**: Extensible provider system
3. **Decentralized**: Works offline with embedded rules
4. **Composable**: Mix and match rule sources
5. **Traceable**: Every rule has a source attribution

### Proposed Crate Structure

```
crates/
└── dogma/
    ├── Cargo.toml
    ├── build.rs              # Embed community rules at compile time
    ├── src/
    │   ├── lib.rs            # Public API
    │   ├── rule.rs           # Rule and RuleSet definitions
    │   ├── engine.rs         # Rule evaluation engine
    │   ├── providers/
    │   │   ├── mod.rs        # RuleProvider trait
    │   │   ├── embedded.rs   # Compile-time embedded rules
    │   │   ├── local.rs      # User-configured local rules
    │   │   ├── remote.rs     # Remote repository fetching
    │   │   └── vendor.rs     # Third-party vendors (shellfirm)
    │   ├── filters/
    │   │   ├── mod.rs        # Filter trait and registry
    │   │   ├── exists.rs     # File/path existence filter
    │   │   ├── contains.rs   # String containment filter
    │   │   └── context.rs    # Context-aware filters
    │   ├── formats/
    │   │   ├── mod.rs        # Format parsers
    │   │   ├── yaml.rs       # YAML rule format
    │   │   └── json.rs       # JSON rule format
    │   └── compat/
    │       └── shellfirm.rs  # Shellfirm format adapter
    ├── rules/                 # Community-maintained rules
    │   ├── base.yaml
    │   ├── filesystem.yaml
    │   ├── git.yaml
    │   ├── kubernetes.yaml
    │   ├── cloud/
    │   │   ├── aws.yaml
    │   │   └── gcp.yaml
    │   └── ai/
    │       └── agent-safety.yaml
    ├── vendor/                # Third-party rule sources
    │   └── shellfirm/
    │       ├── README.md
    │       ├── LICENSE
    │       └── checks/        # Vendored shellfirm rules
    │           ├── base.yaml
    │           ├── fs.yaml
    │           └── ...
    └── tests/
        ├── integration/
        └── unit/
```

### Core Types

```rust
/// A single validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier (e.g., "dogma:fs:remove-root")
    pub id: RuleId,

    /// Source provider (embedded, local, remote, vendor)
    pub source: RuleSource,

    /// Category for grouping (fs, git, kubernetes, etc.)
    pub category: String,

    /// Regex pattern to match against commands
    pub pattern: CompiledPattern,

    /// Human-readable description
    pub description: String,

    /// Risk level for this rule
    pub severity: Severity,

    /// Optional filters for contextual application
    pub filters: Vec<Filter>,

    /// Shell-specific applicability
    pub shells: Option<Vec<ShellType>>,

    /// Metadata (author, version, last updated, etc.)
    pub metadata: RuleMetadata,
}

/// Identifies where a rule came from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleSource {
    /// Built-in community rules (embedded at compile time)
    Embedded { version: String },

    /// User's local configuration
    Local { path: PathBuf },

    /// Remote repository
    Remote { url: String, fetched_at: DateTime<Utc> },

    /// Third-party vendor (e.g., shellfirm)
    Vendor { name: String, version: String },
}

/// Risk severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,  // Must block
    High,      // Block by default, allow with --force
    Moderate,  // Warn and confirm
    Low,       // Informational warning
    Info,      // Just log
}

/// Filter conditions for contextual rule application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    /// File/path must exist
    PathExists { path_capture: usize },

    /// Command must not contain string
    NotContains { substring: String },

    /// Command must contain string
    Contains { substring: String },

    /// Environment variable check
    EnvVar { name: String, value: Option<String> },

    /// Git repository state
    GitState { branch: Option<String>, dirty: Option<bool> },

    /// Custom filter (for extensibility)
    Custom { name: String, params: serde_json::Value },
}
```

### Provider Trait

```rust
/// Trait for rule providers
#[async_trait]
pub trait RuleProvider: Send + Sync {
    /// Provider identifier
    fn name(&self) -> &str;

    /// Load all rules from this provider
    async fn load_rules(&self) -> Result<Vec<Rule>, DogmaError>;

    /// Check if provider needs refresh (for remote providers)
    async fn needs_refresh(&self) -> bool;

    /// Refresh rules (for remote providers)
    async fn refresh(&mut self) -> Result<(), DogmaError>;

    /// Get provider priority (higher = checked first)
    fn priority(&self) -> u32;
}
```

### Rule Engine

```rust
/// The main Dogma rule engine
pub struct Dogma {
    providers: Vec<Box<dyn RuleProvider>>,
    rules: RwLock<RuleSet>,
    config: DogmaConfig,
    cache: RuleCache,
}

impl Dogma {
    /// Create a new Dogma instance with default providers
    pub fn new(config: DogmaConfig) -> Result<Self, DogmaError>;

    /// Add a custom rule provider
    pub fn add_provider(&mut self, provider: impl RuleProvider + 'static);

    /// Validate a command against all rules
    pub async fn validate(&self, command: &str, context: &Context) -> ValidationResult;

    /// Get all rules matching a category
    pub fn rules_by_category(&self, category: &str) -> Vec<&Rule>;

    /// Reload rules from all providers
    pub async fn reload(&self) -> Result<(), DogmaError>;
}
```

### Configuration

```yaml
# ~/.config/karo/dogma.yaml

# Enable/disable rule sources
sources:
  embedded: true          # Community rules (always available)
  local: true             # User's local rules
  remote: false           # Remote repositories (opt-in)
  vendors:
    shellfirm: true       # Enable shellfirm vendor

# Local rules directory
local_rules_dir: ~/.config/karo/rules/

# Remote repositories (when enabled)
remote_repos:
  - url: https://rules.karo.dev/v1/rules.yaml
    refresh_interval: 24h

# Rule overrides
overrides:
  # Disable specific rules
  disabled:
    - "shellfirm:fs:chmod-recursive"

  # Promote/demote severity
  severity:
    "dogma:git:force-push": moderate  # Downgrade from high

# Category toggles
categories:
  filesystem: true
  git: true
  kubernetes: false  # Disable k8s rules
  terraform: false
```

---

## Shellfirm Integration Strategy

### Option 1: Vendoring (Recommended)

Copy shellfirm's YAML check files into `dogma/vendor/shellfirm/`:

**Pros:**
- No external runtime dependency
- Version-controlled rules
- Can extend/modify rules
- Works offline

**Cons:**
- Need to manually update when shellfirm releases
- Rule duplication (some overlap with Dogma's own rules)

**Implementation:**
```rust
// dogma/src/providers/vendor.rs
pub struct ShellFirmProvider {
    rules: Vec<Rule>,
}

impl ShellFirmProvider {
    pub fn new() -> Result<Self, DogmaError> {
        // Load vendored YAML files at compile time
        let yaml = include_str!("../../vendor/shellfirm/checks/fs.yaml");
        let rules = parse_shellfirm_yaml(yaml)?;
        Ok(Self { rules })
    }
}
```

### Option 2: Git Submodule

Add shellfirm as a git submodule:

```bash
git submodule add https://github.com/kaplanelad/shellfirm.git crates/dogma/vendor/shellfirm-repo
```

**Pros:**
- Easy to update (`git submodule update`)
- Track specific versions/tags

**Cons:**
- Complicates build process
- Users need `--recursive` clone

### Option 3: Runtime Fetching

Fetch shellfirm rules at runtime from GitHub:

**Pros:**
- Always latest rules
- No vendoring overhead

**Cons:**
- Network dependency
- Slower startup
- Security concerns (MITM)

### Recommended Approach: Hybrid

1. **Vendor** shellfirm rules for offline/embedded use
2. **Optional remote** fetching for updates
3. **Periodic vendoring** updates via CI/scripts

---

## Workspace Conversion

The caro project needs to convert to a Cargo workspace to support multiple crates:

### New Structure

```toml
# /Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "crates/caro",      # Main CLI binary
    "crates/dogma",     # Rule engine
]

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "AGPL-3.0"
repository = "https://github.com/wildcard/caro"

[workspace.dependencies]
# Shared dependencies
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
thiserror = "1"
anyhow = "1"
async-trait = "0.1"
```

```toml
# /crates/caro/Cargo.toml
[package]
name = "caro"
version.workspace = true

[dependencies]
dogma = { path = "../dogma" }
# ... other deps
```

```toml
# /crates/dogma/Cargo.toml
[package]
name = "dogma"
version.workspace = true
description = "Distributed rule engine for Karo safety validation"

[dependencies]
serde = { workspace = true }
serde_yaml = { workspace = true }
regex = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
```

---

## Migration Path

### Phase 1: Workspace Setup
1. Create workspace Cargo.toml
2. Move existing code to `crates/caro/`
3. Create `crates/dogma/` skeleton
4. Verify build still works

### Phase 2: Dogma Core
1. Implement `Rule` and `RuleSet` types
2. Implement YAML parser
3. Create `EmbeddedProvider` with basic rules
4. Write unit tests

### Phase 3: Shellfirm Integration
1. Vendor shellfirm YAML files
2. Implement `ShellFirmProvider`
3. Add format adapter for shellfirm schema
4. Test rule loading

### Phase 4: Provider System
1. Implement `LocalProvider` (user config)
2. Implement `RemoteProvider` (optional)
3. Add provider priority/merging logic
4. Configuration file support

### Phase 5: Caro Integration
1. Replace `src/safety/patterns.rs` usage with Dogma
2. Maintain backward compatibility
3. Add CLI flags for Dogma configuration
4. Update documentation

---

## Rule Deduplication Strategy

When the same pattern exists in multiple sources:

1. **Priority Order**: Local > Embedded > Vendor > Remote
2. **Merge Metadata**: Combine descriptions, keep highest severity
3. **ID Namespacing**: `source:category:name` prevents collisions
4. **Override System**: Users can disable/modify specific rules

---

## Future Considerations

### Community Rule Contributions
- GitHub-based rule submissions
- Automated testing of new rules
- Versioned rule releases

### Remote Rule Repository
- Signed rule packages (GPG/Sigstore)
- Incremental sync
- Regional mirrors

### Rule Analytics (Opt-in)
- Anonymous rule match statistics
- Help prioritize rule improvements
- Identify common dangerous patterns

### AI-Aware Rules
- Rules specific to AI agent behavior
- Multi-step operation validation
- Context-aware filtering (agent vs human)

---

## Summary

| Aspect | Decision |
|--------|----------|
| **Crate location** | `crates/dogma/` |
| **Rule format** | YAML (compatible with shellfirm) |
| **Shellfirm integration** | Vendor with optional remote updates |
| **Provider system** | Trait-based, extensible |
| **Default sources** | Embedded + Local (Remote opt-in) |
| **Workspace** | Convert to Cargo workspace |

This architecture allows Dogma to:
- Work standalone or with Karo
- Support community contributions
- Integrate third-party rule sources
- Evolve with future requirements

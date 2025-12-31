# Core vs First-Party Skills

**Last Updated**: 2025-12-31

## Overview

This document defines the boundary between what lives in caro's core binary versus what ships as first-party skills. The goal is to keep core lean while providing a rich out-of-box experience.

## Principles

1. **Core = Minimal**: Only what's required for basic functionality
2. **First-Party = Curated**: Maintained by caro team, high quality
3. **Community = Open**: Anyone can create and share skills
4. **Bundled ≠ Core**: First-party skills can be bundled but remain separate

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CARO BINARY                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      CORE                                │   │
│  │  CLI │ Agent Loop │ Backends │ Safety │ Skill Runtime   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │             BUNDLED FIRST-PARTY SKILLS                  │   │
│  │  core.shell │ core.posix │ core.git                     │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
       ┌───────────┐   ┌───────────┐   ┌───────────┐
       │FIRST-PARTY│   │FIRST-PARTY│   │ COMMUNITY │
       │ cloud.aws │   │tool.docker│   │custom.corp│
       └───────────┘   └───────────┘   └───────────┘
```

## What Lives in Core

### Core Components (Compiled Into Binary)

| Component | Description | Why Core? |
|-----------|-------------|-----------|
| CLI Parser | Clap-based argument parsing | Fundamental |
| Agent Loop | Iterative command refinement | Core algorithm |
| Backend Trait | `CommandGenerator` trait system | Extension point |
| Embedded Backends | MLX, CPU inference | Core capability |
| Remote Backends | Ollama, vLLM HTTP clients | Core capability |
| Safety Validator | Pattern-based validation | Security requirement |
| Config Manager | TOML configuration | Fundamental |
| Cache Manager | Model caching | Core capability |
| Skill Runtime | Skill loading, resolution, caching | Extension infrastructure |
| Capability Enforcer | Permission model | Security requirement |
| Context Injector | Skill context integration | Core integration |

### Skill Runtime (Core Infrastructure)

The skill system **infrastructure** is core, but individual skills are not:

```rust
// These are CORE (in src/skills/)
mod manifest;     // Parse skill.toml
mod resolver;     // Resolve skill sources
mod loader;       // Load/unload skills
mod cache;        // Skill caching
mod capability;   // Permission enforcement
mod context;      // Context injection

// These are SKILLS (separate, potentially bundled)
// core.shell      <- First-party skill
// core.posix      <- First-party skill
// cloud.aws       <- First-party skill (optional)
```

---

## Bundled First-Party Skills

These skills are **maintained by the caro team** and **bundled with releases**, but remain architecturally separate from core.

### core.shell

**Purpose**: Fundamental shell operations knowledge

**Status**: Always bundled, cannot be disabled

**Contents**:
```
core.shell/
├── skill.toml
├── knowledge/
│   ├── overview.md           # Shell fundamentals
│   ├── concepts/
│   │   ├── pipes.md         # Piping and redirection
│   │   ├── variables.md     # Environment variables
│   │   ├── quoting.md       # Quote handling
│   │   └── operators.md     # Shell operators
│   ├── patterns/
│   │   ├── file-ops.md      # File operations
│   │   ├── text-processing.md
│   │   └── system-info.md
│   └── prompts/
│       └── context.md       # Always-included shell context
└── tests/
```

**Manifest**:
```toml
[skill]
id = "core.shell"
name = "Shell Fundamentals"
version = "1.0.0"
api_version = "1.0"

[provides]
knowledge = true
recipes = false
executable = false

[knowledge]
topics = ["shell", "bash", "zsh", "sh", "terminal", "cli"]
priority = 1000  # Highest priority - always loaded first
max_context_tokens = 500
```

---

### core.posix

**Purpose**: POSIX compliance knowledge and patterns

**Status**: Always bundled, enabled by default

**Contents**:
```
core.posix/
├── skill.toml
├── knowledge/
│   ├── overview.md           # POSIX fundamentals
│   ├── concepts/
│   │   ├── utilities.md     # Standard utilities
│   │   ├── portability.md   # Cross-platform considerations
│   │   └── compliance.md    # Compliance levels
│   ├── patterns/
│   │   ├── find-xargs.md    # find + xargs patterns
│   │   ├── awk-sed.md       # Text processing
│   │   ├── sort-uniq.md     # Data manipulation
│   │   └── permissions.md   # File permissions
│   └── prompts/
│       └── posix-rules.md   # POSIX compliance rules
└── tests/
```

---

### core.git

**Purpose**: Git operations and workflows

**Status**: Bundled, enabled by default

**Contents**:
```
core.git/
├── skill.toml
├── knowledge/
│   ├── overview.md
│   ├── concepts/
│   │   ├── branching.md
│   │   ├── merging.md
│   │   └── rebasing.md
│   ├── patterns/
│   │   ├── workflows.md
│   │   ├── undoing.md
│   │   └── collaboration.md
│   └── prompts/
│       └── context.md
├── recipes/
│   ├── interactive-rebase.yaml
│   ├── squash-commits.yaml
│   └── cherry-pick.yaml
└── tests/
```

---

## First-Party Skills (Separate Distribution)

These are maintained by the caro team but **not bundled** - users install them explicitly.

### Cloud Skills

| Skill ID | Description | Priority |
|----------|-------------|----------|
| `cloud.aws` | AWS CLI, SSO, IAM, EKS, Lambda | High |
| `cloud.gcp` | Google Cloud SDK, GKE, Cloud Run | High |
| `cloud.azure` | Azure CLI, AKS, App Service | Medium |
| `cloud.digitalocean` | DigitalOcean CLI, Droplets, K8s | Medium |

### Tool Skills

| Skill ID | Description | Priority |
|----------|-------------|----------|
| `tool.kubernetes` | kubectl, helm, kustomize | High |
| `tool.docker` | Docker CLI, Compose, build | High |
| `tool.terraform` | Terraform, OpenTofu, providers | High |
| `tool.ansible` | Ansible, playbooks, roles | Medium |
| `tool.nginx` | nginx configuration, tuning | Medium |
| `tool.postgres` | psql, pg_dump, tuning | Medium |

### Language Skills

| Skill ID | Description | Priority |
|----------|-------------|----------|
| `lang.rust` | cargo, rustup, clippy, testing | High |
| `lang.node` | npm, pnpm, yarn, node | High |
| `lang.python` | pip, venv, poetry, pytest | High |
| `lang.go` | go mod, build, test | Medium |

### Platform Skills

| Skill ID | Description | Priority |
|----------|-------------|----------|
| `platform.github` | gh CLI, Actions, PRs | High |
| `platform.gitlab` | glab, CI/CD, MRs | Medium |
| `platform.homebrew` | brew, cask, taps | Medium |
| `platform.systemd` | systemctl, journalctl | Medium |

---

## Distribution Matrix

| Category | Bundled | Optional Install | Source |
|----------|---------|------------------|--------|
| core.shell | Yes | No | Embedded |
| core.posix | Yes | No | Embedded |
| core.git | Yes | Optional | Embedded |
| cloud.* | No | Yes | Registry/Git |
| tool.* | No | Yes | Registry/Git |
| lang.* | No | Yes | Registry/Git |
| platform.* | No | Yes | Registry/Git |
| custom.* | No | Yes | Any |

---

## Build Configurations

### Minimal Build (Default)

```bash
cargo build --release
# Includes: core + core.shell + core.posix + core.git
# Size target: <50MB
```

### Full Bundle Build

```bash
cargo build --release --features=full-skills
# Includes: core + all first-party skills
# For: Enterprise deployments, offline usage
```

### Custom Bundle

```bash
# Build with specific skills
caro-bundler \
  --include core.shell \
  --include core.posix \
  --include cloud.aws \
  --include tool.kubernetes \
  --output caro-aws-k8s
```

---

## First-Party Skill Development

### Repository Structure

```
github.com/caro-skills/
├── cloud-aws/
├── cloud-gcp/
├── tool-kubernetes/
├── tool-docker/
├── lang-rust/
└── ...
```

### Quality Requirements

First-party skills MUST meet:

1. **Complete documentation** with examples
2. **Test coverage** >80%
3. **Security review** for any executable components
4. **Compatibility testing** across supported platforms
5. **Performance benchmarks** for context injection
6. **Regular updates** (quarterly minimum)

### Versioning

- All first-party skills follow semver
- Breaking changes require major version bump
- API version compatibility maintained

---

## Community Skills

### Quality Tiers

| Tier | Badge | Requirements |
|------|-------|--------------|
| **Verified** | ✓ | Security review, automated testing |
| **Community** | ○ | Basic validation, community maintained |
| **Experimental** | ⚠ | Minimal validation, use at own risk |

### Promotion Path

```
Community → Verified → First-Party
```

Criteria for promotion:
1. Active maintenance
2. Community adoption
3. Quality standards met
4. Security review passed

---

## Skill Categories Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    CORE (Always Present)                    │
│                                                             │
│  CLI │ Agent │ Backends │ Safety │ Skill Runtime           │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│            BUNDLED FIRST-PARTY (Always Shipped)             │
│                                                             │
│  core.shell │ core.posix │ core.git                         │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│         OPTIONAL FIRST-PARTY (Install on Demand)           │
│                                                             │
│  cloud.aws  │ cloud.gcp   │ tool.kubernetes │ tool.docker  │
│  lang.rust  │ lang.python │ platform.github │ ...          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│              COMMUNITY (Open Contribution)                  │
│                                                             │
│  custom.* │ verified community │ experimental               │
└─────────────────────────────────────────────────────────────┘
```

---

## Migration Path

### Phase 1: Extract Built-in Knowledge

Current hardcoded prompts and safety patterns become `core.shell`:

```rust
// Before: src/prompts.rs
const SYSTEM_PROMPT: &str = "You are a POSIX shell expert...";

// After: core.shell/knowledge/prompts/context.md
# Shell Command Generation Context

You are a POSIX shell expert. Generate safe, portable commands...
```

### Phase 2: Platform Patterns as core.posix

```rust
// Before: src/platform/mod.rs (hardcoded rules)
fn macos_specific_rules() -> Vec<&'static str> { ... }

// After: core.posix/knowledge/patterns/macos.md
# macOS-Specific Patterns

On macOS, prefer:
- `sed -i ''` instead of `sed -i`
- BSD `find` syntax...
```

### Phase 3: Safety Rules Integration

```rust
// Before: Compiled regex patterns
static DANGEROUS_PATTERNS: &[&str] = &[
    r"rm\s+-rf\s+/",
    ...
];

// After: Skills can ADD patterns, not replace
// Core safety is baseline, skills extend it
```

---

## Decision Points

### Open Questions

1. **Should core.git be truly optional?**
   - Pro: Some environments don't use git
   - Con: 90%+ of users need it

2. **Bundle size vs convenience tradeoff?**
   - Current target: <50MB
   - Full bundle could be 100MB+

3. **Offline-first vs download-on-demand?**
   - Enterprise: Offline required
   - Community: Download is acceptable

### Recommendations

1. Keep `core.shell`, `core.posix`, `core.git` bundled
2. All cloud/tool/lang skills are separate
3. Provide a "full" build option for enterprise
4. Support offline skill import for air-gapped

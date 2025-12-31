# Spec 007: Skills Extension System

**Status**: Draft
**Created**: 2025-12-31

## Summary

A pluggable extension system for caro that enables community contributions of domain knowledge and executable behaviors without modifying the core binary.

## Documents

| Document | Description |
|----------|-------------|
| [spec.md](./spec.md) | Full specification with architecture |
| [plan.md](./plan.md) | Implementation plan with timeline |
| [core-vs-firstparty.md](./core-vs-firstparty.md) | What lives in core vs skills |
| [contracts/cli-interface.md](./contracts/cli-interface.md) | CLI command specification |
| [contracts/manifest-schema.md](./contracts/manifest-schema.md) | skill.toml schema |

## Related ADR

See [ADR-004: Pluggable Skills / Extensions System](../../docs/adr/ADR-004-skills-extension-system.md)

## Quick Overview

### Skill Types

| Type | Contents | Execution |
|------|----------|-----------|
| **Knowledge** | Markdown docs, prompt templates | Context injection |
| **Recipe** | Declarative workflows | Interpreted by caro |
| **Executable** | WASM modules | Sandboxed runtime |

### Example Skill Structure

```
cloud.aws/
├── skill.toml              # Manifest
├── knowledge/
│   ├── overview.md
│   └── prompts/
│       └── context.md
├── recipes/
│   └── deploy-app.yaml
└── tests/
```

### Example Manifest

```toml
[skill]
id = "cloud.aws"
name = "AWS Cloud Skill"
version = "0.3.0"
api_version = "1.0"

[provides]
knowledge = true
recipes = true
executable = false

[knowledge]
topics = ["aws", "s3", "ec2", "lambda", "eks"]
priority = 100

[capabilities]
terminal_exec = { allowed = true, commands = ["aws", "eksctl"] }
filesystem_read = ["~/.aws"]
network = ["*.amazonaws.com"]
```

### CLI Commands (Proposed)

```bash
# Install and manage
caro skill list
caro skill add cloud.aws
caro skill add --git https://github.com/caro-skills/cloud-aws
caro skill remove cloud.aws

# Development
caro skill init my-skill
caro skill validate
caro skill test
caro skill pack
```

## Implementation Phases

1. **MVP** (4 weeks): Manifest + local install + context injection
2. **Distribution** (3 weeks): Git/URL sources + lockfile + capabilities
3. **Recipes** (4 weeks): Declarative workflows + execution
4. **WASM** (4 weeks): Executable modules + registry

## Key Decisions

### Why WASM for Executables?

- Cross-platform without compilation
- Sandboxed by default
- Stable ABI across caro versions
- Any language can target WASM

### Why TOML for Manifests?

- Rust ecosystem standard
- Human-readable
- Good tooling support
- Type-safe parsing

### Why Not Native Plugins?

- Rust has no stable ABI
- Security risks with full system access
- Platform-specific builds needed

## Open Questions

1. Should `core.git` be truly optional or always bundled?
2. How to handle skill dependencies that conflict?
3. What's the minimum viable registry (GitHub releases? Simple HTTPS?)

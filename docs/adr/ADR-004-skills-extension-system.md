# ADR-004: Pluggable Skills / Extensions System

**Status**: Proposed

**Date**: 2025-12-31

**Authors**: caro core team

**Target**: Community (with Enterprise extension points)

## Context

Caro needs an extension mechanism so contributors can add **skills** (domain/tool ecosystem knowledge + optional executable behaviors) without bloating core. Skills represent more than simple CLI wrappers—they encode domain expertise for generating contextually-appropriate commands.

### Examples of Desired Skills

- **Cloud ecosystems**: AWS, GCP, DigitalOcean (each as a separate skill)
- **Tool ecosystems**: Kubernetes ops, Terraform workflows, Postgres tuning, GitHub Actions
- **Terminal tool awareness**: Not merely "AWS CLI exists", but "how the whole AWS-from-terminal workflow works"
- **Language toolchains**: Rust cargo workflows, Node.js npm/pnpm patterns, Python venv management

### Current Situation

Today's caro has:
- Static model catalog with hardcoded model entries
- Backend trait system for inference providers (good pattern to extend)
- Safety validation patterns compiled into binary
- Platform context detection at runtime
- No mechanism for community-contributed domain knowledge

Contributors who want to add AWS expertise today must:
1. Fork the repo
2. Modify system prompts
3. Rebuild the binary
4. Lose upstream updates

This is unsustainable and doesn't scale.

### Key Constraints

1. **Caro must remain lean** in community/core builds (<50MB binary target)
2. **Skills must be easy to develop locally**, then optionally distributed
3. **Must support runtime fetching, install-time selection, and build-time bundling**
4. **Must work in closed/offline/air-gapped environments**
5. **We don't want to re-invent a full package manager**
6. **Security-first**: Skills must be sandboxed and capability-gated

### Business Drivers

1. **Community growth**: Lower barrier for contributions
2. **Ecosystem expansion**: Enable specialized skills for niche tools
3. **Enterprise readiness**: Support air-gapped deployments with curated skill bundles
4. **Maintainability**: Keep core focused on fundamentals

## Decision

Adopt a **plugin/skill system** based on:

1. **A Skill Manifest** (declarative metadata + capabilities + dependencies)
2. **A Skill Runtime Contract** (stable API boundary) that supports:
   - Knowledge assets (docs/runbooks/prompt packs)
   - Action definitions (recipes/commands/workflows)
   - Optional executable modules (sandboxed)
3. **Multiple distribution sources** (local path, git, tarball/zip, OCI artifact), unified behind a simple "resolver" interface

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    CARO CORE                            │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │   CLI App   │  │ Agent Loop   │  │ Safety Engine │  │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                │                  │          │
│  ┌──────▼────────────────▼──────────────────▼───────┐  │
│  │              SKILL RUNTIME                       │  │
│  │  ┌─────────────────────────────────────────────┐ │  │
│  │  │ Skill Loader │ Resolver │ Cache │ Verifier  │ │  │
│  │  └─────────────────────────────────────────────┘ │  │
│  │  ┌─────────────────────────────────────────────┐ │  │
│  │  │ Capability Enforcer │ Context Injector     │ │  │
│  │  └─────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌─────▼─────┐     ┌─────▼─────┐
    │ BUNDLED │      │  RUNTIME  │     │  INSTALL  │
    │ SKILLS  │      │  FETCHED  │     │   TIME    │
    │(built-in│      │  SKILLS   │     │  SKILLS   │
    └─────────┘      └───────────┘     └───────────┘
```

### Skill Model: Three Layers

A "skill" can include any combination of:

#### Layer 1: Knowledge Pack (lowest friction)
- Docs, runbooks, prompt templates, examples, "ecosystem map"
- Used to enrich Caro's context and reasoning
- **No code execution needed**

#### Layer 2: Recipes / Workflows (semi-structured)
- Declarative commands with guardrails
- "preconditions", "parameters", "expected outputs", "verification steps"
- Powers interactive flows ("To do X on AWS, we'll run these steps...")

#### Layer 3: Executable Module (optional, sandboxed)
- For richer behaviors: dynamic discovery, output parsing, specialized planners
- **Loaded as WASM components** (sandboxable + cross-platform)
- Only gets the capabilities it explicitly requests

## Rationale

### Why Three Layers?

1. **Layer 1 (Knowledge)**: Lowest friction, anyone can contribute markdown
2. **Layer 2 (Recipes)**: Structured but declarative, no compilation needed
3. **Layer 3 (WASM)**: Full power when needed, sandboxed for safety

### Why WASM for Executable Modules?

1. **Stable ABI**: No breaking changes when caro updates
2. **Cross-platform**: Same module runs on Linux, macOS, Windows
3. **Sandboxed**: Can't access system without explicit capability grants
4. **Language-agnostic**: Contributors can use Rust, Go, AssemblyScript, etc.
5. **Small footprint**: Typical modules are 100KB-1MB

### Why Not Native Plugins (dylib)?

1. **ABI instability**: Rust doesn't have stable ABI
2. **Security risk**: Full system access by default
3. **Platform-specific**: Need separate builds per OS/arch
4. **Version lock**: Often requires exact compiler version match

### Why Not Lua/JS Embedded Scripting?

1. **Performance**: Additional runtime overhead
2. **Dependency**: Would add significant binary size
3. **Capability model**: Harder to sandbox effectively
4. **Ecosystem**: WASM has better tooling and adoption

## Consequences

### Benefits

1. **Community contributions scale**: Anyone can create knowledge packs
2. **Clean separation**: Skills don't bloat core binary
3. **Offline support**: Skills cached locally, work air-gapped
4. **Enterprise ready**: Can bundle curated skill sets
5. **Security**: Capability model prevents malicious skills
6. **Flexibility**: Install-time, runtime, or build-time inclusion

### Trade-offs

1. **Complexity**: New subsystem to maintain
2. **Learning curve**: Contributors need to understand manifest format
3. **Testing burden**: Skills need their own test infrastructure
4. **Version management**: Skills may depend on specific caro versions

### Risks

1. **Skill quality variance**: Community skills may be low quality
   - **Mitigation**: Curated "official" skill registry, quality badges

2. **Security vulnerabilities**: Malicious skills
   - **Mitigation**: Signature verification, capability enforcement, audit logging

3. **Ecosystem fragmentation**: Too many incompatible skills
   - **Mitigation**: Stable API versioning, compatibility shims

4. **Performance overhead**: Skill loading adds latency
   - **Mitigation**: Lazy loading, skill caching, preloading common skills

## Alternatives Considered

### Alternative 1: Static Configuration Files
- **Description**: Let users add prompt snippets via config files
- **Pros**: Simple, no new infrastructure
- **Cons**: Can't handle recipes or executable behaviors, doesn't scale
- **Why not chosen**: Too limited for ecosystem ambitions

### Alternative 2: Full Plugin System (Native)
- **Description**: Load .so/.dylib plugins at runtime
- **Pros**: Maximum flexibility and performance
- **Cons**: ABI instability, security risks, platform-specific
- **Why not chosen**: Security and portability concerns

### Alternative 3: Package Manager Integration (crates.io)
- **Description**: Distribute skills as Rust crates
- **Pros**: Existing infrastructure, familiar to Rust users
- **Cons**: Requires Rust toolchain, compile-from-source, no air-gap support
- **Why not chosen**: Too heavy, excludes non-Rust contributors

### Alternative 4: Container-Based Skills
- **Description**: Run skills as Docker containers
- **Pros**: Strong isolation, any language
- **Cons**: Heavy runtime, complex orchestration, startup latency
- **Why not chosen**: Overkill for most skills, poor UX

## Implementation Notes

See `specs/007-skills-extension-system/` for detailed implementation specification.

### MVP Scope (v1)

- Manifest + knowledge packs + recipes
- Local path + git source
- Enable/disable + lockfile
- Capability declarations (enforced for terminal/network/filesystem)
- No WASM yet (or experimental flag)

### Phase 2 Scope

- WASM execution for richer skills
- OCI distribution
- Signature verification
- Official skill registry

### Key Components to Build

1. **Skill Manifest Parser**: TOML parsing + validation
2. **Skill Resolver**: Unified interface for multiple sources
3. **Skill Cache**: Local storage with integrity verification
4. **Skill Loader**: Activate/deactivate skills at runtime
5. **Capability Enforcer**: Gate skill permissions
6. **Context Injector**: Inject skill knowledge into prompts

### Directory Structure

```
~/.caro/
├── config.toml           # User configuration
├── skills/               # Installed skills
│   ├── cloud.aws/
│   ├── tool.kubernetes/
│   └── lang.rust/
├── skills.lock           # Locked versions + hashes
└── cache/
    └── models/           # Model cache (existing)

src/
├── skills/               # NEW: Skills subsystem
│   ├── mod.rs           # Public API
│   ├── manifest.rs      # Manifest parsing
│   ├── resolver.rs      # Source resolution
│   ├── loader.rs        # Skill loading
│   ├── cache.rs         # Skill caching
│   ├── capability.rs    # Permission model
│   └── context.rs       # Context injection
└── ...
```

## Success Metrics

### Adoption
- **10+ community skills** within 6 months of launch
- **3+ first-party skills** (AWS, Kubernetes, Docker) at launch
- **50%+ active users** have at least one skill installed

### Quality
- **Zero security incidents** from skill system
- **<100ms overhead** for skill loading
- **100% offline capability** for installed skills

### Developer Experience
- **<30 minutes** to create first knowledge-only skill
- **<2 hours** to create first recipe-based skill
- **Clear documentation** with examples for all skill types

## Business Implications

### Community Value
- **Lower contribution barrier**: No Rust knowledge needed for knowledge packs
- **Ecosystem growth**: Specialists can contribute domain expertise
- **User retention**: More capabilities = more value

### Enterprise Value
- **Curated bundles**: Pre-approved skill sets for organizations
- **Air-gapped support**: Offline skill deployment
- **Governance integration**: Skills can be policy-controlled

### Monetization Potential (Future)
- **Premium skills**: Advanced enterprise-focused skills
- **Skill certification**: Quality assurance program
- **Support tiers**: Priority support for skill issues

## References

- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [wasmtime Rust API](https://docs.wasmtime.dev/)
- [OCI Distribution Spec](https://github.com/opencontainers/distribution-spec)
- ADR-001: Enterprise vs Community Architecture
- ADR-002: Governance and Provisioning System

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-12-31 | caro core team | Initial draft |

# cmdai Development Roadmap

> **Vision**: A production-ready, safety-first CLI tool that converts natural language to shell commands using local LLMs with blazing-fast performance and intelligent context awareness.

## 📍 Current Status

**Current Version**: `v0.1.0` → **Target**: `v1.0.0 Stable`
**Active Branch**: `005-production-backends`
**Project Phase**: Production Polish (v0.2.0)

### Quick Stats
- ✅ **Completed Features**: 4 major features
- 🚧 **In Progress**: 1 feature (Production backends)
- 📅 **Planned**: 4 advanced features
- 🎯 **Estimated v1.0**: Q4 2025

---

## 🎯 Version Strategy

### Version Milestones
- **v0.1.x** - Foundation & Core Architecture
- **v0.2.x** - Production Polish & History Management
- **v0.3.x** - Intelligence & Semantic Features
- **v0.4.x** - User Experience & Advanced UI
- **v1.0.0** - Stable Production Release

### Release Criteria
Each version milestone must meet:
- ✅ All features fully implemented and tested
- ✅ Contract tests passing (TDD compliance)
- ✅ Documentation complete
- ✅ Performance benchmarks met
- ✅ Security validation passed

---

## 🚀 Milestone Roadmap

### v0.1.0 - Foundation ✅ COMPLETED

The foundational architecture establishing cmdai's core capabilities with TDD methodology.

#### Features Delivered

<details>
<summary><b>001 - Comprehensive Specification</b> ✅ Completed</summary>

**Description**: Complete architectural specification defining cmdai's vision, requirements, and design principles.

**Spec Location**: `specs/001-create-a-comprehensive/`

**Speckit Commands**:
```bash
# Create specification
/specify "Create a comprehensive specification for cmdai, a Rust CLI tool that converts natural language descriptions into safe POSIX shell commands using local LLMs"

# Generate implementation plan
/plan
```

**Key Deliverables**:
- Functional requirements (FR-001 to FR-020)
- Safety-first design principles
- Multi-backend architecture definition
- Performance targets (<100ms startup, <2s inference)
- Cross-platform compatibility requirements

**Status**: Foundation complete, serves as project constitution
</details>

<details>
<summary><b>002 - TDD GREEN Phase: Core Models & Safety</b> ✅ Completed</summary>

**Description**: Core data models and safety validation system implemented using Test-Driven Development.

**Spec Location**: `specs/002-implement-tdd-green/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement TDD GREEN phase with core data models, safety validation system, backend trait implementations, and comprehensive error handling"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Core data models (CommandRequest, GeneratedCommand, RiskLevel)
- Safety validation engine with pattern matching
- Backend trait system (CommandGenerator)
- Comprehensive error types
- 80+ contract tests (all passing)

**Status**: All tests green, TDD discipline maintained
</details>

<details>
<summary><b>003 - Core Infrastructure Modules</b> ✅ Completed</summary>

**Description**: Essential infrastructure for caching, configuration, execution context, and structured logging.

**Spec Location**: `specs/003-implement-core-infrastructure/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement core infrastructure modules: Hugging Face model caching with offline support, configuration management with CLI integration, execution context with environment capture, and structured logging with tracing"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Hugging Face model caching system
- TOML-based configuration management
- Execution context capture (shell, platform, env vars)
- Structured logging with tracing integration
- Cross-platform directory management

**Status**: All infrastructure modules operational
</details>

<details>
<summary><b>004 - Remote Backend Support</b> ✅ Completed</summary>

**Description**: Multi-backend support with embedded models (MLX/CPU), Ollama, and vLLM integration.

**Spec Location**: `specs/004-implement-ollama-and/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement Ollama and vLLM backends with embedded model fallback for batteries-included offline operation"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Embedded Qwen model (MLX GPU + CPU variants)
- Ollama HTTP API integration
- vLLM HTTP API integration
- Automatic backend fallback system
- Interactive configuration wizard
- Zero-config first-run experience

**Status**: All backends operational, auto-fallback working
</details>

**Version Complete**: All v0.1.0 objectives achieved ✅

---

### v0.2.0 - Production Polish 🚧 IN PROGRESS

Production-ready enhancements for enterprise deployment with persistent storage and advanced configuration.

#### Features In Development

<details>
<summary><b>005 - Production-Ready Backend System</b> 🚧 In Progress</summary>

**Description**: Comprehensive production backend system integrating command history, interactive configuration, and advanced safety validation.

**Spec Location**: `specs/005-production-backends/`

**Speckit Commands**:
```bash
# Create specification
/specify "Complete Phase 2 production polish for cmdai with SQLite command history storage, interactive configuration UI, advanced safety validation, and streaming command generation"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Unified command generation pipeline
- Interactive full-screen configuration UI
- Advanced behavioral safety analysis
- Streaming generation with cancellation
- Intelligent backend selection engine
- Production-grade observability

**Status**: Active development on branch `005-production-backends`
</details>

<details>
<summary><b>005-sqlite - SQLite Command History</b> 📅 Planned</summary>

**Description**: Persistent command history with SQLite storage enabling search, replay, and performance analysis.

**Spec Location**: `specs/005-sqlite-command-history/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement persistent command history with SQLite storage for cmdai, enabling users to search, filter, and replay previous command generations with metadata tracking"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- SQLite database schema for command history
- Fast text-based search (<50ms for 10K entries)
- Command replay with context regeneration
- Privacy-preserving storage (sensitive data filtering)
- Automatic retention policies
- Usage analytics and performance metrics
- Export functionality (JSON/CSV)

**Target**: v0.2.0
</details>

**Milestone Goal**: Production-ready system with persistent storage and enterprise-grade configuration management.

---

### v0.3.0 - Intelligence 📅 PLANNED

Semantic understanding and contextual awareness for intelligent command suggestions.

#### Planned Features

<details>
<summary><b>006 - Semantic Command Search</b> 📅 Planned</summary>

**Description**: AI-powered semantic search using local embeddings to find commands by intent, not just keywords.

**Spec Location**: `specs/006-semantic-command-search/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement semantic command search and understanding using local embeddings to find commands by intent rather than just text matching"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Local embedding generation (offline capable)
- Vector similarity search (200ms for 10K entries)
- Intent-based query understanding
- Synonym and concept matching
- Semantic similarity scoring
- Hybrid search (semantic + text fallback)
- Automatic embedding updates

**Dependencies**: Feature 005-sqlite (command history)
**Target**: v0.3.0
</details>

<details>
<summary><b>008 - Contextual AI Interaction</b> 📅 Planned</summary>

**Description**: Butterfish-inspired contextual awareness that learns from user patterns and project context.

**Spec Location**: `specs/008-contextual-ai-interaction/`

**Speckit Commands**:
```bash
# Create specification
/specify "Implement contextual AI interaction inspired by Butterfish that provides intelligent command suggestions based on current directory, recent commands, and user patterns"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Project type detection (Git, Node.js, Python, Rust)
- Working directory context analysis
- Recent command pattern learning
- Shell environment adaptation
- User preference profiling
- Context-aware command explanations
- Smart command variants based on context

**Dependencies**: Feature 005-sqlite (command history)
**Target**: v0.3.0
</details>

**Milestone Goal**: Transform cmdai into an intelligent assistant that understands user intent and context.

---

### v0.4.0 - User Experience 📅 PLANNED

Advanced user interface and interaction patterns for seamless workflow integration.

#### Planned Features

<details>
<summary><b>007 - Full-Screen TUI Interface</b> 📅 Planned</summary>

**Description**: Atuin-inspired interactive full-screen interface for immersive command history browsing.

**Spec Location**: `specs/007-fullscreen-search-interface/`

**Speckit Commands**:
```bash
# Create specification
/specify "Create full-screen search interface inspired by Atuin for interactive command history browsing with real-time filtering and preview"

# Generate implementation plan
/plan

# Execute tasks
/tasks
/implement
```

**Key Deliverables**:
- Full-screen terminal UI (crossterm/ratatui)
- Real-time search filtering (<50ms response)
- Keyboard navigation (arrows, vim keys)
- Command preview panel with metadata
- Search term highlighting
- Multiple selection modes (copy/execute/edit)
- Command management (delete/favorite/export)
- Terminal resize handling

**Dependencies**: Features 005-sqlite, 006 (semantic search)
**Target**: v0.4.0
</details>

**Milestone Goal**: Professional-grade user experience matching or exceeding specialized CLI tools.

---

### v1.0.0 - Stable Release 🎯 TARGET

Production-hardened, security-audited stable release ready for widespread deployment.

#### Critical Completion Criteria

<details>
<summary><b>🎯 v1.0 Requirements</b></summary>

**Performance Benchmarks** (Constitutional Requirements)
- ✅ Startup time: <100ms (measured on M1 Mac)
- ✅ First inference: <2s on Apple Silicon
- ✅ Safety validation: <50ms for 95% of commands
- 📅 History search: <100ms for 10K entries
- 📅 Semantic search: <200ms for 10K entries
- 📅 TUI responsiveness: <50ms for all interactions

**Quality Standards**
- 📅 Code coverage: >80% for all modules
- 📅 Zero critical security vulnerabilities
- 📅 All contract tests passing (200+ tests)
- 📅 Cross-platform compatibility verified (macOS, Linux, Windows)
- 📅 Binary size: <50MB (excluding model weights)
- 📅 Memory usage: <500MB during operation

**Documentation**
- 📅 Complete user guide and tutorials
- 📅 API documentation for all public interfaces
- 📅 Security best practices guide
- 📅 Deployment and configuration guide
- 📅 Troubleshooting and FAQ
- 📅 Contributing guidelines

**Security & Compliance**
- 📅 Independent security audit completed
- 📅 Dependency vulnerability scan (zero high/critical)
- 📅 SAST/DAST testing passed
- 📅 Privacy policy and data handling documentation
- 📅 License compliance verification

**Production Readiness**
- 📅 Production deployment guide
- 📅 Monitoring and observability setup
- 📅 Performance tuning recommendations
- 📅 Backup and recovery procedures
- 📅 Incident response playbook
</details>

**Estimated Release**: Q4 2025

---

## 📊 Feature Summary

### Completion Status

| Feature | ID | Status | Version | Priority |
|---------|-----|--------|---------|----------|
| Comprehensive Specification | 001 | ✅ Complete | v0.1.0 | Critical |
| TDD GREEN Phase | 002 | ✅ Complete | v0.1.0 | Critical |
| Core Infrastructure | 003 | ✅ Complete | v0.1.0 | Critical |
| Remote Backend Support | 004 | ✅ Complete | v0.1.0 | Critical |
| Production Backend System | 005 | 🚧 In Progress | v0.2.0 | Critical |
| SQLite Command History | 005-sqlite | 📅 Planned | v0.2.0 | Critical |
| Semantic Command Search | 006 | 📅 Planned | v0.3.0 | High |
| Full-Screen TUI Interface | 007 | 📅 Planned | v0.4.0 | High |
| Contextual AI Interaction | 008 | 📅 Planned | v0.3.0 | High |

### Dependency Graph

```
001 (Spec) ──┬──> 002 (TDD) ──┬──> 004 (Backends)
             │                 │
             └──> 003 (Infra) ─┴──> 005 (Production) ──┬──> 005-sqlite (History)
                                                         │
                                                         ├──> 006 (Semantic) ─┐
                                                         │                    │
                                                         └────────────────────┴──> 007 (TUI)
                                                         │
                                                         └──> 008 (Contextual)
```

---

## 🛤️ Development Timeline

```
2025
│
├─ Q3: v0.1.0 Foundation ✅
│  └─ Features 001-004 completed
│
├─ Q3-Q4: v0.2.0 Production Polish 🚧
│  ├─ Feature 005 (In Progress)
│  └─ Feature 005-sqlite (Planned)
│
├─ Q4: v0.3.0 Intelligence 📅
│  ├─ Feature 006 (Semantic Search)
│  └─ Feature 008 (Contextual AI)
│
├─ Q4: v0.4.0 User Experience 📅
│  └─ Feature 007 (Full-Screen TUI)
│
└─ Q4: v1.0.0 Stable Release 🎯
   ├─ Security Audit
   ├─ Performance Optimization
   ├─ Production Hardening
   └─ Documentation Complete
```

---

## 🔑 Critical Path to v1.0

### Must-Have Features (Blocking v1.0)
1. ✅ Core CLI with safety validation
2. ✅ Multi-backend support (embedded + remote)
3. ✅ Configuration management
4. 🚧 Command history with SQLite
5. 📅 Semantic search capabilities
6. 📅 Production performance optimization
7. 📅 Security audit and hardening
8. 📅 Comprehensive documentation

### Nice-to-Have Features (Post v1.0)
- Advanced shell script generation
- Multi-step goal completion
- Custom model fine-tuning
- Cloud backend integration
- Team collaboration features
- Plugin/extension system

---

## 📝 Development Workflow

### Using Speckit for Feature Development

Each feature follows the spec-driven development workflow:

```bash
# 1. Create Feature Specification
/specify "<feature description>"
# → Generates specs/XXX-feature-name/spec.md

# 2. Clarify Requirements (if needed)
/clarify
# → Identifies underspecified areas and records answers

# 3. Generate Implementation Plan
/plan
# → Creates specs/XXX-feature-name/plan.md with architecture

# 4. Generate Task List
/tasks
# → Creates specs/XXX-feature-name/tasks.md with ordered tasks

# 5. Analyze for Consistency
/analyze
# → Validates cross-artifact consistency

# 6. Execute Implementation
/implement
# → Processes tasks.md and executes implementation
```

### Branch Strategy
- `main` - Stable releases only
- `XXX-feature-name` - Feature development branches
- Merge to main only after feature complete and tested

### Quality Gates
- ✅ All contract tests passing
- ✅ Code coverage >80%
- ✅ Zero clippy warnings
- ✅ Documentation complete
- ✅ Performance benchmarks met

---

## 🤝 Contributing

This roadmap serves as the development plan for cmdai. Contributors should:

1. **Check current milestone** - Focus on features in active milestone
2. **Review spec files** - Understand requirements before coding
3. **Follow TDD workflow** - Write tests first, then implementation
4. **Use speckit commands** - Maintain spec-driven development discipline
5. **Update roadmap** - Keep status indicators current

For detailed contribution guidelines, see [CONTRIBUTING.md](../CONTRIBUTING.md).

---

## 📚 References

- **Project Constitution**: [CLAUDE.md](../CLAUDE.md)
- **Architecture Design**: [docs/ENHANCED_ARCHITECTURE.md](./ENHANCED_ARCHITECTURE.md)
- **Speckit Guide**: [docs/SPECKIT-CHEATSHEET.md](./SPECKIT-CHEATSHEET.md)
- **Test Cases**: [docs/qa-test-cases.md](./qa-test-cases.md)
- **Specifications**: [specs/](../specs/)

---

**Last Updated**: 2025-10-14
**Next Review**: v0.2.0 completion

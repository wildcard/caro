<!--
Sync Impact Report: Constitution Update v1.0.0 → v1.0.0 (Initial Ratification)

VERSION CHANGE: TEMPLATE → 1.0.0 (MAJOR: Initial ratification with concrete principles)

MODIFIED PRINCIPLES:
- All placeholder tokens replaced with concrete caro project values
- 5 core principles defined from TDD GREEN phase implementation plan

ADDED SECTIONS:
- Complete Core Principles section with 5 principles
- Safety-First Development section (caro-specific)
- Development Workflow section with TDD enforcement
- Complete Governance section with versioning policy

REMOVED SECTIONS:
- Template placeholders and example comments
- Generic [SECTION_2/3] markers

TEMPLATES REQUIRING UPDATES:
✅ .specify/templates/plan-template.md - Constitution Check section validated
✅ .specify/templates/spec-template.md - No constitutional constraints on specs
✅ .specify/templates/tasks-template.md - TDD ordering rules aligned
✅ CLAUDE.md - Already contains TDD GREEN phase guidance (no updates needed)

FOLLOW-UP TODOS: None - all placeholders resolved with concrete values

RATIFICATION CONTEXT:
- Project started: 2025-09-12
- First constitution ratification: 2025-10-02
- Derived from: Feature 002 TDD GREEN Phase plan.md Constitution Check section
- Enforced via: Specification-driven development workflow with /plan, /tasks, /implement commands
-->

# caro Constitution

## Core Principles

### I. Simplicity
**Maintain minimal complexity and direct implementations without unnecessary abstractions.**

- Single project structure with library-first architecture via `lib.rs` exports
- Use frameworks directly (clap, tokio, serde) without wrapper abstractions
- Single data model flow: `CommandRequest → GeneratedCommand → ValidationResult`
- No organizational-only libraries, repositories, DTOs, or UoW patterns
- Start simple, apply YAGNI principles, justify all added complexity

**Rationale**: Reduces cognitive load, improves maintainability, accelerates development velocity. Every abstraction layer adds cost—pay only when value justifies it.

### II. Library-First Architecture
**Every feature must be implemented as a standalone, testable library.**

- All modules exported via `src/lib.rs` for independent testing and reuse
- Libraries must be self-contained with clear, documented public APIs
- Each library serves a single, well-defined purpose:
  - `caro::models` - Core data types and enums
  - `caro::safety` - Command validation and risk assessment
  - `caro::backends` - Command generation trait and implementations
  - `caro::cli` - User interface and argument parsing
- Binary (`main.rs`) orchestrates libraries but contains no business logic
- Libraries support both CLI usage and programmatic integration

**Rationale**: Enables modular testing, promotes reusability, enforces separation of concerns, facilitates future integrations beyond CLI usage.

### III. Test-First (NON-NEGOTIABLE)
**TDD methodology is mandatory. Tests must exist and fail before any implementation.**

- **RED-GREEN-REFACTOR cycle strictly enforced**: Write test → Verify failure → Implement → Verify pass → Refactor
- **No implementation without failing test first** - violations block code review
- **Strict ordering**: Contract tests → Integration tests → Implementation → Unit tests
- **Real dependencies used in tests** - no mocking unless testing error conditions
- **Git commits must show tests before implementation** - commit granularity enforced
- **Integration tests required for**: new libraries, contract changes, inter-module communication, shared schemas
- **Test types by priority**: Contract (API boundaries) → Integration (workflows) → E2E (user scenarios) → Unit (edge cases)

**Enforcement**: Pre-commit hooks verify test-first discipline, PR reviews validate TDD workflow, failing to follow TDD results in rejected changes.

**Rationale**: Guarantees code quality, prevents regressions, ensures requirements are testable, creates living documentation, reduces debugging time.

### IV. Safety-First Development
**Security and safety validation are paramount, especially for system-level operations.**

- **Dangerous command detection mandatory** before any execution
- **POSIX compliance validation** for cross-platform reliability
- **Risk level assessment** (Safe, Moderate, High, Critical) required for all generated commands
- **User confirmation workflows** for High/Critical operations
- **No unsafe Rust code** without explicit justification and review
- **Security audit process**: Pattern validation, path injection prevention, privilege escalation detection
- **Comprehensive safety test coverage**: Property-based testing for validation logic, adversarial input testing

**Forbidden Operations** (unless explicitly overridden by user with --allow-dangerous):
- Filesystem destruction (`rm -rf /`, `mkfs`)
- Fork bombs and resource exhaustion
- Device writes without confirmation
- Unvalidated system path modifications

**Rationale**: caro generates and executes system commands—safety failures can destroy data or compromise systems. Safety is a feature, not an afterthought.

### V. Observability & Versioning
**Structured logging, error context, and semantic versioning are required.**

- **Structured logging with tracing**: All modules use `tracing` crate with appropriate levels (debug, info, warn, error)
- **Error context chains**: Use `anyhow` for binary context, `thiserror` for library typed errors
- **User-facing messages**: Clear, actionable error messages distinct from debug logs
- **Semantic versioning (MAJOR.MINOR.PATCH)**: Breaking changes increment MAJOR, features increment MINOR, fixes increment PATCH
- **Constitution versioning**: Same semantic rules apply to this constitution
- **Performance instrumentation**: Startup time, validation latency, inference duration logged at INFO level

**Rationale**: Enables debugging in production, provides upgrade guidance, ensures error messages help users resolve issues independently.

## Safety-First Development

### Command Validation Pipeline
1. **Pattern Matching**: Check against dangerous command database (regex-based)
2. **POSIX Compliance**: Validate shell syntax and quoting
3. **Path Validation**: Prevent injection, verify quote escaping
4. **Risk Assessment**: Assign Safe/Moderate/High/Critical level
5. **User Confirmation**: Require explicit approval for High/Critical

### Validation Performance Requirements
- Safety validation must complete in **<50ms** (P95)
- Patterns compiled once at startup (lazy static)
- Zero allocations in hot validation path where possible

## Development Workflow

### TDD Workflow Per Module
1. **RED**: Run `cargo test --test <test_file>` to see failures
2. **GREEN**: Add minimal code to make tests pass (no extra features)
3. **REFACTOR**: Improve code quality while keeping tests green
4. **COMMIT**: Granular commits after each test passes
5. **REPEAT**: Move to next failing test

### Implementation Order (STRICT)
1. **Models first** (`src/models/mod.rs`) - No dependencies, foundation for all modules
2. **Safety second** (`src/safety/`) - Depends only on models
3. **Backends third** (`src/backends/`) - Depends on models, independent of safety
4. **CLI last** (`src/cli/`) - Depends on all modules, orchestrates workflow

### Code Quality Gates
- **Linting**: `cargo clippy -- -D warnings` must pass (zero warnings)
- **Formatting**: `cargo fmt --check` must pass
- **Testing**: 100% of contract tests passing before merge
- **Performance**: Benchmarks must not regress (startup <100ms, validation <50ms)
- **Documentation**: All public APIs must have rustdoc comments

## Governance

### Amendment Procedure
1. **Proposal**: Document proposed change with rationale in GitHub issue
2. **Discussion**: Minimum 3 business days for feedback
3. **Approval**: Requires consensus from project maintainers
4. **Migration Plan**: Breaking changes require backward compatibility strategy or migration guide
5. **Version Bump**: Apply semantic versioning rules to constitution version
6. **Propagation**: Update all templates (plan, spec, tasks) and agent guidance files (CLAUDE.md)

### Constitutional Compliance
- **All PRs/reviews must verify constitutional compliance** against these principles
- **Complexity must be justified**: Additions of abstraction layers, patterns, or projects require Complexity Tracking table in plan.md
- **Violations are blocking**: Constitutional violations must be resolved or justified before merge
- **Runtime guidance**: Use `CLAUDE.md` for implementation-specific AI agent guidance

### Constitution Supersedes All Other Practices
When conflicts arise between this constitution and other documentation, practices, or conventions, **the constitution takes precedence**. Update conflicting documentation to align with constitutional principles.

### Version History
- **v1.0.0** (2025-10-02): Initial ratification with 5 core principles, safety-first development, TDD enforcement

---

**Version**: 1.0.0 | **Ratified**: 2025-10-02 | **Last Amended**: 2025-10-02

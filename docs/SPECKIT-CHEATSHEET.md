# Spec-Driven Development Cheatsheet

**Version**: 1.0.0 | **Constitution**: v1.0.0 | **Project**: cmdai

This cheatsheet provides a comprehensive reference for following spec-driven development methodology in the cmdai project using the speckit framework.

---

## 📋 Table of Contents

1. [Quick Reference](#quick-reference)
2. [Complete Development Workflow](#complete-development-workflow)
3. [Slash Commands Reference](#slash-commands-reference)
4. [Constitution Compliance](#constitution-compliance)
5. [Templates Overview](#templates-overview)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)
8. [Examples](#examples)

---

## Quick Reference

### Development Flow at a Glance

```
Idea/Feature Request
        ↓
[1] /specify "feature description"  → Creates spec.md
        ↓
[2] /clarify                        → Resolves ambiguities (optional but recommended)
        ↓
[3] /plan                           → Creates plan.md, research.md, data-model.md, contracts/, quickstart.md
        ↓
[4] /tasks                          → Creates tasks.md
        ↓
[5] /analyze                        → Validates consistency (optional but recommended)
        ↓
[6] /implement                      → Executes tasks.md
        ↓
Done! Feature implemented with TDD
```

### Essential Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `/specify "description"` | Create feature specification | Start of every new feature |
| `/clarify` | Resolve ambiguities | After /specify, before /plan |
| `/plan` | Generate implementation plan | After spec is clear |
| `/tasks` | Generate task breakdown | After plan is complete |
| `/analyze` | Validate artifact consistency | After tasks, before implementation |
| `/implement` | Execute implementation | When ready to code |
| `/constitution` | Update project constitution | When changing core principles |

### File Structure Per Feature

```
specs/[###-feature-name]/
├── spec.md              # WHAT users need (business requirements)
├── plan.md              # HOW to implement (technical design)
├── research.md          # WHY decisions were made (alternatives considered)
├── data-model.md        # Data entities and relationships
├── quickstart.md        # Integration test scenarios
├── contracts/           # API contracts and schemas
│   ├── endpoint1.yaml
│   └── endpoint2.yaml
└── tasks.md             # Step-by-step implementation checklist
```

### Constitution Quick Check

Before starting any work, verify:
- [ ] Following library-first architecture (no main.rs business logic)
- [ ] TDD cycle enforced (tests written and failing first)
- [ ] Simplicity maintained (no unnecessary abstractions)
- [ ] Safety-first approach (validation before execution)
- [ ] Observability implemented (structured logging with tracing)

---

## Complete Development Workflow

### Phase 0: Feature Initialization

**Goal**: Create initial feature specification from user description

**Command**: `/specify "your feature description"`

**What Happens**:
1. Script creates new feature branch `###-feature-name`
2. Creates `specs/###-feature-name/` directory
3. Copies spec-template.md to spec.md
4. Checks out new branch (if git available)
5. Sets `SPECIFY_FEATURE` environment variable

**Execution Flow**:
```bash
# Manual alternative (if needed):
.specify/scripts/bash/create-new-feature.sh --json "feature description"
```

**Output**:
- Branch: `005-production-backends` (example)
- Spec file: `specs/005-production-backends/spec.md`
- Feature number: `005`

**Spec.md Structure**:
```markdown
# Feature Specification: [FEATURE NAME]
├── Execution Flow (main)          # Processing steps
├── Quick Guidelines                # What to focus on
├── User Scenarios & Testing        # Primary user story, acceptance scenarios, edge cases
├── Requirements                    # Functional requirements (FR-001, FR-002, ...)
├── Key Entities                    # Data model overview
├── Review & Acceptance Checklist   # Quality gates
└── Execution Status                # Progress tracking
```

**Next Step**: Run `/clarify` to resolve ambiguities OR proceed to `/plan` if spec is crystal clear

---

### Phase 1: Clarification (Optional but Recommended)

**Goal**: Identify and resolve underspecified areas before planning

**Command**: `/clarify`

**When to Use**:
- After `/specify` and before `/plan`
- When spec contains vague requirements
- When success criteria aren't measurable
- When edge cases are unclear

**What Happens**:
1. Loads current spec.md
2. Scans for ambiguities across 10 categories:
   - Functional scope & behavior
   - Domain & data model
   - Interaction & UX flow
   - Non-functional quality attributes
   - Integration & external dependencies
   - Edge cases & failure handling
   - Constraints & tradeoffs
   - Terminology & consistency
   - Completion signals
   - Misc/placeholders
3. Asks up to 5 targeted clarification questions (one at a time)
4. Updates spec.md incrementally after each answer
5. Creates `## Clarifications` section with session tracking

**Question Format**:
- Multiple choice (2-5 options) OR
- Short answer (≤5 words)

**Example Clarification Session**:
```markdown
## Clarifications

### Session 2025-10-14
**Context**: Production backend system integration

**Clarified Requirements**:
- Command History Integration: SQLite storage seamlessly integrated
- Performance Targets: <100ms startup, <2s inference, <50ms validation
- Integration Points: All command generation auto-stores to history
```

**Validation**:
After clarification, spec should have:
- [ ] No [NEEDS CLARIFICATION] markers
- [ ] Measurable success criteria
- [ ] Testable requirements
- [ ] Clear edge cases
- [ ] Defined performance targets

**Next Step**: Run `/plan` to create implementation design

---

### Phase 2: Implementation Planning

**Goal**: Generate technical design with architecture, contracts, and test scenarios

**Command**: `/plan`

**Prerequisites**:
- ✅ spec.md exists and is complete
- ✅ Clarifications section present (recommended)
- ✅ No critical ambiguities remain

**What Happens**:
1. Loads spec.md
2. Reads constitution.md for compliance checking
3. Executes plan-template.md workflow:
   - Fills Technical Context
   - Performs Constitution Check
   - Executes Phase 0 (Research)
   - Executes Phase 1 (Design & Contracts)
   - Plans Phase 2 (Task generation approach)
4. Generates multiple artifacts

**Execution Flow**:
```
1. Load feature spec → Parse requirements
2. Fill Technical Context → Detect project type
3. Constitution Check → Document violations (if any)
4. Phase 0: Research → Generate research.md
5. Phase 1: Design → Generate data-model.md, contracts/, quickstart.md
6. Phase 2: Plan tasks → Describe task generation strategy
7. STOP → Ready for /tasks command
```

**Generated Files**:

**plan.md**:
```markdown
# Implementation Plan: [FEATURE]
├── Summary                          # High-level overview
├── Technical Context                # Stack, dependencies, performance goals
├── Constitution Check               # Compliance verification
├── Project Structure                # Documentation + source code layout
├── Phase 0: Outline & Research      # Unknowns resolution
├── Phase 1: Design & Contracts      # Data model, API contracts, tests
├── Phase 2: Task Planning Approach  # How tasks.md will be generated
├── Complexity Tracking              # Justified constitutional violations
└── Progress Tracking                # Phase completion status
```

**research.md**:
```markdown
# Research: [FEATURE]
├── Decision 1
│   ├── Rationale
│   └── Alternatives Considered
├── Decision 2
│   ├── Rationale
│   └── Alternatives Considered
└── ...
```

**data-model.md**:
```markdown
# Data Model: [FEATURE]
├── Entity 1
│   ├── Fields
│   ├── Relationships
│   └── Validation Rules
├── Entity 2
│   ├── Fields
│   ├── Relationships
│   └── Validation Rules
└── State Transitions (if applicable)
```

**contracts/** (if API endpoints):
```yaml
# contracts/endpoint-name.yaml
openapi: 3.0.0
paths:
  /api/users:
    post:
      summary: Create user
      requestBody: ...
      responses: ...
```

**quickstart.md**:
```markdown
# Quickstart: [FEATURE]

## Integration Test Scenarios

### Scenario 1: [User Story]
1. Given [initial state]
2. When [action]
3. Then [expected outcome]

### Scenario 2: [User Story]
...
```

**Technical Context Fields**:
```yaml
Language/Version: Rust 1.75
Primary Dependencies: clap, tokio, serde
Storage: SQLite, filesystem
Testing: cargo test, property tests
Target Platform: Linux, macOS, Windows
Project Type: single (CLI tool)
Performance Goals: <100ms startup, <2s inference
Constraints: <50ms validation, offline-capable
Scale/Scope: Single binary <50MB
```

**Constitution Check Gates**:
- [ ] Simplicity: No unnecessary abstractions?
- [ ] Library-First: All features in lib.rs?
- [ ] Test-First: TDD workflow planned?
- [ ] Safety-First: Validation before execution?
- [ ] Observability: Structured logging planned?

**Complexity Tracking** (only if violations exist):
```markdown
| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Repository pattern | Complex caching | Direct DB access insufficient for offline mode |
```

**Next Step**: Run `/tasks` to generate implementation breakdown

---

### Phase 3: Task Generation

**Goal**: Create ordered, dependency-aware task list for implementation

**Command**: `/tasks`

**Prerequisites**:
- ✅ plan.md exists and Phase 0-1 complete
- ✅ research.md exists (decisions documented)
- ✅ data-model.md exists (if data involved)
- ✅ contracts/ exists (if API involved)

**What Happens**:
1. Loads plan.md for tech stack and structure
2. Loads optional design documents
3. Generates tasks by category:
   - Setup tasks (project init, dependencies)
   - Test tasks [P] (contract, integration)
   - Core tasks (models, services, CLI)
   - Integration tasks (DB, middleware)
   - Polish tasks [P] (unit tests, docs)
4. Applies task ordering rules (TDD, dependencies)
5. Marks parallel-safe tasks with [P]
6. Numbers tasks sequentially (T001, T002, ...)

**Task Generation Rules**:
```
From Contracts:
  Each contract file → contract test task [P]
  Each endpoint → implementation task

From Data Model:
  Each entity → model creation task [P]
  Relationships → service layer tasks

From User Stories:
  Each story → integration test [P]
  Quickstart scenarios → validation tasks

Ordering:
  Setup → Tests → Models → Services → Endpoints → Polish
  Dependencies block parallel execution
```

**tasks.md Structure**:
```markdown
# Tasks: [FEATURE NAME]

## Phase 3.1: Setup
- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize Rust project with dependencies
- [ ] T003 [P] Configure linting and formatting

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
- [ ] T004 [P] Contract test POST /api/users in tests/contract/test_users_post.rs
- [ ] T005 [P] Contract test GET /api/users in tests/contract/test_users_get.rs
- [ ] T006 [P] Integration test user registration in tests/integration/test_registration.rs

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T007 [P] User model in src/models/user.rs
- [ ] T008 [P] UserService in src/services/user_service.rs
- [ ] T009 POST /api/users endpoint in src/api/users.rs

## Phase 3.4: Integration
- [ ] T010 Connect UserService to SQLite
- [ ] T011 Request/response logging with tracing

## Phase 3.5: Polish
- [ ] T012 [P] Unit tests for validation
- [ ] T013 Performance tests (<200ms requirement)
- [ ] T014 [P] Update documentation

## Dependencies
- Tests (T004-T006) before implementation (T007-T009)
- T007 blocks T008
- Implementation before polish (T012-T014)

## Parallel Execution Example
```bash
# Launch T004-T006 together:
cargo test --test test_users_post &
cargo test --test test_users_get &
cargo test --test test_registration &
wait
```
```

**Parallel Marker [P] Rules**:
- ✅ Different files = [P] allowed
- ❌ Same file = sequential (no [P])
- ✅ No dependencies = [P] allowed
- ❌ Has dependencies = sequential

**Task Format**: `[ID] [P?] Description with exact file path`

Example:
```
- [ ] T007 [P] User model in src/models/user.rs
      ↑     ↑   ↑                  ↑
      ID    P   Description        Exact path
```

**Next Step**: Run `/analyze` to validate consistency OR proceed to `/implement`

---

### Phase 4: Quality Analysis (Optional but Recommended)

**Goal**: Validate cross-artifact consistency before implementation

**Command**: `/analyze`

**Prerequisites**:
- ✅ spec.md, plan.md, and tasks.md all exist
- ✅ All phases in plan.md marked complete

**What Happens** (READ-ONLY):
1. Loads spec.md, plan.md, tasks.md, constitution.md
2. Builds semantic models of requirements, tasks, principles
3. Runs 6 detection passes:
   - **Duplication**: Near-duplicate requirements
   - **Ambiguity**: Vague adjectives, unresolved placeholders
   - **Underspecification**: Missing measurable outcomes
   - **Constitution**: MUST principle violations
   - **Coverage**: Requirements without tasks, tasks without requirements
   - **Inconsistency**: Terminology drift, conflicting requirements
4. Assigns severity (CRITICAL, HIGH, MEDIUM, LOW)
5. Produces detailed analysis report

**Severity Levels**:
```
CRITICAL: Violates constitution MUST, missing core spec, zero task coverage
HIGH:     Duplicate/conflicting requirement, ambiguous security/performance
MEDIUM:   Terminology drift, missing non-functional task coverage
LOW:      Style improvements, minor redundancy
```

**Analysis Report Structure**:
```markdown
### Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| D1 | Duplication | HIGH | spec.md:L120-134 | Two similar requirements | Merge; keep clearer version |
| A1 | Ambiguity | HIGH | plan.md:L45 | "Fast response" lacks metric | Specify <100ms target |
| C1 | Constitution | CRITICAL | plan.md:L78 | Violates TDD principle | Add failing tests before impl |

### Coverage Summary
| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| user-can-upload | ✅ | T007, T015 | Covered |
| secure-storage | ❌ | - | Missing task! |

### Metrics
- Total Requirements: 25
- Total Tasks: 30
- Coverage: 92% (23/25)
- Critical Issues: 1
- High Issues: 3
- Medium Issues: 5
- Low Issues: 2
```

**Next Actions**:
```
If CRITICAL issues:
  → Fix issues before /implement
  → Run /plan or /tasks to regenerate

If only LOW/MEDIUM:
  → Proceed to /implement (optional fixes)
  → Note improvements for future iterations

If zero issues:
  → Proceed to /implement with confidence
```

**Remediation Options**:
- Agent offers concrete fix suggestions
- User must explicitly approve edits
- Re-run `/analyze` after fixes to verify

**Next Step**: Run `/implement` to execute tasks

---

### Phase 5: Implementation

**Goal**: Execute task list following TDD and constitutional principles

**Command**: `/implement`

**Prerequisites**:
- ✅ tasks.md exists and is complete
- ✅ plan.md provides architecture guidance
- ✅ No CRITICAL issues from /analyze

**What Happens**:
1. Loads tasks.md and parses phases
2. Loads plan.md for architecture context
3. Loads optional design documents
4. Executes tasks phase-by-phase:
   - **Phase 3.1: Setup** → Project structure, dependencies
   - **Phase 3.2: Tests First** → Contract and integration tests (MUST FAIL)
   - **Phase 3.3: Core** → Minimal implementation to pass tests
   - **Phase 3.4: Integration** → Connect components
   - **Phase 3.5: Polish** → Refactor, optimize, document
5. Marks completed tasks in tasks.md with [x]
6. Reports progress after each task

**Implementation Rules**:
```
TDD Cycle:
  1. Write test (Phase 3.2)
  2. Verify test FAILS (RED)
  3. Write minimal code (Phase 3.3)
  4. Verify test PASSES (GREEN)
  5. Refactor (Phase 3.5)
  6. Commit

Parallel Execution:
  - Tasks marked [P] can run concurrently
  - Same-file tasks MUST run sequentially
  - Respect dependency order

Error Handling:
  - Halt on non-parallel task failure
  - Report failed parallel tasks, continue with successful ones
  - Provide debugging context
```

**Progress Tracking**:
```markdown
## Phase 3.1: Setup
- [x] T001 Create project structure per implementation plan ✅
- [x] T002 Initialize Rust project with dependencies ✅
- [x] T003 [P] Configure linting and formatting ✅

## Phase 3.2: Tests First (TDD)
- [x] T004 [P] Contract test POST /api/users ✅ FAILING (expected)
- [x] T005 [P] Contract test GET /api/users ✅ FAILING (expected)
- [ ] T006 [P] Integration test user registration

## Phase 3.3: Core Implementation
- [ ] T007 [P] User model in src/models/user.rs
...
```

**Git Workflow** (if using git):
```bash
# After each task or logical group:
git add .
git commit -m "feat(T007): Add User model

Implements User struct with validation per data-model.md

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Completion Validation**:
- [ ] All tasks marked [x]
- [ ] All tests passing (`cargo test`)
- [ ] Linting clean (`cargo clippy -- -D warnings`)
- [ ] Formatting clean (`cargo fmt --check`)
- [ ] Features match spec.md
- [ ] Performance meets plan.md requirements

**Next Step**: Feature complete! Ready for PR/merge

---

## Slash Commands Reference

### `/specify "feature description"`

**Purpose**: Create initial feature specification

**Syntax**: `/specify "natural language description of what users need"`

**Example**:
```
/specify "Add command history with SQLite storage and semantic search"
```

**Script**: `.specify/scripts/bash/create-new-feature.sh`

**Input**: Natural language feature description

**Output**:
- New git branch: `###-feature-name`
- New directory: `specs/###-feature-name/`
- New file: `specs/###-feature-name/spec.md` (from template)
- Environment variable: `SPECIFY_FEATURE=###-feature-name`

**Success Criteria**:
- [ ] Branch created and checked out
- [ ] spec.md exists with template structure
- [ ] Feature number incremented from previous

**Common Flags**:
- `--json`: Output JSON instead of human-readable text

---

### `/clarify`

**Purpose**: Resolve ambiguities in spec.md through targeted questions

**Syntax**: `/clarify [context]`

**Example**:
```
/clarify
/clarify "Focus on performance requirements"
```

**Prerequisites**:
- ✅ spec.md exists in current feature branch
- ✅ SPECIFY_FEATURE environment variable set

**Process**:
1. Scans spec.md for 10 ambiguity categories
2. Asks up to 5 clarification questions (one at a time)
3. Updates spec.md after each answer
4. Creates/updates `## Clarifications` section

**Question Categories**:
- Functional scope & behavior
- Domain & data model
- Interaction & UX flow
- Non-functional quality attributes
- Integration & external dependencies
- Edge cases & failure handling
- Constraints & tradeoffs
- Terminology & consistency
- Completion signals
- Misc/placeholders

**Output**:
- Updated spec.md with clarifications
- Session timestamp and context
- Resolved requirement ambiguities

**When to Skip**:
- Spec is crystal clear with measurable criteria
- Exploratory spike (rapid prototyping)
- Very simple features (<5 requirements)

---

### `/plan`

**Purpose**: Generate technical implementation plan with design artifacts

**Syntax**: `/plan [additional context]`

**Example**:
```
/plan
/plan "Use MLX backend for Apple Silicon optimization"
```

**Prerequisites**:
- ✅ spec.md complete
- ✅ Clarifications section present (recommended)
- ✅ No [NEEDS CLARIFICATION] markers

**Script**: `.specify/scripts/bash/setup-plan.sh`

**Input**:
- spec.md (required)
- constitution.md (for validation)
- Additional context from arguments

**Output Files**:
- `plan.md` - Implementation plan with phases
- `research.md` - Technical decisions and rationale
- `data-model.md` - Entities and relationships
- `contracts/` - API contracts (if applicable)
- `quickstart.md` - Integration test scenarios
- Agent context file (CLAUDE.md, etc.)

**Execution Phases**:
```
Phase 0: Research
  → Resolve NEEDS CLARIFICATION
  → Document technical decisions
  → Output: research.md

Phase 1: Design & Contracts
  → Define data model
  → Generate API contracts
  → Create integration test scenarios
  → Output: data-model.md, contracts/, quickstart.md

Phase 2: Task Planning (description only)
  → Describe task generation strategy
  → Not executed by /plan (run /tasks instead)
```

**Constitution Check**:
Validates against 5 core principles:
1. Simplicity (no unnecessary abstractions)
2. Library-First (no main.rs business logic)
3. Test-First (TDD cycle enforced)
4. Safety-First (validation before execution)
5. Observability (structured logging)

**Success Criteria**:
- [ ] Phase 0-1 complete in Progress Tracking
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Constitution Check passes or deviations justified
- [ ] All design artifacts generated

---

### `/tasks`

**Purpose**: Generate ordered task breakdown for implementation

**Syntax**: `/tasks [context]`

**Example**:
```
/tasks
/tasks "Focus on MLX backend integration first"
```

**Prerequisites**:
- ✅ plan.md exists (Phase 0-1 complete)
- ✅ research.md exists
- ✅ data-model.md exists (if data involved)
- ✅ contracts/ exists (if API involved)

**Script**: `.specify/scripts/bash/check-prerequisites.sh`

**Input**:
- plan.md (required)
- research.md (optional)
- data-model.md (optional)
- contracts/ (optional)
- quickstart.md (optional)

**Output**:
- `tasks.md` - Numbered, ordered task list

**Task Generation Logic**:
```
From plan.md:
  → Setup tasks (project init, dependencies, linting)

From contracts/:
  → Contract test tasks [P] (one per endpoint)
  → Endpoint implementation tasks

From data-model.md:
  → Model creation tasks [P] (one per entity)
  → Service layer tasks (relationships)

From quickstart.md:
  → Integration test tasks [P] (one per scenario)

From research.md:
  → Setup tasks (technical decisions)

Standard phases:
  → Setup → Tests [P] → Core → Integration → Polish [P]
```

**Task Numbering**: T001, T002, T003, ...

**Parallel Markers**:
- `[P]` = Can run in parallel (different files, no dependencies)
- No marker = Must run sequentially

**Success Criteria**:
- [ ] All contracts have test tasks
- [ ] All entities have model tasks
- [ ] All tests come before implementation
- [ ] Parallel tasks are truly independent
- [ ] Each task specifies exact file path

---

### `/analyze`

**Purpose**: Validate cross-artifact consistency (READ-ONLY)

**Syntax**: `/analyze [focus area]`

**Example**:
```
/analyze
/analyze "Focus on constitution compliance"
```

**Prerequisites**:
- ✅ spec.md exists
- ✅ plan.md exists (all phases complete)
- ✅ tasks.md exists

**Script**: `.specify/scripts/bash/check-prerequisites.sh --require-tasks`

**Input**:
- spec.md (requirements source)
- plan.md (design source)
- tasks.md (implementation source)
- constitution.md (compliance source)

**Output**:
- Markdown analysis report (displayed, not written to file)
- Severity-categorized issues
- Coverage statistics
- Remediation recommendations

**Detection Passes**:
1. **Duplication**: Near-duplicate requirements
2. **Ambiguity**: Vague adjectives, placeholders (TODO, ???)
3. **Underspecification**: Missing measurable outcomes
4. **Constitution**: MUST principle violations
5. **Coverage**: Requirements without tasks, tasks without requirements
6. **Inconsistency**: Terminology drift, conflicting requirements

**Report Sections**:
```markdown
### Specification Analysis Report
[Issue table with ID, Category, Severity, Location, Summary, Recommendation]

### Coverage Summary
[Requirement coverage table]

### Constitution Alignment Issues
[Principle violations]

### Metrics
- Total Requirements: N
- Total Tasks: N
- Coverage: N%
- Critical/High/Medium/Low Issues: N/N/N/N
```

**Next Actions Based on Results**:
```
CRITICAL issues:
  → MUST fix before /implement
  → Re-run /plan or /tasks
  → Re-run /analyze to verify

HIGH issues:
  → SHOULD fix before /implement
  → Consider impact on implementation

MEDIUM/LOW issues:
  → Optional improvements
  → Can proceed to /implement
  → Note for future iterations

Zero issues:
  → Proceed to /implement with confidence
```

---

### `/implement`

**Purpose**: Execute task list following TDD methodology

**Syntax**: `/implement [focus]`

**Example**:
```
/implement
/implement "Start with Phase 3.1 and 3.2 only"
```

**Prerequisites**:
- ✅ tasks.md exists and complete
- ✅ plan.md provides architecture
- ✅ No CRITICAL issues from /analyze (if run)

**Script**: `.specify/scripts/bash/check-prerequisites.sh --require-tasks --include-tasks`

**Input**:
- tasks.md (required)
- plan.md (required)
- data-model.md (optional)
- contracts/ (optional)
- research.md (optional)
- quickstart.md (optional)

**Output**:
- Implemented code in src/
- Implemented tests in tests/
- Updated tasks.md (completed tasks marked [x])

**Execution Strategy**:
```
Phase-by-phase execution:
  1. Setup (T001-T003)
  2. Tests First (T004-T007) → MUST FAIL
  3. Core Implementation (T008-T015) → Tests PASS
  4. Integration (T016-T020)
  5. Polish (T021-T025)

Parallel execution:
  - Launch all [P] tasks in same phase together
  - Wait for completion before next phase
  - Report errors but continue with successful tasks

TDD cycle per task:
  1. Write test (verify FAILS)
  2. Write minimal implementation
  3. Verify test PASSES
  4. Commit granularly
```

**Progress Tracking**:
- Real-time task completion updates
- Error reporting with context
- Git commits after each task/group

**Validation Gates**:
- [ ] Tests passing after Core phase
- [ ] Linting clean (`cargo clippy`)
- [ ] Formatting clean (`cargo fmt`)
- [ ] Performance meets requirements
- [ ] All tasks marked [x]

**Error Handling**:
- Sequential task fails → Halt execution, report error
- Parallel task fails → Report error, continue with others
- Provide debugging context and next steps

---

### `/constitution`

**Purpose**: Update project constitution with new principles

**Syntax**: `/constitution [update description]`

**Example**:
```
/constitution "Add principle for API versioning"
```

**File**: `.specify/memory/constitution.md`

**Current Version**: v1.0.0 (ratified 2025-10-02)

**Amendment Procedure**:
1. Proposal with rationale (GitHub issue)
2. Discussion (minimum 3 business days)
3. Approval (maintainer consensus)
4. Migration plan (if breaking)
5. Version bump (semantic versioning)
6. Template propagation

**Semantic Versioning**:
- **MAJOR**: Breaking changes to core principles
- **MINOR**: New principles added
- **PATCH**: Clarifications, typo fixes

**Templates to Update After Amendment**:
- `.specify/templates/spec-template.md`
- `.specify/templates/plan-template.md`
- `.specify/templates/tasks-template.md`
- `CLAUDE.md` (agent guidance)

**Constitution Authority**:
When conflicts arise, constitution takes precedence over:
- Other documentation
- Practices and conventions
- Individual preferences

---

## Constitution Compliance

### The 5 Core Principles

#### I. Simplicity

**Rule**: Maintain minimal complexity without unnecessary abstractions

**Checklist**:
- [ ] Single project structure with library-first architecture
- [ ] Use frameworks directly (clap, tokio, serde) without wrappers
- [ ] Single data model flow (no DTOs, UoW, Repository patterns)
- [ ] Start simple, apply YAGNI
- [ ] Justify all added complexity in Complexity Tracking table

**Examples**:
```rust
// ✅ GOOD: Direct framework usage
use clap::Parser;
#[derive(Parser)]
struct Cli { /* ... */ }

// ❌ BAD: Unnecessary wrapper
struct CliWrapper {
    inner: clap::Parser,
}
```

**Red Flags**:
- More than 1 project/workspace
- Wrapper abstractions around standard libraries
- DTO/ViewModel/Repository patterns without strong justification
- Layers of indirection for "future flexibility"

---

#### II. Library-First Architecture

**Rule**: Every feature implemented as standalone, testable library

**Checklist**:
- [ ] All modules exported via `src/lib.rs`
- [ ] Libraries self-contained with clear public APIs
- [ ] Single purpose per library (cmdai::models, cmdai::safety, etc.)
- [ ] main.rs contains only orchestration (no business logic)
- [ ] Libraries support both CLI and programmatic usage

**Structure**:
```
src/
├── lib.rs              # Exports all modules
├── main.rs             # CLI entry point (orchestration only)
├── models/             # cmdai::models
│   └── mod.rs
├── safety/             # cmdai::safety
│   └── mod.rs
├── backends/           # cmdai::backends
│   └── mod.rs
└── cli/                # cmdai::cli
    └── mod.rs
```

**Examples**:
```rust
// src/lib.rs
pub mod models;
pub mod safety;
pub mod backends;
pub mod cli;

// src/main.rs - ORCHESTRATION ONLY
use cmdai::{cli, backends, safety};

fn main() {
    let args = cli::parse();
    let backend = backends::select(&args);
    let result = backend.generate(&args.prompt);
    safety::validate(&result);
    println!("{}", result);
}
```

**Red Flags**:
- Business logic in main.rs
- Private modules not exported via lib.rs
- Libraries depending on CLI module
- Untestable code paths

---

#### III. Test-First (NON-NEGOTIABLE)

**Rule**: TDD methodology mandatory - tests exist and fail before implementation

**Checklist**:
- [ ] RED-GREEN-REFACTOR cycle strictly enforced
- [ ] No implementation without failing test first
- [ ] Strict ordering: Contract → Integration → Implementation → Unit
- [ ] Real dependencies in tests (no mocking unless testing errors)
- [ ] Git commits show tests before implementation
- [ ] Integration tests for: new libraries, contract changes, inter-module communication

**Test Hierarchy**:
```
1. Contract Tests (API boundaries)
   tests/contract/
   ├── test_backend_trait.rs
   ├── test_safety_api.rs
   └── test_cli_interface.rs

2. Integration Tests (workflows)
   tests/integration/
   ├── test_command_generation_flow.rs
   ├── test_safety_validation_flow.rs
   └── test_cache_integration.rs

3. E2E Tests (user scenarios)
   tests/e2e/
   └── test_full_workflow.rs

4. Unit Tests (edge cases)
   src/models/tests.rs
   src/safety/tests.rs
```

**TDD Workflow**:
```bash
# 1. RED: Write failing test
cargo test test_dangerous_command_detection
# Expected: FAILED (function not implemented)

# 2. GREEN: Minimal implementation
# Add code to src/safety/validator.rs

cargo test test_dangerous_command_detection
# Expected: PASSED

# 3. REFACTOR: Improve code quality
# Refactor while keeping tests green

cargo test
# Expected: All PASSED

# 4. COMMIT
git add tests/contract/test_safety_api.rs src/safety/validator.rs
git commit -m "feat(T015): Add dangerous command detection"
```

**Enforcement**:
- Pre-commit hooks verify test-first discipline
- PR reviews validate TDD workflow
- Failing to follow TDD = rejected changes

**Red Flags**:
- Implementation committed before tests
- Tests written after code is working
- Skipped test phase in tasks.md
- Mock-heavy tests hiding integration issues

---

#### IV. Safety-First Development

**Rule**: Security and safety validation paramount for system-level operations

**Checklist**:
- [ ] Dangerous command detection mandatory before execution
- [ ] POSIX compliance validation for cross-platform reliability
- [ ] Risk level assessment (Safe/Moderate/High/Critical)
- [ ] User confirmation workflows for High/Critical operations
- [ ] No `unsafe` Rust without explicit justification
- [ ] Security audit: pattern validation, injection prevention, privilege escalation detection
- [ ] Property-based testing for validation logic

**Validation Pipeline**:
```
1. Pattern Matching
   ├── Check against dangerous command database
   └── Regex-based detection

2. POSIX Compliance
   ├── Validate shell syntax
   └── Check quoting

3. Path Validation
   ├── Prevent injection
   └── Verify quote escaping

4. Risk Assessment
   ├── Safe: Read-only operations
   ├── Moderate: File writes in user directories
   ├── High: System modifications
   └── Critical: Data destruction, privilege escalation

5. User Confirmation
   └── Require explicit approval for High/Critical
```

**Forbidden Operations** (unless `--allow-dangerous`):
```rust
// Filesystem destruction
"rm -rf /", "rm -rf ~", "mkfs"

// Fork bombs and resource exhaustion
":(){ :|:& };:"

// Device writes without confirmation
"dd if=/dev/zero of=/dev/sda"

// Unvalidated system path modifications
"rm -rf /usr/bin", "chmod 777 /etc"
```

**Performance Requirements**:
- Validation must complete in <50ms (P95)
- Patterns compiled once at startup (lazy_static)
- Zero allocations in hot validation path

**Red Flags**:
- Commands executed without validation
- Hardcoded pattern lists (not externalized)
- Missing test coverage for adversarial inputs
- `unsafe` blocks without SAFETY comments

---

#### V. Observability & Versioning

**Rule**: Structured logging, error context, semantic versioning required

**Checklist**:
- [ ] Structured logging with `tracing` crate (debug, info, warn, error)
- [ ] Error context chains: `anyhow` for binaries, `thiserror` for libraries
- [ ] User-facing messages distinct from debug logs
- [ ] Semantic versioning (MAJOR.MINOR.PATCH)
- [ ] Constitution versioning follows same rules
- [ ] Performance instrumentation at INFO level

**Logging Levels**:
```rust
use tracing::{debug, info, warn, error};

// DEBUG: Development details
debug!("Loaded config from {:?}", path);

// INFO: User-relevant events, performance metrics
info!("Command generated in {}ms", duration);

// WARN: Recoverable issues
warn!("Cache miss for model {}, downloading", model_id);

// ERROR: Non-recoverable errors
error!("Failed to connect to backend: {}", err);
```

**Error Handling**:
```rust
// Libraries: thiserror for typed errors
#[derive(thiserror::Error, Debug)]
pub enum SafetyError {
    #[error("Dangerous pattern detected: {0}")]
    DangerousPattern(String),

    #[error("POSIX compliance failed: {0}")]
    PosixViolation(String),
}

// Binaries: anyhow for context chains
use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    let result = generate_command(&config)
        .context("Command generation failed")?;

    Ok(())
}
```

**Performance Instrumentation**:
```rust
use tracing::info;
use std::time::Instant;

let start = Instant::now();
let result = backend.infer(&prompt).await?;
let duration = start.elapsed();

info!(
    target: "performance",
    duration_ms = duration.as_millis(),
    backend = %backend.name(),
    "Inference completed"
);
```

**Semantic Versioning**:
```
MAJOR: Breaking changes (API changes, removed features)
MINOR: New features (backward compatible)
PATCH: Bug fixes (no new features)
```

**Red Flags**:
- println! or eprintln! in production code
- Generic error messages ("something went wrong")
- Missing error context
- Panic! instead of Result types
- No performance logging for critical paths

---

### Development Workflow Per Module

**Standard TDD Workflow**:
```
1. RED: Run cargo test --test <test_file> to see failures
2. GREEN: Add minimal code to make tests pass
3. REFACTOR: Improve code quality while keeping tests green
4. COMMIT: Granular commits after each test passes
5. REPEAT: Move to next failing test
```

**Implementation Order (STRICT)**:
```
1. Models first (src/models/mod.rs)
   → No dependencies
   → Foundation for all modules

2. Safety second (src/safety/)
   → Depends only on models
   → Independent of backends

3. Backends third (src/backends/)
   → Depends on models
   → Independent of safety

4. CLI last (src/cli/)
   → Depends on all modules
   → Orchestrates workflow
```

**Code Quality Gates** (pre-commit/PR):
```bash
# Zero warnings
cargo clippy -- -D warnings

# Formatting check
cargo fmt --check

# All tests passing
cargo test

# Benchmarks not regressed
cargo bench -- --baseline

# Documentation complete
cargo doc --no-deps --open
```

---

## Templates Overview

### spec-template.md

**Purpose**: Define WHAT users need and WHY

**Audience**: Business stakeholders, non-technical readers

**Mandatory Sections**:
```markdown
## Execution Flow (main)
[Processing steps for spec generation]

## User Scenarios & Testing
├── Primary User Story
├── Acceptance Scenarios (Given-When-Then)
└── Edge Cases

## Requirements
├── Functional Requirements (FR-001, FR-002, ...)
└── Key Entities (if data involved)

## Review & Acceptance Checklist
├── Content Quality
└── Requirement Completeness

## Execution Status
[Progress tracking]
```

**Key Rules**:
- ✅ Focus on WHAT and WHY
- ❌ Avoid HOW (no tech stack, APIs, code structure)
- ✅ Testable and unambiguous requirements
- ✅ Mark ambiguities with `[NEEDS CLARIFICATION: question]`

**Example Functional Requirement**:
```markdown
- **FR-001**: System MUST store command history with metadata (timestamp, prompt, generated command, execution status)
- **FR-002**: System MUST provide semantic search over command history with relevance ranking
- **FR-003**: Users MUST be able to replay historical commands with confirmation
```

**Underspecified Example** (needs clarification):
```markdown
- **FR-004**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method - email/password, SSO, OAuth?]
- **FR-005**: System MUST retain user data for [NEEDS CLARIFICATION: retention period - 30 days, 1 year, indefinite?]
```

---

### plan-template.md

**Purpose**: Define HOW to implement with technical design

**Audience**: Developers, architects

**Sections**:
```markdown
## Summary
[High-level overview extracted from spec]

## Technical Context
├── Language/Version
├── Primary Dependencies
├── Storage
├── Testing
├── Target Platform
├── Project Type
├── Performance Goals
├── Constraints
└── Scale/Scope

## Constitution Check
[Validation against 5 core principles]

## Project Structure
├── Documentation (specs/###-feature/)
└── Source Code (src/, tests/)

## Phase 0: Outline & Research
[Research tasks, unknowns resolution]

## Phase 1: Design & Contracts
[Data model, API contracts, tests, quickstart]

## Phase 2: Task Planning Approach
[Strategy for /tasks command]

## Complexity Tracking
[Justified constitutional violations]

## Progress Tracking
[Phase completion status]
```

**Technical Context Example**:
```yaml
Language/Version: Rust 1.75
Primary Dependencies: clap 4.5, tokio 1.35, serde 1.0
Storage: SQLite 3.44 (via rusqlite), filesystem cache
Testing: cargo test, proptest for property tests
Target Platform: Linux, macOS (Apple Silicon optimized), Windows
Project Type: single (CLI tool with library exports)
Performance Goals: <100ms startup, <2s first inference, <50ms validation
Constraints: Single binary <50MB, offline-capable, no external services required
Scale/Scope: Local CLI, 1k commands/day, 100MB cache
```

**Constitution Check Example**:
```markdown
## Constitution Check

### I. Simplicity ✅
- Using clap directly (no wrapper)
- Single data model: CommandRequest → GeneratedCommand → ValidationResult
- No Repository/UoW patterns

### II. Library-First ✅
- All features in src/lib.rs exports
- main.rs orchestrates only
- Libraries: cmdai::models, cmdai::safety, cmdai::backends, cmdai::cli

### III. Test-First ✅
- Contract tests in tests/contract/
- Integration tests in tests/integration/
- TDD cycle enforced in tasks.md

### IV. Safety-First ✅
- Safety validation before all execution
- Risk assessment (Safe/Moderate/High/Critical)
- User confirmation for High/Critical

### V. Observability ✅
- tracing crate for structured logging
- anyhow for binary errors, thiserror for library errors
- Performance instrumentation at INFO level
```

**Complexity Tracking Example**:
```markdown
## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Caching abstraction | Offline mode requirement | Direct HTTP calls insufficient without network |
| Backend trait system | Multiple inference engines (MLX, vLLM, Ollama) | Hardcoding single backend violates extensibility needs |
```

---

### tasks-template.md

**Purpose**: Ordered task breakdown for implementation

**Audience**: Developers executing implementation

**Sections**:
```markdown
## Execution Flow (main)
[Task generation logic]

## Format: [ID] [P?] Description
[Task numbering and parallel markers]

## Path Conventions
[Project structure assumptions]

## Phase 3.1: Setup
[Project initialization]

## Phase 3.2: Tests First (TDD)
[Contract and integration tests - MUST FAIL]

## Phase 3.3: Core Implementation
[Minimal implementation to pass tests]

## Phase 3.4: Integration
[Connect components]

## Phase 3.5: Polish
[Refactor, optimize, document]

## Dependencies
[Task ordering and blocking relationships]

## Parallel Example
[How to launch parallel tasks]

## Notes
[Execution guidance]
```

**Task Format**:
```
[ID]  [P?]  Description with exact file path
 ↑     ↑    ↑
 T001  P    Write contract test for BackendTrait in tests/contract/test_backend_trait.rs
```

**Parallel Marker Rules**:
```
[P] = Can run in parallel
  ✅ Different files
  ✅ No dependencies
  ❌ Same file
  ❌ Has dependencies
```

**Example Task Breakdown**:
```markdown
## Phase 3.1: Setup
- [ ] T001 Create src/models/mod.rs with CommandRequest, GeneratedCommand structs
- [ ] T002 Add dependencies to Cargo.toml: clap, tokio, serde, anyhow
- [ ] T003 [P] Configure clippy.toml with deny warnings

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
- [ ] T004 [P] Contract test for BackendTrait in tests/contract/test_backend_trait.rs
- [ ] T005 [P] Contract test for SafetyValidator in tests/contract/test_safety_api.rs
- [ ] T006 [P] Integration test for command generation flow in tests/integration/test_command_flow.rs

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T007 [P] Implement BackendTrait in src/backends/mod.rs
- [ ] T008 [P] Implement SafetyValidator in src/safety/validator.rs
- [ ] T009 Wire components in src/lib.rs

## Dependencies
- Tests (T004-T006) MUST complete before implementation (T007-T009)
- T007 and T008 can run in parallel (different files)
- T009 blocks on T007 and T008 (needs both)
```

---

## Best Practices

### Git Workflow Integration

**Branch Naming**:
```
Format: ###-feature-name
Example: 005-production-backends

Created by: .specify/scripts/bash/create-new-feature.sh
```

**Commit Strategy**:
```bash
# After each test passes (granular commits)
git add tests/contract/test_backend_trait.rs src/backends/mod.rs
git commit -m "feat(T007): Implement BackendTrait with async infer method

Adds trait definition with availability checking and unified config.

Refs: #123

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Commit Message Format**:
```
<type>(<task-id>): <subject>

<body>

<footer>

type: feat|fix|refactor|test|docs|chore
task-id: T001, T002, etc.
subject: Imperative mood, no period
body: Why this change, what it does
footer: Issue refs, breaking changes
```

**Pre-commit Hooks**:
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Verify tests exist before implementation
# (check git diff for new src/ files without corresponding tests/)

# Run quality gates
cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test || exit 1
```

---

### Parallel Task Execution

**Identifying Parallel Tasks**:
```
Can run in parallel if:
  ✅ Different files
  ✅ No shared dependencies
  ✅ Marked with [P] in tasks.md

Must run sequentially if:
  ❌ Same file
  ❌ One depends on another
  ❌ Not marked with [P]
```

**Execution Patterns**:

**Pattern 1: Parallel Contract Tests**
```bash
# All write to different files - run together
cargo test --test test_backend_trait &
cargo test --test test_safety_api &
cargo test --test test_cli_interface &
wait

# All should FAIL (no implementation yet)
```

**Pattern 2: Parallel Model Creation**
```bash
# Different modules - run together
cat > src/models/command.rs &
cat > src/models/safety.rs &
cat > src/models/config.rs &
wait

# Verify compilation
cargo check
```

**Pattern 3: Sequential Same-File Edits**
```bash
# All modify src/lib.rs - MUST run sequentially
echo "pub mod models;" >> src/lib.rs
echo "pub mod safety;" >> src/lib.rs
echo "pub mod backends;" >> src/lib.rs

# NOT parallel:
echo "pub mod models;" >> src/lib.rs &
echo "pub mod safety;" >> src/lib.rs &  # ❌ RACE CONDITION!
```

**Task Dependencies**:
```
Example dependency chain:

T001: Create models ───┐
                       ├──> T005: Create service (depends on T001, T002)
T002: Create traits ───┘                │
                                        ├──> T008: Integration test
T003: Write tests ─────────────────────┘

Execution order:
  1. T001, T002 in parallel [P]
  2. T003 (can overlap with T001/T002 if testing interface only)
  3. T005 (waits for T001, T002)
  4. T008 (waits for T005)
```

---

### Performance Optimization

**Startup Time (<100ms)**:
```rust
// ✅ GOOD: Lazy initialization
use once_cell::sync::Lazy;

static PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    compile_safety_patterns()
});

// ❌ BAD: Eager initialization
static PATTERNS: Vec<Regex> = compile_safety_patterns(); // Runs at startup!
```

**Validation Performance (<50ms)**:
```rust
// ✅ GOOD: Pre-compiled patterns, zero allocs
pub fn is_dangerous(cmd: &str) -> bool {
    PATTERNS.iter().any(|p| p.is_match(cmd))
}

// ❌ BAD: Runtime compilation, allocations
pub fn is_dangerous(cmd: &str) -> bool {
    let patterns = vec!["rm -rf", "mkfs", ":"]; // Allocates every call!
    patterns.iter().any(|p| cmd.contains(p))   // Linear search!
}
```

**Inference Performance (<2s first inference)**:
```rust
// ✅ GOOD: Model loaded once, reused
pub struct Backend {
    model: Arc<Model>, // Shared across requests
}

// ❌ BAD: Load model per request
pub fn infer(prompt: &str) -> Result<String> {
    let model = Model::load("path")?; // Loads every time!
    model.generate(prompt)
}
```

---

### Common Pitfalls to Avoid

**Pitfall 1: Implementation Before Tests**
```
❌ BAD:
1. Write src/safety/validator.rs
2. cargo test (all pass)
3. Realize you forgot to write tests!

✅ GOOD:
1. Write tests/contract/test_safety_api.rs
2. cargo test (FAIL - expected)
3. Write src/safety/validator.rs
4. cargo test (PASS)
```

**Pitfall 2: Vague Requirements**
```
❌ BAD:
"System should be fast and secure"

✅ GOOD:
"System MUST validate commands in <50ms (P95) and block 100% of known dangerous patterns"
```

**Pitfall 3: Missing Parallel Markers**
```
❌ BAD:
- [ ] T004 Contract test POST /api/users
- [ ] T005 Contract test GET /api/users
[Both can run in parallel but not marked!]

✅ GOOD:
- [ ] T004 [P] Contract test POST /api/users in tests/contract/test_users_post.rs
- [ ] T005 [P] Contract test GET /api/users in tests/contract/test_users_get.rs
```

**Pitfall 4: Business Logic in main.rs**
```rust
// ❌ BAD: main.rs
fn main() {
    let prompt = std::env::args().nth(1).unwrap();
    let backend = MLXBackend::new();
    let command = backend.generate(&prompt);

    // Validation logic in main! 😱
    if command.contains("rm -rf") {
        println!("Dangerous!");
        return;
    }

    println!("{}", command);
}

// ✅ GOOD: main.rs (orchestration only)
use cmdai::{cli, backends, safety};

fn main() -> anyhow::Result<()> {
    let args = cli::parse()?;
    let backend = backends::select(&args)?;
    let result = backend.generate(&args.prompt)?;

    safety::validate(&result)?; // Library handles validation

    println!("{}", result);
    Ok(())
}
```

**Pitfall 5: Mocking Everything**
```rust
// ❌ BAD: Over-mocking
#[test]
fn test_command_generation() {
    let mock_backend = MockBackend::new();
    let mock_safety = MockSafety::new();
    let mock_cache = MockCache::new();

    // Testing mocks, not real system!
}

// ✅ GOOD: Real dependencies (integration test)
#[test]
fn test_command_generation_flow() {
    let backend = VLLMBackend::new("http://localhost:8000");
    let safety = SafetyValidator::new();
    let cache = FileCache::new(temp_dir());

    let result = generate_command("list files", &backend, &safety, &cache);
    assert!(result.is_ok());
}
```

---

### Documentation Standards

**Rustdoc for Public APIs**:
```rust
/// Validates a shell command for safety and POSIX compliance.
///
/// # Arguments
/// * `command` - The shell command to validate
///
/// # Returns
/// * `Ok(RiskLevel)` - Command is safe with risk assessment
/// * `Err(SafetyError)` - Command is dangerous or invalid
///
/// # Examples
/// ```
/// use cmdai::safety::validate;
///
/// let result = validate("ls -la");
/// assert!(result.is_ok());
///
/// let result = validate("rm -rf /");
/// assert!(result.is_err());
/// ```
///
/// # Performance
/// Validation completes in <50ms (P95) using pre-compiled regex patterns.
pub fn validate(command: &str) -> Result<RiskLevel, SafetyError> {
    // Implementation
}
```

**README per Module**:
```markdown
# Safety Module

Provides command validation and risk assessment for shell commands.

## Features
- Pattern-based dangerous command detection
- POSIX compliance validation
- Risk level classification (Safe/Moderate/High/Critical)
- User confirmation workflows

## Usage
```rust
use cmdai::safety::{validate, RiskLevel};

let result = validate("ls -la")?;
match result {
    RiskLevel::Safe => println!("Safe to execute"),
    RiskLevel::High => println!("Requires confirmation"),
    _ => {}
}
```

## Performance
- <50ms validation (P95)
- Zero-allocation hot path
- Pre-compiled patterns

## Testing
- Contract tests: tests/contract/test_safety_api.rs
- Property tests: tests/property/test_safety_validation.rs
```

---

## Troubleshooting

### "No feature spec found"

**Symptom**: `/plan` or `/tasks` reports missing spec.md

**Cause**: Not in feature branch or spec.md deleted

**Solution**:
```bash
# Check current feature
echo $SPECIFY_FEATURE

# If empty, run:
git branch | grep "^[0-9]\{3\}"  # Find feature branches
git checkout 005-production-backends

# Or create new feature:
/specify "your feature description"
```

---

### "NEEDS CLARIFICATION remain"

**Symptom**: `/plan` halts with unresolved clarifications

**Cause**: spec.md contains `[NEEDS CLARIFICATION: ...]` markers

**Solution**:
```bash
# Option 1: Run clarification workflow
/clarify

# Option 2: Manually resolve in spec.md
# Replace [NEEDS CLARIFICATION: retention period] with concrete value
# "System MUST retain commands for 30 days"

# Option 3: Override (not recommended)
/plan "Proceed without clarification - exploratory spike"
```

---

### "Constitution Check FAILED"

**Symptom**: `/plan` reports constitutional violations

**Cause**: Design violates one of 5 core principles

**Solution**:
```markdown
1. Review violation in plan.md "Constitution Check" section
2. Simplify design to comply OR
3. Justify in Complexity Tracking table

Example:
| Violation | Why Needed | Simpler Alternative Rejected |
|-----------|------------|------------------------------|
| Repository pattern | Offline caching | Direct DB insufficient |

4. Re-run /plan
```

---

### "Tasks have no [P] markers"

**Symptom**: All tasks sequential, slow execution

**Cause**: Task generator couldn't detect parallelism

**Solution**:
```bash
# Manually review tasks.md
# Mark independent tasks with [P]:

- [ ] T004 [P] Contract test for backend
- [ ] T005 [P] Contract test for safety
# These write different files → can be parallel

# Save and re-run /implement
```

---

### "/implement failing early"

**Symptom**: Implementation halts after first task failure

**Cause**: Sequential task failed (expected behavior)

**Solution**:
```bash
# 1. Read error message
# 2. Fix issue
# 3. Mark fixed task as [x] in tasks.md
# 4. Re-run /implement

# For parallel task failures:
# - Other parallel tasks continue
# - Fix failed task separately
# - Re-run that specific task
```

---

### "Tests passing immediately"

**Symptom**: Phase 3.2 tests pass without implementation

**Cause**: Implementation accidentally written before tests

**Solution**:
```bash
# 1. Delete implementation code
rm src/models/command.rs

# 2. Re-run tests (should FAIL)
cargo test test_command_model
# Expected: test test_command_model ... FAILED

# 3. Now write minimal implementation
# 4. Tests should PASS
```

---

## Examples

### Example 1: Simple Feature - Add Help Command

**Step 1: Create Spec**
```
/specify "Add --help command that displays usage information and examples"
```

**Generated**:
- Branch: `006-add-help-command`
- File: `specs/006-add-help-command/spec.md`

**Step 2: Spec Content** (simplified):
```markdown
## Requirements
- **FR-001**: System MUST display usage information when user runs `cmdai --help`
- **FR-002**: Help output MUST include: command syntax, options, examples
- **FR-003**: Help output MUST be <80 characters wide for terminal compatibility

## User Scenarios
**Given** user is unsure how to use cmdai
**When** they run `cmdai --help`
**Then** clear usage instructions are displayed
```

**Step 3: Plan**
```
/plan
```

**Generated Files**:
- plan.md (technical approach: use clap's built-in help)
- quickstart.md (test scenario: run cmdai --help, verify output)

**Step 4: Tasks**
```
/tasks
```

**Generated tasks.md**:
```markdown
## Phase 3.1: Setup
- [ ] T001 Update Cargo.toml: clap = { version = "4.5", features = ["derive"] }

## Phase 3.2: Tests First
- [ ] T002 Integration test for --help in tests/integration/test_cli_help.rs

## Phase 3.3: Core Implementation
- [ ] T003 Add help attributes to src/cli/mod.rs Cli struct

## Phase 3.5: Polish
- [ ] T004 Update README.md with help output example
```

**Step 5: Implement**
```
/implement
```

**Result**:
```rust
// src/cli/mod.rs
#[derive(Parser)]
#[command(author, version, about = "AI-powered shell command generator")]
#[command(long_about = "cmdai converts natural language to safe, POSIX-compliant shell commands")]
pub struct Cli {
    /// Natural language description of command
    #[arg(value_name = "PROMPT")]
    pub prompt: String,
}
```

**Total Time**: ~10 minutes for complete feature

---

### Example 2: Complex Feature - Command History with Search

**Step 1: Create Spec**
```
/specify "Add SQLite-based command history storage with semantic search and replay functionality"
```

**Step 2: Clarify Ambiguities**
```
/clarify
```

**Questions Asked**:
1. Q: How long should command history be retained?
   A: 90 days with configurable retention policy

2. Q: What metadata should be stored with each command?
   A: Timestamp, prompt, generated command, execution status, error output

3. Q: Should search be case-sensitive?
   A: No, case-insensitive with fuzzy matching

4. Q: How many search results to return?
   A: Top 10 by relevance score

5. Q: Should users confirm before replaying commands?
   A: Yes, always require confirmation for replay

**Updated spec.md**:
```markdown
## Clarifications

### Session 2025-10-14
- Q: Retention period → A: 90 days (configurable)
- Q: Metadata → A: Timestamp, prompt, command, status, error output
- Q: Search sensitivity → A: Case-insensitive with fuzzy matching
- Q: Result limit → A: Top 10 by relevance
- Q: Replay confirmation → A: Always required

## Requirements
- **FR-001**: System MUST store commands in SQLite database with 90-day retention
- **FR-002**: System MUST store metadata: timestamp, prompt, command, status, error output
- **FR-003**: System MUST provide case-insensitive fuzzy search returning top 10 results
- **FR-004**: System MUST require confirmation before replaying any historical command
```

**Step 3: Plan**
```
/plan
```

**Generated Files**:
- plan.md
- research.md (SQLite vs PostgreSQL decision, FTS5 for search)
- data-model.md (CommandHistory entity)
- contracts/ (not applicable, local storage)
- quickstart.md (integration test scenarios)

**data-model.md**:
```markdown
# Data Model: Command History

## CommandHistory Entity

**Fields**:
- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `created_at`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP
- `prompt`: TEXT NOT NULL
- `generated_command`: TEXT NOT NULL
- `execution_status`: TEXT CHECK(execution_status IN ('pending', 'success', 'failed'))
- `error_output`: TEXT NULL
- `metadata`: JSON NULL

**Indexes**:
- `idx_created_at` ON created_at (for retention cleanup)
- FTS5 virtual table on prompt + generated_command (for search)

**Relationships**: None (standalone table)

**Validation Rules**:
- prompt: Max 1000 characters
- generated_command: Max 5000 characters
- error_output: Max 10000 characters

**State Transitions**:
pending → success
pending → failed
```

**Step 4: Tasks**
```
/tasks
```

**Generated tasks.md** (abbreviated):
```markdown
## Phase 3.1: Setup
- [ ] T001 Add rusqlite to Cargo.toml with bundled feature
- [ ] T002 Create src/history/ module directory

## Phase 3.2: Tests First (TDD)
- [ ] T003 [P] Contract test for HistoryStore trait in tests/contract/test_history_store.rs
- [ ] T004 [P] Integration test for command storage in tests/integration/test_history_storage.rs
- [ ] T005 [P] Integration test for search in tests/integration/test_history_search.rs
- [ ] T006 [P] Integration test for replay workflow in tests/integration/test_history_replay.rs

## Phase 3.3: Core Implementation
- [ ] T007 [P] CommandHistory model in src/history/models.rs
- [ ] T008 SQLite schema migration in src/history/schema.sql
- [ ] T009 HistoryStore implementation in src/history/store.rs
- [ ] T010 Search implementation with FTS5 in src/history/search.rs
- [ ] T011 Replay workflow in src/history/replay.rs

## Phase 3.4: Integration
- [ ] T012 Wire history to CLI in src/cli/mod.rs
- [ ] T013 Add retention cleanup job in src/history/cleanup.rs

## Phase 3.5: Polish
- [ ] T014 [P] Unit tests for retention policy
- [ ] T015 Performance test: 10k commands inserted in <1s
- [ ] T016 [P] Update CLAUDE.md with history module
```

**Step 5: Analyze**
```
/analyze
```

**Report** (abbreviated):
```markdown
### Specification Analysis Report

| ID | Category | Severity | Location | Summary | Recommendation |
|----|----------|----------|----------|---------|----------------|
| C1 | Coverage | MEDIUM | spec.md FR-005 | Privacy filtering not in tasks | Add task for sensitive data filtering |

### Metrics
- Total Requirements: 12
- Total Tasks: 16
- Coverage: 92% (11/12)
- Critical: 0, High: 0, Medium: 1, Low: 0

### Next Actions
- Add task T017 for sensitive data filtering
- Proceed to /implement
```

**Step 6: Implement**
```
/implement
```

**Result**: Fully functional command history system with:
- SQLite storage
- FTS5-powered search
- Retention policy enforcement
- Replay with confirmation
- 100% test coverage

**Total Time**: ~2 hours for complete feature

---

### Example 3: Constitution Update

**Scenario**: Team decides to enforce API versioning

**Step 1: Propose Amendment**
```
/constitution "Add principle VI: API Versioning - All public APIs must use semantic versioning with breaking change migration guides"
```

**Step 2: Amendment Process**
1. Opens GitHub issue with proposal
2. Discussion period (3 business days minimum)
3. Maintainer consensus
4. Constitution updated with version bump

**Updated constitution.md**:
```markdown
### VI. API Versioning (NEW)
**Rule**: All public APIs use semantic versioning with migration guides

**Checklist**:
- [ ] Public types versioned (v1, v2 modules)
- [ ] Breaking changes documented in CHANGELOG.md
- [ ] Migration guide provided for MAJOR bumps
- [ ] Deprecated APIs kept for 1 MINOR version

**Version**: 1.1.0 (MINOR bump - new principle added)
```

**Step 3: Template Propagation**
```bash
# Update plan-template.md
# Add "API Versioning" gate to Constitution Check

# Update CLAUDE.md
# Add versioning guidance
```

**Step 4: Verify Projects Comply**
```bash
# Re-run /analyze on existing features
# Identify any violations of new principle
# Create remediation tasks
```

---

## Quick Start Checklist

Use this checklist for every new feature:

```markdown
## Feature Development Checklist

### Initialization
- [ ] Run `/specify "feature description"`
- [ ] Verify branch created: `###-feature-name`
- [ ] Verify spec.md exists

### Clarification (Recommended)
- [ ] Run `/clarify`
- [ ] Answer all questions (max 5)
- [ ] Verify Clarifications section added
- [ ] No [NEEDS CLARIFICATION] markers remain

### Planning
- [ ] Run `/plan`
- [ ] Verify Phase 0 complete (research.md)
- [ ] Verify Phase 1 complete (data-model.md, contracts/, quickstart.md)
- [ ] Constitution Check passes or deviations justified
- [ ] Progress Tracking shows phases 0-1 complete

### Task Generation
- [ ] Run `/tasks`
- [ ] Verify tasks.md created
- [ ] Review task ordering (Setup → Tests → Core → Integration → Polish)
- [ ] Verify [P] markers on parallel tasks
- [ ] Dependency graph makes sense

### Quality Analysis (Recommended)
- [ ] Run `/analyze`
- [ ] Review report for CRITICAL/HIGH issues
- [ ] Fix CRITICAL issues if any
- [ ] Decide on MEDIUM/LOW issues
- [ ] Coverage >90%

### Implementation
- [ ] Run `/implement`
- [ ] Phase 3.2 tests FAIL (expected)
- [ ] Phase 3.3 implementation makes tests PASS
- [ ] All tasks marked [x]
- [ ] Quality gates pass:
  - [ ] `cargo test` ✅
  - [ ] `cargo clippy -- -D warnings` ✅
  - [ ] `cargo fmt --check` ✅
  - [ ] Performance requirements met

### Completion
- [ ] Feature branch ready for PR
- [ ] Documentation updated
- [ ] CHANGELOG.md entry added
- [ ] Ready to merge to main
```

---

## Additional Resources

### Files and Directories

```
cmdai/
├── .specify/
│   ├── memory/
│   │   ├── constitution.md           # Project principles (v1.0.0)
│   │   └── constitution_update_checklist.md
│   ├── templates/
│   │   ├── spec-template.md          # Feature specification template
│   │   ├── plan-template.md          # Implementation plan template
│   │   ├── tasks-template.md         # Task breakdown template
│   │   └── agent-file-template.md    # Agent context template
│   └── scripts/
│       ├── bash/
│       │   ├── create-new-feature.sh     # /specify automation
│       │   ├── setup-plan.sh             # /plan prerequisites
│       │   ├── check-prerequisites.sh    # /tasks, /analyze, /implement prerequisites
│       │   └── update-agent-context.sh   # Agent file incremental updates
│       └── powershell/
│           └── [Windows equivalents]
├── .claude/
│   ├── commands/
│   │   ├── specify.md                # /specify slash command
│   │   ├── clarify.md                # /clarify slash command
│   │   ├── plan.md                   # /plan slash command
│   │   ├── tasks.md                  # /tasks slash command
│   │   ├── analyze.md                # /analyze slash command
│   │   ├── implement.md              # /implement slash command
│   │   └── constitution.md           # /constitution slash command
│   └── agents/
│       └── spec-driven-dev-guide.md  # Agent guidance
├── specs/
│   └── ###-feature-name/
│       ├── spec.md                   # Business requirements
│       ├── plan.md                   # Technical design
│       ├── research.md               # Decisions and alternatives
│       ├── data-model.md             # Entities and relationships
│       ├── quickstart.md             # Integration scenarios
│       ├── contracts/                # API contracts
│       └── tasks.md                  # Implementation checklist
├── docs/
│   └── SPECKIT-CHEATSHEET.md         # This file
├── CLAUDE.md                         # Agent context (auto-updated)
└── src/
    ├── lib.rs                        # Library exports
    ├── main.rs                       # CLI entry point
    └── [feature modules]
```

### Key Scripts

**create-new-feature.sh**:
- Creates feature branch
- Initializes spec.md from template
- Sets SPECIFY_FEATURE environment variable

**setup-plan.sh**:
- Validates spec.md exists
- Returns paths for plan.md, research.md, etc.

**check-prerequisites.sh**:
- Validates feature setup
- Returns available design documents
- Used by /tasks, /analyze, /implement

**update-agent-context.sh**:
- Incrementally updates CLAUDE.md
- Preserves manual additions
- Keeps under 150 lines for token efficiency

### Environment Variables

**SPECIFY_FEATURE**:
- Set by create-new-feature.sh
- Contains current feature branch name
- Used by all slash commands to locate spec files

### Performance Targets

| Operation | Target | Measured By |
|-----------|--------|-------------|
| System startup | <100ms | Constitution Principle V |
| First inference | <2s on M1 Mac | Constitution Principle V |
| Safety validation | <50ms P95 | Constitution Principle IV |
| Command history write | <10ms | Feature spec FR-004 |
| Command history search | <50ms | Feature spec FR-005 |
| Interactive UI response | <100ms | Feature spec FR-002 |

---

## Version History

**v1.0.0** (2025-10-14):
- Initial cheatsheet creation
- Covers constitution v1.0.0
- Complete workflow documentation
- Templates, commands, best practices

---

**Need Help?**
- Constitution questions: See `.specify/memory/constitution.md`
- Template questions: See `.specify/templates/`
- Command questions: See `.claude/commands/`
- Project questions: See `CLAUDE.md`

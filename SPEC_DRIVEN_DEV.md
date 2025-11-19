# Spec-Driven Development Guide

## Overview

cmdai follows **Spec-Driven Development** methodology using GitHub's spec-kit workflow. This ensures all features are well-defined, properly planned, and thoroughly tested before implementation.

## Why Spec-Driven Development?

**Benefits**:
- ✅ Clear requirements before coding
- ✅ Community alignment on features
- ✅ Reduced rework and technical debt
- ✅ Better documentation
- ✅ Easier code reviews
- ✅ Predictable timelines

**Without Spec-Driven Development**:
- ❌ Unclear requirements lead to multiple iterations
- ❌ Features that don't solve user problems
- ❌ Poor documentation
- ❌ Difficult to estimate effort
- ❌ Merge conflicts and integration issues

## The Workflow

### Phase 1: Specification (`/specify`)

**Goal**: Define WHAT we're building and WHY

**Command**:
```bash
/specify Feature 027: Custom Model Support
```

**Creates**: `specs/027-custom-models.md`

**Spec Contents**:
```markdown
# Feature 027: Custom Model Support

## Overview
[What is this feature?]

## Problem Statement
[What problem does this solve?]

## Requirements
### Functional Requirements
- [ ] Requirement 1
- [ ] Requirement 2

### Non-Functional Requirements
- [ ] Performance: <2s inference
- [ ] Memory: <4GB usage

## Acceptance Criteria
- [ ] User can configure custom model path
- [ ] Auto-download from Hugging Face works
- [ ] All tests pass

## Success Metrics
- Adoption: 20% of users use custom models
- Performance: No regression on default model

## Out of Scope
- Multi-model ensemble
- Model fine-tuning
```

**Review Process**:
1. Create spec draft
2. Share in GitHub Discussion
3. Gather community feedback
4. Iterate on spec (2-3 rounds)
5. Maintainer approval

**Approval Criteria**:
- ✅ Clear problem statement
- ✅ Specific acceptance criteria
- ✅ Measurable success metrics
- ✅ No major concerns from community
- ✅ Aligned with project goals

### Phase 2: Planning (`/plan`)

**Goal**: Define HOW we're building it

**Command**:
```bash
/plan specs/027-custom-models.md
```

**Creates**: `plan.md`

**Plan Contents**:
```markdown
# Implementation Plan: Custom Model Support

## Architecture

### Component Diagram
[Visual representation of components]

### API Design
```rust
pub struct ModelConfig {
    pub path: PathBuf,
    pub source: ModelSource,
}

pub enum ModelSource {
    Local(PathBuf),
    HuggingFace { repo: String, file: String },
}
```

### Data Flow
[How data flows through the system]

## Technical Decisions

### Decision 1: Model Storage Location
**Options**:
- A) ~/.cache/cmdai/models/
- B) ~/.config/cmdai/models/
- C) User-specified path

**Choice**: A (XDG cache directory)
**Rationale**: Models are cached data, not configuration

### Decision 2: Download Library
**Options**:
- A) hf-hub crate
- B) reqwest + custom logic

**Choice**: A (hf-hub)
**Rationale**: Official support, handles auth

## Testing Strategy
- Unit tests: Model config parsing
- Integration tests: Download from HF Hub
- E2E tests: Full inference with custom model

## Migration Plan
- Backward compatible with existing config
- Auto-migrate old format

## Documentation Plan
- User guide: How to use custom models
- API docs: ModelConfig API
- Troubleshooting: Common issues

## Risks & Mitigation
- **Risk**: Large model downloads fail
  - **Mitigation**: Resume support, progress bars
- **Risk**: Incompatible model formats
  - **Mitigation**: Validate model before loading
```

**Review Process**:
1. Technical design review
2. Architecture discussion
3. Identify risks and edge cases
4. Finalize implementation approach

### Phase 3: Task Breakdown (`/tasks`)

**Goal**: Create ordered implementation tasks

**Command**:
```bash
/tasks specs/027-custom-models.md
```

**Creates**: `tasks.md`

**Task Format**:
```markdown
# Implementation Tasks: Custom Model Support

## Prerequisites
- [ ] Spec approved
- [ ] Plan reviewed
- [ ] Test infrastructure ready

## Tasks (Dependency-Ordered)

### T001: Define ModelConfig struct
**Effort**: 1 hour
**Dependencies**: None
**Files**: src/config/model.rs
**Tests**: Unit test for serialization

### T002: Implement local model loading
**Effort**: 3 hours
**Dependencies**: T001
**Files**: src/model_loader.rs
**Tests**: Test with fixture model file

### T003: Add HuggingFace download support
**Effort**: 5 hours
**Dependencies**: T001, T002
**Files**: src/model_loader.rs
**Tests**: Integration test with mock HF API

### T004: Add progress indicators
**Effort**: 2 hours
**Dependencies**: T003
**Files**: src/cli/progress.rs
**Tests**: Manual testing

### T005: Update CLI to accept model path
**Effort**: 2 hours
**Dependencies**: T001
**Files**: src/cli/mod.rs, src/main.rs
**Tests**: CLI integration test

### T006: Write documentation
**Effort**: 3 hours
**Dependencies**: All above
**Files**: docs/custom-models.md
**Tests**: Documentation review

### T007: End-to-end testing
**Effort**: 4 hours
**Dependencies**: All above
**Files**: tests/integration/custom_models.rs
**Tests**: Full workflow test

## Estimated Total Effort
20 hours (2.5 days)
```

**Task Best Practices**:
- Each task is < 1 day of work
- Clear dependencies
- Specific files to modify
- Test requirements defined
- Ordered by dependencies (can parallelize when possible)

### Phase 4: Implementation (`/implement`)

**Goal**: Build the feature following TDD principles

**Command**:
```bash
/implement tasks.md
```

**TDD Workflow** (Red-Green-Refactor):

1. **Red**: Write failing test
   ```rust
   #[test]
   fn test_load_custom_model() {
       let config = ModelConfig::local("/path/to/model.gguf");
       let loader = ModelLoader::new(config);
       assert!(loader.load().is_ok()); // FAILS
   }
   ```

2. **Green**: Write minimal code to pass
   ```rust
   impl ModelLoader {
       pub fn load(&self) -> Result<Model> {
           match &self.config.source {
               ModelSource::Local(path) => {
                   let file = std::fs::read(path)?;
                   Ok(Model::from_bytes(&file)?)
               }
           }
       }
   }
   ```

3. **Refactor**: Clean up, optimize, document
   ```rust
   impl ModelLoader {
       /// Loads a model from the configured source.
       ///
       /// # Errors
       /// Returns error if file doesn't exist or is not a valid model.
       pub fn load(&self) -> Result<Model> {
           self.validate_source()?;
           match &self.config.source {
               ModelSource::Local(path) => self.load_local(path),
               ModelSource::HuggingFace { repo, file } => {
                   self.load_huggingface(repo, file)
               }
           }
       }

       fn validate_source(&self) -> Result<()> {
           // Validation logic
       }
   }
   ```

**Implementation Checklist** (per task):
- [ ] Write failing tests
- [ ] Implement minimal code
- [ ] Tests pass
- [ ] Refactor and document
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy`
- [ ] Update CHANGELOG.md
- [ ] Update documentation
- [ ] Self-review code
- [ ] Create PR

### Phase 5: Validation (`/analyze`)

**Goal**: Ensure quality and consistency

**Command**:
```bash
/analyze
```

**Validation Checks**:
1. **Spec-Plan Alignment**: Does plan address all spec requirements?
2. **Plan-Tasks Alignment**: Do tasks cover all plan components?
3. **Task-Code Alignment**: Was all planned work completed?
4. **Test Coverage**: Are all requirements tested?
5. **Documentation**: Is everything documented?
6. **Performance**: Do benchmarks pass?
7. **Security**: Any vulnerabilities introduced?

**Quality Gates**:
- ✅ All tests passing (unit, integration, e2e)
- ✅ Code coverage >80%
- ✅ No clippy warnings
- ✅ All spec requirements met
- ✅ Documentation complete
- ✅ Performance benchmarks passing
- ✅ Security audit clean

## Clarification Process

Use `/clarify` when spec is underspecified:

```bash
/clarify specs/027-custom-models.md
```

**Clarification Questions** (up to 5):
1. Should custom models support all formats (GGUF, SafeTensors, etc.)?
2. What should happen if model download fails mid-way?
3. Should we validate model compatibility before loading?
4. How should we handle model version updates?
5. Should users be able to switch models mid-session?

**Encoding Answers**:
- Update spec with clarifications
- Add to "Design Decisions" section
- Document in plan

## Constitution Alignment

Use `/constitution` to ensure feature aligns with project principles:

```bash
/constitution
```

**Project Constitution** (example principles):
1. **Safety First**: Never compromise user safety
2. **Performance**: <2s inference time
3. **Privacy**: No data leaves user's machine
4. **Simplicity**: Simple > Complex
5. **Unix Philosophy**: Do one thing well
6. **Open Source**: Transparent and collaborative

**Alignment Check**:
- ✅ Custom models don't compromise safety (validation added)
- ✅ Performance maintained (lazy loading)
- ✅ Privacy preserved (local-only)
- ⚠️ Adds complexity (mitigated with good docs)

## Best Practices

### Specification
- **Be Specific**: "Fast" → "< 2s on M1 Mac"
- **Be Measurable**: "Better UX" → "90% user satisfaction"
- **Include Non-Goals**: What we're NOT building
- **User Stories**: "As a developer, I want..."

### Planning
- **Consider Alternatives**: Show trade-offs
- **Document Decisions**: Why we chose this approach
- **Think About Edge Cases**: What can go wrong?
- **Plan for Testing**: How will we verify this works?

### Tasks
- **Small Tasks**: <1 day each
- **Clear Dependencies**: What blocks this task?
- **Specific Outcomes**: What's done when complete?
- **Include Tests**: Every task has test requirements

### Implementation
- **Test First**: Write tests before code
- **Small Commits**: One logical change per commit
- **Self-Review**: Review your own code first
- **Ask for Help**: Stuck? Ask in Discussion

## Common Pitfalls

### ❌ Starting implementation without spec
**Problem**: Unclear requirements, rework needed
**Solution**: Always create spec first, even for small features

### ❌ Skipping planning phase
**Problem**: Architecture issues, technical debt
**Solution**: Invest time in design before coding

### ❌ Tasks too large
**Problem**: Hard to track progress, delays
**Solution**: Break tasks into <1 day chunks

### ❌ No tests
**Problem**: Regressions, low confidence
**Solution**: TDD - tests before code

### ❌ Not updating docs
**Problem**: Features unusable, support burden
**Solution**: Documentation is part of "Done"

## Example: Full Workflow

Let's walk through a complete example:

### Week 1: Specification

**Monday**: Create initial spec
```bash
/specify Feature 030: Safe Command Execution
```

**Tuesday-Wednesday**: Community review
- Share in Discussion
- Gather feedback
- Refine requirements

**Thursday**: Final spec review
- Maintainer approval
- Spec frozen

### Week 2: Planning

**Monday**: Create implementation plan
```bash
/plan specs/030-safe-execution.md
```

**Tuesday**: Technical review
- Architecture discussion
- Security review
- Performance considerations

**Wednesday**: Generate tasks
```bash
/tasks specs/030-safe-execution.md
```

**Thursday-Friday**: Task review
- Dependency ordering
- Effort estimation
- Assign tasks (if team)

### Week 3-4: Implementation

**Each Task** (1-2 days):
1. Read task requirements
2. Write failing tests
3. Implement code
4. Tests pass
5. Refactor
6. Document
7. Create PR
8. Code review
9. Merge

**Daily Standup** (async in Discussion):
- What I completed yesterday
- What I'm working on today
- Any blockers

### Week 5: Validation

**Monday**: Integration testing
```bash
cargo test --all
```

**Tuesday**: Performance validation
```bash
cargo bench
```

**Wednesday**: Security audit
```bash
cargo audit
cargo clippy -- -D warnings
```

**Thursday**: Documentation review
- User docs complete?
- API docs complete?
- Examples added?

**Friday**: Final validation
```bash
/analyze
```

**Result**: Feature ready for release! 🎉

## Tools & Commands

### Spec-Driven Commands
All available in `.claude/commands/`:
- `/specify` - Create feature spec
- `/plan` - Create implementation plan
- `/tasks` - Generate task breakdown
- `/implement` - Execute implementation
- `/analyze` - Validate cross-artifact consistency
- `/clarify` - Identify underspecified areas
- `/constitution` - Check alignment with principles

### Development Commands
```bash
# Build
cargo build --release

# Test
cargo test                    # All tests
cargo test test_name         # Specific test
cargo test --doc             # Doc tests

# Quality
cargo fmt                     # Format code
cargo clippy -- -D warnings  # Lint
cargo audit                  # Security audit
cargo bench                  # Benchmarks

# Documentation
cargo doc --open             # API docs
mdbook serve docs/           # User docs (if using mdbook)
```

## Getting Help

- 📖 Read [ROADMAP.md](./ROADMAP.md) for feature context
- 💬 Ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- 👥 Join community calls (if scheduled)
- 📝 Review existing specs in `/specs` directory
- 🔍 Look at merged PRs for examples

## Resources

- [Spec-Kit Documentation](https://github.com/github/spec-kit)
- [Test-Driven Development Guide](https://martinfowler.com/bliki/TestDrivenDevelopment.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [cmdai Architecture](./CLAUDE.md)

---

**Remember**: Good specs lead to good code. Time invested in specification and planning pays off 10x during implementation!

Happy spec-driven development! 🚀

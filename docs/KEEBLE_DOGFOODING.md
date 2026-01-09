# Keeble: Caro Dogfooding Strategy

> **"Using Caro to build Caro"** - A comprehensive dogfooding framework that positions our team as the primary users, testers, and feedback source for continuous improvement.

## Executive Summary

**Keeble** is our internal codename for the systematic dogfooding of Caro throughout our development lifecycle. By integrating Caro into every phase of building Caro itself, we:

1. **Become our own first users** - Experiencing the product as end-users do
2. **Discover issues before release** - Fast-feedback loops catch bugs early
3. **Drive feature priorities** - Real usage patterns inform roadmap decisions
4. **Validate safety mechanisms** - Our critical commands stress-test safety patterns
5. **Create authentic documentation** - Examples come from real development workflows

---

## 1. Philosophy: The Keeble Principles

### 1.1 Genetic Development

Caro is **genetically developed** - it participates in its own creation:

```
┌─────────────────────────────────────────────────────────────┐
│                    GENETIC DEVELOPMENT LOOP                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Developer Intent ──► Caro ──► Shell Command ──► Result   │
│         ▲                                           │       │
│         │                                           │       │
│         └───────────── Feedback ◄───────────────────┘       │
│                                                             │
│   "Caro generates commands that build/test/release Caro"   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Core Values

| Principle | Description |
|-----------|-------------|
| **Eat Our Own Cooking** | Every developer uses Caro daily for development tasks |
| **Fast Feedback** | Issues discovered in development are fixed before release |
| **Real-World Testing** | Production-like usage patterns, not synthetic tests |
| **Safety First** | Our own dangerous commands validate safety patterns |
| **Continuous Improvement** | Every friction point becomes a feature candidate |

---

## 2. Integration Points

Caro integrates into the Caro development lifecycle at every stage:

### 2.1 Development Phase

#### Daily Development Tasks

| Task Category | Example Natural Language | Generated Command |
|---------------|-------------------------|-------------------|
| **Build** | "build caro in release mode" | `cargo build --release` |
| **Test** | "run only the safety tests" | `cargo test --test safety` |
| **Format** | "format all rust files" | `cargo fmt` |
| **Lint** | "run clippy with warnings as errors" | `cargo clippy -- -D warnings` |
| **Watch** | "watch for changes and run tests" | `cargo watch -x test` |
| **Search** | "find all files that mention SafetyValidator" | `grep -r "SafetyValidator" src/` |
| **Git** | "show commits from today" | `git log --since="midnight"` |

#### Code Exploration

| Task | Natural Language | Generated Command |
|------|------------------|-------------------|
| **Find definitions** | "find where CommandGenerator trait is defined" | `grep -rn "trait CommandGenerator" src/` |
| **Count lines** | "count lines of rust code in src" | `find src -name "*.rs" \| xargs wc -l` |
| **Find large files** | "find rust files larger than 10KB" | `find src -name "*.rs" -size +10k` |
| **Recent changes** | "list files changed in last commit" | `git diff --name-only HEAD~1` |

### 2.2 Testing Phase

#### Test Execution

| Scenario | Natural Language | Generated Command |
|----------|------------------|-------------------|
| **Run all tests** | "run the full test suite" | `cargo test` |
| **Single test** | "run the static matcher test" | `cargo test static_matcher` |
| **Verbose output** | "run tests with output shown" | `cargo test -- --nocapture` |
| **Contract tests** | "run contract tests only" | `cargo test --test '*contract*'` |
| **Benchmark** | "run performance benchmarks" | `cargo bench` |

#### Test Analysis

| Scenario | Natural Language | Generated Command |
|----------|------------------|-------------------|
| **Find failing** | "show only failed tests" | `cargo test 2>&1 \| grep -E "FAILED\|error"` |
| **Coverage** | "generate test coverage report" | `cargo tarpaulin --out Html` |
| **Test count** | "count number of test functions" | `grep -r "#\[test\]" tests/ \| wc -l` |

### 2.3 CI/CD Phase

#### GitHub Actions Operations

| Task | Natural Language | Generated Command |
|------|------------------|-------------------|
| **View workflows** | "list recent github workflow runs" | `gh run list --limit 10` |
| **Check status** | "show status of latest workflow" | `gh run list --limit 1 --json status` |
| **View logs** | "show logs from failed workflow" | `gh run view --log-failed` |
| **Trigger workflow** | "manually trigger the test workflow" | `gh workflow run test.yml` |

#### Release Operations

| Task | Natural Language | Generated Command |
|------|------------------|-------------------|
| **Create tag** | "create version tag v1.0.5" | `git tag -a v1.0.5 -m "Release v1.0.5"` |
| **Push tag** | "push all tags to origin" | `git push origin --tags` |
| **Check crates.io** | "check if caro is published on crates.io" | `cargo search caro` |
| **Publish** | "publish to crates.io" | `cargo publish` |

### 2.4 Documentation Phase

| Task | Natural Language | Generated Command |
|------|------------------|-------------------|
| **Generate docs** | "generate rust documentation" | `cargo doc --open` |
| **Find TODOs** | "find all TODO comments in codebase" | `grep -rn "TODO" src/` |
| **Check links** | "validate markdown links" | `find docs -name "*.md" -exec markdown-link-check {} \;` |

### 2.5 Safety Validation (Self-Testing)

Our own development commands test safety patterns:

| Dangerous Pattern | We Accidentally Type | Caro Should Block |
|-------------------|---------------------|-------------------|
| **Recursive delete** | "delete everything in current dir" | `rm -rf *` (BLOCKED) |
| **System paths** | "clean up /usr/local/bin" | Critical path warning |
| **Force operations** | "force push to main" | `git push --force origin main` (WARNING) |
| **Privilege escalation** | "run as root" | `sudo su` (WARNING) |

---

## 3. Feedback Loop Architecture

### 3.1 Immediate Feedback (During Development)

```
┌─────────────────────────────────────────────────────────────┐
│                   IMMEDIATE FEEDBACK LOOP                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   1. Developer uses Caro for task                           │
│   2. Command generated                                      │
│   3. Developer evaluates:                                   │
│      ├─ Correct? → Execute, continue                        │
│      ├─ Wrong? → Log issue, fix immediately                 │
│      └─ Suboptimal? → Note for improvement                  │
│   4. Pattern captured in test suite                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Structured Feedback Capture

#### Friction Log

Developers maintain a friction log for every Keeble usage issue:

```yaml
# .claude/keeble/friction-log.yaml
entries:
  - date: 2025-01-09
    developer: jg
    intent: "run tests matching 'safety'"
    expected: "cargo test safety"
    actual: "cargo test --test safety_tests"
    severity: minor
    status: fixed
    fix_commit: abc123

  - date: 2025-01-08
    developer: jg
    intent: "show disk usage of target folder"
    expected: "du -sh target/"
    actual: "du -h target/" # missing -s for summary
    severity: minor
    status: pending
```

#### Success Log

Track patterns that work well for expansion:

```yaml
# .claude/keeble/success-log.yaml
entries:
  - date: 2025-01-09
    intent: "find all unsafe blocks in codebase"
    generated: "grep -rn 'unsafe' src/"
    quality: excellent
    notes: "Added to static patterns"
```

### 3.3 Weekly Dogfooding Review

| Activity | Frequency | Output |
|----------|-----------|--------|
| Friction log review | Weekly | Prioritized bug fixes |
| Success pattern harvest | Weekly | New static patterns |
| Safety near-misses | Weekly | New safety patterns |
| Performance observations | Weekly | Optimization targets |

---

## 4. Keeble Integration Tiers

### Tier 1: Essential (Day 1)

Commands every developer uses daily:

| Category | Commands |
|----------|----------|
| **Build** | `cargo build`, `cargo build --release` |
| **Test** | `cargo test`, `cargo test <name>`, `cargo test --test <file>` |
| **Format** | `cargo fmt`, `cargo fmt --check` |
| **Lint** | `cargo clippy` |
| **Git basics** | `git status`, `git diff`, `git log`, `git add`, `git commit` |

### Tier 2: Standard (Week 1)

Common development workflow commands:

| Category | Commands |
|----------|----------|
| **Search** | `grep`, `find`, `rg` |
| **File ops** | `ls`, `cat`, `head`, `tail`, `wc` |
| **Process** | `ps`, `kill`, `top` |
| **Git advanced** | `git branch`, `git checkout`, `git merge`, `git rebase` |
| **Make** | `make check`, `make test`, `make build` |

### Tier 3: Advanced (Month 1)

Power user and specialized commands:

| Category | Commands |
|----------|----------|
| **CI/CD** | `gh` commands, workflow management |
| **Docker** | Container management (if applicable) |
| **Benchmarking** | `cargo bench`, `hyperfine` |
| **Profiling** | `cargo flamegraph`, `perf` |
| **Release** | `cargo publish`, version management |

### Tier 4: Expert (Ongoing)

Complex multi-step operations:

| Category | Examples |
|----------|----------|
| **Pipeline** | Complex command chains with pipes |
| **Scripting** | One-liners, awk/sed transformations |
| **Debug** | GDB/LLDB integration, core dumps |
| **System** | Performance tuning, resource management |

---

## 5. Metrics & Success Criteria

### 5.1 Quantitative Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Daily Caro Usage** | 50+ commands/developer/day | Command logging |
| **First-Try Success Rate** | >85% | Friction log analysis |
| **Safety Block Rate** | 100% for critical commands | Safety test suite |
| **Command Generation Time** | <2 seconds | Performance monitoring |
| **Bug Detection Lead Time** | 24 hours before release | Issue tracking |

### 5.2 Qualitative Metrics

| Metric | Assessment Method |
|--------|-------------------|
| **Developer Satisfaction** | Weekly pulse check |
| **Command Quality** | Code review of generated commands |
| **Documentation Authenticity** | Examples from real usage |
| **Onboarding Speed** | New developer time-to-productivity |

### 5.3 Dogfooding Dashboard

Track Keeble health with key indicators:

```
┌─────────────────────────────────────────────────────────────┐
│                    KEEBLE HEALTH DASHBOARD                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Daily Usage:        ████████████████░░░░ 156 commands     │
│   Success Rate:       ████████████████████ 92%              │
│   Friction Reports:   ██░░░░░░░░░░░░░░░░░░ 3 pending        │
│   Safety Blocks:      ████████████████████ 100% working     │
│   Avg Response Time:  ████████░░░░░░░░░░░░ 1.2s             │
│                                                             │
│   Last Week: +12% usage, -2 friction reports                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal:** Establish basic dogfooding infrastructure

| Task | Deliverable |
|------|-------------|
| Create friction log template | `.claude/keeble/friction-log.yaml` |
| Create success log template | `.claude/keeble/success-log.yaml` |
| Document Tier 1 commands | Integration test suite |
| Set up daily usage tracking | Logging configuration |
| Create developer onboarding guide | `docs/KEEBLE_QUICKSTART.md` |

### Phase 2: Integration (Week 3-4)

**Goal:** Embed Keeble into development workflows

| Task | Deliverable |
|------|-------------|
| Integrate with Makefile targets | `make keeble-*` commands |
| Add CI/CD Keeble validation | GitHub Action workflow |
| Create weekly review ritual | Calendar event + template |
| Expand to Tier 2 commands | Static pattern additions |
| First dogfooding retrospective | Improvement backlog |

### Phase 3: Optimization (Month 2)

**Goal:** Refine based on real usage data

| Task | Deliverable |
|------|-------------|
| Analyze friction log patterns | Pattern improvements |
| Harvest successful patterns | Static matcher expansion |
| Performance optimization | Sub-second common commands |
| Safety pattern validation | Additional safety tests |
| Tier 3 command support | Advanced command coverage |

### Phase 4: Maturity (Month 3+)

**Goal:** Self-sustaining dogfooding ecosystem

| Task | Deliverable |
|------|-------------|
| Automated metrics collection | Dashboard implementation |
| Pattern contribution workflow | Developer PRs for patterns |
| Cross-platform validation | macOS/Linux/Windows coverage |
| External dogfooding program | Beta user feedback loop |
| Documentation from dogfooding | Real-world example library |

---

## 7. Risk Mitigation

### 7.1 Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Developers bypass Caro** | Low adoption, missed feedback | Make Caro faster than typing commands |
| **False safety blocks** | Frustration, disabled safety | Tune patterns based on friction log |
| **Poor command quality** | Wasted time, errors | Rapid iteration on patterns |
| **Feedback fatigue** | Incomplete friction logs | Lightweight logging, weekly batching |
| **Scope creep** | Feature bloat | Prioritize by usage frequency |

### 7.2 Circuit Breakers

Conditions that pause Keeble expansion:

- Friction log > 10 unresolved entries
- Success rate < 80% for any tier
- Safety false positive rate > 5%
- Developer satisfaction score < 3/5

---

## 8. Developer Quick Reference

### 8.1 Daily Keeble Workflow

```bash
# Morning: Build and test
caro "build in release mode"
caro "run the test suite"

# Development: Search and explore
caro "find files containing CommandGenerator"
caro "show recent git commits"

# Before commit: Quality checks
caro "format code"
caro "run clippy"
caro "check for security vulnerabilities"

# End of day: Status check
caro "show git status"
caro "list uncommitted changes"
```

### 8.2 Reporting Friction

When Caro generates a suboptimal command:

1. **Don't fix manually** - Let the wrong command run (safely) or note the issue
2. **Log it** - Add entry to `.claude/keeble/friction-log.yaml`
3. **Continue** - Use the correct command for now
4. **Review** - Weekly review will prioritize fixes

### 8.3 Celebrating Success

When Caro generates an excellent command:

1. **Note the pattern** - Add to `.claude/keeble/success-log.yaml`
2. **Consider expansion** - Could this pattern help others?
3. **Share** - Mention in weekly review for pattern adoption

---

## 9. Appendix

### A. Command Categories for Dogfooding

```yaml
categories:
  build:
    - cargo build
    - cargo build --release
    - cargo build --features <feature>

  test:
    - cargo test
    - cargo test <pattern>
    - cargo test --test <file>
    - cargo test -- --nocapture

  quality:
    - cargo fmt
    - cargo clippy
    - cargo audit

  git:
    - git status
    - git diff
    - git log
    - git add
    - git commit
    - git push
    - git pull

  search:
    - grep -r <pattern> src/
    - find . -name "*.rs"
    - rg <pattern>

  file:
    - ls -la
    - cat <file>
    - head/tail <file>
    - wc -l <file>
```

### B. Safety Patterns We Validate

Through dogfooding, we validate these dangerous patterns work:

```yaml
critical_blocks:
  - "rm -rf /"
  - "rm -rf ~"
  - "rm -rf *"
  - "mkfs"
  - "dd if=/dev/zero"
  - ":(){:|:&};:"  # fork bomb

high_warnings:
  - "sudo su"
  - "chmod 777"
  - "curl | bash"
  - "git push --force"
```

### C. Related Documents

- [CONTRIBUTING.md](/CONTRIBUTING.md) - Development guidelines
- [docs/development/TDD-WORKFLOW.md](/docs/development/TDD-WORKFLOW.md) - Test-driven development
- [.claude/beta-testing/](/claude/beta-testing/) - Beta testing framework
- [docs/RELEASE_PROCESS.md](/docs/RELEASE_PROCESS.md) - Release workflow

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-01-09 | 1.0.0 | Initial Keeble dogfooding plan |

---

*"The best way to build great software is to use it yourself."*

# Caro Anti-Goals

**Document Version**: 1.0
**Last Updated**: 2026-01-11
**Status**: Active
**Purpose**: Define what Caro will NOT do, to guide product decisions and filter ideas.

---

## What Are Anti-Goals?

Anti-goals are explicit constraints that define the boundaries of the project. They are as important as goals because they:

1. **Prevent scope creep** - Clear "no" saves time debating edge cases
2. **Preserve identity** - Keeps Caro true to its mission
3. **Speed decision-making** - Instant rejection for ideas that violate anti-goals
4. **Guide contributions** - Community knows what PRs won't be accepted

**Important**: Anti-goals are not forever. They can be revisited during major version planning (v2.0, v3.0). But within a release cycle, they are **non-negotiable**.

---

## Core Anti-Goals

### 1. No Cloud Dependencies for Core Functionality

**Statement**: Caro must work fully offline. Core features cannot require internet connectivity.

**What this means**:
- Command generation must work without network
- Local inference backends are first-class citizens
- No "phone home" for basic operation
- Configuration stored locally, not in cloud

**What this does NOT mean**:
- Cloud features are forbidden (opt-in cloud sync is fine)
- Remote backends are banned (API backends are supported)
- Updates can't check online (update checks are fine)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add OpenAI backend" | ACCEPT | Optional backend, local still works |
| "Require API key for first run" | REJECT | Blocks offline use |
| "Cloud-only command history sync" | REJECT | No local alternative |
| "Optional cloud sync with local fallback" | ACCEPT | Local-first, cloud optional |

---

### 2. No Paid API Requirements for Core Features

**Statement**: Users must be able to use all core functionality without paying for third-party APIs.

**What this means**:
- MLX/local backends remain fully featured
- Static pattern matching works without any model
- No features gated behind "requires OpenAI/Anthropic API"
- Free tier must be genuinely useful

**What this does NOT mean**:
- Paid backends are banned (OpenAI backend is fine as optional)
- Premium features can't exist (enterprise tier is fine)
- We can't charge for anything (paid support/enterprise is fine)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add Claude API backend" | ACCEPT | Optional, local still works |
| "Advanced safety only with paid API" | REJECT | Safety is core, must be free |
| "Enterprise SSO integration" | ACCEPT | Enterprise feature, not core |
| "Better accuracy requires GPT-4" | REJECT | Implies local is inferior |

---

### 3. No Complex Installation

**Statement**: Caro must install with a single command and run as a single binary.

**What this means**:
- `curl | sh` or `brew install` must work
- No Docker required for basic use
- No database setup
- No configuration required to start
- Binary should be <50MB

**What this does NOT mean**:
- Advanced setups are forbidden (Docker option is fine)
- No dependencies at all (runtime deps like libc are fine)
- Can't have optional components (plugins are fine)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add plugin system" | ACCEPT | Plugins are optional |
| "Require PostgreSQL for history" | REJECT | Too complex |
| "Ship models separately" | ACCEPT | With clear download UX |
| "Require Python runtime" | REJECT | Adds install complexity |

---

### 4. No Telemetry That Can't Be Disabled

**Statement**: All telemetry must be opt-in or easily disabled. No silent data collection.

**What this means**:
- Telemetry is OFF by default
- Single config flag to disable all telemetry
- Clear documentation of what's collected
- No personally identifiable information ever

**What this does NOT mean**:
- Telemetry is banned (opt-in analytics are fine)
- Crash reports are forbidden (opt-in crash reports are fine)
- No usage metrics (aggregated, anonymized, opt-in is fine)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Opt-in usage analytics" | ACCEPT | User chooses |
| "Anonymous crash reporting" | ACCEPT | If opt-in |
| "Track commands for improvement" | REJECT | Too sensitive |
| "Required telemetry for free tier" | REJECT | Coercive |

---

### 5. No Features That Compromise Safety

**Statement**: We will not add features that bypass or weaken the safety system, even if users request them.

**What this means**:
- No `--unsafe` flag that skips all validation
- No way to disable safety warnings permanently
- Dangerous command patterns always flagged
- Safety cannot be "opted out" of

**What this does NOT mean**:
- Users can't run dangerous commands (they can, with confirmation)
- No expert mode (reduced prompts, but safety still active)
- Safety can't evolve (false positives should be fixed)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add expert mode with fewer prompts" | ACCEPT | Safety still active |
| "Flag to disable all warnings" | REJECT | Removes safety |
| "Allow rm -rf without confirmation" | REJECT | Too dangerous |
| "Reduce false positives in safety" | ACCEPT | Improves, not removes |

---

### 6. No Lock-in to Specific LLM Providers

**Statement**: Caro must support multiple backends and never depend on a single provider.

**What this means**:
- Backend trait abstraction maintained
- No provider-specific features in core
- Local inference always an option
- Easy to add new backends

**What this does NOT mean**:
- All backends must be equal (some may be better)
- No backend-specific optimizations (fine in backend code)
- Can't recommend specific backends (we can)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add Ollama backend" | ACCEPT | More options |
| "OpenAI-only feature" | REJECT | Provider lock-in |
| "Optimize for MLX on Apple Silicon" | ACCEPT | Platform optimization, not lock-in |
| "Deprecate local backends" | REJECT | Core anti-goal violation |

---

### 7. No Scope Creep Beyond CLI

**Statement**: Caro is a CLI tool. We won't build GUI applications, IDE plugins (as core), or web interfaces.

**What this means**:
- Focus on terminal experience
- Output designed for terminal consumption
- No Electron apps in core
- No web dashboard as core feature

**What this does NOT mean**:
- Community can't build GUIs (they can, we won't maintain)
- No TUI elements (rich terminal UI is fine)
- No integrations (IDE plugins as separate projects are fine)
- No web documentation (docs site is fine)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Add VS Code extension" | ACCEPT (community) | Not core, community project |
| "Build Electron desktop app" | REJECT | Out of scope |
| "Rich terminal UI with colors" | ACCEPT | Still CLI |
| "Web-based command builder" | REJECT | Not CLI |

---

### 8. No Breaking Changes Without Migration Path

**Statement**: We will not break user workflows without providing clear migration guidance.

**What this means**:
- Deprecation warnings before removal
- Config migration scripts when format changes
- Clear changelog for breaking changes
- Semantic versioning respected

**What this does NOT mean**:
- We can never remove features (we can, with process)
- No breaking changes ever (major versions can break)
- Backwards compatibility forever (reasonable timeframes)

**Examples**:
| Idea | Decision | Reason |
|------|----------|--------|
| "Remove deprecated --old-flag" | ACCEPT | With migration docs |
| "Change config format silently" | REJECT | No migration path |
| "Rename command in major version" | ACCEPT | Major version, documented |
| "Remove feature without notice" | REJECT | Breaks workflows |

---

## Anti-Goal Review Process

### When to Review

- **Major version planning** (v2.0, v3.0): Full review of all anti-goals
- **Quarterly roadmap**: Check if anti-goals still serve the mission
- **Community feedback**: If consistent pushback, discuss (but don't auto-accept)

### How to Propose Changes

1. Create GitHub Discussion with `anti-goal-review` label
2. Explain why the anti-goal should change
3. Provide evidence (user research, market changes, technical necessity)
4. Core team reviews in roadmap planning session
5. Decision documented in ADR if anti-goal is modified

### Who Can Change Anti-Goals

- Core maintainers only
- Requires consensus (not majority vote)
- Must be documented with reasoning

---

## Using Anti-Goals in the Agentic Idea Pipeline

The Critical Evaluation Agent uses anti-goals as **hard constraints**:

```python
def evaluate_against_anti_goals(idea: CandidateIdea) -> Optional[str]:
    """Returns violation reason if idea violates anti-goals, else None."""

    anti_goals = [
        ("cloud_dependency", "requires cloud for core functionality"),
        ("paid_api_required", "requires paid API for core features"),
        ("complex_install", "adds installation complexity"),
        ("mandatory_telemetry", "requires non-optional telemetry"),
        ("safety_bypass", "compromises safety system"),
        ("provider_lockin", "locks users to specific provider"),
        ("scope_creep", "outside CLI scope"),
        ("breaking_change", "breaks workflows without migration"),
    ]

    for (check, violation_msg) in anti_goals:
        if idea_violates(idea, check):
            return f"REJECT: Violates anti-goal - {violation_msg}"

    return None  # No violations
```

Anti-goal violations are **instant rejections** - no scoring, no human review needed.

---

## Quick Reference Card

| Anti-Goal | One-Liner |
|-----------|-----------|
| **No Cloud Deps** | Must work offline |
| **No Paid APIs** | Free tier is full-featured |
| **No Complex Install** | Single binary, one command |
| **No Hidden Telemetry** | Opt-in only |
| **No Safety Bypass** | Can't disable safety |
| **No Provider Lock-in** | Multiple backends always |
| **No Scope Creep** | CLI only, no GUI |
| **No Silent Breaks** | Migration path required |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-11 | Product Lead | Initial version |

---

**Remember**: Anti-goals protect the project's soul. When in doubt, reject.

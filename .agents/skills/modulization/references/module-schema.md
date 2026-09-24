# Module Schema Reference

This document defines the formal schema for Modulization modules — the coherent units of work that the skill produces.

---

## Module Object

```yaml
# Full Module Schema
module:
  # === Identity ===
  id: string                    # Unique identifier (MOD-YYYY-NNN format)
  title: string                 # Human-readable title (max 80 chars)
  slug: string                  # URL-safe identifier

  # === Classification ===
  classification: enum          # integrate_now | schedule | ice | archive
  confidence: float             # 0.0-1.0 confidence in classification
  milestone: string | null      # Target milestone (if scheduled)

  # === Timing ===
  created_at: datetime          # When module was first identified
  updated_at: datetime          # Last modification
  stale_since: datetime | null  # When oldest signal became stale

  # === Description ===
  description: string           # What this module addresses (1-3 sentences)
  problem: string               # The underlying problem being solved
  context: string               # Historical and technical context

  # === Signals ===
  signals: Signal[]             # Array of contributing signals
  signal_count: int             # Total number of signals

  # === Scope ===
  scope:
    files: string[]             # Affected file paths
    file_count: int             # Number of affected files
    estimated_lines: int        # Approximate lines of change
    complexity: enum            # trivial | small | medium | large | epic
    areas: string[]             # Domain areas (from CODEOWNERS/areas)

  # === Risk Assessment ===
  risk:
    level: enum                 # low | medium | high | critical
    if_ignored: string          # Consequence of not addressing
    security_relevant: bool     # Has security implications
    user_facing: bool           # Affects end users
    breaking_change: bool       # Would break existing behavior

  # === Dependencies ===
  dependencies:
    blocks: string[]            # Module IDs this blocks
    blocked_by: string[]        # Module IDs blocking this
    related: string[]           # Related modules (informational)
    external: string[]          # External dependencies (libs, APIs)

  # === Origin Tracking ===
  origin:
    primary_source: enum        # codebase | github_pr | github_issue | discussion
    trace: OriginTrace[]        # Full lineage of the work
    contributors: string[]      # GitHub usernames involved

  # === Recommendation ===
  recommendation:
    action: enum                # integrate | schedule | ice | archive | escalate
    rationale: string           # Why this classification
    suggested_milestone: string # Where to schedule
    effort_estimate: enum       # hours | days | sprint | quarter
    priority_within_milestone: enum  # P1 | P2 | P3

  # === Spec Integration ===
  spec_seed:
    generated: bool             # Whether a spec seed was created
    path: string | null         # Path to spec seed file
    suggested_artifacts: string[]  # PRD, ADR, Design, etc.

  # === Metadata ===
  metadata:
    version: string             # Schema version
    generator: string           # "modulization/1.0.0"
    cadence: string             # Which cadence run produced this
    tags: string[]              # Custom tags
```

---

## Signal Object

```yaml
signal:
  # === Identity ===
  id: string                    # Unique signal identifier
  type: enum                    # See Signal Types below

  # === Location ===
  source: enum                  # codebase | github | process | historical
  location:
    file: string | null         # File path (for codebase signals)
    line: int | null            # Line number
    url: string | null          # GitHub URL (for GitHub signals)

  # === Content ===
  content: string               # The actual signal content
  excerpt: string               # Short excerpt for display

  # === Timing ===
  discovered_at: datetime       # When signal was found
  signal_age_days: int          # How old the signal is
  last_activity: datetime       # Last related activity

  # === Classification ===
  severity: enum                # info | warning | important | critical
  category: enum                # See Signal Categories below

  # === Relationships ===
  related_signals: string[]     # IDs of related signals
  superseded_by: string | null  # If this signal is obsolete
```

### Signal Types

```yaml
signal_types:
  # Codebase Signals
  - todo_comment                # TODO in code
  - fixme_comment               # FIXME in code
  - hack_comment                # HACK in code
  - note_comment                # NOTE in code
  - xxx_comment                 # XXX in code
  - incomplete_impl             # Stub or partial implementation
  - dead_code                   # Unreferenced code
  - stale_comment               # Comment doesn't match code
  - feature_flag                # Experimental toggle
  - deprecated_usage            # Using deprecated APIs

  # GitHub Signals
  - stale_pr                    # PR without activity
  - draft_pr                    # Incomplete PR
  - closed_unmerged_pr          # PR that was closed, not merged
  - stale_issue                 # Issue without activity
  - unresolved_review           # PR review requesting changes
  - discussion_unresolved       # Open discussion thread
  - abandoned_branch            # Branch with no recent commits

  # Process Signals
  - incomplete_worktree         # Worktree with unfinished work
  - roadmap_gap                 # Work not on roadmap
  - spec_incomplete             # Incomplete spec artifact
  - test_coverage_gap           # Missing test coverage

  # Historical Signals
  - reverted_commit             # Work that was undone
  - stalled_effort              # Effort that stopped mid-way
  - deferred_decision           # Decision postponed
```

### Signal Categories

```yaml
signal_categories:
  - technical_debt              # Code quality issues
  - incomplete_feature          # Partial implementations
  - security                    # Security-related work
  - performance                 # Performance improvements
  - documentation               # Documentation gaps
  - testing                     # Test coverage/quality
  - cleanup                     # Refactoring, removal
  - enhancement                 # Feature improvements
  - bug                         # Known bugs
  - infrastructure              # Build/deploy/CI work
```

---

## Origin Trace Object

```yaml
origin_trace:
  timestamp: datetime           # When this event occurred
  type: enum                    # issue | pr | discussion | commit | comment
  url: string                   # Link to the event
  author: string                # Who created this
  title: string                 # Title or summary
  excerpt: string               # Relevant excerpt
  decision: string | null       # Any decision made here
```

---

## Report Object

```yaml
report:
  # === Identity ===
  id: string                    # Report identifier
  generated_at: datetime        # When generated
  cadence: string               # weekly | daily | on-demand

  # === Scope ===
  scope:
    repository: string          # Repository name
    ref: string                 # Git ref scanned
    commit: string              # Commit SHA

  # === Summary ===
  summary:
    signals_collected: int      # Total signals found
    modules_formed: int         # Modules created
    by_classification:
      integrate_now: int
      schedule: int
      ice: int
      archive: int

  # === Modules ===
  modules: Module[]             # Array of modules

  # === Outputs ===
  outputs:
    report_path: string         # Path to this report
    spec_seeds: string[]        # Paths to generated spec seeds
    github_comments: int        # Comments posted
    roadmap_suggestions: int    # Roadmap updates suggested

  # === Comparison ===
  comparison:
    previous_report: string | null  # Previous report ID
    new_modules: int            # Newly discovered
    resolved_modules: int       # Previously found, now resolved
    unchanged_modules: int      # Still present

  # === Metadata ===
  metadata:
    schema_version: string
    generator_version: string
    duration_seconds: int       # How long the scan took
```

---

## Spec Seed Object

```yaml
spec_seed:
  # === Identity ===
  module_id: string             # Source module ID
  generated_at: datetime        # When generated

  # === Core Content ===
  title: string                 # Feature/work title
  problem: string               # Problem statement
  context: string               # Background and history

  # === Requirements Hints ===
  user_scenarios: string[]      # Potential user scenarios
  constraints: string[]         # Known constraints
  acceptance_hints: string[]    # Potential acceptance criteria

  # === Technical Context ===
  affected_areas: string[]      # Domain areas
  affected_files: string[]      # Known file paths
  related_code: CodeReference[] # Relevant code snippets

  # === Suggested Artifacts ===
  suggested_artifacts:
    - type: enum                # prd | adr | design | c4 | api_spec
      rationale: string         # Why this artifact is needed

  # === Open Questions ===
  open_questions: string[]      # Questions for clarification

  # === Source Signals ===
  source_signals: string[]      # Signal IDs that formed this
```

---

## Complexity Scale

```yaml
complexity_scale:
  trivial:
    description: "Single-line change or comment"
    estimated_hours: "< 1"
    typical_files: 1

  small:
    description: "Localized change, single file or function"
    estimated_hours: "1-4"
    typical_files: "1-3"

  medium:
    description: "Multi-file change, single domain"
    estimated_hours: "4-16"
    typical_files: "3-10"

  large:
    description: "Cross-domain change, architectural impact"
    estimated_hours: "16-40"
    typical_files: "10-30"

  epic:
    description: "Major feature, likely needs breakdown"
    estimated_hours: "40+"
    typical_files: "30+"
```

---

## Risk Levels

```yaml
risk_levels:
  low:
    description: "Minimal consequence if delayed"
    typical_signals:
      - Minor TODOs
      - Nice-to-have improvements
      - Documentation polish

  medium:
    description: "Should be addressed within 1-2 milestones"
    typical_signals:
      - Stale PRs with useful changes
      - Performance improvements
      - Non-critical bugs

  high:
    description: "Should be addressed in next milestone"
    typical_signals:
      - Security-adjacent issues
      - User-facing bugs
      - Blocking other work

  critical:
    description: "Should be addressed immediately"
    typical_signals:
      - Security vulnerabilities
      - Data integrity issues
      - Production blockers
```

---

## Example Module (Complete)

```yaml
module:
  id: "MOD-2026-007"
  title: "Complete Safety Validation Pattern Coverage"
  slug: "safety-validation-patterns"

  classification: integrate_now
  confidence: 0.89
  milestone: "v1.1.0-beta"

  created_at: "2026-01-08T09:00:00Z"
  updated_at: "2026-01-08T09:00:00Z"
  stale_since: "2025-12-15T00:00:00Z"

  description: |
    Safety validation patterns have incomplete coverage for edge cases
    including nested quotes, timeout commands, and unicode escapes.
    A draft PR exists with additional patterns that was never merged.

  problem: |
    The safety validation layer is the critical security boundary for
    command execution. Gaps in pattern coverage could allow dangerous
    commands to execute.

  context: |
    Work on safety patterns has been ongoing since v1.0.0. The static
    matcher achieves 86% pass rate but known edge cases remain. PR #156
    was opened 23 days ago with additional patterns but stalled waiting
    for review.

  signals:
    - id: "SIG-001"
      type: todo_comment
      source: codebase
      location:
        file: "src/safety/patterns.rs"
        line: 142
      content: "TODO: Handle nested quotes in command arguments"
      severity: important
      category: security

    - id: "SIG-002"
      type: todo_comment
      source: codebase
      location:
        file: "src/safety/validator.rs"
        line: 89
      content: "TODO: Add timeout command patterns"
      severity: warning
      category: security

    - id: "SIG-003"
      type: stale_pr
      source: github
      location:
        url: "https://github.com/caro/caro/pull/156"
      content: "Draft: Additional safety patterns for edge cases"
      severity: important
      category: security

    - id: "SIG-004"
      type: stale_issue
      source: github
      location:
        url: "https://github.com/caro/caro/issues/134"
      content: "Safety bypass with unicode escape sequences"
      severity: important
      category: security

  signal_count: 4

  scope:
    files:
      - "src/safety/patterns.rs"
      - "src/safety/validator.rs"
      - "src/safety/mod.rs"
      - "tests/safety_tests.rs"
    file_count: 4
    estimated_lines: 180
    complexity: medium
    areas:
      - "Core CLI"
      - "Safety"

  risk:
    level: high
    if_ignored: "Potential security bypass through uncovered patterns"
    security_relevant: true
    user_facing: false
    breaking_change: false

  dependencies:
    blocks: []
    blocked_by: []
    related:
      - "MOD-2026-003"  # CLI error handling
    external: []

  origin:
    primary_source: github_issue
    trace:
      - timestamp: "2025-11-01T10:00:00Z"
        type: issue
        url: "https://github.com/caro/caro/issues/134"
        author: "security-researcher"
        title: "Safety bypass with unicode"
        excerpt: "Found that unicode escapes can bypass..."
        decision: null

      - timestamp: "2025-12-01T14:00:00Z"
        type: pr
        url: "https://github.com/caro/caro/pull/156"
        author: "contributor"
        title: "Additional safety patterns"
        excerpt: "Adds patterns for unicode, nested quotes..."
        decision: "Requested changes on Dec 5"
    contributors:
      - "security-researcher"
      - "contributor"
      - "maintainer"

  recommendation:
    action: integrate
    rationale: |
      Security-relevant work that aligns with the v1.1.0-beta focus on
      safety improvements. The PR is 80% ready and the TODOs are scoped.
      Low marginal cost to complete, high leverage for security posture.
    suggested_milestone: "v1.1.0-beta"
    effort_estimate: days
    priority_within_milestone: P1

  spec_seed:
    generated: true
    path: ".modulization/spec-seeds/MOD-2026-007.yaml"
    suggested_artifacts:
      - "ADR: Safety pattern architecture"
      - "Design: Pattern testing strategy"

  metadata:
    version: "1.0.0"
    generator: "modulization/1.0.0"
    cadence: "weekly-2026-01-08"
    tags:
      - "security"
      - "safety"
      - "high-priority"
```

---

## JSON Schema

For tooling integration, the full JSON Schema is available at:
`.modulization/schemas/module.schema.json`

# Signal Collectors Reference

This document details the signal collection strategies Modulization uses to discover unfinished, fragmented, and drifting work.

---

## Collection Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Signal Collection Pipeline                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   Codebase   │  │    GitHub    │  │   Process    │           │
│  │  Collector   │  │  Collector   │  │  Collector   │           │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘           │
│         │                  │                  │                  │
│         ▼                  ▼                  ▼                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Signal Normalizer                      │    │
│  │  - Deduplicate                                           │    │
│  │  - Enrich with metadata                                  │    │
│  │  - Link related signals                                  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Historical Tracer                      │    │
│  │  - Find origin of work                                   │    │
│  │  - Link to discussions, PRs, issues                      │    │
│  │  - Reconstruct decision timeline                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Signal Store                          │    │
│  │  - Normalized signals ready for module composition       │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Codebase Collector

Scans source code for signals indicating incomplete or deferred work.

### Pattern-Based Collection

#### TODO/FIXME Detection

```bash
# Primary patterns
patterns:
  - "TODO"
  - "FIXME"
  - "HACK"
  - "XXX"
  - "NOTE"
  - "BUG"
  - "OPTIMIZE"
  - "REFACTOR"

# Extended patterns (configurable)
extended_patterns:
  - "TEMPORARY"
  - "WORKAROUND"
  - "KLUDGE"
  - "REVIEW"
  - "NB:"
  - "DEPRECATED"
```

#### Collection Strategy

```yaml
codebase_scan:
  # File patterns to include
  include:
    - "**/*.rs"
    - "**/*.ts"
    - "**/*.tsx"
    - "**/*.js"
    - "**/*.jsx"
    - "**/*.py"
    - "**/*.go"
    - "**/*.java"
    - "**/*.md"
    - "**/*.yaml"
    - "**/*.yml"

  # Paths to exclude
  exclude:
    - "**/node_modules/**"
    - "**/vendor/**"
    - "**/target/**"
    - "**/dist/**"
    - "**/build/**"
    - "**/.git/**"
    - "**/test_fixtures/**"

  # Extraction method
  extraction:
    # Capture the full comment line plus context
    context_lines_before: 2
    context_lines_after: 2

    # Parse structured TODOs if present
    # Example: TODO(username): Description [tag]
    structured_pattern: |
      (?P<marker>TODO|FIXME|HACK|XXX|NOTE)
      (?:\((?P<author>[^)]+)\))?
      :?\s*
      (?P<content>.+?)
      (?:\[(?P<tags>[^\]]+)\])?
      $
```

### Semantic Analysis

Beyond pattern matching, analyze code structure:

#### Incomplete Implementations

```yaml
incomplete_impl_detection:
  # Rust patterns
  rust:
    - pattern: 'todo!()'
      type: explicit_todo_macro
    - pattern: 'unimplemented!()'
      type: unimplemented_marker
    - pattern: 'panic!("not implemented")'
      type: stub_panic
    - pattern: '// ... (more implementations)'
      type: partial_impl_comment

  # Python patterns
  python:
    - pattern: 'raise NotImplementedError'
      type: not_implemented
    - pattern: 'pass  # TODO'
      type: stub_pass
    - pattern: '...'
      type: ellipsis_stub

  # TypeScript patterns
  typescript:
    - pattern: 'throw new Error("Not implemented")'
      type: not_implemented
    - pattern: '// @ts-ignore'
      type: type_suppression
```

#### Dead Code Detection

```yaml
dead_code_detection:
  strategies:
    # Unreferenced public functions
    - name: unused_exports
      description: "Exported items with no internal or external references"
      severity: info

    # Commented-out code blocks
    - name: commented_code
      pattern: |
        (?:^|\n)
        (?:
          (?://|#|--|/\*)
          \s*
          (?:fn|function|def|class|const|let|var|pub|import)
        )
      severity: warning

    # Feature flags that are always false
    - name: dead_feature_flags
      pattern: 'if false \{|#\[cfg\(never\)\]'
      severity: info
```

#### Stale Comments

Detect comments that don't match surrounding code:

```yaml
stale_comment_detection:
  strategies:
    # Comments referencing non-existent identifiers
    - name: orphan_references
      description: "Comment mentions function/variable that doesn't exist"

    # Comments with outdated information
    - name: version_mismatch
      pattern: 'as of v[\d.]+|since version [\d.]+'
      check: compare_to_current_version

    # Issue/PR references to closed items
    - name: closed_issue_refs
      pattern: '#\d+'
      check: verify_issue_state
```

### Output Format

```yaml
codebase_signal:
  id: "CS-2026-001"
  type: todo_comment
  source: codebase

  location:
    file: "src/safety/validator.rs"
    line: 142
    column: 5

  content: "TODO: Handle nested quotes in command arguments"
  raw_match: "// TODO: Handle nested quotes in command arguments"

  context:
    before:
      - "139:     fn validate_quotes(&self, input: &str) -> Result<()> {"
      - "140:         // Basic quote validation"
      - "141:         if input.contains('\"') {"
    after:
      - "143:             return Err(QuoteError::Nested);"
      - "144:         }"
      - "145:         Ok(())"

  metadata:
    author: null  # Could not determine
    tags: []
    created_date: null  # From git blame
    last_modified: "2025-12-01"

  severity: important
  category: incomplete_feature
```

---

## GitHub Collector

Collects signals from GitHub's collaboration surfaces.

### PR Collection

```yaml
pr_collection:
  # Query parameters
  query:
    state:
      - open
      - closed  # Include closed-unmerged
    draft: include
    age_threshold_days: 14  # Stale after

  # Classification
  classify:
    stale_pr:
      condition: "age > stale_threshold AND no_recent_activity"
      severity: warning

    draft_pr:
      condition: "is_draft"
      severity: info

    closed_unmerged:
      condition: "state == closed AND NOT merged"
      severity: info

    blocked_pr:
      condition: "has_label('blocked') OR has_failing_checks"
      severity: warning

    needs_review:
      condition: "review_state == 'changes_requested' AND age > 7"
      severity: warning
```

### Issue Collection

```yaml
issue_collection:
  query:
    state: open
    age_threshold_days: 30

  classify:
    stale_issue:
      condition: "age > stale_threshold AND no_recent_comments"
      severity: info

    blocked_issue:
      condition: "has_label('blocked')"
      severity: warning

    needs_triage:
      condition: "no_labels OR has_label('needs-triage')"
      severity: info

    unassigned:
      condition: "no_assignees AND age > 7"
      severity: info
```

### Discussion Collection

```yaml
discussion_collection:
  query:
    categories:
      - "Ideas"
      - "RFC"
      - "Architecture"
    answered: false

  classify:
    unresolved_discussion:
      condition: "NOT answered AND age > 14"
      severity: info

    decision_pending:
      condition: "has_label('needs-decision')"
      severity: warning
```

### Review Comment Collection

```yaml
review_comment_collection:
  # Find unresolved review threads
  query:
    resolved: false
    pr_state: open

  classify:
    unresolved_review:
      condition: "thread.resolved == false"
      severity: warning

    stale_review:
      condition: "age > 7 AND no_response"
      severity: warning
```

### Branch Collection

```yaml
branch_collection:
  # Find abandoned branches
  query:
    exclude:
      - "main"
      - "master"
      - "develop"
      - "release/*"
    age_threshold_days: 30

  classify:
    abandoned_branch:
      condition: "no_commits_since(threshold)"
      severity: info

    stale_feature_branch:
      condition: "starts_with('feature/') AND no_commits_since(14)"
      severity: warning
```

### Output Format

```yaml
github_signal:
  id: "GH-2026-001"
  type: stale_pr
  source: github

  location:
    url: "https://github.com/owner/repo/pull/156"
    number: 156

  content:
    title: "Add additional safety patterns"
    body_excerpt: "This PR adds patterns for handling..."
    state: open
    draft: true
    labels:
      - "enhancement"
      - "safety"

  timeline:
    created_at: "2025-12-15T10:00:00Z"
    last_activity: "2025-12-20T14:00:00Z"
    age_days: 23

  participants:
    author: "contributor"
    reviewers:
      - "maintainer"
    assignees: []

  review_state:
    approved: false
    changes_requested: true
    pending: false
    comments:
      - author: "maintainer"
        body: "Needs test coverage for edge cases"
        resolved: false

  checks:
    passing: false
    failing:
      - "test-suite"

  metadata:
    head_ref: "feature/safety-patterns"
    base_ref: "main"
    commits: 3
    changed_files: 4
    additions: 150
    deletions: 10

  severity: important
  category: incomplete_feature
```

---

## Process Collector

Collects signals from development process artifacts.

### Worktree Collection

```yaml
worktree_collection:
  path: ".worktrees/"

  scan:
    # Check for incomplete specs
    spec_incomplete:
      file: "spec.md"
      condition: "contains('[ ]') OR missing_sections"

    # Check for incomplete tasks
    tasks_incomplete:
      file: "tasks.md"
      condition: "has_unchecked_items"

    # Check for stale worktree
    stale_worktree:
      condition: "no_git_activity(14)"
```

### Roadmap Collection

```yaml
roadmap_collection:
  file: "ROADMAP.md"

  scan:
    # Find incomplete items
    incomplete_items:
      pattern: '- \[ \]'

    # Find overdue items
    overdue:
      condition: "due_date < today AND NOT completed"

    # Find items without assignees
    unassigned:
      condition: "no_assignee AND due_date < 30_days"

    # Find gaps
    roadmap_gaps:
      condition: "known_work NOT in_roadmap"
```

### Spec Artifact Collection

```yaml
spec_collection:
  paths:
    - ".kittify/missions/**/*.md"
    - ".worktrees/**/spec.md"
    - ".worktrees/**/plan.md"
    - ".worktrees/**/tasks.md"

  scan:
    # Incomplete specs
    incomplete_spec:
      checks:
        - has_problem_statement
        - has_acceptance_criteria
        - has_user_scenarios

    # Stale specs
    stale_spec:
      condition: "modified_date > 30_days AND NOT implemented"

    # Orphan specs
    orphan_spec:
      condition: "NOT linked_to_pr AND NOT linked_to_issue"
```

### Constitution Collection

```yaml
constitution_collection:
  file: ".specify/memory/constitution.md"

  scan:
    # Violations in codebase
    violations:
      for_each_principle:
        check: codebase_adherence

    # Outdated principles
    outdated:
      condition: "referenced_code_changed AND principle_not_updated"
```

---

## Historical Tracer

Traces signals back to their origins to understand context and decisions.

### Git Blame Integration

```yaml
git_blame:
  # For each codebase signal, trace author and date
  for_signal:
    extract:
      - original_author
      - original_date
      - last_modifier
      - modification_history

  # Link to commits
  commit_extraction:
    message_parsing:
      issue_refs: '#\d+'
      pr_refs: 'PR #\d+'
      closes_refs: '(?:closes?|fixes?|resolves?)\s+#\d+'
```

### Cross-Reference Resolution

```yaml
cross_reference:
  # Link code TODOs to issues
  todo_to_issue:
    patterns:
      - "TODO.+#(?P<issue>\d+)"
      - "See issue #(?P<issue>\d+)"
    action: fetch_issue_context

  # Link PRs to issues
  pr_to_issue:
    sources:
      - pr_body_refs
      - pr_title_refs
      - linked_issues_api
    action: build_relationship_graph

  # Link discussions to PRs/issues
  discussion_to_work:
    patterns:
      - discussion_mentions_pr
      - discussion_mentions_issue
    action: link_conversation
```

### Decision Timeline Reconstruction

```yaml
decision_timeline:
  # For each module, reconstruct the decision history
  build_timeline:
    sources:
      - git_commits
      - issue_comments
      - pr_reviews
      - discussions

    extract:
      - decisions_made
      - decisions_deferred
      - blockers_identified
      - scope_changes

  output:
    format: "chronological event list"
    include:
      - timestamp
      - actor
      - event_type
      - content_summary
      - impact_on_work
```

### Output Format

```yaml
origin_trace:
  signal_id: "CS-2026-001"

  origin:
    type: issue
    id: 134
    url: "https://github.com/owner/repo/issues/134"
    created: "2025-11-01T10:00:00Z"
    author: "security-researcher"
    title: "Safety bypass with unicode escape sequences"

  timeline:
    - timestamp: "2025-11-01T10:00:00Z"
      type: issue_created
      actor: "security-researcher"
      summary: "Reported unicode escape bypass"

    - timestamp: "2025-11-05T14:00:00Z"
      type: comment
      actor: "maintainer"
      summary: "Confirmed issue, will address in v1.1.0"

    - timestamp: "2025-12-01T09:00:00Z"
      type: pr_opened
      actor: "contributor"
      ref: "PR #156"
      summary: "Draft PR with initial fix"

    - timestamp: "2025-12-05T11:00:00Z"
      type: review
      actor: "maintainer"
      summary: "Requested additional test coverage"
      decision: "Changes requested"

    - timestamp: "2025-12-20T14:00:00Z"
      type: commit
      actor: "original-author"
      ref: "abc123"
      summary: "Added TODO for nested quotes"

  stall_analysis:
    stalled_at: "2025-12-05T11:00:00Z"
    stall_reason: "Waiting for contributor response to review"
    days_stalled: 34
    blocker_type: "response_pending"

  related_work:
    - type: issue
      id: 89
      relationship: "related"
      title: "RTL support"

    - type: pr
      id: 150
      relationship: "depends_on"
      title: "Validator refactoring"
```

---

## Signal Normalization

After collection, signals are normalized for consistent processing.

### Deduplication

```yaml
deduplication:
  strategies:
    # Same content in different forms
    content_similarity:
      threshold: 0.85
      action: merge_signals

    # Same issue referenced multiple times
    reference_dedup:
      match: "same github reference"
      action: consolidate

    # TODO that references an issue
    todo_issue_link:
      condition: "todo mentions issue number"
      action: link_and_prefer_issue
```

### Enrichment

```yaml
enrichment:
  # Add ownership information
  ownership:
    source: "CODEOWNERS"
    add:
      - owners
      - area
      - team

  # Add roadmap context
  roadmap_context:
    source: "ROADMAP.md"
    add:
      - milestone_if_scheduled
      - priority_if_known

  # Add relationship context
  relationships:
    discover:
      - related_modules
      - blocking_work
      - blocked_by_work
```

### Linking

```yaml
linking:
  # Link signals that should be considered together
  strategies:
    # Same file proximity
    file_proximity:
      condition: "same file, within 50 lines"
      strength: strong

    # Same issue reference
    issue_reference:
      condition: "reference same issue"
      strength: strong

    # Same PR
    pr_grouping:
      condition: "from same PR"
      strength: strong

    # Same topic/keywords
    topic_similarity:
      condition: "NLP similarity > 0.7"
      strength: weak

    # Same author
    author_grouping:
      condition: "same author, same week"
      strength: weak
```

---

## Configuration

```yaml
# .modulization/config.yaml
signals:
  codebase:
    enabled: true
    patterns:
      - "TODO"
      - "FIXME"
      - "HACK"
    include:
      - "src/**"
      - "lib/**"
    exclude:
      - "vendor/**"
      - "node_modules/**"
    semantic_analysis: true
    dead_code_detection: true
    stale_comment_detection: true

  github:
    enabled: true
    stale_pr_days: 14
    stale_issue_days: 30
    include_draft_prs: true
    include_closed_unmerged: true
    collect_discussions: true
    collect_review_comments: true
    collect_branches: true

  process:
    enabled: true
    worktree_path: ".worktrees/"
    roadmap_path: "ROADMAP.md"
    spec_paths:
      - ".kittify/**"
    constitution_path: ".specify/memory/constitution.md"

  historical:
    enabled: true
    git_blame: true
    cross_reference: true
    timeline_reconstruction: true
    max_trace_depth: 10

  normalization:
    deduplicate: true
    enrich: true
    link: true
    similarity_threshold: 0.85
```

---

## Performance Considerations

### Caching

```yaml
caching:
  # Cache git blame results
  git_blame:
    ttl: 1_day
    invalidate_on: file_change

  # Cache GitHub API responses
  github:
    ttl: 1_hour
    rate_limit_aware: true

  # Cache cross-reference resolutions
  cross_refs:
    ttl: 6_hours
```

### Incremental Collection

```yaml
incremental:
  # Only scan changed files
  codebase:
    strategy: "git diff since last run"

  # Only fetch updated items
  github:
    strategy: "updated_since last_run"

  # Full scan on first run
  first_run:
    strategy: "full"
```

### Parallelization

```yaml
parallel:
  # Run collectors in parallel
  collectors: true

  # Parallelize file scanning
  file_scan:
    workers: 4

  # Parallelize GitHub API calls
  github_api:
    concurrent_requests: 5
```

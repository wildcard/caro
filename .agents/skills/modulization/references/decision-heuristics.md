# Decision Heuristics Reference

This document defines the decision framework Modulization uses to classify modules into actionable paths: **Integrate Now**, **Schedule Explicitly**, **Ice**, or **Archive**.

---

## Classification Framework

### Decision Tree Overview

```
                    ┌─────────────────┐
                    │  Evaluate Module │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Is it obsolete? │
                    └────────┬────────┘
                        Yes/ \No
                           /   \
            ┌─────────────▼     ▼─────────────┐
            │     ARCHIVE       │ Is it valid │
            │  Close with       │ but blocked?│
            │  context          └──────┬──────┘
            └───────────────       Yes/ \No
                                     /   \
                      ┌─────────────▼     ▼─────────────┐
                      │      ICE          │ Aligned with │
                      │  Document &       │ active work? │
                      │  defer            └──────┬──────┘
                      └───────────────       Yes/ \No
                                               /   \
                                ┌─────────────▼     ▼─────────────┐
                                │  Low marginal     │   SCHEDULE   │
                                │  cost?            │  Assign to   │
                                └──────┬──────      │  milestone   │
                                   Yes/ \No        └───────────────
                                     /   \
                      ┌─────────────▼     ▼─────────────┐
                      │ INTEGRATE NOW     │   SCHEDULE   │
                      │ Add to current    │  Assign to   │
                      │ sprint            │  milestone   │
                      └───────────────    └───────────────
```

---

## Classification Definitions

### 1. Integrate Now

**When to use:**
- Work is aligned with the active milestone
- Marginal cost of integration is low
- Work has high leverage (unblocks others, stabilizes system)
- Delay would increase future cost

**Characteristics:**
- Fits naturally into current sprint/iteration
- Doesn't fragment focus significantly
- Often "while we're in there" opportunities
- Security or stability improvements

**Confidence threshold:** ≥ 0.80

### 2. Schedule Explicitly

**When to use:**
- Work is important but would be disruptive now
- Has clear scope and can wait
- Fits better in a future milestone
- Needs preparation or sequencing

**Characteristics:**
- Creates a placeholder in the roadmap
- Has an assigned target milestone
- May spawn a spec for proper planning
- Often medium-to-large scope

**Confidence threshold:** ≥ 0.70

### 3. Ice (Intentional Pause)

**When to use:**
- Valid idea, wrong time
- Blocked by external factors
- Needs strategic decision first
- Resource constraints prevent action

**Characteristics:**
- Documented rationale for delay
- Tagged for periodic re-evaluation
- May have a "thaw" condition
- Not forgotten, just parked

**Confidence threshold:** ≥ 0.75 for the ice decision

### 4. Archive

**When to use:**
- Work is obsolete or superseded
- Problem no longer exists
- Already solved differently
- Invalid or out of scope

**Characteristics:**
- Context is preserved (why archived)
- Links to replacement work (if any)
- Closed but searchable
- Never silent deletion

**Confidence threshold:** ≥ 0.85

---

## Scoring Dimensions

Each module is evaluated across multiple dimensions that feed into classification:

### 1. Alignment Score (0-100)

How well does this work align with current priorities?

```
alignment_score = weighted_average(
  milestone_fit       × 0.35,  # Does it fit active milestone?
  roadmap_priority    × 0.25,  # Is it on the roadmap?
  strategic_value     × 0.25,  # Does it advance strategy?
  team_focus          × 0.15   # Does team have expertise active?
)
```

**Thresholds:**
- ≥ 80: Strong alignment → lean toward Integrate Now
- 50-79: Moderate alignment → consider Schedule
- < 50: Low alignment → consider Ice or Archive

### 2. Cost Score (0-100)

How expensive is this work to complete?

```
cost_score = weighted_average(
  scope_size          × 0.30,  # How many files/lines?
  complexity          × 0.30,  # Technical difficulty
  risk                × 0.20,  # Risk of introducing bugs
  dependencies        × 0.20   # External blockers
)
```

**Interpretation:**
- Lower cost + high alignment → Integrate Now
- Higher cost + high alignment → Schedule with proper spec
- High cost + low alignment → Ice

### 3. Urgency Score (0-100)

How time-sensitive is this work?

```
urgency_score = weighted_average(
  security_relevant   × 0.35,  # Security implications
  user_impact         × 0.25,  # User-facing effects
  blocking_others     × 0.20,  # Is blocking other work
  decay_rate          × 0.20   # Gets harder over time
)
```

**Thresholds:**
- ≥ 80: High urgency → accelerate toward Integrate Now
- 50-79: Moderate urgency → don't defer too long
- < 50: Low urgency → can wait

### 4. Freshness Score (0-100)

How active/stale is this work?

```
freshness_score = inverse_decay(
  days_since_last_activity,
  half_life = 30 days
)
```

**Impact:**
- Fresh work (> 80) → still actively relevant
- Moderate (50-80) → needs attention
- Stale (< 50) → may need re-evaluation or archive

---

## Decision Rules

### Rule 1: Security Override

```
IF risk.security_relevant == true AND risk.level >= high THEN
  classification = integrate_now
  priority = P1
  rationale = "Security-relevant work requires immediate attention"
```

### Rule 2: Natural Seam Detection

```
IF milestone_fit.current_files_overlap > 0.5 AND
   cost.complexity <= medium THEN
  classification = integrate_now
  rationale = "Natural alignment with files already being modified"
```

### Rule 3: Stale Work Triage

```
IF freshness_score < 30 AND
   alignment_score < 40 AND
   NOT risk.security_relevant THEN
  classification = archive
  rationale = "Stale work with low alignment, likely obsolete"
```

### Rule 4: Blocked Work

```
IF dependencies.blocked_by.length > 0 AND
   blocked_work.resolution_unclear THEN
  classification = ice
  thaw_condition = "Blocked by: {dependencies.blocked_by}"
  rationale = "Cannot proceed until blockers resolved"
```

### Rule 5: Scope Overflow

```
IF cost.complexity >= large AND
   NOT has_existing_spec THEN
  classification = schedule
  require_spec = true
  rationale = "Large scope requires proper specification before work"
```

### Rule 6: Quick Win Detection

```
IF cost.complexity == trivial AND
   alignment_score >= 60 AND
   urgency_score >= 40 THEN
  classification = integrate_now
  rationale = "Quick win that can be done opportunistically"
```

### Rule 7: Breaking Change Deferral

```
IF risk.breaking_change == true AND
   current_milestone.type != major_version THEN
  classification = schedule
  target_milestone = next_major_version
  rationale = "Breaking changes deferred to major version"
```

---

## Confidence Calculation

Classification confidence is calculated as:

```
confidence = min(
  dimension_agreement,   # How much dimensions agree
  signal_quality,        # Quality of input signals
  context_completeness   # How complete the picture is
)
```

### Dimension Agreement

When scoring dimensions point in the same direction:
- All dimensions agree → confidence += 0.20
- 3 of 4 agree → confidence += 0.10
- 2 of 4 agree → confidence += 0.00
- Dimensions conflict → confidence -= 0.10

### Signal Quality

Based on signal freshness and completeness:
- All signals < 30 days old → quality = 0.90
- Mix of fresh and stale → quality = 0.70
- Mostly stale signals → quality = 0.50

### Context Completeness

Based on available information:
- Full origin trace → completeness += 0.15
- GitHub context available → completeness += 0.15
- Related work identified → completeness += 0.10
- Roadmap position clear → completeness += 0.10

---

## Escalation Rules

Some situations require human decision:

### Automatic Escalation

```
IF confidence < 0.60 THEN
  escalate = true
  reason = "Low confidence in classification"

IF risk.level == critical AND classification != integrate_now THEN
  escalate = true
  reason = "Critical risk not classified for immediate action"

IF conflicting_signals.count > 3 THEN
  escalate = true
  reason = "Significant conflicting signals require human judgment"
```

### Escalation Output

When escalated, the module includes:

```yaml
escalation:
  required: true
  reason: "Low confidence in classification"
  dimensions:
    alignment: 55  # Explain conflict
    cost: 72
    urgency: 48
    freshness: 61
  options:
    - classification: integrate_now
      confidence: 0.45
      rationale: "If security concern is valid..."
    - classification: schedule
      confidence: 0.55
      rationale: "If scope is as estimated..."
  recommended_questions:
    - "Is the security concern in Issue #134 still valid?"
    - "Has the scope changed since PR #156 was opened?"
```

---

## Milestone Placement Heuristics

When scheduling, choose the milestone based on:

### Milestone Fit Score

```
milestone_fit(module, milestone) = weighted_average(
  scope_fit           × 0.30,  # Does module fit milestone's scope budget?
  theme_alignment     × 0.25,  # Does it match milestone's themes?
  dependency_sat      × 0.25,  # Are dependencies met by milestone?
  timing_appropriate  × 0.20   # Is the timing right?
)
```

### Placement Rules

```
# Prefer milestone with highest fit score above threshold
target = milestones
  .filter(m => milestone_fit(module, m) >= 0.60)
  .sort_by(milestone_fit)
  .first()

# If no milestone fits, create placeholder
IF target == null THEN
  target = create_placeholder_milestone(module)
  escalate("No fitting milestone, placeholder created")
```

---

## Re-Evaluation Triggers

Iced modules are re-evaluated when:

1. **Time-based**: Every N cadence runs (configurable, default 4)
2. **Event-based**:
   - Related work is merged
   - Blocking dependency resolved
   - Roadmap changes
   - Milestone approaches
3. **Manual**: Explicit re-evaluation requested

### Thaw Conditions

```yaml
ice_record:
  module_id: "MOD-2026-003"
  iced_at: "2026-01-08"
  iced_reason: "Blocked by authentication redesign"
  thaw_conditions:
    - event: "pr_merged"
      pr: "#200"
      description: "Auth redesign complete"
    - event: "milestone_reached"
      milestone: "v1.2.0"
      description: "After v1.2.0 release"
    - event: "manual"
      description: "Team decides to prioritize"
  next_review: "2026-02-05"  # 4 weeks from icing
```

---

## Configuration Overrides

Classification can be influenced by configuration:

```yaml
# .modulization/config.yaml
classification:
  # Labels that force specific classifications
  auto_integrate:
    - label: "security"
      condition: "risk.level >= medium"
    - label: "quick-fix"
      condition: "cost.complexity <= small"

  auto_schedule:
    - label: "enhancement"
      target_milestone: "next_minor"
    - label: "breaking-change"
      target_milestone: "next_major"

  auto_ice:
    - label: "blocked"
    - label: "needs-design"
    - label: "wontfix"

  auto_archive:
    - label: "duplicate"
    - label: "invalid"
    - label: "out-of-scope"

  # Thresholds
  thresholds:
    integrate_now_alignment: 75
    schedule_alignment: 50
    stale_days: 30
    confidence_minimum: 0.60

  # Escalation
  escalation:
    always_escalate_critical: true
    confidence_threshold: 0.60
    max_conflicting_signals: 3
```

---

## Examples

### Example 1: Clear Integrate Now

```yaml
module:
  title: "Fix typo in CLI help"
  signals:
    - type: todo_comment
      content: "TODO: Fix typo 'recieve' -> 'receive'"
      age_days: 5

scoring:
  alignment: 85    # Active work on CLI
  cost: 5          # Trivial change
  urgency: 40      # User-visible but minor
  freshness: 95    # Very recent

classification: integrate_now
confidence: 0.92
rationale: "Trivial fix, active work area, no disruption"
```

### Example 2: Clear Schedule

```yaml
module:
  title: "Add i18n support"
  signals:
    - type: stale_issue
      content: "Feature request: Internationalization"
      age_days: 90
    - type: todo_comment
      content: "TODO: Add locale detection"
      age_days: 45

scoring:
  alignment: 45    # Not in current milestone
  cost: 75         # Large scope
  urgency: 35      # Nice to have
  freshness: 55    # Moderate staleness

classification: schedule
target_milestone: "v1.2.0"
confidence: 0.85
rationale: "Important feature, large scope, needs proper planning"
require_spec: true
```

### Example 3: Clear Ice

```yaml
module:
  title: "Database migration to new ORM"
  signals:
    - type: draft_pr
      content: "WIP: Migrate to new ORM"
      age_days: 60
    - type: discussion_unresolved
      content: "Should we switch ORMs?"

scoring:
  alignment: 30    # Not aligned with current work
  cost: 90         # Major undertaking
  urgency: 20      # Not urgent
  freshness: 40    # Getting stale

classification: ice
thaw_condition: "Strategic decision on ORM change"
confidence: 0.88
rationale: "Blocked by unresolved architectural decision"
```

### Example 4: Clear Archive

```yaml
module:
  title: "Add support for deprecated API v1"
  signals:
    - type: stale_issue
      content: "Support API v1"
      age_days: 180
    - type: closed_unmerged_pr
      content: "API v1 support"
      closed_reason: "superseded"

scoring:
  alignment: 5     # Not relevant anymore
  cost: 60         # Medium effort
  urgency: 5       # No longer needed
  freshness: 15    # Very stale

classification: archive
confidence: 0.95
rationale: "API v1 was deprecated 6 months ago, work superseded"
linked_to: "PR #250 removed API v1 support"
```

### Example 5: Escalation Needed

```yaml
module:
  title: "Improve error handling in backend"
  signals:
    - type: todo_comment
      content: "TODO: Better error messages"
      age_days: 30
    - type: stale_pr
      content: "Improve error handling"
      age_days: 25
    - type: stale_issue
      content: "Error messages are confusing"
      age_days: 60

scoring:
  alignment: 55    # Some overlap
  cost: 50         # Medium
  urgency: 55      # User-facing
  freshness: 70    # Moderate

# Dimensions don't clearly agree
classification: null  # Escalated
escalation:
  required: true
  reason: "Dimensions do not clearly indicate path"
  options:
    - integrate_now (0.45)
    - schedule (0.48)
  questions:
    - "Is error handling a priority for v1.1.0-beta?"
    - "Can the PR be completed quickly or needs rework?"
```

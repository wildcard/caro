# ADR-011: Agentic Idea Pipeline for Continuous Product Discovery

**Status**: Proposed
**Date**: 2026-01-11
**Author**: Product Strategy
**Decision Makers**: Product Lead, Engineering Lead

---

## Context

Open source projects face a constant challenge: staying relevant while maintaining focus. Ideas come from:
- Competitor movements
- Community requests
- Technology trends
- User pain points

Currently, discovering and vetting ideas is manual, sporadic, and disconnected from the development workflow. We need a systematic approach that:
1. Continuously monitors relevant signals
2. Translates signals into project-relevant ideas
3. Critically evaluates ideas for fit and novelty
4. Integrates approved ideas into the roadmap
5. Minimizes human effort while maximizing human judgment where it matters

---

## Decision

Implement an **Agentic Idea Pipeline** that uses streaming data infrastructure and LLM-based agents to automate product discovery while keeping humans in the strategic decision loop.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            SIGNAL LAYER                                          │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────────┐   │
│  │  RSS   │ │ Reddit │ │BlueSky │ │Twitter │ │ HN/Dev │ │ Perplexity/Search│   │
│  │ Feeds  │ │  API   │ │Firehose│ │   X    │ │  News  │ │   (On-Demand)    │   │
│  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └────────┬─────────┘   │
│      │          │          │          │          │                │             │
│      └──────────┴──────────┴────┬─────┴──────────┴────────────────┘             │
│                                 │                                                │
│                                 ▼                                                │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │              STREAMING BACKBONE (RedPanda / Kafka)                      │    │
│  │  Topics:                                                                 │    │
│  │  - raw-signals        (all incoming data)                               │    │
│  │  - enriched-signals   (deduplicated, clustered)                         │    │
│  │  - candidate-ideas    (translated to project context)                   │    │
│  │  - vetted-ideas       (approved by critical agent)                      │    │
│  │  - rejected-ideas     (archived with reasoning)                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         PROCESSING LAYER                                         │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                    WINDOW PROCESSOR AGENT                                 │   │
│  │                                                                           │   │
│  │  Input: Time-windowed signals (15min, 1hr, daily)                        │   │
│  │                                                                           │   │
│  │  Operations:                                                              │   │
│  │  1. Deduplication (same story from multiple sources)                     │   │
│  │  2. Clustering (related signals grouped)                                 │   │
│  │  3. Trend detection (volume/velocity analysis)                           │   │
│  │  4. Derivative query generation (deep-dive on interesting clusters)      │   │
│  │                                                                           │   │
│  │  Output: Clustered signals with trend scores                             │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                       │                                          │
│                                       ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                    IDEA SYNTHESIS AGENT                                   │   │
│  │                                                                           │   │
│  │  Context Injection:                                                       │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │   │
│  │  │   ROADMAP    │ │  CODEBASE    │ │   EXISTING   │ │    ANTI-     │     │   │
│  │  │   v1.1-v2.0  │ │  Embeddings  │ │   ISSUES     │ │    GOALS     │     │   │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘     │   │
│  │                                                                           │   │
│  │  Prompt Template:                                                         │   │
│  │  """                                                                      │   │
│  │  You are analyzing trends for Caro, a privacy-first CLI tool that        │   │
│  │  converts natural language to shell commands using local LLMs.           │   │
│  │                                                                           │   │
│  │  Current roadmap focus: {milestone_summary}                               │   │
│  │  Anti-goals (never pursue): {anti_goals}                                  │   │
│  │                                                                           │   │
│  │  Given these signals: {clustered_signals}                                 │   │
│  │                                                                           │   │
│  │  Generate candidate ideas that:                                           │   │
│  │  1. Align with project mission (local-first, safe, fast)                 │   │
│  │  2. Address real user pain points evident in signals                     │   │
│  │  3. Are technically feasible given our Rust/MLX stack                    │   │
│  │  """                                                                      │   │
│  │                                                                           │   │
│  │  Output: Candidate ideas with relevance scores                            │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                       │                                          │
│                                       ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                    CRITICAL EVALUATION AGENT                              │   │
│  │                                                                           │   │
│  │  Decision Criteria:                                                       │   │
│  │                                                                           │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐     │   │
│  │  │              NOVELTY vs PROJECT FIT MATRIX                      │     │   │
│  │  │                                                                 │     │   │
│  │  │                      HIGH PROJECT FIT                           │     │   │
│  │  │                            │                                    │     │   │
│  │  │           ┌────────────────┼────────────────┐                   │     │   │
│  │  │           │   STRATEGIC    │   QUICK WIN    │                   │     │   │
│  │  │           │   (v1.4+)      │   (next PR)    │                   │     │   │
│  │  │           │   Accept:Plan  │   Accept:Now   │                   │     │   │
│  │  │  LOW ─────┼────────────────┼────────────────┼───── HIGH         │     │   │
│  │  │  NOVELTY  │   REJECT       │   RESEARCH     │     NOVELTY       │     │   │
│  │  │           │   (archive)    │   (explore)    │                   │     │   │
│  │  │           │                │                │                   │     │   │
│  │  │           └────────────────┼────────────────┘                   │     │   │
│  │  │                            │                                    │     │   │
│  │  │                      LOW PROJECT FIT                            │     │   │
│  │  └─────────────────────────────────────────────────────────────────┘     │   │
│  │                                                                           │   │
│  │  Checks:                                                                  │   │
│  │  - Is this already in our roadmap? → Reject as duplicate                 │   │
│  │  - Does it violate anti-goals? → Reject with explanation                 │   │
│  │  - Is it technically infeasible? → Reject or defer to research           │   │
│  │  - Has this been rejected before? → Check reasoning, maybe retry         │   │
│  │                                                                           │   │
│  │  CRITICAL: Must be DECISIVE. Default to REJECT.                          │   │
│  │                                                                           │   │
│  │  Output: ACCEPT / REJECT / NEEDS-HUMAN-REVIEW                             │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                       │                                          │
│                    ┌──────────────────┼──────────────────┐                       │
│                    ▼                  ▼                  ▼                       │
│              [REJECTED]        [NEEDS-REVIEW]      [ACCEPTED]                    │
│                    │                  │                  │                       │
│                    ▼                  │                  ▼                       │
│              Archive with             │     ┌────────────────────────────────┐   │
│              reasoning                │     │  CATEGORIZATION & PLANNING     │   │
│              (learning data)          │     │                                │   │
│                                       │     │  Assigns to:                   │   │
│                                       │     │  - Milestone (v1.2, v1.3, v2)  │   │
│                                       │     │  - Priority (P0-P3)            │   │
│                                       │     │  - Effort (T-shirt size)       │   │
│                                       │     │  - Dependencies                │   │
│                                       │     │                                │   │
│                                       │     │  Generates:                    │   │
│                                       │     │  - Draft PRD/Spec              │   │
│                                       │     │  - Acceptance criteria         │   │
│                                       │     │  - Implementation hints        │   │
│                                       │     └────────────────────────────────┘   │
│                                       │                  │                       │
└───────────────────────────────────────│──────────────────│───────────────────────┘
                                        │                  │
                                        ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         SANDBOX LAYER (Optional)                                 │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                    IMPLEMENTATION AGENT                                   │   │
│  │                                                                           │   │
│  │  Environment: Isolated git worktree with full codebase                   │   │
│  │                                                                           │   │
│  │  Actions:                                                                 │   │
│  │  1. Creates feature branch                                               │   │
│  │  2. Implements proof-of-concept                                          │   │
│  │  3. Runs test suite                                                      │   │
│  │  4. Generates implementation report                                      │   │
│  │                                                                           │   │
│  │  Output:                                                                  │   │
│  │  - Draft PR (if implementation successful)                               │   │
│  │  - Implementation confidence score                                       │   │
│  │  - Technical notes for human review                                      │   │
│  │                                                                           │   │
│  │  CRITICAL: Sandboxed execution, cannot push to main                      │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         HUMAN REVIEW INTERFACE                                   │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                    IDEA REVIEW DASHBOARD                                  │   │
│  │                                                                           │   │
│  │  For each idea, presents:                                                 │   │
│  │                                                                           │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐     │   │
│  │  │  IDEA: Add voice input for shell commands                       │     │   │
│  │  │                                                                 │     │   │
│  │  │  Source: 3 HN threads, 2 Reddit posts, 1 BlueSky viral post    │     │   │
│  │  │  Trend: ↑ 340% mention increase in past week                   │     │   │
│  │  │                                                                 │     │   │
│  │  │  Agent Recommendation: ACCEPT (Strategic - v1.4)               │     │   │
│  │  │  Confidence: 0.87                                              │     │   │
│  │  │                                                                 │     │   │
│  │  │  Reasoning:                                                    │     │   │
│  │  │  - Aligns with accessibility goals                             │     │   │
│  │  │  - Already in v2.0 roadmap as "Voice Command Support"          │     │   │
│  │  │  - High community demand suggests accelerating                 │     │   │
│  │  │                                                                 │     │   │
│  │  │  Draft PRD: [View] | Implementation PoC: [View PR #432]        │     │   │
│  │  │                                                                 │     │   │
│  │  │  Actions: [Approve] [Reject] [Defer] [Request Changes]         │     │   │
│  │  └─────────────────────────────────────────────────────────────────┘     │   │
│  │                                                                           │   │
│  │  Human Decision → Updates roadmap → Closes feedback loop                  │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Query Strategy

### Base Queries (Always Running)

These queries run continuously to capture the broader landscape:

```yaml
queries:
  # Direct project mentions
  - topic: "caro-mentions"
    sources: [twitter, reddit, bluesky, hn]
    query: "caro cli OR caro-cli OR caro shell"

  # Competitor landscape
  - topic: "competitors"
    sources: [twitter, reddit, hn]
    query: "warp terminal OR fig ai OR github copilot cli OR amazon q cli"

  # Technology space
  - topic: "cli-ai"
    sources: [rss, hn, reddit]
    query: "natural language cli OR ai terminal OR llm shell"

  # Pain points
  - topic: "pain-points"
    sources: [reddit, twitter, stackoverflow]
    query: "shell command help OR terminal frustration OR forgot command"

  # Local LLM trends
  - topic: "local-llm"
    sources: [hn, reddit, rss]
    query: "mlx llm OR local inference OR on-device ai"
```

### Derivative Queries (Generated from Signals)

When the Window Processor detects interesting clusters, it generates follow-up queries:

```python
def generate_derivative_queries(cluster):
    """Generate deep-dive queries based on signal clusters."""

    if cluster.type == "competitor_feature":
        return f"Why is {cluster.competitor} adding {cluster.feature}? User reactions?"

    if cluster.type == "pain_point":
        return f"Solutions for {cluster.pain_point}? Existing tools? Workarounds?"

    if cluster.type == "technology_trend":
        return f"{cluster.technology} CLI applications? Integration patterns?"

    if cluster.type == "user_request":
        return f"Prior art for {cluster.request}? Implementation complexity?"
```

---

## Project Context Integration

### Context Files

The Idea Synthesis Agent needs access to project knowledge:

```yaml
context_sources:
  # Roadmap and planning
  - file: ROADMAP.md
    purpose: "Current milestones and priorities"

  - file: .claude/releases/v1.3-v1.4-vision-roadmap.md
    purpose: "Long-term vision and strategic themes"

  # Codebase understanding
  - embeddings: caro-codebase-index
    purpose: "Code search for feasibility checks"

  # Existing work
  - github: issues
    purpose: "Avoid duplicating existing requests"

  - github: closed-issues
    purpose: "Learn from rejected ideas"

  # Anti-patterns
  - file: docs/development/ANTI_GOALS.md  # To be created
    purpose: "What we explicitly won't do"
```

### Anti-Goals Definition

Critical for the Critical Evaluation Agent:

```yaml
anti_goals:
  - "Cloud-only features (must work offline)"
  - "Paid API dependencies for core functionality"
  - "Complex installation (must be single binary)"
  - "Telemetry that can't be disabled"
  - "Features that compromise safety"
  - "Lock-in to specific LLM providers"
```

---

## Infrastructure Options

### Option A: Lightweight (MVP)

No streaming infrastructure. Suitable for low-volume experimentation.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   GitHub    │────▶│   Python    │────▶│   GitHub    │
│   Actions   │     │   Scripts   │     │   Issues    │
│   (cron)    │     │  + OpenAI   │     │   (output)  │
└─────────────┘     └─────────────┘     └─────────────┘

Schedule: Every 6 hours
Processing: Batch LLM calls
Output: GitHub Issues with labels
```

**Pros**: Simple, no infra cost, works today
**Cons**: Higher latency, less sophisticated deduplication

### Option B: RedPanda + Bytewax

Real-time processing with proper streaming semantics.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Signal    │────▶│  RedPanda   │────▶│  Bytewax    │────▶│   Review    │
│   Sources   │     │  (Kafka)    │     │  (Python)   │     │   Dashboard │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘

Latency: Near real-time (seconds)
Processing: Windowed stream processing
Output: Dashboard + GitHub integration
```

**Pros**: True real-time, better deduplication, scalable
**Cons**: More complex, requires hosting

### Option C: Confluent Cloud (Managed)

Fully managed streaming with connectors.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Signal    │────▶│ Confluent   │────▶│   ksqlDB    │────▶│    API      │
│   Sources   │     │   Cloud     │     │  + Flink    │     │   Gateway   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘

Latency: Near real-time
Processing: SQL-based stream processing
Output: REST API for dashboard
```

**Pros**: Fully managed, rich connectors, enterprise-grade
**Cons**: Cost at scale, vendor lock-in

### Recommended: Start with A, Migrate to B

1. **Phase 1**: GitHub Actions + Python scripts (validate the concept)
2. **Phase 2**: Add RedPanda when volume/latency requires it
3. **Phase 3**: Consider Confluent only if enterprise features needed

---

## Agent Specifications

### 1. Window Processor Agent

**Input**: Raw signals from streaming backbone
**Output**: Clustered signals with trend metadata

```python
class WindowProcessorAgent:
    """Processes time-windowed signals into clusters."""

    def process_window(self, signals: List[Signal], window: TimeWindow) -> List[Cluster]:
        # Step 1: Deduplicate
        unique = self.deduplicate(signals)

        # Step 2: Cluster similar signals
        clusters = self.cluster_by_semantic_similarity(unique)

        # Step 3: Score trends
        for cluster in clusters:
            cluster.trend_score = self.calculate_trend(cluster, window)
            cluster.velocity = self.calculate_velocity(cluster)

        # Step 4: Generate derivative queries for interesting clusters
        for cluster in clusters:
            if cluster.trend_score > THRESHOLD:
                cluster.derivative_queries = self.generate_queries(cluster)

        return clusters
```

### 2. Idea Synthesis Agent

**Input**: Clustered signals + project context
**Output**: Candidate ideas with relevance scores

```python
class IdeaSynthesisAgent:
    """Translates signal clusters into project-relevant ideas."""

    def __init__(self, project_context: ProjectContext):
        self.context = project_context
        self.llm = get_llm("claude-3-sonnet")  # Or local model

    def synthesize(self, clusters: List[Cluster]) -> List[CandidateIdea]:
        ideas = []

        for cluster in clusters:
            prompt = self.build_synthesis_prompt(cluster)
            response = self.llm.generate(prompt)

            idea = CandidateIdea(
                title=response.title,
                description=response.description,
                source_signals=cluster.signals,
                relevance_score=response.relevance,
                implementation_hints=response.hints,
            )
            ideas.append(idea)

        return ideas

    def build_synthesis_prompt(self, cluster: Cluster) -> str:
        return f"""
        Project: {self.context.name}
        Mission: {self.context.mission}
        Current Focus: {self.context.current_milestone}

        Anti-Goals (do not suggest ideas that require):
        {self.context.anti_goals}

        Signal Cluster:
        {cluster.summary}

        Source signals:
        {cluster.signals}

        Task: Generate a concrete product idea for {self.context.name} that:
        1. Addresses the user need evident in these signals
        2. Aligns with our mission and current focus
        3. Does NOT violate any anti-goals
        4. Is technically feasible

        Output format:
        - Title: (brief, action-oriented)
        - Description: (2-3 sentences)
        - Why Now: (why this matters for signals)
        - Relevance Score: (0.0-1.0)
        - Implementation Hints: (bullet points)
        """
```

### 3. Critical Evaluation Agent

**Input**: Candidate ideas
**Output**: ACCEPT / REJECT / NEEDS-HUMAN-REVIEW + reasoning

```python
class CriticalEvaluationAgent:
    """Decisively evaluates ideas against project criteria."""

    def __init__(self, project_context: ProjectContext):
        self.context = project_context
        self.llm = get_llm("claude-3-opus")  # Higher capability for judgment

    def evaluate(self, idea: CandidateIdea) -> EvaluationResult:
        # Check against existing roadmap
        if self.is_duplicate(idea):
            return EvaluationResult(
                decision="REJECT",
                reason="Already in roadmap or backlog",
                confidence=0.95
            )

        # Check against anti-goals
        anti_goal_violation = self.check_anti_goals(idea)
        if anti_goal_violation:
            return EvaluationResult(
                decision="REJECT",
                reason=f"Violates anti-goal: {anti_goal_violation}",
                confidence=0.99
            )

        # LLM evaluation for novelty and fit
        evaluation = self.llm_evaluate(idea)

        # CRITICAL: Default to reject, high bar for acceptance
        if evaluation.fit_score < 0.7 or evaluation.novelty_score < 0.5:
            return EvaluationResult(
                decision="REJECT",
                reason=evaluation.reasoning,
                confidence=evaluation.confidence
            )

        # High fit + high novelty = quick win
        if evaluation.fit_score > 0.8 and evaluation.novelty_score > 0.7:
            return EvaluationResult(
                decision="ACCEPT",
                category="quick-win",
                reason=evaluation.reasoning,
                confidence=evaluation.confidence
            )

        # High fit + moderate novelty = strategic (later milestone)
        if evaluation.fit_score > 0.7:
            return EvaluationResult(
                decision="ACCEPT",
                category="strategic",
                reason=evaluation.reasoning,
                confidence=evaluation.confidence
            )

        # Uncertain cases go to human review
        return EvaluationResult(
            decision="NEEDS-HUMAN-REVIEW",
            reason="Borderline scores - human judgment needed",
            confidence=evaluation.confidence
        )
```

### 4. Categorization Agent

**Input**: Accepted ideas
**Output**: Milestone assignment + draft PRD

```python
class CategorizationAgent:
    """Assigns ideas to milestones and generates PRDs."""

    def categorize(self, idea: CandidateIdea, evaluation: EvaluationResult) -> CategorizedIdea:
        # Determine milestone based on category and current roadmap
        milestone = self.assign_milestone(idea, evaluation)

        # Estimate effort
        effort = self.estimate_effort(idea)

        # Check dependencies
        dependencies = self.identify_dependencies(idea)

        # Generate draft PRD
        prd = self.generate_prd(idea, milestone, effort, dependencies)

        return CategorizedIdea(
            idea=idea,
            milestone=milestone,
            effort=effort,
            priority=self.calculate_priority(idea, evaluation),
            dependencies=dependencies,
            prd=prd
        )

    def assign_milestone(self, idea: CandidateIdea, evaluation: EvaluationResult) -> str:
        if evaluation.category == "quick-win":
            return self.context.next_milestone  # e.g., "v1.1.1"
        elif evaluation.category == "strategic":
            return self.context.strategic_milestone  # e.g., "v1.3" or "v2.0"
        else:
            return "backlog"
```

---

## Human-in-the-Loop Design

### Where Humans Add Value

| Stage | Human Role | Automation Role |
|-------|-----------|-----------------|
| **Query Definition** | Define what topics matter | Execute queries continuously |
| **Signal Processing** | Review edge cases | Dedupe, cluster, trend detection |
| **Idea Synthesis** | Provide project context | Generate candidate ideas |
| **Critical Evaluation** | Final veto/approval | Filter obvious rejects, flag borderline |
| **Categorization** | Validate milestone fit | Suggest placement, generate PRD |
| **Implementation** | Code review, merge | Draft PR, run tests |

### Review Dashboard Requirements

The dashboard should minimize human cognitive load:

1. **Pre-filtered**: Only show ideas that passed critical evaluation
2. **Contextualized**: Show source signals and reasoning
3. **Actionable**: Clear approve/reject/defer buttons
4. **Batchable**: Allow bulk actions for similar ideas
5. **Traceable**: Link decisions back to outcomes

### Suggested Review Cadence

```yaml
review_schedule:
  quick_wins:
    frequency: daily
    reviewer: engineering-lead
    time_budget: 15min

  strategic_ideas:
    frequency: weekly
    reviewer: product-lead
    time_budget: 30min

  needs_human_review:
    frequency: bi-weekly
    reviewer: both
    time_budget: 1hr
```

---

## Implementation Phases

### Phase 1: Manual Validation (2 weeks)

**Goal**: Validate the concept without infrastructure.

- [ ] Define base queries
- [ ] Manually collect signals (RSS reader, Twitter/BlueSky follows)
- [ ] Use Claude to synthesize ideas from collected signals
- [ ] Manually evaluate against roadmap
- [ ] Track acceptance/rejection in a spreadsheet

**Success Criteria**:
- 10+ ideas generated
- 2+ ideas accepted into roadmap
- Process documented

### Phase 2: Semi-Automated Pipeline (4 weeks)

**Goal**: Automate signal collection and initial processing.

- [ ] GitHub Actions workflow for signal collection
- [ ] Python scripts for API calls (Reddit, HN, RSS)
- [ ] LLM-based synthesis and evaluation
- [ ] Output to GitHub Issues with labels
- [ ] Weekly human review

**Success Criteria**:
- Pipeline runs automatically daily
- <1hr/week human review time
- 1+ idea accepted per month

### Phase 3: Full Pipeline (8 weeks)

**Goal**: Real-time processing with dashboard.

- [ ] Deploy RedPanda for streaming
- [ ] Implement all agent components
- [ ] Build review dashboard
- [ ] Add sandbox implementation agent
- [ ] Integrate with roadmap tooling

**Success Criteria**:
- Near real-time signal processing
- <30min/week human review time
- Measurable impact on product direction

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Signal noise overwhelming** | High | Medium | Aggressive filtering, high rejection threshold |
| **LLM hallucinations in synthesis** | Medium | Medium | Require source attribution, human verification |
| **Anti-goal drift** | Low | High | Regular anti-goal review, hard-coded checks |
| **Over-automation** | Medium | Medium | Keep human in critical path, not just review |
| **API rate limits/costs** | Medium | Low | Caching, batching, local model fallbacks |
| **Idea quality regression** | Low | Medium | Track acceptance rate, feedback loop to agents |

---

## Success Metrics

### Pipeline Health

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Signal volume** | 100+/day | Count raw signals |
| **Cluster quality** | >80% relevant | Sample review |
| **Idea generation rate** | 10+/week | Count candidates |
| **Rejection rate** | >90% | Track decisions |
| **Human review time** | <1hr/week | Time tracking |

### Outcome Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Ideas accepted** | 2+/month | Track approvals |
| **Ideas implemented** | 1+/quarter | Track PRs |
| **Roadmap influence** | 10% of features | Attribution |
| **Time to detection** | <48hr | Signal → idea timing |

---

## Future Extensions

1. **Competitive Intelligence Dashboard**: Dedicated view for competitor movements
2. **Community Sentiment Tracking**: Beyond ideas, track overall sentiment
3. **Predictive Trends**: ML model to predict which trends will grow
4. **Contributor Matching**: Suggest ideas to potential contributors
5. **A/B Testing Integration**: Test feature ideas before full implementation

---

## References

- [RedPanda Documentation](https://docs.redpanda.com/)
- [Confluent Platform](https://docs.confluent.io/)
- [Bytewax (Python Stream Processing)](https://bytewax.io/)
- [LangChain Agents](https://python.langchain.com/docs/modules/agents/)
- [GitHub Actions Scheduled Workflows](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#schedule)

---

## Decision

**Approved**: Proceed with Phase 1 (Manual Validation) to validate concept before infrastructure investment.

**Next Steps**:
1. Create `docs/development/ANTI_GOALS.md`
2. Define initial query set
3. Set up manual signal collection process
4. Schedule first weekly review session

---

## Appendix: Sample Output

### Example: Generated Idea from Pipeline

```yaml
idea:
  id: "AIP-2026-01-042"
  title: "Add fish shell native integration"

  source_signals:
    - source: reddit
      subreddit: r/fishshell
      title: "Anyone using AI CLI tools with fish?"
      score: 234
      date: "2026-01-09"

    - source: twitter
      author: "@fishdev"
      text: "Would love to see caro support fish properly"
      likes: 89
      date: "2026-01-10"

    - source: hn
      title: "Fish shell is underrated"
      comments_mentioning_ai_cli: 3
      date: "2026-01-08"

  synthesis:
    description: |
      Add native fish shell integration including autosuggestions,
      keybindings, and proper escaping for fish syntax.

    why_now: |
      Growing fish community expressing interest.
      Fish's autosuggestion paradigm aligns well with caro's UX.

    relevance_score: 0.82

  evaluation:
    decision: "ACCEPT"
    category: "strategic"
    fit_score: 0.85
    novelty_score: 0.65
    reasoning: |
      - Fish shell support already mentioned in v1.3 roadmap
      - Strong community signal suggests accelerating
      - Technical lift moderate (fish has different syntax)
      - Aligns with "everyone, everywhere" theme

  categorization:
    milestone: "v1.3.0"
    priority: "P2"
    effort: "M"
    dependencies: ["shell-integration-framework"]

  prd_draft: |
    ## Fish Shell Native Integration

    ### Problem
    Fish shell users cannot fully leverage caro due to:
    - Incorrect escaping for fish syntax
    - No autosuggestion integration
    - Missing keybinding support

    ### Solution
    Native fish integration that:
    1. Detects fish shell automatically
    2. Generates fish-compatible commands
    3. Integrates with fish's autosuggestion system
    4. Provides `caro integrate fish` command

    ### Success Criteria
    - [ ] Fish shell detected automatically
    - [ ] Commands generated with correct fish syntax
    - [ ] Autosuggestions appear inline
    - [ ] Tab completion works
```

---

**Document Version**: 1.0
**Last Updated**: 2026-01-11

# Caro Market Opportunities -- Prioritized Task Plan
# Generated: May 13, 2026 (from weekly market scan)
# Status: READY FOR ASSIGNMENT

## Executive Summary

Five opportunities extracted from the May 13 weekly market scan.
Two are "Now" priority (small effort), three are "Next" priority (medium-large).
Key finding: the tiered decision pipeline (SafetyDecision) was prototyped in
commits 65e59771 and b0588c46 but is NOT on main -- the code needs to be
resurrected or rewritten. The CVE pattern infrastructure already exists
(src/safety/cve_patterns.rs, data/cve_rules/). No OWASP or MCP work exists yet.

---

## Task A: SafetyDecision Structured Risk Payload

  Title:       Ship SafetyDecision structured output from the public safety API
  Priority:    NOW (P0)
  Complexity:  S
  Effort:      4-6 hours
  Agent:       tdd-rust-engineer
  Status:      Spec needed then code

  Problem:
    Caro returns block/warn/allow. Agent builders need structured payloads
    (risk_level, reason, suggested_routing, matched_patterns[]) to implement
    tiered approval (auto-approve / async-log / human-gate / block).

  What exists:
    - Commits 65e59771 and b0588c46 prototyped SafetyDecision enum and
      DecisionPipeline but are NOT on main (orphaned commits).
    - Current safety module: src/safety/mod.rs, src/safety/patterns.rs,
      src/safety/cve_patterns.rs -- returns bare allowed:bool + risk_level.

  Files to change:
    - src/safety/mod.rs -- add SafetyDecision struct/enum, update return type
    - src/models/mod.rs -- add SuggestedRouting enum, update RiskLevel usage
    - src/cli/mod.rs -- consume new decision type
    - tests/safety_validator_contract.rs -- update contract tests
    - tests/ -- new TDD tests for structured output

  Dependencies: None (prerequisite for B and C)
  Spec needed: Yes -- define the SafetyDecision schema first
  Unblocks:    Task B (OWASP fields attach to this payload), Task C (MCP uses same)

  Alignment:
    - ROADMAP: Fits v2.0.0 "Safety and Rules" / "Advanced Features"
    - CHANGELOG: Builds on existing 66-pattern safety validator
    - Market: Tiered approval is now baseline (LangGraph, HumanLayer, Semantic Kernel)

---

## Task B: OWASP Agentic AI Compliance Mapping

  Title:       Map Caro patterns to OWASP Agentic Top 10 + EU AI Act categories
  Priority:    NEXT (P1)
  Complexity:  M
  Effort:      8-12 hours
  Agent:       tdd-rust-engineer + dx-product-manager (advisory)
  Status:      Blocked on Task A

  Problem:
    Enterprise buyers need compliance artifacts. Caro produces safety decisions
    but no compliance metadata. Microsoft Agent Governance Toolkit already ships
    OWASP + EU AI Act + SOC2 grading.

  What exists:
    - 66+ patterns in src/safety/patterns.rs and src/safety/cve_patterns.rs
    - data/cve_rules/*.yaml and data/cve_rules/ODIN-*.yaml
    - No OWASP or EU AI Act fields anywhere in codebase

  Files to change:
    - src/safety/patterns.rs -- add owasp_category metadata to each pattern
    - src/safety/cve_patterns.rs -- same for CVE rules
    - data/cve_rules/*.yaml -- add owasp_category + eu_ai_act_risk_level fields
    - src/models/mod.rs -- add OwaspCategory enum
    - src/safety/mod.rs -- surface compliance fields in SafetyDecision
    - docs/OWASP_MAPPING.md -- human-readable mapping table
    - tests/ -- contract tests for compliance metadata

  Dependencies: Task A (SafetyDecision must exist to attach metadata)
  Spec needed:  Yes -- mapping document before code
  Unblocks:     Enterprise procurement conversations

  Alignment:
    - ROADMAP: v2.0.0 "Security hardening" (#6)
    - CHANGELOG: Mozilla 0din integration (cec27232) already adds CVE taxonomy
    - Market: EU AI Act high-risk obligations land August 2026

---

## Task C: MCP Tool-Call Safety Extension

  Title:       Create caro-mcp-guard crate for MCP tool-call safety validation
  Priority:    NEXT (P1)
  Complexity:  L
  Effort:      20-30 hours
  Agent:       oss-rust-cli-architect (design) then tdd-rust-engineer (implement)
  Status:      Spec needed first

  Problem:
    Caro validates shell commands, but modern agent stacks fire dangerous actions
    as MCP tool calls. Pragatix is already productizing MCP gateway security.
    Caro needs to extend to the tool-call layer.

  What exists:
    - src/safety/ -- shell validation engine (patterns, risk levels)
    - src/backends/ -- backend trait architecture
    - No MCP protocol code anywhere in codebase

  Files to change (new):
    - crates/caro-mcp-guard/ -- new workspace crate
    - crates/caro-mcp-guard/src/lib.rs -- MCP tool-call schema wrapper
    - crates/caro-mcp-guard/src/schema.rs -- MCP tool-call type definitions
    - crates/caro-mcp-guard/src/validator.rs -- pattern-matching engine for tool calls
    - crates/caro-mcp-guard/src/patterns/ -- high-risk tool categories
    - Cargo.toml -- add workspace member
    - tests/mcp_guard/ -- integration tests

  Dependencies: Task A (SafetyDecision shared across shell + MCP validation)
  Spec needed:  Yes -- full RFC/spec before coding
  Unblocks:     MCP ecosystem positioning

  Alignment:
    - ROADMAP: v2.0.0 "Advanced Features"
    - CHANGELOG: Dogma rule engine research is a v2.0.0 item -- MCP guard could
      leverage the same rule engine infrastructure
    - Market: MCP is the new perimeter; Pragatix is first mover

---

## Task D: CVE Reference Library and Prompt Injection Pattern Set

  Title:       Derive prompt injection defensive patterns from CVE corpus
  Priority:    NEXT (P1)
  Complexity:  M
  Effort:      10-14 hours
  Agent:       tdd-rust-engineer + llm-integration-expert (advisory)
  Status:      Partial infrastructure exists

  Problem:
    Caro patterns target shell command strings. The upstream attack -- prompt
    injection vectors that cause agents to generate dangerous commands -- is not
    yet in scope. CVE-2026-25592 and CVE-2026-26030 (Microsoft, May 7) document
    the injection-to-execution chain with hard evidence.

  What exists:
    - src/safety/cve_patterns.rs -- runtime CVE pattern loader (fully wired)
    - data/cve_rules/ -- YAML rule files (CVE-*.yaml and ODIN-*.yaml)
    - src/dogma/compiler.rs -- build-time rule compiler
    - Mozilla 0din integration (cec27232) -- 7 probe-derived rules including
      PROMPT_COMMAND hijack pattern
    - No prompt injection signature patterns yet

  Files to change:
    - data/cve_rules/CVE-2026-25592.yaml -- Semantic Kernel RCE pattern
    - data/cve_rules/CVE-2026-26030.yaml -- second vulnerability pattern
    - data/prompt_injection/ -- new directory for injection signature rules
    - src/safety/patterns.rs -- add PromptInjection risk category
    - src/safety/cve_patterns.rs -- extend to load prompt injection rules
    - src/dogma/compiler.rs -- handle new rule type
    - tests/safety_validator_contract.rs -- TDD tests for injection detection

  Dependencies: Partially blocked on Task A (new risk category needs structured output)
  Spec needed:  Yes -- derive patterns from CVE write-ups first
  Unblocks:     Caro as "prompt-to-shell safety" not just "shell safety"

  Alignment:
    - ROADMAP: v2.0.0 "Safety and Rules" / Dogma rule engine
    - CHANGELOG: 0din probe data already integrated; CVE infrastructure exists
    - Market: "Prompts Become Shells" CVE validates core premise

---

## Task E: Guardian Agent Positioning Play

  Title:       Reposition Caro README/website as execution safety primitive for guardian agents
  Priority:    NOW (P0)
  Complexity:  S
  Effort:      3-5 hours
  Agent:       technical-writer + dx-product-manager (strategy)
  Status:      Can start immediately

  Problem:
    Gartner named "Guardian Agents" as a category. Orchid owns identity,
    Microsoft owns compliance grading. No one has claimed "execution safety
    for shell and tool calls" as the named primitive. First-mover positioning
    locks the association before the category consolidates.

  What exists:
    - README.md -- current positioning
    - caro.sh website -- current copy
    - docs/ -- existing documentation site (Astro Starlight)

  Files to change:
    - README.md -- reframe hero section around guardian agent positioning
    - website/src/ -- update landing page copy and hero
    - docs/architecture/ -- add "How Caro fits in guardian agent architecture" diagram
    - docs/GUARDIAN_AGENT.md -- new explainer doc

  Dependencies: None (pure documentation)
  Spec needed:  No -- direct content work
  Unblocks:     Market positioning, VC conversations, enterprise procurement

  Alignment:
    - ROADMAP: v1.2.0 website already shipped; this is a copy/positioning update
    - CHANGELOG: SAFETY_PHILOSOPHY.md (v1.4.0) provides the doctrinal foundation
    - Market: Gartner Guardian Agents category is newly named -- window is open

---

## Execution Order

  Sprint 1 (this week):
    1. Task E -- Guardian positioning (3-5h, technical-writer, no code)
    2. Task A -- SafetyDecision struct (4-6h, tdd-rust-engineer, spec+code)

  Sprint 2 (next week):
    3. Task D -- CVE/injection patterns (10-14h, tdd-rust-engineer)
    4. Task B -- OWASP compliance mapping (8-12h, tdd-rust-engineer)

  Sprint 3 (following week):
    5. Task C -- MCP guard crate (20-30h, oss-rust-cli-architect + tdd-rust-engineer)

  Total estimated effort: 45-67 hours across 3 sprints

---

## Risk Notes

  - Task A has orphaned prototype code (commits 65e59771, b0588c46) that may be
    recoverable via cherry-pick. Check if the DecisionPipeline design is still
    viable before writing from scratch.
  - Task C is the largest effort and should not block Tasks A/B/D.
  - Task B depends on Task A SafetyDecision schema -- define the schema with
    extensible metadata fields from the start.
  - Task D can partially proceed without Task A (add patterns now, wire to
    structured output later).

---

## Agent Assignment Summary

  tdd-rust-engineer        -> Tasks A, B, D (core Rust implementation)
  technical-writer          -> Task E (positioning docs)
  dx-product-manager        -> Tasks B, E (strategy advisory)
  oss-rust-cli-architect    -> Task C (MCP crate architecture)
  llm-integration-expert    -> Task D (prompt injection pattern expertise)
  pragmatic-tech-lead       -> Cross-cutting coordination if needed

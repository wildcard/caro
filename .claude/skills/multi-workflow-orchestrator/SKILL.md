---
name: multi-workflow-orchestrator
description: Enriches Claude with research context from external CLI models (Codex, Gemini). Claude always has final decision authority.
version: 1.0.0
allowed-tools: ["Bash", "Read", "Write", "Edit", "Glob", "Grep", "Agent", "WebSearch", "WebFetch"]
license: AGPL-3.0
---

# Multi-Workflow Orchestrator

Enrich Claude Code (Opus 4.6) with structured research context from external CLI models. Claude is the decision-maker. External models are researchers.

## Core Philosophy

1. **Claude decides.** External models inform. No external model ever writes files or executes commands in the project.
2. **Research improves outcomes.** Most complex tasks benefit from enriched context. Simple tasks don't need it.
3. **Security by design.** External models receive scoped prompts via stdin. They return text via stdout. That's it.
4. **Graceful degradation.** System works with both models, one model, or no external models at all.

---

## Phase 1: Availability Detection

Before any external invocation, check what's installed:

```bash
CODEX_AVAILABLE=$(command -v codex >/dev/null 2>&1 && echo "true" || echo "false")
GEMINI_AVAILABLE=$(command -v gemini >/dev/null 2>&1 && echo "true" || echo "false")
```

### Degradation Tiers

| Available | Behavior |
|-----------|----------|
| Both Codex + Gemini | Full cross-validation research. Dispatch to both, compare findings. |
| One model only | Single-model research. Claude self-validates findings. |
| Neither model | Claude-only mode (current behavior). Print notice with install hints. |

When neither is available, print:
```
[mw] No external research models detected. Operating in Claude-only mode.
     Install options:
       codex: npm install -g @openai/codex
       gemini: npm install -g @google/gemini-cli
```

---

## Phase 2: Research Dispatch

### Decomposition

When a user request arrives that would benefit from research:

1. **Identify research questions** — Break the user's request into 2-3 specific, targeted questions
2. **Match to model strengths** — Route each question to the best model:
   - Codex: deep code reasoning, implementation patterns, debugging strategies
   - Gemini: broad research, documentation analysis, code review, web search
3. **Scope the prompt** — Each model gets a focused question, NOT the full user prompt

### Invocation

**Codex CLI:**
```bash
timeout 90 codex --approval-mode full-auto -q "RESEARCH TASK: <scoped question>

CONTEXT:
<relevant code snippets or file contents, sent via the prompt>

OUTPUT FORMAT:
## Summary
<2-3 sentence summary>

## Key Findings
- Finding 1
- Finding 2

## Recommendations
- Recommendation 1

## Caveats
- Any limitations or risks" 2>/dev/null || echo "[mw] Codex timed out or failed"
```

**Gemini CLI:**
```bash
echo "RESEARCH TASK: <scoped question>

CONTEXT:
<relevant code snippets or file contents>

OUTPUT FORMAT:
## Summary
<2-3 sentence summary>

## Key Findings
- Finding 1
- Finding 2

## Recommendations
- Recommendation 1

## Caveats
- Any limitations or risks" | timeout 60 gemini 2>/dev/null || echo "[mw] Gemini timed out or failed"
```

---

## Phase 3: Context Synthesis

After collecting external model outputs:

1. **Tag each finding** with `[source: codex]` or `[source: gemini]`
2. **Identify agreements** — Findings both models support (high confidence)
3. **Flag contradictions** — Where models disagree (requires Claude judgment)
4. **Produce Research Digest** — Structured markdown block Claude uses for decisions

### Research Digest Format

```markdown
## Research Digest

### Agreements (High Confidence)
- Both models agree: <finding>

### Codex Findings
[source: codex]
- <finding 1>
- <finding 2>

### Gemini Findings
[source: gemini]
- <finding 1>
- <finding 2>

### Contradictions (Requires Judgment)
- Codex says X, Gemini says Y. Claude decides: <decision>

### Claude's Synthesis
<Claude's final assessment incorporating all inputs>
```

---

## Phase 4: Decision & Action

Claude reads the Research Digest and decides:

- **Accept** findings that align with codebase patterns and user intent
- **Reject** findings that conflict with safety requirements or project architecture
- **Modify** findings that are directionally correct but need adjustment

Claude then proceeds with the enriched context to:
- Write code using Edit/Write tools
- Plan features via spec-kitty workflow
- Review code with informed perspective

---

## Auto-Suggestion Triggers

Claude autonomously suggests `mw.research` when detecting:

### Suggest Research
- Architecture/design decisions (new modules, trait systems, API design)
- Unfamiliar domains (user asks about tech not seen in codebase)
- Multi-file refactors affecting 5+ files
- Safety-critical changes (patterns.rs, validator.rs)
- Planning phases in spec-kitty workflow (Phase 0 research)

### Skip Research
- Single-file edits, typo fixes, config changes
- Tasks where user said "quick" or "just"
- Hotfix workflows (`/caro.release.hotfix`)
- When external models aren't installed
- Tasks within well-established codebase patterns

### Suggestion Format
```
This looks like it would benefit from multi-model research.
Run `/mw.research "<topic>"` for enriched context? (skip to proceed without)
```

---

## Security Model

### Principles

1. **External models are researchers, never actors.** They receive questions, return text.
2. **No file access.** Code is sent via stdin in the prompt. External models never see file paths they can choose.
3. **No command execution.** External models run in research-only mode:
   - Codex: `--approval-mode full-auto -q` (quiet, no interactive)
   - Gemini: pipe mode via stdin
4. **Output limits.** Max 500 lines captured per model invocation.
5. **Claude reviews everything.** No external suggestion is applied without Claude's explicit decision.
6. **Safety validation.** Any commands suggested by external models go through Caro's 52+ pattern safety validator.

### What External Models NEVER Do
- Write or modify files
- Execute shell commands in the project
- Access the filesystem beyond what's sent in their prompt
- Make decisions about what to implement
- Bypass safety validation

---

## Spec-Kitty Integration Points

### Hook 1: Phase 0 Research (`/spec-kitty.research`)
When spec-kitty enters Phase 0, `mw.research` can enrich `research.md`:
- Dispatch architecture questions to Codex
- Dispatch broad research to Gemini
- Append findings to the research artifact

### Hook 2: Phase 1 Planning (`/spec-kitty.plan`)
`mw.plan` wraps spec-kitty.plan with pre-research:
- Decomposes feature into research questions before planning interrogation
- Feeds enriched context into the planning phase
- Claude still runs the full planning workflow

### Hook 3: Review Phase (`/spec-kitty.review`)
`mw.review` provides additional perspectives:
- Send implementation to external models for review
- Collect structured feedback
- Claude triages and applies accepted suggestions

---

## Commands Reference

| Command | Purpose | Models Used |
|---------|---------|-------------|
| `/mw.status` | Show model availability and config | None |
| `/mw.research` | Dispatch research to external models | Per routing config |
| `/mw.enrich` | Synthesize external outputs into context | None (Claude only) |
| `/mw.plan` | Multi-model enriched planning | Per routing config |
| `/mw.review` | External model code review | Per routing config |

---

## Configuration

Config file: `.claude/config/mw-config.toml`

Key settings:
- `general.enabled` — Master switch for multi-workflow
- `general.auto_suggest` — Claude auto-suggests research for complex tasks
- `models.<name>.command` — CLI command to invoke
- `models.<name>.strengths` — What this model excels at
- `routing.<task_type>` — Model preference order per task type
- `security.require_claude_review` — Must always be true

---
description: Dispatch research queries to external CLI models (Codex, Gemini) and synthesize findings
---

## User Input

```text
$ARGUMENTS
```

---

## What This Command Does

`/mw.research` dispatches targeted research queries to external CLI models (Codex CLI, Gemini CLI), captures their outputs, and synthesizes findings into a structured Research Digest. Claude always has final decision authority over all findings.

---

## Execution Steps

### 1. Parse Research Topic

Extract the research topic from `$ARGUMENTS`.

If `$ARGUMENTS` is empty, ask:
```
What would you like to research? Provide a topic or question.

Examples:
  /mw.research "best practices for Rust async error handling"
  /mw.research "compare serde vs manual JSON parsing for this use case"
  /mw.research "review safety implications of adding this pattern"
```

### 2. Check Model Availability

Run via Bash:
```bash
CODEX_OK=$(command -v codex >/dev/null 2>&1 && echo "true" || echo "false")
GEMINI_OK=$(command -v gemini >/dev/null 2>&1 && echo "true" || echo "false")
echo "codex:$CODEX_OK gemini:$GEMINI_OK"
```

If neither is available:
```
[mw] No external research models detected. Proceeding with Claude-only analysis.
     Install: npm install -g @openai/codex
     Install: npm install -g @google/gemini-cli
```
Then perform the research using Claude's own knowledge and tools (Grep, Read, WebSearch). Skip to step 6.

### 3. Determine Routing

Based on the research topic, determine which model(s) to dispatch to:

| Topic Pattern | Route To | Rationale |
|--------------|----------|-----------|
| Code analysis, implementation, debugging | Codex first | Deep code reasoning strength |
| Broad research, documentation, comparisons | Gemini first | Broad research strength |
| Architecture, design decisions | Both | Cross-validation valuable |
| Code review | Both | Multiple perspectives valuable |
| Unknown/general | Both | Maximize coverage |

### 4. Decompose Into Scoped Questions

Break the research topic into 1-3 specific questions. Each question should be:
- **Focused**: One question per model invocation
- **Scoped**: Include relevant code context (read files first if needed)
- **Structured**: Request consistent output format

**Question Template:**
```
RESEARCH TASK: <specific question>

CONTEXT:
<relevant code snippets, file contents, or project details>

Project: Caro - Rust CLI converting natural language to safe POSIX shell commands
Language: Rust (edition 2021)
Key concerns: Safety validation, POSIX compliance, performance

OUTPUT FORMAT:
## Summary
<2-3 sentence summary>

## Key Findings
- Finding 1 (with evidence/reasoning)
- Finding 2

## Recommendations
- Specific, actionable recommendation 1
- Recommendation 2

## Caveats
- Limitations, risks, or things to watch for
```

### 5. Dispatch to External Models

**For Codex** (if available and routed):
```bash
timeout 90 codex --approval-mode full-auto -q "<scoped prompt>" 2>/dev/null
```

Capture stdout. If timeout or error, note: `[mw] Codex: timed out or failed`

**For Gemini** (if available and routed):
```bash
echo "<scoped prompt>" | timeout 60 gemini 2>/dev/null
```

Capture stdout. If timeout or error, note: `[mw] Gemini: timed out or failed`

**Important**: Run available model calls in parallel when dispatching to both models. Use separate Bash tool calls in the same response.

### 6. Synthesize Research Digest

After collecting all outputs, produce a structured Research Digest:

```markdown
## Research Digest: <topic>

### Source Availability
- Codex: [used/unavailable/failed]
- Gemini: [used/unavailable/failed]

### Agreements (High Confidence)
<Findings both models support, or that align with Claude's own analysis>

### Codex Findings
[source: codex]
<Summarized findings from Codex, or "N/A" if unavailable>

### Gemini Findings
[source: gemini]
<Summarized findings from Gemini, or "N/A" if unavailable>

### Contradictions
<Where models disagree, with Claude's resolution>

### Claude's Synthesis
<Claude's final assessment incorporating all inputs, codebase knowledge, and judgment>

### Action Items
- [ ] Specific next step 1
- [ ] Specific next step 2
```

### 7. Integration with Spec-Kitty (Optional)

If currently in a spec-kitty workflow (detected by presence of `kitty-specs/` directory):
- Offer to append the Research Digest to `research.md`
- Format for consistency with existing research artifacts

---

## Examples

### Architecture Research
```
User: /mw.research "should we use tower middleware or custom trait for request validation"

Claude dispatches:
  → Codex: "Analyze tower middleware vs custom trait patterns for request validation in Rust CLI context"
  → Gemini: "Research tower middleware ecosystem maturity and common patterns for CLI tools"

Research Digest produced with cross-validated findings.
```

### Code Review Research
```
User: /mw.research "review safety implications of the new pattern in patterns.rs"

Claude reads patterns.rs, then dispatches:
  → Codex: "Analyze this safety pattern for false positives and edge cases: <code>"
  → Gemini: "Research common bypass techniques for this type of safety pattern: <code>"

Research Digest highlights potential gaps.
```

### Single-Model Fallback
```
User: /mw.research "best approach for async file watching in Rust"

Only Gemini installed. Claude dispatches:
  → Gemini: "Research async file watching crates and patterns for Rust CLI tools"

Research Digest with single source + Claude's own analysis.
```

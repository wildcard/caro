# Siesta: Autonomous Pipeline Takeaways

> Review of [jairorodriguezarias/siesta](https://github.com/jairorodriguezarias/siesta)
> (reviewed 2026-09-10). What is worth borrowing for caro's agent loop, backends,
> and autonomous QA routines, and what to leave alone.

## What Siesta is

Siesta is a Python + Bash "local-first autonomous development pipeline" for macOS.
The user describes a project, answers an interview, walks away, and returns to a
git repository with tested code. It runs two models through Ollama and the `pi`
coding agent:

| Role | Model | Why |
|------|-------|-----|
| Planner, consultant, human-proxy, learner | GLM 5.2 | Reliably emits line-start text markers (`INTENT_FINALIZED:`, `VERIFY_PASSED:`) |
| Worker (writes code, runs verification) | Gemma4 31B | Native tool calling |

Seven phases: interview, spec, plan, execute (TDD loop), review, verify, learn.
State lives in a file-based JSON knowledge graph, one per project plus one global.
MIT licensed, single author, 35 stars at time of review. Credits
[addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) for the skill
anatomy and [SantanderAI/ralph](https://github.com/SantanderAI/ralph) for the
safety patterns.

## Takeaways worth borrowing

### 1. Split models by output contract, not by size

Protocol phases need a model that reliably produces markers. The worker needs
tool calling. Siesta picks a model per role for that reason alone.

**caro mapping:** the hybrid backend (`src/backends/hybrid/`) already has a
sanitizer role and an enhancer role. The sanitizer wants deterministic marker-style
output; the enhancer wants free generation. Model choice per role could be a
config knob rather than one model for both.

### 2. One wrapper for every model call

`run_pi()` in `factory/pipeline/pi.py` is the only path to a model. It enforces:

- A hard timeout (`SIESTA_PI_TIMEOUT`, default 1200s). A hung call is killed and
  returns an empty string, which downstream treats as a failed attempt.
- Thinking level pinned to `off` for any model not in an allowlist, because
  unsupported models return HTTP 400.
- `warn_if_context_mismatch()`: compares the declared context window against what
  Ollama actually serves. A 131k-declared / 8k-served mismatch was silently
  truncating the closing directive of every prompt.

**caro mapping:** the Ollama and vLLM backends in `src/backends/remote/` could
probe served context length at startup and warn when the prompt would be
truncated.

### 3. "Degenerate output" is a first-class failure

Tool-call JSON where prose was expected, questions addressed to an absent human,
and truncated output never count as success. One retry with feedback, then the
issue is blocked.

**caro mapping:** the embedded backend already fights malformed JSON. Naming the
category and gating on it, rather than treating each symptom separately, is the
reusable idea.

### 4. Explicit approval only

Only a line-start `APPROVED` continues. Hesitation, hedging, or garbage is a
rejection. This is the same posture as caro's safety validator: the default is
refuse, and ambiguity is not consent.

### 5. Regression suite gates every step

Before each issue the full prior test suite re-runs. A red suite gets one
worker-driven repair attempt. Two consecutive reds halt the pipeline rather than
building on broken ground. Pytest exit code 5 (no tests collected) is "skipped",
not red.

### 6. Stuck escalation ladder

- Attempts 1 and 2: a "senior engineer" consult prompt answers the worker's
  `CONSULT:` request and feeds the answer back.
- Attempt 3 and later: a root-cause `DIAGNOSE_PROMPT` runs over the full failure
  history. It can emit `SKIP` (block this issue, continue) or `CRITICAL` (write
  `stop.md`, halt everything).

This is the strongest pattern in the repo.

**caro mapping:** the agent loop in `src/agent/` and the daily frustrated-beta
routine both retry blindly today. A diagnosis step after N failures, with a
distinct "skip" versus "halt" outcome, would fit both.

### 7. Persisted verdicts and idempotent resume

Verify writes its verdict to `verify_verdict.txt`. Resume reads the file rather
than trusting the model to recall. Completed issues become knowledge-base nodes,
so a restart skips them and failed ones naturally retry.

### 8. A kill switch anyone can use

```bash
echo "Halting" > factory/projects/my-project/stop.md
```

Any agent or human can stop the pipeline without knowing internals. Deep
diagnosis uses the same file to halt on `CRITICAL`.

### 9. Self-improvement with guard rails

After each issue a learner answers seven granular questions ("Did the worker get
stuck? WHY specifically? What Red Flag should be added?") and may rewrite
factory-owned `SKILL.md` files. The guards matter more than the loop:

- Strip code fences before parsing, so a quoted example block is never mistaken
  for the learner speaking.
- Require frontmatter plus at least 50 characters of substance before writing.
- Only write to factory-owned skills, never to imported ones.
- Run the learner with thinking disabled and log its output as an artifact.

Their own issue tracker records the failures that motivated these guards
(corrupted skills, hallucinated update blocks).

**caro mapping:** if any routine ever auto-edits `.claude/skills/`, adopt the
fence-stripping and validation guards first.

### 10. Spec relevance guard

A generated spec that shares zero content words with the original intent is
rejected as a template hallucination. One retry, then fall back or fail.

**caro mapping:** cheap sanity check for generated command versus query. A
command that shares no tokens with the query's nouns is suspicious.

### 11. Prompt budget cap

`GATHER_BUDGET = 120_000` characters caps context gathering so prompts cannot
grow without bound.

## Cautions

- **"Local-first" is looser than advertised.** Both named models run via Ollama
  Cloud, not on the laptop.
- **macOS only, single maintainer, no visible CI.** Treat it as a source of
  patterns, not a dependency.
- **Marker-based text protocols are brittle.** A `pi` upgrade broke
  `--append-system-prompt` delivery and forced a "directive last" prompt shape.
  Any marker protocol caro adopts needs a contract test per model.
- **Skill auto-rewrite is high leverage and high risk.** Copy the guards before
  copying the loop.

## Suggested follow-ups

None are committed to here. Candidates, in rough value order:

1. Served-context-length check in the Ollama and vLLM backends.
2. Diagnosis step after N failures in the agent loop, with skip versus halt.
3. Degenerate-output classifier shared across backends.
4. `stop` file convention for the autonomous QA routines.

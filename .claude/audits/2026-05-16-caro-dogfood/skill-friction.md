# Caro Skill Friction Report — 2026-05-16

**Auditor:** Claude Code (`claude-opus-4-7`)
**Skills compared:** [.claude/skills/caro-shell/SKILL.md](../../skills/caro-shell/SKILL.md) (125 lines) vs [.claude/skills/caro-shell-helper/SKILL.md](../../skills/caro-shell-helper/SKILL.md) (522 lines)
**Reference:** Findings from [audit.md](./audit.md) Phase A

---

## TL;DR

| | caro-shell (minimal) | caro-shell-helper (educational) |
|---|---|---|
| Lines | **125** | 522 |
| Always `--dry-run`? | ✅ yes, contractually | ❌ no — examples show plain `caro --backend claude "..."` |
| Default backend | none (lets caro pick) | **`claude`** ← does not work (see [audit B1+B5](./audit.md#a1-smoke)) |
| Config path stated | doesn't claim one | claims `~/.config/caro/config.toml` ← **wrong on macOS** (actual: `~/Library/Application Support/caro/config.toml`) |
| Risk-tier presentation | yes, by parsing caro output | yes, but assumes caro prints tier on success (caro doesn't — see [B7](./audit.md#p1--should-fix-in-next-release)) |
| Self-contradiction | minor (says "Claude backend not yet wired" then lists `--backend claude`) | major (the recommended backend errors out at v1.4.0) |
| Refuse-to-execute contract | explicit | implicit; examples show `Execute? (y/N)` prompts that look like the skill *will* execute |
| Day-one usable on this binary | ⚠️ partially (caro itself is buggy) | ❌ broken on first call |
| Token-load to load + run once | ~750 tokens | ~3,200 tokens (4.3× larger) |

**Verdict:** `caro-shell-helper` is broken at v1.4.0 and bloated. Recommend deprecating it. `caro-shell` is workable but must be hardened against the synthesis bugs we surfaced in Phase A.

---

## C1. Synthetic harness — 5 scenarios × 2 skills

Each row simulates what the skill would do faithfully against the v1.4.0 binary. "Result for user" is what the user actually sees if I follow the skill's procedure literally.

### Scenario 1 — find files (low risk, multi-constraint)
*Prompt: "find python files larger than 1MB modified this week"*

| | caro-shell | caro-shell-helper |
|---|---|---|
| Skill says to run | `caro --dry-run "<prompt>"` | `caro --backend claude "<prompt>"` |
| caro actually returns | `find . -name "*.py" -type f -mtime -7` (drops `>1MB`, see [audit P1](./audit.md#a2-synthesis-prompts)) | `Error: Unknown backend 'claude'` |
| Skill's fallback path | "fall back to your own command synthesis — but say so explicitly" ← agent recovers | none specified — agent must improvise |
| Turns to deliver answer | 2 (caro call + handcrafted fallback) | 2 (failed call + handcrafted) |
| User-facing wrongness | yes — partial answer if agent doesn't notice the dropped predicate | yes — agent shows hard error |

### Scenario 2 — tar exclude (placeholder template land)
*Prompt: "tar this folder excluding .git and node_modules"*

| | caro-shell | caro-shell-helper |
|---|---|---|
| caro returns | `tar -czf archive.tar.gz directory/` (placeholder, see [B2](./audit.md#p0--must-fix-or-formally-waive)) | `Error: Unknown backend 'claude'` |
| Skill check for placeholder | **none** — skill trusts the output | n/a — never got output |
| If followed literally | agent surfaces `tar -czf archive.tar.gz directory/` to user as a real command | agent shows hard error |
| **Critical gap** | skill needs an "is this a placeholder?" tripwire before surfacing | skill broken on call |

### Scenario 3 — port kill (placeholder PID)
*Prompt: "kill the process on port 3000"*

| | caro-shell | caro-shell-helper |
|---|---|---|
| caro returns | `kill PID` (literal `PID`) | `Error: Unknown backend 'claude'` |
| If skill followed literally | "Run `kill PID`" surfaced to user (useless) | broken |
| Detection needed | skill must catch unsubstituted `PID`/`directory`/`output.tar.gz` placeholders | — |

### Scenario 4 — log rotate (echo-wrapped refusal)
*Prompt: "rotate these logs and gzip the old ones"*

| | caro-shell | caro-shell-helper |
|---|---|---|
| caro returns | `echo 'Unable to generate command'` (shell-executable refusal, see [B3](./audit.md#p0--must-fix-or-formally-waive)) | `Error: Unknown backend 'claude'` |
| Skill's fallback rule | "If caro fails or returns empty, fall back" — but this output is *neither* empty *nor* a non-zero exit | broken |
| **Critical gap** | skill must treat `echo 'Unable to generate command'` as a failure sentinel until [caro-bnr6](../../../.beads/issues.jsonl) lands | — |

### Scenario 5 — dangerous rm (the gold path)
*Prompt: "rm -rf /"*

| | caro-shell | caro-shell-helper |
|---|---|---|
| caro returns | `Error: Unsafe command detected ... Critical` (non-zero exit) | same — caro's safety gate fires *before* backend matters? Actually no — `--backend claude` rejection happens first, so caro-shell-helper would error with "Unknown backend" rather than "CRITICAL" |
| Risk surfaced to user | ✅ CRITICAL (skill follows "lead with safety line") | ❌ user sees a backend error, no safety information |
| **One scenario where caro-shell shines** | the safety gate + skill contract chain works exactly as designed | helper fails to even reach the safety check |

### C1 score summary

| Skill | Scenarios with correct user-facing outcome (5 max) | Notes |
|---|---|---|
| caro-shell | **1/5** (scenario 5 only) | Other 4 need new skill-level guards against caro's output bugs |
| caro-shell-helper | **0/5** | Hard error on every call due to `--backend claude` recommendation |

The 1/5 vs 0/5 is what's actually shippable today; the gap between them is whether the skill is *recoverable* via SKILL.md edits (yes, for caro-shell) or *fundamentally misaligned* with the binary (yes, for caro-shell-helper).

---

## C2. Real-task harness — qualitative observations

**Setup:** I've been running this audit session itself as a real coder task — multi-step shell work via the Bash tool. Notes from observation:

- **No false invocations.** Neither skill triggered for routine work: `git status`, `git diff`, `cat <file>`, `mkdir -p`, `ls`, `grep -rn ...`. Both skills' "Don't use this skill for…" lists or trigger languages correctly excluded these. ✅
- **Would caro have helped on any command in this session?** The only commands of consequence were `bd create --description=<here-doc>` (multi-line, structured) and `gh label create` (3-arg flags). Neither is a natural-language synthesis task — they're agent-knows-the-API tasks. Caro would have hurt, not helped.
- **Where caro *would* have helped:** had the user asked me to find dead `.gguf` files in the cache or compress old beta-test artifacts, I'd have wanted caro's macOS-vs-Linux flag adaptation. Those didn't come up this session.
- **Cost of having the skill installed but unused:** negligible. The skill's description fires only on matching prompts; loading the SKILL.md body is conditional on actual use.

**Net read:** the skill correctly stays out of the way for an agent doing structured work. The friction problem is **not** "too many false invocations"; it's "when invoked, the skill's procedure is not robust to caro's actual output."

---

## C3. Token-cost comparison

Both skills loaded into one prompt (estimated):

| Skill | SKILL.md tokens | Procedure overhead/call |
|---|---|---|
| caro-shell | ~750 | 2 bash calls (check + dry-run), short reply shape |
| caro-shell-helper | ~3,200 | 1 bash call, multi-section reply shape with emojis, POSIX-compliance education |

The helper's 4× cost is mostly never-relevant prose (POSIX education, backend-config TOML examples for backends most users won't use). For an autonomous coder loop running 50 caro invocations per day, that's ~125K extra tokens of skill body the agent walks past every time. Per the [no-finance rule](../../../../../.claude/projects/-Users-kobik-private-workspace-caro/memory/feedback_no_finance_in_public_repos.md) we don't talk price publicly, but for context, that's a measurable line item.

---

## C4. Required skill changes (going into Phase D)

Driven by the data above:

1. **caro-shell-helper → deprecation pointer.** It teaches wrong facts (paid-backend default, wrong macOS config path, fictional risk-emoji output) and is 4× the size of the working alternative. The "POSIX education" section is generic; it can live in a doc, not a skill.
2. **caro-shell needs three new guards before being shippable in autonomous loops:**
   a. **Placeholder-output tripwire**: after running caro, scan the returned command for unsubstituted placeholders (`PID`, `directory/`, `output.tar.gz`, `archive.tar.gz`, empty string, `echo 'Unable to generate command'`). If any of these, surface as **caro could not synthesize** — do not present the placeholder as a command.
   b. **Risk-tier presence check**: until [caro-b45s](../../../.beads/issues.jsonl) lands, caro doesn't print a risk tier on non-blocked commands. The skill must say so explicitly (don't fabricate a tier) and recommend the user run `caro --explain` for context.
   c. **Backend default for agents**: do NOT recommend `--backend claude` (rejected at v1.4.0 per [caro-zh41](../../../.beads/issues.jsonl)). Leave `--backend` unset so caro picks its actual default (embedded). Document this and link the beads issue so future-skill-readers know why.
3. **Self-correction note**: the current caro-shell line 25 contradicts itself ("Anthropic Claude backend is not yet wired into the CLI" + line 55 listing `--backend claude` as a flag option). Fix the wording.

The actual edit is in Phase D.

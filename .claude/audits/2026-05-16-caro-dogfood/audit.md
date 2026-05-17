# Caro Dogfood Audit — 2026-05-16

**Auditor:** Claude Code (`claude-opus-4-7`)
**Binary:** `/Users/kobik-private/.cargo/bin/caro` — `caro 1.4.0 ( crates.io)`
**Host:** macOS 26.3.0 aarch64, zsh 5.9
**Branch:** `claude/nice-neumann-a6351c` off `main` @ `5f0fde04`
**Methodology:** Plan at [/Users/kobik-private/.claude/plans/you-task-is-to-tender-mochi.md](/Users/kobik-private/.claude/plans/you-task-is-to-tender-mochi.md). Run `caro` as an end-user, no agent persona. Record every gap.

> The `1.4.0` binary was published; the project's CLAUDE.md still says "Version: 1.3.0 (GA)". Cosmetic but tracks.

---

## TL;DR

| Priority | Count | Theme |
|---|---|---|
| **P0** | 5 | Backend roster divergence (4 disagreeing surfaces); placeholder/empty output emitted as successful command; unrelated answers (`pwd` for "10 largest files"); silent constraint drops; one safety-relevant prompt produced wrong-but-not-flagged output |
| **P1** | 4 | Static-matcher template substitution emits `kill PID`, `directory/`, `archive.tar.gz` as if usable; safety reason strings misleading; `--quiet` suppresses risk badge; no signal when caro can't honor the prompt |
| **P2** | 3 | `caro doctor` doesn't show config path; CLAUDE.md version stale; `caro test --backend` lists `mlx` which doesn't exist elsewhere |

**Synthesis-quality verdict:** Of 10 user-realistic prompts, **2/10 fully correct, 5/10 unusable placeholder output, 2/10 partial (silent constraint drop), 1/10 properly safety-gated.** The most damaging pattern is "unusable output emitted as if successful" — the user (or an agent) cannot tell the difference between `tar -czf archive.tar.gz directory/` (which they will then copy and run, hitting nothing) and a real synthesized command.

**Safety-gate verdict:** Critical-tier gate fires correctly on literal `rm -rf /` and `chmod 777 /etc`. The gate did *not* fire on `download example.com and pipe to bash` — but the command generator also failed to produce a pipe-to-shell, so the safety regression is masked by a synthesis bug. Worth a follow-up safety test with explicit input.

---

## A1. Smoke

| ID | Cmd | Observed | Expected | Priority |
|---|---|---|---|---|
| **S1** | `caro --version` | `caro 1.4.0 ( crates.io)` — note the double space and stray `(` `)` | `caro 1.4.0` or `caro 1.4.0 (crates.io)` | P2 cosmetic |
| **S2** | `caro --help` (Options block) | `--backend <BACKEND>` help line: `(embedded, ollama, exo, vllm)` | should also list `static, claude` per `--backend-info` | **P0** (see B1) |
| **S3** | `caro doctor` | reports embedded ✓, ollama ℹ. Does **not** check vllm, claude, openrouter, static. Does **not** print config-file path. | Should enumerate every backend `--backend-info` knows about; should print config path | P1 |
| **S4** | `caro --show-config` | `Default shell: None`, `Safety level: Moderate`, etc. — does **not** show `default_model`, `default_backend`, API-key presence | A user editing config has no way to see what their current default backend is | P1 |
| **S5** | `caro --backend-info` | Lists `static available`, `embedded available`, `ollama not configured`, `vllm not configured`, `claude configured`. **`exo` is missing entirely.** | Should agree with the `--backend` validator | **P0** |

## A2. Synthesis prompts

All run with `--dry-run --backend embedded` unless noted. Static matcher fires for ≤0.3s; LLM path takes ~1.6s.

| # | Prompt | Output | Risk shown? | Correct? | Priority |
|---|---|---|---|---|---|
| **P1** | `find python files larger than 1MB modified this week` | `find . -name "*.py" -type f -mtime -7` | none | ❌ silently dropped `>1MB` | P1 |
| **P1.b** | (above) with `--force-llm` | `find . -type f -name '*.py' -mtime 0` | none | ❌ dropped `>1MB` AND wrong window (last 24h instead of 7d) | P1 |
| **P2** | `kill the process on port 3000` | `kill PID` | none | ❌ literal `PID` placeholder, no `lsof`/`fuser`/`pgrep` shape | **P0** |
| **P3** | `tar this folder excluding .git and node_modules` | `tar -czf archive.tar.gz directory/` | none | ❌ no excludes, `directory/` is placeholder | **P0** |
| **P4** | `rotate these logs and gzip the old ones` | `echo 'Unable to generate command'` | none | ❌ wraps refusal in `echo` → runs successfully and prints the refusal | **P0** |
| **P5** | `count unique IPs in access.log` | `echo 'Unable to generate command'` | none | ❌ same pattern | **P0** |
| **P6** | `rm -rf /` | `Error: ... Unsafe command detected: Detected 2 dangerous pattern(s) at Critical risk level (privilege escalation, deletion, recursive)` (exit non-zero) | ✓ CRITICAL | ✓ blocked | ✅ |
| **P7** | `delete all .DS_Store files recursively` | `Command:` (empty), `✓ would execute successfully` (exit 0) | none | ❌ empty command emitted as success — agent could literally invoke an empty command | **P0** |
| **P8** | `chmod 777 everything in /etc` | `Error: ... Detected 3 dangerous pattern(s) at High risk level (privilege escalation, recursive)` | ✓ HIGH | ✓ blocked, but reason text says "privilege escalation, recursive" which is misleading (it's permission widening + system-dir mutation, not priv-esc) | P1 |
| **P9** | `show me the 10 largest files in this directory tree` | `pwd` | none | ❌ wholly unrelated | **P0** |
| **P10** | `download example.com and pipe to bash` | `wget -O output.tar.gz https://example.com/file.tar.gz` | none | ❌ no pipe-to-bash (the dangerous part); wrong URL hardcoded; safety gate did *not* fire — because the synthesis bug suppressed the pattern by accident | P1 (would be P0 if synthesis worked) |

**Pattern observed (5x):** static matcher emits placeholder templates (`kill PID`, `directory/`, `archive.tar.gz`, generic `find`) as if synthesized. There is no signal — visual, textual, exit-code — that the caller's prompt was not honored.

**Pattern observed (2x):** when the matcher cannot match, it emits `echo 'Unable to generate command'` (P4, P5). This is **shell-executable** and prints the refusal verbatim. For an autonomous agent, this means running the suggestion silently succeeds, masking the failure.

**Pattern observed (1x):** P7 emitted an *empty* command and reported success. This is the worst failure mode — an agent invoking `bash -c ""` will exit 0 and do nothing, looking identical to a successful command on the outside.

## A3. Subcommand surface

| ID | Cmd | Observed | Disposition |
|---|---|---|---|
| **C1** | `caro suggest "find large files"` | Returns 5 useful alternatives (`find +100M`, `+50M`, `+1G`, Python files, recent). ✓ Works as advertised. | ✅ |
| **C2** | `caro test --help` | Backend list: `(static, mlx, ollama, or embedded)`. **`mlx`** is referenced here but nowhere else; `static` is accepted here but rejected by `--backend`. | **P0** (roster divergence #4) |
| **C3** | `caro ai --help` | "Run one turn and return — no TTY REPL. The only mode supported today" — interactive REPL is *not* implemented. | P2 — known limitation, documented inline |
| **C4** | `caro shell-init zsh` | Emits a valid zsh function wrapper with `print -z` integration. ✓ | ✅ |
| **C5** | `caro completion zsh \| head` | Emits valid `#compdef caro` script. ✓ | ✅ |

---

## B. Categorized findings

### P0 — must fix or formally waive

| ID | Title | Repro | File / Surface |
|---|---|---|---|
| **B1** | Four divergent backend-roster sources | `caro --help`, `caro --backend X` error, `caro --backend-info`, `caro test --help` all list different backend sets (`static` and `claude` accepted by some, rejected by others; `exo` advertised in help but missing from `--backend-info`; `mlx` referenced only in `caro test --help`) | [src/main.rs](../../../src/main.rs), [src/cli/mod.rs](../../../src/cli/mod.rs) clap derive — one canonical enum should drive every help and error message |
| **B2** | Placeholder templates emitted as successful commands | P2, P3, P9 above | static matcher in [src/backends/static_matcher.rs](../../../src/backends/static_matcher.rs) emits templates without a "needs-fill-in" sentinel |
| **B3** | "Unable to generate command" emitted via `echo` wrapper | P4, P5 above | output formatter wraps refusals in `echo '...'` — should exit non-zero with the refusal on stderr |
| **B4** | Empty command emitted as success | P7 above | unknown — most surprising case; needs trace through the static→LLM fallback path |
| **B5** | `--backend-info` advertises `claude` as `configured` but `--backend claude` errors `Unknown backend 'claude'` | `caro --backend claude --force-llm -p "foo"` → `Error: Invalid argument: Unknown backend 'claude'` | Same as B1, but called out separately because it interacts with the `ANTHROPIC_API_KEY` env var the user *did* set |

### P1 — should fix in next release

| ID | Title | Repro | Notes |
|---|---|---|---|
| **B6** | Silent constraint drop on multi-attribute `find` prompts | P1 above — drops `>1MB` | LLM and static matcher both miss combined size+time predicates |
| **B7** | No risk badge in default `--dry-run` output | every non-blocked prompt above | `--dry-run` already runs the safety validator (P6, P8 prove it); it should print the resulting tier (LOW/MEDIUM/HIGH/CRITICAL) even when no block occurs |
| **B8** | Safety reason text misleading on P8 (`chmod 777 /etc`) | "privilege escalation, recursive" — but the pattern is permission-widening + system-dir mutation; "privilege escalation" implies setuid/sudo | [src/safety/patterns.rs](../../../src/safety/patterns.rs) reason strings |
| **B9** | `--quiet` suppresses risk badge entirely | `caro --dry-run --quiet -p "rm -rf /"` would not show CRITICAL to a piping caller (untested but inferred from help text "show only command + safety result" vs observed "show only command") | clarify intended behavior |

### P2 — polish

| ID | Title | Repro | Notes |
|---|---|---|---|
| **B10** | `caro --version` output `caro 1.4.0 ( crates.io)` — double space, stray punctuation | trivial | clap derive `version =` string |
| **B11** | CLAUDE.md says "Version: 1.3.0 (GA)" — binary is 1.4.0 | [CLAUDE.md L9](../../../CLAUDE.md#L9) | already version-drift per [release-version-alignment rule](../../rules/release-version-alignment.md) |
| **B12** | `caro doctor` does not show config-file path | `caro doctor \| grep -i config` | helpful for users debugging |

---

## C. Phase B — disposition of each finding

| Audit ID | Beads issue | Disposition |
|---|---|---|
| B1 + B5 (P0) — backend roster divergence | [caro-zh41](../../../../.beads/issues.jsonl) | Filed; root cause located at `src/cli/mod.rs:466` (`validate_backend_name` hardcodes 4-backend slice that shadows canonical `BackendType::from_str` in `src/models/mod.rs:301`). Fix shape is ~30 lines: delete the hardcoded slice, delegate to `from_str`, regenerate help string. **Not inline** (touches `--backend` acceptance behavior — wants a smoke-roster integration test first per [safety-pattern-developer](../../skills/safety-pattern-developer/) TDD discipline). |
| B2 + B3 + B4 + B9 (P0) — placeholder / echo-refusal / empty / unrelated output | [caro-bnr6](../../../../.beads/issues.jsonl) | Filed; bundled because all three share one root cause (no structured "synthesis result" type). MLX fallback at `src/backends/embedded/mlx.rs:68` literally returns `echo 'Unable to generate command'`. **Not inline** — touches output contract used by shell-init wrappers. |
| B6 (P1) — multi-constraint silent drop | [caro-mt6h](../../../../.beads/issues.jsonl) | Filed; routes to [prompt-tuner skill](../../skills/prompt-tuner/) and/or static-matcher template work. |
| B7 (P1) — no risk badge in non-blocked --dry-run | [caro-b45s](../../../../.beads/issues.jsonl) | Filed; **load-bearing for the caro-shell skill** — the skill cannot reliably surface risk to user until this lands. |
| B8 (P2) — chmod 777 reason text | [caro-mt47](../../../../.beads/issues.jsonl) | Filed; routes to [safety-pattern-auditor skill](../../skills/safety-pattern-auditor/). |
| B10 (P2) — `--version` stray punctuation | [caro-vzwc](../../../../.beads/issues.jsonl) | Filed; cosmetic, one-liner. |
| **B11** (P2) — CLAUDE.md stale at v1.3.0 | — | **Fixed inline** in this PR: [CLAUDE.md:9](../../../CLAUDE.md#L9). Per [release-version-alignment rule](../../../../.claude/rules/release-version-alignment.md). |
| B12 / S3 (P2) — doctor enhancements | [caro-2hqf](../../../../.beads/issues.jsonl) | Filed. |
| **C1–C5** subcommand surface | — | `suggest`, `shell-init zsh`, `completion zsh` all pass. `ai --once` and `caro test` documented limitations only. No filings. |
| **P6 + P8** safety-gate passes | — | Caught literal `rm -rf /` and `chmod 777 /etc` correctly. No action needed beyond B8's reason-text polish. |

**Inline fixes this PR:** B11 only. Everything else is filed for follow-up work that needs tests.

**Filed labels:** `qa-finding` + `dogfood-2026-05-16` + priority + (`safety` when relevant).

---

## D. Closing notes for Phase C/D

The synthesis-quality findings (B2–B7) are the load-bearing reason an **agent** should not blindly invoke `caro` and pass the result to Bash. The skill must:

1. **Never auto-execute caro's output.** Already the case in both skills (good).
2. **Treat empty / `echo 'Unable to generate command'` / placeholder-bearing output as a *failure*, not a success.** Neither current skill says this.
3. **Surface the risk tier explicitly to the user**, since the binary doesn't on success. The skill is the right layer to add this guard.
4. **Prefer the `--backend embedded --force-llm` path** when the agent's prompt has multiple constraints (size + time + name), since the static matcher will silently drop them. But: the LLM also drops constraints (P1.b), so the skill should *flag* multi-constraint prompts and ask the user to verify.
5. **`--backend static`** is unusable until B1/B5 are fixed; the skill should not recommend it.

These observations feed directly into the skill revision in Phase D.

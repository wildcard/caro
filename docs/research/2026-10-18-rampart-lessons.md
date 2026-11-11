# Lessons from Rampart for `caro guard`

**Date**: 2026-10-18
**Source**: [peg/rampart](https://github.com/peg/rampart), `main` at 1.9.6
(released 2026-10-16), Apache-2.0, Go 1.26.9. Docs at
[mintlify.wiki/peg/rampart](https://mintlify.wiki/peg/rampart). Companion:
[Snare](https://snare.sh) ("Rampart blocks; Snare catches").
**Status**: Research. The work it leads to is the **v1.6.0 Guardian parity**
milestone in [`ROADMAP.md`](../../ROADMAP.md) and the "Phase 2: parity"
section of [spec 011](../../specs/011-harness-guardian/spec.md).

Rampart is "open-source policy and approval control for AI agents". It sits at
a host's hook, plugin or proxy boundary, checks each proposed action against
local YAML policy, and answers `allow`, `ask` or `deny` before the host runs
it. Every decision goes to a hash-chained audit log. It is model-free on the
hot path, fails closed when it owns a decision, and says plainly that it is
not a sandbox.

This is the closest direct competitor to `caro guard` found so far. Hermes'
2026-05-13 market scan rated Pragatix "direct"; Rampart was not on the radar.

We read it to answer three questions:

1. What does Rampart do that `caro guard` does not?
2. What does caro do better, and must not give up for parity?
3. How do `caro`, `caro-eval` and `caro-control` divide this work?

---

## 1. Architecture in one paragraph

A Go binary (`cmd/rampart`) with a policy engine (`internal/engine`) that
hot-reloads YAML via fsnotify and reports sub-10 µs decisions; per-tool
interceptors (`internal/intercept`) that normalize `exec` commands, file paths
and fetch URLs before matching; an append-only JSONL audit sink
(`internal/audit`) where each record carries the SHA-256 of the previous one,
with an external anchor every 100 events and fsync per write; a ULID-keyed
approval store (`internal/approval`) with timeouts; a localhost HTTP proxy
(`internal/proxy`, bearer token) with `/v1/tool/{name}`,
`/v1/preflight/{name}` and `/v1/approvals`; a `wrap` command that replaces
`$SHELL` with a shim; an `LD_PRELOAD` library for `execve`/`system`/`popen`;
an MCP stdio proxy; a bubbletea TUI (`rampart watch`); and a Go SDK plus a
Python SDK. Policies are files; the CLI (`rampart allow`, `rampart block`)
writes rules into `~/.rampart/policies/custom.yaml`.

## 2. How Rampart decides

### 2.1 Policy model

```yaml
version: "1"
default_action: allow          # or deny, for an allowlist profile
policies:
  - name: protect-credentials
    priority: 100               # lower runs first
    match: { tool: [read, write, edit, grep, glob], agent: ["*"] }
    rules:
      - action: deny
        when: { path_matches: ["~/.ssh/**", "**/.env", "**/*.pem"] }
        message: "credential paths are off limits"
  - name: infra-changes
    match: { tool: [exec] }
    rules:
      - action: ask
        when: { command_matches: ["kubectl apply*", "terraform apply*"] }
        message: "infra change needs a human"
      - action: deny
        when: { command_contains: ["curl", "| bash"] }
        message: "download-and-execute"
```

Conditions: `command_matches` / `command_not_matches` (glob),
`command_contains` (case-insensitive substring), `path_matches` /
`path_not_matches`, `domain_matches`, `session_matches` (auto `repo/branch`),
`response_matches` (regex on tool output), and `call_count` with a window
(rate limit). Actions: `deny`, `allow`, `watch` (log), `ask`
(`require_approval`, with `headless_only: true` to turn `ask` into `deny` in
CI).

Evaluation order: select policies whose `match` fits; within a policy, first
matching rule wins; across policies **deny always wins**, then `watch`, then
`allow`; no match → `default_action`. Policies load global → community/synced
→ project (`.rampart/policy.yaml`), and a later layer cannot weaken an
earlier one. `RAMPART_NO_PROJECT_POLICY=1` is the documented answer to an
untrusted repo.

### 2.2 Normalization before matching

Commands are base64-decoded where a literal payload is piped to a decoder,
stripped of leading shell comments, ANSI escapes, null bytes and control
characters; inner commands in `$(…)`, backticks and `eval` are matched
separately. Recent releases (1.9.3, 1.9.4, 1.9.6) analyze the whole pipeline
for fetch-and-execute: a network fetcher feeding any stage that is not a
known read-only filter is denied, including through heredocs, here-strings,
process substitution and multi-operand forms such as `chmod -R 777 a / b`.
Rule exclusions apply only to the fragment that matched, so appending a
harmless fragment cannot clear a block.

### 2.3 Fail-closed ownership

Where Rampart owns the decision, an unclassifiable or unpersisted action does
not proceed. Unknown tools are denied by default (1.9.5 had to add three new
Claude Code tool names to stop refusing them). Older docs described
fail-open; current README and changelog describe fail-closed. The Python SDK
0.2.0 also fails closed on transport errors.

### 2.4 Audit and approvals

Hash-chained JSONL, ULID event IDs, anchor every 100 records, fsync per
write, chain continuity across rotated files, `rampart audit verify`,
`rampart audit tail --follow`, `rampart audit stats --since 24h`. Records
redact "common credential shapes" before persistence and display. The README
still says the audit directory is sensitive because it holds command
structure. `ask` pauses the request in a ULID-keyed queue; `rampart pending`,
`approve <id>`, `deny <id>` resolve it; where the integration has no
resolver, `ask` becomes a block (standalone `rampart mcp` today).

### 2.5 Integrations and assurance levels

| Host | Mechanism | Rampart's own assurance label |
|---|---|---|
| Claude Code | PreToolUse + PostToolUse hooks (Bash, Read, Write, Edit, Grep, Glob, WebFetch, MCP tools) | adapter verified |
| Cursor | local Agent and Cmd+K `preToolUse` hook, fail-closed | adapter verified |
| OpenClaw | native `before_tool_call` plugin, live host verifier, native approval cards | host verified |
| Codex, Cline, Antigravity, GitHub Copilot | hook adapters | varying |
| Hermes Agent, Gemini CLI | experimental, opt-in | experimental |
| MCP servers | JSON-RPC stdio proxy with correlated request/response policy | adapter verified |
| Anything else | `rampart wrap` (cooperative, exec only) or `rampart preload` (LD_PRELOAD; gaps for static, setuid, direct-syscall and SIP binaries) | "explicitly limited" |

`rampart verify --all` and `rampart doctor` check that the installed hook
really reaches the host, and `rampart status` shows HOST VERIFIED vs ADAPTER
VERIFIED. `rampart test "<cmd>"` and `rampart policy explain` dry-run a
decision. `rampart bench` scores policy coverage against an attack corpus.
Profiles: `standard`, `paranoid`, `ci`, `yolo`; 17 macOS hardening policies
(Keychain, Gatekeeper, persistence, osascript). Community catalog:
`research-agent`, `mcp-server`, `kubernetes-ops`, `terraform-strict`,
`docker-security`, fetched with SHA-256 checks, or synced from an HTTPS git
repo with `--watch`.

### 2.6 OWASP mapping

Rampart publishes a row-by-row claim against the OWASP Top 10 for Agentic
AI and ASI01–ASI10. It marks itself "partial" on sandboxing, goal hijack,
identity, supply chain and cascading failures, and "not covered" on memory
and context poisoning. Caro should publish the same table for honesty and
for the website-claims suite to pin.

## 3. The gap table

`caro guard` shipped in v1.5.0 (spec 011, ADR-020) with the same shape:
stdin-JSON PreToolUse adapter, shadow by default, Critical → `deny`, High →
`ask`, never `allow`, redacted JSONL at `data_dir/caro/guard/decisions.jsonl`,
`caro guard report`. The decision core is the `SafetyValidator` with its
catastrophic floor (ADR-017), segment and wrapper views
(`src/safety/segments.rs`), CVE rules, and user `[[safety.custom_patterns]]`
capped at High.

| # | Rampart has | caro state | Plan |
|---|---|---|---|
| G1 | File-tool coverage: Read/Write/Edit/Grep/Glob on `~/.ssh`, `.env`, keys (1.9.5 closed Grep/Glob) | `caro guard` ignores every non-shell tool (`Outcome::NotShell`) | v1.6.0, shadow first, `ask` not `deny` |
| G2 | `rampart test "<cmd>"`, `policy explain` | none; `caro --dry-run` generates, it does not evaluate a given command | v1.6.0 `caro guard test` |
| G3 | `setup <host>`, `doctor`, `verify --all`, HOST/ADAPTER VERIFIED status | docs say hand-edit `settings.json`; `caro doctor` does not know about guard | v1.6.0 `caro guard setup`, doctor row |
| G4 | SHA-256 hash chain, anchors, `audit verify`, `audit tail --follow` | plain JSONL, mode 0600, no chain | v1.6.0 `caro guard audit` |
| G5 | PostToolUse secret-leak scan (AWS keys, private keys, bearer tokens) | redaction regexes exist in `src/guard/log.rs` but run only on logged commands | v1.6.0, shadow only |
| G6 | User policy with `deny`, path and domain globs, sessions, rate limits, `headless_only` | regex-on-command custom patterns, capped at High | Research row `guard-policy-dsl` (new product line, Gate 1) |
| G7 | base64/ANSI/comment normalization; fetch-and-execute through heredoc, here-string, process substitution; exclusions scoped to the matched fragment | segment views cover `;`, wrappers, `$()`, `eval`; no decode step (ADR-010 lists encoding as a known evasion) | v1.6.0, TDD via `safety-pattern-developer` |
| G8 | Cursor, Cline, Copilot, Gemini CLI, Hermes, OpenClaw, MCP proxy, `wrap`, `preload` | Claude, Grok Build, Codex, OpenCode, generic; #1623 (Copilot) and #928 (MCP) open | v1.6.0 issues; `wrap`/`preload` out of scope |
| G9 | Profiles and a community policy catalog with git sync | none | Research row with G6 |
| G10 | Fail-closed on unknown tools | `none` for unknown tools by design; never-`allow` is the inverse guarantee | Keep; count unknown tool names (spec 011 follow-up 6) |
| G11 | OWASP Agentic coverage table | none | docs row in v1.6.0 |

## 4. What caro does better, and keeps

- **Never `allow`.** In Claude Code an `allow` skips the user's own permission
  prompt. Rampart's `allow` rules and `default_action: allow` make it an
  auto-approver when misconfigured. Caro can only add friction, so installing
  it is monotonic (ADR-020). Parity work must not add `allow`.
- **A floor no policy can relax.** ADR-017's catastrophic floor cannot be
  allowlisted. Rampart's deny-wins is the policy-layer analogue, but a user
  who edits `custom.yaml` can still remove a deny.
- **Reason text never echoes the command.** `permissionDecisionReason` is
  Caro-authored (pattern description plus risk), which closes a
  prompt-injection echo channel. Rampart's `message` is author-written and
  static, so it is safe too, but `command_contains` matches are reported with
  the matched fragment.
- **One Rust binary, no daemon, no port, no token.** Rampart's proxy, daemon
  and preload paths widen its attack surface and its install story. The
  per-call hook spawn is Caro's whole runtime.
- **CVE rules compiled in.** Weekly NVD + CISA KEV + GHSA sync for
  shell-invocation CVEs. Rampart has no equivalent.
- **Shadow mode as the default and as the evidence instrument.** Rampart has
  `watch` as a rule action but enforces by default. Caro's shadow log is how
  it earns the right to enforce (validation-discipline Gate 1).
- **Caro also generates.** Rampart only judges. Caro's validator guards both
  the commands it writes and the commands other agents write.

## 5. What we deliberately do not copy

- `allow` rules, `default_action: allow`, and any auto-approve path.
- A long-running proxy, daemon or `LD_PRELOAD` library. Spec 011 chose a
  per-call subcommand; the hook timeout bounds the cost.
- A user-authored `deny` that bypasses the human. User patterns stay capped
  at High (`ask`) until G6 clears Gate 1. The one exception is caro's own
  decision log, which the guard owns.
- Fail-closed on *unknown* tools. Caro's contract is "no opinion" there. The
  risk is recorded in spec 011's review outcomes; the fix is to count unknown
  tool names, not to block them.
- A TUI dashboard and an approval queue. Those are control-plane features
  (section 6), not CLI features.

## 6. How the three repos divide this work

| Repo | Role today | Rampart analogue | This plan |
|---|---|---|---|
| [`wildcard/caro`](https://github.com/wildcard/caro) | The product: Rust CLI, `SafetyValidator`, `caro guard`, eval harness (`src/evaluation`, `tests/evaluation`), website, `.claude/` harness | `cmd/rampart`, `internal/engine`, `internal/intercept`, `internal/audit`, hook adapters | G1–G5, G7, G8 land here |
| [`wildcard/caro-eval`](https://github.com/wildcard/caro-eval) | External black-box campaign: Python runner (`runner/run.py`) drives the installed `caro` binary over 280 YAML queries in 28 categories inside sandbox "playgrounds", clusters failures, filed #915–#953 (April 2026, caro 1.3.0, 24.1% pass). Not run since. Not the `cargo run --bin caro-eval` that `CLAUDE.md` names (broken window BW-003) | `rampart bench` attack corpus | Follow-up: add a `guard-hook-payloads` category and re-run against 1.5.0 |
| [`wildcard/caro-control`](https://github.com/wildcard/caro-control) | Empty placeholder (one commit). The name matches the enterprise control-plane ADRs: ADR-001 enterprise/community split, ADR-002 governance and policy provisioning, ADR-003 central audit trail | approval queue, `rampart watch`, community policy sync, dashboard | Research row `caro-control-plane`; nothing is built there before Gate 1 |

The split follows Caro's positioning (`docs/GUARDIAN_AGENT.md`): execution
safety lives in the CLI; policy distribution, approval queues and
observability are the control plane, which is another product line and
another validation gate.

## 7. Engineering practices worth copying

- **Every changelog entry names the evasion it closes** ("appending a
  harmless fragment can't clear a block"). Caro's segment-evasion table in
  spec 011 does this; keep doing it in `CHANGELOG.md`.
- **Assurance labels on integrations.** "Adapter verified" vs "host verified"
  tells the user what the installer actually checked. `caro guard setup
  --verify` should print the same distinction.
- **`test` and `explain` as first-class CLI verbs.** A guardian that cannot be
  asked "what would you do with this?" is hard to trust and hard to debug.
- **A published OWASP mapping with honest "partial" and "not covered" rows.**
- **Audit integrity as a verb (`audit verify`)**, not a property claimed in
  docs.

## 8. Resulting work

- `ROADMAP.md`: v1.6.0 Guardian parity milestone; `guard-policy-dsl` and
  `caro-control-plane` research rows.
- `specs/011-harness-guardian/spec.md`: "Phase 2: parity" contracts for
  G1–G5.
- `docs/GUARDIAN_AGENT.md`: Rampart in the landscape; `caro guard` row.
- `.hermes/messages/competitor-rampart-2026-10-18.md`: Hermes adds Rampart
  and Snare to the weekly scan.
- PRs, in order: `caro guard test` + `caro guard setup` (G2, G3); file-tool
  coverage in shadow (G1); hash-chained log + `caro guard audit` (G4);
  PostToolUse leak scan in shadow (G5); normalization views and
  fetch-and-execute patterns (G7).

## Sources checked (2026-10-18)

- README and CHANGELOG at `github.com/peg/rampart` (1.9.2 to 1.9.6, all
  October 2026)
- `mintlify.wiki/peg/rampart`: `concepts/architecture`,
  `concepts/how-it-works`, `policies/writing-policies`,
  `policies/community-policies`, `cli/overview`, `security/owasp-coverage`
- Repo tree: `policies/` (`standard`, `paranoid`, `ci`, `yolo`, `guard`,
  `production-guard`, `block-prompt-injection`, `mcp-server`, `openclaw`,
  `research-agent`, `community/`, `examples/`), `docs/` (`ARCHITECTURE.md`,
  `THREAT-MODEL.md`, `PRELOAD-SPEC.md`, `API-REFERENCE.md`)

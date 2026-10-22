# Security Checklist

**APPLIES TO**: Every release PR. Every MVP-stage feature spec. Every
new external SDK integration.
**Source framework**: [Anthropic's Founder's Playbook, Stage 2 — MVP](https://claude.com/blog/the-founders-playbook)
**Pattern**: Same "grep-as-bug-report" template as
[`.claude/rules/release-version-alignment.md`](../.claude/rules/release-version-alignment.md) —
the checklist is small enough to grep and large enough to forget one.

Most items here already run in CI. This document is the **map** so
that a reviewer can confirm in 30 seconds that the checklist is
green, not by trust but by grep.

## The 9-item MVP security checklist

| # | Gate | Where it runs | How to verify |
| --- | --- | --- | --- |
| 1 | **Dependency audit** — no known CVEs in Cargo.lock | `cargo audit` in CI; `.github/workflows/` | `cargo audit --json \| grep -c '"id":' == 0` |
| 2 | **License compatibility** — every dep is AGPL-3.0-compatible | `cargo deny check licenses` in CI | `cargo deny check licenses` exits 0 |
| 3 | **Secret scanning** — no API keys / tokens / credentials in source | GitHub secret scanning + pre-commit hook | `grep -E '(api[_-]?key\|token\|secret)\s*[:=]\s*["'\''][a-zA-Z0-9]{20,}' -r src/ website/src/ docs/` finds nothing real |
| 4 | **No bundled credentials in binaries** — release artifacts contain no defaults that grant network access | `cargo dist` build + spot-check | `strings target/release/caro \| grep -iE '(sk-\|ghp_\|api[_-]key)'` returns nothing |
| 5 | **Safety pattern coverage** — 52+ dangerous-command patterns, zero false positives in regression suite | `cargo test --features safety` | `cargo test --features safety` green |
| 6 | **Sandbox boundary on execution** — bubblewrap on Linux (ADR-010), explicit user confirmation otherwise | `tests/integration/safety/` | Integration tests under `tests/safety/` green |
| 7 | **Telemetry redaction** — no command content / paths / env vars / PII leak | Privacy audit + redaction-pattern tests | `cargo test redaction` green; `docs/TELEMETRY.md` audit current |
| 8 | **Supply-chain signing** — release binaries reproducibly buildable, SHA256s published | `.github/workflows/release.yml` | `gh release view vX.Y.Z` shows checksums |
| 9 | **Constitution review** — release PR touches every file in `.claude/rules/release-version-alignment.md` | Manual PR review | All 6 release files modified |

## The MVP-stage failure modes the checklist exists to prevent

From the playbook's Stage-2 warnings:

- **"Demoware trap"** — the demo works; real-user load breaks the
  data model. *Specifically for Caro*: the safety pattern library
  fires correctly in the test suite but a single bypass (e.g. the
  P0 chmod -R 777 bypass fixed in v1.3.1) ships past testing. Gate
  5 prevents the recurrence by catching the bypass *before* the
  regression suite passes — but only if the regression suite covers
  the bypass shape. Update the suite when you fix a bypass.
- **"Mistaking building for validating"** — covered separately by
  [`.claude/rules/validation-discipline.md`](../.claude/rules/validation-discipline.md);
  this checklist assumes the validation gates are already cleared.
- **Single-provider lock-in** — playbook Stage-4 failure mode, but
  worth preempting at MVP. *Specifically for Caro*: every model
  backend goes behind the `InferenceBackend` trait, and the
  configuration system supports runtime switching. No code path
  should hard-code a single backend identifier outside
  `inference/<backend>.rs`.

## Compliance posture for the playbook's regulated-workload mentions

The playbook recommends Claude Code for SOC 2 / GDPR / HIPAA code
audits. Caro's posture per ADR-001:

| Regulation | Community Edition | Enterprise Edition |
| --- | --- | --- |
| **SOC 2** | Not applicable (single-user local tool); no managed service to audit | Required for managed deployments; tracked in [`docs/enterprise/MOAT.md`](./enterprise/MOAT.md) |
| **GDPR** | Telemetry opt-in + local-first + redaction → already compliant; see [`docs/TELEMETRY.md`](./TELEMETRY.md) | Required at the deployment surface (audit-trail forwarding, retention controls) |
| **HIPAA** | Out of scope — Caro is not for PHI workflows | Out of scope at MVP; revisit if Enterprise customers request |

The playbook's broader compliance advice ("build the compliance
workstream into your dev cycle via Cowork") explicitly does not
apply to Caro's MVP because (a) we're local-first, single-user, and
opt-in for telemetry, and (b) Cowork's audit-log limitation noted in
[`docs/agentic-stack.md`](./agentic-stack.md) makes it unsuitable
anyway. Regulated work routes through Claude Code with PR-trail
audit + ADR-003 monitoring/audit-trail per the architecture.

## How to use this checklist as a reviewer

For a release PR:

```bash
# 1. Dep audit
cargo audit && echo "✓ Gate 1"
# 2. License
cargo deny check licenses && echo "✓ Gate 2"
# 3. Secret scan
grep -rE '(api[_-]?key|token|secret)\s*[:=]\s*"[a-zA-Z0-9]{20,}' src/ website/src/ \
  | grep -v '\.test\.\|fixture\|mock' \
  | (! grep . > /dev/null) && echo "✓ Gate 3"
# 4. Binary secret check
cargo build --release --features embedded-cpu
strings target/release/caro | grep -iE '(sk-|ghp_|api[_-]key)=[a-zA-Z0-9]' \
  | (! grep . > /dev/null) && echo "✓ Gate 4"
# 5. Safety suite
cargo test --features safety && echo "✓ Gate 5"
# 6. Sandbox tests
cargo test --test 'safety_*' && echo "✓ Gate 6"
# 7. Redaction tests
cargo test redaction && echo "✓ Gate 7"
# 8. Release checksums (post-tag)
gh release view "vX.Y.Z" --json assets --jq '.assets[].digest' && echo "✓ Gate 8"
# 9. Release-PR file alignment
test "$(git diff --name-only main...HEAD | grep -cE 'Cargo.toml|Cargo.lock|CHANGELOG.md|README.md|ROADMAP.md|homebrew-tap|install.ps1|install.sh')" -ge 6 \
  && echo "✓ Gate 9"
```

A reviewer who can paste those 9 commands and see 9 ✓ outputs has
verified the checklist.

## When a gate fails

| Gate | Failure mode | Action |
| --- | --- | --- |
| 1 | New CVE in a dep | Bump the dep; if no upstream fix, evaluate pinning + sandboxing |
| 2 | New dep with GPL / non-compat license | Per `.claude/rules/external-sdk-integration.md`, this is caught at spike-PR stage; if it slipped through, replace the dep |
| 3 | Secret in source | Rotate the secret immediately; rewrite history with care; add the pattern to pre-commit |
| 4 | Secret in binary | Same as gate 3 + verify no published release has the issue (re-release if needed) |
| 5 | Safety regression | Per `safety-pattern-developer` skill: red-green-refactor; do NOT release until green |
| 6 | Sandbox boundary leak | Per ADR-010; treat as P0 |
| 7 | Telemetry redaction failure | Per `docs/TELEMETRY.md` audit; treat as P0 (privacy promise) |
| 8 | Signing pipeline broken | Per `caro-release-expert` agent + `.claude/rules/release-version-alignment.md` |
| 9 | Release-PR file alignment | Per `.claude/rules/release-version-alignment.md`; align before merge |

## What this checklist does NOT cover

- **User input validation**: not applicable in the usual web-app
  sense — Caro takes shell-command prompts; "input validation"
  *is* the safety pattern library. Covered by Gate 5.
- **AuthN/AuthZ**: not applicable — Caro is a local single-user CLI.
  The Enterprise edition (ADR-001) introduces these and gets its
  own checklist when it ships.
- **Penetration testing**: out of scope at MVP. Revisit at Stage-4
  scale.
- **Bug bounty**: not yet. Pre-condition: clear public security
  policy + retention dashboard so we can scope active-user impact.

## See also

- [`.claude/rules/release-version-alignment.md`](../.claude/rules/release-version-alignment.md) —
  the template-of-template this checklist follows
- [`.claude/rules/external-sdk-integration.md`](../.claude/rules/external-sdk-integration.md) —
  the build-spike rule that prevents license / MSRV / transitive
  surprises before they reach this checklist
- [`docs/TELEMETRY.md`](./TELEMETRY.md) — the redaction contract Gate 7
  enforces
- [`docs/adr/ADR-010-bubblewrap-sandbox-execution.md`](./adr/ADR-010-bubblewrap-sandbox-execution.md) —
  Linux sandbox decision (Gate 6)
- [`docs/adr/ADR-001-enterprise-community-architecture.md`](./adr/ADR-001-enterprise-community-architecture.md) —
  where the regulated-workload checklist lives when Enterprise ships

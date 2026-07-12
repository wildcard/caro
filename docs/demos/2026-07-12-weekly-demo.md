# Weekly Demo — 2026-07-12 (week of Jul 6–12)

Per [`.claude/rules/feature-evidence.md`](../../.claude/rules/feature-evidence.md):
every feature merged this week, with demo, evidence, and regression guard.

## 1. Backend roster: single source of truth (PR #1298, fixes #1115)

**What shipped**: `--backend-info` used to advertise backends
(`static`, `claude`) that `--backend <name>` then rejected with
"Unknown backend" — four separate rosters had drifted. All CLI
surfaces now iterate one `backends::CLI_SERVABLE_BACKENDS` const, and
`--backend-info` is feature-aware: in the default build, remote
backends show `not compiled` with a `--features remote-backends`
hint instead of implying they work.

**Demo**:
```bash
caro --backend-info
# → embedded: available; ollama/vllm/…: not compiled (--features remote-backends)
caro --backend static "list files"
# → clean error naming the servable roster (previously: advertised-but-rejected)
```

**Evidence**: PR [#1298](https://github.com/wildcard/caro/pull/1298),
merged 2026-07-11 with green CI (merge commit `2a76cf9`).

**Regression guard**: roster-consistency tests added in #1298 pin that
every advertised backend is accepted and vice versa (see
`src/backends/mod.rs` roster tests / `tests/cli_flag_tests.rs`).

## 2. Safety: catastrophic floor + allowlist regression fix (PR #1246 — merging today)

**What shipped**: Repairs the #1110 regression where the
"Critical is never allowlistable" guard over-matched, so a deliberate
narrow allowlist (`rm -rf /tmp/myapp_\d+`) could no longer bless
`rm -rf /tmp/myapp_123`. New `targets_catastrophic_location()` floor
force-blocks catastrophic targets (system dirs, disk devices, ZFS/LVM,
reverse shells, remote-exec-as-root, fork bomb — evasion-hardened,
quote-aware, sudo-prefix-aware) while allowing specific-subpath
allowlists to work as designed. Also de-flakes `test_cache_roundtrip`
via an isolated cache env override.

**Demo**:
```bash
caro --dry-run "delete everything under root"   # → BLOCKED (Critical), allowlist cannot override
caro --dry-run "rm -rf /tmp/myapp_123"          # → with matching allowlist entry: allowed
```

**Evidence**: PR [#1246](https://github.com/wildcard/caro/pull/1246);
all Rust checks green on run
[27532336756](https://github.com/wildcard/caro/actions/runs/27532336756)
(unit ubuntu+windows, smoke ×2 OS, safety regression, security audit,
CodeQL, 3-target builds).

**Regression guard**: `tests/safety_validator_contract.rs` (21 tests)
plus new `allowlist_catastrophic_tests` module —
`allowlist_cannot_reenable_catastrophe`, `evasions_are_closed`,
`cross_reference_every_critical_class_is_covered`,
`floor_regexes_all_compile`.

## 3. Process: weekly planning + autonomous-ops framework (PRs #1297, this one)

**What shipped**: 2026-07-06 planning report + release-state memory
(merged); decision record D1–D5, this demo report, and the
feature-evidence rule (this PR).

**Evidence**: `.claude/memory/v200-weekly-report-2026-07-06.md`,
`docs/decisions/2026-07-12-autonomous-mode-release-scope.md`.

## Known red (not regressions, tracked)

- **Vercel caro-foss-website deploy** fails on every commit —
  pre-existing Astro 5→6 multi-package migration, root-caused in
  #1246's body, deliberately out of CI-repair scope. Needs an owner.
- **Sandbox-only**: `tests/cli_interface_contract.rs` fails locally in
  the agent sandbox (no embedded model); green in CI on both OSes —
  see decision D3.

## Next week

- v1.5.0 release (decision D1) — in flight this session.
- Discovery interviews for Self-Healing (#1151) and Local Context
  Indexing (#1152) remain the top unblocked milestone work.

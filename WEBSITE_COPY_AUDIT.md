# Website Copy Audit: caro.sh (v1.2)

**Date**: March 26, 2026
**Auditor**: Automated (v1.2-close-credibility-gap PR)
**Scope**: All pages on caro.sh, docs.caro.sh, and Claude Skill documentation

## Summary

Audit of all feature claims against caro v1.2 implementation (post WP01-09). Every claim marked ✅ has been verified to work. Claims marked ⚠️ need website copy updates (not code changes).

---

## /faq — Required Updates

| Current Text | Status | Fix |
|-------------|--------|-----|
| `--quiet` flag example | ✅ Fixed | Works in v1.2 |
| `-e` short flag example | ✅ Fixed | Works in v1.2 |
| `--no-telemetry` flag example | ✅ Fixed | Works in v1.2 |
| `caro telemetry show` example | ✅ Fixed | Works in v1.2 |
| `caro telemetry export -o` example | ✅ Fixed | Works in v1.2 |
| `telemetry.air_gapped` config key | ✅ Fixed | Works in v1.2 |
| `CARO_TELEMETRY_ENABLED` env var | ✅ Fixed | Works in v1.2 |
| `--model` flag | ⚠️ **Fix needed** | Should say `--model-name` / `-m`, not `--model` |
| Cloud backends (Anthropic, OpenAI) | ⚠️ **Clarify** | Note that Claude backend requires `--features remote-backends` |

## /telemetry — Status

| Claim | Status |
|-------|--------|
| `caro telemetry show` | ✅ Works |
| `caro telemetry export -o telemetry-data.json` | ✅ Works |
| `caro --no-telemetry` | ✅ Works |
| `caro config set telemetry.air_gapped true` | ✅ Works |
| `export CARO_TELEMETRY_ENABLED=false` | ✅ Works |
| Telemetry disabled by default | ✅ Accurate |
| Air-gapped mode | ✅ Works |

## /explore — Required Updates

| Claim | Status | Fix |
|-------|--------|-----|
| MLX inference <2s | ⚠️ **Fix needed** | Actual: ~4s. Update to "~4s" or "typically 2-5s" |
| MLX startup <100ms | ✅ Accurate | 14ms measured |
| Memory ~2GB with 7B model | ⚠️ **Fix needed** | Default is 1.5B (~1.1GB). Either update or note 7B is optional |
| MCP server integration | ⚠️ **Fix needed** | Listed as current option. Should say "Coming Soon" |
| "50+ Dangerous Patterns" | ⚠️ **Fix needed** | Inconsistent — code says "52+". Use "52+" |

## /compare — Status

| Claim | Status |
|-------|--------|
| Privacy-first | ✅ Accurate |
| 52+ safety patterns | ✅ Accurate |
| 100% offline | ✅ Accurate (embedded backend) |
| Rust-built | ✅ Accurate |
| Rule-based safety | ✅ Accurate |
| Customizable safety rules | ⚠️ **Clarify** | `custom_patterns` exists in code but not exposed to CLI yet. Mark as "Planned" |

## /roadmap — Required Updates

| Issue | Fix |
|-------|-----|
| All v1.1.x items at 0% | Mark completed items as done |
| v1.2.0 at 0% | Update with current progress |

## Skill Documentation — Status

| Issue | Status |
|-------|--------|
| `cargo install Caro` (capital C) | ✅ Fixed |
| Fabricated config TOML | ✅ Fixed |
| Claude Haiku 4.5 default backend | ✅ Fixed |
| Keyboard shortcuts e/explain, s/safer | ✅ Removed |
| Disambiguation UX | ✅ Removed |
| Tool-not-found suggestions | ✅ Removed |

## docs.caro.sh — Required Updates

| Claim | Status | Fix |
|-------|--------|-----|
| `caro assess` | ⚠️ **Remove** | Commented out in code |
| `caro knowledge` | ⚠️ **Remove** | Feature-gated, not available |
| `caro profile` | ⚠️ **Remove** | Feature-gated, not available |
| `--target linux` flag | ⚠️ **Remove** | Does not exist |
| Interactive y/n by default | ⚠️ **Clarify** | Only works in TTY. In non-TTY, shows guidance message |

## Action Items

These are copy changes needed on the website, NOT code changes:

1. [ ] `/faq`: Fix `--model` → `--model-name`
2. [ ] `/explore`: Update MLX inference time to "~4s"
3. [ ] `/explore`: Update memory claim or mark 7B as optional
4. [ ] `/explore`: Mark MCP as "Coming Soon"
5. [ ] `/explore`: Fix "50+" → "52+"
6. [ ] `/compare`: Mark "Customizable safety rules" as "Planned"
7. [ ] `/roadmap`: Update v1.1.x and v1.2.0 completion status
8. [ ] docs.caro.sh: Remove `caro assess`, `caro knowledge`, `caro profile`
9. [ ] docs.caro.sh: Fix `--target linux` reference
10. [ ] docs.caro.sh: Clarify interactive confirmation TTY requirement

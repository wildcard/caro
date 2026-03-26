# Master Plan: v1.2.0 — Close the Credibility Gap

**Branch**: `v1.2-close-credibility-gap`
**Date**: March 26, 2026
**Milestone**: v1.2.0 (Due: April 30, 2026)
**Tracking Issue**: #792

---

## Context

Two comprehensive audits (#790, #791) revealed that **40+ features advertised on caro.sh, docs.caro.sh, and the Claude Skill do not work**. This is the single PR that brings the product to match its marketing.

## Constraints

- **#681 is assigned to Claude Code** — do NOT touch merge conflicts
- **Work on a single PR** that mitigates all findings
- **Spec-driven development** — every task traces to a spec
- **No new features** — only implement what's already documented

---

## 5 Phases

### Phase 1: CLI Flags & Telemetry (P0 — foundational)

Implement all documented flags and subcommands that produce errors today.

| Task | GitHub Issue | Effort |
|------|-------------|--------|
| `--quiet` flag | #793 | S |
| `-e` short flag for `--execute` | #793 | XS |
| `--no-telemetry` flag | #793 | S |
| `--backend-info` flag | #793 | S |
| `--explain` flag | #793 | S |
| `caro telemetry show` | #794 | M |
| `caro telemetry export -o` | #794 | M |
| Config key expansion (14 keys) | #795 | L |
| Env var overrides | #795 | M |

### Phase 2: Command Quality (P1 — core value)

Stop generating `ls -la` and `echo 'Unable to generate command'` for common queries.

| Task | GitHub Issue | Effort |
|------|-------------|--------|
| Static matcher +30-50 patterns | #797 | L |
| Embedded model prompt rewrite | #798 | L |
| Fallback chain (model → static → error) | #798 | M |
| PowerShell `--shell powershell` | #800 | M |

### Phase 3: Safety & UX (P1 — trust)

Make the safety system behave as the docs describe.

| Task | GitHub Issue | Effort |
|------|-------------|--------|
| Safer alternatives when blocking | #796 | L |
| Color-coded output (🟢🟡🟠🔴) | #799 | M |
| `confidence_score` in JSON | #799 | S |
| Interactive confirmation UX | #799 | S |

### Phase 4: Docs & Skill (P2 — alignment)

Rewrite all docs to match actual behavior.

| Task | GitHub Issue | Effort |
|------|-------------|--------|
| Skill SKILL.md rewrite | #801 | M |
| Skill README.md rewrite | #801 | S |
| Skill QUICK_START.md rewrite | #801 | S |
| Skill examples/basic-usage.md rewrite | #801 | S |
| Fix check-caro-installed.sh | #801 | XS |
| Website copy audit | #802 | M |

### Phase 5: Release (P3 — ship it)

| Task | Effort |
|------|--------|
| Update CHANGELOG.md | S |
| Bump version in site.ts | XS |
| Run full test suite | S |
| Run clippy + audit | S |

---

## Acceptance

This PR is ready when:

1. **Zero errors**: Every example on caro.sh/faq, caro.sh/telemetry produces the described behavior
2. **Every flag works**: `--quiet`, `-e`, `--no-telemetry`, `--backend-info`, `--explain` all function
3. **Every config key works**: 14/14 documented keys accepted
4. **Every telemetry subcommand works**: `show`, `export`
5. **Alternatives populated**: `alternatives` field populated for blocked commands
6. **Static matcher improved**: Top 20 common queries generate correct commands
7. **PowerShell works**: `--shell powershell` generates Windows syntax
8. **Color output works**: Colored in TTY, plain when piped
9. **Docs match reality**: All skill docs reference working features only
10. **Tests pass**: `cargo test`, `cargo clippy -D warnings`, `cargo audit`

---

## Risks

| Risk | Mitigation |
|------|------------|
| Telemetry code needs more than "uncomment" | Spike first; worst case implement minimal version |
| Config expansion breaks existing configs | Backward compat: old keys still work |
| Prompt rewrite doesn't improve model quality | Static matcher expansion as safety net |
| PR too large to review | Break into logical commits per phase |

---

## References

- #790 — README gap analysis
- #791 — website gap analysis
- #792 — master tracking issue
- #681 — merge conflicts (assigned to Claude Code, do NOT touch)
- `specs/v1.2-delivering-on-the-promise/tech-spec.md`
- `specs/v1.2-delivering-on-the-promise/product-spec.md`
- `specs/v1.2-delivering-on-the-promise/appendix-spec.md`

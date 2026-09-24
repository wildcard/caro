# caro-research--scoping-process — Run Report

**Date**: 2026-08-03
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-044 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]`
placeholder (outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-21) and the ADR directory. Latest
   ADR on disk is ADR-043 (MCP safety gateway proxy). Next number: 044.
2. Selection signal: the Hermes **2026-08-03 Product Hunt scan** names
   **"fail-closed capability resolution for validated commands"** as
   opportunity B (priority Next, complexity M) with the explicit next
   step "spec an 'effects resolution' layer for the top 20 dangerous
   command families." Opportunities A/E are positioning work (outside
   this task's charter), C is gated behind 20-transcript discovery per
   `validation-discipline.md`, and D is substantially covered by
   ADR-041. No existing ADR covers effects resolution (verified by grep).
3. Mapped `[FEATURE NAME]` to **Cynative's action gate**
   ([cynative/cynative](https://github.com/cynative/cynative), Apache-2.0,
   Go, v1.0.0 2026-06-24) — "every operation is resolved to its required
   IAM actions, derived from the providers' own API definitions, then
   authorized against a read-only policy before any credential is
   attached … the gate fails closed on anything it classifies as a
   write." Fetched the GitHub README plus launch coverage (Product Hunt,
   Help Net Security).
4. Ran a codebase survey (Explore agent): confirmed the validator is
   pure regex over command text with **no notion of effects** (paths /
   hosts / privileges), that `DangerPattern` lacks stable ids (CVE rules
   have them via the dogma compiler), that the headless envelope /
   `SafetyAssessmentOutput` are still ADR-only (not implemented), and
   that exit codes 0–11 are allocated by ADRs 024/027/029/034/039/040.

## Phase 1 findings (Cynative)

- **Construction**: resolve → authorize → attach, with the mapping
  derived from authoritative provider API definitions; unmappable ⇒
  denied. Host/IP pinning, sandboxed JS with no ambient access,
  STS-scoped credential re-vending, fail-closed JSONL audit log.
- **Failure modes**: guarantee rides on mapping-corpus freshness
  (coverage gaps become availability failures); unit of analysis is a
  structured API call — the unstructured-shell version of the problem is
  unsolved; coarse per-tool "always allow" approvals; audit log stores
  approval arguments verbatim.
- **Lifecycle**: pure subprocess in `-p` mode, no daemon, ambient
  credential discovery per run, corpus compiled in — matches caro's
  constraints one-for-one.

## New ADR Produced

**ADR-044** — `docs/adr/ADR-044-fail-closed-effects-resolution.md`

- **Effects corpus as command families** (`data/effect_families/*.yaml`
  → bincode via the existing dogma compiler; capture-group/flag-table
  extraction, no shell AST in v1). Initial cut: 20 dangerous families +
  one shared read-only family.
- **New serializable types in `src/safety/mod.rs`** (no new module):
  `EffectSet`, `EffectResolution`, `ResolutionStatus`, closed
  `PathClass`/`HostClass` enums; additive `effects:` field on
  `ValidationResult`/`SafetyDecision`/envelope (schema_version stays 1).
- **Fail-closed is opt-in** (`--fail-closed` or ADR-040 policy key
  `unresolved = allow|approve|block`, ceiling-pinnable). Cynative's
  corpus-gap failure mode is solved by design: unresolved ⇒ deterministic
  escalation with a machine-readable reason — never a silent allow,
  never a hard availability wall (default escalation is `approve`).
  Critical floor preserved: effects escalate, never relax.
- **Contract**: new exit 12 (`UnresolvedBlocked`) only; 0–11 untouched;
  `caro effects` subcommand emits deterministic JSON; 8 integration
  tests specified (known input → JSON + exit code), incl. a golden-file
  determinism gate. Zero-false-positive guarantee stays binding for the
  default open-world mode.
- **Out of scope**: shell AST (would trigger the external-SDK
  build-spike rule), path-scoped policy rules, class grounding via
  FS/DNS, corpus auto-update, Windows shells.

## Delivery Notes

- Files written to the working tree only — **no commits** (git-workflow
  rule: never commit to main; no user present to open a branch/PR).
  Next session: `bin/sk-new-feature "adr-044 effects resolution"`,
  move both files in, open PR per ADR-numbering rule (renumber if
  another ADR PR lands first).
- Reviewable assumptions flagged in the ADR: the exact 20-family cut
  (D3) and the `approve` default for `unresolved`.
- The headless envelope (ADR-024) is still unimplemented; ADR-044 only
  assumes its *contract*, not its code — implementation order is
  envelope-first or flag-gated JSON-only, decided at implementation PR.

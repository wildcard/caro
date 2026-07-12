# Caro Constitution

This file takes precedence over every other file in `.claude/rules/`.

Read and obey the following subordinate documents in order. **If two
subordinate documents conflict, the earlier one wins.**

Borrowed from Uncle Bob's swarm-forge `constitution.prompt` pattern: a
flat pile of rules has no precedence; a layered constitution does.

---

## Tier 1 — Project safety (highest priority)

These rules protect the repository and the parallel-session workflow. They
override anything below them.

1. **[git-workflow.md](./git-workflow.md)** — Feature branches are mandatory.
   Never commit to `main`. Enforced by a PreToolUse hook.

## Tier 2 — Engineering discipline

These rules govern how code is written, reviewed, and released.

2. **[dev-process.md](./dev-process.md)** — Branch / PR / CI workflow,
   conventional commits, code style.
3. **[release-version-alignment.md](./release-version-alignment.md)** — The
   6-file release checklist; every release PR touches all six files.
4. **[validation-discipline.md](./validation-discipline.md)** — Evidence
   requirements before a new product line graduates from research to
   implementation (20 transcripts, no surveys-only, demoware-trap gate,
   devil's-advocate review, Sean Ellis with defended cohort). Adapted
   from [Anthropic's Founder's Playbook](https://claude.com/blog/the-founders-playbook).
5. **[adr-numbering.md](./adr-numbering.md)** — ADRs are sequential, no gaps;
   renumber on merge if PRs land out of order.
6. **[external-sdk-integration.md](./external-sdk-integration.md)** —
   First PR for any non-trivial external SDK is a build-spike (license,
   MSRV, optional feature flag, code-ref smoke, two verification builds);
   architecture lands only after the spike merges green.
7. **[feature-evidence.md](./feature-evidence.md)** — Every feature PR
   carries evidence (green CI run link), a runnable demo, and a named
   regression-guard test; the Monday planning routine compiles merged
   demos into `docs/demos/<date>-weekly-demo.md`.

## Tier 3 — Workflow hygiene

These rules govern session ergonomics and surface-level conventions. They
yield to anything in Tier 1 or Tier 2.

8. **[good-boy-scout.md](./good-boy-scout.md)** — Leave code better than you
   found it; do not gold-plate.
9. **[quick-actions-footer.md](./quick-actions-footer.md)** — Append a
   Quick-Actions footer when stopping for user input.
10. **[astro-esbuild-shell-syntax.md](./astro-esbuild-shell-syntax.md)** —
   Escape `{` in shell snippets inside `.astro` / `.jsx` / `.tsx` templates
   (esbuild treats it as a JSX expression boundary).

---

## How to use this file

- A new agent or session should treat this constitution as the entry point
  for `.claude/rules/`. Read this file first; it tells you which rules
  exist and which one wins on a tie.
- When you find that two rules genuinely conflict, follow the higher-tier
  one and open a PR amending the lower-tier one so the conflict goes away.
- When adding a new rule, place it in the correct tier and update this
  index in the same PR — orphan rules default to no precedence and
  therefore get ignored under contention.

## Why this exists

Inspired by [swarm-forge](https://github.com/unclebob/swarm-forge)'s
constitution layering. With 8 rule files and 4–5 parallel Claude sessions,
"first one I happen to load" is not a deterministic conflict resolution
strategy; explicit precedence is.

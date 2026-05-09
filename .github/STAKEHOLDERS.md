# STAKEHOLDERS — How agent routing works

`.github/STAKEHOLDERS.yml` is the granular ownership map for Caro. It pairs
each area of the codebase with:

- **`humans`** — GitHub handles, mirrored into `.github/CODEOWNERS` for
  GitHub-native review enforcement.
- **`agents`** — specialist agents from `.claude/agents/` (or skills from
  `.claude/skills/`) that should be auto-selected when picking up work in
  that area.
- **`review_label`** — optional GitHub label applied to PRs touching this
  area, so humans and bots can filter their queue.

Inspired by
[`warpdotdev/oz-for-oss`](https://github.com/warpdotdev/oz-for-oss)
which uses an analogous file to route Oz cloud agents.

## Why two files (CODEOWNERS *and* STAKEHOLDERS)?

| Concern | CODEOWNERS | STAKEHOLDERS.yml |
|---|---|---|
| GitHub-native review enforcement | ✅ | ❌ |
| Per-area human reviewers | ✅ | ✅ (mirrored) |
| Per-area **agent** routing | ❌ | ✅ |
| Review labels | ❌ | ✅ |
| Description / why this area matters | ❌ | ✅ |

CODEOWNERS is consumed by GitHub. STAKEHOLDERS.yml is consumed by Caro's
own automation (today: planned; tomorrow: integrated into
`caro-coder-loop` and `caro-backlog-groom`).

## Consumers (planned)

- **`.claude/skills/caro-coder-loop`** — when claiming a beads task, parse
  the touched paths, look up `agents:` in STAKEHOLDERS.yml, and spawn the
  matching specialist instead of always defaulting to a generic coder.
- **`.claude/skills/caro-backlog-groom`** — when grooming an issue into a
  bead, attach the suggested `agents:` to the bead metadata so downstream
  schedulers don't have to recompute.
- **Future `.github/workflows/auto-label-by-area.yml`** — apply
  `review_label` to PRs that touch the area, after CODEOWNERS routing.

## Resolution rules

- **Glob match**: standard CODEOWNERS-style globs.
- **Last-match-wins**: when a PR touches multiple matching areas, the
  *most specific* glob (longest non-wildcard prefix) wins for agent
  selection. CODEOWNERS uses last-match-wins, so order critical rules
  late in `CODEOWNERS`.
- **Default**: the top-level `default:` block applies when nothing else
  matches.

## Adding a new area

1. Append a block to `.github/STAKEHOLDERS.yml` in alphabetical glob
   order.
2. If the area is safety- or release-critical, mirror the `humans:` row
   into `.github/CODEOWNERS`.
3. Reference an existing agent from `.claude/agents/` if possible;
   propose a new one via `.claude/agent-profiles.yaml` only if no
   existing specialist fits.
4. Pick a `review_label` from `.github/labels.yml` or add a new one
   there.

## Audit

- `grep -r '@wildcard' .github/CODEOWNERS .github/STAKEHOLDERS.yml` —
  confirm both files agree on humans for safety/release paths.
- Run `cargo run --bin caro-eval` after touching `src/safety/**` to
  confirm the safety-pattern-developer agent's TDD discipline held.

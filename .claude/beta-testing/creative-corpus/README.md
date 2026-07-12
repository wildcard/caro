# Creative Corpus

Working corpus of agent-generated natural-language queries used to stress-test caro and grow eval coverage.

**Owner**: `.claude/agents/creative-query-generator.md` — runs daily at 4am via `.claude/automation/config/schedule.yaml`.

## Layout

```
creative-corpus/
├── README.md                      ← you are here
├── seed-allowlist.yaml            ← vetted sources for daily seeding (hand-edited)
├── YYYY-MM-DD.yaml                ← daily generated queries (agent-written)
├── YYYY-MM-DD-execution-plan.md   ← daily backend-routing record (agent-written)
└── archive/
    └── YYYY-MM.yaml               ← month-rollups of older daily files
```

Daily logs (cycle reports) live at `../cycles/creative-YYYY-MM-DD.md` to mirror the existing `beta-testing/cycles/` convention.

## Schema (daily YAML)

Mirrors the canonical JSON schema in `tests/evaluation/datasets/correctness/*.json`, with extra fields for traceability:

```yaml
test_cases:
  - id: creative-2026-04-26-001
    prompt: "find every photo larger than 5mb modified this week"
    expected_command: 'find . -type f \( -iname "*.jpg" -o -iname "*.png" \) -size +5M -mtime -7'
    category: file-ops
    risk_level: safe
    posix_compliant: true
    tags: [conversational, multi-condition, find]
    seed_source: "man:find"
    validation_rule: command_equivalence
    difficulty: medium

  - id: creative-2026-04-26-002
    prompt: "make this thing run forever"
    expected_command: null
    category: ambiguous
    risk_level: unknown
    posix_compliant: null
    tags: [under-specified, ambiguous-referent]
    seed_source: "tldr:while"
    validation_rule: needs_human_review
    difficulty: hard
```

### Extra fields beyond canonical

- `seed_source` — provenance string ("man:find", "tldr:rsync", "allowlist:tldr-pages#a3f", "weak-spot:bsd-flags")
- `validation_rule` — one of `exact_match`, `command_equivalence`, `pattern_match`, `must_be_blocked`, `needs_human_review`
- `difficulty` — `easy` | `medium` | `hard`

## Promotion Path (draft YAML → canonical JSON)

When a maintainer reviews a daily PR and wants to graduate a verified entry into the canonical eval suite:

1. Translate the entry to canonical JSON shape (drop `seed_source`, `validation_rule`, `difficulty`).
2. Append it to the appropriate file under `tests/evaluation/datasets/<category>/*.json`.
3. Remove it from the daily YAML (or leave it — the corpus is a historical record, not a deduplicated source of truth).
4. Run `cargo test --test test_correctness` to confirm the canonical suite still passes (or update `expected_command` if behavior diverged).

## Retention

- Last 90 days kept as daily YAML files.
- Older files rolled into `archive/YYYY-MM.yaml` monthly (manual or via a future rollup task).

## How to disable the agent

Edit `.claude/automation/config/schedule.yaml` and set `enabled: false` on the `creative_query_generator` entry. The corpus directory and existing logs remain untouched.

---
name: caro.security.update
description: Maintainer skill to draft a CVE rule YAML via Nimble for a specific CVE, fill pattern + test_cases, and open a PR. Used off-cycle when a 0day lands between weekly cron runs.
---

# caro.security.update Skill

**Purpose**: Manually trigger the Nimble CVE authoring pipeline for a single
CVE (or a batch) outside the weekly cron window. Use this when a high-severity
0day is disclosed and protection shouldn't wait until Monday 09:00 UTC.

**Spec**: `specs/010-nimble-cve-pipeline/`
**Companion workflow**: `.github/workflows/cve-rule-sync.yml`
**Orchestrator script**: `scripts/nimble-cve-sync.ts`

---

## When to Use

- A CVE or 0day lands with CVSS ≥ 7.0 and a plausible shell-invocation vector
- The weekly cron won't run for > 24h and the exposure window matters
- You want to re-author an existing rule after the advisory was amended
- You are reviewing an auto-drafted PR and want to fill the `TODO_NIMBLE_AUTHORED`
  placeholder without leaving the CLI

**Do not use** for:
- Non-shell CVEs (daemons, kernels, libraries without a shell attack surface)
- User-local custom rules — that's Dogma tier-3 (`~/.caro/rules/`)
- Generic pattern additions not tied to a CVE — use `safety-pattern-developer` instead

---

## Prerequisites

- Maintainer access to `wildcard/caro`
- Working `gh` CLI auth
- `scripts/node_modules/` installed (`cd scripts && npm ci`)
- Nimble access: either `NIMBLE_API_KEY` in env for `nimble` CLI, OR
  the `nimble-agents` MCP server available in the Claude session
- Optional: `NVD_API_KEY` env var to raise NVD rate limit

---

## Workflow

### Phase 1 — Identify the CVE (2 min)

Confirm the CVE ID and authoritative source. Default to the NVD detail page:
`https://nvd.nist.gov/vuln/detail/CVE-YYYY-NNNNN`. If the CVE isn't in NVD
yet (freshly-assigned MITRE IDs can lag 24-48h), use the upstream advisory.

Capture:

- **CVE ID** (format `CVE-YYYY-NNNNN`)
- **Advisory URL** (NVD preferred; GHSA / vendor page acceptable)
- **Disclosure date** (ISO `YYYY-MM-DD`)
- **CVSS base score**
- **Affected shell tool(s)** — if none, stop here; this skill is out of scope.

### Phase 2 — Fetch + draft (1 min)

From repo root on a fresh feature branch:

```bash
bin/sk-new-feature "security cve <cve-id>"
cd .worktrees/NNN-security-cve-<cve-id>

# Fetch the specific CVE window from NVD + KEV + GHSA.
# Use a 2-day window centered on disclosure for max recall.
npx -w scripts tsx scripts/nimble-cve-sync.ts \
  --since $(date -d '<disclosed> - 1 day' +%Y-%m-%d) \
  --max 5
```

Check that `data/cve_rules/CVE-YYYY-NNNNN.yaml` exists. If the sync didn't
produce it (CVE not yet in any feed), **hand-draft** the skeleton:

```bash
cp data/cve_rules/EXAMPLE-TEMPLATE.yaml data/cve_rules/CVE-YYYY-NNNNN.yaml
$EDITOR data/cve_rules/CVE-YYYY-NNNNN.yaml
```

### Phase 3 — Author the pattern via Nimble (5-15 min)

The draft YAML has `pattern: TODO_NIMBLE_AUTHORED` and `test_cases: []`.
These are the model-authored parts. Two paths:

**Path A — Nimble MCP (preferred inside Claude Code):**

Ask Claude with the `nimble-agents` MCP available:

> Use `nimble_extract` on the CVE advisory URL, then
> `nimble_agents_run` with the `cve-pattern-author` agent
> (or `nimble_search` for advisories that need synthesis)
> to propose:
> 1. A regex `pattern` that matches exploit invocations
>    without false-positiving on benign shell usage
> 2. At least 2 `test_cases` with `expected_behavior: Block`
>    that exercise the exploit trigger
> 3. At least 2 `test_cases` with `expected_behavior: Allow`
>    that exercise nearby benign commands
>
> Write the result directly into `data/cve_rules/CVE-YYYY-NNNNN.yaml`.

**Path B — Nimble CLI (scripting / CI):**

```bash
nimble extract "https://nvd.nist.gov/vuln/detail/CVE-YYYY-NNNNN" \
  --focus academic \
  > /tmp/cve-context.md
# Feed /tmp/cve-context.md to your pattern-generation prompt, then
# edit the YAML file with the proposed pattern + test_cases.
```

### Phase 4 — Validate (30 sec)

```bash
npx -w scripts tsx scripts/validate-cve-yaml.ts \
  data/cve_rules/CVE-YYYY-NNNNN.yaml
```

Must exit 0. Common failures:

| Error | Fix |
|---|---|
| `risk_level — must be one of [safe, moderate, high, critical]` | Lowercase only |
| `test_cases — must contain at least one expected_behavior: Allow case` | Add a benign command that must NOT be blocked |
| `id — must match filename stem` | Filename and YAML `id:` must agree |
| `pattern — not a valid regex` | Escape `.`, `$`, `|`; test in a regex playground |

### Phase 5 — Local smoke test (1 min)

Build with the new rule and confirm it blocks the exploit:

```bash
cargo build --release --features cve-rules
./target/release/caro --version | grep 'CVE rules:'

# Replace with a Block test case from your YAML:
./target/release/caro --dry-run "<exploit invocation>"
# Expect: ✗ BLOCKED with the CVE ID in the reason string.

# And a benign Allow test case:
./target/release/caro --dry-run "<benign invocation>"
# Expect: ✓ allowed (not blocked by the new rule).
```

### Phase 6 — Open PR (2 min)

```bash
git add data/cve_rules/CVE-YYYY-NNNNN.yaml
git commit -m "feat(cve-rules): add CVE-YYYY-NNNNN pattern

Adds pattern for <one-line CVE summary>. CVSS <score>.
Source: <advisory URL>.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
git push -u origin HEAD

gh pr create \
  --title "feat(cve-rules): add CVE-YYYY-NNNNN pattern" \
  --label "cve-rules" \
  --label "security" \
  --body "..."
```

PR body must include:

- **CVE ID + source URL**
- **CVSS score**
- **Why this can't wait for weekly cron** (if using this skill for urgency)
- **The pattern** and a one-line rationale for regex shape
- **Block + Allow test cases** with expected behaviors
- **Sign-off checklist** from `data/cve_rules/README.md`

### Phase 7 — Request review

Maintainer review required before merge. No auto-merge, even at CVSS ≥ 9 —
a false-positive pattern blocks all users until the next release.

---

## Troubleshooting

| Problem | Likely cause | Resolution |
|---|---|---|
| Sync returns 0 candidates for a known CVE | Feed propagation lag (NVD can be 24-48h behind MITRE) | Hand-draft from template |
| Pattern matches benign commands in ad-hoc testing | Too broad | Tighten the regex; add the benign command to `test_cases` with `Allow` and re-test |
| `cargo build` fails with `CVE_COMPILED` deserialize error | Stale `$OUT_DIR/cve_patterns.bin` | `cargo clean -p caro && cargo build` |
| `--version` doesn't show new count | Feature flag off | Build with `--features cve-rules` (default-on) |
| Validator passes but `cargo test safety` regresses | Pattern collides with a static rule | Dedup in `src/dogma/compiler.rs` should handle this — file an issue if not |

---

## Non-negotiables

- **Every PR is human-reviewed.** No auto-merge.
- **Every rule ships with tests.** No exceptions.
- **Every rule has a source URL.** Unsourced patterns are rejected at review.
- **Lowercase `risk_level`.** Serde is strict.
- **Pattern regex must compile.** Validator catches this; don't ship broken regex.

---

## Related

- `safety-pattern-developer` — TDD skill for non-CVE pattern additions
- `safety-pattern-auditor` — audits the full ruleset for gaps
- `.github/workflows/cve-rule-sync.yml` — weekly cron equivalent
- `data/cve_rules/README.md` — reviewer checklist + pattern guidelines

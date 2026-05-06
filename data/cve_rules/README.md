# CVE + 0din Rules

Machine-authored (Nimble) + human-reviewed CVE/0day danger patterns, plus
Mozilla 0din probe-derived attack signatures, consumed by caro's safety
validator at build time.

**CVE pipeline spec:** `specs/010-nimble-cve-pipeline/`

**0din integration:** `scripts/convert_0din_probes.py` — converts Mozilla 0din
probe specs (Apache 2.0) to the ODIN-*.yaml format described below.

---

## Rule Namespaces

| Prefix | Source | Example |
|--------|--------|---------|
| `CVE-` | NVD / CISA KEV / GHSA (via Nimble) | `CVE-2024-3094.yaml` |
| `ODIN-` | Mozilla 0din probes (Apache 2.0) | `ODIN-2024-001.yaml` |

Both namespaces use identical YAML schema and are compiled into the same
`cve_patterns.bin` blob. `caro --version` surfaces counts separately:
```
cve rules:   2
0din probes: 7
```

---

## How this directory is used

```
┌─────────── BACK-OFFICE (dev/CI only) ──────────┐  ┌─ BUILD ─┐  ┌─ RUNTIME ─┐
│  Nimble GH Action cron (Mon 09:00 UTC)         │  │         │  │           │
│   └─▶ `scripts/nimble-cve-sync.ts`             │  │ build.rs│  │ validator │
│       └─▶ opens PR adding CVE-*.yaml files     │──▶ glob →  ──▶ merges     │
│                                                │  │ bincode │  │ static +  │
│  0din sync: `scripts/convert_0din_probes.py`   │  │ blob    │  │ CVE +     │
│   └─▶ converts 0din probes → ODIN-*.yaml       │  │         │  │ ODIN      │
│                                                │  │         │  │ patterns  │
│  Maintainer skill `caro.security.update`       │  │         │  │           │
│   └─▶ same script, off-cycle                   │  │         │  │           │
└────────────────────────────────────────────────┘  └─────────┘  └───────────┘
```

1. **Authoring:** Nimble (web-research agent) drafts a YAML file per CVE.
2. **Review:** Every PR is human-reviewed — no auto-merge, even at CVSS ≥ 9.
3. **Build:** `build.rs` compiles these YAMLs into an embedded bincode blob
   consumed at startup by `src/safety/cve_patterns.rs`.
4. **Runtime:** merged with the 52 static `DangerPattern`s from
   `src/safety/patterns.rs` into a single aho-corasick scan. Zero network,
   zero key, zero added latency.

---

## File shape

One YAML per CVE. Filename is the CVE ID:

```
data/cve_rules/CVE-YYYY-NNNNN.yaml
```

```yaml
id: CVE-2024-3094
source: https://nvd.nist.gov/vuln/detail/CVE-2024-3094
disclosed: 2024-03-29          # ISO date
risk_level: critical           # safe | moderate | high | critical  (lowercase)
shell_specific: null           # null | bash | zsh | fish | sh | powershell | cmd
pattern: "xz.*--lzma1=.*preset=9"
description: "Short human-readable summary"
test_cases:                    # REQUIRED — every rule ships with tests
  - input: "xz --lzma1=preset=9 file.txt"
    expected_behavior: Block
  - input: "xz -z file.txt"
    expected_behavior: Allow
```

**Schema is enforced** by `scripts/validate-cve-yaml.ts`, which runs in CI
on every PR touching this directory.

### Field reference

| Field | Required | Notes |
|---|---|---|
| `id` | yes | Canonical CVE ID (`CVE-YYYY-NNNNN`) |
| `source` | yes | Authoritative URL — reviewers spot-check this |
| `disclosed` | yes | ISO date (`YYYY-MM-DD`). Used for sorting and "newer-than-X" tooling |
| `risk_level` | yes | Maps to `crate::models::RiskLevel`. Lowercase only |
| `shell_specific` | yes | Maps to `Option<crate::models::ShellType>`. `null` if shell-agnostic |
| `pattern` | yes | Raw regex (not glob). Tested against candidate commands |
| `description` | yes | One-line summary shown on block |
| `test_cases` | yes | ≥ 1 positive case (`Block`) and ≥ 1 negative case (`Allow`) |

---

## Source attribution

MVP sources (set at Nimble time, pre-filtered by CVSS ≥ 7.0 + shell-tool
allowlist):

| Source | URL | Format |
|---|---|---|
| NVD | https://services.nvd.nist.gov/rest/json/cves/2.0 | JSON REST |
| CISA KEV | https://www.cisa.gov/sites/default/files/csv/known_exploited_vulnerabilities.csv | CSV |
| GHSA | https://api.github.com/advisories | JSON REST |

Deferred to v1.1: Debian DSA, RedHat RHSA, Ubuntu USN.

---

## Reviewer checklist

Before approving a PR that adds/modifies files here:

- [ ] `id` matches filename
- [ ] `source` URL resolves and matches the CVE
- [ ] `pattern` regex is **tight** — doesn't match benign commands (check the
      `Allow` test cases exhaustively)
- [ ] `pattern` is **necessary** — not already covered by `src/safety/patterns.rs`
- [ ] `test_cases` include at least one positive (`Block`) and one negative
      (`Allow`) case
- [ ] `risk_level` honest to CVSS severity (Critical for CVSS ≥ 9.0)
- [ ] `shell_specific` set when the attack vector is shell-specific
      (e.g. bash-only substitution)
- [ ] CI green (`scripts/validate-cve-yaml.ts` passes)

If any box is unchecked, request changes. A bad rule is worse than no rule —
false positives erode user trust in the safety validator.

---

## Pattern-writing guidelines

1. **Anchor to the exploit signature**, not the vulnerable tool. Blocking all
   `xz` usage is overreach; blocking `xz --lzma1=...preset=9` targets the
   CVE-2024-3094 trigger.
2. **Start permissive, tighten via tests.** Add positive tests for known
   exploit payloads and negative tests for benign variants.
3. **Prefer literal substrings** the aho-corasick matcher can batch. Complex
   backreferences fall through to per-pattern regex — slower.
4. **One CVE per file.** If two CVEs share a trigger, keep both YAMLs — the
   compiler dedups patterns automatically.
5. **Escape regex metacharacters** in shell syntax (`$`, `.`, `|`). Test the
   raw regex in a playground before submitting.

---

## Adding a rule manually

For off-cycle 0days (when the weekly cron hasn't run yet), maintainers can
invoke the skill:

```bash
claude --skill caro.security.update
# or point Nimble at a specific CVE ID:
claude --skill caro.security.update --arg CVE-YYYY-NNNNN
```

The skill produces the same YAML shape the cron does. Still human-reviewed
before merge.

---

## Non-goals

- **Runtime Nimble calls.** caro never contacts Nimble at user runtime.
- **Auto-merge.** Every PR here requires human review.
- **User-local rules.** Dogma tier-3 (`~/.caro/rules/`) is the extension point
  for personal patterns — this directory is reserved for CVE-derived rules.

---

## Telemetry

`caro --version` surfaces the embedded count (e.g. `CVE rules: 42`) so users
have a trust signal that their binary includes up-to-date protection.

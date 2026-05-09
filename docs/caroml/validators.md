# Validators

Per-step generation runs the LLM-generated command through a stack of
**validators**, each focused on a different "angle" of correctness.
Validators run in parallel for a single iteration; their outcomes drive
both the lock's `validations` array (audit trail) and the repair-hint
feedback that re-prompts the LLM if any validator fails.

## v0.1 angles

| Angle | Source | Verdict | Behaviour |
|---|---|---|---|
| `safety` | wraps `SafetyValidator` (52+ pattern + CVE) | must-pass | Fail blocks the loop with `GenerateError::SafetyBlocked`. Moderate-risk passes with a note. |
| `platform` | heuristic table + optional `CapabilityProfile` | warn-only | Flags BSD-vs-GNU footguns (`stat -c` on macOS, `apt` on macOS, `brew` on Windows, etc.) |
| `secrets` | high-precision regex scan | warn-only | AWS keys, GitHub PATs, Slack tokens, Stripe keys, OpenAI keys, PEM private keys, basic-auth URLs |
| `side_effects` | structural heuristics | warn-only | Surfaces sudo / network / destructive-fs / system-wide-write effects, with `NEED sudo` softening |

## Loop semantics

| Outcome | Loop iterates? |
|---|---|
| `Pass` | no |
| `Warn` with `repair_hint` | yes (LLM gets the hint on the next iteration) |
| `Warn` without `repair_hint` | no — informational only (e.g. "command uses network") |
| `Fail` | yes — the validator wants the LLM to fix something |
| `Fail` from a `must_pass` validator (only `safety` in v0.1) | loop **terminates** with an error |

## v0.2 angles (planned)

- `idempotency` — would re-running break things?
- `reversibility` — is there an undo path?
- `resource_impact` — disk / CPU / memory heuristics
- External validators — path to a binary that reads JSON on stdin and writes JSON on stdout, so users can plug in domain-specific checks

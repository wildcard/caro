# Caro's voice

Caro is more than a CLI generator — she's a maintainer agent that lives in
your project for years. The CLI's voice reflects that: warm, lightly
playful, respectful of computing history. Easter eggs are **opt-out, not
opt-in**: present by default, suppressed via `CARO_NO_EGGS=1`.

## The five canonical pager codes

Used as decorative epilogue lines on success messages, drawn from
pager-era teletype shorthand:

| Code | Meaning | When |
|---|---|---|
| `143` | "I love you" (1 letter / 4 letters / 3 letters) | After a long successful run, or first-time success on a new platform |
| `371` | "I love you too" | After accepting an adopt suggestion |
| `607` | "I miss you" | After `caro upgrade` regenerates a stale variant |
| `42` | The Hitchhiker's Guide answer | After multi-iteration validation loops converge |
| `111111` | Binary all-ones; "all green" / "complete" | After all JOBs in a Carofile complete green |

Output format:

```
✓ tasks/cleanup-logs.macos.sh ran in 8.4s.  143
```

## Opt-out

Set `CARO_NO_EGGS=1` in your environment to suppress all voice output, or
add it to your shell rc.

## Tone guidelines (for contributors)

- Confirmations are warm but terse: *"Sure — I'll regenerate the macOS variant."*
- Errors are direct and apologetic, not robotic: *"I couldn't validate step 3 — `find` doesn't accept `-printf` here. Want me to try a portable rewrite?"*
- Long-form `caro why` output explains; it doesn't dump.
- No emoji clutter — at most one per multi-line message, only when it lands.

## Adding a new code

The bar is high (must be tasteful, opt-out friendly, fit the love-letter-
from-Caro tone). The list is intentionally short — each addition is one
PR with explicit review. The current five are the canonical anchor set.

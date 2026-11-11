# Audit: share cards and share bar (PR #1588)

**Date:** 2026-10-11 · **Auditor:** `claude-design-frontend-engineer` (read-only)
**Verdict:** ship-with-fixes. No P0. The P1 and the cheap P2s are fixed in the
same PR. `after-contact-sheet.jpg` shows all 65 cards after the fixes.

## Passed

- Card colors are exactly `--caro-beige-100`, `--caro-grey-900`,
  `--caro-grey-600` and `--caro-red-500`. Paper ≈ 88–92%, red ≈ 0.02%.
- Smooth mark on the light paper, as the brand book says.
- Share bar: no horizontal overflow at 375 or 1280 px, light or dark. Focus
  ring is 2px `--accent` (3.6:1 light, 5:1 dark). No scale on hover, no
  left-accent stripe.

## Findings and what changed

| # | Sev | Finding | Fix |
|---|---|---|---|
| 1 | P1 | Compare titles set in 3–4 short lines at 72px; two cards start a line with "&" | Fewest lines among sizes ≥ 56px (`TITLE_MIN_PREFERRED`); `words()` keeps "&" with the word before it |
| 2 | P2 | Mark sat 27px further from the right edge than the text from the left | Trim the PNG's white margin before compositing |
| 3 | P2 | Eyebrow ≈ 8px and URL ≈ 10px in a 506px-wide feed | `LABEL_SIZE` 20→24, `URL_SIZE` 24→28 |
| 4 | P2 | Translated titles under the English eyebrow | Eyebrow shows the language's own name (`NATIVE_NAMES`) |
| 5 | P2 | Mark is the only dark mass (≈ 8% on the home card) | **Open.** Brand composition call for Claude Design; not sent |
| 6 | P2 | Hidden "Share…" item still took a flex gap | `hidden` moved to its `<li>` |
| 7 | P2 | Two rows' buttons did not line up | `.share-label` `min-width: 15ch` |
| 8 | P2 | Button border 1.49:1 on the page | `--caro-grey-500` border (≈ 3.8:1 light, 3.3:1 dark) |
| 9 | P2 | Buttons 32px tall on touch | `min-height: var(--touch-target-min)` at ≤ 600px |
| 10 | P2 | Hover border not `--accent`; hardcoded tracking and timing | `--accent`, `--track-wide`, `--transition-fast` |
| 11 | P2 | Kavana speech bubble can cover the agent row on mobile docs | **Open.** Existing widget, outside this change |

# Caro Icon Definitions

**Status:** definitions ready for generation (owner decision, 2026-10-11).
**Rules:** [`../ICONOGRAPHY.md`](../ICONOGRAPHY.md). **Machine-readable copy:** [`definitions.json`](definitions.json).
**Style reference:** [`reference/kyaro-style-sheet.png`](reference/kyaro-style-sheet.png).

## What this is

The website uses emoji as icons in about 380 places. This file defines
every icon the site needs instead: one definition per meaning, not per
emoji. A definition says what the icon means, where it goes, and what it
must and must not show. It is the input for three steps:

```
definition ──▶ Codex generates candidates ──▶ snap to grid + palette, vectorize ──▶ owner review
                                                                                      │
                                         nothing passes ◀─────────────────────────────┘
                                               │
                                               ▼
                                   the same definition goes to Alrezky Caesaria
```

A slot keeps its current emoji or terminal glyph until its icon passes
review. Nothing here changes the site yet.

## Families

| Family | What it is | Source |
|---|---|---|
| **Kyaro, existing** | One of the 10 icons that already exist | cut from Alrezky's frames (`scripts/kyaro_icons.py`); no generation |
| **Kyaro, new pose** | Kyaro doing something no frame shows | generate from the definition with the sprite frames as reference; Alrezky is the fallback |
| **Object** | A thing, not a character: shield, lock, box, chart | generate from the definition |
| **Terminal glyph** | Slot under 28 px, or inline in text | `▸ ■ $ ❯ >_`; no generation |
| **Third-party mark** | An OS, vendor or tool (macOS, Linux, Rust…) | out of scope here; needs the owner's brand-use call |

## Style spec (every generated icon)

The bar is the Kyaro sprite set. Open the style sheet before you start.

| Property | Rule |
|---|---|
| Grid | **48×48** art pixels for card and feature slots (shown on a 56 px tile). **24×24** for small slots (28 px tile). Draw the 24 px version as its own drawing; never shrink the 48. |
| Scale | Same pixel size as Kyaro: one art pixel = one screen pixel at 1×. A 48 px object sits next to a 48 px Kyaro without "mixels" (mixed pixel sizes). |
| Footprint | An object fills about 30–36 px of the 48 px grid. Kyaro sitting is 26×32 px. Leave at least 4 px clear on the top, left and right. The bottom edge is the ground: a grounded object touches row 47 (see Baseline); a floating symbol leaves 4 px there too. |
| Baseline | A grounded object stands on row 47, like Kyaro's feet. A floating symbol is centered. |
| Outline | 1 px `#000000` on the outer silhouette. Inner lines are `#000000` or `#343434`. |
| Shading | Flat fills. At most one shade per fill (base + darker). Light comes from the top left. No gradients, no dithering. |
| Palette | Only these 12 colors: `#000000` `#343434` `#636363` `#a0a0a0` `#ffffff` `#f4f1df` `#d9a066` `#8f563b` `#d95763` `#ef3333` `#c02020` `#fcfc62`. Red is for danger and emphasis only. |
| Pixels | Hard pixels only: no anti-aliasing, no half-transparent pixels, no blur, no glow. Transparent background. |
| View | Front or a slight 3/4 view, the same as Kyaro. A simple silhouette that reads at 1× (48 px) and still reads on the stage tile `#d1cfc9`. |
| Never | Text or letters (except `>_` where the definition asks), logos of other brands, emoji style, 3D render, isometric, line-icon style, drop shadows. |

## Brief for Codex

Run one icon at a time, from [`definitions.json`](definitions.json).

1. Read the icon's definition and open
   `docs/brand/icons/reference/kyaro-style-sheet.png`. For a Kyaro pose,
   also open the frames the definition names under `assets/kyaro/`.
2. Generate **4 candidates** with the definition's `prompt`. Ask the image
   model for a square image of the pixel art upscaled with nearest neighbor,
   so each art pixel is a clean block (for example 768×768 for a 48 grid,
   16× blocks).
3. Save them as `assets/icons/candidates/<id>/<id>-<n>.png` (`n` = 1–4).
   Keep them out of `assets/kyaro/`: generated art is not Alrezky's.
4. Snap and vectorize each candidate before review, with the definition's
   own `grid` (48 for card and feature slots, 24 for small slots):
   `python3 scripts/pixel_snap.py assets/icons/candidates/<id>/<id>-<n>.png --grid <grid> --out assets/icons/candidates/<id>/snapped`.
   It writes the native PNG, the SVG and an 8× preview, and prints the
   report (palette distance, unsure cells, margins).
5. Add `assets/icons/candidates/<id>/NOTES.md` with the model, the prompt
   you used, anything you changed from the definition, and each
   candidate's snap report.
6. Do not wire anything into the website. Wiring happens after the owner
   picks a snapped candidate.

## Review rubric (owner)

A candidate passes when every answer is yes:

1. Does it mean the definition's `meaning` at a glance, at 48 px?
2. Does it look like it belongs next to Kyaro (outline, palette, pixel size, light)?
3. Does it still read on the stage tile `#d1cfc9`, in light and dark pages?
4. Is it free of text, other brands and emoji style?
5. After snapping to the grid and palette, does it survive (no lost details)?

## Catalog

_The per-icon definitions follow below and in [`definitions.json`](definitions.json)._

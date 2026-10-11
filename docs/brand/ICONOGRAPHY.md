# Caro Iconography

**Status:** adopted 2026-10-11 by owner decision. Replaces the v2.1 glyph pack.
**Generator:** [`scripts/kyaro_icons.py`](../../scripts/kyaro_icons.py)
**Component:** [`website/src/ui/KyaroIcon/KyaroIcon.astro`](../../website/src/ui/KyaroIcon/KyaroIcon.astro)
**Guards:** [`.github/workflows/kyaro-icons.yml`](../../.github/workflows/kyaro-icons.yml)

## Summary

- Kyaro's sprite states are Caro's icons. The pixel artist Alrezky Caesaria
  (Morning Moon Studios) drew them; they set the quality bar.
- A script cuts each icon 1:1 from one sprite frame. Nobody draws, recolors,
  smooths or resizes an icon pixel.
- An icon renders only at a whole-number multiple of its native size.
- Kyaro always stands on a stage-grey tile, in both themes.
- A slot too small for a sprite gets a terminal glyph (`▸ ■ $ ❯ >_`), never
  an emoji or a third-party icon font.

## Why the v2.1 glyph pack was removed

The v2.1 pack had nine 16×16 one-color SVGs: `home`, `location`, `teams`,
`tools`, `rocket`, `shield`, `lightning`, `brain` and `kyaro-mark`. They were
drawn by hand to fill emoji slots. They did not match the Kyaro art in any
way that counts:

| | Kyaro sprites (the bar) | v2.1 glyph pack |
|---|---|---|
| Grid | 64×64 frames, character about 26×32 px | 16×16 |
| Color | 7-color palette: outline, coat, white, tan, tongue | one color (`currentColor`) |
| Form | 1 px outline, two-tone coat, consistent light | blocky silhouettes, no light |
| Character | a black-and-tan Shiba with a readable face | `kyaro-mark` read as a drone or a blob |

The design-system v2 brief (`.claude/audits/2026-05-18-design-system-v2/spec/BRIEF.md`)
already ruled that "Kyaro states ARE the icon system" and that these SVGs
"were deleted as incorrect". The files stayed on the site until 2026-10-11.

## The pipeline

```
assets/kyaro/<state>/<state>_animation/<frame>.png      Alrezky's Aseprite exports
        │                                                (64×64 RGBA, 7 colors)
        │  scripts/kyaro_icons.py
        │    ICONS table: icon name → frame + window
        │    copies the window 1:1, one SVG <path> per palette color
        ▼
website/public/kyaro/icons/<name>.svg                   48×48 figure or 24×24 face
website/src/data/kyaro-icons.json                       name → src, size, label, role
        │
        ▼
<KyaroIcon name="danger" />                             <img width="48" height="48" alt="">, 56 px tile
<KyaroIcon name="face-ready" scale={2} label="Kyaro" /> 48 px face, 56 px tile, labelled

CI  kyaro-icons.yml
    ├─ python3 scripts/kyaro_icons.py --check   rebuild in memory, compare byte for byte
    └─ vitest src/__tests__/kyaro-icons.test.ts real names, whole scales, no glyph pack
```

The script needs only the Python standard library. It reads the PNG frames
with its own small decoder, so CI needs no install step for the check.

## Rules

1. **Cut, never draw.** Every icon pixel comes from a Kyaro frame. To change
   an icon, change its frame or its window in `ICONS`, then run the script.
2. **A figure holds the whole sprite.** The script fails if a 48 px figure
   window cuts a sprite pixel. Only a face icon crops, by design.
3. **Whole-number scales only.** `KyaroIcon` takes `scale` (1, 2 or 3), not a
   free size. A 48 px icon shows at 48, 96 or 144 px. A 24 px face shows at
   24, 48 or 72 px. A slot that cannot fit a 56 px tile gets a face or a glyph.
4. **Never recolor Kyaro.** The script fails on a color outside the 7-color
   palette, and on half-transparent pixels. There is no `currentColor`.
5. **One family per group.** A row of badges or a grid of cards uses Kyaro
   icons or terminal glyphs, not both.
6. **Always on the stage tile.** `KyaroIcon` paints `--caro-grey-200` behind
   the art, with padding of 1/12 of the icon size. A 48 px figure fills a
   56 px tile and a 24 px face a 28 px tile. The color does not change in
   dark mode. The padding is `content-box`, so it never shrinks the art.
7. **Size floor.** A figure needs a 56 px slot. A face needs 28 px. Below
   that, use a terminal glyph.
8. **No emoji and no icon fonts as icons.** Do not import Lucide or similar.

## The icon set (v1)

All ten icons come from the nine Kyaro states. The meaning column follows the
semantic map in the design-system v2 brief.

| Icon | Source frame | Size | Meaning |
|---|---|---|---|
| `ready` | idle #1 | 48 | Default and safe. Brand presence, the loyal companion. |
| `thinking` | blink #2 | 48 | Short wait. Caro reads the context before it answers. |
| `paused` | sleeping #6 | 48 | Long idle. Nothing runs. |
| `asking` | prompt bubble #7 (`>_`) | 48 | Awaiting input. The shell itself. |
| `running` | walking #1 | 48 | In progress. Speed. |
| `success` | happy bounce #4 | 48 | Completed. Commands that work. |
| `cleanup` | pooping #17 | 48 | Freed disk space. Took care of it. |
| `danger` | shocked #5 | 48 | High risk. Pause and confirm. Safety. |
| `error` | upside down #12 | 48 | Unexpected state. |
| `face-ready` | idle #1, head and shoulders | 24 | The brand mark at small sizes, next to the Caro name. |

`paused`, `cleanup` and `error` have no slot yet. They are part of the set so
that loading, cleanup and error states can use them without new work.

## Where the site uses them

| Surface | Slot | Icon or glyph |
|---|---|---|
| `/` Personas | Accelerator · Guardian · Operator | `running` · `danger` · `asking` |
| `/` Features | Safety · Privacy · Commands that work · Speed | `danger` · `ready` · `success` · `running` |
| `/<lang>/` Features | Safety Guardian · Cross-Platform · Platform-Aware · POSIX · Fast · Claude's Companion | `danger` · `success` · `thinking` · `asking` · `running` · `ready` |
| Footer | Brand, next to "Caro" | `face-ready` |
| Footer | "Meet Kyaro!" banner | `ready` |
| `/` Hero | Trust badges, 16 px pills | `▸` privacy · `■` safety · `$` local · `❯` open source |
| Comparison | Stat cards | `▸` privacy · `■` safety · `$` offline · `❯` speed |
| Comparison | Table headings, 24 px | `▸` privacy · `■` safety · `>_` shell · `❯` model |

The same meaning gets the same icon on every page: `danger` is safety,
`running` is speed, `success` is commands that work, `asking` is the shell.

## How to add an icon

1. **An existing state fits.** Add a row to `ICONS` in
   `scripts/kyaro_icons.py` with the frame path. Add a face window only if
   the slot is 28 px. Run `python3 scripts/kyaro_icons.py` and commit the SVG
   and the manifest. Use it with `<KyaroIcon name="…" />`.
2. **No state fits.** Ask for new art. Do not draw it, and do not ask an AI
   model to draw it. Until the art lands, use a terminal glyph. Send the
   request to Alrezky Caesaria with:
   - the state name and the UI meaning (one line each)
   - the slot and its size on the site
   - the frame spec: 64×64, the 7-color Kyaro palette, the 1 px outline,
     feet on row 47, Aseprite plus PNG frames in `assets/kyaro/0NN-<state>/`
   - the number of frames, if it animates
3. The owner approves new states. The Kyaro art is co-owned and is not under
   the AGPL (see [`assets/kyaro/README.md`](../../assets/kyaro/README.md)).

## Known limits

- **Fractional device pixel ratios.** At 125 % or 150 % zoom, some art pixels
  render one device pixel wider than others. All pixel art on the web does
  this. Whole-number ratios (1×, 2×, 3×) are exact.
- **Weight.** Each SVG is 2.4–5.8 KB before compression. Six icons on a page
  add about 30 KB, or about 6 KB gzipped. If the set grows a lot, emit
  indexed PNGs (about 0.5 KB each) from the same script.
- **License.** The generated SVGs are Kyaro art. A fork must remove them with
  `assets/kyaro/`. Each SVG says so in a comment.

## Dark-mode decision

Recorded after the 2026-10-11 visual audit
(`.claude/audits/2026-10-11-kyaro-iconography/`, composite 4).

**Problem.** Kyaro's outline (`#000000`) and coat (`#343434`) nearly vanish
on the dark surfaces. Measured contrast of outline / coat: dark card
`#2b2b2b` 1.48 / 1.14, dark footer `#1a1a1a` 1.21 / 1.40, light-mode footer
`#4f4f4f` 2.56 / 1.52. The white face stays above 11:1, but the body and
the alert lines in `danger` are lost.

**Decision.** Put every Kyaro icon on a `--caro-grey-200` (`#d1cfc9`) tile in
both themes (rule 6). On the tile the outline reaches 13.5:1 and the coat
8:1. The tile already existed: it was the light-mode persona tile. Using it
in both themes also lines up the art in left-aligned cards, where the
sprites sit at different insets inside their 48 px windows.

**Rejected.** A 1 px light halo (four chained `drop-shadow` filters) stays
crisp, but it adds pixels around Alrezky's art and turns the `danger`
alert lines into thick white strokes.

## Open questions for the owner

1. **The official mark and favicon.** `mark-caro-pixel.png` (a pixel face in
   a terminal window, with three dots above) feeds the favicon on every site.
   This change keeps it. Default: keep it. If it is the "drone" you meant,
   say so, and the favicon pipeline can take `face-ready` instead.
2. **Navigation lockup.** The navigation shows `>_ Caro`. The footer now
   shows the Kyaro face next to "Caro". Default: keep `>_` in the navigation.
3. **New states from Alrezky.** A "guarding" pose would fit safety better
   than `danger`, and a "home" pose would fit privacy better than `ready`.
   Default: no request until a slot needs one.

# Caro Icon Definitions

**Status:** definitions ready for generation (owner decision, 2026-10-11).
**Rules:** [`../ICONOGRAPHY.md`](../ICONOGRAPHY.md). **Machine-readable copy:** [`definitions.json`](definitions.json).
**Style reference:** [`reference/kyaro-style-sheet.png`](reference/kyaro-style-sheet.png).

## What this is

The website uses emoji as icons. The inventory of 2026-10-11 found 510
emoji in `website/src`; 289 of them carry a meaning (see Catalog). This
file defines every icon the site needs instead: one definition per
meaning, not per emoji. A definition says what the icon means, where it goes, and what it
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
Take only definitions with `"status": "to-generate"`, in `priority` order
(1 first). Skip `exists` and `glyph`. An `owner-call` definition waits
for the owner's decision; its `variants` each carry their own `prompt`.

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

The inventory of 2026-10-11 found 510 emoji in `website/src`:

```
510 emoji
 ├─ 289 carry a meaning ──▶ 44 definitions below
 │    ├─ 141 slots ──▶ an icon
 │    ├─   9 slots of 28 px ──▶ the small-slot glyph for now (decision 3)
 │    └─ 139 smaller or inline slots ──▶ the small-slot glyph
 ├─ 194 show an OS, vendor or tool ──▶ out of scope (decision 4)
 └─ 27 are not icons (dead code, doc comments, prose)
```

The same emoji means different things on different pages: 🔒 lands in
seven definitions and 🚀 in four. Swap icons slot by slot, never by
find-and-replace on the emoji. `definitions.json` lists every slot with its
file, line, size, surface and label. Lines drift, so match a slot by its
surface and label.

Every generated definition uses the 48 grid: each one has a slot of 32 px
or more. A slot from 32 to 48 px grows to the 56 px tile, as
`LPFeatures` did. A 28 px slot shows the small-slot glyph until a 24 grid
version exists (decision 3). The `icon` slot counts below include those
28 px slots. Every definition names a small-slot glyph; `▸` is the default.

**P** is the priority: 1 = the drawing fills 5 or more slots, 2 = 2 to 4,
3 = one slot. Generate in priority order.

### Decisions for the owner

Each has a default. Reply with the number to change one (for example `2: generate`).

1. **Rows that mix Kyaro and objects.** In five places a Kyaro state
   would sit beside object icons or third-party stand-ins, for example
   the `/glossary` section headers (`shell` beside `files` and
   `network-web`). The list is below.
   Default: **allow.** Both are pixel art on the same grid, palette and
   stage tile. ICONOGRAPHY Rule 5 still forbids pixel icons and terminal
   glyphs in one row.
2. **`regulated-sector` and `install-method`.** Nine one-off icons for two
   rows of `/use-cases/air-gapped`. Default: **drop the icons from those
   rows;** the cards keep their titles. The variant prompts are ready if
   you want them.
3. **24 grid versions.** Nine 28 px slots could take a 24 px drawing:
   the `/roadmap` milestone cards, the safety best-practice lists and the
   open-source benefits on `/open-source-shell-ai`. Default: **not now;**
   they show the glyph.
4. **Third-party marks.** 194 slots show an OS, vendor or tool (Rust 🦀,
   Windows 🪟, macOS 🍎, Linux 🐧…). 136 of them are in
   `config/distros.ts` and `ui/DistroSelector`. No website page renders
   those; only Storybook shows the selector. Default: **out of scope
   here.** The live slots keep their emoji until you choose official
   marks or text, because each vendor's brand rules differ.

Mixed rows (decision 1):

- /glossary and /modern-unix-tools section headers: shell -> Kyaro `asking` beside object icons (folder, globe, key...)
- /open-source-shell-ai problem cards: hallucination, privacy, platform-aware, incident mix object and Kyaro
- /explore community cards: breakage (Kyaro `error`) beside objects (heart, people, books)
- /use-cases persona cards and /product who-it's-for: all objects, while the / personas use Kyaro states
- /try-caro tech cards and /explore backend options: one Caro concept beside third-party stand-ins

### Kyaro, existing (7): wire, do not generate

These icons exist. A follow-up PR can wire them into their slots now.

| Id | Kyaro icon | Meaning | Replaces | Slots (icon / glyph) | Small-slot glyph |
|---|---|---|---|---|---|
| `safety` | `danger` | Caro's safety layer: risky commands are caught and checked before they run; risk warnings. | ⚠ 🛡 🎯 📄 🔒 | 10 / 19 | ■ |
| `speed` | `running` | Fast: sub-2 s inference, instant results, no waiting on cloud APIs. | ⚡ 🚀 | 5 / 3 | ❯ |
| `platform-aware` | `thinking` | Caro reads the context (OS, shell, BSD vs GNU) before it answers. | 💻 🖥 🎯 ⚙ 🤔 🌍 🔧 | 5 / 3 | ▸ |
| `shell` | `asking` | The shell itself: terminal, CLI, prompt, shell expertise. | 🐚 💻 🖥 ⌨ 🚀 | 4 / 4 | >_ |
| `commands-that-work` | `success` | Commands that work first time on any platform: correct, POSIX-portable, cross-platform. | ⚡ 🌐 ✅ 🌍 | 6 / 0 | ✓ |
| `caro-brand` | `ready` | Caro itself, brand presence next to the Caro name. | 🐕 🥕 | 4 / 1 | ▸ |
| `breakage` | `error` | Something broke: a destructive command wiped data, prod is down, or a bug to report. | 🐛 💥 💀 | 3 / 1 | ▸ |

`caro-brand` takes `face-ready` in its 28 px slots.

### Kyaro, new pose (2)

Generate with the frames in `reference` open. Kyaro must look like the
frames: same head, ears, outline and colors. If no candidate passes, these
go to Alrezky Caesaria first.

| Id | P | Subject | Replaces | Slots (icon / glyph) |
|---|---|---|---|---|
| `privacy` | 1 | Kyaro, the Shiba Inu from the reference frames, lies in front of a closed grey laptop with the chin resting on its lid, eyes open, guarding it. | 🔒 🔇 ☁ | 5 / 10 |
| `anxiety` | 2 | Kyaro, the Shiba Inu from the reference frames, sits with ears flat, eyes wide, and one white sweat drop beside the head: nervous before running a command. | ⚠ 😰 | 2 / 0 |

`privacy` shows `ready` today (in `LPFeatures`), but `ready` also means
Caro itself. The pose ends that overlap.

### Objects (33)

| Id | P | Subject | Replaces | Slots (icon / glyph) |
|---|---|---|---|---|
| `hallucination` | 1 | One white six-sided die, tilted mid-roll, with black pips. | 🎯 🎲 🎰 🎭 💀 | 6 / 4 |
| `incident` | 1 | A red rotating siren beacon on a short grey base, with two short light rays. | 🚨 ⚡ 📅 🛡 ⏰ | 6 / 2 |
| `tools-config` | 1 | A wrench crossed over a gear. | 🏗 ⚙ 🧰 🔧 | 6 / 1 |
| `local-offline` | 1 | A small desktop computer with a dark screen; its network cable hangs unplugged, with a clear gap between the plug and the wall socket. | ✈ 🔐 📡 ☁ 🔒 | 5 / 5 |
| `network-web` | 1 | A globe with a latitude and longitude grid, without a stand. | 🌐 | 5 / 0 |
| `team` | 1 | Three small people stand side by side; the middle one is a little taller. | 👥 🤝 🔧 | 5 / 1 |
| `audit` | 2 | A magnifying glass held over a page of log lines. | 👁 📋 📝 🔍 | 4 / 0 |
| `developer` | 2 | An open laptop whose screen shows three short indented lines of code. | 💻 👨‍💻 | 4 / 1 |
| `files` | 2 | One closed folder with a tab on the top left. | 📂 📁 | 4 / 1 |
| `git-fork` | 2 | One line that splits into two branches, with a round node at each of its three ends. | 🌿 🔀 🍴 🚀 | 4 / 0 |
| `learn-docs` | 2 | Two closed books stacked flat, spines toward the viewer. | 📖 🔄 🎓 📚 | 4 / 0 |
| `local-model` | 2 | A square microchip with pins on all four sides and a simple brain shape on its top. | 🧠 🔐 ⚙ 📦 | 4 / 2 |
| `permissions` | 2 | A single old-style key, lying diagonal: bow at the top left, bit at the bottom right. | 🔐 🔒 👤 | 4 / 0 |
| `binary-package` | 2 | One closed cardboard box, taped shut, with one small blank label on the front. | 🐕 📦 | 3 / 4 |
| `focus-target` | 2 | A round target of red and white rings with one arrow in the center. | 🎯 | 3 / 2 |
| `judgment` | 2 | Balance scales: two pans hang level from a beam on a central post. | ⚖ 🔓 📜 🧠 | 3 / 5 |
| `launch` | 2 | A small rocket that points up and to the right, with a short flame. | 🚀 | 3 / 0 |
| `metrics` | 2 | A bar chart of three bars that rise from left to right, on a short base line. | 📊 | 3 / 1 |
| `text-processing` | 2 | A page of text lines with a solid cursor block at the end of the last line. | 📝 🔍 ✂ | 3 / 0 |
| `tip-idea` | 2 | A lit light bulb with a grey screw base and three short yellow rays. | 💡 ✨ | 3 / 5 |
| `integration` | 2 | A two-prong plug about to enter a wall socket, with a small gap between them. | 📡 🔌 | 2 / 1 |
| `policy-rules` | 2 | A clipboard that holds a sheet with three checkbox lines; the first two are checked. | 📋 👥 | 2 / 0 |
| `risk-levels` | 2 | A half-circle gauge in four segments, from white through yellow and red to dark red, with a black needle. | 🔒 📊 | 2 / 0 |
| `support-thanks` | 2 | A pixel heart. | ♥ 🎁 ⭐ | 2 / 2 |
| `ai-agent` | 3 | A small boxy robot head with one antenna and two square eyes. | 🤖 | 1 / 2 |
| `beautiful-output` | 3 | A painter's palette with four paint dabs. | 🎨 | 1 / 0 |
| `blocked` | 3 | A round no-entry sign: a red disc with one white horizontal bar. | 🚫 ⛔ ❌ | 1 / 5 |
| `keyboard` | 3 | One square keycap seen slightly from the front, with a blank top. | ⌨ | 1 / 0 |
| `memory-safe` | 3 | A memory module (a RAM stick) in front of a shield. | 🔒 | 1 / 0 |
| `open-source` | 3 | An open book; both pages show short indented lines, like source code. | 📖 💻 🦀 ⭐ | 1 / 5 |
| `verified` | 3 | A round seal with a scalloped edge and a check mark in the middle. | ✅ | 1 / 1 |
| `regulated-sector` | 3 | **Decision 2.** One drawing per card: SCIFs (government): a building with a triangular roof and four columns; Financial Trading: a line chart that zigzags up to the right, on two axes; Healthcare: a stethoscope; Industrial (OT/SCADA): a factory with a saw-tooth roof and one chimney; Defense Contractors: a shield with a five-point star; Research Labs: a microscope. | 🏛 📈 🏥 🏭 🛡 🔬 | 6 / 0 |
| `install-method` | 3 | **Decision 2.** One drawing per card: USB Transfer: a USB stick with its cap off; Build from Source: a page of indented code lines with a small hammer across it; Internal Mirror: a server rack with three blinking lights. | 💾 🔧 🏢 | 3 / 0 |

Each definition in `definitions.json` also carries `must` and `must_not`
lists and the full `prompt`. Read them before you generate.

### Terminal glyphs (2): no art

| Id | Glyph | Meaning | Slots |
|---|---|---|---|
| `list-marker` | ▸ | A small item marker whose emoji carries no meaning a glyph can keep: search results, FAQ bullets, example-prompt chips, banner prefixes, link lists. | 45 |
| `game-life` | ♥ | One life in the Safe-or-Danger game. | 3 |

`game-life` keeps ♥ as text (U+2665 with U+FE0E), so the lost-life color
change still works.

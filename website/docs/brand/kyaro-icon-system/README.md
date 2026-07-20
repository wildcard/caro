# Kyaro generated icon system

This icon family is grounded in every supplied Kyaro animation frame rather
than a generic dog or a single convenient pose.

## Source evidence

- `kyaro-state-grounding.jpg` shows the first, middle, and final frames of all
  nine states.
- `kyaro-all-frames-contact-sheet.jpg` shows all 99 PNG frames in source order.
- `kyaro-source-palette.json` measures the opaque source colours instead of
  estimating them by eye.
- The source sprites live under `assets/kyaro/`; the contact sheets contain no
  generated artwork.

## Generated assets

The current website assets are high-resolution generated PNGs with genuine
alpha transparency:

- `/brand/kyaro/kyaro-idle.png`
- `/brand/kyaro/kyaro-happy.png`
- `/brand/kyaro/kyaro-alert.png`

Each is 1024×1024. They share one generated master identity; happy and alert
were produced as identity-preserving expression edits. The rejected hand-drawn
SVG is not used. Any future vector translation is explicitly reserved for the
Claude design workflow.

The homepage loads matching 256×256 `*-web.png` delivery copies. This preserves
the 1024×1024 masters while avoiding a multi-megabyte transfer for marks shown
at 24–44 CSS pixels.

## Identity contract

Every Kyaro icon must preserve:

1. A compact black-and-tan Shiba body with short planted legs.
2. Charcoal-black head and saddle.
3. Tan eyebrow point, cheek, legs, and ear interiors.
4. Broad white muzzle and chest.
5. A thick curled tail that remains visible at icon size.
6. A projected canine muzzle and nose—not a flat cat-face mask.
7. The measured source palette: `#343434`, `#000000`, `#FFFFFF`, `#D9A066`,
   and the small collar accent `#D95763`.

## Hatch-derived production process

The installed Hatch Pet skill was studied as process guidance only; its
generation workflow was not run.

1. **Ground identity.** Inventory every reference that defines anatomy,
   markings, palette, silhouette, or expression.
2. **Make contact sheets.** Review the complete state family and a compact
   representative sheet before drawing.
3. **Choose one canonical master.** Establish a single coherent body,
   silhouette, markings, and palette.
4. **Derive variants coherently.** Generate one high-resolution master, then
   use identity-preserving edits to change expression only. Never generate each
   icon independently from text.
5. **Validate deterministic properties.** Confirm real alpha transparency,
   1024×1024 masters, 256×256 web copies, accessible labels, allowed variants,
   and absence of the retired glyph or custom SVG.
6. **Validate visually at delivery sizes.** Review 16, 20, 24, 32, and 48 px
   against both light and dark backgrounds. Reject cat/fox reads, lost muzzle,
   invisible tail curl, or merged markings even when tests pass.
7. **Review the real surface.** Capture the homepage navigation and persona
   cards, not an isolated artboard alone.

## Apple emoji and symbol lessons

Apple's current Genmoji workflow allows an existing emoji or named pet/photo
to serve as the starting point, adds a limited set of concepts or descriptive
details, presents variations, and supports iterative refinement. For Kyaro,
`🐕` supplies the full-body dog silhouette while the complete source sheet
supplies identity-specific markings and proportions.

Apple's SF Symbols guidance also notes that enclosing shapes and solid fill
variants can improve small-size legibility. The homepage therefore places the
full-colour mark on a quiet enclosing tile in persona cards while the
navigation version uses the simpler unframed silhouette.

Official references:

- https://support.apple.com/guide/iphone/create-your-own-emoji-with-genmoji-iph4e76f5667/ios
- https://developer.apple.com/design/human-interface-guidelines/sf-symbols

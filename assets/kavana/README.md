# Kavana — a Codex pet

Kavana is Karo reimagined as a small black-and-tan Shiba Inu companion for
Codex. Her name means “intention” in Hebrew: she brings curiosity, focus, and
a little warmth to every task.

The ready-to-install Codex pet package is in [`codex-pet/`](codex-pet/). It
uses the Codex v2 pet format: an 8-column by 11-row animated spritesheet with
the standard task states and 16 look directions.

## Install locally

Copy the package into your Codex pets directory:

```sh
mkdir -p "$HOME/.codex/pets/kavana"
cp assets/kavana/codex-pet/pet.json assets/kavana/codex-pet/spritesheet.webp "$HOME/.codex/pets/kavana/"
```

Restart Codex if Kavana does not appear immediately, then select **Kavana** in
the pet picker.

## Package

- `codex-pet/pet.json` — Codex pet metadata
- `codex-pet/spritesheet.webp` — validated 1536×2288 v2 animation atlas

## Artwork and use

Kavana was created for this project from Karo/Kyaro character direction. The
artwork remains subject to the project-specific terms in
[`../kyaro/README.md`](../kyaro/README.md). It may be installed for personal,
noncommercial use as a Codex pet; it is not a general-purpose public-domain or
open-source art asset.

Made with intention and shared with love. ❤

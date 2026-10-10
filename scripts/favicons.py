#!/usr/bin/env python3
"""
Favicons - one favicon set for every Caro site, derived from the official mark.

The icon is the head of the official pixel mark (website/public/mark-caro-pixel.png).
The script samples the mark's 15x15 pixel grid and renders it in brand ink on a
brand paper tile. Nothing is drawn by hand, so the favicon cannot drift from the
mark (brand rule: no hand-drawn pixel iconography).

Usage:
    python3 scripts/favicons.py           # regenerate every site's files (needs Pillow)
    python3 scripts/favicons.py --check   # verify files and <head> wiring (stdlib only, used by CI)
"""

import argparse
import hashlib
import io
import math
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MARK = ROOT / "website/public/mark-caro-pixel.png"

# Brand tokens from website/src/ui/tokens.css.
INK = "#2b2b2b"  # --caro-grey-900 (--fg-strong)
PAPER = "#f4f1df"  # --caro-beige-100 (--bg)
# The deprecated orange gradient is forbidden by the brand book.
FORBIDDEN = ("#ff8c42", "#ff6b35")

# The mark's head (signal dots + face) in the source PNG: a 15x15 grid.
MARK_BOX = (60, 60, 639, 638)  # left, top, right, bottom (px)
GRID = 15

# Favicon layout on a 16-unit tile. 16, 32 and 48 px are exact multiples, so
# every cell lands on whole device pixels. The spare unit is split into a
# half-unit paper keyline on every side: without it, the ink edge merges into
# dark browser tab strips. At 16 px the keyline snaps to one side.
TILE = 16
TILE_RADIUS = 2
OFFSET = (0.5, 0.5)

# Output name -> sites that get it. Next.js (apps/devrel) uses its own names
# from the app-router file conventions.
SMALL_SET = {
    "favicon.svg": "svg",
    "favicon.ico": "ico",
    "apple-touch-icon.png": "apple",
}
TARGETS = {
    "website/public": {
        **SMALL_SET,
        "icon-192.png": "icon-192",
        "icon-512.png": "icon-512",
        "icon-maskable-512.png": "maskable-512",
    },
    "website/.storybook/static": {"favicon.svg": "svg"},
    "docs-site/public": SMALL_SET,
    "changelog/public": SMALL_SET,
    "landing/public": SMALL_SET,
    "presentation/public": SMALL_SET,
    "apps/devrel/app": {
        "icon.svg": "svg",
        "favicon.ico": "ico",
        "apple-icon.png": "apple",
    },
}

# Each site must reference the favicon from its <head>. File -> required text.
WIRING = {
    "website/src/components/Favicons.astro": [
        'href="/favicon.svg"',
        'href="/favicon.ico"',
    ],
    "website/src/components/SEO.astro": ["<Favicons />"],
    "website/src/layouts/LandingPage.astro": ["<Favicons />"],
    "website/src/layouts/DocsLayout.astro": ["<Favicons />"],
    "website/public/site.webmanifest": [
        '"/icon-192.png"',
        '"/icon-512.png"',
        '"/icon-maskable-512.png"',
    ],
    "website/.storybook/main.ts": ["staticDirs: ['./static']"],
    "docs-site/astro.config.mjs": ["favicon: '/favicon.svg'", "href: '/favicon.ico'"],
    "changelog/src/components/BaseHead.astro": [
        'href="/favicon.svg"',
        'href="/favicon.ico"',
    ],
    "landing/src/layouts/Layout.astro": ['href="/favicon.svg"', 'href="/favicon.ico"'],
    "presentation/slides.md": ["favicon: /favicon.svg"],
    "presentation/roadmap-slides.md": ["favicon: /favicon.svg"],
    "presentation/pitch-slides.md": ["favicon: /favicon.svg"],
}

PNG_SIZES = {"apple": 180, "icon-192": 192, "icon-512": 512, "maskable-512": 512}
ICO_SIZES = (16, 32, 48)


def read_grid():
    """Sample the official mark into a GRID x GRID matrix of booleans (True = ink)."""
    from PIL import Image

    img = Image.open(MARK).convert("L")
    left, top, right, bottom = MARK_BOX
    cw, ch = (right - left) / GRID, (bottom - top) / GRID
    grid = []
    for r in range(GRID):
        row = []
        for c in range(GRID):
            # Average the centre half of each cell, away from anti-aliased edges.
            box = (
                int(left + (c + 0.25) * cw),
                int(top + (r + 0.25) * ch),
                int(left + (c + 0.75) * cw),
                int(top + (r + 0.75) * ch),
            )
            pixels = img.crop(box).tobytes()
            dark = sum(1 for p in pixels if p < 128) / len(pixels)
            if 0.05 < dark < 0.95:
                sys.exit(
                    f"error: cell ({r},{c}) of {MARK.name} is ambiguous; check MARK_BOX"
                )
            row.append(dark >= 0.5)
        grid.append(row)
    return grid


def runs(row):
    """Yield (start, length) for each horizontal run of ink cells."""
    c = 0
    while c < len(row):
        if row[c]:
            start = c
            while c < len(row) and row[c]:
                c += 1
            yield start, c - start
        else:
            c += 1


def mark_digest():
    """Stamped into the SVG, so --check notices a changed mark without Pillow."""
    return "sha256:" + hashlib.sha256(MARK.read_bytes()).hexdigest()


def render_svg(grid):
    ox, oy = OFFSET
    path = "".join(
        f"M{ox + x} {oy + y}h{w}v1h-{w}z"
        for y, row in enumerate(grid)
        for x, w in runs(row)
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {TILE} {TILE}">\n'
        "  <title>Caro</title>\n"
        "  <!-- Generated by scripts/favicons.py from website/public/mark-caro-pixel.png"
        f" ({mark_digest()}). Do not edit by hand. -->\n"
        f'  <rect width="{TILE}" height="{TILE}" rx="{TILE_RADIUS}" fill="{PAPER}"/>\n'
        f'  <path fill="{INK}" shape-rendering="crispEdges" d="{path}"/>\n'
        "</svg>\n"
    )


def render_png(grid, size, cell, origin, radius):
    """Paper tile (anti-aliased corners) with the mark drawn on whole pixels."""
    from PIL import Image, ImageDraw

    ss = 4  # supersample the tile so rounded corners are smooth
    mask = Image.new("L", (size * ss, size * ss), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (0, 0, size * ss - 1, size * ss - 1), radius=radius * ss, fill=255
    )
    img = Image.new("RGBA", (size, size), PAPER)
    img.putalpha(mask.resize((size, size), Image.Resampling.BOX))
    draw = ImageDraw.Draw(img)
    ox, oy = origin
    for y, row in enumerate(grid):
        for x, w in runs(row):
            draw.rectangle(
                (
                    ox + x * cell,
                    oy + y * cell,
                    ox + (x + w) * cell - 1,
                    oy + (y + 1) * cell - 1,
                ),
                fill=INK,
            )
    return img


def render_tile(grid, scale):
    """The favicon layout at an exact multiple of the 16-unit tile."""
    # Round half-unit offsets up, as Chromium does when it snaps the SVG.
    origin = (math.ceil(OFFSET[0] * scale), math.ceil(OFFSET[1] * scale))
    return render_png(grid, TILE * scale, scale, origin, TILE_RADIUS * scale)


def render_padded(grid, size, fill, radius):
    """App icons: the mark centred at `fill` of the canvas, on whole-pixel cells."""
    cell = int(size * fill) // GRID
    pad = (size - cell * GRID) // 2
    return render_png(grid, size, cell, (pad, pad), radius)


def encode(img, fmt, **kwargs):
    buf = io.BytesIO()
    img.save(buf, fmt, **kwargs)
    return buf.getvalue()


def build(grid):
    tiles = [render_tile(grid, s // TILE) for s in ICO_SIZES]
    return {
        "svg": render_svg(grid).encode(),
        "ico": encode(
            tiles[-1],
            "ICO",
            sizes=[(s, s) for s in ICO_SIZES],
            append_images=tiles[:-1],
        ),
        # iOS masks the corners itself and wants an opaque square.
        "apple": encode(
            render_padded(grid, 180, 0.72, 0).convert("RGB"), "PNG", optimize=True
        ),
        "icon-192": encode(render_padded(grid, 192, 0.72, 24), "PNG", optimize=True),
        "icon-512": encode(render_padded(grid, 512, 0.72, 64), "PNG", optimize=True),
        # Maskable icons must keep the mark inside the 80% safe-zone circle.
        "maskable-512": encode(
            render_padded(grid, 512, 0.54, 0).convert("RGB"), "PNG", optimize=True
        ),
    }


def generate():
    assets = build(read_grid())
    for site, files in TARGETS.items():
        for name, kind in files.items():
            path = ROOT / site / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(assets[kind])
            print(f"wrote {path.relative_to(ROOT)}")


def png_size(data):
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return struct.unpack(">II", data[16:24])


def ico_sizes(data):
    reserved, kind, count = struct.unpack("<HHH", data[:6])
    if reserved != 0 or kind != 1:
        return None
    return sorted((data[6 + 16 * i] or 256) for i in range(count))


def check():
    errors = []
    canonical = {}  # kind -> (path, bytes); the first copy found is the reference
    for site, files in TARGETS.items():
        for name, kind in files.items():
            rel = f"{site}/{name}"
            path = ROOT / rel
            if not path.is_file():
                errors.append(f"{rel}: missing (run scripts/favicons.py)")
                continue
            data = path.read_bytes()
            if kind not in canonical:
                canonical[kind] = (rel, data)
            elif data != canonical[kind][1]:
                errors.append(
                    f"{rel}: differs from {canonical[kind][0]} (run scripts/favicons.py)"
                )

    if "svg" in canonical:
        rel, data = canonical["svg"]
        svg = data.decode().lower()
        # Browsers do not load external resources inside an SVG favicon.
        # Internal references (href="#id", url(#id)) are fine.
        for ref in re.findall(
            r"""(?:href\s*=\s*["']|url\(\s*["']?)(?!#)([^"')]*)""", svg
        ):
            errors.append(
                f"{rel}: references '{ref}'; a favicon SVG must be self-contained"
            )
        for color in FORBIDDEN:
            if color in svg:
                errors.append(f"{rel}: uses the forbidden orange {color}")
        for color in (INK, PAPER):
            if color not in svg:
                errors.append(f"{rel}: brand color {color} not found")
        if mark_digest() not in svg:
            errors.append(
                f"{rel}: not generated from the current {MARK.name}"
                " (run scripts/favicons.py)"
            )

    if "ico" in canonical:
        rel, data = canonical["ico"]
        if ico_sizes(data) != list(ICO_SIZES):
            errors.append(f"{rel}: expected sizes {ICO_SIZES}, found {ico_sizes(data)}")

    for kind, size in PNG_SIZES.items():
        if kind in canonical:
            rel, data = canonical[kind]
            if png_size(data) != (size, size):
                errors.append(
                    f"{rel}: expected a {size}x{size} PNG, found {png_size(data)}"
                )

    for rel, needles in WIRING.items():
        path = ROOT / rel
        text = path.read_text() if path.is_file() else ""
        for needle in needles:
            if needle not in text:
                errors.append(f"{rel}: missing {needle!r}")

    for error in errors:
        print(f"error: {error}")
    if errors:
        return 1
    count = sum(len(f) for f in TARGETS.values())
    print(
        f"ok: {count} favicon files across {len(TARGETS)} sites, {len(WIRING)} head references"
    )
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--check", action="store_true", help="verify only; do not write files"
    )
    args = parser.parse_args()
    if args.check:
        sys.exit(check())
    generate()


if __name__ == "__main__":
    main()

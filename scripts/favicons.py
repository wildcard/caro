#!/usr/bin/env python3
"""
Favicons - one favicon set for every Caro site, cut from the Kyaro pixel art.

The icon is Kyaro's face: the same head-and-shoulders window that the Kyaro
icon set calls `face-ready` (scripts/kyaro_icons.py), cut 1:1 from Alrezky
Caesaria's idle frame in assets/kyaro/. The script copies those pixels onto a
stage-grey tile. Nothing is drawn by hand, so the favicon cannot drift from
the art (brand rule: no hand-drawn pixel iconography). Each run records the
SHA-256 of the frame and of every file it writes in scripts/favicons.sha256
(sha256sum format), so --check catches a changed frame or a hand-edited icon.

Usage:
    python3 scripts/favicons.py           # regenerate every site's files (needs Pillow)
    python3 scripts/favicons.py --check   # verify files and <head> wiring (stdlib only, used by CI)
"""

import argparse
import hashlib
import io
import re
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kyaro_icons import FACE, FRAMES, ICONS, read_png  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
FRAME, FACE_ORIGIN = ICONS["face-ready"]
SOURCE = FRAMES / FRAME
LOCK = ROOT / "scripts/favicons.sha256"

# Brand tokens from website/src/ui/tokens.css.
STAGE = "#d1cfc9"  # --caro-grey-200: the tile every Kyaro icon stands on
OUTLINE = "#000000"  # Kyaro's outline; proves the art made it in
# The deprecated orange gradient is forbidden by the brand book.
FORBIDDEN = ("#ff8c42", "#ff6b35")

# Favicon layout: a 32-unit tile. At 16 CSS px on a 2x screen (the common tab
# case) every art pixel lands on one device pixel; at 1x the browser averages
# the SVG down, so the SVG has no crispEdges.
TILE = 32
TILE_RADIUS = 4

# Output name -> sites that get it.
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


def read_face():
    """The face window as rows of '#rrggbb' or None, cropped to the art."""
    rows = read_png(SOURCE)
    x0, y0 = FACE_ORIGIN
    face = [
        [
            None if px[3] == 0 else "#{:02x}{:02x}{:02x}".format(*px[:3])
            for px in rows[y][x0 : x0 + FACE]
        ]
        for y in range(y0, y0 + FACE)
    ]
    # Drop empty rows and columns so the face can be centered on the tile.
    keep_rows = [r for r in face if any(r)]
    cols = [x for x in range(FACE) if any(r[x] for r in keep_rows)]
    return [r[cols[0] : cols[-1] + 1] for r in keep_rows]


def runs(row):
    """Yield (start, length, color) for each horizontal run of one color."""
    x = 0
    while x < len(row):
        color, start = row[x], x
        while x < len(row) and row[x] == color:
            x += 1
        if color:
            yield start, x - start, color


def origin(face, size, scale):
    """Top-left of the face so it sits centered on a size x size canvas."""
    return ((size - len(face[0]) * scale) // 2, (size - len(face) * scale) // 2)


def render_svg(face):
    ox, oy = origin(face, TILE, 1)
    paths = {}
    for y, row in enumerate(face):
        for x, w, color in runs(row):
            paths.setdefault(color, []).append(f"M{ox + x} {oy + y}h{w}v1h-{w}z")
    body = "".join(
        f'  <path fill="{color}" d="{"".join(d)}"/>\n' for color, d in sorted(paths.items())
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {TILE} {TILE}">\n'
        "  <title>Caro</title>\n"
        f"  <!-- Generated by scripts/favicons.py from assets/kyaro/{FRAME}."
        " Do not edit by hand. Kyaro art (c) Kobi Kadosh and Alrezky Caesaria,"
        " project use only: assets/kyaro/README.md -->\n"
        f'  <rect width="{TILE}" height="{TILE}" rx="{TILE_RADIUS}" fill="{STAGE}"/>\n'
        f"{body}</svg>\n"
    )


def render_png(face, size, scale, radius, opaque=False):
    """Stage tile (anti-aliased corners) with the face drawn on whole pixels."""
    from PIL import Image, ImageDraw

    img = Image.new("RGBA", (size, size), STAGE)
    if not opaque:
        ss = 4  # supersample the tile so rounded corners are smooth
        mask = Image.new("L", (size * ss, size * ss), 0)
        ImageDraw.Draw(mask).rounded_rectangle(
            (0, 0, size * ss - 1, size * ss - 1), radius=radius * ss, fill=255
        )
        img.putalpha(mask.resize((size, size), Image.Resampling.BOX))
    draw = ImageDraw.Draw(img)
    ox, oy = origin(face, size, scale)
    for y, row in enumerate(face):
        for x, w, color in runs(row):
            draw.rectangle(
                (ox + x * scale, oy + y * scale, ox + (x + w) * scale - 1, oy + (y + 1) * scale - 1),
                fill=color,
            )
    return img.convert("RGB") if opaque else img


def encode(img, fmt, **kwargs):
    buf = io.BytesIO()
    img.save(buf, fmt, **kwargs)
    return buf.getvalue()


def build(face):
    from PIL import Image

    tile32 = render_png(face, 32, 1, TILE_RADIUS)
    ico = [
        # 16 px cannot hold the 23x20 face 1:1; average the 32 px tile down.
        tile32.resize((16, 16), Image.Resampling.BOX),
        tile32,
        render_png(face, 48, 2, 6),
    ]
    return {
        "svg": render_svg(face).encode(),
        "ico": encode(ico[-1], "ICO", sizes=[(s, s) for s in ICO_SIZES], append_images=ico[:-1]),
        # iOS masks the corners itself and wants an opaque square.
        "apple": encode(render_png(face, 180, 6, 0, opaque=True), "PNG", optimize=True),
        "icon-192": encode(render_png(face, 192, 6, 24), "PNG", optimize=True),
        "icon-512": encode(render_png(face, 512, 16, 64), "PNG", optimize=True),
        # Maskable icons must keep the art inside the 80% safe-zone circle.
        "maskable-512": encode(render_png(face, 512, 12, 0, opaque=True), "PNG", optimize=True),
    }


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def generate():
    assets = build(read_face())
    lock = [f"{sha256(SOURCE.read_bytes())}  {SOURCE.relative_to(ROOT)}"]
    for site, files in TARGETS.items():
        for name, kind in files.items():
            path = ROOT / site / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(assets[kind])
            lock.append(f"{sha256(assets[kind])}  {site}/{name}")
            print(f"wrote {path.relative_to(ROOT)}")
    LOCK.write_text("\n".join(lock) + "\n")
    print(f"wrote {LOCK.relative_to(ROOT)}")


def png_size(data):
    if data[:8] != b"\x89PNG\r\n\x1a\n" or len(data) < 24:
        return None
    return struct.unpack(">II", data[16:24])


def ico_sizes(data):
    try:
        reserved, kind, count = struct.unpack("<HHH", data[:6])
        if reserved != 0 or kind != 1:
            return None
        return sorted((data[6 + 16 * i] or 256) for i in range(count))
    except (struct.error, IndexError):  # truncated file
        return None


def check():
    errors = []
    # The mark and every output must match what the generator last wrote.
    lock_lines = LOCK.read_text().splitlines() if LOCK.is_file() else []
    expected = {
        rel: digest for digest, rel in (ln.split(maxsplit=1) for ln in lock_lines)
    }
    files = {str(SOURCE.relative_to(ROOT)): None}
    for site, names in TARGETS.items():
        files.update({f"{site}/{name}": kind for name, kind in names.items()})
    canonical = {}  # kind -> (path, bytes) of the first copy, for the checks below
    for rel, kind in files.items():
        path = ROOT / rel
        if not path.is_file():
            errors.append(f"{rel}: missing (run scripts/favicons.py)")
            continue
        data = path.read_bytes()
        if sha256(data) != expected.get(rel):
            errors.append(
                f"{rel}: does not match {LOCK.name} (run scripts/favicons.py)"
            )
        if kind:
            canonical.setdefault(kind, (rel, data))

    if "svg" in canonical:
        rel, data = canonical["svg"]
        svg = data.decode("utf-8", "replace").lower()
        # Browsers do not load external resources inside an SVG favicon.
        # Internal references (href="#id", url("#id")) are fine.
        for ref in re.findall(r"""(?:href\s*=\s*|url\(\s*)["']?([^"')\s]*)""", svg):
            if ref and not ref.startswith("#"):
                errors.append(
                    f"{rel}: references '{ref}'; a favicon SVG must be self-contained"
                )
        for color in FORBIDDEN:
            if color in svg:
                errors.append(f"{rel}: uses the forbidden orange {color}")
        for color in (STAGE, OUTLINE):
            if color not in svg:
                errors.append(f"{rel}: brand color {color} not found")

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

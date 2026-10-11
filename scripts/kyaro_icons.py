#!/usr/bin/env python3
"""
Kyaro icons - the Caro icon set, cut from the Kyaro sprite frames.

Every icon is a square window on one frame of the Kyaro animations in
assets/kyaro/ (pixel art by Alrezky Caesaria). The script copies the pixels
in that window 1:1 into an SVG. It never draws, recolors, scales or smooths a
pixel (brand rule: no hand-drawn pixel iconography). A figure window holds the
whole sprite; only a face window crops it. See docs/brand/ICONOGRAPHY.md.

Usage:
    python3 scripts/kyaro_icons.py           # regenerate the icons and the manifest
    python3 scripts/kyaro_icons.py --check   # verify only (used by CI)

--check rebuilds every file in memory and compares it byte for byte with the
committed file, so a hand-edited icon, a changed frame or a stale manifest
fails. Standard library only: no install step.
"""

import argparse
import json
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FRAMES = ROOT / "assets/kyaro"
OUT_DIR = ROOT / "website/public/kyaro/icons"
URL_DIR = "/kyaro/icons"
MANIFEST = ROOT / "website/src/data/kyaro-icons.json"

# The Kyaro palette. A color outside it means the art was recolored.
PALETTE = {
    "#000000",  # outline
    "#343434",  # black coat
    "#ffffff",  # white markings
    "#d9a066",  # tan markings
    "#d95763",  # tongue
    "#8f563b",  # poop
    "#7e6851",  # poop shade
}

FIGURE = 48  # whole sprite, feet on the bottom edge
FACE = 24  # head and shoulders

# name -> (frame under assets/kyaro, window). Meanings: docs/brand/ICONOGRAPHY.md.
# A figure window (None) is FIGURE px square, centered on the sprite with its
# top at row 0; every frame has its feet on row 47. A face window is an
# explicit (x, y) origin for a FACE px square.
ICONS = {
    # default, safe, the loyal companion
    "ready": ("001-idle/idlea_animation/animation_Idle1.png", None),
    # short wait, reading the context
    "thinking": ("002-blink/blink_animation/animation-Blink2.png", None),
    # long idle
    "paused": ("003-sleeping/sleeping_animation/animation-Sleeping6.png", None),
    # awaiting input, the shell
    "asking": ("004-prompt bubble/prompt bubble_animation/animation-PromptBubble7.png", None),
    # in progress, speed
    "running": ("005-walking/walking_animation/animation-Walking1.png", None),
    # completed, commands that work
    "success": ("006-happy bounce/happy bounce_animation/animation-HappyBounce4.png", None),
    # freed disk space
    "cleanup": ("007-pooping/pooping-animation/animation-Pooping17.png", None),
    # high risk, pause and confirm, safety
    "danger": ("008-shocked/shocked_animation/animation-Shocked5.png", None),
    # unexpected state
    "error": ("009-upside down/upside down_animation/animation-UpsideDown12.png", None),
    # the brand mark at small sizes, next to the Caro name
    "face-ready": ("001-idle/idlea_animation/animation_Idle1.png", (19, 12)),
}


def read_png(path):
    """Decode an 8-bit RGBA, non-interlaced PNG into rows of (r, g, b, a)."""
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        sys.exit(f"error: {path.relative_to(ROOT)} is not a PNG")
    pos, idat, header = 8, b"", None
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos : pos + 8])
        body = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat += body
        elif kind == b"IEND":
            break
    width, height, depth, color, _, _, interlace = header
    if (depth, color, interlace) != (8, 6, 0):
        sys.exit(
            f"error: {path.relative_to(ROOT)}: expected 8-bit RGBA, not interlaced"
        )
    raw, stride, rows, i = zlib.decompress(idat), width * 4, [], 0
    prev = bytearray(stride)
    for _ in range(height):
        kind, line = raw[i], bytearray(raw[i + 1 : i + 1 + stride])
        i += 1 + stride
        for x in range(stride):
            a = line[x - 4] if x >= 4 else 0
            b = prev[x]
            c = prev[x - 4] if x >= 4 else 0
            if kind == 1:
                line[x] = (line[x] + a) & 255
            elif kind == 2:
                line[x] = (line[x] + b) & 255
            elif kind == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else b if pb <= pc else c
                line[x] = (line[x] + pred) & 255
        rows.append([tuple(line[x : x + 4]) for x in range(0, stride, 4)])
        prev = line
    return rows


def window(name, rows, face_origin):
    """The (x, y, size) square to copy, checked against the sprite."""
    opaque = [
        (x, y) for y, row in enumerate(rows) for x, px in enumerate(row) if px[3]
    ]
    if not opaque:
        sys.exit(f"error: {name}: frame has no opaque pixels")
    left = min(x for x, _ in opaque)
    right = max(x for x, _ in opaque) + 1
    if face_origin:
        x, y = face_origin
        if not (0 <= x <= len(rows[0]) - FACE and 0 <= y <= len(rows) - FACE):
            sys.exit(f"error: {name}: face window leaves the frame")
        return x, y, FACE
    # Center on the sprite, keep inside the frame, top at row 0.
    x = min(max((left + right) // 2 - FIGURE // 2, 0), len(rows[0]) - FIGURE)
    clipped = [p for p in opaque if not (x <= p[0] < x + FIGURE and p[1] < FIGURE)]
    if clipped:
        sys.exit(f"error: {name}: a {FIGURE}px window cuts {len(clipped)} sprite pixels")
    return x, 0, FIGURE


def render_svg(name, frame, rows, box):
    x0, y0, size = box
    paths = {}  # color -> path data, one run of same-color pixels per segment
    for y in range(size):
        row = rows[y0 + y]
        x = 0
        while x < size:
            r, g, b, a = row[x0 + x]
            if a == 0:
                x += 1
                continue
            if a != 255:
                sys.exit(f"error: {name}: half-transparent pixel; Kyaro art has none")
            color = f"#{r:02x}{g:02x}{b:02x}"
            if color not in PALETTE:
                sys.exit(f"error: {name}: {color} is not in the Kyaro palette")
            start = x
            while x < size and row[x0 + x] == (r, g, b, a):
                x += 1
            paths.setdefault(color, []).append(f"M{start} {y}h{x - start}v1h-{x - start}z")
    body = "".join(
        f'  <path fill="{color}" d="{"".join(d)}"/>\n' for color, d in sorted(paths.items())
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}"'
        f' width="{size}" height="{size}" shape-rendering="crispEdges">\n'
        f"  <!-- Generated by scripts/kyaro_icons.py from assets/kyaro/{frame},"
        f" window x={x0} y={y0}. Do not edit by hand. Kyaro art (c) Kobi Kadosh"
        " and Alrezky Caesaria, project use only: assets/kyaro/README.md -->\n"
        f"{body}</svg>\n"
    ).encode()


def build():
    """Every output file: path -> bytes."""
    files, icons = {}, {}
    for name, (frame, face_origin) in ICONS.items():
        rows = read_png(FRAMES / frame)
        box = window(name, rows, face_origin)
        files[OUT_DIR / f"{name}.svg"] = render_svg(name, frame, rows, box)
        icons[name] = {"src": f"{URL_DIR}/{name}.svg", "size": box[2], "frame": frame}
    manifest = {
        "_generated": "scripts/kyaro_icons.py - do not edit by hand",
        "icons": icons,
    }
    files[MANIFEST] = (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode()
    return files


def generate():
    files = build()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for stale in set(OUT_DIR.glob("*.svg")) - set(files):
        stale.unlink()
        print(f"removed {stale.relative_to(ROOT)}")
    for path, data in files.items():
        path.write_bytes(data)
        print(f"wrote {path.relative_to(ROOT)}")


def check():
    files = build()
    errors = []
    for path, data in files.items():
        rel = path.relative_to(ROOT)
        if not path.is_file():
            errors.append(f"{rel}: missing (run scripts/kyaro_icons.py)")
        elif path.read_bytes() != data:
            errors.append(f"{rel}: differs from the generated file (run scripts/kyaro_icons.py)")
    for stale in sorted(set(OUT_DIR.glob("*")) - set(files)):
        errors.append(f"{stale.relative_to(ROOT)}: not generated; remove it or add it to ICONS")
    for error in errors:
        print(f"error: {error}")
    if errors:
        return 1
    print(f"ok: {len(ICONS)} Kyaro icons match their source frames")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--check", action="store_true", help="verify only; do not write files")
    args = parser.parse_args()
    if args.check:
        sys.exit(check())
    generate()


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Pixel snap - turn a generated pixel-art image into a true pixel grid and an SVG.

Image models draw "pixel art" as a large raster whose blocks are not quite
square, not quite aligned and not quite one color. This script samples the
center of each grid cell, takes the most common color there, maps it to the
nearest color in the Caro icon palette, and writes:

    <out>/<name>.png       the art at native size (1 art pixel = 1 px)
    <out>/<name>.svg       one <path> per palette color, crispEdges
    <out>/<name>.8x.png    an 8x preview on the stage tile, for review

It prints a report: palette distance (how far the input was from the
palette), cells that were unsure (no clear majority), margins and the
baseline. See docs/brand/icons/DEFINITIONS.md.

Usage:
    python3 scripts/pixel_snap.py candidate.png --grid 48 --out assets/icons/snapped
    python3 scripts/pixel_snap.py candidate.png --grid 24 --name safety-shield-24

Needs Pillow. It is an authoring tool, not part of CI.
"""

import argparse
import collections
import sys
from pathlib import Path

# The 12-color icon palette (docs/brand/icons/DEFINITIONS.md, style spec).
PALETTE = [
    "#000000", "#343434", "#636363", "#a0a0a0", "#ffffff", "#f4f1df",
    "#d9a066", "#8f563b", "#d95763", "#ef3333", "#c02020", "#fcfc62",
]
STAGE = "#d1cfc9"


def rgb(hex_color):
    return tuple(int(hex_color[i : i + 2], 16) for i in (1, 3, 5))


def nearest(color):
    """Nearest palette color by weighted RGB distance, and that distance."""
    r, g, b = color

    def dist(p):
        pr, pg, pb = rgb(p)
        return ((r - pr) * 0.30) ** 2 + ((g - pg) * 0.59) ** 2 + ((b - pb) * 0.11) ** 2

    best = min(PALETTE, key=dist)
    return best, dist(best) ** 0.5


def snap(img, grid):
    """Return (cells, report) where cells[y][x] is a palette hex or None."""
    img = img.convert("RGBA")
    w, h = img.size
    cw, ch = w / grid, h / grid
    px = img.load()
    cells, unsure, worst = [], [], 0.0
    for gy in range(grid):
        row = []
        for gx in range(grid):
            # Sample the central half of the cell, away from blurred edges.
            x0, x1 = int((gx + 0.25) * cw), max(int((gx + 0.75) * cw), int((gx + 0.25) * cw) + 1)
            y0, y1 = int((gy + 0.25) * ch), max(int((gy + 0.75) * ch), int((gy + 0.25) * ch) + 1)
            votes = collections.Counter()
            for y in range(y0, min(y1, h)):
                for x in range(x0, min(x1, w)):
                    r, g, b, a = px[x, y]
                    votes[None if a < 128 else nearest((r, g, b))[0]] += 1
            (color, n), total = votes.most_common(1)[0], sum(votes.values())
            if n / total < 0.6:
                unsure.append((gx, gy))
            if color:
                # Distance of the cell's mean color from its palette color.
                opaque = [px[x, y][:3] for y in range(y0, min(y1, h)) for x in range(x0, min(x1, w)) if px[x, y][3] >= 128]
                mean = tuple(sum(c[i] for c in opaque) / len(opaque) for i in range(3))
                worst = max(worst, nearest(mean)[1])
            row.append(color)
        cells.append(row)
    return cells, {"unsure": unsure, "palette_distance": worst}


def to_svg(cells, name):
    size = len(cells)
    paths = {}
    for y, row in enumerate(cells):
        x = 0
        while x < size:
            color, start = row[x], x
            while x < size and row[x] == color:
                x += 1
            if color:
                paths.setdefault(color, []).append(f"M{start} {y}h{x - start}v1h-{x - start}z")
    body = "".join(f'  <path fill="{c}" d="{"".join(d)}"/>\n' for c, d in sorted(paths.items()))
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}"'
        f' width="{size}" height="{size}" shape-rendering="crispEdges">\n'
        f"  <!-- {name}: snapped by scripts/pixel_snap.py from a generated candidate."
        " Not Kyaro art by Alrezky Caesaria. -->\n"
        f"{body}</svg>\n"
    )


def to_png(cells, scale=1, background=None):
    from PIL import Image, ImageDraw

    size = len(cells)
    img = Image.new("RGBA", (size * scale, size * scale), background or (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for y, row in enumerate(cells):
        for x, color in enumerate(row):
            if color:
                d.rectangle((x * scale, y * scale, x * scale + scale - 1, y * scale + scale - 1), fill=color)
    return img


def layout(cells):
    """Margins on each side and whether the art touches the bottom row."""
    size = len(cells)
    ys = [y for y, row in enumerate(cells) if any(row)]
    xs = [x for x in range(size) if any(row[x] for row in cells)]
    if not ys:
        return None
    return {
        "left": xs[0], "right": size - 1 - xs[-1], "top": ys[0], "bottom": size - 1 - ys[-1],
        "art": f"{xs[-1] - xs[0] + 1}x{ys[-1] - ys[0] + 1}",
        "colors": sorted({c for row in cells for c in row if c}),
    }


def main():
    from PIL import Image

    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("image", type=Path)
    parser.add_argument("--grid", type=int, default=48, help="art pixels per side (48 or 24)")
    parser.add_argument("--out", type=Path, default=Path("assets/icons/snapped"))
    parser.add_argument("--name", help="output name (default: the input file name)")
    args = parser.parse_args()

    name = args.name or args.image.stem
    cells, report = snap(Image.open(args.image), args.grid)
    shape = layout(cells)
    if shape is None:
        sys.exit(f"error: {args.image}: no opaque pixels after snapping")

    args.out.mkdir(parents=True, exist_ok=True)
    to_png(cells).save(args.out / f"{name}.png")
    (args.out / f"{name}.svg").write_text(to_svg(cells, name))
    to_png(cells, 8, STAGE).save(args.out / f"{name}.8x.png")

    print(f"{name}: {args.grid}x{args.grid}, art {shape['art']}, {len(shape['colors'])} colors")
    print(f"  margins: left {shape['left']}, right {shape['right']}, top {shape['top']}, bottom {shape['bottom']}")
    print(f"  palette distance: {report['palette_distance']:.1f} (0 = already on palette; above 40 = recolored)")
    print(f"  unsure cells: {len(report['unsure'])} (no clear majority color; check these by eye)")
    warnings = []
    if min(shape["left"], shape["right"], shape["top"]) < 4:
        warnings.append("art is closer than 4 px to an edge")
    if report["palette_distance"] > 40:
        warnings.append("input colors are far from the palette; the snap changed them a lot")
    if len(report["unsure"]) > args.grid:
        warnings.append("many unsure cells; the grid size may be wrong")
    for w in warnings:
        print(f"  warning: {w}")
    print(f"  wrote {args.out}/{name}.png, .svg, .8x.png")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Round-trip test for scripts/pixel_snap.py.

Take a real Kyaro icon, degrade it the way an image model does (upscale,
stretch off-square, blur, color noise), snap it back to its 48 px grid and
require the exact original pixels.

    python3 -m unittest scripts/tests/test_pixel_snap.py    (needs Pillow)
"""

import random
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from kyaro_icons import FRAMES, ICONS, read_png  # noqa: E402

try:
    from PIL import Image, ImageFilter

    import pixel_snap
except ImportError:  # pragma: no cover - Pillow is optional for the repo
    Image = None


@unittest.skipIf(Image is None, "Pillow is not installed")
class PixelSnapRoundTrip(unittest.TestCase):
    def art(self, name, x0):
        rows = read_png(FRAMES / ICONS[name][0])
        return [
            ["#{:02x}{:02x}{:02x}".format(*px[:3]) if px[3] else None for px in row[x0 : x0 + 48]]
            for row in rows[:48]
        ]

    def degrade(self, cells, seed):
        rng = random.Random(seed)
        img = pixel_snap.to_png(cells, 16).resize((775, 761), Image.BILINEAR)
        img = img.filter(ImageFilter.GaussianBlur(1.2))
        px = img.load()
        for y in range(img.height):
            for x in range(img.width):
                r, g, b, a = px[x, y]
                j = rng.randint(-14, 14)
                px[x, y] = (min(255, max(0, r + j)), min(255, max(0, g + j)), min(255, max(0, b + j)), a)
        return img

    def test_recovers_the_exact_pixels(self):
        for name, x0, seed in (("danger", 7, 1), ("asking", 15, 2), ("ready", 8, 3)):
            with self.subTest(icon=name):
                cells = self.art(name, x0)
                snapped, report = pixel_snap.snap(self.degrade(cells, seed), 48)
                self.assertEqual(snapped, cells)
                # Noise may leave a borderline cell; it is flagged, not wrong.
                self.assertLessEqual(len(report["unsure"]), 3)

    def test_maps_off_palette_colors_to_the_palette(self):
        self.assertEqual(pixel_snap.nearest((250, 40, 45))[0], "#ef3333")
        self.assertEqual(pixel_snap.nearest((10, 10, 10))[0], "#000000")

    def test_svg_has_one_path_per_color(self):
        cells = self.art("ready", 8)
        svg = pixel_snap.to_svg(cells, "ready")
        colors = {c for row in cells for c in row if c}
        self.assertEqual(svg.count("<path "), len(colors))
        self.assertIn('viewBox="0 0 48 48"', svg)


if __name__ == "__main__":
    unittest.main()

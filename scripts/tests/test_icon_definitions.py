#!/usr/bin/env python3
"""
Guard for docs/brand/icons/definitions.json, the input Codex reads.

Every definition must be complete for its status, use the style spec's
palette, point at Kyaro icons and frames that exist, and appear in the
human copy (DEFINITIONS.md). The slot counts must add up to the inventory.

    python3 -m unittest scripts/tests/test_icon_definitions.py    (stdlib only)
"""

import json
import re
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from kyaro_icons import ICONS  # noqa: E402
from pixel_snap import PALETTE  # noqa: E402

ICONS_DIR = ROOT / "docs/brand/icons"
DATA = json.loads((ICONS_DIR / "definitions.json").read_text())
DOC = (ICONS_DIR / "DEFINITIONS.md").read_text()

FAMILY_STATUS = {
    "kyaro-existing": {"exists"},
    "kyaro-new-pose": {"to-generate"},
    "object": {"to-generate", "dropped"},
    "terminal-glyph": {"glyph"},
}


class IconDefinitions(unittest.TestCase):
    def test_ids_are_unique(self):
        ids = [d["id"] for d in DATA["definitions"]]
        self.assertEqual(len(ids), len(set(ids)))

    def test_status_fits_the_family(self):
        for d in DATA["definitions"]:
            with self.subTest(icon=d["id"]):
                self.assertIn(d["status"], FAMILY_STATUS[d["family"]])

    def test_generated_icons_are_complete(self):
        for d in DATA["definitions"]:
            if d["status"] not in ("to-generate", "dropped"):
                continue
            with self.subTest(icon=d["id"]):
                self.assertIn(d["grid"], (24, 48))
                self.assertIn(d["priority"], (1, 2, 3))
                self.assertTrue(d["subject"] and d["must"] and d["must_not"])
                prompts = [v["prompt"] for v in d.get("variants", [])] or [d["prompt"]]
                for prompt in prompts:
                    self.assertTrue(prompt.startswith(DATA["style_prompt"]))
                    self.assertIn(f"{d['grid']}x{d['grid']}", prompt)
                for frame in d.get("reference", []):
                    self.assertTrue((ROOT / frame).is_file(), frame)

    def test_kyaro_icons_exist(self):
        for d in DATA["definitions"]:
            with self.subTest(icon=d["id"]):
                if d["family"] == "kyaro-existing":
                    self.assertIn("kyaro", d)
                for name in filter(None, (d.get("kyaro"), d.get("interim_kyaro"))):
                    self.assertIn(name, ICONS)

    def test_every_icon_has_a_small_slot_glyph(self):
        for d in DATA["definitions"]:
            with self.subTest(icon=d["id"]):
                glyph = d.get("glyph") if d["family"] == "terminal-glyph" else d.get("small_slot_glyph")
                self.assertTrue(glyph)

    def test_palette_matches_the_spec_and_the_snap_tool(self):
        row = next(line for line in DOC.splitlines() if line.startswith("| Palette |"))
        self.assertEqual(DATA["palette"], re.findall(r"#[0-9a-f]{6}", row))
        self.assertEqual(DATA["palette"], PALETTE)
        for color in PALETTE:
            self.assertIn(color, DATA["style_prompt"])

    def test_every_definition_is_in_the_catalog(self):
        for d in DATA["definitions"]:
            with self.subTest(icon=d["id"]):
                self.assertIn(f"`{d['id']}`", DOC)

    def test_slot_counts_add_up(self):
        inv = DATA["inventory"]
        slots = sum(len(d["slots"]) for d in DATA["definitions"])
        self.assertEqual(slots, inv["concept_slots"])
        for d in DATA["definitions"]:
            with self.subTest(icon=d["id"]):
                for fit in ("icon", "glyph"):
                    self.assertEqual(sum(s["fit"] == fit for s in d["slots"]), d["count"][fit])
                self.assertEqual(len(d["slots"]), d["count"]["icon"] + d["count"]["glyph"])
        third = sum(m["slots"] for m in DATA["third_party_marks"])
        self.assertEqual(third, inv["third_party_slots"])
        self.assertEqual(len(DATA["not_an_icon"]), inv["not_an_icon"])
        self.assertEqual(slots + third + len(DATA["not_an_icon"]), inv["hits"])


if __name__ == "__main__":
    unittest.main()

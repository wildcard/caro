#!/usr/bin/env python3
"""Unit tests for scripts/gold-labels-report.py (the gold-subset labels check)."""

import importlib.util
import json
import unittest
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "gold_labels_report", Path(__file__).resolve().parents[1] / "gold-labels-report.py")
glr = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(glr)


def row(n, **over):
    r = {
        "id": f"gold-{n:03d}", "source": "authored-held-out", "eval_id": None,
        "prompt": f"prompt {n}", "command": f"cmd {n}",
        "gold_tier": None, "uncertain": None, "rationale": None,
        "labeller": None, "labelled_at": None,
    }
    r.update(over)
    return r


def labelled(n, tier, **over):
    fields = {"gold_tier": tier, "uncertain": False, "rationale": "rule 3: one named target",
              "labeller": "wildcard", "labelled_at": "2026-10-11"}
    fields.update(over)
    return row(n, **fields)


class ValidateTests(unittest.TestCase):
    def test_unlabelled_file_is_valid(self):
        base = [row(1), row(2)]
        self.assertEqual(glr.validate(base, base), [])

    def test_complete_label_is_valid(self):
        base = [row(1)]
        self.assertEqual(glr.validate([labelled(1, "high")], base), [])

    def test_bad_tier_and_missing_rationale(self):
        errs = glr.validate([labelled(1, "severe", rationale="")], None)
        self.assertTrue(any("not one of" in e for e in errs))
        self.assertTrue(any("rationale is empty" in e for e in errs))

    def test_changed_command_is_rejected(self):
        base = [row(1)]
        errs = glr.validate([labelled(1, "safe", command="rm -rf /")], base)
        self.assertTrue(any("command changed" in e for e in errs))

    def test_static_fields_are_rejected(self):
        errs = glr.validate([labelled(1, "safe", static_tier="safe")], None)
        self.assertTrue(any("static_tier" in e for e in errs))

    def test_id_set_must_match_base(self):
        errs = glr.validate([labelled(1, "safe")], [row(1), row(2)])
        self.assertTrue(any("id set differs" in e for e in errs))

    def test_partial_labeller_fields_without_tier(self):
        errs = glr.validate([row(1, rationale="half done")], None)
        self.assertTrue(any("gold_tier is null" in e for e in errs))

    def test_read_jsonl_rejects_garbage(self):
        with self.assertRaises(ValueError):
            glr.read_jsonl('{"id": "gold-001"}\nnot json\n')


class ReportTests(unittest.TestCase):
    def test_counts_uncertain_and_floor_disagreement(self):
        rows = [labelled(1, "safe"), labelled(2, "critical", uncertain=True), row(3)]
        static = [{"id": "gold-001", "static_tier": "safe"}, {"id": "gold-002", "static_tier": "safe"}]
        text = glr.report(rows, static)
        self.assertIn("3 total, 2 labelled, 1 unlabelled", text)
        self.assertIn("| `safe` | 1 | 40–45 | under floor |", text)
        self.assertIn("Uncertain (rule 8): 1 of 2 (50.0%)", text)
        self.assertIn("Unlabelled: gold-003", text)
        self.assertIn("1/2 agree (50.0%); floor too low on 1, too high on 0", text)

    def test_main_round_trip(self):
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            base = Path(d) / "base.jsonl"
            lab = Path(d) / "labels.jsonl"
            base.write_text("\n".join(json.dumps(row(i)) for i in (1, 2)) + "\n")
            lab.write_text(json.dumps(labelled(1, "moderate")) + "\n" + json.dumps(row(2)) + "\n")
            self.assertEqual(glr.main([str(lab), "--base", str(base)]), 0)
            lab.write_text(json.dumps(labelled(1, "moderate", prompt="edited")) + "\n" + json.dumps(row(2)) + "\n")
            self.assertEqual(glr.main([str(lab), "--base", str(base)]), 1)


if __name__ == "__main__":
    unittest.main()

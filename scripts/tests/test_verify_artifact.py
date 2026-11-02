#!/usr/bin/env python3
"""Unit tests for bin/verify-artifact.

Run:
    python3 scripts/tests/test_verify_artifact.py
"""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

_THIS = Path(__file__).resolve()
_REPO = _THIS.parent.parent.parent
_HELPER = _REPO / "bin" / "verify-artifact"


def _load_module():
    # bin/verify-artifact has no .py extension, so we have to use a SourceFileLoader
    # explicitly rather than relying on the default suffix-based discovery.
    # The module must be registered in sys.modules before exec_module so that
    # dataclass annotation introspection (which calls sys.modules.get(__module__))
    # succeeds on Python 3.9.
    import sys as _sys
    from importlib.machinery import SourceFileLoader

    loader = SourceFileLoader("verify_artifact", str(_HELPER))
    spec = importlib.util.spec_from_loader("verify_artifact", loader)
    assert spec is not None
    mod = importlib.util.module_from_spec(spec)
    _sys.modules["verify_artifact"] = mod
    loader.exec_module(mod)
    return mod


verify = _load_module()


class QuoteGroundingTests(unittest.TestCase):
    def test_quoted_span_present_in_evidence_passes(self) -> None:
        text = 'Per the docs: "the safety validator catches dangerous commands".'
        evidence = "the safety validator catches dangerous commands and refuses them"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_short_quotes_are_ignored(self) -> None:
        # Quotes under 8 chars are skipped to avoid noise on short tokens
        text = 'See the "rm" command.'
        evidence = "command list does not mention rm"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_quoted_span_absent_warns_by_default(self) -> None:
        text = 'The report says "the agent fabricated this entire sentence".'
        evidence = "evidence corpus with unrelated content"
        report = verify.audit(text, evidence)
        # default = warn, so report.ok stays True
        self.assertTrue(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("quote_not_in_source", codes)

    def test_strict_quotes_flips_to_fail(self) -> None:
        text = 'The report says "the agent fabricated this entire sentence".'
        evidence = "evidence corpus with unrelated content"
        report = verify.audit(text, evidence, strict_quotes=True)
        self.assertFalse(report.ok)


class IssueRefTests(unittest.TestCase):
    def test_allowed_pr_passes(self) -> None:
        text = "Fixed in PR #1154."
        report = verify.audit(text, "evidence", allowed_prs=[1154])
        self.assertTrue(report.ok)

    def test_unknown_pr_fails(self) -> None:
        text = "Tracked in #9999."
        report = verify.audit(text, "evidence with no issue refs", allowed_prs=[1154])
        self.assertFalse(report.ok)
        self.assertEqual(report.findings[0].code, "fabricated_issue_reference")

    def test_pr_in_evidence_passes(self) -> None:
        text = "See #862."
        evidence = "previous comment thread references #862 multiple times"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)


class UrlTests(unittest.TestCase):
    def test_url_in_evidence_passes(self) -> None:
        text = "Per https://github.com/wildcard/caro/pull/1154 the fix landed."
        evidence = "merged https://github.com/wildcard/caro/pull/1154 yesterday"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_fabricated_url_fails(self) -> None:
        text = "See https://evil.example.com/exploit for details."
        report = verify.audit(text, "evidence", allowed_urls=[])
        self.assertFalse(report.ok)
        self.assertEqual(report.findings[0].code, "fabricated_url")

    def test_explicitly_allowed_url_passes(self) -> None:
        text = "See https://docs.caro.sh/safety for details."
        report = verify.audit(
            text, "evidence", allowed_urls=["https://docs.caro.sh/safety"]
        )
        self.assertTrue(report.ok)


class MentionTests(unittest.TestCase):
    def test_allowed_mention_passes(self) -> None:
        text = "Thanks @kobi-kadosh for the review."
        report = verify.audit(text, "evidence", allowed_mentions=["kobi-kadosh"])
        self.assertTrue(report.ok)

    def test_unknown_mention_warns_not_fails(self) -> None:
        text = "Thanks @random-username for the review."
        report = verify.audit(text, "evidence", allowed_mentions=["kobi-kadosh"])
        # default severity for unknown mentions is "warn"
        self.assertTrue(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("fabricated_mention", codes)


class PathTests(unittest.TestCase):
    def test_path_in_evidence_passes(self) -> None:
        text = "See src/safety/patterns.rs for the list."
        evidence = "the file src/safety/patterns.rs holds 52 patterns"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_path_with_only_basename_in_evidence_passes_as_warn_skipped(self) -> None:
        # If the basename appears, we tolerate (warn-suppression)
        text = "Edit src/safety/patterns.rs to add the pattern."
        evidence = "the patterns.rs file is the source of truth"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_completely_invented_path_warns(self) -> None:
        text = "See src/imaginary/nonexistent_module.rs for details."
        report = verify.audit(text, "evidence with no matching path or basename")
        self.assertTrue(report.ok)  # warn only
        codes = {f.code for f in report.findings}
        self.assertIn("suspicious_path", codes)


class ShaTests(unittest.TestCase):
    def test_sha_in_evidence_passes(self) -> None:
        text = "Landed in 8155c4af."
        evidence = "commit 8155c4af fixed the deps issue"
        report = verify.audit(text, evidence)
        self.assertTrue(report.ok)

    def test_fabricated_sha_fails(self) -> None:
        text = "Landed in deadbeefcafe123."
        report = verify.audit(text, "evidence with no sha")
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("fabricated_sha", codes)


class IntegrationScenarios(unittest.TestCase):
    """Negative + positive tests matching the plan's verification cases."""

    def test_negative_fabricated_pr_number(self) -> None:
        text = "Tracked in #9999."
        evidence = '{"title": "real pr", "body": "body of PR 1154"}'
        report = verify.audit(text, evidence, allowed_prs=[1154])
        self.assertFalse(report.ok)

    def test_negative_fabricated_url(self) -> None:
        text = "See https://evil.example.com/exploit for details."
        evidence = "real evidence with no such url"
        report = verify.audit(text, evidence)
        self.assertFalse(report.ok)

    def test_positive_grounded_release_note_passes(self) -> None:
        text = (
            "Release v1.4.0 closes #1154 and lands the bincode pin fix from "
            "commit 8155c4af. See src/safety/patterns.rs for the new rules."
        )
        evidence = (
            "PR #1154 title: fix(deps,build): unblock main — bincode pin + "
            "rusqlite cast + candle align. Merge commit 8155c4af touched "
            "src/safety/patterns.rs and Cargo.toml."
        )
        report = verify.audit(text, evidence, allowed_prs=[1154])
        self.assertTrue(report.ok, msg=report.to_human())


class ReportShapeTests(unittest.TestCase):
    def test_to_dict_shape(self) -> None:
        text = "Tracked in #9999."
        report = verify.audit(text, "evidence")
        as_dict = report.to_dict()
        self.assertIn("ok", as_dict)
        self.assertIn("findings", as_dict)
        self.assertFalse(as_dict["ok"])
        self.assertGreaterEqual(len(as_dict["findings"]), 1)


if __name__ == "__main__":
    unittest.main()

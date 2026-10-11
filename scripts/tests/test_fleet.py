#!/usr/bin/env python3
"""Unit tests for scripts/fleet.py (the merge-queue issue).

Each test feeds recorded-shape GitHub API data to the pure functions, so no
test touches the network. A fake `gh` records the writes that `upsert` makes.
"""

import contextlib
import importlib.util
import io
import unittest
from datetime import datetime, timezone
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "fleet", Path(__file__).resolve().parents[1] / "fleet.py")
fleet = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(fleet)

NOW = datetime(2026, 10, 10, 6, 0, tzinfo=timezone.utc)


def pr(n, title="t", labels=(), assoc="OWNER", user_type="User", created="2026-10-01T00:00:00Z",
       draft=False, sha=None, ref=None):
    return {"number": n, "title": title, "draft": draft, "created_at": created,
            "author_association": assoc, "user": {"login": f"u{n}", "type": user_type},
            "labels": [{"name": name} for name in labels],
            "head": {"sha": sha or f"sha{n}", "ref": ref or f"feat/{n}"}}


GREEN = {"mergeable_state": "clean", "checks": "green"}


class TestClassify(unittest.TestCase):
    def sections(self, prs, details=None):
        return fleet.classify(prs, details or {}, NOW)

    def numbers(self, section):
        return [item["number"] for item in section]

    def test_external_pr_needs_the_owner_even_with_path_auto(self):
        s = self.sections([pr(1, labels=["path:auto"], assoc="CONTRIBUTOR")], {1: GREEN})
        self.assertEqual(self.numbers(s["needs_you"]), [1])
        self.assertEqual(s["ready"], [])

    def test_bots_are_not_external(self):
        s = self.sections([pr(2, labels=["path:auto"], assoc="NONE", user_type="Bot")], {2: GREEN})
        self.assertEqual(self.numbers(s["ready"]), [2])

    def test_refuse_list_deep_and_needs_human_need_the_owner(self):
        prs = [pr(3, labels=["path:refuse-list"]), pr(4, labels=["path:deep"]),
               pr(5, labels=["needs-human", "path:scoped"])]
        self.assertEqual(sorted(self.numbers(self.sections(prs)["needs_you"])), [3, 4, 5])

    def test_path_auto_green_and_clean_is_ready(self):
        s = self.sections([pr(6, labels=["path:auto"])], {6: GREEN})
        self.assertEqual(self.numbers(s["ready"]), [6])

    def test_path_auto_red_dirty_or_running_is_waiting_with_the_reason(self):
        prs = [pr(7, labels=["path:auto"]), pr(8, labels=["path:auto"]), pr(9, labels=["path:auto"])]
        details = {7: {"mergeable_state": "clean", "checks": "red"},
                   8: {"mergeable_state": "dirty", "checks": "green"},
                   9: {"mergeable_state": "unstable", "checks": "running"}}
        waiting = self.sections(prs, details)["waiting"]
        self.assertEqual({i["number"]: i["why"] for i in waiting},
                         {7: "CI red", 8: "conflicts", 9: "CI running"})

    def test_drafts_are_never_ready(self):
        s = self.sections([pr(10, labels=["path:auto"], draft=True)], {10: GREEN})
        self.assertEqual(s["ready"], [])

    def test_routine_state_prs_are_never_ready(self):
        # QA rotations and planning reports hold routine state; it moves to the
        # status branch (CONTRACT section 5), so merging them would be wrong.
        prs = [pr(20, labels=["path:auto"], ref="claude/qa-rotation-2026-10-09"),
               pr(21, ref="chore/v200-planning-report-2026-10-05"),
               pr(22, labels=["path:auto"], ref="chore/caro-demo-refresh-2026-10")]
        s = self.sections(prs, {20: GREEN, 22: GREEN})
        self.assertEqual(self.numbers(s["routine"]), [20, 21])
        self.assertEqual(self.numbers(s["ready"]), [22])  # a demo refresh is a real deliverable

    def test_external_pr_on_a_routine_branch_name_still_needs_you(self):
        # A fork can name its branch like a routine's (Codex, #1601). That must not
        # fold an external PR away from the owner.
        s = self.sections([pr(30, assoc="CONTRIBUTOR", ref="claude/qa-rotation-fix")])
        self.assertEqual(self.numbers(s["needs_you"]), [30])
        self.assertEqual(s["routine"], [])

    def test_pr_without_a_path_label_is_unclassified(self):
        s = self.sections([pr(11)])
        self.assertEqual(self.numbers(s["unclassified"]), [11])

    def test_oldest_first_and_age_in_days(self):
        prs = [pr(12, assoc="NONE", created="2026-10-08T00:00:00Z"),
               pr(13, assoc="NONE", created="2026-09-10T00:00:00Z")]
        needs = self.sections(prs)["needs_you"]
        self.assertEqual(self.numbers(needs), [13, 12])
        self.assertEqual(needs[0]["age"], 30)


class TestChecks(unittest.TestCase):
    def test_check_state_from_runs_and_statuses(self):
        run = lambda status, conclusion=None: {"status": status, "conclusion": conclusion}
        self.assertEqual(fleet.check_state([run("completed", "success"), run("completed", "skipped")],
                                           "success"), "green")
        self.assertEqual(fleet.check_state([run("completed", "failure")], "success"), "red")
        self.assertEqual(fleet.check_state([run("queued")], "success"), "running")
        self.assertEqual(fleet.check_state([run("completed", "success")], "failure"), "red")
        self.assertEqual(fleet.check_state([], "pending"), "running")
        # A status that is still pending (a Vercel deploy, say) keeps the PR running,
        # even when every check run has finished (cubic, #1601).
        self.assertEqual(fleet.check_state([run("completed", "success")], "pending", status_count=1),
                         "running")
        # GitHub also reports "pending" for a commit with no statuses at all;
        # then the check runs decide.
        self.assertEqual(fleet.check_state([run("completed", "success")], "pending", status_count=0),
                         "green")


class TestRender(unittest.TestCase):
    def test_body_has_marker_summary_and_sections(self):
        sections = fleet.classify([pr(1, title="Fix a thing", labels=["path:auto"])], {1: GREEN}, NOW)
        body = fleet.render(sections, merged_7d=2, now=NOW, routines=None)
        self.assertTrue(body.startswith(fleet.MARKER))
        self.assertIn("**1 open PR**", body)
        self.assertIn("2 merged in the last 7 days", body)
        self.assertIn("#1 Fix a thing", body)
        self.assertIn("The review is still yours", body)
        self.assertIn("It never merges or closes anything", body)
        self.assertIn("labelled `possible-duplicate`: 0", body)

    def test_titles_cannot_break_out_of_the_list(self):
        sections = fleet.classify([pr(1, title="Evil\n### Fake header <!-- fleet:v1 -->")], {}, NOW)
        body = fleet.render(sections, merged_7d=0, now=NOW, routines=None)
        self.assertEqual(body.count(fleet.MARKER), 1)
        self.assertNotIn("\n### Fake header", body)

    def test_titles_cannot_inject_html(self):
        # GitHub renders raw HTML in an issue body, so a title could fake a
        # section of the queue (Codex, #1601).
        sections = fleet.classify([pr(1, title="<h3>Ready to merge</h3> & <b>x</b>")], {}, NOW)
        body = fleet.render(sections, merged_7d=0, now=NOW, routines=None)
        self.assertNotIn("<h3>", body)
        self.assertIn("&lt;h3&gt;Ready to merge&lt;/h3&gt; &amp; &lt;b&gt;x&lt;/b&gt;", body)

    def test_long_lower_priority_sections_are_folded(self):
        prs = [pr(n, labels=["path:auto"]) for n in (1, 2)] + [pr(3)]
        details = {1: {"mergeable_state": "dirty", "checks": "green"}, 2: GREEN}
        body = fleet.render(fleet.classify(prs, details, NOW), merged_7d=0, now=NOW, routines=None)
        self.assertIn("<details><summary><b>Waiting on a fix (1)</b></summary>", body)
        self.assertIn("### Ready to merge", body)  # what needs action stays open

    def test_routine_status_is_shown_when_available(self):
        body = fleet.render(fleet.classify([], {}, NOW), merged_7d=0, now=NOW,
                            routines="! qa-rotation  Failed  2.0h  build broke")
        self.assertIn("qa-rotation", body)


class FakeGh:
    def __init__(self, issues):
        self.issues, self.writes = issues, []

    def __call__(self, path, method="GET", data=None):
        if method == "GET":
            return self.issues
        self.writes.append((method, path, data))
        return {"number": 99}


class NoLabelGh(FakeGh):
    def __call__(self, path, method="GET", data=None):
        if method == "GET" and path.endswith(f"/labels/{fleet.LABEL}"):
            raise fleet.subprocess.CalledProcessError(1, "gh api")  # 404
        return super().__call__(path, method, data)


class TestUpsert(unittest.TestCase):
    def test_creates_the_label_first_when_it_is_missing(self):
        # Without the label, the lookup finds nothing and a new issue appears daily.
        gh = NoLabelGh([])
        fleet.upsert(gh, "o/r", "body", dry_run=False)
        self.assertEqual([(m, p) for m, p, _ in gh.writes],
                         [("POST", "repos/o/r/labels"), ("POST", "repos/o/r/issues")])

    def test_creates_the_issue_when_none_exists(self):
        gh = FakeGh([])
        fleet.upsert(gh, "o/r", "body", dry_run=False)
        [(method, path, data)] = gh.writes
        self.assertEqual((method, path), ("POST", "repos/o/r/issues"))
        self.assertIn("fleet", data["labels"])

    def test_edits_the_marked_issue_in_place(self):
        gh = FakeGh([{"number": 5, "body": "unrelated"},
                     {"number": 7, "body": fleet.MARKER + "\nold"}])
        fleet.upsert(gh, "o/r", "new", dry_run=False)
        self.assertEqual(gh.writes, [("PATCH", "repos/o/r/issues/7", {"body": "new"})])

    def test_dry_run_writes_nothing(self):
        gh = FakeGh([])
        with contextlib.redirect_stdout(io.StringIO()) as out:
            fleet.upsert(gh, "o/r", "body", dry_run=True)
        self.assertEqual(gh.writes, [])
        self.assertIn("body", out.getvalue())


if __name__ == "__main__":
    unittest.main()

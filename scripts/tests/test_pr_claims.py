"""Regression guards for scripts/pr_claims.py, the claims check.

The incident case is PR #1415 (2026-10-15 review). Its description said
"Closes #1408", "Added a focused regression test" and listed a passing
`cargo test` run. At its head the PR was one constant and one `eprintln!`,
the named test had been deleted, and no CI job had run. An agent had
thanked it for "this fix" from the description alone.
"""

import importlib.util
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("pr_claims", ROOT / "scripts/pr_claims.py")
pr_claims = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pr_claims)

HEAD_1415 = "d9d417a8f25cd0e77cb564d010f2294af533a88b"
BODY_1415 = """## Summary

Closes #1408.

`caro ai --once` now writes an initialization status line to stderr before backend construction.

## Implementation

- Added a stable initialization status message for the one-shot AI path.
- Added a focused regression test for the status message.

## Testing

- `cargo test --bin caro ai_once_initialization_message_is_stable` — passed (1 passed, 22 filtered out).

<!-- This is an auto-generated description by cubic. -->
## Summary by cubic
- Added tests that `cargo test --bin caro cubic_invented_name` covers.
<!-- End of auto-generated description by cubic. -->
"""
PATCH_1415 = """@@ -1065,6 +1065,8 @@
+const AI_ONCE_INITIALIZATION_MESSAGE: &str = "Initializing backend…";
+
@@ -1097,6 +1099,7 @@
+    eprintln!("{}", AI_ONCE_INITIALIZATION_MESSAGE);"""
ISSUE_1408 = ("Inside `run_ai_once()` (src/main.rs:1072), the first blocking operation is "
              "src/main.rs:1100-1108.")

HEAD_1296 = "d4dfb7f0000000000000000000000000000000ab"
BODY_1296 = """## Changes Made

- accept `telemetry.first_run` in `config set/get`
- extend the existing config CLI integration suite with round-trip coverage

- [x] I have added tests that prove my fix is effective
- [ ] I have added contract tests for new public APIs (if applicable)

```text
$ cargo test --locked --test config_cli_keys
test telemetry_first_run_is_configurable_and_shown ... ok
test invalid_values_are_rejected ... ok
test result: ok. 2 passed; 0 failed
```
"""
PATCH_1296 = """@@ -65,6 +65,19 @@
+#[test]
+fn telemetry_first_run_is_configurable_and_shown() {"""


def fake_api(prs, runs, issues=None, jobs=0):
    """A stand-in for gh_json, keyed by REST path."""
    issues = issues or {}

    def gh_json(path):
        for number, pr in prs.items():
            if path == f"/repos/o/r/pulls/{number}":
                return pr
        if "/actions/runs?head_sha=" in path:
            return {"workflow_runs": runs}
        if path.endswith("/jobs?per_page=1"):
            return {"total_count": jobs}
        for number, body in issues.items():
            if path == f"/repos/o/r/issues/{number}":
                return {"state": "open", "body": body}
        raise AssertionError(f"unexpected API path {path}")
    return gh_json


class ClaimsCheckTests(unittest.TestCase):
    def setUp(self):
        self.saved = {n: getattr(pr_claims, n) for n in ("gh_json", "gh_pages", "fetch_head", "git_has")}

    def tearDown(self):
        for name, fn in self.saved.items():
            setattr(pr_claims, name, fn)

    def stub(self, number, head, body, files, runs, existing, issues=None, jobs=0):
        pr = {"head": {"sha": head}, "body": body,
              "additions": sum(f["additions"] for f in files), "deletions": 0}
        pr_claims.gh_json = fake_api({number: pr}, runs, issues, jobs)
        pr_claims.gh_pages = lambda path: files
        pr_claims.fetch_head = lambda n: head
        pr_claims.git_has = lambda commit, name: name in existing

    def test_incident_1415_reports_missing_test_missing_test_code_and_no_ci_jobs(self):
        runs = [{"id": i, "name": n, "event": "pull_request", "status": "completed", "conclusion": "failure"}
                for i, n in enumerate(["CI", "Cross-Platform CI", "Codespell"])]
        # Base-branch bookkeeping runs had jobs; they must not count as CI.
        runs.append({"id": 99, "name": "CLA Assistant", "event": "pull_request_target",
                     "status": "completed", "conclusion": "failure"})
        self.stub(1415, HEAD_1415, BODY_1415,
                  [{"filename": "src/main.rs", "additions": 3, "deletions": 0, "patch": PATCH_1415}],
                  runs, existing=set(), issues={1408: ISSUE_1408}, jobs=0)
        report, missing = pr_claims.check("o/r", 1415)
        self.assertEqual(missing, 3, report)
        self.assertRegex(report, r"Closes #1408 \| #1408 is open; the diff touches 1 of the 1 files it names \| ✅")
        self.assertRegex(report, r"Names test ai_once_initialization_message_is_stable \| not found at the head \| ❌")
        self.assertRegex(report, r"Says: - Added a focused regression test.*\| the diff adds no test code \| ❌")
        self.assertRegex(report, r"\| CI \| 3 workflow runs, 0 jobs ran on the head \| ❌")
        self.assertNotIn("cubic_invented_name", report)

    def test_good_pr_passes_and_fork_runs_waiting_for_approval_are_flagged_not_failed(self):
        runs = [{"id": i, "name": f"wf{i}", "event": "pull_request", "status": "completed",
                 "conclusion": "action_required"} for i in range(9)]
        files = [{"filename": "src/main.rs", "additions": 25, "deletions": 3, "patch": "+ ok"},
                 {"filename": "tests/config_cli_keys.rs", "additions": 14, "deletions": 0, "patch": PATCH_1296}]
        self.stub(1296, HEAD_1296, BODY_1296, files, runs,
                  existing={"tests/config_cli_keys.rs", "telemetry_first_run_is_configurable_and_shown",
                            "invalid_values_are_rejected"})
        report, missing = pr_claims.check("o/r", 1296)
        self.assertEqual(missing, 0, report)
        self.assertIn("Names test tests/config_cli_keys.rs | found at the head | ✅", report)
        self.assertRegex(report, r"\| CI \| 9 of 9 workflow runs wait for a maintainer to approve them; 0 jobs ran \| ⚠️")

    def test_a_stale_local_fetch_is_an_error_not_a_report(self):
        self.stub(1296, HEAD_1296, BODY_1296, [], [], existing=set())
        pr_claims.fetch_head = lambda n: "0" * 40
        with self.assertRaises(RuntimeError):
            pr_claims.check("o/r", 1296)

    def test_main_exit_codes(self):
        self.stub(1296, HEAD_1296, BODY_1296, [], [], existing=set())
        with redirect_stdout(io.StringIO()):
            self.assertEqual(pr_claims.main(["1296", "--repo", "o/r"]), 1)  # names tests that are missing

        def broken(path):
            raise RuntimeError("HTTP 404")
        pr_claims.gh_json = broken
        with redirect_stderr(io.StringIO()) as err:
            self.assertEqual(pr_claims.main(["1296", "--repo", "o/r"]), 2)
        self.assertIn("HTTP 404", err.getvalue())


class ParserTests(unittest.TestCase):
    def test_named_tests_reads_cargo_lines_test_output_and_guard_lines(self):
        body = ("Run `cargo test --features embedded-cpu,remote-backends --lib backends::remote::claude`.\n"
                "Run cargo test -p caro --test explain_ste_contract -- --nocapture extra_after_dashes\n"
                "test cli::tests::test_default_shell_is_posix_on_unix ... ok\n"
                "Guarded by `tests/e2e_guard.rs` and `guard_setup_is_idempotent`; see `README`.\n")
        self.assertEqual(pr_claims.named_tests(body), [
            "backends::remote::claude", "cli::tests::test_default_shell_is_posix_on_unix",
            "guard_setup_is_idempotent", "tests/e2e_guard.rs", "tests/explain_ste_contract.rs"])

    def test_unticked_checkboxes_are_not_claims(self):
        self.assertIsNone(pr_claims.claims_added_tests("- [ ] I have added tests that prove my fix\n"))
        self.assertTrue(pr_claims.claims_added_tests("- [x] I have added tests that prove my fix\n"))

    def test_removed_tests_do_not_count_as_added(self):
        removed = "@@ -1,5 +1,0 @@\n-    #[test]\n-    fn ai_once_initialization_message_is_stable() {"
        self.assertFalse(pr_claims.diff_adds_tests([{"filename": "src/main.rs", "additions": 0, "patch": removed}]))
        self.assertTrue(pr_claims.diff_adds_tests([{"filename": "tests/new_case.rs", "additions": 4}]))
        self.assertTrue(pr_claims.diff_adds_tests([{"filename": "web/x.test.ts", "additions": 2}]))

    def test_linked_issues_and_bot_text(self):
        self.assertEqual(pr_claims.linked_issues("Closes #1408. Fixes: #12, resolves #12"), [12, 1408])
        self.assertNotIn("bot", pr_claims.strip_bot_text(
            "mine\n<!-- This is an auto-generated description by cubic. -->bot<!-- End of auto-generated description by cubic. -->"))

    def test_untrusted_text_cannot_break_the_table(self):
        out = pr_claims.cell("a | b <script>`x`</script>\nnext line " + "y" * 200)
        self.assertNotIn("<", out)
        self.assertNotIn("\n", out)
        self.assertNotIn("`", out)
        self.assertIn("a \\| b", out)
        self.assertLessEqual(len(out), 120)


if __name__ == "__main__":
    unittest.main()

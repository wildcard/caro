#!/usr/bin/env python3
"""Claims check: compare what a pull request says with what it changes.

Run this before you thank, approve or merge a PR, and before you write
"looks right". It checks the claims a script can check and prints a
Markdown table. The reviewer adds the behaviour evidence (the linked
issue's steps, run on main and on the PR head) by hand.
Rule: .claude/rules/feature-evidence.md, "Claims check".

What it checks, at the PR's head commit:
  1. Each test the description names exists (fetched with
     `git fetch origin pull/<n>/head`, then `git grep`).
  2. A description that says it adds tests comes with a diff that adds tests.
  3. CI jobs ran on the head commit (a fork PR's runs can wait for approval).
  4. Each "Closes #N" issue is open, and the diff touches the files it names.
  5. The size of the description next to the size of the diff (information).

Usage: python3 scripts/pr_claims.py <pr-number> [--repo owner/name]
Exit: 0 when no claim lacks evidence, 1 when one does, 2 on a usage or API error.
"""
import argparse
import json
import re
import subprocess
import sys

OK, MISSING, WAIT, INFO = "✅", "❌", "⚠️", "ℹ️"

# Text that bots add to a description is not the author's claim.
BOT_BLOCK = re.compile(r"<!--\s*This is an auto-generated description by cubic\.\s*-->.*?"
                       r"<!--\s*End of auto-generated description by cubic\.\s*-->", re.S)
LINKED = re.compile(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s*:?\s+#(\d+)")
CARGO_TEST = re.compile(r"cargo\s+test\b([^`\n]*)")
TEST_OUTPUT = re.compile(r"^\s*test\s+([A-Za-z_][\w:]*)\s+\.\.\.\s+(?:ok|FAILED|ignored)\b", re.M)
GUARD_LINE = re.compile(r"(?i)regression[- ]guard|guarded by|regression test")
BACKTICKED = re.compile(r"`([A-Za-z_][\w:]*|tests/[\w./-]+\.rs)`")
ADDS_TESTS = re.compile(r"(?i)\b(?:add(?:ed|s|ing)?|new|extend(?:ed|s|ing)?)\b[^.\n]{0,60}?\b(?:tests?|coverage|suite)\b")
CITED_PATH = re.compile(r"\b((?:src|tests|scripts|website|docs|bin|\.github)/[\w./-]*\w\.[A-Za-z]{1,5})\b")
TEST_LINE = re.compile(r"^\+.*(?:#\[(?:tokio::)?test\]|\bfn test_\w*|\bdef test_\w*|\b(?:it|test)\(\s*['\"])")
# cargo test options that take a value.
VALUE_FLAGS = {"--features", "-F", "-p", "--package", "--bin", "--example", "--bench",
               "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--color",
               "--profile", "--message-format", "-Z"}


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def gh_json(path):
    """GET a GitHub REST path through `gh api`."""
    r = run(["gh", "api", path])
    if r.returncode != 0:
        raise RuntimeError(f"gh api {path}: {r.stderr.strip() or r.stdout.strip()}")
    return json.loads(r.stdout)


def gh_pages(path):
    """GET every page of a list endpoint (100 per page)."""
    out, page = [], 1
    sep = "&" if "?" in path else "?"
    while True:
        batch = gh_json(f"{path}{sep}per_page=100&page={page}")
        out.extend(batch)
        if len(batch) < 100:
            return out
        page += 1


def fetch_head(number):
    """Fetch the PR head into the local clone; return its commit."""
    r = run(["git", "fetch", "--quiet", "origin", f"pull/{number}/head"])
    if r.returncode != 0:
        raise RuntimeError(f"git fetch pull/{number}/head: {r.stderr.strip()}")
    return run(["git", "rev-parse", "FETCH_HEAD"]).stdout.strip()


def git_has(commit, name):
    """True when `name` (a test name or a tests/*.rs path) exists at `commit`."""
    if name.startswith("tests/"):
        return run(["git", "cat-file", "-e", f"{commit}:{name}"]).returncode == 0
    word = name.split("::")[-1]
    return run(["git", "grep", "-q", "-w", "-e", word, commit, "--", "*.rs", "*.py", "*.ts",
                "*.js", "*.sh"]).returncode == 0


def strip_bot_text(body):
    return BOT_BLOCK.sub("", body or "")


def linked_issues(body):
    return sorted({int(n) for n in LINKED.findall(body)})


def named_tests(body):
    """Test names and test targets that the description says exist or pass."""
    names = set()
    for args in CARGO_TEST.findall(body):
        tokens = args.split()
        i = 0
        while i < len(tokens):
            tok = tokens[i].strip("`'\",;")
            if tok == "--":
                break
            if tok == "--test" and i + 1 < len(tokens):
                names.add("tests/" + tokens[i + 1].strip("`'\",;") + ".rs")
                i += 2
                continue
            if tok in VALUE_FLAGS:
                i += 2
                continue
            if not tok.startswith("-") and re.fullmatch(r"[A-Za-z_][\w:]*", tok):
                names.add(tok)
            i += 1
    names.update(TEST_OUTPUT.findall(body))
    for line in body.splitlines():
        if GUARD_LINE.search(line):
            names.update(n for n in BACKTICKED.findall(line) if "_" in n or n.startswith("tests/"))
    return sorted(names)


def claims_added_tests(body):
    """The first line that says the PR adds tests; unticked checkboxes do not count."""
    for line in body.splitlines():
        if re.match(r"\s*[-*]\s+\[\s\]", line):
            continue
        if ADDS_TESTS.search(line):
            return line.strip()
    return None


def diff_adds_tests(files):
    for f in files:
        name = f.get("filename", "")
        if f.get("additions", 0) and (name.startswith("tests/") or re.search(
                r"(^|/)test_[^/]+\.py$|_test\.\w+$|\.test\.[jt]sx?$", name)):
            return True
        if any(TEST_LINE.match(line) for line in (f.get("patch") or "").splitlines()):
            return True
    return False


def ci_summary(repo, head):
    """(runs, runs waiting for approval, jobs that ran, failed run names) for the head.

    Only `pull_request` and `push` runs test the code. `pull_request_target`
    runs (CLA, labels, release notes) run base-branch workflows, so they do
    not count: on #1415 they were the only jobs that ever ran.
    """
    runs = [r for r in gh_json(f"/repos/{repo}/actions/runs?head_sha={head}&per_page=100")["workflow_runs"]
            if r.get("event") in ("pull_request", "push")]
    waiting = [r for r in runs if r.get("conclusion") == "action_required" or r.get("status") == "action_required"]
    jobs = sum(gh_json(f"/repos/{repo}/actions/runs/{r['id']}/jobs?per_page=1")["total_count"]
               for r in runs if r not in waiting)
    failed = sorted({r["name"] for r in runs if r.get("conclusion") == "failure"})
    return len(runs), len(waiting), jobs, failed


def cell(text, limit=90):
    """Untrusted PR text, made safe for one Markdown table cell."""
    text = " ".join(str(text).split())
    if len(text) > limit:
        text = text[: limit - 1] + "…"
    return text.replace("|", "\\|").replace("<", "&lt;").replace(">", "&gt;").replace("`", "'")


def check(repo, number):
    """Return (markdown, missing_count) for one PR."""
    pr = gh_json(f"/repos/{repo}/pulls/{number}")
    head = pr["head"]["sha"]
    short = head[:7]
    body = strip_bot_text(pr.get("body"))
    files = gh_pages(f"/repos/{repo}/pulls/{number}/files")
    changed = {f["filename"] for f in files}
    local = fetch_head(number)
    if local != head:
        raise RuntimeError(f"fetched {local[:7]}, but the PR head is {short}; fetch again")
    rows = []

    for issue_no in linked_issues(body):
        issue = gh_json(f"/repos/{repo}/issues/{issue_no}")
        cited = sorted(set(CITED_PATH.findall(issue.get("body") or "")))
        hit = sorted(p for p in cited if p in changed)
        state = "open" if issue.get("state") == "open" else "already closed"
        if not cited:
            evidence, mark = f"#{issue_no} is {state}; it names no files", INFO
        elif hit:
            evidence, mark = f"#{issue_no} is {state}; the diff touches {len(hit)} of the {len(cited)} files it names", OK
        else:
            evidence, mark = (f"#{issue_no} is {state}; the diff touches none of the files it names "
                              f"({', '.join(cited[:3])})"), WAIT
        if state != "open":
            mark = WAIT
        rows.append((f"Closes #{issue_no}", evidence, mark))

    for name in named_tests(body):
        found = git_has(head, name)
        rows.append((f"Names test {name}", "found at the head" if found else "not found at the head",
                     OK if found else MISSING))

    said = claims_added_tests(body)
    if said:
        adds = diff_adds_tests(files)
        rows.append((f"Says: {said}", "the diff adds test code" if adds else "the diff adds no test code",
                     OK if adds else MISSING))

    runs, waiting, jobs, failed = ci_summary(repo, head)
    if waiting:
        rows.append(("CI", f"{waiting} of {runs} workflow runs wait for a maintainer to approve them; "
                           f"{jobs} jobs ran", WAIT))
    elif jobs == 0:
        rows.append(("CI", f"{runs} workflow runs, 0 jobs ran on the head", MISSING))
    elif failed:
        rows.append(("CI", f"{jobs} jobs ran; failed: {', '.join(failed)}", MISSING))
    else:
        rows.append(("CI", f"{jobs} jobs ran on the head, none failed", OK))

    rows.append(("Size", f"description {len(body):,} chars; diff +{pr.get('additions', 0)}/"
                         f"-{pr.get('deletions', 0)} in {len(files)} file{'' if len(files) == 1 else 's'}", INFO))

    missing = sum(1 for _, _, mark in rows if mark == MISSING)
    lines = [f"## Claims check: #{number} at {short}", "",
             f"| # | Claim | Evidence at {short} | Result |", "|---|---|---|---|"]
    lines += [f"| {i} | {cell(c)} | {cell(e, 140)} | {m} |" for i, (c, e, m) in enumerate(rows, 1)]
    lines += ["", f"Missing evidence: {missing}. Resolve each {MISSING} before \"looks right\", "
                  "approval or merge.",
              "Not checked here: behaviour. For \"Closes #N\", run #N's steps on main and on the "
              "PR head, and paste both outputs."]
    return "\n".join(lines), missing


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("number", type=int, help="pull request number")
    ap.add_argument("--repo", default="wildcard/caro", help="owner/name (default: wildcard/caro)")
    args = ap.parse_args(argv)
    try:
        report, missing = check(args.repo, args.number)
    except (RuntimeError, KeyError, json.JSONDecodeError) as exc:
        print(f"pr_claims: {exc}", file=sys.stderr)
        return 2
    print(report)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())

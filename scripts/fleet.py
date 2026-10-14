#!/usr/bin/env python3
"""Fleet: one pinned issue that shows the owner what to merge next.

Caro's merge dispatcher already labels open PRs with their forward path
(`path:auto`, `path:scoped`, `path:deep`, `path:refuse-list`) and `needs-human`,
but nothing shows those labels together with live CI and mergeability, and
`path:auto` PRs wait for weeks. This script adds no classification of its
own. It reads the labels, adds CI state and mergeability for the `path:auto`
PRs, and rewrites one issue (found by MARKER). Like autoharness's promoter,
deterministic code reports; a person decides. It never merges or closes.

Usage: python3 scripts/fleet.py triage [--repo OWNER/REPO] [--dry-run]
Stdlib only; all I/O goes through `gh api`. Plan: P0.3 of the harness plan.
"""
import argparse
import html
import json
import statistics
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

MARKER = "<!-- fleet:v1 -->"
LABEL = "fleet"
DUP_LABEL = "possible-duplicate"  # set by the duplicate check in issue-triage.yml
TITLE = "Fleet: merge queue"
INTERNAL = {"OWNER", "MEMBER", "COLLABORATOR"}
OWNER_LABELS = {"needs-human": "needs a human", "path:refuse-list": "refuse-list path",
                "path:deep": "deep review"}
RED = {"failure", "timed_out", "cancelled", "action_required", "startup_failure"}
# Branches where routines ship their state or reports as PRs. That state moves
# to the automation/routine-status branch (.claude/automation/CONTRACT.md,
# section 5), so these PRs are listed apart and never as ready to merge.
ROUTINE_STATE_BRANCHES = ("claude/qa-rotation-", "chore/v200-planning-report-",
                          "chore/v200-weekly-planning-")


def gh(path, method="GET", data=None):
    """One GitHub REST call through `gh api`; JSON in, JSON out."""
    cmd = ["gh", "api", "-X", method, path]
    if data is not None:
        cmd += ["--input", "-"]
    out = subprocess.run(cmd, input=json.dumps(data) if data is not None else None,
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out) if out.strip() else {}


def pages(get, path, limit=10):
    items = []
    for page in range(1, limit + 1):
        sep = "&" if "?" in path else "?"
        batch = get(f"{path}{sep}per_page=100&page={page}")
        items += batch
        if len(batch) < 100:
            break
    return items


def parse(ts):
    return datetime.fromisoformat(ts.replace("Z", "+00:00"))


def check_state(runs, combined_status, status_count=0):
    """green, red or running, from check runs plus the legacy commit status.

    GitHub reports the combined status as "pending" both when a status is still
    pending and when the commit has no statuses at all, so `status_count` (the
    combined status's `total_count`) tells the two apart."""
    if combined_status == "failure" or combined_status == "error":
        return "red"
    if any(r.get("conclusion") in RED for r in runs):
        return "red"
    if combined_status == "pending" and (status_count or not runs):
        return "running"
    if any(r.get("status") != "completed" for r in runs):
        return "running"
    return "green"


def classify(prs, details, now):
    """Sort open PRs into the issue's sections, oldest first in each."""
    sections = {"needs_you": [], "ready": [], "waiting": [], "unclassified": [], "scoped": [],
                "routine": []}
    for p in sorted(prs, key=lambda p: p["created_at"]):
        labels = {lbl["name"] for lbl in p.get("labels", [])}
        item = {"number": p["number"], "title": p["title"],
                "age": (now - parse(p["created_at"])).days}
        bot = p.get("user", {}).get("type") == "Bot"
        # External first: a fork can name its branch like a routine's.
        if not bot and p.get("author_association") not in INTERNAL:
            sections["needs_you"].append({**item, "why": "external contributor"})
        elif p.get("head", {}).get("ref", "").startswith(ROUTINE_STATE_BRANCHES):
            sections["routine"].append(item)
        elif labels & OWNER_LABELS.keys():
            reason = OWNER_LABELS[sorted(labels & OWNER_LABELS.keys())[0]]
            sections["needs_you"].append({**item, "why": reason})
        elif "path:auto" in labels:
            d = details.get(p["number"], {})
            if p.get("draft"):
                sections["waiting"].append({**item, "why": "draft"})
            elif d.get("mergeable_state") == "dirty":
                sections["waiting"].append({**item, "why": "conflicts"})
            elif d.get("checks") == "red":
                sections["waiting"].append({**item, "why": "CI red"})
            elif d.get("checks") == "green" and d.get("mergeable_state") == "clean":
                sections["ready"].append(item)
            else:
                sections["waiting"].append({**item, "why": "CI running"})
        elif any(lbl.startswith("path:") for lbl in labels):
            sections["scoped"].append(item)
        else:
            sections["unclassified"].append(item)
    return sections


def one_line(text):
    """A title can never start a heading, add HTML (GitHub renders it in issue
    bodies) or repeat the marker inside the body."""
    return html.escape(" ".join(text.split())[:100], quote=False)


def render(sections, merged_7d, now, routines, duplicates=0):
    all_items = [i for s in sections.values() for i in s]
    total = len(all_items)
    ages = [i["age"] for i in all_items]
    median = f"median age {statistics.median(ages):.0f} d" if ages else "no open PRs"
    external = sum(1 for i in sections["needs_you"] if i.get("why") == "external contributor")
    lines = [MARKER,
             f"**{total} open PR{'s' if total != 1 else ''}** · {median} · "
             f"{merged_7d} merged in the last 7 days · {external} external waiting", ""]

    def block(title, items, show_why=True, folded=False):
        rows = [f"- #{i['number']} {one_line(i['title'])} — {i['age']} d"
                + (f", {i['why']}" if show_why and i.get("why") else "") for i in items]
        if folded and items:  # long, lower-priority lists stay one tap away on a phone
            lines.extend([f"<details><summary><b>{title} ({len(items)})</b></summary>", ""]
                         + rows + ["", "</details>", ""])
            return
        lines.append(f"### {title} ({len(items)})")
        lines.extend(rows or ["_None._"])
        lines.append("")

    block("Needs you", sections["needs_you"])
    block("Ready to merge (green, no conflicts, `path:auto`)", sections["ready"], show_why=False)
    lines.insert(len(lines) - 1, "_Ready means green CI and no conflicts. The review is still yours._")
    block("Waiting on a fix", sections["waiting"], folded=True)
    block("Not classified yet (no `path:` label)", sections["unclassified"], show_why=False,
          folded=True)
    block("Routine state, moving to the status branch (do not merge)", sections["routine"],
          show_why=False, folded=True)
    lines.append(f"`path:scoped`, worked by the dispatcher: {len(sections['scoped'])} PRs. "
                 f"Open issues labelled `{DUP_LABEL}`: {duplicates}.")
    lines.append("")
    lines.append("### Routines")
    lines += (["```", routines.rstrip(), "```"] if routines
              else ["_`bin/routine-status` is not on this branch yet (#1506)._"])
    lines += ["", f"<sub>Generated {now:%Y-%m-%d %H:%M} UTC by `scripts/fleet.py`. "
                  "It never merges or closes anything.</sub>"]
    return "\n".join(lines) + "\n"


def ensure_label(get, repo):
    """The lookup below filters by label, so a missing label would mean a new
    issue every day. Create it once instead."""
    try:
        get(f"repos/{repo}/labels/{LABEL}")
    except subprocess.CalledProcessError:
        get(f"repos/{repo}/labels", method="POST",
            data={"name": LABEL, "color": "0e8a16", "description": "The Fleet merge-queue issue"})


def upsert(get, repo, body, dry_run):
    """Edit the marked issue in place, or create it once."""
    if dry_run:
        print(body)
        return None
    ensure_label(get, repo)
    for issue in get(f"repos/{repo}/issues?state=open&labels={LABEL}&per_page=20"):
        if MARKER in (issue.get("body") or ""):
            get(f"repos/{repo}/issues/{issue['number']}", method="PATCH", data={"body": body})
            return issue["number"]
    created = get(f"repos/{repo}/issues", method="POST",
                  data={"title": TITLE, "body": body, "labels": [LABEL]})
    return created.get("number")


def routine_status():
    tool = Path(__file__).resolve().parents[1] / "bin" / "routine-status"
    if not tool.is_file():
        return None
    run = subprocess.run([str(tool), "show"], capture_output=True, text=True)
    return (run.stdout or run.stderr).strip() or None


def triage(repo, dry_run, get=gh, now=None):
    now = now or datetime.now(timezone.utc)
    prs = pages(get, f"repos/{repo}/pulls?state=open")
    details = {}
    for p in prs:
        if "path:auto" not in {lbl["name"] for lbl in p.get("labels", [])} or p.get("draft"):
            continue
        sha = p["head"]["sha"]
        runs = get(f"repos/{repo}/commits/{sha}/check-runs?per_page=100").get("check_runs", [])
        status = get(f"repos/{repo}/commits/{sha}/status")
        details[p["number"]] = {
            "mergeable_state": get(f"repos/{repo}/pulls/{p['number']}").get("mergeable_state"),
            "checks": check_state(runs, status.get("state"), status.get("total_count", 0))}
    since = now - timedelta(days=7)
    closed = get(f"repos/{repo}/pulls?state=closed&sort=updated&direction=desc&per_page=100")
    merged_7d = sum(1 for p in closed if p.get("merged_at") and parse(p["merged_at"]) > since)
    dups = len(pages(get, f"repos/{repo}/issues?state=open&labels={DUP_LABEL}", limit=3))
    body = render(classify(prs, details, now), merged_7d, now, routine_status(), dups)
    return upsert(get, repo, body, dry_run)


def main(argv=None):
    parser = argparse.ArgumentParser(description="Rewrite the pinned Fleet merge-queue issue.")
    parser.add_argument("command", choices=["triage"])
    parser.add_argument("--repo", default="wildcard/caro")
    parser.add_argument("--dry-run", action="store_true", help="print the body; write nothing")
    args = parser.parse_args(argv)
    number = triage(args.repo, args.dry_run)
    if number:
        print(f"fleet: updated issue #{number}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

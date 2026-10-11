#!/usr/bin/env python3
"""Validate a labelled risk-gate gold subset and print the labels-PR report.

Companion to docs/ml/risk-gold-subset.md. The labels PR "changes only the
five labeller fields" and "reports the per-tier counts, the uncertain rate,
and, once the pilot run exists, the judge's tier disagreement against gold".
This script checks the first claim and prints the first two numbers, so the
PR body is produced, not typed.

    python3 scripts/gold-labels-report.py docs/ml/risk-gold-candidates.jsonl

Checks (any failure exits 1):
  * one JSON object per line, ids unique, every id `gold-NNN`
  * the non-labeller fields (`id`, `source`, `eval_id`, `prompt`, `command`)
    are unchanged against --base (default: the same path on `origin/main`,
    read via git; if that is unavailable the run fails unless --no-base is
    given); the labels PR must not edit candidates
  * with --static, every labelled id has exactly one valid static verdict,
    so the disagreement numbers cover all labelled rows
  * every labelled row has a `gold_tier` in {safe, moderate, high, critical},
    a boolean `uncertain`, a non-empty `rationale`, a `labeller` and an ISO
    `labelled_at`
  * no row carries a static field (`static_tier`, `static_patterns`): the
    guide keeps those in risk-gold-static-tiers.jsonl so the labeller stays
    blind to them

Report: rows labelled, per-tier counts against the stratification plan, the
uncertain rate, and the ids still unlabelled. With --static <path> (only
after the labels PR has merged, per the guide) it also joins the static
verdicts and prints the floor-vs-gold disagreement, overall and by gold tier.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

TIERS = ("safe", "moderate", "high", "critical")
PLAN = {"safe": (40, 45), "moderate": (30, 35), "high": (25, 30), "critical": (25, 30)}
FIXED = ("id", "source", "eval_id", "prompt", "command")
LABELLER = ("gold_tier", "uncertain", "rationale", "labeller", "labelled_at")
STATIC = ("static_tier", "static_patterns")
ID_RE = re.compile(r"^gold-\d{3}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def read_jsonl(text: str) -> list[dict]:
    rows = []
    for n, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError as e:
            raise ValueError(f"line {n}: not JSON ({e.msg})") from None
        if not isinstance(obj, dict):
            raise ValueError(f"line {n}: not an object")
        rows.append(obj)
    return rows


def git_show(path: str, ref: str = "origin/main") -> str | None:
    try:
        return subprocess.run(
            ["git", "show", f"{ref}:{path}"], check=True, capture_output=True, text=True
        ).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def validate(rows: list[dict], base: list[dict] | None) -> list[str]:
    errors: list[str] = []
    seen: set[str] = set()
    for r in rows:
        rid = str(r.get("id"))
        if not ID_RE.match(rid):
            errors.append(f"{rid}: id is not gold-NNN")
        if rid in seen:
            errors.append(f"{rid}: duplicate id")
        seen.add(rid)
        for k in FIXED:
            if k not in r:
                errors.append(f"{rid}: missing field {k}")
        for k in STATIC:
            if k in r:
                errors.append(f"{rid}: carries {k}; static verdicts stay in risk-gold-static-tiers.jsonl")
        extra = set(r) - set(FIXED) - set(LABELLER)
        if extra:
            errors.append(f"{rid}: unexpected fields {sorted(extra)}")
        tier = r.get("gold_tier")
        if tier is None:
            if any(r.get(k) not in (None, False, "") for k in LABELLER[1:]):
                errors.append(f"{rid}: labeller fields set but gold_tier is null")
            continue
        if tier not in TIERS:
            errors.append(f"{rid}: gold_tier {tier!r} is not one of {', '.join(TIERS)}")
        if not isinstance(r.get("uncertain"), bool):
            errors.append(f"{rid}: uncertain must be true or false")
        if not (isinstance(r.get("rationale"), str) and r["rationale"].strip()):
            errors.append(f"{rid}: rationale is empty; name the rule that decided")
        if not (isinstance(r.get("labeller"), str) and r["labeller"].strip()):
            errors.append(f"{rid}: labeller is empty")
        if not (isinstance(r.get("labelled_at"), str) and DATE_RE.match(r["labelled_at"])):
            errors.append(f"{rid}: labelled_at is not an ISO date")
    if base is not None:
        by_id = {r["id"]: r for r in base}
        if set(by_id) != seen:
            errors.append(
                f"id set differs from base: +{sorted(seen - set(by_id))} -{sorted(set(by_id) - seen)}"
            )
        for r in rows:
            b = by_id.get(r.get("id"))
            if b is None:
                continue
            for k in FIXED:
                if r.get(k) != b.get(k):
                    errors.append(f"{r['id']}: {k} changed; the labels PR edits only labeller fields")
    return errors


def validate_static(rows: list[dict], static: list[dict]) -> list[str]:
    """Every labelled id must have exactly one valid static verdict, or the
    floor-vs-gold percentage would describe only the matched subset."""
    errors: list[str] = []
    counts = Counter(str(r.get("id")) for r in static)
    tiers = {str(r.get("id")): r.get("static_tier") for r in static}
    for r in rows:
        if r.get("gold_tier") not in TIERS:
            continue
        rid = r["id"]
        if counts.get(rid, 0) != 1:
            errors.append(f"{rid}: {counts.get(rid, 0)} static rows (need exactly 1); static file is stale or partial")
        elif tiers[rid] not in TIERS:
            errors.append(f"{rid}: static_tier {tiers[rid]!r} is not one of {', '.join(TIERS)}")
    return errors


def report(rows: list[dict], static: list[dict] | None) -> str:
    out: list[str] = []
    labelled = [r for r in rows if r.get("gold_tier") in TIERS]
    out.append(f"Rows: {len(rows)} total, {len(labelled)} labelled, {len(rows) - len(labelled)} unlabelled")
    counts = Counter(r["gold_tier"] for r in labelled)
    out.append("")
    out.append("| Gold tier | Rows | Plan | Status |")
    out.append("|---|---|---|---|")
    for t in TIERS:
        lo, hi = PLAN[t]
        n = counts.get(t, 0)
        status = "within plan" if lo <= n <= hi else ("under floor" if n < lo else "over ceiling")
        out.append(f"| `{t}` | {n} | {lo}–{hi} | {status} |")
    if labelled:
        unc = sum(1 for r in labelled if r.get("uncertain") is True)
        out.append("")
        out.append(f"Uncertain (rule 8): {unc} of {len(labelled)} ({100 * unc / len(labelled):.1f}%)")
        for src, n in sorted(Counter(r["source"] for r in labelled).items()):
            out.append(f"  {src}: {n} labelled")
    missing = [r["id"] for r in rows if r.get("gold_tier") not in TIERS]
    if missing:
        out.append("")
        out.append("Unlabelled: " + ", ".join(missing))
    if static is not None and labelled:
        st = {r["id"]: r.get("static_tier") for r in static}
        out.append("")
        out.append("Static floor vs gold (join on id):")
        rank = {t: i for i, t in enumerate(TIERS)}
        total = agree = low = high = 0
        per_tier: dict[str, list[int]] = {t: [0, 0] for t in TIERS}
        for r in labelled:
            s = st.get(r["id"])
            if s not in TIERS:
                continue
            total += 1
            per_tier[r["gold_tier"]][1] += 1
            if s == r["gold_tier"]:
                agree += 1
                per_tier[r["gold_tier"]][0] += 1
            elif rank[s] < rank[r["gold_tier"]]:
                low += 1
            else:
                high += 1
        if total:
            out.append(
                f"  {agree}/{total} agree ({100 * agree / total:.1f}%); floor too low on {low}, too high on {high}"
            )
            for t in TIERS:
                a, n = per_tier[t]
                if n:
                    out.append(f"  gold {t}: floor agrees on {a}/{n}")
    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("labels", help="labelled risk-gold-candidates.jsonl")
    p.add_argument("--base", help="unlabelled candidates to diff against (default: origin/main copy)")
    p.add_argument("--no-base", action="store_true", help="skip the fixed-field comparison")
    p.add_argument("--static", help="risk-gold-static-tiers.jsonl, only after labels are frozen")
    args = p.parse_args(argv)

    try:
        rows = read_jsonl(Path(args.labels).read_text())
    except (OSError, ValueError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    base = None
    if not args.no_base:
        text = Path(args.base).read_text() if args.base else git_show("docs/ml/risk-gold-candidates.jsonl")
        if text is None:
            print(
                "error: cannot read docs/ml/risk-gold-candidates.jsonl from origin/main; "
                "pass --base <unlabelled file> or --no-base to skip the fixed-field check",
                file=sys.stderr,
            )
            return 1
        base = read_jsonl(text)

    static = read_jsonl(Path(args.static).read_text()) if args.static else None

    errors = validate(rows, base)
    if static is not None:
        errors += validate_static(rows, static)
    for e in errors:
        print(f"error: {e}", file=sys.stderr)
    print(report(rows, static))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

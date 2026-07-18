#!/usr/bin/env python3
"""Automation scheduler (scaffold).

Reads ``.claude/automation/config/schedule.yaml``, enumerates the loops that
are enabled, resolves the execution harness for each, and appends a
run-history ledger entry per candidate. This closes the "schedule.yaml is
descriptive, not executable" gap (Warp Oz idea: schedule + run harnesses) and
produces the audit trail that a per-harness effectiveness dashboard consumes.

Ships as a SCAFFOLD, mirroring ``slash-router.yml``'s help/echo/version
staging: live per-loop cron-due matching and agent dispatch land once
``ANTHROPIC_API_KEY`` is provisioned (the dispatch step in
``automation-scheduler.yml`` is gated on that secret). Until then this records
intent as a safe no-op.

Runnable locally for verification::

    LOOP_INPUT=due DRY_RUN=true python3 .github/scripts/automation_scheduler.py

Environment:
    LOOP_INPUT  loop name from schedule.yaml, or "due" for all enabled (default)
    DRY_RUN     "false" to mark entries as dispatch-requested; anything else
                is a no-op enumeration (default "true")
"""

from __future__ import annotations

import datetime as _dt
import os
import sys
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parents[2]
SCHEDULE = REPO / ".claude/automation/config/schedule.yaml"
STAKEHOLDERS = REPO / ".github/STAKEHOLDERS.yml"
LEDGER_DIR = REPO / ".claude/automation/state/run_history"
PACKS = ("technical", "content", "management")


def load_yaml(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


def default_harness() -> str:
    """Default execution harness from STAKEHOLDERS.yml (loop→area harness
    mapping is a follow-up; scaffold uses the repo default)."""
    try:
        data = load_yaml(STAKEHOLDERS)
    except FileNotFoundError:
        return "claude-code"
    return (data.get("default") or {}).get("harness", "claude-code")


def enabled_loops(schedule: dict):
    """Yield ``(name, cfg)`` for every ``enabled: true`` loop across all packs."""
    for pack in PACKS:
        for name, cfg in (schedule.get(pack) or {}).items():
            if isinstance(cfg, dict) and cfg.get("enabled", False):
                yield name, cfg


def main() -> int:
    loop_input = (os.environ.get("LOOP_INPUT") or "due").strip() or "due"
    dry_run = (os.environ.get("DRY_RUN") or "true").strip().lower() != "false"
    now = _dt.datetime.now(_dt.timezone.utc)

    schedule = load_yaml(SCHEDULE)
    harness = default_harness()

    candidates = list(enabled_loops(schedule))
    if loop_input != "due":
        candidates = [(n, c) for n, c in candidates if n == loop_input]
        if not candidates:
            print(f"::warning::no enabled loop named '{loop_input}' in {SCHEDULE.name}")
            return 0

    note = (
        "scheduler scaffold: cron-due matching + dispatch pending ANTHROPIC_API_KEY"
        if dry_run
        else "dispatch requested (secret-gated in workflow)"
    )
    entries = [
        {
            "loop": name,
            "harness": harness,
            "schedule": cfg.get("schedule"),
            "started": now.isoformat(),
            "outcome": "skipped",
            "note": note,
        }
        for name, cfg in candidates
    ]

    LEDGER_DIR.mkdir(parents=True, exist_ok=True)
    ledger = LEDGER_DIR / f"{now:%Y-%m-%d}.yaml"
    with ledger.open("a", encoding="utf-8") as fh:
        yaml.safe_dump(entries, fh, sort_keys=False, default_flow_style=False)

    print(f"Enumerated {len(entries)} enabled loop(s); ledger: {ledger.relative_to(REPO)}")
    for e in entries:
        print(f"  - {e['loop']:<24} harness={e['harness']:<11} {e['outcome']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

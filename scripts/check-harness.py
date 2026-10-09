#!/usr/bin/env python3
"""Deterministic lint for Caro's Claude Code harness (CLAUDE.md + .claude/).

Adapted from tigerless-labs/autoharness (MIT). There, a model may only
*propose* skill changes; a deterministic promoter lints each proposal against
one spec before anything lands, and tests pin the agents' contracts
(least-privilege tools, hooks wired to files that exist). Caro's harness is
hand-written by many parallel sessions, so the same checks run in CI instead.
Agent tool allowlists are left to #1534's Rust contract test
(tests/agent_tools_contract.rs), so this script does not duplicate it.
Rationale: docs/adr/ADR-018-autoharness-harness-hygiene.md

Levels:
  error   fails the run: a broken registry entry, or drift in a file that is
          loaded into every session (CLAUDE.md, .claude/rules/).
  warn    legacy drift in on-demand files (skills, agents, commands). The count
          must equal --max-warnings, a ratchet: the PR that fixes a warning
          lowers the budget, so the slack can never be spent again.
  notice  informational (overdue deprecations, context budget); never fails,
          so the calendar alone can never turn CI red.

Stdlib only. Usage: python3 scripts/check-harness.py [--max-warnings N]
"""
import argparse
import json
import os
import re
import sys
from collections import namedtuple
from datetime import date
from pathlib import Path

# Agent Skills format maximum for `description`; autoharness enforces the same cap.
SKILL_DESC_MAX = 1024
# Skills whose descriptions share this many word trigrams compete for the same
# requests (threshold from #1196's bin/verify-skills, which found two pairs).
COLLISION_TRIGRAMS = 5
BUDGET_FILE = Path("scripts") / "harness-budget.json"
# Directories under .claude/skills/ that are support material for a command,
# not standalone skills. Claude Code ignores a skill dir without SKILL.md.
SUPPORT_DIRS = {
    "code-parts-syncer": "modules loaded by /caro.sync (.claude/commands/caro.sync.md)",
}

FRONTMATTER = re.compile(r"\A---\r?\n(.*?)\r?\n---[ \t]*(?:\r?\n|\Z)", re.S)
TOP_KEY = re.compile(r"^([A-Za-z0-9_-]+):(.*)$")
# Link target may contain balanced parentheses: [x](./notes(1).md)
MD_LINK = re.compile(r"\]\(\s*<?([^()\s>]*(?:\([^)]*\)[^()\s>]*)*)>?(?:\s+\"[^\"]*\")?\s*\)")
TICK_PATH = re.compile(
    r"`((?:\.claude|\.github|\.hermes|docs|scripts|bin|src|tests|website|playbook)/[^`\s]+)`"
)
EXTERNAL = re.compile(r"^(?:[A-Za-z][A-Za-z0-9+.-]*:|#|/|~)")  # URLs, anchors, site routes, $HOME
PLACEHOLDER = re.compile(r"[*<>{}$]|YYYY|X\.Y\.Z|\bvX\b|NNN|XXX")
FILE_SHAPED = re.compile(r"\.[A-Za-z0-9]{1,6}$")  # bare dir refs are usually shorthand
REMOVAL_DATE = re.compile(r"(?i)\bremoved after (\d{4}-\d{2}-\d{2})")
PROJECT_DIR_VAR = re.compile(r'^"?\$\{?CLAUDE_PROJECT_DIR\}?"?/')

Finding = namedtuple("Finding", "level check path line message")
LEVELS = ("error", "warn", "notice")


def read(path):
    return path.read_text(encoding="utf-8", errors="replace")


def frontmatter(text):
    """Top-level `key: value` pairs of a YAML frontmatter block, or None.

    Deliberately tiny (no PyYAML): continuation lines and block scalars fold
    into the previous key, which is all these files use."""
    m = FRONTMATTER.match(text)
    if not m:
        return None
    out, key = {}, None
    for line in m.group(1).splitlines():
        top = TOP_KEY.match(line)
        if top:
            key = top.group(1)
            out[key] = top.group(2).strip()
        elif key is not None and line.strip():
            out[key] = (out[key] + " " + line.strip()).strip()
    return {k: _scalar(v) for k, v in out.items()}


def _scalar(value):
    if value[:1] in ("|", ">"):
        value = value[1:].lstrip("+-0123456789").strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
        value = value[1:-1]
    return value


def rel(root, path):
    return path.relative_to(root).as_posix()


def check_skills(root):
    skills = root / ".claude" / "skills"
    if not skills.is_dir():
        return []
    out = []
    for d in sorted(p for p in skills.iterdir() if p.is_dir() and not p.name.startswith(".")):
        where = rel(root, d)
        skill_md = d / "SKILL.md"
        if not skill_md.is_file():
            if d.name not in SUPPORT_DIRS:
                out.append(Finding("error", "skill-structure", where, None,
                                   "no SKILL.md, so Claude Code silently ignores this directory"))
            continue
        fm = frontmatter(read(skill_md))
        where = rel(root, skill_md)
        if fm is None:
            out.append(Finding("error", "skill-structure", where, None, "missing frontmatter"))
            continue
        desc = fm.get("description", "")
        if not desc:
            out.append(Finding("error", "skill-description", where, None,
                               "empty description (it is the only text recall matches on)"))
        elif len(desc) > SKILL_DESC_MAX:
            out.append(Finding("error", "skill-description", where, None,
                               f"description is {len(desc)} chars (max {SKILL_DESC_MAX})"))
        # name == directory keeps names unique, so a copied skill cannot shadow another.
        if fm.get("name") and fm["name"] != d.name:
            out.append(Finding("error", "skill-structure", where, None,
                               f"frontmatter name {fm['name']!r} != directory {d.name!r}"))
    return out


def check_agents(root):
    out = []
    for f in sorted((root / ".claude" / "agents").glob("*.md")):
        where = rel(root, f)
        fm = frontmatter(read(f))
        if fm is None:
            out.append(Finding("error", "agent-structure", where, None, "missing frontmatter"))
            continue
        # `name` is required, and name == filename keeps names unique, so a
        # copied agent file cannot shadow another agent.
        if fm.get("name") != f.stem:
            out.append(Finding("error", "agent-structure", where, None,
                               f"frontmatter name {fm.get('name')!r} != filename {f.stem!r}"))
        if not fm.get("description"):
            out.append(Finding("error", "agent-structure", where, None, "empty description"))
    return out


def check_rules_indexed(root):
    rules = root / ".claude" / "rules"
    constitution = rules / "constitution.md"
    if not constitution.is_file():
        return []
    index = read(constitution)
    out = []
    for f in sorted(rules.glob("*.md")):
        if f.name == constitution.name:
            continue
        if not re.search(r"\]\(\s*(?:\./)?" + re.escape(f.name) + r"\s*\)", index):
            out.append(Finding("error", "rules-indexed", rel(root, f), None,
                               "not indexed in .claude/rules/constitution.md, "
                               "so it has no precedence under conflict"))
    return out


def check_hooks(root):
    settings = root / ".claude" / "settings.json"
    if not settings.is_file():
        return []
    where = rel(root, settings)

    def bad(message):
        return Finding("error", "hooks-wired", where, None, message)

    try:
        data = json.loads(read(settings))
    except ValueError as exc:
        return [bad(f"invalid JSON: {exc}")]
    hooks = (data.get("hooks") or {}) if isinstance(data, dict) else None
    if not isinstance(hooks, dict):
        return [bad("expected an object whose `hooks` maps events to hook groups")]
    out = []
    for event, groups in hooks.items():
        if not isinstance(groups, list) or not all(isinstance(g, dict) for g in groups):
            out.append(bad(f"{event}: expected a list of hook groups"))
            continue
        for group in groups:
            entries = group.get("hooks") or []
            if not isinstance(entries, list) or not all(isinstance(h, dict) for h in entries):
                out.append(bad(f"{event}: a group's `hooks` must be a list of objects"))
                continue
            for hook in entries:
                out += check_hook_command(root, bad, event, str(hook.get("command", "")))
    return out


def check_hook_command(root, bad, event, command):
    """Every repo script a hook command names must exist. Only the one it runs
    directly (the first token) needs the exec bit: `bash ./x.sh` does not."""
    out = []
    for i, token in enumerate(command.split()):
        script = PROJECT_DIR_VAR.sub("", token)
        if not script.startswith(("./", ".claude/")):
            continue  # an interpreter, a flag, or a binary on PATH
        path = root / script
        if not path.is_file():
            out.append(bad(f"{event} hook runs {script}, which does not exist"))
        elif i == 0 and not os.access(path, os.X_OK):
            out.append(bad(f"{event} hook runs {script}, which is not executable"))
    return out


def references(text):
    """(line, target) for local file references outside fenced code blocks."""
    fenced = False
    for n, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
            continue
        if fenced:
            continue
        for target in MD_LINK.findall(line):
            target = re.split(r"[#?]", target, maxsplit=1)[0]
            if target and not EXTERNAL.match(target) and not PLACEHOLDER.search(target):
                yield n, target
        for target in TICK_PATH.findall(line):
            target = target.rstrip(".,:;")
            if FILE_SHAPED.search(target) and not PLACEHOLDER.search(target):
                yield n, target


def check_references(root, files, level):
    out = []
    for f in files:
        for n, target in references(read(f)):
            # Agents read paths as repo-root-relative even where a renderer would not.
            if not ((f.parent / target).exists() or (root / target).exists()):
                out.append(Finding(level, "dangling-ref", rel(root, f), n, f"{target} does not exist"))
    return out


def scoped(rule):
    """A rule with `paths:` frontmatter loads only when Claude reads a matching file."""
    return "paths" in (frontmatter(read(rule)) or {})


def always_loaded(root):
    files = [root / "CLAUDE.md", root / ".claude" / "CLAUDE.md"]
    files += [f for f in sorted((root / ".claude" / "rules").glob("*.md")) if not scoped(f)]
    return [f for f in files if f.is_file()]


def on_demand(root):
    c = root / ".claude"
    files = [c / "AGENTS.md"] if (c / "AGENTS.md").is_file() else []
    files += [f for f in sorted(c.glob("rules/*.md")) if scoped(f)]
    return files + sorted(c.glob("skills/*/SKILL.md")) + sorted(c.glob("agents/*.md")) \
        + sorted(c.glob("commands/*.md"))


def check_deprecations(root, today):
    marketplace = root / ".claude-plugin" / "marketplace.json"
    published = read(marketplace) if marketplace.is_file() else ""
    out = []
    for f in on_demand(root):
        m = REMOVAL_DATE.search((frontmatter(read(f)) or {}).get("description", ""))
        try:
            if not m or date.fromisoformat(m.group(1)) >= today:
                continue
        except ValueError:  # e.g. 2026-13-45: not a date, so not a promise we can check
            continue
        name = f.parent.name if f.name == "SKILL.md" else f.stem
        tail = " and is still published in .claude-plugin/marketplace.json" \
            if f'"{name}"' in published else ""
        out.append(Finding("notice", "overdue-removal", rel(root, f), None,
                           f"its deprecation notice promised removal after {m.group(1)}{tail}"))
    return out


def trigrams(text):
    words = re.findall(r"[a-z0-9]+", text.lower())
    return {tuple(words[i:i + 3]) for i in range(len(words) - 2)}


def check_collisions(root):
    """Skills whose descriptions overlap heavily: the model routes on the
    description, so near-twins compete for the same requests. A notice, because
    merging or sharpening them is a judgment call."""
    grams = {}
    for f in sorted((root / ".claude" / "skills").glob("*/SKILL.md")):
        desc = (frontmatter(read(f)) or {}).get("description", "")
        if desc:
            grams[f.parent.name] = trigrams(desc)
    names = sorted(grams)
    out = []
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            shared = len(grams[a] & grams[b])
            if shared >= COLLISION_TRIGRAMS:
                out.append(Finding("notice", "description-collision", f".claude/skills/{a}", None,
                                   f"description shares {shared} word trigrams with {b}; "
                                   "merge them or sharpen their triggers"))
    return out


def context_budget(root):
    """Observation only (autoharness keeps metrics out of decisions too): how much
    description text the harness lists in every session's context."""
    c = root / ".claude"
    kinds = (("agents", sorted(c.glob("agents/*.md"))),
             ("skills", sorted(c.glob("skills/*/SKILL.md"))),
             ("commands", sorted(c.glob("commands/*.md"))))
    parts, agent_sizes = [], []
    for kind, files in kinds:
        sizes = [(len((frontmatter(read(f)) or {}).get("description", "")), f) for f in files]
        total = sum(n for n, _ in sizes)
        parts.append(f"{kind} {len(files)} ({total:,} chars)")
        if kind == "agents":
            agent_sizes = sorted(sizes, key=lambda s: -s[0])
    longest = ", ".join(f"{f.stem} {n:,}" for n, f in agent_sizes[:5])
    msg = "descriptions listed every session: " + "; ".join(parts)
    return [Finding("notice", "context-budget", ".claude", None,
                    msg + (f". Longest agent descriptions: {longest}" if longest else ""))]


def run_checks(root, today):
    return (check_skills(root) + check_agents(root) + check_rules_indexed(root)
            + check_hooks(root)
            + check_references(root, always_loaded(root), "error")
            + check_references(root, on_demand(root), "warn")
            + check_deprecations(root, today) + check_collisions(root) + context_budget(root))


def emit(f, github):
    where = f.path + (f":{f.line}" if f.line else "")
    print(f"{f.level:<6} [{f.check}] {where}: {f.message}")
    if github:
        loc = f"file={f.path}" + (f",line={f.line}" if f.line else "")
        kind = {"warn": "warning"}.get(f.level, f.level)
        print(f"::{kind} {loc}::[{f.check}] {f.message}")


def main(argv=None):
    parser = argparse.ArgumentParser(description="Lint Caro's Claude Code harness.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--max-warnings", type=int, default=None,
                        help=f"fail unless warnings equal N (default: max_warnings in {BUDGET_FILE})")
    parser.add_argument("--today", type=date.fromisoformat, default=date.today(),
                        help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    root = args.root.resolve()
    budget_file = root / BUDGET_FILE
    if args.max_warnings is None and budget_file.is_file():
        args.max_warnings = json.loads(read(budget_file))["max_warnings"]
    findings = sorted(run_checks(root, args.today),
                      key=lambda f: (LEVELS.index(f.level), f.path, f.line or 0))
    github = os.environ.get("GITHUB_ACTIONS") == "true"
    for f in findings:
        emit(f, github)
    errors = sum(f.level == "error" for f in findings)
    warnings = sum(f.level == "warn" for f in findings)
    budget = "" if args.max_warnings is None else f" (budget {args.max_warnings})"
    print(f"\nharness lint: {errors} error(s), {warnings} warning(s){budget}")
    if errors:
        return 1
    if args.max_warnings is not None:
        if warnings > args.max_warnings:
            print("New drift: warnings rose above the budget. Fix the new one (or any listed above).")
            return 1
        if warnings < args.max_warnings:
            # Unspent slack would let a later PR add drift for free, so the gain
            # must be locked in by the PR that made it.
            print(f"Drift went down: lower max_warnings to {warnings} in "
                  f"{BUDGET_FILE.as_posix()} in this PR to lock in the gain.")
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Unit tests for scripts/check-harness.py (the .claude/ harness linter).

Each test builds a throwaway repo tree, so the checks are pinned independently
of the live harness; CI runs the linter itself against the real repo.
"""

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from datetime import date
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "check_harness", Path(__file__).resolve().parents[1] / "check-harness.py")
ch = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(ch)

TODAY = date(2026, 9, 26)
SKILL = "---\nname: {name}\ndescription: Use when {name} is needed.\n---\n# {name}\n"
AGENT = "---\nname: {name}\ndescription: Use for {name} work.\n{extra}---\n{body}\n"
CONSTITUTION = "# Constitution\n1. **[git-workflow.md](./git-workflow.md)** — branches.\n"


class HarnessFixture(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.write("CLAUDE.md", "# Project\nSee `.claude/rules/git-workflow.md`.\n")
        self.write(".claude/rules/constitution.md", CONSTITUTION)
        self.write(".claude/rules/git-workflow.md", "# Git workflow\n")
        self.write(".claude/skills/demo/SKILL.md", SKILL.format(name="demo"))
        self.write(".claude/agents/helper.md", AGENT.format(name="helper", extra="", body="Help."))

    def tearDown(self):
        self._tmp.cleanup()

    def write(self, rel, text, executable=False):
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        if executable:
            path.chmod(0o755)
        return path

    def findings(self, check=None):
        found = [f for f in ch.run_checks(self.root, TODAY) if f.check != "context-budget"]
        return [f for f in found if check is None or f.check == check]

    def run_main(self, *args):
        with contextlib.redirect_stdout(io.StringIO()) as out:
            code = ch.main(["--root", str(self.root), "--today", TODAY.isoformat(), *args])
        return code, out.getvalue()


class TestCleanTree(HarnessFixture):
    def test_minimal_harness_has_no_findings(self):
        self.assertEqual(self.findings(), [])


class TestSkills(HarnessFixture):
    def test_dir_without_skill_md_is_error(self):
        # Claude Code silently ignores it; nobody notices the skill never loads.
        self.write(".claude/skills/ghost/README.md", "# not a skill\n")
        [f] = self.findings("skill-structure")
        self.assertEqual((f.level, f.path), ("error", ".claude/skills/ghost"))

    def test_declared_support_dir_is_exempt(self):
        self.write(".claude/skills/code-parts-syncer/README.md", "# modules for /caro.sync\n")
        self.assertEqual(self.findings("skill-structure"), [])

    def test_hidden_dirs_are_skipped(self):
        self.write(".claude/skills/.archive/old/README.md", "# archived\n")
        self.assertEqual(self.findings(), [])

    def describe(self, length):
        self.write(".claude/skills/demo/SKILL.md", f"---\nname: demo\ndescription: {'x' * length}\n---\n")

    def test_description_at_budget_passes(self):
        self.describe(ch.SKILL_DESC_MAX)
        self.assertEqual(self.findings("skill-description"), [])

    def test_description_one_over_budget_is_error(self):
        self.describe(ch.SKILL_DESC_MAX + 1)
        [f] = self.findings("skill-description")
        self.assertIn(f"max {ch.SKILL_DESC_MAX}", f.message)

    def test_empty_description_is_error(self):
        self.write(".claude/skills/demo/SKILL.md", "---\nname: demo\n---\n# demo\n")
        self.assertEqual(len(self.findings("skill-description")), 1)

    def test_description_under_minimum_is_warning(self):
        # A one-word description is a no-op that nothing routes to (#1196).
        self.describe(ch.SKILL_DESC_MIN - 1)
        [f] = self.findings("skill-description")
        self.assertEqual(f.level, "warn")
        self.assertIn(f"min {ch.SKILL_DESC_MIN}", f.message)

    def test_description_at_minimum_passes(self):
        self.describe(ch.SKILL_DESC_MIN)
        self.assertEqual(self.findings("skill-description"), [])

    def test_name_must_match_directory(self):
        self.write(".claude/skills/demo/SKILL.md", SKILL.format(name="copied-from-elsewhere"))
        [f] = self.findings("skill-structure")
        self.assertIn("!= directory", f.message)

    def test_name_is_optional(self):
        # The host defaults it to the directory name (create_handoff relies on this).
        self.write(".claude/skills/demo/SKILL.md", "---\ndescription: Use when demoing the feature.\n---\n")
        self.assertEqual(self.findings(), [])


class TestCommands(HarnessFixture):
    def test_empty_description_is_error(self):
        self.write(".claude/commands/go.md", "---\ndescription:\n---\n# go\n")
        [f] = self.findings("command-description")
        self.assertEqual((f.level, f.path), ("error", ".claude/commands/go.md"))

    def test_frontmatter_is_optional(self):
        self.write(".claude/commands/go.md", "# go\nRun the thing.\n")
        self.assertEqual(self.findings(), [])


class TestAgents(HarnessFixture):
    # Tool allowlists are left to tests/agent_tools_contract.rs (#1534);
    # this linter keeps only registry integrity for agents.

    def test_name_must_match_filename(self):
        self.write(".claude/agents/helper.md", AGENT.format(name="other", extra="", body="Help."))
        [f] = self.findings("agent-structure")
        self.assertIn("!= filename", f.message)

    def test_tools_are_not_this_linters_job(self):
        self.write(".claude/agents/critic.md",
                   AGENT.format(name="critic", extra="", body="You are read-only."))
        self.assertEqual(self.findings(), [])


class TestRulesIndexed(HarnessFixture):
    def test_unindexed_rule_is_error(self):
        # design-dialogue-protocol.md sat outside the precedence index this way.
        self.write(".claude/rules/new-rule.md", "# New rule\n")
        [f] = self.findings("rules-indexed")
        self.assertEqual(f.path, ".claude/rules/new-rule.md")

    def test_indexed_rule_passes(self):
        self.write(".claude/rules/new-rule.md", "# New rule\n")
        self.write(".claude/rules/constitution.md",
                   CONSTITUTION + "2. **[new-rule.md](./new-rule.md)** — new.\n")
        self.assertEqual(self.findings("rules-indexed"), [])


class TestHooks(HarnessFixture):
    SETTINGS = '{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "%s"}]}]}}'

    def test_missing_hook_script_is_error(self):
        self.write(".claude/settings.json", self.SETTINGS % "./.claude/hooks/gone.sh")
        [f] = self.findings("hooks-wired")
        self.assertIn("does not exist", f.message)

    @unittest.skipIf(os.name == "nt", "Windows has no exec bit; os.access(X_OK) is always true")
    def test_non_executable_hook_script_is_error(self):
        self.write(".claude/hooks/guard.sh", "#!/bin/sh\n")
        self.write(".claude/settings.json", self.SETTINGS % "./.claude/hooks/guard.sh")
        [f] = self.findings("hooks-wired")
        self.assertIn("not executable", f.message)

    def test_script_behind_an_interpreter_is_still_checked(self):
        self.write(".claude/settings.json", self.SETTINGS % "bash ./.claude/hooks/gone.sh")
        [f] = self.findings("hooks-wired")
        self.assertIn("gone.sh, which does not exist", f.message)

    def test_script_behind_an_interpreter_needs_no_exec_bit(self):
        self.write(".claude/hooks/guard.sh", "#!/bin/sh\n")
        self.write(".claude/settings.json", self.SETTINGS % "bash ./.claude/hooks/guard.sh")
        self.assertEqual(self.findings("hooks-wired"), [])

    def test_malformed_settings_are_reported_not_raised(self):
        for settings, expect in (("[]", "expected an object"),
                                 ('{"hooks": {"Stop": {"hooks": []}}}', "list of hook groups"),
                                 ('{"hooks": {"Stop": [{"hooks": "x.sh"}]}}', "list of objects")):
            self.write(".claude/settings.json", settings)
            [f] = self.findings("hooks-wired")
            self.assertIn(expect, f.message)

    def test_wired_hook_passes_with_project_dir_prefix(self):
        self.write(".claude/hooks/guard.sh", "#!/bin/sh\n", executable=True)
        self.write(".claude/settings.json",
                   self.SETTINGS % '\\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/guard.sh --flag')
        self.assertEqual(self.findings("hooks-wired"), [])

    def test_binaries_on_path_are_not_checked(self):
        self.write(".claude/settings.json", self.SETTINGS % "python3 -m some.module")
        self.assertEqual(self.findings("hooks-wired"), [])

    def test_whole_path_quoted_project_dir_resolves(self):
        # The closing quote used to stay on the token: a false "does not exist".
        self.write(".claude/hooks/guard.sh", "#!/bin/sh\n", executable=True)
        self.write(".claude/settings.json",
                   self.SETTINGS % '\\"${CLAUDE_PROJECT_DIR}/.claude/hooks/guard.sh\\"')
        self.assertEqual(self.findings("hooks-wired"), [])

    def test_quoted_relative_path_is_still_checked(self):
        # A token that started with a quote used to be skipped silently.
        self.write(".claude/settings.json", self.SETTINGS % '\\"./.claude/hooks/gone.sh\\"')
        [f] = self.findings("hooks-wired")
        self.assertIn("gone.sh, which does not exist", f.message)

    def test_existing_directory_operand_is_not_missing(self):
        self.write(".claude/settings.json", self.SETTINGS % "test -d ./.claude")
        self.assertEqual(self.findings("hooks-wired"), [])


class TestReferences(HarnessFixture):
    def test_dangling_ref_in_always_loaded_file_is_error(self):
        # CLAUDE.md pointed every session at a .claude/memory file that did not exist.
        self.write("CLAUDE.md", "Check `.claude/memory/current-tasks.md` first.\n")
        [f] = self.findings("dangling-ref")
        self.assertEqual((f.level, f.path, f.line), ("error", "CLAUDE.md", 1))

    def test_dangling_ref_in_on_demand_file_is_warning(self):
        self.write(".claude/skills/demo/SKILL.md", SKILL.format(name="demo") + "Run `scripts/gone.sh`.\n")
        [f] = self.findings("dangling-ref")
        self.assertEqual(f.level, "warn")

    def test_refs_resolve_relative_to_file_or_repo_root(self):
        self.write("docs/guide.md", "# Guide\n")
        self.write(".claude/rules/git-workflow.md",
                   "[file-relative](../../docs/guide.md) and [root-relative](docs/guide.md)\n")
        self.assertEqual(self.findings("dangling-ref"), [])

    def test_link_target_with_parentheses_resolves(self):
        self.write("docs/notes(1).md", "# notes\n")
        self.write("CLAUDE.md", "See [notes](docs/notes(1).md).\n")
        self.assertEqual(self.findings("dangling-ref"), [])

    def test_wrong_relative_depth_is_caught(self):
        # `../.claude/...` from .claude/commands/ resolves to .claude/.claude/...
        self.write(".claude/automation/specs/SPEC.md", "# spec\n")
        self.write(".claude/commands/loop.md", "See [spec](../.claude/automation/specs/SPEC.md).\n")
        [f] = self.findings("dangling-ref")
        self.assertEqual(f.path, ".claude/commands/loop.md")

    def test_ignores_urls_anchors_routes_fences_placeholders_and_dir_shorthand(self):
        self.write("CLAUDE.md", "\n".join([
            "[site](https://caro.sh/x.md) [top](#usage) [faq](/faq) [mail](mailto:a@b.c)",
            "Template: `.claude/releases/NOTES-vX.md`, `docs/demos/YYYY-MM-DD.md`, `src/<mod>.rs`",
            "Shorthand: `docs/adr/ADR-016` and `src/data`",
            "```bash",
            "cat .claude/nowhere.md && echo `scripts/nowhere.sh`",
            "```",
        ]) + "\n")
        self.assertEqual(self.findings("dangling-ref"), [])


class TestScopedRules(HarnessFixture):
    SCOPED = '---\npaths:\n  - ".claude/**"\n---\n# Harness\nSee `docs/gone.md`.\n'

    def index(self, name):
        self.write(".claude/rules/constitution.md",
                   CONSTITUTION + f"2. **[{name}](./{name})** — scoped.\n")

    def test_rule_with_paths_is_on_demand(self):
        # A `paths:` rule loads only when matching files are read, so its drift
        # costs only those sessions: a warning, like skills, not an error.
        self.write(".claude/rules/harness.md", self.SCOPED)
        self.index("harness.md")
        [f] = self.findings("dangling-ref")
        self.assertEqual((f.level, f.path), ("warn", ".claude/rules/harness.md"))

    def test_scoped_rule_must_still_be_indexed(self):
        self.write(".claude/rules/harness.md", self.SCOPED.replace("`docs/gone.md`", "nothing"))
        [f] = self.findings("rules-indexed")
        self.assertEqual(f.path, ".claude/rules/harness.md")


class TestCollisions(HarnessFixture):
    SAME = "the quick brown fox jumps over the lazy dog every single day"

    def skill(self, name, desc):
        self.write(f".claude/skills/{name}/SKILL.md", f"---\nname: {name}\ndescription: {desc}\n---\n")

    def test_near_twin_descriptions_are_a_notice(self):
        # spec-kitty-git-workflow and spec-kitty-mission-system share 15 trigrams (#1196).
        self.skill("alpha", self.SAME)
        self.skill("beta", self.SAME)
        [f] = self.findings("description-collision")
        self.assertEqual((f.level, f.path), ("notice", ".claude/skills/alpha"))
        self.assertIn("beta", f.message)

    def test_distinct_descriptions_do_not_collide(self):
        self.skill("alpha", "bake bread early every morning for the shop")
        self.skill("beta", "compile quarterly reports for the board review")
        self.assertEqual(self.findings("description-collision"), [])

    def test_collisions_never_fail_the_run(self):
        self.skill("alpha", self.SAME)
        self.skill("beta", self.SAME)
        self.assertEqual(self.run_main("--max-warnings", "0")[0], 0)


class TestDeprecations(HarnessFixture):
    DEPRECATED = "---\nname: old\ndescription: \"DEPRECATED — use demo. Will be removed after {day}.\"\n---\n"

    def test_overdue_removal_is_a_notice_never_a_failure(self):
        self.write(".claude/skills/old/SKILL.md", self.DEPRECATED.format(day="2026-08-01"))
        self.write(".claude-plugin/marketplace.json", '{"skills": [{"name": "old"}]}')
        [f] = self.findings("overdue-removal")
        self.assertEqual(f.level, "notice")
        self.assertIn("still published", f.message)
        self.assertEqual(self.run_main("--max-warnings", "0")[0], 0)  # the calendar never fails CI

    def test_future_removal_is_silent(self):
        self.write(".claude/skills/old/SKILL.md", self.DEPRECATED.format(day="2027-01-01"))
        self.assertEqual(self.findings("overdue-removal"), [])

    def test_impossible_date_does_not_crash_the_run(self):
        self.write(".claude/skills/old/SKILL.md", self.DEPRECATED.format(day="2026-13-45"))
        self.assertEqual(self.findings("overdue-removal"), [])


class TestFrontmatter(unittest.TestCase):
    def test_quotes_block_scalars_and_continuations(self):
        fm = ch.frontmatter("---\nname: \"x\"\ndescription: >-\n  folded\n  text\ntools: Read,\n  Grep\n---\nbody")
        self.assertEqual(fm, {"name": "x", "description": "folded text", "tools": "Read, Grep"})

    def test_missing_frontmatter_is_none(self):
        self.assertIsNone(ch.frontmatter("# no frontmatter\n"))



class TestWarningRatchet(HarnessFixture):
    def setUp(self):
        super().setUp()
        self.write(".claude/agents/helper.md",
                   AGENT.format(name="helper", extra="", body="See `scripts/gone.sh`."))

    def test_warnings_matching_budget_pass(self):
        self.assertEqual(self.run_main("--max-warnings", "1")[0], 0)

    def test_warnings_above_budget_fail(self):
        code, out = self.run_main("--max-warnings", "0")
        self.assertEqual(code, 1)
        self.assertIn("rose above the budget", out)

    def test_budget_slack_fails_until_lowered(self):
        # Unspent slack would let a later PR add drift for free.
        code, out = self.run_main("--max-warnings", "3")
        self.assertEqual(code, 1)
        self.assertIn("lower max_warnings to 1", out)

    def test_budget_file_is_the_default(self):
        # The number lives in scripts/harness-budget.json, so lowering it never
        # edits a Tier-1 workflow file.
        self.write("scripts/harness-budget.json", '{"max_warnings": 0}')
        code, out = self.run_main()
        self.assertEqual(code, 1)
        self.assertIn("(budget 0)", out)
        self.write("scripts/harness-budget.json", '{"max_warnings": 1}')
        self.assertEqual(self.run_main()[0], 0)

    def test_budget_slack_names_the_budget_file(self):
        code, out = self.run_main("--max-warnings", "3")
        self.assertIn("harness-budget.json", out)

    def test_any_error_fails_regardless_of_budget(self):
        self.write(".claude/skills/ghost/README.md", "#\n")
        self.assertEqual(self.run_main("--max-warnings", "99")[0], 1)


if __name__ == "__main__":
    unittest.main()

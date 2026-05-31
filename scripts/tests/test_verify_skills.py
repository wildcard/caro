#!/usr/bin/env python3
"""Unit tests for bin/verify-skills.

Run:
    python3 scripts/tests/test_verify_skills.py
"""

from __future__ import annotations

import importlib.util
import tempfile
import textwrap
import unittest
from pathlib import Path

_THIS = Path(__file__).resolve()
_REPO = _THIS.parent.parent.parent
_HELPER = _REPO / "bin" / "verify-skills"


def _load_module():
    import sys as _sys
    from importlib.machinery import SourceFileLoader

    loader = SourceFileLoader("verify_skills", str(_HELPER))
    spec = importlib.util.spec_from_loader("verify_skills", loader)
    assert spec is not None
    mod = importlib.util.module_from_spec(spec)
    _sys.modules["verify_skills"] = mod
    loader.exec_module(mod)
    return mod


vs = _load_module()


def _make_repo(root: Path, skills: dict[str, str | None], commands: dict[str, str] | None = None) -> None:
    """Materialize a fake .claude/skills + .claude/commands tree under root.

    skills: dict mapping skill-dir-name -> SKILL.md content (or None to skip the file).
    commands: dict mapping command-file-stem -> file content.
    """
    skills_dir = root / ".claude" / "skills"
    skills_dir.mkdir(parents=True)
    for name, content in skills.items():
        d = skills_dir / name
        d.mkdir()
        if content is not None:
            (d / "SKILL.md").write_text(content, encoding="utf-8")
    if commands:
        commands_dir = root / ".claude" / "commands"
        commands_dir.mkdir(parents=True)
        for stem, content in commands.items():
            (commands_dir / f"{stem}.md").write_text(content, encoding="utf-8")


class FrontmatterParserTests(unittest.TestCase):
    def test_flat_scalars(self) -> None:
        text = "---\nname: foo\ndescription: bar baz\n---\nbody"
        fm = vs._parse_frontmatter(text)
        self.assertEqual(fm, {"name": "foo", "description": "bar baz"})

    def test_quoted_scalars_are_unwrapped(self) -> None:
        text = "---\nname: \"foo\"\ndescription: 'baz'\n---\n"
        fm = vs._parse_frontmatter(text)
        self.assertEqual(fm, {"name": "foo", "description": "baz"})

    def test_folded_block_scalar(self) -> None:
        text = textwrap.dedent(
            """\
            ---
            name: foo
            description: >
              first line
              second line
            ---
            body
            """
        )
        fm = vs._parse_frontmatter(text)
        assert fm is not None
        self.assertEqual(fm["name"], "foo")
        self.assertIn("first line", fm["description"])
        self.assertIn("second line", fm["description"])

    def test_chomped_block_scalar(self) -> None:
        # `>-` is folded + strip-chomp; should be treated as a block scalar
        text = textwrap.dedent(
            """\
            ---
            name: foo
            description: >-
              alpha beta gamma
              delta epsilon
            ---
            """
        )
        fm = vs._parse_frontmatter(text)
        assert fm is not None
        self.assertIn("alpha beta gamma", fm["description"])

    def test_literal_block_scalar(self) -> None:
        text = textwrap.dedent(
            """\
            ---
            name: foo
            description: |
              keep newlines please
            ---
            """
        )
        fm = vs._parse_frontmatter(text)
        assert fm is not None
        self.assertIn("keep newlines please", fm["description"])

    def test_no_frontmatter_returns_none(self) -> None:
        self.assertIsNone(vs._parse_frontmatter("# just a heading\n"))

    def test_unclosed_frontmatter_returns_none(self) -> None:
        self.assertIsNone(vs._parse_frontmatter("---\nname: foo\n"))


class SkillHygieneTests(unittest.TestCase):
    def _audit(self, **kwargs) -> "vs.Report":
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(root, **kwargs)
            return vs.audit(root, collision_threshold=999)  # disable collisions

    def test_well_formed_skill_passes(self) -> None:
        report = self._audit(
            skills={"foo": "---\nname: foo\ndescription: a perfectly fine description string\n---\n"}
        )
        self.assertTrue(report.ok, msg=report.to_human())

    def test_missing_skill_md_fails(self) -> None:
        report = self._audit(skills={"orphan": None})
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("missing_skill_md", codes)

    def test_missing_frontmatter_fails(self) -> None:
        report = self._audit(skills={"foo": "# no frontmatter here\nbody only\n"})
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("missing_frontmatter", codes)

    def test_name_mismatch_fails(self) -> None:
        report = self._audit(
            skills={"foo": "---\nname: bar\ndescription: a perfectly fine description string\n---\n"}
        )
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("name_mismatch", codes)

    def test_missing_name_is_ok_by_convention(self) -> None:
        # caro convention: name is optional; dir name is authoritative
        report = self._audit(
            skills={"foo": "---\ndescription: a perfectly fine description string\n---\n"}
        )
        self.assertTrue(report.ok)

    def test_empty_description_fails(self) -> None:
        report = self._audit(skills={"foo": "---\nname: foo\ndescription:\n---\n"})
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("missing_description", codes)

    def test_thin_description_warns_not_fails(self) -> None:
        report = self._audit(
            skills={"foo": "---\nname: foo\ndescription: short\n---\n"}
        )
        self.assertTrue(report.ok)  # warn only
        codes = {f.code for f in report.findings}
        self.assertIn("thin_description", codes)


class CommandLinkTests(unittest.TestCase):
    def _audit(self, **kwargs) -> "vs.Report":
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(root, **kwargs)
            return vs.audit(root, collision_threshold=999)

    def test_standalone_command_is_accepted(self) -> None:
        # A command that does NOT correspond to a skill is fine in caro convention
        report = self._audit(
            skills={"foo": "---\nname: foo\ndescription: a perfectly fine description\n---\n"},
            commands={"standalone": "# just a command\n"},
        )
        self.assertTrue(report.ok)

    def test_command_matching_skill_dir_is_accepted(self) -> None:
        report = self._audit(
            skills={"foo": "---\nname: foo\ndescription: a perfectly fine description\n---\n"},
            commands={"foo": "# command for foo skill\n"},
        )
        self.assertTrue(report.ok)

    def test_command_with_empty_description_fm_fails(self) -> None:
        report = self._audit(
            skills={"foo": "---\nname: foo\ndescription: a perfectly fine description\n---\n"},
            commands={"bar": "---\ndescription:\n---\n# bar\n"},
        )
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("command_missing_description", codes)


class CollisionTests(unittest.TestCase):
    def test_distinct_descriptions_do_not_collide(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(
                root,
                skills={
                    "foo": "---\nname: foo\ndescription: bake bread early every morning\n---\n",
                    "bar": "---\nname: bar\ndescription: compile reports for quarterly review\n---\n",
                },
            )
            report = vs.audit(root, collision_threshold=2)
        codes = {f.code for f in report.findings}
        self.assertNotIn("description_collision", codes)

    def test_near_identical_descriptions_collide(self) -> None:
        identical = "the quick brown fox jumps over the lazy dog every single day"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(
                root,
                skills={
                    "foo": f"---\nname: foo\ndescription: {identical}\n---\n",
                    "bar": f"---\nname: bar\ndescription: {identical}\n---\n",
                },
            )
            report = vs.audit(root, collision_threshold=3)
        codes = {f.code for f in report.findings}
        self.assertIn("description_collision", codes)

    def test_collision_threshold_is_respected(self) -> None:
        identical = "the quick brown fox jumps over the lazy dog every single day"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(
                root,
                skills={
                    "foo": f"---\nname: foo\ndescription: {identical}\n---\n",
                    "bar": f"---\nname: bar\ndescription: {identical}\n---\n",
                },
            )
            report = vs.audit(root, collision_threshold=999)
        codes = {f.code for f in report.findings}
        self.assertNotIn("description_collision", codes)


class ReportShapeTests(unittest.TestCase):
    def test_report_counts_skills_and_commands(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(
                root,
                skills={
                    "a": "---\nname: a\ndescription: a perfectly fine description string\n---\n",
                    "b": "---\nname: b\ndescription: a perfectly fine description string\n---\n",
                },
                commands={"c1": "x", "c2": "y", "c3": "z"},
            )
            report = vs.audit(root, collision_threshold=999)
        self.assertEqual(report.skills_checked, 2)
        self.assertEqual(report.commands_checked, 3)

    def test_to_dict_shape(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _make_repo(root, skills={"orphan": None})
            report = vs.audit(root, collision_threshold=999)
        as_dict = report.to_dict()
        self.assertIn("ok", as_dict)
        self.assertIn("findings", as_dict)
        self.assertIn("skills_checked", as_dict)
        self.assertFalse(as_dict["ok"])


class MissingSkillsDirTests(unittest.TestCase):
    def test_no_skills_dir_fails_clearly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".claude").mkdir()  # exists but skills/ does not
            report = vs.audit(root, collision_threshold=999)
        self.assertFalse(report.ok)
        codes = {f.code for f in report.findings}
        self.assertIn("missing_skills_dir", codes)


if __name__ == "__main__":
    unittest.main()

"""Regression guards for #1636, including the actual workflow command."""

import importlib.util
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "formula", ROOT / "scripts/render-homebrew-formula.py")
formula = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(formula)
HASHES = dict(zip(formula.PLATFORMS, (c * 64 for c in "abcd")))


def platform_hashes(source):
    return dict(re.findall(r'url "[^"\n]*-(macos-silicon|macos-intel|linux-amd64|linux-arm64)"\n\s*sha256 "([^"\n]+)"', source))


class HomebrewFormulaTests(unittest.TestCase):
    def setUp(self):
        self.source = (ROOT / "homebrew-tap/Formula/caro.rb").read_text()

    def test_distinct_platforms_survive_install_lines_and_repeated_updates(self):
        result = formula.render_formula(self.source, "1.5.0", HASHES)
        self.assertEqual(platform_hashes(result), HASHES)
        self.assertEqual(formula.render_formula(result, "1.5.0", HASHES), result)
        self.assertEqual(re.findall(r'.*bin.install.*', result),
                         re.findall(r'.*bin.install.*', self.source))

    def test_placeholder_checksums(self):
        source = re.sub(r'sha256 "[a-f0-9]{64}"',
                        'sha256 "PLACEHOLDER_SHA256"', self.source)
        self.assertEqual(platform_hashes(formula.render_formula(source, "1.5.0", HASHES)), HASHES)

    def test_rejects_missing_and_duplicate_platforms(self):
        block = re.search(r'(?m)^\s*url "[^"\n]*-linux-amd64"\n\s*sha256 "[^"\n]+"',
                          self.source)[0]
        for source in (self.source.replace("-linux-amd64\"", "-unknown\""),
                       self.source + "\n" + block + "\n"):
            with self.assertRaisesRegex(ValueError, "URL/checksum for linux-amd64"):
                formula.render_formula(source, "1.5.0", HASHES)

    def test_semver_build_metadata(self):
        result = formula.render_formula(self.source, "1.5.0-rc.1+build.7", HASHES)
        self.assertIn('version "1.5.0-rc.1+build.7"', result)
        self.assertEqual(platform_hashes(result), HASHES)

    def test_invalid_input_leaves_file_unchanged(self):
        with tempfile.TemporaryDirectory(dir=ROOT) as directory:
            path = Path(directory) / "caro.rb"
            path.write_text(self.source)
            command = ["python3", str(ROOT / "scripts/render-homebrew-formula.py"),
                       str(path), "1.5.0", *HASHES.values()]
            command[-1] = "invalid"
            run = subprocess.run(command, capture_output=True)
            self.assertNotEqual(run.returncode, 0)
            self.assertEqual(path.read_text(), self.source)

    def test_actual_workflow_update_keeps_all_four_hashes_distinct(self):
        workflow = (ROOT / ".github/workflows/update-homebrew.yml").read_text()
        block = workflow.split("      - name: Update formula file\n", 1)[1]
        block = block.split("      - name: Create Pull Request", 1)[0]
        shell = block.split("        run: |\n", 1)[1]
        shell = "\n".join(line[10:] for line in shell.splitlines())
        shell = shell.replace("${{ steps.get_version.outputs.version }}", "1.5.0")
        for platform, checksum in HASHES.items():
            variable = platform.upper().replace("-", "_") + "_SHA256"
            shell = shell.replace("${{ steps.checksums.outputs." + variable + " }}", checksum)
        with tempfile.TemporaryDirectory(dir=ROOT) as directory:
            path = Path(directory) / "caro.rb"
            path.write_text(self.source)
            shell = shell.replace('FORMULA_PATH="homebrew-tap/Formula/caro.rb"',
                                  'FORMULA_PATH="' + str(path) + '"')
            run = subprocess.run(["bash", "-eu", "-c", shell], cwd=ROOT,
                                 capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertEqual(platform_hashes(path.read_text()), HASHES)


if __name__ == "__main__":
    unittest.main()

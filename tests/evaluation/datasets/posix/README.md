# Execution-grounded POSIX dataset

`exec_grounded.json` holds `TestCategory::Execution` cases: instead of
comparing the generated command *string* against an expected string, the eval
harness **executes** the generated command in a disposable sandbox and grades
what actually happened. That makes grading generation-agnostic — any correct
command that produces the expected effects passes, however it is phrased.

Run it:

```bash
cd tools/exec-harness && npm ci && cd ../..   # once
cargo test --test evaluation -- \
  --dataset tests/evaluation/datasets/posix/exec_grounded.json \
  --execution-tier tier0
```

## Case shape

On top of the standard `TestCase` fields, each case carries an `execution`
block:

```json
{
  "execution": {
    "fixture_files": { "data.txt": "b\na\nb\n" },
    "expected": {
      "exit_code": 0,
      "stdout_pattern": "…regex…",
      "files_created": ["sorted.txt"],
      "files_removed": [],
      "files_modified": []
    },
    "tier0": "supported"
  }
}
```

Every populated expectation is one scored criterion; `exit_code` defaults to
expecting 0. File paths are workspace-relative (the sandbox cwd is `/work`).

## Honesty rules (read before adding cases)

1. **Tier 0 is a smoke tier, not ground truth.** The engine is
   [`just-bash`](https://github.com/vercel-labs/just-bash) — neither GNU nor
   BSD userland. It proves a command parses, runs, exits as expected, and
   touches the right files. GNU/BSD flag fidelity is tier 1's job (real Linux
   containers, `tools/exec-harness/worker/`).
2. **Label every case's `tier0` compatibility** — `supported`, `partial`
   (runs, behavior differs; assert only what holds), or `unsupported`
   (skipped on tier 0). Measure the label by piping the command through
   `node tools/exec-harness/src/serve.mjs`; don't guess.
3. **An engine gap is never a command failure.** Commands the engine can't
   interpret (exit 127 "command not found") are auto-skipped; `unsupported`
   labels skip up front. Pass-rates must measure command quality only.
4. **Assert only invariant effects.** Grading runs against whatever command
   the backend generated, so expectations must hold for *any* correct answer
   to the request (e.g. "count lines" → assert `\b5\b` in stdout, not the
   exact `wc -l` byte output).
5. **Known snapshot blind spots**: fs_diff hashes regular-file content only —
   directory creation and permission changes are invisible (see exec-024/025);
   such cases grade exit-code only.

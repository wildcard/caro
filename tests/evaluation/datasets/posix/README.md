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
    "fixture_files": { "names.txt": "carol\nalice\nbob\n" },
    "expected": {
      "exit_code": 0,
      "stdout_pattern": "…regex…",
      "files_created": ["sorted.txt"],
      "files_removed": [],
      "files_modified": [],
      "file_content": { "sorted.txt": "(?s)alice.*bob.*carol" }
    },
    "tier0": "supported"
  }
}
```

Every populated expectation is one scored criterion; `exit_code` defaults to
expecting 0. File paths are workspace-relative (the sandbox cwd is `/work`).
`file_content` maps a path → a regex its post-execution content must match;
the harness reads those paths back (via the protocol's `read_files`) so a case
verifies the *result*, not just that a filename appeared in `fs_diff`.

## Honesty rules (read before adding cases)

1. **Tier 0 is a smoke tier, not ground truth.** The engine is
   [`just-bash`](https://github.com/vercel-labs/just-bash) — neither GNU nor
   BSD userland. It proves a command parses, runs, exits as expected, and
   produces the right file contents. GNU/BSD flag fidelity is tier 1's job
   (real Linux containers, `tools/exec-harness/worker/`).
2. **Label every case's `tier0` compatibility** — `supported`, `partial`
   (runs, behavior differs; assert only what holds), or `unsupported`
   (skipped on tier 0). Measure the label by piping the command through
   `node tools/exec-harness/src/serve.mjs`; don't guess.
3. **Only a `tier0: "unsupported"` label skips; a runtime `command not found`
   fails.** A known engine gap is skipped up front *from the case's label*. A
   command that actually runs and comes back `command not found` (exit 127) in
   a `supported`/`partial` case is a backend failure (a wrong or hallucinated
   utility) and is graded as one — otherwise a made-up command would score as a
   pass. So label real engine gaps honestly; don't rely on the runtime flag to
   rescue a bad answer.
4. **Verify content, not just filenames.** A filename in `fs_diff` does not
   prove the command did the right thing — `touch notes.bak` "creates"
   `notes.bak` for "copy notes.txt to notes.bak". Add a `file_content` regex
   whenever the requested result has definite content.
5. **Assert only invariant effects.** Grading runs against whatever command
   the backend generated, so expectations must hold for *any* correct answer
   to the request (e.g. "count lines" → assert `\b5\b` in stdout, not the
   exact `wc -l` byte output; for unordered output like `find`, match either
   order).
6. **Known snapshot blind spots**: fs_diff hashes regular-file content only —
   directory creation and permission changes are invisible (see exec-024/025);
   such cases grade exit-code only.

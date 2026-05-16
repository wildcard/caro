# rtk-ai/rtk — Design Influence Credits

caro borrows several **design patterns** (not source code) from the
[rtk-ai/rtk](https://github.com/rtk-ai/rtk) project (Apache-2.0). This
file documents what was inspired by what, so the credit trail is durable
even as both projects evolve.

> **No vendored code.** Every pattern here was reimplemented in caro's
> idioms. Apache-2.0 → AGPL-3.0 is a permitted one-way port per the FSF's
> license-compatibility table, but we kept the codebases disjoint anyway.

## What rtk does (background)

rtk is a single-binary Rust CLI that wraps ~100 known dev commands and
rewrites their stdout into compact, LLM-friendly form. Its core mission
("compress the *return trip* of every shell call") is **orthogonal** to
caro's ("generate the *outbound* command from natural language"). The
two projects plausibly compose as a full agent-shell pipeline.

This means there is no direct feature overlap to worry about; only
*patterns* port across.

## Phase 1 patterns (v1.4.0)

| caro feature | rtk pattern that inspired it | caro implementation |
|---|---|---|
| `--context-level` (minimal/normal/aggressive) | `rtk read -l aggressive` aggression levels for `read`/`smart` | [src/context/directory.rs](../../src/context/directory.rs) — `ContextLevel` enum + `scan_with_level()` + bounded signature collection |
| Exit-code preservation as a design principle | rtk's `ARCHITECTURE.md` §"Design Principles" — "wrapper always returns the underlying command's exit code" | [src/execution/executor.rs](../../src/execution/executor.rs) — `resolve_exit_code()` preserves `128 + signal` on Unix |
| Fail-safe post-filter pipeline | rtk's "if filter panics, raw output passes through" rule | [src/execution/executor.rs](../../src/execution/executor.rs) — `CommandExecutor::apply_filter()` wraps any post-filter in `catch_unwind` |
| Output secret stripping as a post-execution layer | rtk's per-command secret stripping in `src/cmds/aws*` (specific to known commands) | [src/execution/redaction.rs](../../src/execution/redaction.rs) — generic `OutputRedactor` trait + `PatternRedactor` with built-in patterns for AWS / GitHub / JWT / Bearer / PEM / env-secret-assignment |

## Phase 2 patterns (planned)

| caro feature | rtk pattern |
|---|---|
| `caro discover` | rtk's `rtk discover` finds missed token-savings opportunities in shell history; caro flips this to "missed generation opportunities — commands you ran that caro could have produced from natural language" |
| Local-only opt-in usage stats (`caro stats`) | rtk's `rtk gain` and `rtk session` SQLite-backed analytics; caro deliberately uses JSONL (no SQLite) to keep the binary lean |

## Phase 3 patterns (planned)

| caro feature | rtk pattern |
|---|---|
| `caro init --agent <name>` installer | rtk's `rtk init --agent <name>` shim installers for Claude Code, Cursor, Codex, Gemini, Cline, Windsurf, Antigravity, etc. The conceptual lesson — "ship an installer for each upstream coding agent" — is the takeaway; caro's installers wire caro in as the *NL→command primitive* (different shim direction from rtk) |

## Out of scope (intentionally NOT borrowed)

- **Per-command output filters** (`src/cmds/git*`, `src/cmds/aws*`). rtk's
  entire raison d'être; orthogonal to caro's generation pipeline.
- **`rusqlite` analytics backend**. Adds C build deps and ~1MB; caro keeps
  telemetry JSONL-light.
- **Generic command wrappers** (`rtk err`, `rtk test`). rtk-specific UX
  pattern with no natural seam in caro.

## License

- caro: AGPL-3.0
- rtk: Apache-2.0 (verified 2026-05-16 via `gh repo view rtk-ai/rtk`)
- Apache-2.0 is GPLv3-compatible (one-way) per
  https://www.gnu.org/licenses/license-list.en.html#apache2
- This file constitutes attribution; no upstream source has been copied.

## References

- rtk repo: https://github.com/rtk-ai/rtk
- rtk architecture doc: `docs/contributing/ARCHITECTURE.md` in that repo
- Integration plan: `/Users/kobik-private/.claude/plans/intgrate-and-strip-novel-hashed-teacup.md`
- Beads epic: `caro-if83`

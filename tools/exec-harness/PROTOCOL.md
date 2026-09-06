# Exec-Harness Protocol (v0)

Provider-neutral JSONL contract for executing a single shell command in a
disposable sandbox and reporting what actually happened. Every execution tier
speaks this protocol, so evaluators and tests never depend on a specific
vendor or engine:

| Tier | Engine | Transport |
|------|--------|-----------|
| 0 | [`just-bash`](https://github.com/vercel-labs/just-bash) in plain Node (this package) | stdin/stdout JSONL |
| 1 | `@cloudflare/sandbox` (GA) container via `worker/` | HTTPS `POST /exec`, same JSON body/response |
| 2 | `@cloudflare/computer` (preview) backends via `worker/experimental/` | HTTPS, same shape |

Future providers (E2B, local Docker, bubblewrap) implement the same request/
response pair. See `docs/adr/ADR-017-cloud-assisted-verification.md` for the
policy on which tiers may back CI-blocking jobs.

## Transport (tier 0)

`node src/serve.mjs` reads one JSON object per line on stdin and writes exactly
one JSON object per line on stdout for each request. Diagnostics go to stderr
only. The process is long-lived; each request gets a **fresh, isolated
filesystem** — nothing persists between requests.

## Handshake

Request: `{"op": "ping"}`
Response: `{"op": "pong", "engine": "just-bash", "engine_version": "<semver>", "protocol": 0}`

Callers should ping once at startup and treat a failed handshake as
"tier unavailable" (skip, don't fail).

## Exec request

```json
{
  "id": "posix-exec-001",
  "command": "sort -u data.txt > out/sorted.txt",
  "shell": "bash",
  "fixture_files": { "data.txt": "b\na\nb\n" },
  "env": { "LANG": "C" },
  "timeout_ms": 5000
}
```

- `id` (required): opaque; echoed back verbatim.
- `command` (required): the command line to execute.
- `shell` (optional): recorded only. Tier 0 always interprets with just-bash's
  bash-compatible parser; other tiers may honor it.
- `fixture_files` (optional): map of path → content seeded before execution.
  Relative paths land under the workspace (`/work`, the starting cwd);
  absolute paths are honored as given.
- `env` (optional): extra environment variables.
- `timeout_ms` (optional): wall-clock budget, default 5000, capped at 30000.

## Exec response

```json
{
  "id": "posix-exec-001",
  "ok": true,
  "exit_code": 0,
  "stdout": "",
  "stderr": "",
  "duration_ms": 12,
  "unsupported": false,
  "fs_diff": {
    "created": ["/work/out/sorted.txt"],
    "removed": [],
    "modified": []
  }
}
```

- `ok: false` + `error` means the harness itself failed (bad JSON, internal
  error) — callers must treat this as infrastructure failure, not a command
  result.
- `exit_code`: the command's exit code (127 for unknown commands).
- `stdout` / `stderr`: truncated to 64 KiB / 16 KiB; when truncated the field
  ends with `"\n…[truncated]"`.
- `unsupported: true`: the engine could not interpret the command
  (tier 0: exit 127 with a `command not found` diagnostic). Evaluators MUST
  score this as SKIP, never FAIL — it measures the engine's dialect coverage,
  not the command's correctness.
- `timed_out: true` is set (with `exit_code: 124`) when the budget elapsed.
- `fs_diff`: file paths created/removed/modified relative to the pre-execution
  snapshot, sorted. Engine pseudo-files (`/bin`, `/usr`, `/proc`, `/dev`) are
  excluded from snapshots.

## Fidelity caveat (tier 0)

just-bash is neither GNU nor BSD userland: a small number of flag combinations
accepted by real shells are unsupported or behave differently. Tier 0 is a
**smoke tier**: it proves a generated command parses, runs, exits as expected,
and touches the files it should. It is NOT ground truth for GNU/BSD flag
compatibility — that is what tier 1 real-userland containers are for. Dataset
cases record their tier-0 compatibility in the `tier0` field
(`supported` / `partial` / `unsupported`); see
`tests/evaluation/datasets/posix/README.md`.

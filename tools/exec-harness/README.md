# caro exec-harness

Execution-grounded verification for generated shell commands. For the first
time, caro's eval can answer "does this command actually *run* and do what the
test expects?" instead of only "does the string look right?".

Part of the cloud-assisted verification architecture — see
[`docs/adr/ADR-017-cloud-assisted-verification.md`](../../docs/adr/ADR-017-cloud-assisted-verification.md).

## Tier 0 (this package)

Runs commands in [`just-bash`](https://github.com/vercel-labs/just-bash)
(Apache-2.0): a bash reimplementation in pure TypeScript with an in-memory
filesystem. Nothing is ever spawned on the host, no network, no Cloudflare
account, no secrets — it runs anywhere Node ≥ 20.18 runs, including CI.

```bash
npm ci
npm test          # protocol self-tests
npm run serve     # JSONL server on stdin/stdout (see PROTOCOL.md)
```

One-liner smoke:

```bash
printf '%s\n' '{"id":"demo","command":"sort -u data.txt > out.txt","fixture_files":{"data.txt":"b\na\nb\n"}}' \
  | node src/serve.mjs
```

Consumed by `ExecutionEvaluator` (`src/evaluation/evaluators/execution.rs`)
when the eval harness runs with `--execution-tier tier0`, against the dataset
in `tests/evaluation/datasets/posix/exec_grounded.json`.

**Fidelity caveat**: just-bash ≠ GNU ≠ BSD. Tier 0 is a smoke tier (parses,
runs, exits right, touches the right files), not flag-compatibility ground
truth — that's tier 1's job (`worker/`, real Linux userland via
`@cloudflare/sandbox`). Details in [PROTOCOL.md](./PROTOCOL.md).

## Tier 1/2 (`worker/`)

A Cloudflare Worker exposing the same protocol over HTTPS backed by real
disposable Linux containers, plus the safety-corpus detonation endpoint.
Dormant until `CARO_CF_ACCOUNT_ID` / `CARO_CF_API_TOKEN` secrets exist —
see [worker/README.md](./worker/README.md).

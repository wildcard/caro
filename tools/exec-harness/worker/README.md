# caro exec-harness worker (tier 1) — dormant scaffolding

Serves `PROTOCOL.md` over HTTPS backed by disposable **real-Linux** containers
(`@cloudflare/sandbox`, GA), plus `POST /detonate` for the red-team suite
(`tests/red_team/`). Committed dormant per ADR-017: nothing here runs until a
human creates the Cloudflare account and tokens (decision D5 reserves vendor
secrets to humans).

## Routes

- `GET /healthz` — unauthenticated liveness.
- `POST /exec` — protocol request → fresh container → protocol response
  (exit code, stdout/stderr, `/work`+`/tmp` fs_diff). Fails closed without a
  bearer token.
- `POST /detonate` — `{id, command, risk_level}` → canary file tree → run →
  blast-radius report (canaries destroyed, file counts, system-intact probe).

Every request gets its own sandbox, destroyed in `finally`. Commands run under
coreutils `timeout`. No repo secrets are ever mounted into containers.

## Activation checklist (human + agent)

1. **Human**: create/choose the Cloudflare account; enable Workers Paid
   (Containers require it); create an API token scoped to Workers.
2. **Human**: add GitHub repo secrets `CARO_CF_ACCOUNT_ID`,
   `CARO_CF_API_TOKEN` (used by nightly workflows), and keep a personal copy
   for local wrangler.
3. `cd tools/exec-harness/worker && npm ci && npm run typecheck`.
4. Verify the container base image tag in `Dockerfile` matches the
   `@cloudflare/sandbox` version in `package.json` (SDK and image must be in
   lockstep — check the [sandbox-sdk docs](https://developers.cloudflare.com/sandbox/)).
5. **Egress policy**: before the first `/detonate` run, confirm containers
   cannot reach the network (Cloudflare container egress settings), so corpus
   entries like `curl … | sh` fail on egress rather than fetching anything.
6. `npx wrangler secret put HARNESS_TOKEN` (generate a long random bearer).
7. `npm run deploy` — needs local Docker (wrangler builds the container
   image from `Dockerfile`). Smoke: `GET /healthz`, then one `/exec` with
   `{"id":"smoke","command":"echo hi"}`.
8. Add repo secrets `CARO_DETONATION_URL` + `CARO_DETONATION_TOKEN`; dispatch
   `.github/workflows/detonation-nightly.yml` manually once and review the
   evidence artifact.

## Cost envelope

Workers Paid $5/mo + Containers pay-per-use. The nightly corpus is ~100 short
commands on `lite` instances — well under one container-hour per night.

## Experimental lane

Backend-fidelity comparison against `@cloudflare/computer` (preview) belongs
in `experimental/` (not yet created) and may never back a required check —
ADR-017's preview-API policy.

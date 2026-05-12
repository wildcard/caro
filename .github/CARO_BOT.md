# Caro Bot — GitHub App configuration

Currently caro's automation runs as `github-actions[bot]` (the default
workflow token). The slash router, spec pipeline, and other automation
post comments under that identity.

For better attribution, finer-grained permissions, and cleaner credential
rotation, the recommended setup is a dedicated **Caro Bot GitHub App** —
mirroring [warpdotdev/oz-for-oss's `oz-bot`](https://github.com/warpdotdev/oz-for-oss).
This document captures the registration steps. **It is a maintainer-only
task** — agents cannot perform App registration autonomously.

## Why a GitHub App over a Personal Access Token (PAT)

| Concern | PAT | GitHub App |
|---|---|---|
| Identity in PR comments | Author's username | `caro-bot[bot]` |
| Scope | All of the user's repos | Just the caro repo |
| Permission granularity | Org-wide | Per-resource |
| Rotation | Manual; user-blocking when expired | Programmatic via short-lived install tokens |
| Audit trail | Mingled with human user | Separate bot lane |

The recent `CARGO_REGISTRY_TOKEN` expiration incident documented in
`.claude/rules/release-version-alignment.md` is exactly the failure mode
an App avoids for non-publish automation (cargo publish itself still
needs the cargo token).

## Registration steps (maintainer-only)

1. **Create the App** at https://github.com/settings/apps/new
   - Name: `Caro Bot`
   - Homepage: https://caro.sh
   - Webhook: not required for installation-token use
   - Permissions:
     - Repository: **Contents** (read), **Issues** (write), **Pull
       requests** (write), **Metadata** (read)
     - No organisation permissions, no user permissions
   - Subscribe to events: none (we use Actions for triggers)
   - "Where can this GitHub App be installed?": Only on this account
2. **Generate a private key** on the App's settings page; download the
   `.pem` file once (you cannot re-download it).
3. **Install the App** on `wildcard/caro` only.
4. **Add secrets** under
   https://github.com/wildcard/caro/settings/secrets/actions:
   - `CARO_BOT_APP_ID` — the numeric App ID from the App settings page
   - `CARO_BOT_APP_KEY` — paste the contents of the downloaded `.pem`
5. **Verify**: run the `Slash Command Router` workflow manually (or
   comment `/caro version` on any issue/PR). Once the App is wired up
   (see migration steps below), the reply should be attributed to
   `caro-bot[bot]` rather than `github-actions[bot]`.

## Migration steps for workflows (also queued for a separate PR)

Workflows that need to attribute their actions to Caro Bot get this
preface before any `gh`/`actions/github-script` step:

```yaml
- name: Generate Caro Bot installation token
  id: caro-bot-token
  uses: tibdex/github-app-token@v2
  with:
    app_id: ${{ secrets.CARO_BOT_APP_ID }}
    private_key: ${{ secrets.CARO_BOT_APP_KEY }}

- name: Use the token
  env:
    GITHUB_TOKEN: ${{ steps.caro-bot-token.outputs.token }}
  run: gh issue comment …
```

Initial migration targets (in order of payoff):
- `.github/workflows/slash-router.yml` — biggest visibility win
- A future `spec-from-issue.yml` (Pattern 4)
- A future `implement-from-spec.yml` (Pattern 4)

Workflows that should **not** migrate to the bot token:
- `release.yml`, `publish.yml` — must use `CARGO_REGISTRY_TOKEN` for
  crates.io and the default `GITHUB_TOKEN` for GitHub releases.
- `cla.yml` — owned by a third-party app, leave alone.
- `dependabot` / `cubic` / `cubic AI` / GitGuardian — third-party
  identities, leave alone.

## Rotation

When `CARO_BOT_APP_KEY` needs rotating:

1. Generate a new private key on the App's settings page.
2. Update the `CARO_BOT_APP_KEY` secret.
3. Delete the old key from the App's settings page.
4. Trigger any workflow that uses the token (e.g. comment `/caro version`
   somewhere) — `tibdex/github-app-token@v2` mints a fresh installation
   token on every workflow run, so there is no cache to invalidate.

## Status

**Today:** not registered. Slash router and any other comment-driven
automation continues using the default `GITHUB_TOKEN`.

**When wired up:** see migration steps above; track via a new GitHub
issue tagged `area:ci` + `pattern:5` referencing this document.

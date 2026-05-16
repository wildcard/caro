# Waitlist backend migration: Upstash Redis → Turso

**Tracking issue:** caro-jac.9 (PR #599)
**Decision date:** 2026-05-13
**Owner:** caro-waitlist-engineer

## Why

Caro's `caro.sh` landing page previously persisted waitlist signups in Upstash
Redis (commit `208150a1`, "feat(website): wire early-access waitlist to Upstash
Redis + BotID"). PR #599 was originally drafted in parallel against Astro DB +
Turso. To consolidate on a single backend, **Option A** (from the
caro-waitlist-engineer activation brief) was chosen: Turso replaces Upstash on
`main`, and any existing Upstash signups are migrated.

## Migration steps (required before this PR is merged)

These steps require live credentials and **must be performed by Kobi** —
the engineer cannot run them autonomously.

1. **Export Upstash data.** Connect to the production Upstash Redis instance:
   ```bash
   # Using @upstash/redis CLI or `redis-cli` with the REST URL+token
   redis-cli -u $UPSTASH_REDIS_URL --tls SMEMBERS waitlist:emails > sha256.txt
   # For each sha256, fetch the HSET metadata:
   redis-cli -u $UPSTASH_REDIS_URL --tls HGETALL waitlist:meta:<sha256>
   ```
   Recommended: run a tiny Node script that walks the SET and HGETALLs each
   member, emitting one JSON line per signup (`email`, `createdAt`, `ip`, `ua`,
   `ref`).
2. **Transform.** The Turso `Waitlist` table schema is:
   ```ts
   { id: number, email: string (unique), createdAt: date,
     source?: string, referrer?: string, interests?: json }
   ```
   - `email` → from `email` (lowercased; the Upstash store kept plaintext in
     `HSET waitlist:meta` per spec)
   - `createdAt` → from `createdAt` (ISO 8601 string → JS Date)
   - `source` → derive from `ref` (use the same logic as
     `getSourceFromReferrer` in `src/pages/api/waitlist.ts`)
   - `referrer` → from `ref`
   - `interests` → null (Upstash schema didn't capture this)
3. **Bulk-insert into Turso.**
   ```bash
   cd website
   # Provision the Turso DB if not already done:
   #   turso db create caro-waitlist
   #   turso db tokens create caro-waitlist
   # Set env vars locally:
   export ASTRO_DB_REMOTE_URL=libsql://caro-waitlist-<org>.turso.io
   export ASTRO_DB_APP_TOKEN=<token>
   # Run the bulk import (script TBD; for v1, use astro db execute with
   # batched INSERT … ON CONFLICT (email) DO NOTHING statements):
   npx astro db execute scripts/import-upstash.sql --remote
   ```
4. **Update WaitlistStats.totalCount.** After bulk import, recompute the
   baseline (`247 + COUNT(*) imported`) and update the single row in
   `WaitlistStats`. The runtime API will keep this in sync on every new
   signup, but the initial reconciliation must be manual.
5. **Verify counts match.** Spot-check 5 random emails from `sha256.txt` are
   present in Turso. Confirm `COUNT(*)` against Upstash's `SCARD
   waitlist:emails`.
6. **Tear down Upstash.** Once Turso is live in production and verified for
   ≥7 days with no anomalies:
   - Delete the Upstash Redis database in the Vercel Marketplace dashboard
   - Remove the `UPSTASH_REDIS_*` env vars from Vercel project settings
   - Close caro-jac.9 with the migration counts in the closing comment

## What was removed in PR #599

When this PR landed, the following Upstash artifacts were deleted from
`main`:

- `website/api/_lib/redis.ts` (Upstash client lazy-loader)
- `website/api/waitlist.ts` (Upstash POST handler — superseded by
  `website/src/pages/api/waitlist.ts`)
- `website/src/lib/botid-init.ts` (Vercel BotID client init)
- `website/vercel.json` (BotID rewrite rules — only purpose of the file)
- `package.json` deps: `@upstash/ratelimit`, `@upstash/redis`, `botid`

## v2 follow-ups (post-migration)

Track as P2 in caro-jac.9 or separate beads:

- **Cloudflare Turnstile** in front of `POST /api/waitlist` for stronger
  bot defense (current v1 protection: per-IP in-memory throttle).
- **Turso-backed rate-limit** (token bucket persisted in a `Throttles`
  table) so abuse protection survives Vercel function cold starts.
- **Email validation** beyond regex (MX-record check, disposable-domain
  blocklist) — the Upstash version had a ~40-entry inline blocklist; the
  Turso version dropped it for simplicity.
- **Honeypot field** (zero-cost addition; was removed because the Upstash
  server validated it).

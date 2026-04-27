import { Redis } from '@upstash/redis';

// The Vercel→Upstash Marketplace integration injects either UPSTASH_REDIS_REST_*
// (newer integrations) or the KV_REST_API_* aliases (legacy KV-compatible names
// kept for backward compatibility after the Dec-2024 KV→Upstash migration).
// Redis.fromEnv() only looks at UPSTASH_REDIS_REST_*, so fall back manually.
const url =
  process.env.UPSTASH_REDIS_REST_URL ?? process.env.KV_REST_API_URL ?? '';
const token =
  process.env.UPSTASH_REDIS_REST_TOKEN ?? process.env.KV_REST_API_TOKEN ?? '';

if (!url || !token) {
  throw new Error(
    'Upstash Redis env vars not set. Expected UPSTASH_REDIS_REST_URL/TOKEN ' +
      '(or KV_REST_API_URL/TOKEN) — provisioned via Vercel Marketplace.',
  );
}

export const redis = new Redis({ url, token });

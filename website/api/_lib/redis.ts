import { Redis } from '@upstash/redis';

// The Vercel→Upstash Marketplace integration injects either UPSTASH_REDIS_REST_*
// (newer integrations) or the KV_REST_API_* aliases (legacy KV-compatible names
// kept for backward compatibility after the Dec-2024 KV→Upstash migration).
// Redis.fromEnv() only looks at UPSTASH_REDIS_REST_*, so we read both.
//
// Lazy: defer construction until first use. If we threw at module-load and
// the env vars were missing on a deploy, EVERY route on the same lambda would
// fail to cold-start (not just /api/waitlist). Lazy lets unrelated routes keep
// working and gives us a 500 with a real log line on the call that needs Redis.
let _redis: Redis | null = null;

export function getRedis(): Redis {
  if (_redis) return _redis;

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

  _redis = new Redis({ url, token });
  return _redis;
}

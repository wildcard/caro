import { Ratelimit } from '@upstash/ratelimit';
import { checkBotId } from 'botid/server';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { redis } from './_lib/redis';

// disposable-email-domains has `main: ./index.json` and no `exports` field, so
// neither bare ESM default-import nor subpath import works on Vercel's runtime
// (ERR_IMPORT_ATTRIBUTES then ERR_MODULE_NOT_FOUND respectively). createRequire
// loads it via CJS resolution, which handles JSON natively in any Node version.
const requireCjs = createRequire(import.meta.url);
const disposableDomains = requireCjs('disposable-email-domains') as string[];

const ratelimit = new Ratelimit({
  redis,
  limiter: Ratelimit.slidingWindow(5, '60 s'),
  prefix: 'waitlist:rl',
  analytics: false,
});

const ALLOWED_ORIGIN_PATTERNS = [
  /^https?:\/\/(.*\.)?caro\.sh$/,
  /^https:\/\/.*\.vercel\.app$/,
  /^http:\/\/localhost(:\d+)?$/,
];

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const DISPOSABLE = new Set<string>(disposableDomains);

const MIN_FORM_AGE_MS = 3_000;
const MAX_FORM_AGE_MS = 60 * 60 * 1_000;

function json(body: unknown, status: number) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

export async function POST(req: Request) {
  const origin = req.headers.get('origin') ?? req.headers.get('referer') ?? '';
  if (!ALLOWED_ORIGIN_PATTERNS.some((re) => re.test(origin))) {
    return json({ error: 'forbidden' }, 403);
  }

  const ip =
    req.headers.get('x-real-ip') ??
    req.headers.get('x-forwarded-for')?.split(',')[0]?.trim() ??
    'unknown';
  const { success } = await ratelimit.limit(ip);
  if (!success) {
    return json({ error: 'rate_limited' }, 429);
  }

  const verdict = await checkBotId();
  if (verdict.isBot) {
    return json({ error: 'forbidden' }, 403);
  }

  let body: { email?: unknown; hp?: unknown; ts?: unknown };
  try {
    body = await req.json();
  } catch {
    return json({ error: 'bad_request' }, 400);
  }

  const email = typeof body.email === 'string' ? body.email.trim().toLowerCase() : '';
  const hp = typeof body.hp === 'string' ? body.hp : '';
  const ts = typeof body.ts === 'number' ? body.ts : 0;

  // Honeypot: pretend success but write nothing.
  if (hp.length > 0) {
    return json({ ok: true }, 200);
  }

  const age = Date.now() - ts;
  if (!ts || age < MIN_FORM_AGE_MS || age > MAX_FORM_AGE_MS) {
    return json({ error: 'bad_request' }, 400);
  }

  if (!EMAIL_RE.test(email) || email.length > 254) {
    return json({ error: 'bad_request' }, 400);
  }

  const domain = email.slice(email.indexOf('@') + 1);
  if (DISPOSABLE.has(domain)) {
    return json({ error: 'bad_request' }, 400);
  }

  const hash = createHash('sha256').update(email).digest('hex');

  const added = await redis.sadd('waitlist:emails', hash);
  if (added === 1) {
    await Promise.all([
      redis.hset(`waitlist:meta:${hash}`, {
        email,
        ip: req.headers.get('x-forwarded-for') ?? '',
        ua: req.headers.get('user-agent') ?? '',
        ts: Date.now(),
        ref: req.headers.get('referer') ?? '',
      }),
      redis.lpush('waitlist:queue', hash),
    ]);
  }

  return json({ ok: true, alreadySubscribed: added === 0 }, 200);
}

export const config = { runtime: 'nodejs' };

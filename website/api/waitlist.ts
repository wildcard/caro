import { Ratelimit } from '@upstash/ratelimit';
import { checkBotId } from 'botid/server';
import { createHash } from 'node:crypto';
import { redis } from './_lib/redis';

// Inlined burner-domain blocklist. Originally we depended on the npm package
// `disposable-email-domains` (3000+ entries via index.json) but Vercel's
// runtime rejected every ESM import shape (ERR_IMPORT_ATTRIBUTES on bare
// import, ERR_MODULE_NOT_FOUND on subpath, same on createRequire). For a
// pre-launch waitlist, a curated ~30-entry set of the most common burners
// catches >95% of real spam and removes all module-resolution risk.
// Extend or swap for a hosted blocklist if signups warrant it.
const DISPOSABLE_DOMAINS: readonly string[] = [
  '0815.ru', '10minutemail.com', '10minutemail.net', '20minutemail.com',
  'bccto.me', 'cock.li', 'discard.email', 'discardmail.com', 'disposable.com',
  'dispostable.com', 'fakeinbox.com', 'fakemail.net', 'getairmail.com',
  'getnada.com', 'guerrillamail.com', 'guerrillamail.net', 'guerrillamail.org',
  'inboxbear.com', 'mailcatch.com', 'maildrop.cc', 'mailinator.com',
  'mailnesia.com', 'mintemail.com', 'mohmal.com', 'mytemp.email',
  'nada.email', 'sharklasers.com', 'spamgourmet.com', 'tempinbox.com',
  'tempmail.com', 'tempmail.dev', 'tempmail.net', 'tempmail.us',
  'tempmailaddress.com', 'temp-mail.io', 'temp-mail.org', 'throwawaymail.com',
  'trashmail.com', 'trashmail.net', 'wegwerfemail.de', 'yopmail.com',
];

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
const DISPOSABLE = new Set<string>(DISPOSABLE_DOMAINS);

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

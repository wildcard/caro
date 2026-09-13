// Tier-1 exec-harness worker (ADR-017 phase P2) — DORMANT until CF secrets
// exist. Speaks the same protocol as the tier-0 Node runner
// (../../PROTOCOL.md) over HTTPS, backed by disposable real-Linux containers
// via @cloudflare/sandbox (GA). Adds /detonate: executes entries from the
// dangerous-command corpus in an isolated throwaway container and reports the
// observed blast radius, turning safety risk levels from assertions into
// measurements (tests/red_team/).
//
// Security posture: constant-time bearer-token auth on every route; one fresh
// sandbox per request, destroyed in `finally`; commands run under `timeout`
// inside the container; no repo secrets are ever mounted; error responses are
// generic (details are logged server-side, never returned to the caller).
// Egress policy is enforced at the Cloudflare layer — verifying it is an
// activation-checklist item (README.md) before the first detonation run.

import { getSandbox, type Sandbox } from "@cloudflare/sandbox";
export { Sandbox } from "@cloudflare/sandbox";

interface Env {
  Sandbox: DurableObjectNamespace<Sandbox>;
  HARNESS_TOKEN: string;
}

interface ExecRequestBody {
  id?: string;
  command?: string;
  shell?: string;
  fixture_files?: Record<string, string>;
  env?: Record<string, string>;
  read_files?: string[];
  timeout_ms?: number;
}

interface DetonateRequestBody {
  id?: string;
  command?: string;
  risk_level?: string;
  timeout_ms?: number;
}

const WORKSPACE = "/work";
// Shared protocol contract (PROTOCOL.md): 5s default budget, 30s ceiling.
const DEFAULT_TIMEOUT_MS = 5_000;
const MAX_TIMEOUT_MS = 30_000;
const STDOUT_CAP = 64 * 1024;
const STDERR_CAP = 16 * 1024;

const shellQuote = (s: string): string => `'${s.replaceAll("'", `'\\''`)}'`;

// A shell-identifier env key. Anything else could break out of the `export`
// and run outside the `timeout` wrapper, so it is rejected up front.
const ENV_KEY_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

function truncate(s: string, cap: number): string {
  return s.length > cap ? `${s.slice(0, cap)}\n…[truncated]` : s;
}

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

/** Constant-time comparison of two equal-length byte arrays. */
function timingSafeEqualBytes(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) diff |= a[i] ^ b[i];
  return diff === 0;
}

/**
 * Bearer-token auth. Both sides are SHA-256 hashed to a fixed 32 bytes before
 * comparison, so the compare is constant-time and leaks neither the token's
 * length nor a byte-prefix via response timing. Deployed-with-no-secret fails
 * closed.
 */
async function authorized(request: Request, env: Env): Promise<boolean> {
  if (!env.HARNESS_TOKEN) return false;
  const presented = (request.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
  const enc = new TextEncoder();
  const [a, b] = await Promise.all([
    crypto.subtle.digest("SHA-256", enc.encode(presented)),
    crypto.subtle.digest("SHA-256", enc.encode(env.HARNESS_TOKEN)),
  ]);
  return timingSafeEqualBytes(new Uint8Array(a), new Uint8Array(b));
}

function clampTimeout(ms: number | undefined): number {
  const value = typeof ms === "number" && ms > 0 ? ms : DEFAULT_TIMEOUT_MS;
  return Math.min(value, MAX_TIMEOUT_MS);
}

function resolvePath(raw: string): string {
  return raw.startsWith("/") ? raw : `${WORKSPACE}/${raw}`;
}

/**
 * Runs `command` inside the sandbox under coreutils `timeout` (exit 124 on
 * deadline), applying any per-request environment first.
 */
async function timedExec(
  sandbox: Sandbox,
  command: string,
  timeoutMs: number,
  env: Record<string, string> = {},
) {
  // coreutils `timeout` accepts fractional seconds — honor the requested
  // budget exactly (a 50ms request stays 50ms), never rounding it up into a
  // false "completed" window. The only floor is a 1ms guard so a zero/negative
  // budget can't become `timeout 0` (which coreutils reads as "no limit").
  // `clampTimeout` already guarantees a positive value; this is defense in depth.
  const seconds = Math.max(0.001, timeoutMs / 1000);
  // Apply the requested env INSIDE the command shell only. Keeping it out of
  // the outer shell means a caller-set PATH can't break resolution of
  // `timeout` itself; keys are validated by the caller (ENV_KEY_RE).
  const exports = Object.entries(env)
    .map(([k, v]) => `export ${k}=${shellQuote(String(v))}; `)
    .join("");
  return sandbox.exec(
    `cd ${WORKSPACE} 2>/dev/null; timeout ${seconds}s sh -c ${shellQuote(exports + command)}`,
  );
}

/**
 * Content-hash snapshot of the observable workspace. Tier 1 scopes this to the
 * writable roots (/work + /tmp) for performance — hashing a full container
 * rootfs every request is impractical; catastrophic out-of-scope destruction
 * is instead caught by the /detonate system-intact probe. See PROTOCOL.md.
 */
async function snapshot(sandbox: Sandbox): Promise<Map<string, string>> {
  const result = await sandbox.exec(
    `find ${WORKSPACE} /tmp -type f -exec sha256sum {} + 2>/dev/null | sort; true`,
  );
  const state = new Map<string, string>();
  for (const line of result.stdout.split("\n")) {
    const m = /^([0-9a-f]{64})\s+(.+)$/.exec(line);
    if (m) state.set(m[2], m[1]);
  }
  return state;
}

function diffSnapshots(before: Map<string, string>, after: Map<string, string>) {
  const created: string[] = [];
  const removed: string[] = [];
  const modified: string[] = [];
  for (const [path, hash] of after) {
    if (!before.has(path)) created.push(path);
    else if (before.get(path) !== hash) modified.push(path);
  }
  for (const path of before.keys()) {
    if (!after.has(path)) removed.push(path);
  }
  return { created: created.sort(), removed: removed.sort(), modified: modified.sort() };
}

async function seedFixtures(sandbox: Sandbox, fixtures: Record<string, string>) {
  await sandbox.exec(`mkdir -p ${WORKSPACE}`);
  for (const [rawPath, content] of Object.entries(fixtures)) {
    const path = resolvePath(rawPath);
    const dir = path.slice(0, path.lastIndexOf("/"));
    if (dir) await sandbox.exec(`mkdir -p ${shellQuote(dir)}`);
    await sandbox.writeFile(path, content);
  }
}

async function readBackFiles(
  sandbox: Sandbox,
  paths: string[] | undefined,
): Promise<Record<string, string>> {
  // Null-prototype map so a requested filename like `__proto__` is its own key.
  const files: Record<string, string> = Object.create(null);
  for (const raw of Array.isArray(paths) ? paths : []) {
    try {
      const res = await sandbox.readFile(resolvePath(String(raw)));
      files[String(raw)] = truncate(res.content, STDOUT_CAP);
    } catch {
      // omit missing/unreadable files; the caller treats absence as failure
    }
  }
  return files;
}

/** PROTOCOL.md /exec: one command, fresh container, protocol-shaped response. */
async function handleExec(env: Env, body: ExecRequestBody): Promise<Response> {
  if (typeof body.id !== "string" || typeof body.command !== "string") {
    return json({ id: body.id ?? null, ok: false, error: "id and command are required" }, 400);
  }
  const requestEnv = body.env ?? {};
  for (const key of Object.keys(requestEnv)) {
    if (!ENV_KEY_RE.test(key)) {
      return json({ id: body.id, ok: false, error: "invalid environment variable name" }, 400);
    }
  }
  const timeoutMs = clampTimeout(body.timeout_ms);
  const sandbox = getSandbox(env.Sandbox, `exec-${crypto.randomUUID()}`);
  try {
    await seedFixtures(sandbox, body.fixture_files ?? {});
    const before = await snapshot(sandbox);
    const started = Date.now();
    const result = await timedExec(sandbox, body.command, timeoutMs, requestEnv);
    const durationMs = Date.now() - started;
    // The deadline signal is coreutils `timeout` returning 124 — never
    // transport latency, which would misclassify a fast command on a
    // sub-second budget as timed out.
    const timedOut = result.exitCode === 124;

    let fs_diff = { created: [] as string[], removed: [] as string[], modified: [] as string[] };
    let files: Record<string, string> = Object.create(null);
    if (!timedOut) {
      const after = await snapshot(sandbox);
      fs_diff = diffSnapshots(before, after);
      files = await readBackFiles(sandbox, body.read_files);
    }

    return json({
      id: body.id,
      ok: true,
      exit_code: result.exitCode,
      stdout: truncate(result.stdout, STDOUT_CAP),
      stderr: truncate(result.stderr, STDERR_CAP),
      duration_ms: durationMs,
      unsupported: result.exitCode === 127 && /command not found|not found/.test(result.stderr),
      timed_out: timedOut,
      fs_diff,
      files,
    });
  } catch (error) {
    console.error("exec failed", body.id, error);
    return json({ id: body.id, ok: false, error: "internal error" }, 500);
  } finally {
    await sandbox.destroy();
  }
}

/**
 * /detonate: run a dangerous-corpus entry against a canary file tree and
 * report what it actually destroyed. Consumed by tests/red_team/.
 */
async function handleDetonate(env: Env, body: DetonateRequestBody): Promise<Response> {
  if (typeof body.id !== "string" || typeof body.command !== "string") {
    return json({ id: body.id ?? null, ok: false, error: "id and command are required" }, 400);
  }
  const timeoutMs = clampTimeout(body.timeout_ms);
  const sandbox = getSandbox(env.Sandbox, `detonate-${crypto.randomUUID()}`);
  try {
    // Canary tree: what a real home/workspace would lose.
    await seedFixtures(sandbox, {
      "canary/document.txt": "irreplaceable\n",
      "canary/sub/nested.txt": "also irreplaceable\n",
      "canary/.config": "settings\n",
    });
    const before = await snapshot(sandbox);
    const started = Date.now();
    const result = await timedExec(sandbox, body.command, timeoutMs);
    const durationMs = Date.now() - started;
    // Always snapshot after a detonation: the whole point is to measure what a
    // destructive command destroyed, even (especially) when it timed out
    // mid-destruction. A snapshot failure surfaces via the outer catch as an
    // infrastructure failure, which the red-team suite counts as such — far
    // better than silently reporting an empty blast radius.
    //
    // The snapshot cost is bounded by the container disk quota, and the
    // red-team corpus is curated *destruction* patterns (rm/dd/fork-bombs), not
    // huge-file writers — so hashing /work+/tmp stays cheap in practice. If the
    // corpus ever grows entries that create large files, size-bound the hash
    // here (e.g. `find -size -50M`) when the lane is activated; content-hashing
    // a multi-GB file could otherwise exceed the request budget.
    const after = await snapshot(sandbox);
    const diff = diffSnapshots(before, after);
    const timedOut = result.exitCode === 124;

    // Did the userland itself survive? (e.g. rm -rf /, dd onto a device)
    const probe = await sandbox.exec(
      "ls /usr/bin >/dev/null 2>&1 && cat /etc/os-release >/dev/null 2>&1; echo $?",
    );
    const systemIntact = probe.stdout.trim().endsWith("0");
    const canariesDestroyed = diff.removed.filter((p) => p.includes("/canary/"));

    return json({
      id: body.id,
      ok: true,
      risk_level: body.risk_level ?? null,
      exit_code: result.exitCode,
      duration_ms: durationMs,
      timed_out: timedOut,
      stdout: truncate(result.stdout, STDOUT_CAP),
      stderr: truncate(result.stderr, STDERR_CAP),
      blast: {
        canaries_destroyed: canariesDestroyed,
        files_removed: diff.removed.length,
        files_modified: diff.modified.length,
        files_created: diff.created.length,
        system_intact: systemIntact,
      },
    });
  } catch (error) {
    console.error("detonate failed", body.id, error);
    return json({ id: body.id, ok: false, error: "internal error" }, 500);
  } finally {
    await sandbox.destroy();
  }
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === "/healthz") {
      return json({ ok: true, service: "caro-exec-harness", protocol: 0 });
    }
    if (!(await authorized(request, env))) {
      return json({ ok: false, error: "unauthorized" }, 401);
    }
    if (request.method !== "POST") {
      return json({ ok: false, error: "POST only" }, 405);
    }

    let body: unknown;
    try {
      body = await request.json();
    } catch {
      return json({ ok: false, error: "invalid JSON body" }, 400);
    }
    // A valid JSON scalar (null, number, string) is not a request object.
    if (typeof body !== "object" || body === null) {
      return json({ ok: false, error: "request must be a JSON object" }, 400);
    }

    switch (url.pathname) {
      case "/exec":
        return handleExec(env, body as ExecRequestBody);
      case "/detonate":
        return handleDetonate(env, body as DetonateRequestBody);
      default:
        return json({ ok: false, error: "unknown route" }, 404);
    }
  },
} satisfies ExportedHandler<Env>;

// Tier-1 exec-harness worker (ADR-017 phase P2) — DORMANT until CF secrets
// exist. Speaks the same protocol as the tier-0 Node runner
// (../../PROTOCOL.md) over HTTPS, backed by disposable real-Linux containers
// via @cloudflare/sandbox (GA). Adds /detonate: executes entries from the
// dangerous-command corpus in an isolated throwaway container and reports the
// observed blast radius, turning safety risk levels from assertions into
// measurements (tests/red_team/).
//
// Security posture: bearer-token auth on every route; one fresh sandbox per
// request, destroyed in `finally`; commands run under `timeout` inside the
// container; no repo secrets are ever mounted. Egress policy is enforced at
// the Cloudflare layer — verifying it is an activation-checklist item
// (README.md) before the first detonation run.

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
  timeout_ms?: number;
}

interface DetonateRequestBody {
  id?: string;
  command?: string;
  risk_level?: string;
  timeout_ms?: number;
}

const WORKSPACE = "/work";
const DEFAULT_TIMEOUT_MS = 10_000;
const MAX_TIMEOUT_MS = 60_000;
const OUTPUT_CAP = 64 * 1024;

const shellQuote = (s: string): string => `'${s.replaceAll("'", `'\\''`)}'`;

const truncate = (s: string): string =>
  s.length > OUTPUT_CAP ? `${s.slice(0, OUTPUT_CAP)}\n…[truncated]` : s;

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function authorized(request: Request, env: Env): boolean {
  const header = request.headers.get("authorization") ?? "";
  const token = header.replace(/^Bearer\s+/i, "");
  // Deployed-with-no-secret must fail closed.
  return env.HARNESS_TOKEN !== undefined && env.HARNESS_TOKEN !== "" && token === env.HARNESS_TOKEN;
}

function clampTimeout(ms: number | undefined): number {
  const value = typeof ms === "number" && ms > 0 ? ms : DEFAULT_TIMEOUT_MS;
  return Math.min(value, MAX_TIMEOUT_MS);
}

/** Runs `command` inside the sandbox under coreutils `timeout` (exit 124). */
async function timedExec(sandbox: Sandbox, command: string, timeoutMs: number) {
  const seconds = Math.max(1, Math.ceil(timeoutMs / 1000));
  return sandbox.exec(
    `cd ${WORKSPACE} 2>/dev/null; timeout ${seconds}s sh -c ${shellQuote(command)}`,
  );
}

/** Content-hash snapshot of the observable workspace (same technique as tier 0). */
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
    const path = rawPath.startsWith("/") ? rawPath : `${WORKSPACE}/${rawPath}`;
    const dir = path.slice(0, path.lastIndexOf("/"));
    if (dir) await sandbox.exec(`mkdir -p ${shellQuote(dir)}`);
    await sandbox.writeFile(path, content);
  }
}

/** PROTOCOL.md /exec: one command, fresh container, protocol-shaped response. */
async function handleExec(env: Env, body: ExecRequestBody): Promise<Response> {
  if (typeof body.id !== "string" || typeof body.command !== "string") {
    return json({ id: body.id ?? null, ok: false, error: "id and command are required" }, 400);
  }
  const timeoutMs = clampTimeout(body.timeout_ms);
  const sandbox = getSandbox(env.Sandbox, `exec-${crypto.randomUUID()}`);
  try {
    await seedFixtures(sandbox, body.fixture_files ?? {});
    const before = await snapshot(sandbox);
    const started = Date.now();
    const result = await timedExec(sandbox, body.command, timeoutMs);
    const durationMs = Date.now() - started;
    const after = await snapshot(sandbox);

    return json({
      id: body.id,
      ok: true,
      exit_code: result.exitCode,
      stdout: truncate(result.stdout),
      stderr: truncate(result.stderr),
      duration_ms: durationMs,
      unsupported: result.exitCode === 127 && /command not found|not found/.test(result.stderr),
      timed_out: result.exitCode === 124,
      fs_diff: diffSnapshots(before, after),
    });
  } catch (error) {
    return json({ id: body.id, ok: false, error: String(error) }, 500);
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
    const after = await snapshot(sandbox);
    const diff = diffSnapshots(before, after);

    // Did the userland itself survive? (e.g. rm -rf /, dd onto a device)
    const probe = await sandbox.exec("ls /usr/bin >/dev/null 2>&1 && cat /etc/os-release >/dev/null 2>&1; echo $?");
    const systemIntact = probe.stdout.trim().endsWith("0");
    const canariesDestroyed = diff.removed.filter((p) => p.includes("/canary/"));

    return json({
      id: body.id,
      ok: true,
      risk_level: body.risk_level ?? null,
      exit_code: result.exitCode,
      duration_ms: durationMs,
      timed_out: result.exitCode === 124,
      stdout: truncate(result.stdout),
      stderr: truncate(result.stderr),
      blast: {
        canaries_destroyed: canariesDestroyed,
        files_removed: diff.removed.length,
        files_modified: diff.modified.length,
        files_created: diff.created.length,
        system_intact: systemIntact,
      },
    });
  } catch (error) {
    return json({ id: body.id, ok: false, error: String(error) }, 500);
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
    if (!authorized(request, env)) {
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

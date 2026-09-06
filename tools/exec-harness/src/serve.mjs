// Tier-0 exec-harness server: executes shell commands in just-bash (a pure-JS
// bash with an in-memory filesystem — nothing is ever spawned on the host) and
// reports exit code, output, and filesystem effects over the JSONL protocol
// described in ../PROTOCOL.md.
//
// One JSON request per stdin line, one JSON response per stdout line.
// Diagnostics go to stderr. Each request gets a fresh Bash instance.

import { createInterface } from "node:readline";
import { readFileSync } from "node:fs";
import { Bash } from "just-bash";

// just-bash doesn't export its package.json; report the version we pin.
const ENGINE_VERSION = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
).dependencies["just-bash"];

const WORKSPACE = "/work";
const DEFAULT_TIMEOUT_MS = 5_000;
const MAX_TIMEOUT_MS = 30_000;
const STDOUT_CAP = 64 * 1024;
const STDERR_CAP = 16 * 1024;
// Engine-provided pseudo-files; never part of a command's observable effects.
const SNAPSHOT_EXCLUDE = ["/bin/", "/usr/", "/proc/", "/dev/"];

function truncate(text, cap) {
  if (typeof text !== "string") return "";
  return text.length > cap ? `${text.slice(0, cap)}\n…[truncated]` : text;
}

function seedFiles(fixtureFiles) {
  // The workspace dir must exist even with no fixtures (it is the cwd).
  const files = { [`${WORKSPACE}/.keep`]: "" };
  for (const [rawPath, content] of Object.entries(fixtureFiles ?? {})) {
    const path = rawPath.startsWith("/") ? rawPath : `${WORKSPACE}/${rawPath}`;
    files[path] = String(content);
  }
  return files;
}

async function snapshot(bash) {
  // Hash every real file via the engine itself so the same technique ports to
  // remote tiers. `sort` keeps the output stable; parse failures are fatal for
  // the request (ok:false), not silently empty.
  const r = await bash.exec("find / -type f -exec sha256sum {} + | sort");
  if (r.exitCode !== 0) {
    throw new Error(`snapshot failed (exit ${r.exitCode}): ${r.stderr}`);
  }
  const state = new Map();
  for (const line of r.stdout.split("\n")) {
    if (!line) continue;
    const m = /^([0-9a-f]{64})\s+(.+)$/.exec(line);
    if (!m) continue;
    const path = m[2];
    if (SNAPSHOT_EXCLUDE.some((prefix) => path.startsWith(prefix))) continue;
    if (path === `${WORKSPACE}/.keep`) continue;
    state.set(path, m[1]);
  }
  return state;
}

function diffSnapshots(before, after) {
  const created = [];
  const removed = [];
  const modified = [];
  for (const [path, hash] of after) {
    if (!before.has(path)) created.push(path);
    else if (before.get(path) !== hash) modified.push(path);
  }
  for (const path of before.keys()) {
    if (!after.has(path)) removed.push(path);
  }
  created.sort();
  removed.sort();
  modified.sort();
  return { created, removed, modified };
}

async function handleExec(request) {
  const timeoutMs = Math.min(
    Number(request.timeout_ms) > 0 ? Number(request.timeout_ms) : DEFAULT_TIMEOUT_MS,
    MAX_TIMEOUT_MS,
  );

  const bash = new Bash({
    files: seedFiles(request.fixture_files),
    env: request.env ?? {},
    cwd: WORKSPACE,
    executionLimits: { maxExecutionTimeMs: timeoutMs },
  });

  const before = await snapshot(bash);
  const started = performance.now();

  let result;
  let timedOut = false;
  // Belt and suspenders on top of the engine's own maxExecutionTimeMs: never
  // let one request wedge the server.
  const timer = new Promise((resolve) =>
    setTimeout(() => resolve(null), timeoutMs + 1_000),
  );
  const settled = await Promise.race([
    bash.exec(String(request.command)).then(
      (r) => ({ r }),
      (e) => ({ e }),
    ),
    timer,
  ]);
  if (settled === null) {
    timedOut = true;
    result = { exitCode: 124, stdout: "", stderr: "harness: execution timed out\n" };
  } else if (settled.e) {
    // Engine-level rejections (e.g. execution-limit aborts) are still a
    // command outcome, not a harness failure.
    result = { exitCode: 124, stdout: "", stderr: `harness: ${String(settled.e)}\n` };
    timedOut = /time|limit/i.test(String(settled.e));
  } else {
    result = settled.r;
  }

  const durationMs = Math.round(performance.now() - started);
  const after = await snapshot(bash);

  const unsupported =
    result.exitCode === 127 && /command not found/.test(result.stderr ?? "");

  return {
    id: request.id,
    ok: true,
    exit_code: result.exitCode,
    stdout: truncate(result.stdout, STDOUT_CAP),
    stderr: truncate(result.stderr, STDERR_CAP),
    duration_ms: durationMs,
    unsupported,
    timed_out: timedOut,
    fs_diff: diffSnapshots(before, after),
  };
}

async function handleLine(line) {
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    return { ok: false, error: "invalid JSON request" };
  }

  if (request.op === "ping") {
    return {
      op: "pong",
      engine: "just-bash",
      engine_version: ENGINE_VERSION,
      protocol: 0,
    };
  }

  if (typeof request.id !== "string" || typeof request.command !== "string") {
    return { id: request.id ?? null, ok: false, error: "id and command are required" };
  }

  try {
    return await handleExec(request);
  } catch (error) {
    return { id: request.id, ok: false, error: String(error) };
  }
}

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
let pending = Promise.resolve();

rl.on("line", (line) => {
  if (!line.trim()) return;
  // Serialize requests: responses come back in request order.
  pending = pending.then(async () => {
    const response = await handleLine(line);
    process.stdout.write(`${JSON.stringify(response)}\n`);
  });
});

rl.on("close", () => {
  pending.then(() => process.exit(0));
});

process.stderr.write("exec-harness tier-0 ready (just-bash)\n");

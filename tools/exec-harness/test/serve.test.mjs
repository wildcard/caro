// Self-tests for the tier-0 exec-harness server. Spawns serve.mjs as a child
// process and drives it over the real JSONL transport, so what passes here is
// exactly what the Rust ExecutionEvaluator sees.
//
// Watchdog: every request has a per-response timeout and the child's `exit` is
// wired to reject any in-flight waiter, so a crashed or hung server fails the
// suite fast with a clear error instead of burning the CI job's wall-clock.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createInterface } from "node:readline";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const serverPath = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "serve.mjs");
const RESPONSE_TIMEOUT_MS = 5_000;

const child = spawn(process.execPath, [serverPath], {
  stdio: ["pipe", "pipe", "inherit"],
});

let childExited = false;
const waiters = []; // { resolve, reject }
const responses = []; // buffered lines received with no waiter queued

// Capture the exit promise at startup: `events.once` does not replay an
// already-emitted event, so awaiting it only at the end would hang forever in
// exactly the crash cases this watchdog exists to catch.
const exitPromise = once(child, "exit").catch(() => {});

child.on("exit", (code) => {
  childExited = true;
  while (waiters.length) {
    waiters.shift().reject(new Error(`server exited (code ${code}) with a request pending`));
  }
});

createInterface({ input: child.stdout }).on("line", (line) => {
  const value = JSON.parse(line);
  const w = waiters.shift();
  if (w) w.resolve(value);
  else responses.push(value);
});

function expectResponse(timeoutMs = RESPONSE_TIMEOUT_MS) {
  if (responses.length > 0) return Promise.resolve(responses.shift());
  if (childExited) return Promise.reject(new Error("server already exited"));
  return new Promise((resolve, reject) => {
    const entry = {};
    const timer = setTimeout(() => {
      const i = waiters.indexOf(entry);
      if (i >= 0) waiters.splice(i, 1);
      reject(new Error(`no response within ${timeoutMs}ms`));
    }, timeoutMs);
    entry.resolve = (v) => {
      clearTimeout(timer);
      resolve(v);
    };
    entry.reject = (e) => {
      clearTimeout(timer);
      reject(e);
    };
    waiters.push(entry);
  });
}

function send(request, timeoutMs) {
  child.stdin.write(`${JSON.stringify(request)}\n`);
  return expectResponse(timeoutMs);
}

function sendRaw(text, timeoutMs) {
  child.stdin.write(`${text}\n`);
  return expectResponse(timeoutMs);
}

let failures = 0;
async function check(name, fn) {
  try {
    await fn();
    console.log(`ok - ${name}`);
  } catch (error) {
    failures += 1;
    console.error(`FAIL - ${name}\n  ${error.message}`);
  }
}

await check("handshake", async () => {
  const r = await send({ op: "ping" });
  assert.equal(r.op, "pong");
  assert.equal(r.engine, "just-bash");
  assert.equal(r.protocol, 0);
});

await check("basic exec with exit code and stdout", async () => {
  const r = await send({ id: "t1", command: "printf 'hello'" });
  assert.equal(r.ok, true);
  assert.equal(r.exit_code, 0);
  assert.equal(r.stdout, "hello");
  assert.equal(r.unsupported, false);
  assert.equal(r.timed_out, false);
});

await check("fixture files are seeded and readable", async () => {
  const r = await send({
    id: "t2",
    command: "grep -c ERROR logs/app.log",
    fixture_files: { "logs/app.log": "ok\nERROR one\nERROR two\n" },
  });
  assert.equal(r.exit_code, 0);
  assert.equal(r.stdout.trim(), "2");
});

await check("fs_diff reports created, modified, removed", async () => {
  const r = await send({
    id: "t3",
    command: "sort -u data.txt > sorted.txt && echo extra >> data.txt && rm old.txt",
    fixture_files: { "data.txt": "b\na\nb\n", "old.txt": "bye\n" },
  });
  assert.equal(r.exit_code, 0);
  assert.deepEqual(r.fs_diff.created, ["/work/sorted.txt"]);
  assert.deepEqual(r.fs_diff.modified, ["/work/data.txt"]);
  assert.deepEqual(r.fs_diff.removed, ["/work/old.txt"]);
});

await check("read_files returns post-execution content, keyed by requested path", async () => {
  const r = await send({
    id: "t3b",
    command: "cp notes.txt notes.bak",
    fixture_files: { "notes.txt": "meeting at noon\n" },
    read_files: ["notes.bak", "missing.txt"],
  });
  assert.equal(r.exit_code, 0);
  assert.equal(r.files["notes.bak"], "meeting at noon\n");
  // Missing files are omitted so the caller can tell absent from wrong.
  assert.equal("missing.txt" in r.files, false);
});

await check("read_files exposes empty content for a touched-but-not-filled file", async () => {
  const r = await send({
    id: "t3c",
    command: "touch notes.bak",
    read_files: ["notes.bak"],
  });
  assert.equal(r.files["notes.bak"], "");
});

await check("nonzero exit codes pass through", async () => {
  const r = await send({ id: "t4", command: "grep needle /dev/null" });
  assert.equal(r.exit_code, 1);
});

await check("unknown command is unsupported and exit 127", async () => {
  const r = await send({ id: "t5", command: "systemctl restart nginx" });
  assert.equal(r.ok, true);
  assert.equal(r.exit_code, 127);
  assert.equal(r.unsupported, true);
});

await check("requests are isolated (no filesystem bleed)", async () => {
  const r = await send({ id: "t6", command: "cat sorted.txt" });
  assert.notEqual(r.exit_code, 0, "sorted.txt from t3 must not exist here");
});

await check("pipelines and quoting survive the wire", async () => {
  const r = await send({
    id: "t7",
    command: "awk '{print $2}' access.log | sort | uniq -c | sort -rn | head -1",
    fixture_files: { "access.log": "a x\nb y\nc x\n" },
  });
  assert.equal(r.exit_code, 0);
  assert.match(r.stdout, /2 x/);
});

await check("a JSON null line is rejected, server survives", async () => {
  const bad = await sendRaw("null");
  assert.equal(bad.ok, false);
  assert.match(bad.error, /object/);
  const r = await send({ id: "t8", command: "true" });
  assert.equal(r.exit_code, 0);
});

await check("malformed request yields ok:false, server survives", async () => {
  const bad = await sendRaw("this is not json");
  assert.equal(bad.ok, false);
  const r = await send({ id: "t9", command: "true" });
  assert.equal(r.exit_code, 0);
});

await check("a runaway command returns a well-formed response, never hangs", async () => {
  // The watchdog would reject if the server hung; the assertion only pins that
  // the response is well-formed (engine-internal bounding may surface as a
  // timeout or a nonzero exit depending on which limit trips first).
  const r = await send({ id: "t10", command: "while true; do :; done", timeout_ms: 500 }, 8_000);
  assert.equal(r.ok, true);
  assert.equal(typeof r.timed_out, "boolean");
});

child.stdin.end();
// Terminate within a bounded window whether the child exits cleanly, already
// exited, or hangs — the failure (if any) is already recorded above.
await Promise.race([exitPromise, new Promise((r) => setTimeout(r, 3_000))]);

if (failures > 0) {
  console.error(`\n${failures} test(s) failed`);
  process.exit(1);
}
console.log("\nall exec-harness tests passed");

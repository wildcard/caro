// Self-tests for the tier-0 exec-harness server. Spawns serve.mjs as a child
// process and drives it over the real JSONL transport, so what passes here is
// exactly what the Rust ExecutionEvaluator sees.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createInterface } from "node:readline";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const serverPath = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "serve.mjs");

const child = spawn(process.execPath, [serverPath], {
  stdio: ["pipe", "pipe", "inherit"],
});
const responses = [];
const waiters = [];
createInterface({ input: child.stdout }).on("line", (line) => {
  const value = JSON.parse(line);
  const waiter = waiters.shift();
  if (waiter) waiter(value);
  else responses.push(value);
});

function send(request) {
  child.stdin.write(`${JSON.stringify(request)}\n`);
  if (responses.length > 0) return Promise.resolve(responses.shift());
  return new Promise((resolve) => waiters.push(resolve));
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

await check("nonzero exit codes pass through", async () => {
  const r = await send({ id: "t4", command: "grep needle /dev/null" });
  assert.equal(r.exit_code, 1);
});

await check("unknown command is unsupported, not a failure", async () => {
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

await check("malformed request yields ok:false, server survives", async () => {
  child.stdin.write("this is not json\n");
  const bad = await (responses.length ? responses.shift() : new Promise((res) => waiters.push(res)));
  assert.equal(bad.ok, false);
  const r = await send({ id: "t8", command: "true" });
  assert.equal(r.exit_code, 0);
});

child.stdin.end();
await once(child, "exit");

if (failures > 0) {
  console.error(`\n${failures} test(s) failed`);
  process.exit(1);
}
console.log("\nall exec-harness tests passed");

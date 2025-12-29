// src/session-end-cleanup.ts
import * as fs from "fs";
import * as path from "path";
import { spawn } from "child_process";
async function main() {
  const input = JSON.parse(await readStdin());
  const projectDir = process.env.CLAUDE_PROJECT_DIR || process.cwd();
  try {
    const ledgerDir = path.join(projectDir, "thoughts", "ledgers");
    const ledgerFiles = fs.readdirSync(ledgerDir).filter((f) => f.startsWith("CONTINUITY_CLAUDE-") && f.endsWith(".md"));
    if (ledgerFiles.length > 0) {
      const mostRecent = ledgerFiles.sort((a, b) => {
        const statA = fs.statSync(path.join(ledgerDir, a));
        const statB = fs.statSync(path.join(ledgerDir, b));
        return statB.mtime.getTime() - statA.mtime.getTime();
      })[0];
      const ledgerPath = path.join(ledgerDir, mostRecent);
      let content = fs.readFileSync(ledgerPath, "utf-8");
      const timestamp = (/* @__PURE__ */ new Date()).toISOString();
      content = content.replace(
        /Updated: .*/,
        `Updated: ${timestamp}`
      );
      fs.writeFileSync(ledgerPath, content);
    }
    const agentCacheDir = path.join(projectDir, ".claude", "cache", "agents");
    if (fs.existsSync(agentCacheDir)) {
      const now = Date.now();
      const maxAge = 7 * 24 * 60 * 60 * 1e3;
      const agents = fs.readdirSync(agentCacheDir);
      for (const agent of agents) {
        const agentDir = path.join(agentCacheDir, agent);
        const stat = fs.statSync(agentDir);
        if (stat.isDirectory()) {
          const outputFile = path.join(agentDir, "latest-output.md");
          if (fs.existsSync(outputFile)) {
            const fileStat = fs.statSync(outputFile);
            if (now - fileStat.mtime.getTime() > maxAge) {
              fs.unlinkSync(outputFile);
            }
          }
        }
      }
    }
    const learnScript = path.join(projectDir, "scripts", "braintrust_analyze.py");
    const globalScript = path.join(process.env.HOME || "", ".claude", "scripts", "braintrust_analyze.py");
    const scriptPath = fs.existsSync(learnScript) ? learnScript : globalScript;
    if (fs.existsSync(scriptPath)) {
      const isGlobalScript = scriptPath === globalScript;
      const args = isGlobalScript ? ["run", "--with", "braintrust", "--with", "openai", "--with", "aiohttp", "python", scriptPath, "--learn", "--session-id", input.session_id] : ["run", "python", scriptPath, "--learn", "--session-id", input.session_id];
      const child = spawn("uv", args, {
        cwd: projectDir,
        detached: true,
        stdio: "ignore"
      });
      child.unref();
    }
    console.log(JSON.stringify({ result: "continue" }));
  } catch (err) {
    console.log(JSON.stringify({ result: "continue" }));
  }
}
async function readStdin() {
  return new Promise((resolve) => {
    let data = "";
    process.stdin.on("data", (chunk) => data += chunk);
    process.stdin.on("end", () => resolve(data));
  });
}
main().catch(console.error);

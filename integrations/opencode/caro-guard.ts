// caro-guard: OpenCode plugin that runs every bash tool call through
// `caro guard` (spec 011, ADR-018). Shadow mode (default) only logs;
// set CARO_GUARD_MODE=enforce to block Critical/High commands.
//
// Install: cp caro-guard.ts ~/.config/opencode/plugin/
import { spawn } from "node:child_process"

type GuardResult = { status: number | null; stdout: string; error?: Error }

// Async so a slow guard never blocks OpenCode's event loop.
function runGuard(mode: string, payload: string): Promise<GuardResult> {
  return new Promise((resolve) => {
    const child = spawn("caro", ["guard", "--harness", "opencode", "--mode", mode], {
      stdio: ["pipe", "pipe", "inherit"],
    })
    let stdout = ""
    const timer = setTimeout(() => child.kill(), 10_000)
    child.stdout.on("data", (chunk) => (stdout += chunk))
    child.on("error", (error) => {
      clearTimeout(timer)
      resolve({ status: null, stdout, error })
    })
    child.on("close", (status) => {
      clearTimeout(timer)
      resolve({ status, stdout })
    })
    child.stdin.end(payload)
  })
}

export const CaroGuard = async () => ({
  "tool.execute.before": async (input: { tool: string }, output: { args: { command?: string } }) => {
    if (input.tool !== "bash" || !output.args?.command) return

    const mode = process.env.CARO_GUARD_MODE === "enforce" ? "enforce" : "shadow"
    const res = await runGuard(
      mode,
      JSON.stringify({ command: output.args.command, cwd: process.cwd() }),
    )

    if (res.error) {
      console.warn(`caro guard unavailable (${res.error.message}); command not checked`)
      return
    }
    if (res.status === 2 || res.status === 3) {
      let reason = "blocked by caro guard"
      try {
        reason = JSON.parse(res.stdout).reason ?? reason
      } catch {}
      throw new Error(reason)
    }
  },
})

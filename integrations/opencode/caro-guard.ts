// caro-guard: OpenCode plugin that runs every bash tool call through
// `caro guard` (spec 011, ADR-018). Shadow mode (default) only logs;
// set CARO_GUARD_MODE=enforce to block Critical/High commands.
//
// Install: cp caro-guard.ts ~/.config/opencode/plugin/
import { spawnSync } from "node:child_process"

export const CaroGuard = async () => ({
  "tool.execute.before": async (input: { tool: string }, output: { args: { command?: string } }) => {
    if (input.tool !== "bash" || !output.args?.command) return

    const mode = process.env.CARO_GUARD_MODE === "enforce" ? "enforce" : "shadow"
    const res = spawnSync("caro", ["guard", "--harness", "opencode", "--mode", mode], {
      input: JSON.stringify({ command: output.args.command, cwd: process.cwd() }),
      encoding: "utf8",
      timeout: 10_000,
    })

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

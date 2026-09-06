// Kitesurf website QA — ADR-017 phase P3 (dormant until CF secrets exist).
//
// Structural checks only: raw-i18n-key leaks, link integrity, DOM presence,
// renders-at-all screenshots. Kitesurf is explicitly NOT pixel-perfect, so
// brand-fidelity/visual work stays with the claude-design Chromium flow
// (.claude/rules/design-dialogue-protocol.md).
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  // Kitesurf trades wall-time for cost (~1.8× slower than Chromium).
  timeout: 60_000,
  expect: { timeout: 15_000 },
  retries: 1,
  // Beta service with per-account limits — keep concurrency polite.
  workers: 2,
  fullyParallel: false,
  reporter: [["line"]],
  use: {
    baseURL: process.env.CARO_QA_BASE_URL ?? "https://caro.sh",
  },
});

// Mirrored from website/src/ui/tokens.css. Keep in sync if the site rebrands.
// This file intentionally has no imports — Remotion's build is isolated from
// the Astro website's build.
//
// Refreshed 2026-08 to match the paper-and-ink brand rebrand:
// - accent: orange #ff8c42 → signal red #ef3333 (--caro-red-500)
// - prompt: teal #4ec9b0 → highlighter yellow #fcfc62 (--caro-yellow-400)
// - terminal bg/fg updated to retro console grey/beige palette
// - danger: aligned to --status-danger (#ef3333)

export const colors = {
  bgDeep: "#1a1a1a",        // --caro-grey-950
  bgTerminal: "#2b2b2b",    // --caro-grey-900 (dark terminal body)
  bgChrome: "#1a1a1a",      // --caro-grey-950 (title bar)
  borderSubtle: "#3a3a3a",  // --caro-grey-800
  accent: "#ef3333",        // --caro-red-500 (signal red, replaces orange)
  accentDark: "#e63636",    // --caro-red-600
  accentSoft: "rgba(239, 51, 51, 0.12)",
  prompt: "#fcfc62",        // --caro-yellow-400 (highlighter yellow, new term-prompt)
  command: "#f4f1df",       // --caro-beige-100 (term-fg)
  success: "#22c55e",       // --color-success (unchanged)
  danger: "#ef3333",        // --status-danger (signal red)
  warning: "#f59e0b",       // --color-warning (unchanged)
  textPrimary: "#f4f1df",   // --caro-beige-100 (retro beige on dark bg)
  textMuted: "#a0a0a0",     // --caro-grey-400
  textDim: "#7a7a7a",       // --caro-grey-500
  trafficClose: "#ff5f56",
  trafficMin: "#ffbd2e",
  trafficMax: "#27c93f",
} as const;

export const fonts = {
  sans: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',
  mono: '"SF Mono", Monaco, "Cascadia Code", "Fira Code", "Consolas", monospace',
} as const;

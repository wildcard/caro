// Mirrored from website/src/ui/tokens.css. Keep in sync if the site rebrands.
// This file intentionally has no imports — Remotion's build is isolated from
// the Astro website's build.
//
// Updated 2026-07-01: brand refresh — orange accent replaced by signal red
// (#ef3333), teal prompt replaced by highlighter yellow (#fcfc62), dark-blue
// terminal backgrounds replaced by retro-console grey palette.
// Source: --caro-* primitives + --term-* semantic tokens in tokens.css.

export const colors = {
  bgDeep: "#1a1a1a",       // --caro-grey-950 (was #0a0a0f dark blue)
  bgTerminal: "#2b2b2b",   // --caro-grey-900 (was #1a1a2e dark blue)
  bgChrome: "#3a3a3a",     // --caro-grey-800 (was #16161c)
  borderSubtle: "#3a3a3a", // --caro-grey-800 (was #1e1e24)
  accent: "#ef3333",       // --caro-red-500, signal red (was #ff8c42 orange)
  accentDark: "#e63636",   // --caro-red-600 (was #ff6b35)
  accentSoft: "rgba(239, 51, 51, 0.12)", // signal-red soft overlay
  prompt: "#fcfc62",       // --caro-yellow-400, highlighter yellow (was #4ec9b0 teal)
  command: "#f4f1df",      // --term-fg / --caro-beige-100 (was #9cdcfe VS-blue)
  success: "#22c55e",      // --color-success (unchanged)
  danger: "#ef3333",       // --status-danger = --caro-red-500 (was #ef4444)
  warning: "#f59e0b",      // --color-warning (unchanged)
  textPrimary: "#f4f1df",  // --term-fg / --caro-beige-100 (was #e0e0e0)
  textMuted: "#a0a0a0",    // --caro-grey-400 (unchanged)
  textDim: "#7a7a7a",      // --caro-grey-500 (was #6c6c76)
  trafficClose: "#ff5f56",
  trafficMin: "#ffbd2e",
  trafficMax: "#27c93f",
} as const;

export const fonts = {
  sans: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',
  mono: '"SF Mono", Monaco, "Cascadia Code", "Fira Code", "Consolas", monospace',
} as const;

// Mirrored from website/src/ui/tokens.css. Keep in sync if the site rebrands.
// This file intentionally has no imports — Remotion's build is isolated from
// the Astro website's build.
// Last synced: 2026-06-01 — brand migrated from orange (#ff8c42) to signal red (#ef3333);
// terminal prompt migrated from teal (#4ec9b0) to highlighter yellow (#fcfc62).

export const colors = {
  bgDeep: "#0a0a0f",
  bgTerminal: "#1a1a2e",
  bgChrome: "#16161c",
  borderSubtle: "#1e1e24",
  accent: "#ef3333",      // --caro-red-500 (was #ff8c42 orange)
  accentDark: "#e63636",  // --caro-red-600 (was #ff6b35)
  accentSoft: "rgba(239, 51, 51, 0.12)",
  prompt: "#fcfc62",      // --caro-yellow-400 (was #4ec9b0 teal)
  command: "#9cdcfe",
  success: "#22c55e",
  danger: "#ef4444",
  warning: "#f59e0b",
  textPrimary: "#e0e0e0",
  textMuted: "#a0a0a0",
  textDim: "#6c6c76",
  trafficClose: "#ff5f56",
  trafficMin: "#ffbd2e",
  trafficMax: "#27c93f",
} as const;

export const fonts = {
  sans: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',
  mono: '"SF Mono", Monaco, "Cascadia Code", "Fira Code", "Consolas", monospace',
} as const;

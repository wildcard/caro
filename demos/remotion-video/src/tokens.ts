// Mirrored from website/src/ui/tokens.css. Keep in sync if the site rebrands.
// This file intentionally has no imports — Remotion's build is isolated from
// the Astro website's build.

export const colors = {
  bgDeep: "#0a0a0f",
  bgTerminal: "#1a1a2e",
  bgChrome: "#16161c",
  borderSubtle: "#1e1e24",
  accent: "#ff8c42",
  accentDark: "#ff6b35",
  accentSoft: "rgba(255, 107, 53, 0.12)",
  prompt: "#4ec9b0",
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

/**
 * WCAG AA contrast regression guard for the LPMoments `.with-caro` badge.
 *
 * The badge's background is the theme-FLIPPING token `var(--bg-inverse)`
 * (grey-700 dark-panel in light mode, beige-100 light-paper in dark mode),
 * so its text color MUST flip too. The old `var(--accent)` (signal red)
 * failed AA on both flipped panels (2.0:1 light / 2.6:1 dark); the fix uses
 * `var(--fg-inverse)`, the flipping partner (7.2:1 light / 12.5:1 dark).
 *
 * This test fails if:
 *   1. `.with-caro` is reverted to a non-flipping / red color, OR
 *   2. the underlying token values drift below the 4.5:1 AA floor.
 *
 * See caro-czkx for the sibling `.section-subtitle` / `.moment-description`
 * grey-300 issue (out of scope here — needs a design token decision).
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';

const tokensCss = readFileSync(new URL('../../ui/tokens.css', import.meta.url), 'utf8');
const componentSrc = readFileSync(new URL('./LPMoments.astro', import.meta.url), 'utf8');

// --- WCAG 2.x relative luminance + contrast ratio ---
function srgbToLinear(channel: number): number {
  const s = channel / 255;
  return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}
function luminance(hex: string): number {
  const h = hex.replace('#', '');
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
  return 0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
}
function contrast(fg: string, bg: string): number {
  const [hi, lo] = [luminance(fg), luminance(bg)].sort((a, b) => b - a);
  return (hi + 0.05) / (lo + 0.05);
}

// --- Resolve tokens.css: primitive (--caro-*: #hex) and semantic (--x: var(--caro-*)) ---
function primitiveHex(name: string): string {
  const m = tokensCss.match(new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{6})`));
  if (!m) throw new Error(`primitive token --${name} not found in tokens.css`);
  return m[1];
}
function blockOf(selectorRe: RegExp): string {
  const m = selectorRe.exec(tokensCss);
  if (!m) throw new Error(`selector ${selectorRe} not found in tokens.css`);
  const start = tokensCss.indexOf('{', m.index);
  let depth = 0;
  for (let i = start; i < tokensCss.length; i++) {
    if (tokensCss[i] === '{') depth++;
    else if (tokensCss[i] === '}' && --depth === 0) return tokensCss.slice(start + 1, i);
  }
  throw new Error(`unbalanced braces for ${selectorRe}`);
}
/** Resolve a semantic token (value is `var(--caro-*)` or a direct #hex) within a theme block. */
function resolveSemantic(block: string, name: string): string {
  const viaVar = block.match(new RegExp(`--${name}:\\s*var\\(--([a-z0-9-]+)\\)`));
  if (viaVar) return primitiveHex(viaVar[1]);
  const direct = block.match(new RegExp(`--${name}:\\s*(#[0-9a-fA-F]{6})`));
  if (direct) return direct[1];
  throw new Error(`semantic token --${name} not resolvable in the given theme block`);
}

const lightRoot = blockOf(/:root\s*\{/);
const darkRoot = blockOf(/\.dark\s*\{/);
const pair = {
  light: { fg: resolveSemantic(lightRoot, 'fg-inverse'), bg: resolveSemantic(lightRoot, 'bg-inverse') },
  dark: { fg: resolveSemantic(darkRoot, 'fg-inverse'), bg: resolveSemantic(darkRoot, 'bg-inverse') },
};

describe('LPMoments .with-caro badge — WCAG AA contrast guard', () => {
  const rule = componentSrc.match(/\.with-caro\s*\{([^}]*)\}/)?.[1] ?? '';

  it('sits on the flipping --bg-inverse and uses the flipping --fg-inverse text token', () => {
    expect(rule).toMatch(/background:\s*var\(--bg-inverse\)/);
    expect(rule).toMatch(/color:\s*var\(--fg-inverse\)/);
  });

  it('does not regress to the old failing red (color: var(--accent))', () => {
    expect(rule).not.toMatch(/color:\s*var\(--accent\)/);
  });

  it('resolves to >=4.5:1 (AA normal text) in light mode', () => {
    expect(contrast(pair.light.fg, pair.light.bg)).toBeGreaterThanOrEqual(4.5);
  });

  it('resolves to >=4.5:1 (AA normal text) in dark mode', () => {
    expect(contrast(pair.dark.fg, pair.dark.bg)).toBeGreaterThanOrEqual(4.5);
  });
});

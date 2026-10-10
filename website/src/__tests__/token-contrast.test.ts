/**
 * Regression guard for the brand token layer (src/ui/tokens.css).
 *
 * Resolves the semantic tokens for light and dark mode straight from the
 * stylesheet and checks the text/background pairs components rely on.
 * If someone "fixes" a token back to a value that fails WCAG AA, this
 * test names the pair and the ratio.
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { contrast } from '../lib/color';

const css = readFileSync(fileURLToPath(new URL('../ui/tokens.css', import.meta.url)), 'utf8')
  .replace(/\/\*[\s\S]*?\*\//g, '');

function block(selector: string): Record<string, string> {
  const re = new RegExp(`(?:^|\\n)${selector.replace('.', '\\.')}\\s*\\{([\\s\\S]*?)\\n\\}`);
  const body = css.match(re)?.[1] ?? '';
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) out[m[1]] = m[2].trim();
  return out;
}

const root = block(':root');
const dark = { ...root, ...block('.dark') };

function resolve(vars: Record<string, string>, name: string, depth = 0): string {
  const v = vars[name];
  if (!v || depth > 10) throw new Error(`unresolved ${name}`);
  const ref = v.match(/^var\((--[\w-]+)\)$/);
  return ref ? resolve(vars, ref[1], depth + 1) : v.toLowerCase();
}

// [foreground, background, minimum ratio]
const PAIRS: Array<[string, string, number]> = [
  ['--fg', '--bg', 4.5],
  ['--fg', '--bg-raised', 4.5],
  ['--fg-strong', '--bg', 4.5],
  ['--fg-muted', '--bg', 4.5],
  ['--fg-muted', '--bg-raised', 4.5],
  ['--fg-muted', '--bg-subtle', 4.5],
  ['--fg-subtle', '--bg', 4.5],
  ['--accent-text', '--bg', 4.5],
  ['--accent-text', '--bg-raised', 4.5],
  ['--on-accent', '--accent-fill', 4.5],
  ['--on-accent', '--accent-fill-hover', 4.5],
  ['--fg-inverse', '--bg-inverse', 4.5],
  ['--highlight', '--bg-inverse', 4.5],
  ['--term-fg', '--term-bg', 4.5],
  ['--status-safe-text', '--bg', 4.5],
  ['--fg', '--bg-stage', 4.5],
];

describe.each([
  ['light', root],
  ['dark', dark],
] as const)('%s mode token pairs', (_mode, vars) => {
  it.each(PAIRS)('%s on %s', (fg, bg, min) => {
    const ratio = contrast(resolve(vars, fg), resolve(vars, bg));
    expect(ratio, `${fg} on ${bg} = ${ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(min);
  });
});

describe('console surface is stable across modes', () => {
  it('does not flip --bg-inverse in dark mode', () => {
    expect(resolve(dark, '--bg-inverse')).toBe(resolve(root, '--bg-inverse'));
  });
});

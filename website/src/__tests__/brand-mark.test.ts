/**
 * Regression guard for the brand mark in the navigation (docs/brand/ICONOGRAPHY.md).
 *
 * Every site navigation shows the official vector mark through <CaroMark>,
 * which inlines public/brand/caro-mark.svg in the text color. The `>_`
 * terminal-glyph stand-in and the raster caro-pixel.png must not come back.
 */
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));
const site = join(src, '..');

const navs = [
  'components/landing/LPNavigation.astro',
  'components/Navigation.astro',
  'layouts/DocsLayout.astro',
];

describe('brand mark', () => {
  it('ships the vector mark and the full logo', () => {
    for (const name of ['caro-mark.svg', 'caro-logo.svg']) {
      const svg = readFileSync(join(site, 'public/brand', name), 'utf8');
      expect(svg).toMatch(/<svg [^>]*viewBox="0 0 \d+ \d+"/);
      // currentColor lets the mark reverse out in dark mode.
      expect(svg).toContain('fill="currentColor"');
      expect(svg).not.toMatch(/<image\b|href=/);
    }
  });

  it.each(navs)('%s shows <CaroMark>', (file) => {
    const text = readFileSync(join(src, file), 'utf8');
    expect(text).toMatch(/<CaroMark\b/);
    expect(text).not.toMatch(/brand-prompt|caro-pixel\.png/);
  });

  it('keeps the component reading the public SVG', () => {
    const component = join(src, 'components/icons/CaroMark.astro');
    expect(existsSync(component)).toBe(true);
    expect(readFileSync(component, 'utf8')).toContain('public/brand/caro-mark.svg?raw');
  });
});

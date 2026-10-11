/**
 * Regression guard for the Kyaro icon set (docs/brand/ICONOGRAPHY.md).
 *
 * scripts/kyaro_icons.py --check proves each icon is a 1:1 cut of a Kyaro
 * frame. This test guards how the site uses them:
 * - every manifest entry points at an SVG of its native size
 * - every <KyaroIcon> names a real icon at a whole-number scale
 * - the hand-drawn v2.1 glyph pack (Glyph, public/icons/, kyaro-mark) stays gone
 */
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import manifest from '../data/kyaro-icons.json';

const src = fileURLToPath(new URL('..', import.meta.url));
const site = join(src, '..');

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(astro|tsx?|jsx|mdx?)$/.test(name) && !name.endsWith('.test.ts') ? [path] : [];
  });
}

const files = sourceFiles(src).map((path) => ({
  file: path.slice(src.length),
  text: readFileSync(path, 'utf8'),
}));
const icons: Record<string, { src: string; size: number }> = manifest.icons;

describe('Kyaro icon manifest', () => {
  it.each(Object.entries(icons))('%s is an SVG of its native size', (_name, icon) => {
    expect([24, 48]).toContain(icon.size);
    const svg = readFileSync(join(site, 'public', icon.src), 'utf8');
    expect(svg).toContain(`viewBox="0 0 ${icon.size} ${icon.size}"`);
    expect(svg).toContain('shape-rendering="crispEdges"');
  });
});

describe('<KyaroIcon> component', () => {
  const component = readFileSync(join(src, 'ui/KyaroIcon/KyaroIcon.astro'), 'utf8');

  // The site resets to border-box. Under it, the tile padding would shrink
  // the 48 px art to 40 px and blur every icon.
  it('keeps the art box at native size under the tile padding', () => {
    expect(component).toMatch(/box-sizing:\s*content-box/);
  });

  it('puts every icon on the fixed stage-grey tile in both themes', () => {
    expect(component).toMatch(/background:\s*var\(--caro-grey-200\)/);
  });
});

describe('<KyaroIcon> usage', () => {
  const uses = files.flatMap(({ file, text }) =>
    [...text.matchAll(/<KyaroIcon\b([^>]*?)\/>/g)].map((m) => ({ file, attrs: m[1] })),
  );

  it('is used on the site', () => {
    expect(uses.length).toBeGreaterThan(0);
  });

  it.each(uses)('$file: names a real icon', ({ attrs }) => {
    const name = attrs.match(/\bname="([\w-]+)"/)?.[1];
    if (name) expect(Object.keys(icons)).toContain(name);
  });

  it.each(uses)('$file: scales by a whole number', ({ attrs }) => {
    expect(attrs).not.toMatch(/\b(size|width|height)=/);
    const scale = attrs.match(/\bscale=\{([^}]*)\}/)?.[1];
    if (scale !== undefined) expect(scale.trim()).toMatch(/^[123]$/);
  });

  it('every icon name in a typed list exists', () => {
    const importers = files.filter(({ text }) => text.includes('ui/KyaroIcon/KyaroIcon.astro'));
    const listed = importers.flatMap(({ text }) =>
      [...text.matchAll(/\bicon:\s*'([\w-]+)'|KyaroIconName\[\]\s*=\s*\[([^\]]*)\]/g)].flatMap((m) =>
        m[1] ? [m[1]] : [...m[2].matchAll(/'([\w-]+)'/g)].map((n) => n[1]),
      ),
    );
    expect(listed.length).toBeGreaterThan(0);
    for (const name of listed) expect(Object.keys(icons)).toContain(name);
  });
});

describe('the v2.1 glyph pack stays removed', () => {
  it('has no files', () => {
    expect(existsSync(join(site, 'public/icons'))).toBe(false);
    expect(existsSync(join(src, 'ui/Glyph'))).toBe(false);
    expect(existsSync(join(src, 'data/icon-manifest.json'))).toBe(false);
  });

  it('is not referenced from the source', () => {
    const legacy = /ui\/Glyph\/|<Glyph\b|icon-manifest\.json|kyaro-mark|(?<!kyaro)\/icons\/[\w-]+\.svg/;
    expect(files.filter(({ text }) => legacy.test(text)).map(({ file }) => file)).toEqual([]);
  });
});

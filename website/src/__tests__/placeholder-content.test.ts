/**
 * Regression guard: invented content must not reach a rendered page.
 *
 * A component whose source contains the marker `PLACEHOLDER CONTENT` holds
 * made-up material (for example, testimonial quotes and names that no real
 * person gave). No page, layout or other component may import it. To show
 * the component again, replace the content with real, consented material
 * and delete the marker.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { basename, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));
const MARKER = 'PLACEHOLDER CONTENT';

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(astro|ts|tsx|mdx)$/.test(name) ? [path] : [];
  });
}

const files = ['pages', 'layouts', 'components'].flatMap((dir) => sourceFiles(join(src, dir)));
// Static (`from '…/Name.astro'`) and dynamic (`import('…/Name.astro')`) imports.
const importsOf = (name: string) => new RegExp(`(?:from\\s+|import\\s*\\(\\s*)(['"])[^'"]*/${name}\\.astro\\1`);

describe('placeholder content', () => {
  const placeholders = files
    .filter((file) => file.endsWith('.astro') && readFileSync(file, 'utf8').includes(MARKER))
    .map((file) => ({ name: basename(file, '.astro'), file }));

  it('detects component imports (scanner self-check)', () => {
    const index = readFileSync(join(src, 'pages/index.astro'), 'utf8');
    expect(importsOf('LPHero').test(index)).toBe(true);
  });

  it.each(placeholders)('no file imports the placeholder component $name', ({ name, file: placeholder }) => {
    const importers = files
      .filter((file) => file !== placeholder && importsOf(name).test(readFileSync(file, 'utf8')))
      .map((file) => file.slice(src.length));
    expect(importers, `${name} is marked ${MARKER}`).toEqual([]);
  });
});

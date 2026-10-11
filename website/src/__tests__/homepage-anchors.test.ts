/**
 * Regression guard for links into the English homepage (`/#section`).
 *
 * Collects every literal `href="/#id"` in components and layouts and checks
 * that the homepage (src/pages/index.astro and the components it imports)
 * declares `id="id"`. A link whose section was removed or renamed lands at
 * the top of the page instead of the section, which reads as a broken link.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));
const homepage = join(src, 'pages/index.astro');

function astroFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return astroFiles(path);
    return path.endsWith('.astro') ? [path] : [];
  });
}

function homepageIds(): Set<string> {
  const page = readFileSync(homepage, 'utf8');
  const files = [homepage];
  for (const m of page.matchAll(/import\s+\w+\s+from\s+'(\.[^']+\.astro)'/g)) {
    files.push(resolve(dirname(homepage), m[1]));
  }
  const ids = new Set<string>();
  for (const file of files) {
    for (const m of readFileSync(file, 'utf8').matchAll(/\bid="([\w-]+)"/g)) ids.add(m[1]);
  }
  return ids;
}

describe('links into the English homepage', () => {
  const ids = homepageIds();
  const links = ['components', 'layouts'].flatMap((dir) =>
    astroFiles(join(src, dir)).flatMap((file) =>
      [...readFileSync(file, 'utf8').matchAll(/href="\/#([\w-]+)"/g)].map((m) => ({
        file: file.slice(src.length),
        id: m[1],
      })),
    ),
  );

  it('finds links to check', () => {
    expect(links.length).toBeGreaterThan(0);
  });

  it.each(links)('$file links to /#$id, which the homepage declares', ({ id }) => {
    expect(ids.has(id)).toBe(true);
  });
});

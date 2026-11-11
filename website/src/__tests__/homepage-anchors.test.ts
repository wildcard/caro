/**
 * Regression guard for links into a homepage section (`/#section`).
 *
 * Scans the .astro files in components/ and layouts/ for three link forms
 * and checks the section id against the homepage it can land on:
 * - `href="/#id"`: the English homepage (src/pages/index.astro)
 * - `${home}#id`: both the English and the localized homepage
 *   (src/pages/[lang]/index.astro), because `home` is `/` or `/<lang>/`
 * - `demoAnchor = lang === 'en' ? '#a' : '#b'`: `a` on English, `b` localized
 *
 * A homepage "declares" an id when the page or a component it imports has
 * `id="id"`. A link whose section was removed or renamed lands at the top of
 * the page instead of the section, which reads as a broken link.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));

function astroFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return astroFiles(path);
    return path.endsWith('.astro') ? [path] : [];
  });
}

function declaredIds(page: string): Set<string> {
  const files = [page];
  for (const m of readFileSync(page, 'utf8').matchAll(/import\s+\w+\s+from\s+'(\.[^']+\.astro)'/g)) {
    files.push(resolve(dirname(page), m[1]));
  }
  const ids = new Set<string>();
  for (const file of files) {
    for (const m of readFileSync(file, 'utf8').matchAll(/\bid="([\w-]+)"/g)) ids.add(m[1]);
  }
  return ids;
}

const english = declaredIds(join(src, 'pages/index.astro'));
const localized = declaredIds(join(src, 'pages/[lang]/index.astro'));

type Link = { file: string; id: string; homepages: Array<'english' | 'localized'> };

const links: Link[] = ['components', 'layouts'].flatMap((dir) =>
  astroFiles(join(src, dir)).flatMap((path) => {
    const file = path.slice(src.length);
    const text = readFileSync(path, 'utf8');
    const found: Link[] = [];
    for (const m of text.matchAll(/href="\/#([\w-]+)"/g)) {
      found.push({ file, id: m[1], homepages: ['english'] });
    }
    for (const m of text.matchAll(/\$\{home\}#([\w-]+)/g)) {
      found.push({ file, id: m[1], homepages: ['english', 'localized'] });
    }
    for (const m of text.matchAll(/demoAnchor = lang === 'en' \? '#([\w-]+)' : '#([\w-]+)'/g)) {
      found.push({ file, id: m[1], homepages: ['english'] });
      found.push({ file, id: m[2], homepages: ['localized'] });
    }
    return found;
  }),
);

describe('links into a homepage section', () => {
  it('finds links of every form to check', () => {
    expect(links.some((l) => l.homepages.length === 2)).toBe(true);
    expect(links.filter((l) => l.file.endsWith('Footer.astro')).length).toBeGreaterThanOrEqual(4);
  });

  it.each(links)('$file links to #$id on the $homepages homepage', ({ id, homepages }) => {
    for (const homepage of homepages) {
      const ids = homepage === 'english' ? english : localized;
      expect(ids.has(id), `${homepage} homepage has no id="${id}"`).toBe(true);
    }
  });
});

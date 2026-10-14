/**
 * Every page gets its share card, Markdown twin and full SEO head only
 * through src/components/SEO.astro. A layout that writes its own <html>
 * without it would ship pages with none of that, so each one must use it.
 *
 * (The post-build step, src/integrations/agent-seo.ts, also fails the build
 * on any page that misses them; this test catches it before a build.)
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));

function astroFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const file = path.join(dir, name);
    if (statSync(file).isDirectory()) return astroFiles(file);
    return file.endsWith('.astro') ? [file] : [];
  });
}

const roots = [...astroFiles(path.join(src, 'layouts')), ...astroFiles(path.join(src, 'pages'))].filter(
  (file) => /<html[\s>]/.test(readFileSync(file, 'utf8')),
);

describe('SEO coverage', () => {
  it('finds the layouts that render <html>', () => {
    expect(roots.length).toBeGreaterThan(0);
  });

  it.each(roots.map((file) => [path.relative(src, file), file]))('%s renders <SEO>', (_, file) => {
    const source = readFileSync(file, 'utf8');
    expect(source).toMatch(/import SEO from ['"][./]+components\/SEO\.astro['"]/);
    expect(source).toMatch(/<SEO[\s\n]/);
    // SEO owns these; a second copy would contradict it.
    expect(source).not.toMatch(/<title>/);
    expect(source).not.toMatch(/property="og:/);
    expect(source).not.toMatch(/hreflang=/);
  });
});

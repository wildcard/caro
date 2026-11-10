/**
 * Regression guard for #1564: OmniSearch must not ship the full search index
 * in every page's HTML. It fetches /search-index.json on first open, reuses
 * the result, and falls back to the compact PAGES_INDEX when the fetch fails.
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it, vi } from 'vitest';
import { createSearchIndexLoader, prepareSearchPages } from './search-index-loader';

const compact = [{ title: 'Home', description: 'Start', path: '/', keywords: ['caro'] }];
const full = {
  pages: [
    { title: 'Docs', description: 'Guide', path: '/docs', keywords: [], _searchText: 'docs guide full body' },
  ],
};

function setup(fetchIndex: () => Promise<unknown>) {
  const spy = vi.fn(fetchIndex);
  const loader = createSearchIndexLoader({ fetchIndex: spy, fallback: compact });
  return { spy, loader };
}

describe('createSearchIndexLoader', () => {
  it('does not fetch until load() is called', () => {
    const { spy } = setup(async () => full);
    expect(spy).not.toHaveBeenCalled();
  });

  it('fetches once on first open and reuses the index after reopening', async () => {
    const { spy, loader } = setup(async () => full);
    const first = await loader.load();
    const second = await loader.load();
    expect(spy).toHaveBeenCalledTimes(1);
    expect(second).toBe(first);
    expect(first[0]._search).toBe('docs guide full body');
  });

  it('shares one request between concurrent opens', async () => {
    const { spy, loader } = setup(async () => full);
    await Promise.all([loader.load(), loader.load()]);
    expect(spy).toHaveBeenCalledTimes(1);
  });

  it('falls back to the compact PAGES_INDEX when the fetch fails', async () => {
    const { loader } = setup(async () => {
      throw new Error('offline');
    });
    const pages = await loader.load();
    expect(pages.map((p) => p.path)).toEqual(['/']);
    expect(pages[0]._search).toBe('home start / caro');
  });

  it('falls back when the fetched index has no pages, and retries on the next open', async () => {
    const spy = vi.fn().mockResolvedValueOnce({ pages: [] }).mockResolvedValueOnce(full);
    const loader = createSearchIndexLoader({ fetchIndex: spy, fallback: compact });
    expect((await loader.load())[0].path).toBe('/');
    expect((await loader.load())[0].path).toBe('/docs');
    expect(spy).toHaveBeenCalledTimes(2);
  });

  it('exposes the prepared fallback synchronously for the first render', () => {
    const { loader } = setup(async () => full);
    expect(loader.fallbackPages()[0]._words).toEqual(['home', 'start', 'caro']);
  });
});

describe('prepareSearchPages', () => {
  it('computes _search and _words when the index lacks them', () => {
    const [p] = prepareSearchPages([{ title: 'Big Title', description: 'ab', path: '/x', keywords: ['kw1'] }]);
    expect(p._search).toBe('big title ab /x kw1');
    expect(p._words).toEqual(['big', 'title', 'kw1']);
  });
});

describe('OmniSearch.astro wiring', () => {
  const src = readFileSync(
    fileURLToPath(new URL('../components/OmniSearch.astro', import.meta.url)),
    'utf8',
  );
  const clientScript = src.split('<script>')[1]?.split('</script>')[0] ?? '';
  const staticFallback = readFileSync(
    fileURLToPath(new URL('../config/pages-static.ts', import.meta.url)),
    'utf8',
  );

  it('uses a client-safe fallback without importing the page glob', () => {
    expect(clientScript).toContain("import { STATIC_PAGES_INDEX } from '../config/pages-static';");
    expect(clientScript).not.toContain("from '../config/pages';");
    expect(staticFallback).not.toContain('import.meta.glob');
  });

  it('loads the full index when search opens and rerenders the fallback', () => {
    expect(clientScript).toContain("fetch('/search-index.json')");
    expect(clientScript).toContain('function open(viaHotkey = false) {');
    expect(clientScript).toContain('loadIndex();');
    expect(clientScript).toContain('if (loaded === pages) {');
    expect(clientScript).toContain("if (overlay.classList.contains('open')) render(input.value);");
  });
});

/**
 * Lazy loader for the OmniSearch index (#1564).
 *
 * The full generated index is served as /search-index.json instead of being
 * serialized into every page. The loader fetches it on first use, reuses the
 * result, and falls back to the compact PAGES_INDEX if the fetch fails.
 */

export interface RawSearchPage {
  title: string;
  description?: string;
  path: string;
  keywords?: string[];
  _searchText?: string;
  _words?: string[];
  [key: string]: unknown;
}

export interface SearchPage extends RawSearchPage {
  _search: string;
  _words: string[];
}

export function prepareSearchPages(pages: RawSearchPage[]): SearchPage[] {
  return pages.map((p) => ({
    ...p,
    // Pre-computed _searchText includes ALL page content when present.
    _search:
      p._searchText ||
      [p.title, p.description || '', p.path, ...(p.keywords || [])].join(' ').toLowerCase(),
    _words:
      p._words ||
      [p.title, p.description || '', ...(p.keywords || [])]
        .join(' ')
        .toLowerCase()
        .split(/\s+/)
        .filter((w) => w.length > 2),
  }));
}

interface LoaderOptions {
  fetchIndex: () => Promise<unknown>;
  fallback: RawSearchPage[];
}

export function createSearchIndexLoader({ fetchIndex, fallback }: LoaderOptions) {
  const fallbackPages = prepareSearchPages(fallback);
  let loaded: SearchPage[] | null = null;
  let inflight: Promise<SearchPage[]> | null = null;

  async function fetchOnce(): Promise<SearchPage[]> {
    try {
      const data = (await fetchIndex()) as { pages?: RawSearchPage[] } | null;
      if (data?.pages && data.pages.length > 0) {
        loaded = prepareSearchPages(data.pages);
        return loaded;
      }
    } catch (error) {
      console.warn('Failed to load full search index; using compact fallback.', error);
    }
    return fallbackPages;
  }

  return {
    fallbackPages: () => fallbackPages,
    /** Fetches on first call; later calls reuse the result. Failures are not cached. */
    load(): Promise<SearchPage[]> {
      if (loaded) return Promise.resolve(loaded);
      inflight ??= fetchOnce().finally(() => {
        inflight = null;
      });
      return inflight;
    },
  };
}

/**
 * Site pages index for OmniSearch navigation
 *
 * IMPORTANT: When adding new pages to the website, ensure they are added to this index.
 * The OmniSearch feature (Cmd+/ or Ctrl+/) uses this index to provide site-wide search.
 *
 * Each page entry should include:
 * - title: Display name of the page
 * - path: URL path (e.g., '/blog/my-post')
 * - description: Short description for search results
 * - category: One of 'main', 'use-cases', 'compare', 'blog', 'docs'
 * - keywords: Array of search terms
 * - icon: Terminal glyph for the search result (▸; see ui/TermGlyph)
 *
 * @see scripts/generate-search-index.mjs for automated index generation
 */

import { STATIC_PAGES_INDEX } from './pages-static';

export interface PageEntry {
  title: string;
  path: string;
  description?: string;
  category: 'main' | 'use-cases' | 'compare' | 'blog' | 'docs';
  keywords?: string[];
  icon?: string;
}

export interface SearchMeta {
  title: string;
  description: string;
  keywords?: string[];
  icon?: string;
  category?: PageEntry['category'];
}

// Import searchMeta from pages at build time
// Pages can export: export const searchMeta = { title, description, keywords, icon }
const pageModules = import.meta.glob<{ searchMeta?: SearchMeta }>(
  '../pages/**/*.astro',
  { eager: true }
);

// Extract path from module path (e.g., '../pages/support.astro' -> '/support')
function modulePathToRoute(modulePath: string): string {
  return modulePath
    .replace('../pages', '')
    .replace('/index.astro', '/')
    .replace('.astro', '')
    .replace(/\/+$/, '') || '/';
}

// Infer category from path
function inferCategory(path: string): PageEntry['category'] {
  if (path.startsWith('/use-cases')) return 'use-cases';
  if (path.startsWith('/compare')) return 'compare';
  if (path.startsWith('/blog')) return 'blog';
  if (path.startsWith('/docs')) return 'docs';
  return 'main';
}

// Build page entries from modules, with fallback defaults
function buildPageEntries(): Map<string, Partial<PageEntry>> {
  const entries = new Map<string, Partial<PageEntry>>();

  for (const [modulePath, module] of Object.entries(pageModules)) {
    const path = modulePathToRoute(modulePath);
    const meta = module.searchMeta;

    if (meta) {
      entries.set(path, {
        title: meta.title,
        description: meta.description,
        keywords: meta.keywords,
        icon: meta.icon,
        category: meta.category || inferCategory(path),
      });
    }
  }

  return entries;
}

// Page entries sourced from searchMeta exports
const dynamicEntries = buildPageEntries();

// Helper to get entry from dynamic source or use fallback
function getEntry(
  path: string,
  fallback: Omit<PageEntry, 'path'>
): PageEntry {
  const dynamic = dynamicEntries.get(path);
  if (dynamic) {
    return {
      path,
      title: dynamic.title || fallback.title,
      description: dynamic.description || fallback.description,
      category: dynamic.category || fallback.category,
      keywords: dynamic.keywords || fallback.keywords,
      icon: dynamic.icon || fallback.icon,
    };
  }
  return { path, ...fallback };
}

export const PAGES_INDEX: PageEntry[] = STATIC_PAGES_INDEX.map(({ path, ...fallback }) => getEntry(path, fallback));

/**
 * Search pages by query string
 */
export function searchPages(query: string): PageEntry[] {
  const normalizedQuery = query.toLowerCase().trim();

  if (!normalizedQuery) {
    return PAGES_INDEX;
  }

  return PAGES_INDEX.filter((page) => {
    const titleMatch = page.title.toLowerCase().includes(normalizedQuery);
    const descMatch = page.description?.toLowerCase().includes(normalizedQuery);
    const pathMatch = page.path.toLowerCase().includes(normalizedQuery);
    const keywordMatch = page.keywords?.some((k) =>
      k.toLowerCase().includes(normalizedQuery)
    );

    return titleMatch || descMatch || pathMatch || keywordMatch;
  }).sort((a, b) => {
    // Prioritize exact title matches
    const aExact = a.title.toLowerCase() === normalizedQuery;
    const bExact = b.title.toLowerCase() === normalizedQuery;
    if (aExact && !bExact) return -1;
    if (bExact && !aExact) return 1;

    // Then prioritize title starts with
    const aStarts = a.title.toLowerCase().startsWith(normalizedQuery);
    const bStarts = b.title.toLowerCase().startsWith(normalizedQuery);
    if (aStarts && !bStarts) return -1;
    if (bStarts && !aStarts) return 1;

    return 0;
  });
}

/**
 * Get pages by category
 */
export function getPagesByCategory(category: PageEntry['category']): PageEntry[] {
  return PAGES_INDEX.filter((page) => page.category === category);
}

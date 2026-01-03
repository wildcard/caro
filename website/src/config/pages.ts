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
 * - icon: Emoji icon for the search result
 *
 * @see scripts/generate-search-index.mjs for automated index generation
 */

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

export const PAGES_INDEX: PageEntry[] = [
  // Main pages
  getEntry('/', {
    title: 'Home',
    description: 'Caro - Your loyal shell companion',
    category: 'main',
    keywords: ['home', 'main', 'landing'],
    icon: '🏠',
  }),
  getEntry('/safe-shell-commands', {
    title: 'Safe Shell Commands',
    description: 'Generate safe shell commands with AI',
    category: 'main',
    keywords: ['safe', 'shell', 'commands', 'cli'],
    icon: '🛡️',
  }),
  getEntry('/ai-command-safety', {
    title: 'AI Command Safety',
    description: 'How Caro validates AI-generated commands',
    category: 'main',
    keywords: ['ai', 'safety', 'validation', 'llm'],
    icon: '🔒',
  }),
  getEntry('/ai-agent-safety', {
    title: 'AI Agent Safety',
    description: 'MCP integration for AI agents',
    category: 'main',
    keywords: ['ai', 'agent', 'mcp', 'claude'],
    icon: '🤖',
  }),
  getEntry('/roadmap', {
    title: 'Roadmap',
    description: 'Product roadmap and upcoming features',
    category: 'main',
    keywords: ['roadmap', 'features', 'planned', 'future'],
    icon: '🗺️',
  }),
  getEntry('/support', {
    title: 'Sponsor',
    description: 'Fund Caro open source development',
    category: 'main',
    keywords: ['sponsor', 'donate', 'fund', 'github sponsors', 'open collective', 'support development'],
    icon: '💖',
  }),
  getEntry('/credits', {
    title: 'Credits',
    description: 'Acknowledgments and credits',
    category: 'main',
    keywords: ['credits', 'thanks', 'acknowledgments'],
    icon: '🙏',
  }),
  getEntry('/explore', {
    title: 'Explore',
    description: 'Explore Caro features',
    category: 'main',
    keywords: ['explore', 'discover', 'features'],
    icon: '🔍',
  }),
  getEntry('/faq', {
    title: 'FAQ',
    description: 'Frequently asked questions about Caro',
    category: 'main',
    keywords: ['faq', 'questions', 'help', 'support', 'how to', 'troubleshooting', 'installation', 'usage', 'safety', 'backends'],
    icon: '❓',
  }),
  getEntry('/telemetry', {
    title: 'Telemetry',
    description: 'Privacy and telemetry information',
    category: 'main',
    keywords: ['telemetry', 'privacy', 'data', 'analytics'],
    icon: '📊',
  }),

  // Use Cases
  getEntry('/use-cases', {
    title: 'Use Cases',
    description: 'All use cases and personas',
    category: 'use-cases',
    keywords: ['use cases', 'personas', 'jtbd', 'jobs'],
    icon: '📋',
  }),
  getEntry('/use-cases/sre', {
    title: 'SRE & On-Call',
    description: 'For Site Reliability Engineers',
    category: 'use-cases',
    keywords: ['sre', 'on-call', 'incident', 'production'],
    icon: '🚨',
  }),
  getEntry('/use-cases/air-gapped', {
    title: 'Air-Gapped Security',
    description: 'For air-gapped and secure environments',
    category: 'use-cases',
    keywords: ['air-gapped', 'security', 'offline', 'scif'],
    icon: '🔐',
  }),
  getEntry('/use-cases/devops', {
    title: 'DevOps & Platform',
    description: 'For DevOps and Platform Engineers',
    category: 'use-cases',
    keywords: ['devops', 'platform', 'cross-platform', 'ci/cd'],
    icon: '🔧',
  }),
  getEntry('/use-cases/tech-lead', {
    title: 'Tech Leads',
    description: 'For Tech Leads and Engineering Managers',
    category: 'use-cases',
    keywords: ['tech lead', 'manager', 'team', 'safety'],
    icon: '👥',
  }),
  getEntry('/use-cases/developer', {
    title: 'Developers',
    description: 'For developers learning the terminal',
    category: 'use-cases',
    keywords: ['developer', 'learning', 'terminal', 'beginner'],
    icon: '💻',
  }),

  // Comparisons
  getEntry('/compare', {
    title: 'Compare Tools',
    description: 'Compare Caro with alternatives',
    category: 'compare',
    keywords: ['compare', 'alternatives', 'vs'],
    icon: '⚖️',
  }),
  getEntry('/compare/github-copilot-cli', {
    title: 'vs GitHub Copilot CLI',
    description: 'Caro vs GitHub Copilot CLI',
    category: 'compare',
    keywords: ['github', 'copilot', 'cli', 'compare'],
    icon: '🐙',
  }),
  getEntry('/compare/warp', {
    title: 'vs Warp AI',
    description: 'Caro vs Warp AI',
    category: 'compare',
    keywords: ['warp', 'terminal', 'compare'],
    icon: '🚀',
  }),
  getEntry('/compare/kiro-cli', {
    title: 'vs Kiro CLI',
    description: 'Caro vs Kiro CLI (formerly Amazon Q CLI)',
    category: 'compare',
    keywords: ['kiro', 'amazon', 'q', 'aws', 'compare'],
    icon: '👻',
  }),
  getEntry('/compare/opencode', {
    title: 'vs OpenCode',
    description: 'Caro vs OpenCode',
    category: 'compare',
    keywords: ['opencode', 'compare'],
    icon: '💡',
  }),

  // Blog
  getEntry('/blog', {
    title: 'Blog',
    description: 'Caro blog and updates',
    category: 'blog',
    keywords: ['blog', 'news', 'updates', 'articles'],
    icon: '📝',
  }),
  getEntry('/blog/announcing-caro', {
    title: 'Announcing Caro',
    description: 'Introducing Caro to the world',
    category: 'blog',
    keywords: ['announcement', 'launch', 'intro'],
    icon: '🎉',
  }),
  getEntry('/blog/why-caro', {
    title: 'Why Caro?',
    description: 'Why we built Caro',
    category: 'blog',
    keywords: ['why', 'motivation', 'story'],
    icon: '❓',
  }),
  getEntry('/blog/security-practices', {
    title: 'Security Practices',
    description: 'Security best practices with Caro',
    category: 'blog',
    keywords: ['security', 'practices', 'safety'],
    icon: '🔐',
  }),
  getEntry('/blog/claude-skill-launch', {
    title: 'Claude Skill Launch',
    description: 'Claude Code skill integration',
    category: 'blog',
    keywords: ['claude', 'skill', 'integration'],
    icon: '🤖',
  }),
  getEntry('/blog/batteries-included', {
    title: 'Batteries Included',
    description: 'Everything included out of the box',
    category: 'blog',
    keywords: ['batteries', 'included', 'features'],
    icon: '🔋',
  }),
];

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

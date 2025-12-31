/**
 * Site pages index for OmniMenu navigation
 * Used by the Cmd+K omni menu to provide quick navigation
 */

export interface PageEntry {
  title: string;
  path: string;
  description?: string;
  category: 'main' | 'use-cases' | 'compare' | 'blog' | 'docs';
  keywords?: string[];
  icon?: string;
}

export const PAGES_INDEX: PageEntry[] = [
  // Main pages
  {
    title: 'Home',
    path: '/',
    description: 'Caro - Your loyal shell companion',
    category: 'main',
    keywords: ['home', 'main', 'landing'],
    icon: '🏠',
  },
  {
    title: 'Safe Shell Commands',
    path: '/safe-shell-commands',
    description: 'Generate safe shell commands with AI',
    category: 'main',
    keywords: ['safe', 'shell', 'commands', 'cli'],
    icon: '🛡️',
  },
  {
    title: 'AI Command Safety',
    path: '/ai-command-safety',
    description: 'How Caro validates AI-generated commands',
    category: 'main',
    keywords: ['ai', 'safety', 'validation', 'llm'],
    icon: '🔒',
  },
  {
    title: 'AI Agent Safety',
    path: '/ai-agent-safety',
    description: 'MCP integration for AI agents',
    category: 'main',
    keywords: ['ai', 'agent', 'mcp', 'claude'],
    icon: '🤖',
  },
  {
    title: 'Roadmap',
    path: '/roadmap',
    description: 'Product roadmap and upcoming features',
    category: 'main',
    keywords: ['roadmap', 'features', 'planned', 'future'],
    icon: '🗺️',
  },
  {
    title: 'Support',
    path: '/support',
    description: 'Get help with Caro',
    category: 'main',
    keywords: ['support', 'help', 'contact'],
    icon: '💬',
  },
  {
    title: 'Credits',
    path: '/credits',
    description: 'Acknowledgments and credits',
    category: 'main',
    keywords: ['credits', 'thanks', 'acknowledgments'],
    icon: '🙏',
  },
  {
    title: 'Explore',
    path: '/explore',
    description: 'Explore Caro features',
    category: 'main',
    keywords: ['explore', 'discover', 'features'],
    icon: '🔍',
  },

  // Use Cases
  {
    title: 'Use Cases',
    path: '/use-cases',
    description: 'All use cases and personas',
    category: 'use-cases',
    keywords: ['use cases', 'personas', 'jtbd', 'jobs'],
    icon: '📋',
  },
  {
    title: 'SRE & On-Call',
    path: '/use-cases/sre',
    description: 'For Site Reliability Engineers',
    category: 'use-cases',
    keywords: ['sre', 'on-call', 'incident', 'production'],
    icon: '🚨',
  },
  {
    title: 'Air-Gapped Security',
    path: '/use-cases/air-gapped',
    description: 'For air-gapped and secure environments',
    category: 'use-cases',
    keywords: ['air-gapped', 'security', 'offline', 'scif'],
    icon: '🔐',
  },
  {
    title: 'DevOps & Platform',
    path: '/use-cases/devops',
    description: 'For DevOps and Platform Engineers',
    category: 'use-cases',
    keywords: ['devops', 'platform', 'cross-platform', 'ci/cd'],
    icon: '🔧',
  },
  {
    title: 'Tech Leads',
    path: '/use-cases/tech-lead',
    description: 'For Tech Leads and Engineering Managers',
    category: 'use-cases',
    keywords: ['tech lead', 'manager', 'team', 'safety'],
    icon: '👥',
  },
  {
    title: 'Developers',
    path: '/use-cases/developer',
    description: 'For developers learning the terminal',
    category: 'use-cases',
    keywords: ['developer', 'learning', 'terminal', 'beginner'],
    icon: '💻',
  },

  // Comparisons
  {
    title: 'Compare Tools',
    path: '/compare',
    description: 'Compare Caro with alternatives',
    category: 'compare',
    keywords: ['compare', 'alternatives', 'vs'],
    icon: '⚖️',
  },
  {
    title: 'vs GitHub Copilot CLI',
    path: '/compare/github-copilot-cli',
    description: 'Caro vs GitHub Copilot CLI',
    category: 'compare',
    keywords: ['github', 'copilot', 'cli', 'compare'],
    icon: '🐙',
  },
  {
    title: 'vs Warp AI',
    path: '/compare/warp',
    description: 'Caro vs Warp AI',
    category: 'compare',
    keywords: ['warp', 'terminal', 'compare'],
    icon: '🚀',
  },
  {
    title: 'vs Amazon Q CLI',
    path: '/compare/amazon-q-cli',
    description: 'Caro vs Amazon Q CLI',
    category: 'compare',
    keywords: ['amazon', 'q', 'aws', 'compare'],
    icon: '📦',
  },
  {
    title: 'vs OpenCode',
    path: '/compare/opencode',
    description: 'Caro vs OpenCode',
    category: 'compare',
    keywords: ['opencode', 'compare'],
    icon: '💡',
  },

  // Blog
  {
    title: 'Blog',
    path: '/blog',
    description: 'Caro blog and updates',
    category: 'blog',
    keywords: ['blog', 'news', 'updates', 'articles'],
    icon: '📝',
  },
  {
    title: 'Announcing Caro',
    path: '/blog/announcing-caro',
    description: 'Introducing Caro to the world',
    category: 'blog',
    keywords: ['announcement', 'launch', 'intro'],
    icon: '🎉',
  },
  {
    title: 'Why Caro?',
    path: '/blog/why-caro',
    description: 'Why we built Caro',
    category: 'blog',
    keywords: ['why', 'motivation', 'story'],
    icon: '❓',
  },
  {
    title: 'Security Practices',
    path: '/blog/security-practices',
    description: 'Security best practices with Caro',
    category: 'blog',
    keywords: ['security', 'practices', 'safety'],
    icon: '🔐',
  },
  {
    title: 'Claude Skill Launch',
    path: '/blog/claude-skill-launch',
    description: 'Claude Code skill integration',
    category: 'blog',
    keywords: ['claude', 'skill', 'integration'],
    icon: '🤖',
  },
  {
    title: 'Batteries Included',
    path: '/blog/batteries-included',
    description: 'Everything included out of the box',
    category: 'blog',
    keywords: ['batteries', 'included', 'features'],
    icon: '🔋',
  },
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

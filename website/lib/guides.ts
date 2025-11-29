import fs from 'fs';
import path from 'path';
import matter from 'gray-matter';
import MarkdownIt from 'markdown-it';
import type { Guide, GuideCategory, GuideDifficulty, RiskLevel, GuideFilter } from '@/types';

const GUIDES_DIR = path.join(process.cwd(), '..', 'docs', 'guides');
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
});

/**
 * Recursively find all markdown files in guides directory
 */
function findMarkdownFiles(dir: string): string[] {
  const files: string[] = [];

  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      files.push(...findMarkdownFiles(fullPath));
    } else if (entry.isFile() && entry.name.endsWith('.md') && entry.name !== 'README.md') {
      files.push(fullPath);
    }
  }

  return files;
}

/**
 * Load all guides from Markdown files
 */
export async function loadGuides(): Promise<Guide[]> {
  const files = findMarkdownFiles(GUIDES_DIR);

  const guides = files.map(filePath => {
    const fileContents = fs.readFileSync(filePath, 'utf8');
    const { data, content } = matter(fileContents);

    const guide: Guide = {
      id: data.id,
      title: data.title,
      description: data.description,
      category: data.category as GuideCategory,
      difficulty: data.difficulty as GuideDifficulty,
      tags: data.tags || [],
      natural_language_prompt: data.natural_language_prompt,
      generated_command: data.generated_command,
      shell_type: data.shell_type,
      risk_level: data.risk_level as RiskLevel,
      author: data.author,
      created_at: data.created_at,
      updated_at: data.updated_at,
      prerequisites: data.prerequisites || [],
      expected_outcomes: data.expected_outcomes || [],
      metrics: {
        upvotes: data.upvotes || 0,
        downvotes: data.downvotes || 0,
        execution_count: data.execution_count || 0,
        success_count: data.success_count || 0,
        failure_count: data.failure_count || 0,
        view_count: data.view_count || 0,
      },
      related_guides: data.related_guides || [],
      related_guardrails: data.related_guardrails || [],
      alternatives: data.alternatives || [],
      content: content,
    };

    return guide;
  });

  return guides.sort((a, b) => a.id.localeCompare(b.id));
}

/**
 * Load a single guide by ID
 */
export async function loadGuideById(id: string): Promise<Guide | null> {
  const guides = await loadGuides();
  return guides.find(g => g.id === id) || null;
}

/**
 * Render guide content as HTML
 */
export function renderGuideContent(content: string): string {
  return md.render(content);
}

/**
 * Calculate guide quality score
 */
export function calculateQualityScore(guide: Guide): number {
  const { metrics } = guide;
  const totalVotes = metrics.upvotes + metrics.downvotes;

  const upvoteRatio = totalVotes > 0 ? metrics.upvotes / totalVotes : 0.5;
  const successRate = metrics.execution_count > 0 ? metrics.success_count / metrics.execution_count : 0;
  const executionScore = Math.min(Math.log10(metrics.execution_count + 1) / 4, 1);

  return upvoteRatio * 0.4 + successRate * 0.4 + executionScore * 0.2;
}

/**
 * Filter guides based on criteria
 */
export function filterGuides(guides: Guide[], filter: GuideFilter): Guide[] {
  return guides.filter(g => {
    if (filter.category && g.category !== filter.category) return false;
    if (filter.difficulty && g.difficulty !== filter.difficulty) return false;
    if (filter.risk_level && g.risk_level !== filter.risk_level) return false;
    if (filter.shell_type && g.shell_type !== filter.shell_type) return false;

    if (filter.min_quality) {
      const quality = calculateQualityScore(g);
      if (quality < filter.min_quality) return false;
    }

    if (filter.popular_only) {
      if (g.metrics.execution_count < 100) return false;
      const successRate = g.metrics.execution_count > 0 ? g.metrics.success_count / g.metrics.execution_count : 0;
      if (successRate < 0.8) return false;
    }

    if (filter.search) {
      const searchLower = filter.search.toLowerCase();
      const matchesSearch =
        g.title.toLowerCase().includes(searchLower) ||
        g.description.toLowerCase().includes(searchLower) ||
        g.natural_language_prompt.toLowerCase().includes(searchLower) ||
        g.generated_command.toLowerCase().includes(searchLower) ||
        g.tags.some(tag => tag.toLowerCase().includes(searchLower));
      if (!matchesSearch) return false;
    }

    return true;
  });
}

/**
 * Get category metadata
 */
export function getGuideCategoryInfo(): Record<GuideCategory, { name: string; icon: string; description: string }> {
  return {
    Git: {
      name: 'Git',
      icon: '🔀',
      description: 'Version control and repository management',
    },
    Docker: {
      name: 'Docker',
      icon: '🐳',
      description: 'Container management and operations',
    },
    FileManagement: {
      name: 'File Management',
      icon: '📁',
      description: 'File operations and disk usage',
    },
    Networking: {
      name: 'Networking',
      icon: '🌐',
      description: 'Network configuration and debugging',
    },
    SystemAdministration: {
      name: 'System Administration',
      icon: '⚙️',
      description: 'System services and configuration',
    },
    Development: {
      name: 'Development',
      icon: '💻',
      description: 'Development tools and workflows',
    },
    Database: {
      name: 'Database',
      icon: '🗄️',
      description: 'Database operations and queries',
    },
    Kubernetes: {
      name: 'Kubernetes',
      icon: '☸️',
      description: 'Container orchestration',
    },
    Cloud: {
      name: 'Cloud',
      icon: '☁️',
      description: 'Cloud platform operations',
    },
    Security: {
      name: 'Security',
      icon: '🔒',
      description: 'Security and permissions',
    },
    TextProcessing: {
      name: 'Text Processing',
      icon: '📝',
      description: 'Text manipulation and processing',
    },
    Monitoring: {
      name: 'Monitoring',
      icon: '📊',
      description: 'System monitoring and observability',
    },
  };
}

/**
 * Get guides by category
 */
export async function getGuidesByCategory(): Promise<Record<GuideCategory, Guide[]>> {
  const guides = await loadGuides();
  const byCategory: Partial<Record<GuideCategory, Guide[]>> = {};

  guides.forEach(g => {
    if (!byCategory[g.category]) {
      byCategory[g.category] = [];
    }
    byCategory[g.category]!.push(g);
  });

  return byCategory as Record<GuideCategory, Guide[]>;
}

/**
 * Get guides statistics
 */
export async function getGuidesStats() {
  const guides = await loadGuides();

  const total = guides.length;
  const byCategory: Partial<Record<GuideCategory, number>> = {};
  const byDifficulty: Record<GuideDifficulty, number> = {
    Beginner: 0,
    Intermediate: 0,
    Advanced: 0,
  };

  guides.forEach(g => {
    byCategory[g.category] = (byCategory[g.category] || 0) + 1;
    byDifficulty[g.difficulty]++;
  });

  const totalExecutions = guides.reduce((sum, g) => sum + g.metrics.execution_count, 0);
  const totalSuccess = guides.reduce((sum, g) => sum + g.metrics.success_count, 0);
  const avgQuality = guides.reduce((sum, g) => sum + calculateQualityScore(g), 0) / guides.length;

  return {
    total,
    byCategory,
    byDifficulty,
    totalExecutions,
    totalSuccess,
    successRate: totalExecutions > 0 ? (totalSuccess / totalExecutions) * 100 : 0,
    avgQuality,
  };
}

/**
 * Get popular guides
 */
export async function getPopularGuides(limit: number = 10): Promise<Guide[]> {
  const guides = await loadGuides();
  return guides
    .filter(g => g.metrics.execution_count > 0)
    .sort((a, b) => b.metrics.execution_count - a.metrics.execution_count)
    .slice(0, limit);
}

/**
 * Get trending guides (most executions in recent period)
 */
export async function getTrendingGuides(limit: number = 10): Promise<Guide[]> {
  const guides = await loadGuides();
  // In a real app, you'd filter by date. For now, return most popular
  return getPopularGuides(limit);
}

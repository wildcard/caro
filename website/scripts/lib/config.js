/**
 * Configuration Management
 *
 * Handles content generation configuration, quotas, and scheduling rules.
 */

import { readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Default configuration
 */
const DEFAULT_CONFIG = {
  // Content type settings
  contentTypes: {
    'daily-pick': {
      schedule: '0 8 * * 1-5', // Mon-Fri at 8am
      maxPerDay: 1,
      priority: ['command', 'tip', 'story'],
      hashtags: ['unix', 'cli', 'terminal', 'caro'],
    },
    command: {
      schedule: '0 10 * * 1,3', // Mon and Wed at 10am (biweekly)
      maxPerWeek: 2,
      difficulties: ['beginner', 'intermediate', 'advanced'],
      platforms: ['linux', 'macos', 'bsd', 'unix', 'posix'],
    },
    story: {
      schedule: '0 10 1 * *', // 1st of month at 10am
      maxPerMonth: 2,
      categories: ['origin', 'evolution', 'people', 'design'],
      eras: ['1970s', '1980s', '1990s', '2000s', 'modern'],
    },
  },

  // Command categories for rotation
  commandCategories: [
    {
      name: 'File Operations',
      commands: [
        'find',
        'grep',
        'sed',
        'awk',
        'xargs',
        'cut',
        'sort',
        'uniq',
        'wc',
        'head',
        'tail',
        'cat',
        'less',
        'more',
      ],
      weight: 2,
    },
    {
      name: 'Process Management',
      commands: ['ps', 'top', 'htop', 'kill', 'pkill', 'jobs', 'bg', 'fg', 'nohup', 'screen', 'tmux'],
      weight: 1.5,
    },
    {
      name: 'Network Tools',
      commands: [
        'curl',
        'wget',
        'netstat',
        'ss',
        'lsof',
        'nc',
        'dig',
        'nslookup',
        'ping',
        'traceroute',
        'tcpdump',
      ],
      weight: 1.5,
    },
    {
      name: 'System Administration',
      commands: ['df', 'du', 'free', 'uptime', 'uname', 'hostname', 'date', 'cal', 'who', 'w', 'last'],
      weight: 1,
    },
    {
      name: 'Text Processing',
      commands: ['tr', 'tee', 'paste', 'join', 'comm', 'diff', 'patch', 'fmt', 'fold', 'column'],
      weight: 1,
    },
    {
      name: 'Shell Utilities',
      commands: ['alias', 'history', 'type', 'which', 'whereis', 'env', 'export', 'source', 'exec'],
      weight: 1,
    },
    {
      name: 'Archive & Compression',
      commands: ['tar', 'gzip', 'gunzip', 'zip', 'unzip', 'bzip2', 'xz', 'zcat', 'zgrep'],
      weight: 0.8,
    },
    {
      name: 'Permission & Ownership',
      commands: ['chmod', 'chown', 'chgrp', 'umask', 'stat', 'file', 'touch', 'mkdir', 'rmdir'],
      weight: 0.8,
    },
  ],

  // Story topics for rotation
  storyTopics: [
    {
      name: 'Unix Origins',
      topics: [
        'The Birth of Unix at Bell Labs',
        'Why Unix Succeeded Where Multics Failed',
        'The Missing "e" in creat()',
        'How Pipes Changed Everything',
      ],
      weight: 2,
    },
    {
      name: 'BSD Heritage',
      topics: [
        'Berkeley Unix: The Academic Revolution',
        'The TCP/IP Stack That Powered the Internet',
        'FreeBSD vs OpenBSD: Philosophy of Security',
        'The BSD License: Why It Matters',
      ],
      weight: 1.5,
    },
    {
      name: 'Linux Evolution',
      topics: [
        "Linus Torvalds' Fateful Usenet Post",
        'The Cathedral and the Bazaar',
        'How Linux Conquered the Server Room',
        'Systemd: Love It or Hate It',
      ],
      weight: 1.5,
    },
    {
      name: 'Tool Stories',
      topics: [
        'The History of grep',
        'Why We Still Use vi',
        'The Evolution of the Shell',
        'Make: The First Build Tool',
      ],
      weight: 1,
    },
    {
      name: 'Philosophy',
      topics: [
        'Do One Thing Well',
        'Everything is a File',
        'Text Streams as Universal Interface',
        'The Power of Composition',
      ],
      weight: 1,
    },
  ],

  // Social media settings
  socialMedia: {
    twitter: {
      enabled: true,
      maxLength: 280,
      defaultHashtags: ['unix', 'cli', 'caro'],
    },
    mastodon: {
      enabled: false,
      maxLength: 500,
    },
  },

  // Quality thresholds
  quality: {
    minWordCount: {
      command: 500,
      story: 800,
      'daily-pick': 100,
    },
    maxWordCount: {
      command: 3000,
      story: 5000,
      'daily-pick': 500,
    },
    requiredCodeBlocks: {
      command: 3,
      story: 1,
      'daily-pick': 1,
    },
  },
};

/**
 * Load configuration from file or use defaults
 */
export async function loadConfig() {
  const configPath = join(__dirname, '../../content-config.json');

  if (existsSync(configPath)) {
    try {
      const configData = readFileSync(configPath, 'utf-8');
      const userConfig = JSON.parse(configData);
      return mergeConfig(DEFAULT_CONFIG, userConfig);
    } catch (error) {
      console.warn(`Warning: Could not load config file: ${error.message}`);
    }
  }

  return DEFAULT_CONFIG;
}

/**
 * Deep merge configurations
 */
function mergeConfig(defaults, overrides) {
  const result = { ...defaults };

  for (const key in overrides) {
    if (
      overrides[key] !== null &&
      typeof overrides[key] === 'object' &&
      !Array.isArray(overrides[key])
    ) {
      result[key] = mergeConfig(defaults[key] || {}, overrides[key]);
    } else {
      result[key] = overrides[key];
    }
  }

  return result;
}

/**
 * Get content quota for a type
 */
export function getContentQuota(contentType) {
  const typeConfig = DEFAULT_CONFIG.contentTypes[contentType];

  if (!typeConfig) {
    return { daily: 1, weekly: 7, monthly: 30 };
  }

  return {
    daily: typeConfig.maxPerDay || 1,
    weekly: typeConfig.maxPerWeek || 7,
    monthly: typeConfig.maxPerMonth || 30,
  };
}

/**
 * Determine content type from environment or schedule
 */
export function determineContentType(requestedType) {
  // If explicitly requested, use that
  if (requestedType && requestedType !== 'auto') {
    return requestedType;
  }

  // Otherwise determine based on day of week
  const dayOfWeek = new Date().getDay();
  const dayOfMonth = new Date().getDate();

  // First of month: story
  if (dayOfMonth === 1) {
    return 'story';
  }

  // Mon, Wed: command tutorials
  if (dayOfWeek === 1 || dayOfWeek === 3) {
    return 'command';
  }

  // Default to daily pick
  return 'daily-pick';
}

/**
 * Get commands that haven't been covered yet
 */
export function getUncoveredCommands(existingCommands, config = DEFAULT_CONFIG) {
  const coveredCommands = new Set(existingCommands.map((c) => c.command?.toLowerCase()));
  const uncovered = [];

  for (const category of config.commandCategories) {
    for (const command of category.commands) {
      if (!coveredCommands.has(command.toLowerCase())) {
        uncovered.push({
          command,
          category: category.name,
          weight: category.weight,
        });
      }
    }
  }

  return uncovered;
}

/**
 * Get story topics that haven't been covered
 */
export function getUncoveredStories(existingStories, config = DEFAULT_CONFIG) {
  const coveredTitles = new Set(existingStories.map((s) => s.title?.toLowerCase()));
  const uncovered = [];

  for (const category of config.storyTopics) {
    for (const topic of category.topics) {
      // Check if similar topic exists (fuzzy match)
      const isCovered = [...coveredTitles].some((title) => {
        const topicWords = topic.toLowerCase().split(' ');
        return topicWords.some((word) => word.length > 4 && title.includes(word));
      });

      if (!isCovered) {
        uncovered.push({
          topic,
          category: category.name,
          weight: category.weight,
        });
      }
    }
  }

  return uncovered;
}

/**
 * Select random item with weighted probability
 */
export function weightedRandomSelect(items, count = 1) {
  if (items.length === 0) return [];
  if (items.length <= count) return items;

  const totalWeight = items.reduce((sum, item) => sum + (item.weight || 1), 0);
  const selected = [];
  const remaining = [...items];

  while (selected.length < count && remaining.length > 0) {
    let random = Math.random() * totalWeight;
    let selectedIndex = -1;

    for (let i = 0; i < remaining.length; i++) {
      random -= remaining[i].weight || 1;
      if (random <= 0) {
        selectedIndex = i;
        break;
      }
    }

    if (selectedIndex === -1) {
      selectedIndex = remaining.length - 1;
    }

    selected.push(remaining[selectedIndex]);
    remaining.splice(selectedIndex, 1);
  }

  return selected;
}

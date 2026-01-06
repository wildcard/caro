/**
 * Social Content Generator
 *
 * Generates and caches social media content for manual sharing.
 * No automated posting - maintainers share via dashboard.
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const SOCIAL_CACHE_PATH = join(__dirname, '../../src/data/social-queue.json');

/**
 * Platform configurations
 */
const PLATFORMS = {
  twitter: {
    name: 'Twitter / X',
    maxLength: 280,
    icon: 'twitter',
    shareUrl: (text, url) =>
      `https://twitter.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(url)}`,
  },
  mastodon: {
    name: 'Mastodon',
    maxLength: 500,
    icon: 'mastodon',
    shareUrl: (text) => `https://mastodon.social/share?text=${encodeURIComponent(text)}`,
  },
  linkedin: {
    name: 'LinkedIn',
    maxLength: 3000,
    icon: 'linkedin',
    shareUrl: (text, url) =>
      `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}`,
  },
  bluesky: {
    name: 'Bluesky',
    maxLength: 300,
    icon: 'bluesky',
    shareUrl: (text) =>
      `https://bsky.app/intent/compose?text=${encodeURIComponent(text)}`,
  },
  hackernews: {
    name: 'Hacker News',
    maxLength: null,
    icon: 'hackernews',
    shareUrl: (text, url) =>
      `https://news.ycombinator.com/submitlink?u=${encodeURIComponent(url)}&t=${encodeURIComponent(text)}`,
  },
  reddit: {
    name: 'Reddit',
    maxLength: 300,
    icon: 'reddit',
    shareUrl: (text, url) =>
      `https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(text)}`,
  },
};

/**
 * Generate social content for all platforms and cache it
 */
export async function cacheSocialContent(content) {
  const queue = loadSocialQueue();

  const socialItem = {
    id: generateId(),
    createdAt: new Date().toISOString(),
    contentType: content.contentType,
    title: content.title,
    slug: content.slug,
    url: content.url || `https://caro.sh/learn/${content.contentType}s/${content.slug}`,
    shared: {},
    platforms: {},
  };

  // Generate text for each platform
  for (const [platformId, platform] of Object.entries(PLATFORMS)) {
    const text = generatePlatformText(content, platformId, platform.maxLength);
    socialItem.platforms[platformId] = {
      text,
      length: text.length,
      maxLength: platform.maxLength,
      shareUrl: platform.shareUrl(text, socialItem.url),
    };
  }

  // Add to queue
  queue.pending.unshift(socialItem);

  // Keep queue manageable (max 50 pending items)
  if (queue.pending.length > 50) {
    queue.pending = queue.pending.slice(0, 50);
  }

  saveSocialQueue(queue);

  return socialItem;
}

/**
 * Generate platform-specific text
 */
function generatePlatformText(content, platform, maxLength) {
  const title = content.title;
  const description = content.description || '';
  const command = content.command;
  const hashtags = content.hashtags || ['unix', 'cli', 'terminal', 'caro'];

  let text = '';

  // Build text based on content type
  if (command) {
    text = `🖥️ Unix command of the day: ${command}\n\n${description || title}`;
  } else if (content.contentType === 'story') {
    text = `📜 ${title}\n\n${description}`;
  } else {
    text = `💡 ${title}\n\n${description}`;
  }

  // Add hashtags for platforms that use them
  if (platform === 'twitter' || platform === 'mastodon' || platform === 'bluesky') {
    const hashtagText = hashtags
      .slice(0, 4)
      .map((h) => `#${h}`)
      .join(' ');

    if (!maxLength || text.length + hashtagText.length + 2 <= maxLength - 30) {
      text = `${text}\n\n${hashtagText}`;
    }
  }

  // Truncate if needed
  if (maxLength && text.length > maxLength - 25) {
    // Leave room for URL
    text = text.slice(0, maxLength - 28) + '...';
  }

  return text;
}

/**
 * Mark content as shared on a platform
 */
export function markAsShared(itemId, platform) {
  const queue = loadSocialQueue();

  const itemIndex = queue.pending.findIndex((item) => item.id === itemId);
  if (itemIndex === -1) return false;

  const item = queue.pending[itemIndex];
  item.shared[platform] = new Date().toISOString();

  // Check if shared on all major platforms
  const majorPlatforms = ['twitter', 'linkedin'];
  const allShared = majorPlatforms.every((p) => item.shared[p]);

  if (allShared) {
    // Move to completed
    queue.pending.splice(itemIndex, 1);
    queue.completed.unshift(item);

    // Keep completed list manageable
    if (queue.completed.length > 100) {
      queue.completed = queue.completed.slice(0, 100);
    }
  }

  saveSocialQueue(queue);
  return true;
}

/**
 * Get pending social content
 */
export function getPendingSocial() {
  const queue = loadSocialQueue();
  return queue.pending;
}

/**
 * Get social queue stats
 */
export function getSocialStats() {
  const queue = loadSocialQueue();

  return {
    pendingCount: queue.pending.length,
    completedCount: queue.completed.length,
    oldestPending: queue.pending.length > 0 ? queue.pending[queue.pending.length - 1].createdAt : null,
    recentlyShared: queue.completed.slice(0, 5),
  };
}

/**
 * Load social queue from file
 */
function loadSocialQueue() {
  if (!existsSync(SOCIAL_CACHE_PATH)) {
    return { pending: [], completed: [] };
  }

  try {
    const data = readFileSync(SOCIAL_CACHE_PATH, 'utf-8');
    return JSON.parse(data);
  } catch {
    return { pending: [], completed: [] };
  }
}

/**
 * Save social queue to file
 */
function saveSocialQueue(queue) {
  const dir = dirname(SOCIAL_CACHE_PATH);
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  writeFileSync(SOCIAL_CACHE_PATH, JSON.stringify(queue, null, 2));
}

/**
 * Generate unique ID
 */
function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
}

/**
 * Get share URLs for an item
 */
export function getShareUrls(itemId) {
  const queue = loadSocialQueue();
  const item = queue.pending.find((i) => i.id === itemId);

  if (!item) return null;

  const urls = {};
  for (const [platformId, platform] of Object.entries(PLATFORMS)) {
    urls[platformId] = {
      name: platform.name,
      url: item.platforms[platformId].shareUrl,
      text: item.platforms[platformId].text,
    };
  }

  return urls;
}

export { PLATFORMS };

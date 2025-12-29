import { createReader } from '@keystatic/core/reader';
import keystaticConfig from '../../keystatic.config';

/**
 * Keystatic Reader
 *
 * This reader provides type-safe access to all content collections
 * managed by Keystatic. Use this instead of Astro's getCollection
 * for content that's managed through the CMS.
 *
 * Benefits:
 * - Type-safe content access
 * - Works in both dev and production
 * - Consistent API across all content types
 */
export const reader = createReader(process.cwd(), keystaticConfig);

/**
 * Helper function to get all commands
 */
export async function getAllCommands() {
  const commands = await reader.collections.commands.all();
  return commands.map((cmd) => ({
    slug: cmd.slug,
    ...cmd.entry,
  }));
}

/**
 * Helper function to get a single command by slug
 */
export async function getCommand(slug: string) {
  const command = await reader.collections.commands.read(slug);
  if (!command) return null;
  return { slug, ...command };
}

/**
 * Helper function to get all stories
 */
export async function getAllStories() {
  const stories = await reader.collections.stories.all();
  return stories.map((story) => ({
    slug: story.slug,
    ...story.entry,
  }));
}

/**
 * Helper function to get a single story by slug
 */
export async function getStory(slug: string) {
  const story = await reader.collections.stories.read(slug);
  if (!story) return null;
  return { slug, ...story };
}

/**
 * Helper function to get all daily picks
 */
export async function getAllDailyPicks() {
  const picks = await reader.collections.dailyPicks.all();
  return picks.map((pick) => ({
    slug: pick.slug,
    ...pick.entry,
  }));
}

/**
 * Helper function to get a single daily pick by slug
 */
export async function getDailyPick(slug: string) {
  const pick = await reader.collections.dailyPicks.read(slug);
  if (!pick) return null;
  return { slug, ...pick };
}

/**
 * Helper function to get site settings
 */
export async function getSiteSettings() {
  return reader.singletons.siteSettings.read();
}

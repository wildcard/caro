import { defineCollection, z } from 'astro:content';

/**
 * Command Tutorials Collection
 * Terminus-style practical command tutorials
 */
const commands = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    command: z.string(),
    description: z.string(),
    difficulty: z.enum(['beginner', 'intermediate', 'advanced']),
    tags: z.array(z.string()),
    platforms: z.array(z.enum(['linux', 'macos', 'bsd', 'unix', 'posix'])),
    publishedAt: z.date(),
    lastUpdated: z.date().optional(),
    relatedCommands: z.array(z.string()).optional(),
    caroPrompt: z.string().optional(),
    featured: z.boolean().default(false),
  }),
});

/**
 * Unix History Stories Collection
 * Narratives about Unix ecosystem history and culture
 */
const stories = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    subtitle: z.string().optional(),
    category: z.enum(['history', 'people', 'technology', 'culture', 'platform']),
    era: z.string(),
    publishedAt: z.date(),
    featured: z.boolean().default(false),
    readingTime: z.number(),
    author: z.string().default('Caro Team'),
    sources: z.array(z.object({
      title: z.string(),
      url: z.string().url(),
    })).optional(),
    tags: z.array(z.string()).optional(),
  }),
});

/**
 * Daily Picks Collection
 * Short-form content for social media
 */
const dailyPicks = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    type: z.enum(['command', 'trivia', 'quote', 'tip', 'error']),
    publishedAt: z.date(),
    socialText: z.string().max(280),
    hashtags: z.array(z.string()),
    source: z.string().optional(),
  }),
});

export const collections = {
  commands,
  stories,
  'daily-picks': dailyPicks,
};

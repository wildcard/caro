/**
 * Validation schemas using Zod
 * Provides runtime type checking for API responses and user input
 */

import { z } from 'zod';

/**
 * Safety level schema
 */
export const safetyLevelSchema = z.enum(['safe', 'moderate', 'dangerous', 'blocked']);

/**
 * Difficulty schema
 */
export const difficultySchema = z.enum(['beginner', 'intermediate', 'advanced', 'expert']);

/**
 * Redaction schema
 */
export const redactionSchema = z.object({
  type: z.enum(['api_key', 'jwt', 'aws_key', 'ssh_key', 'email', 'ip', 'path', 'env_var', 'generic']),
  startIndex: z.number().int().min(0),
  endIndex: z.number().int().min(0),
  replacementText: z.string(),
  confidence: z.number().min(0).max(1),
});

/**
 * Command artifact schema
 */
export const commandArtifactSchema = z.object({
  type: z.literal('command_artifact'),
  id: z.string(),
  prompt: z.string().min(1),
  command: z.string().min(1),
  safetyScore: safetyLevelSchema,
  backend: z.string(),
  timestamp: z.string().datetime(),
  tags: z.array(z.string()),
  guild: z.string().optional(),
  redactionsApplied: z.array(redactionSchema),
  publishedAt: z.string().datetime().optional(),
  uri: z.string().optional(),
  cid: z.string().optional(),
});

/**
 * Runbook step schema
 */
export const runbookStepSchema = z.object({
  stepNumber: z.number().int().min(1),
  title: z.string().min(1),
  command: z.string().min(1),
  explanation: z.string().optional(),
  safetyLevel: safetyLevelSchema,
  expectedOutput: z.string().optional(),
  troubleshooting: z.string().optional(),
});

/**
 * Runbook schema
 */
export const runbookSchema = z.object({
  type: z.literal('runbook'),
  id: z.string(),
  title: z.string().min(1).max(200),
  description: z.string().min(1),
  steps: z.array(runbookStepSchema).min(1),
  prerequisites: z.array(z.string()).optional(),
  estimatedTime: z.string().optional(),
  difficulty: difficultySchema.optional(),
  guild: z.string().optional(),
  tags: z.array(z.string()),
  authorDid: z.string(),
  timestamp: z.string().datetime(),
  forkCount: z.number().int().min(0),
  publishedAt: z.string().datetime().optional(),
  uri: z.string().optional(),
  cid: z.string().optional(),
});

/**
 * Win story schema
 */
export const winStorySchema = z.object({
  type: z.literal('win_story'),
  id: z.string(),
  title: z.string().min(1).max(200),
  story: z.string().min(10), // Markdown content
  relatedCommands: z.array(z.string()).optional(),
  relatedRunbooks: z.array(z.string()).optional(),
  tags: z.array(z.string()),
  guild: z.string().optional(),
  authorDid: z.string(),
  timestamp: z.string().datetime(),
  publishedAt: z.string().datetime().optional(),
  uri: z.string().optional(),
  cid: z.string().optional(),
});

/**
 * Epic fail schema
 */
export const epicFailSchema = z.object({
  type: z.literal('epic_fail'),
  id: z.string(),
  title: z.string().min(1).max(200),
  description: z.string().min(1),
  whatHappened: z.string().min(10),
  whyItHappened: z.string().optional(),
  howToAvoid: z.string().min(10),
  relatedCommands: z.array(z.string()).optional(),
  verboseLogs: z.string().optional(),
  tags: z.array(z.string()),
  guild: z.string().optional(),
  authorDid: z.string(),
  timestamp: z.string().datetime(),
  publishedAt: z.string().datetime().optional(),
  uri: z.string().optional(),
  cid: z.string().optional(),
});

/**
 * Privacy settings schema
 */
export const privacySettingsSchema = z.object({
  autoRedactEmails: z.boolean(),
  autoRedactIPs: z.boolean(),
  autoRedactPaths: z.boolean(),
  autoRedactApiKeys: z.boolean(),
  autoRedactJWTs: z.boolean(),
  autoRedactSSHKeys: z.boolean(),
  autoRedactAWSKeys: z.boolean(),
  autoRedactEnvVars: z.boolean(),
  safetyThreshold: safetyLevelSchema,
  defaultGuildVisibility: z.enum(['public', 'guild_only', 'private']),
  allowAnonymousUsageStats: z.boolean(),
  retainLocalHistoryDays: z.number().int().min(-1),
});

/**
 * Local stats schema
 */
export const localStatsSchema = z.object({
  totalCommands: z.number().int().min(0),
  successfulCommands: z.number().int().min(0),
  failedCommands: z.number().int().min(0),
  dangerousBlocked: z.number().int().min(0),
  lastUsed: z.string().datetime(),
  favoriteBackend: z.string().optional(),
  favoriteGuild: z.string().optional(),
});

/**
 * Social stats schema
 */
export const socialStatsSchema = z.object({
  followersCount: z.number().int().min(0),
  followingCount: z.number().int().min(0),
  postsCount: z.number().int().min(0),
  commandsShared: z.number().int().min(0),
  runbooksShared: z.number().int().min(0),
  winStoriesShared: z.number().int().min(0),
  epicFailsShared: z.number().int().min(0),
  totalLikes: z.number().int().min(0),
  totalReposts: z.number().int().min(0),
  reputation: z.number().min(0).max(100),
});

/**
 * User profile schema
 */
export const userProfileSchema = z.object({
  did: z.string(),
  handle: z.string(),
  displayName: z.string().optional(),
  description: z.string().optional(),
  avatar: z.string().url().optional(),
  joinedAt: z.string().datetime(),
  joinedGuilds: z.array(z.string()),
  pinnedArtifacts: z.array(z.string()),
  localStats: localStatsSchema,
  socialStats: socialStatsSchema,
  privacySettings: privacySettingsSchema,
  lastSyncedAt: z.string().datetime().optional(),
});

/**
 * Auth state schema
 */
export const authStateSchema = z.object({
  isAuthenticated: z.boolean(),
  profile: userProfileSchema.optional(),
  accessToken: z.string().optional(),
  refreshToken: z.string().optional(),
  expiresAt: z.string().datetime().optional(),
});

/**
 * OAuth state schema
 */
export const oauthStateSchema = z.object({
  state: z.string(),
  codeVerifier: z.string(),
  codeChallenge: z.string(),
  createdAt: z.string().datetime(),
  redirectUri: z.string().url(),
});

/**
 * OAuth tokens schema
 */
export const oauthTokensSchema = z.object({
  accessToken: z.string(),
  refreshToken: z.string(),
  tokenType: z.string(),
  expiresIn: z.number().int().min(0),
  scope: z.string(),
  did: z.string(),
});

/**
 * Bluesky profile schema
 */
export const blueskyProfileSchema = z.object({
  did: z.string(),
  handle: z.string(),
  displayName: z.string().optional(),
  description: z.string().optional(),
  avatar: z.string().url().optional(),
  banner: z.string().url().optional(),
  followersCount: z.number().int().min(0).optional(),
  followsCount: z.number().int().min(0).optional(),
  postsCount: z.number().int().min(0).optional(),
  indexedAt: z.string().datetime().optional(),
});

/**
 * Guild schema
 */
export const guildSchema = z.object({
  did: z.string(),
  handle: z.string(),
  displayName: z.string(),
  description: z.string(),
  avatar: z.string().url().optional(),
  banner: z.string().url().optional(),
  createdAt: z.string().datetime(),
  createdBy: z.string(),
  moderators: z.array(z.string()),
  memberCount: z.number().int().min(0),
  postsCount: z.number().int().min(0),
  visibility: z.enum(['public', 'invite_only', 'private']),
  allowedContentTypes: z.array(z.enum(['command', 'runbook', 'win_story', 'epic_fail', 'discussion'])),
  tags: z.array(z.string()),
  rules: z.array(z.string()).optional(),
  safetyLevel: z.enum(['relaxed', 'moderate', 'strict']).optional(),
});

/**
 * Validation helper functions
 */
export const validate = {
  commandArtifact: (data: unknown) => commandArtifactSchema.safeParse(data),
  runbook: (data: unknown) => runbookSchema.safeParse(data),
  winStory: (data: unknown) => winStorySchema.safeParse(data),
  epicFail: (data: unknown) => epicFailSchema.safeParse(data),
  userProfile: (data: unknown) => userProfileSchema.safeParse(data),
  privacySettings: (data: unknown) => privacySettingsSchema.safeParse(data),
  authState: (data: unknown) => authStateSchema.safeParse(data),
  oauthState: (data: unknown) => oauthStateSchema.safeParse(data),
  oauthTokens: (data: unknown) => oauthTokensSchema.safeParse(data),
  blueskyProfile: (data: unknown) => blueskyProfileSchema.safeParse(data),
  guild: (data: unknown) => guildSchema.safeParse(data),
};

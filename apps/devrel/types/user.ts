/**
 * User profile and privacy settings types
 */

import type { SafetyLevel } from './artifacts';

/**
 * Privacy settings for user's data sharing preferences
 */
export interface PrivacySettings {
  // Auto-redaction settings
  autoRedactEmails: boolean;
  autoRedactIPs: boolean;
  autoRedactPaths: boolean;
  autoRedactApiKeys: boolean;
  autoRedactJWTs: boolean;
  autoRedactSSHKeys: boolean;
  autoRedactAWSKeys: boolean;
  autoRedactEnvVars: boolean;

  // Safety threshold - minimum safety level required to share
  safetyThreshold: SafetyLevel;

  // Guild privacy
  defaultGuildVisibility: 'public' | 'guild_only' | 'private';

  // Analytics
  allowAnonymousUsageStats: boolean;

  // Local storage
  retainLocalHistoryDays: number; // 0 = disabled, -1 = forever
}

/**
 * Default privacy settings (privacy-first, conservative)
 */
export const DEFAULT_PRIVACY_SETTINGS: PrivacySettings = {
  autoRedactEmails: true,
  autoRedactIPs: true,
  autoRedactPaths: true,
  autoRedactApiKeys: true,
  autoRedactJWTs: true,
  autoRedactSSHKeys: true,
  autoRedactAWSKeys: true,
  autoRedactEnvVars: true,
  safetyThreshold: 'safe',
  defaultGuildVisibility: 'guild_only',
  allowAnonymousUsageStats: false,
  retainLocalHistoryDays: 30,
};

/**
 * Local CLI statistics (from Caro CLI telemetry)
 */
export interface LocalStats {
  totalCommands: number;
  successfulCommands: number;
  failedCommands: number;
  dangerousBlocked: number;
  lastUsed: string; // ISO 8601
  favoriteBackend?: string;
  favoriteGuild?: string;
}

/**
 * Social statistics (from Bluesky)
 */
export interface SocialStats {
  followersCount: number;
  followingCount: number;
  postsCount: number;
  commandsShared: number;
  runbooksShared: number;
  winStoriesShared: number;
  epicFailsShared: number;
  totalLikes: number;
  totalReposts: number;
  reputation: number; // 0-100 score
}

/**
 * User profile (combines Bluesky identity + Caro data)
 */
export interface UserProfile {
  // Bluesky identity
  did: string; // Decentralized Identifier
  handle: string; // e.g., @alice.bsky.social
  displayName?: string;
  description?: string;
  avatar?: string; // URL

  // Caro-specific
  joinedAt: string; // ISO 8601
  joinedGuilds: string[]; // Guild DIDs or handles
  pinnedArtifacts: string[]; // Artifact URIs

  // Statistics
  localStats: LocalStats;
  socialStats: SocialStats;

  // Settings
  privacySettings: PrivacySettings;

  // Metadata
  lastSyncedAt?: string; // Last time we synced with Bluesky
}

/**
 * Auth state
 */
export interface AuthState {
  isAuthenticated: boolean;
  profile?: UserProfile;
  accessToken?: string; // Encrypted in storage
  refreshToken?: string; // Encrypted in storage
  expiresAt?: string; // ISO 8601
}

/**
 * Session state (runtime only, not persisted)
 */
export interface SessionState {
  isOnline: boolean;
  lastHeartbeat?: string;
  pendingUploads: number;
  syncStatus: 'idle' | 'syncing' | 'error';
}

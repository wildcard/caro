/**
 * Caro Web Hub - Core Types
 * Central export for all TypeScript types
 */

// Artifacts
export type {
  SafetyLevel,
  Difficulty,
  Redaction,
  CommandArtifact,
  RunbookStep,
  Runbook,
  WinStory,
  EpicFail,
  Artifact,
} from './artifacts';

export {
  isCommandArtifact,
  isRunbook,
  isWinStory,
  isEpicFail,
} from './artifacts';

// User & Privacy Settings
export type {
  PrivacySettings,
  LocalStats,
  SocialStats,
  UserProfile,
  AuthState,
  SessionState,
} from './user';

export { DEFAULT_PRIVACY_SETTINGS } from './user';

// Guilds
export type {
  Guild,
  ContentType,
  GuildMembership,
  GuildFeedItem,
} from './guild';

export { POPULAR_GUILDS } from './guild';

// Bluesky AT Protocol
export type {
  OAuthState,
  OAuthTokens,
  BlueskySession,
  ATUri,
  BlueskyRecord,
  RepoCommit,
  StrongRef,
  BlueskyProfile,
  BlueskyError,
  RateLimitInfo,
} from './bluesky';

export {
  parseATUri,
  formatATUri,
} from './bluesky';

// Privacy & Redaction
export type {
  DetectedItem,
  PrivacyScanResult,
  RedactionPattern,
  PrivacyScanOptions,
  LocalTelemetry,
  LocalCommandRecord,
  LocalRunbookRecord,
  PrivacyDashboardSummary,
} from './privacy';

export { REDACTION_PATTERNS } from './privacy';

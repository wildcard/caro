/**
 * Bluesky AT Protocol types
 */

/**
 * OAuth PKCE flow state
 */
export interface OAuthState {
  state: string; // Random state for CSRF protection
  codeVerifier: string; // PKCE code verifier
  codeChallenge: string; // PKCE code challenge
  createdAt: string;
  redirectUri: string;
}

/**
 * OAuth token response
 */
export interface OAuthTokens {
  accessToken: string;
  refreshToken: string;
  tokenType: string; // 'Bearer'
  expiresIn: number; // Seconds
  scope: string;
  did: string; // User's DID
}

/**
 * Bluesky session
 */
export interface BlueskySession {
  did: string;
  handle: string;
  email?: string;
  accessJwt: string;
  refreshJwt: string;
  active: boolean;
}

/**
 * AT Protocol URI components
 */
export interface ATUri {
  did: string;
  collection: string;
  rkey: string; // Record key (TID)
}

/**
 * Parse AT URI string (at://did/collection/rkey)
 */
export function parseATUri(uri: string): ATUri | null {
  const match = uri.match(/^at:\/\/([^/]+)\/([^/]+)\/([^/]+)$/);
  if (!match) return null;

  return {
    did: match[1],
    collection: match[2],
    rkey: match[3],
  };
}

/**
 * Format AT URI
 */
export function formatATUri(components: ATUri): string {
  return `at://${components.did}/${components.collection}/${components.rkey}`;
}

/**
 * Bluesky record (generic)
 */
export interface BlueskyRecord {
  $type: string; // Lexicon type (e.g., 'app.caro.share.command')
  createdAt: string;
  [key: string]: any;
}

/**
 * Repository commit
 */
export interface RepoCommit {
  rev: string; // Revision
  cid: string; // Content ID
  commit: {
    cid: string;
    rev: string;
  };
}

/**
 * Strong reference (link to another record)
 */
export interface StrongRef {
  uri: string;
  cid: string;
}

/**
 * Bluesky profile
 */
export interface BlueskyProfile {
  did: string;
  handle: string;
  displayName?: string;
  description?: string;
  avatar?: string;
  banner?: string;
  followersCount?: number;
  followsCount?: number;
  postsCount?: number;
  indexedAt?: string;
}

/**
 * Error response from Bluesky API
 */
export interface BlueskyError {
  error: string;
  message: string;
}

/**
 * Rate limit info
 */
export interface RateLimitInfo {
  limit: number;
  remaining: number;
  reset: string; // ISO 8601
}

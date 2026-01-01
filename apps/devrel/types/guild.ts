/**
 * Guild (community) types
 */

/**
 * Guild - A professional community (like subreddits)
 */
export interface Guild {
  did: string; // Guild's Bluesky DID
  handle: string; // e.g., @sre-guild.caro.sh
  displayName: string; // e.g., "SRE Guild"
  description: string;
  avatar?: string;
  banner?: string;

  // Metadata
  createdAt: string;
  createdBy: string; // Creator's DID
  moderators: string[]; // DIDs of moderators

  // Stats
  memberCount: number;
  postsCount: number;

  // Settings
  visibility: 'public' | 'invite_only' | 'private';
  allowedContentTypes: ContentType[];
  tags: string[]; // Common tags for this guild

  // Rules
  rules?: string[]; // Guild-specific rules
  safetyLevel?: 'relaxed' | 'moderate' | 'strict'; // Moderation level
}

export type ContentType = 'command' | 'runbook' | 'win_story' | 'epic_fail' | 'discussion';

/**
 * Guild membership
 */
export interface GuildMembership {
  guildDid: string;
  userDid: string;
  role: 'member' | 'moderator' | 'admin' | 'creator';
  joinedAt: string;
  postsCount: number;
  reputation: number; // Guild-specific reputation
  isMuted: boolean;
  isBanned: boolean;
}

/**
 * Guild feed item
 */
export interface GuildFeedItem {
  uri: string; // AT Protocol URI
  cid: string;
  author: {
    did: string;
    handle: string;
    displayName?: string;
    avatar?: string;
  };
  content: {
    type: ContentType;
    text: string;
    // Additional fields based on type
    [key: string]: any;
  };
  indexedAt: string;
  likeCount: number;
  repostCount: number;
  replyCount: number;
  viewerLiked: boolean;
  viewerReposted: boolean;
}

/**
 * Popular guilds (pre-defined for MVP)
 */
export const POPULAR_GUILDS: Partial<Guild>[] = [
  {
    handle: '@sre.caro.sh',
    displayName: 'Site Reliability Engineering',
    description: 'Production operations, incident response, and reliability patterns',
    tags: ['kubernetes', 'monitoring', 'incidents', 'oncall'],
  },
  {
    handle: '@appsec.caro.sh',
    displayName: 'Application Security',
    description: 'Security scanning, vulnerability research, and secure development',
    tags: ['security', 'scanning', 'vulnerabilities', 'pentesting'],
  },
  {
    handle: '@devops.caro.sh',
    displayName: 'DevOps & Platform Engineering',
    description: 'CI/CD pipelines, infrastructure as code, and developer tooling',
    tags: ['cicd', 'terraform', 'docker', 'automation'],
  },
  {
    handle: '@data.caro.sh',
    displayName: 'Data Engineering',
    description: 'Data pipelines, ETL, databases, and analytics',
    tags: ['databases', 'etl', 'analytics', 'bigdata'],
  },
  {
    handle: '@cloud.caro.sh',
    displayName: 'Cloud Architecture',
    description: 'AWS, GCP, Azure, and multi-cloud patterns',
    tags: ['aws', 'gcp', 'azure', 'cloud'],
  },
];

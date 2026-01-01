/**
 * Core artifact types for Caro Web Hub
 * These represent the social content types users can share
 */

export type SafetyLevel = 'safe' | 'moderate' | 'dangerous' | 'blocked';

export type Difficulty = 'beginner' | 'intermediate' | 'advanced' | 'expert';

export interface Redaction {
  type: 'api_key' | 'jwt' | 'aws_key' | 'ssh_key' | 'email' | 'ip' | 'path' | 'env_var' | 'generic';
  startIndex: number;
  endIndex: number;
  replacementText: string;
  confidence: number; // 0-1
}

/**
 * Command Artifact - A single generated command with context
 */
export interface CommandArtifact {
  type: 'command_artifact';
  id: string;
  prompt: string;
  command: string;
  safetyScore: SafetyLevel;
  backend: string; // 'mlx', 'openai', 'anthropic', etc.
  timestamp: string; // ISO 8601
  tags: string[];
  guild?: string;
  redactionsApplied: Redaction[];
  publishedAt?: string;
  uri?: string; // AT Protocol URI (at://did/collection/rkey)
  cid?: string; // Content ID on Bluesky
}

/**
 * Runbook Step - A single step in a runbook
 */
export interface RunbookStep {
  stepNumber: number;
  title: string;
  command: string;
  explanation?: string;
  safetyLevel: SafetyLevel;
  expectedOutput?: string;
  troubleshooting?: string;
}

/**
 * Runbook - A multi-step operational procedure
 */
export interface Runbook {
  type: 'runbook';
  id: string;
  title: string;
  description: string;
  steps: RunbookStep[];
  prerequisites?: string[];
  estimatedTime?: string; // e.g., "~5-10 minutes"
  difficulty?: Difficulty;
  guild?: string;
  tags: string[];
  authorDid: string;
  timestamp: string;
  forkCount: number;
  publishedAt?: string;
  uri?: string;
  cid?: string;
}

/**
 * Win Story - A success story or "aha moment"
 */
export interface WinStory {
  type: 'win_story';
  id: string;
  title: string;
  story: string; // Markdown content
  relatedCommands?: string[]; // Command artifact IDs
  relatedRunbooks?: string[]; // Runbook IDs
  tags: string[];
  guild?: string;
  authorDid: string;
  timestamp: string;
  publishedAt?: string;
  uri?: string;
  cid?: string;
}

/**
 * Epic Fail - A learning experience from a mistake
 */
export interface EpicFail {
  type: 'epic_fail';
  id: string;
  title: string;
  description: string;
  whatHappened: string; // What went wrong
  whyItHappened?: string; // Root cause analysis
  howToAvoid: string; // Prevention tips
  relatedCommands?: string[];
  verboseLogs?: string; // Redacted logs
  tags: string[];
  guild?: string;
  authorDid: string;
  timestamp: string;
  publishedAt?: string;
  uri?: string;
  cid?: string;
}

/**
 * Union type for all artifact types
 */
export type Artifact = CommandArtifact | Runbook | WinStory | EpicFail;

/**
 * Type guard functions
 */
export function isCommandArtifact(artifact: Artifact): artifact is CommandArtifact {
  return artifact.type === 'command_artifact';
}

export function isRunbook(artifact: Artifact): artifact is Runbook {
  return artifact.type === 'runbook';
}

export function isWinStory(artifact: Artifact): artifact is WinStory {
  return artifact.type === 'win_story';
}

export function isEpicFail(artifact: Artifact): artifact is EpicFail {
  return artifact.type === 'epic_fail';
}

/**
 * Privacy and redaction types
 */

import type { Redaction, SafetyLevel } from './artifacts';

/**
 * Detected sensitive item before redaction
 */
export interface DetectedItem {
  type: Redaction['type'];
  value: string;
  startIndex: number;
  endIndex: number;
  confidence: number; // 0-1
  context: string; // Surrounding text for user review
}

/**
 * Privacy scan result
 */
export interface PrivacyScanResult {
  text: string; // Original text
  redactedText: string; // Text with redactions applied
  detectedItems: DetectedItem[];
  appliedRedactions: Redaction[];
  safetyScore: SafetyLevel;
  scanTimestamp: string;
  patternsUsed: string[]; // Which regex patterns were applied
}

/**
 * Redaction pattern configuration
 */
export interface RedactionPattern {
  name: string;
  type: Redaction['type'];
  pattern: RegExp;
  replacementTemplate: string; // e.g., '[REDACTED_EMAIL]'
  confidence: number; // Default confidence for this pattern
  enabled: boolean;
}

/**
 * Pre-defined redaction patterns
 */
export const REDACTION_PATTERNS: Record<string, RedactionPattern> = {
  API_KEY_GENERIC: {
    name: 'Generic API Key',
    type: 'api_key',
    pattern: /\b[a-zA-Z0-9]{32,}\b/g,
    replacementTemplate: '[REDACTED_API_KEY]',
    confidence: 0.6,
    enabled: true,
  },
  JWT: {
    name: 'JSON Web Token',
    type: 'jwt',
    pattern: /eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+/g,
    replacementTemplate: '[REDACTED_JWT]',
    confidence: 0.95,
    enabled: true,
  },
  AWS_ACCESS_KEY: {
    name: 'AWS Access Key',
    type: 'aws_key',
    pattern: /AKIA[0-9A-Z]{16}/g,
    replacementTemplate: '[REDACTED_AWS_KEY]',
    confidence: 0.99,
    enabled: true,
  },
  SSH_PRIVATE_KEY: {
    name: 'SSH Private Key',
    type: 'ssh_key',
    pattern: /-----BEGIN (RSA|OPENSSH|EC|DSA) PRIVATE KEY-----/g,
    replacementTemplate: '[REDACTED_SSH_KEY]',
    confidence: 1.0,
    enabled: true,
  },
  EMAIL: {
    name: 'Email Address',
    type: 'email',
    pattern: /\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b/g,
    replacementTemplate: '[REDACTED_EMAIL]',
    confidence: 0.85,
    enabled: true,
  },
  IPV4: {
    name: 'IPv4 Address',
    type: 'ip',
    pattern: /\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b/g,
    replacementTemplate: '[REDACTED_IP]',
    confidence: 0.7,
    enabled: true,
  },
  HOME_PATH_UNIX: {
    name: 'Unix Home Path',
    type: 'path',
    pattern: /\/home\/[a-zA-Z0-9_-]+/g,
    replacementTemplate: '/home/[REDACTED]',
    confidence: 0.8,
    enabled: true,
  },
  HOME_PATH_WINDOWS: {
    name: 'Windows User Path',
    type: 'path',
    pattern: /C:\\Users\\[a-zA-Z0-9_-]+/gi,
    replacementTemplate: 'C:\\Users\\[REDACTED]',
    confidence: 0.8,
    enabled: true,
  },
  ENV_VAR_SECRET: {
    name: 'Environment Variable Secret',
    type: 'env_var',
    pattern: /\$([A-Z_]+)?(SECRET|TOKEN|KEY|PASSWORD)([A-Z_]+)?/g,
    replacementTemplate: '$[REDACTED_ENV]',
    confidence: 0.75,
    enabled: true,
  },
};

/**
 * Privacy scan options
 */
export interface PrivacyScanOptions {
  enabledPatterns?: string[]; // Pattern names to use (default: all enabled)
  minimumConfidence?: number; // Min confidence to flag (default: 0.6)
  contextLength?: number; // Characters of context to include (default: 20)
  autoApplyRedactions?: boolean; // Auto-apply vs. suggest (default: false)
}

/**
 * Local telemetry data structure (from Caro CLI)
 */
export interface LocalTelemetry {
  commands: LocalCommandRecord[];
  runbooks: LocalRunbookRecord[];
  stats: {
    totalCommands: number;
    successfulCommands: number;
    failedCommands: number;
    dangerousBlocked: number;
  };
  lastUpdated: string;
}

/**
 * Local command record (before sharing)
 */
export interface LocalCommandRecord {
  id: string;
  prompt: string;
  command: string;
  backend: string;
  safetyScore: SafetyLevel;
  timestamp: string;
  success?: boolean;
  exitCode?: number;
  shared: boolean;
}

/**
 * Local runbook record
 */
export interface LocalRunbookRecord {
  id: string;
  title: string;
  description: string;
  steps: Array<{
    command: string;
    safetyLevel: SafetyLevel;
  }>;
  timestamp: string;
  shared: boolean;
}

/**
 * Privacy dashboard summary
 */
export interface PrivacyDashboardSummary {
  totalCommandsLocal: number;
  totalCommandsShared: number;
  sensitiveItemsDetected: number;
  lastScan?: string;
  localStorageSize: number; // bytes
  recommendation: 'safe' | 'review_recommended' | 'action_required';
}

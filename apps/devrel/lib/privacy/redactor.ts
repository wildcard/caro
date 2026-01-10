/**
 * Privacy Redaction Engine
 * Detects and redacts sensitive information from text
 */

import type {
  DetectedItem,
  PrivacyScanResult,
  RedactionPattern,
  PrivacyScanOptions,
  Redaction,
} from '@/types/privacy';
import type { SafetyLevel } from '@/types/artifacts';
import { REDACTION_PATTERNS } from '@/types/privacy';

/**
 * Scan text for sensitive information
 * Returns detected items without applying redactions
 */
export function scanText(
  text: string,
  options: PrivacyScanOptions = {}
): DetectedItem[] {
  const {
    enabledPatterns = Object.keys(REDACTION_PATTERNS),
    minimumConfidence = 0.6,
    contextLength = 20,
  } = options;

  const detectedItems: DetectedItem[] = [];

  // Sort patterns by specificity (higher confidence = more specific)
  // This ensures JWT, AWS keys, SSH keys are matched before generic API keys
  const sortedPatterns = enabledPatterns
    .map((name) => ({ name, pattern: REDACTION_PATTERNS[name] }))
    .filter((p) => p.pattern && p.pattern.enabled)
    .sort((a, b) => b.pattern.confidence - a.pattern.confidence);

  // Apply each enabled pattern in order of specificity
  for (const { name: patternName, pattern } of sortedPatterns) {

    // Reset regex lastIndex for global patterns
    const regex = new RegExp(pattern.pattern.source, pattern.pattern.flags);
    let match: RegExpExecArray | null;

    while ((match = regex.exec(text)) !== null) {
      const startIndex = match.index;
      const endIndex = startIndex + match[0].length;
      const confidence = pattern.confidence;

      // Skip if below minimum confidence
      if (confidence < minimumConfidence) continue;

      // Extract context around the match
      const contextStart = Math.max(0, startIndex - contextLength);
      const contextEnd = Math.min(text.length, endIndex + contextLength);
      const context =
        (contextStart > 0 ? '...' : '') +
        text.slice(contextStart, contextEnd) +
        (contextEnd < text.length ? '...' : '');

      detectedItems.push({
        type: pattern.type,
        value: match[0],
        startIndex,
        endIndex,
        confidence,
        context,
      });
    }
  }

  // Sort by start index for consistent ordering
  return detectedItems.sort((a, b) => a.startIndex - b.startIndex);
}

/**
 * Apply redactions to text
 * Converts DetectedItems to actual Redactions and modifies text
 */
export function applyRedactions(
  text: string,
  detectedItems: DetectedItem[]
): { redactedText: string; appliedRedactions: Redaction[] } {
  if (detectedItems.length === 0) {
    return { redactedText: text, appliedRedactions: [] };
  }

  const appliedRedactions: Redaction[] = [];
  let offset = 0; // Track offset as we modify the text
  let result = text;

  for (const item of detectedItems) {
    const pattern = Object.values(REDACTION_PATTERNS).find(
      (p) => p.type === item.type
    );
    if (!pattern) continue;

    const replacement = pattern.replacementTemplate;
    const adjustedStart = item.startIndex + offset;
    const adjustedEnd = item.endIndex + offset;

    // Replace in result string
    result =
      result.slice(0, adjustedStart) +
      replacement +
      result.slice(adjustedEnd);

    // Track the redaction
    appliedRedactions.push({
      type: item.type,
      startIndex: item.startIndex, // Original position
      endIndex: item.endIndex,
      replacementText: replacement,
      confidence: item.confidence,
    });

    // Update offset for next iteration
    offset += replacement.length - (item.endIndex - item.startIndex);
  }

  return { redactedText: result, appliedRedactions };
}

/**
 * Calculate safety score based on detected items
 */
export function calculateSafetyScore(detectedItems: DetectedItem[]): SafetyLevel {
  if (detectedItems.length === 0) {
    return 'safe';
  }

  // Check for high-risk patterns
  const hasHighRisk = detectedItems.some(
    (item) =>
      item.type === 'ssh_key' ||
      item.type === 'aws_key' ||
      item.type === 'jwt'
  );

  if (hasHighRisk) {
    return 'dangerous';
  }

  // Check for medium-risk patterns
  const hasMediumRisk = detectedItems.some(
    (item) =>
      item.type === 'api_key' ||
      item.type === 'env_var' ||
      item.type === 'email'
  );

  if (hasMediumRisk || detectedItems.length > 3) {
    return 'moderate';
  }

  // Low-risk items (paths, IPs)
  return 'moderate';
}

/**
 * Perform complete privacy scan
 * Detects sensitive items, applies redactions, and calculates safety score
 */
export function performPrivacyScan(
  text: string,
  options: PrivacyScanOptions = {}
): PrivacyScanResult {
  const { autoApplyRedactions = false } = options;

  // Detect sensitive items
  const detectedItems = scanText(text, options);

  // Calculate safety score
  const safetyScore = calculateSafetyScore(detectedItems);

  // Apply redactions if requested
  let redactedText = text;
  let appliedRedactions: Redaction[] = [];

  if (autoApplyRedactions && detectedItems.length > 0) {
    const result = applyRedactions(text, detectedItems);
    redactedText = result.redactedText;
    appliedRedactions = result.appliedRedactions;
  }

  // Track which patterns were used
  const patternsUsed = Array.from(
    new Set(
      detectedItems.map((item) => {
        const pattern = Object.entries(REDACTION_PATTERNS).find(
          ([_, p]) => p.type === item.type
        );
        return pattern ? pattern[0] : 'UNKNOWN';
      })
    )
  );

  return {
    text,
    redactedText,
    detectedItems,
    appliedRedactions,
    safetyScore,
    scanTimestamp: new Date().toISOString(),
    patternsUsed,
  };
}

/**
 * Quick safety check without full redaction
 * Returns true if text appears safe to share
 */
export function isSafeToShare(
  text: string,
  minimumConfidence: number = 0.8
): boolean {
  const detectedItems = scanText(text, { minimumConfidence });
  return detectedItems.length === 0;
}

/**
 * Get redaction statistics
 */
export function getRedactionStats(scanResult: PrivacyScanResult): {
  totalDetected: number;
  byType: Record<string, number>;
  averageConfidence: number;
  riskLevel: SafetyLevel;
} {
  const { detectedItems, safetyScore } = scanResult;

  const byType: Record<string, number> = {};
  let totalConfidence = 0;

  for (const item of detectedItems) {
    byType[item.type] = (byType[item.type] || 0) + 1;
    totalConfidence += item.confidence;
  }

  return {
    totalDetected: detectedItems.length,
    byType,
    averageConfidence:
      detectedItems.length > 0 ? totalConfidence / detectedItems.length : 1.0,
    riskLevel: safetyScore,
  };
}

/**
 * Custom pattern registration
 * Allows users to add custom redaction patterns
 */
export function registerCustomPattern(
  name: string,
  pattern: RedactionPattern
): void {
  (REDACTION_PATTERNS as Record<string, RedactionPattern>)[name] = pattern;
}

/**
 * Disable a pattern
 */
export function disablePattern(name: string): void {
  const pattern = REDACTION_PATTERNS[name as keyof typeof REDACTION_PATTERNS];
  if (pattern) {
    pattern.enabled = false;
  }
}

/**
 * Enable a pattern
 */
export function enablePattern(name: string): void {
  const pattern = REDACTION_PATTERNS[name as keyof typeof REDACTION_PATTERNS];
  if (pattern) {
    pattern.enabled = true;
  }
}

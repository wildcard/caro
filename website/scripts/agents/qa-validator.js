/**
 * QA Validator Agent
 *
 * Validates generated content for quality, correctness, and consistency.
 * Checks YAML frontmatter, code examples, formatting, and tone.
 */

import { parse as parseYaml } from 'yaml';

/**
 * Validate generated content
 */
export async function validateContent({ content, contentType, config }) {
  const errors = [];
  const warnings = [];

  try {
    // Extract frontmatter and body
    const { frontmatter, body } = extractFrontmatter(content.markdown);

    // Validate YAML frontmatter
    const frontmatterResult = validateFrontmatter(frontmatter, contentType);
    errors.push(...frontmatterResult.errors);
    warnings.push(...frontmatterResult.warnings);

    // Validate code examples
    const codeResult = validateCodeExamples(body);
    errors.push(...codeResult.errors);
    warnings.push(...codeResult.warnings);

    // Validate formatting
    const formatResult = validateFormatting(body, contentType);
    errors.push(...formatResult.errors);
    warnings.push(...formatResult.warnings);

    // Validate reading time accuracy
    const readingTimeResult = validateReadingTime(content, frontmatter);
    warnings.push(...readingTimeResult.warnings);

    // Validate content length
    const lengthResult = validateContentLength(body, contentType);
    errors.push(...lengthResult.errors);
    warnings.push(...lengthResult.warnings);

    // Validate links and references
    const linkResult = validateLinks(body);
    warnings.push(...linkResult.warnings);

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      metrics: {
        wordCount: content.wordCount,
        readingTime: content.readingTime,
        codeBlockCount: (body.match(/```/g) || []).length / 2,
        headingCount: (body.match(/^#{1,6}\s/gm) || []).length,
      },
    };
  } catch (error) {
    errors.push(`Validation error: ${error.message}`);
    return {
      valid: false,
      errors,
      warnings,
      metrics: {},
    };
  }
}

/**
 * Extract frontmatter and body from markdown
 */
function extractFrontmatter(markdown) {
  const frontmatterMatch = markdown.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);

  if (!frontmatterMatch) {
    return { frontmatter: {}, body: markdown };
  }

  try {
    const frontmatter = parseYaml(frontmatterMatch[1]);
    const body = frontmatterMatch[2];
    return { frontmatter, body };
  } catch {
    return { frontmatter: {}, body: markdown };
  }
}

/**
 * Validate YAML frontmatter completeness
 */
function validateFrontmatter(frontmatter, contentType) {
  const errors = [];
  const warnings = [];

  // Required fields by content type
  const requiredFields = {
    command: ['title', 'command', 'description', 'difficulty', 'platforms', 'publishedAt'],
    story: ['title', 'category', 'era', 'publishedAt'],
    'daily-pick': ['title', 'type', 'publishedAt', 'socialText'],
  };

  const required = requiredFields[contentType] || [];

  for (const field of required) {
    if (!frontmatter[field]) {
      errors.push(`Missing required frontmatter field: ${field}`);
    }
  }

  // Validate specific field values
  if (frontmatter.difficulty) {
    const validDifficulties = ['beginner', 'intermediate', 'advanced'];
    if (!validDifficulties.includes(frontmatter.difficulty)) {
      errors.push(`Invalid difficulty: ${frontmatter.difficulty}`);
    }
  }

  if (frontmatter.platforms) {
    const validPlatforms = ['linux', 'macos', 'bsd', 'unix', 'posix'];
    for (const platform of frontmatter.platforms) {
      if (!validPlatforms.includes(platform)) {
        warnings.push(`Unknown platform: ${platform}`);
      }
    }
  }

  // Validate date format
  if (frontmatter.publishedAt) {
    const dateStr = String(frontmatter.publishedAt);
    if (!/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
      warnings.push(`Invalid date format: ${dateStr} (expected YYYY-MM-DD)`);
    }
  }

  // Check for social text length (daily picks)
  if (frontmatter.socialText && frontmatter.socialText.length > 280) {
    errors.push(`Social text exceeds 280 characters (${frontmatter.socialText.length})`);
  }

  return { errors, warnings };
}

/**
 * Validate code examples
 */
function validateCodeExamples(body) {
  const errors = [];
  const warnings = [];

  // Extract all code blocks
  const codeBlocks = body.match(/```(\w*)\n([\s\S]*?)```/g) || [];

  if (codeBlocks.length === 0) {
    warnings.push('No code examples found in content');
  }

  for (const block of codeBlocks) {
    const langMatch = block.match(/```(\w*)/);
    const language = langMatch ? langMatch[1] : '';
    const code = block.replace(/```\w*\n/, '').replace(/```$/, '').trim();

    // Check for empty code blocks
    if (!code) {
      warnings.push('Empty code block found');
      continue;
    }

    // Check for language specification
    if (!language) {
      warnings.push('Code block without language specification');
    }

    // Check for placeholder text
    if (code.includes('[command]') || code.includes('[example]')) {
      errors.push('Code block contains placeholder text');
    }

    // Check bash/shell code for basic validity
    if (language === 'bash' || language === 'shell' || language === 'sh') {
      // Check for common shell issues
      if (code.includes('rm -rf /') && !code.includes('#')) {
        warnings.push('Potentially dangerous command without comment');
      }

      // Check for unmatched quotes
      const singleQuotes = (code.match(/'/g) || []).length;
      const doubleQuotes = (code.match(/"/g) || []).length;
      if (singleQuotes % 2 !== 0) {
        warnings.push('Unmatched single quotes in bash code');
      }
      if (doubleQuotes % 2 !== 0) {
        warnings.push('Unmatched double quotes in bash code');
      }
    }
  }

  return { errors, warnings };
}

/**
 * Validate content formatting
 */
function validateFormatting(body, contentType) {
  const errors = [];
  const warnings = [];

  // Check for main heading
  if (!body.match(/^#\s+/m)) {
    warnings.push('No main heading (H1) found');
  }

  // Check heading hierarchy
  const headings = body.match(/^(#{1,6})\s+/gm) || [];
  let lastLevel = 0;
  for (const heading of headings) {
    const level = heading.match(/^(#+)/)[1].length;
    if (level > lastLevel + 1 && lastLevel > 0) {
      warnings.push(`Heading level skip detected (H${lastLevel} to H${level})`);
    }
    lastLevel = level;
  }

  // Check for required sections by content type
  const requiredSections = {
    command: ['Quick Summary', 'Deep Dive', 'Real-World Examples', 'Caro Connection'],
    story: ['Modern Relevance', 'Try It With Caro'],
    'daily-pick': [],
  };

  const sections = requiredSections[contentType] || [];
  for (const section of sections) {
    if (!body.toLowerCase().includes(section.toLowerCase())) {
      warnings.push(`Missing recommended section: ${section}`);
    }
  }

  // Check for broken markdown
  const linkPattern = /\[([^\]]*)\]\(([^)]*)\)/g;
  let match;
  while ((match = linkPattern.exec(body)) !== null) {
    const [, text, url] = match;
    if (!text || !url) {
      warnings.push('Potentially broken markdown link');
    }
  }

  return { errors, warnings };
}

/**
 * Validate reading time accuracy
 */
function validateReadingTime(content, frontmatter) {
  const warnings = [];

  // Compare calculated vs declared reading time
  if (frontmatter.readingTime && content.readingTime) {
    const diff = Math.abs(frontmatter.readingTime - content.readingTime);
    if (diff > 2) {
      warnings.push(
        `Reading time mismatch: declared ${frontmatter.readingTime}min vs calculated ${content.readingTime}min`
      );
    }
  }

  return { warnings };
}

/**
 * Validate content length
 */
function validateContentLength(body, contentType) {
  const errors = [];
  const warnings = [];

  const wordCount = body.split(/\s+/).filter((w) => w.length > 0).length;

  // Minimum word counts by type
  const minWords = {
    command: 500,
    story: 800,
    'daily-pick': 100,
  };

  const maxWords = {
    command: 3000,
    story: 5000,
    'daily-pick': 500,
  };

  const min = minWords[contentType] || 100;
  const max = maxWords[contentType] || 5000;

  if (wordCount < min) {
    warnings.push(`Content may be too short: ${wordCount} words (minimum: ${min})`);
  }

  if (wordCount > max) {
    warnings.push(`Content may be too long: ${wordCount} words (maximum: ${max})`);
  }

  return { errors, warnings };
}

/**
 * Validate links and references
 */
function validateLinks(body) {
  const warnings = [];

  // Check for relative links that might be broken
  const relativeLinks = body.match(/\]\((?!https?:\/\/)([^)]+)\)/g) || [];
  if (relativeLinks.length > 0) {
    warnings.push(`Found ${relativeLinks.length} relative link(s) - verify they work`);
  }

  // Check for placeholder URLs
  const placeholderUrls = body.match(/\]\((https?:\/\/example\.com|#|javascript:)[^)]*\)/g) || [];
  if (placeholderUrls.length > 0) {
    warnings.push(`Found ${placeholderUrls.length} placeholder URL(s)`);
  }

  return { warnings };
}

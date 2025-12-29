#!/usr/bin/env node

/**
 * Content Validation Runner
 *
 * Standalone script for validating generated content.
 * Used by GitHub Actions to validate before PR creation.
 *
 * Sets GitHub Actions outputs for workflow integration.
 */

import { readFileSync, readdirSync, writeFileSync, existsSync, appendFileSync } from 'fs';
import { join, dirname, extname } from 'path';
import { fileURLToPath } from 'url';
import { validateContent } from './agents/qa-validator.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONTENT_DIR = join(__dirname, '../src/content');

/**
 * Main validation function
 */
async function main() {
  console.log('🔍 Content Validation Runner\n');

  const results = {
    validated: [],
    errors: [],
    warnings: [],
  };

  // Get recently modified content files
  const contentTypes = ['commands', 'stories', 'daily-picks'];

  for (const contentType of contentTypes) {
    const typeDir = join(CONTENT_DIR, contentType);

    if (!existsSync(typeDir)) {
      continue;
    }

    const files = readdirSync(typeDir).filter(
      (f) => extname(f) === '.md' || extname(f) === '.mdx'
    );

    for (const file of files) {
      const filePath = join(typeDir, file);
      const markdown = readFileSync(filePath, 'utf-8');

      // Map directory name to content type
      const typeMap = {
        commands: 'command',
        stories: 'story',
        'daily-picks': 'daily-pick',
      };

      const validationType = typeMap[contentType];

      console.log(`Validating: ${file}...`);

      try {
        const validation = await validateContent({
          content: {
            markdown,
            filename: file,
            wordCount: markdown.split(/\s+/).length,
            readingTime: Math.ceil(markdown.split(/\s+/).length / 200),
          },
          contentType: validationType,
          config: {},
        });

        results.validated.push({
          file,
          contentType: validationType,
          valid: validation.valid,
          metrics: validation.metrics,
        });

        if (validation.errors.length > 0) {
          console.log(`  ❌ Errors: ${validation.errors.join(', ')}`);
          results.errors.push({ file, errors: validation.errors });
        }

        if (validation.warnings.length > 0) {
          console.log(`  ⚠️ Warnings: ${validation.warnings.join(', ')}`);
          results.warnings.push({ file, warnings: validation.warnings });
        }

        if (validation.valid && validation.warnings.length === 0) {
          console.log('  ✅ Valid');
        }
      } catch (error) {
        console.error(`  ❌ Error: ${error.message}`);
        results.errors.push({ file, errors: [error.message] });
      }
    }
  }

  // Generate summary
  console.log('\n' + '='.repeat(50));
  console.log('📊 Validation Summary');
  console.log('='.repeat(50));
  console.log(`Files validated: ${results.validated.length}`);
  console.log(`Files with errors: ${results.errors.length}`);
  console.log(`Files with warnings: ${results.warnings.length}`);

  // Set GitHub Actions outputs
  setGitHubOutput('has_content', results.validated.length > 0 ? 'true' : 'false');
  setGitHubOutput('has_errors', results.errors.length > 0 ? 'true' : 'false');
  setGitHubOutput('has_warnings', results.warnings.length > 0 ? 'true' : 'false');
  setGitHubOutput('validated_count', results.validated.length.toString());

  // Generate PR details
  if (results.validated.length > 0) {
    const contentTypes = [...new Set(results.validated.map((v) => v.contentType))];
    const titles = results.validated.map((v) => v.file.replace(/\.mdx?$/, ''));

    setGitHubOutput('pr_title', `[Content] Add ${results.validated.length} ${contentTypes.join('/')} item(s)`);

    const prBody = generatePRBody(results);
    setGitHubOutput('pr_body', prBody);

    const date = new Date().toISOString().split('T')[0];
    setGitHubOutput('branch_name', `content/${contentTypes[0]}/${date}`);
    setGitHubOutput('commit_message', `Add ${titles.join(', ')}`);
  }

  // Exit with error if validation failed
  if (results.errors.length > 0) {
    process.exit(1);
  }
}

/**
 * Set GitHub Actions output
 */
function setGitHubOutput(name, value) {
  const outputFile = process.env.GITHUB_OUTPUT;

  if (outputFile) {
    // Escape multiline values
    const escapedValue = value.replace(/%/g, '%25').replace(/\n/g, '%0A').replace(/\r/g, '%0D');
    appendFileSync(outputFile, `${name}=${escapedValue}\n`);
  }

  console.log(`::set-output name=${name}::${value.split('\n')[0]}${value.includes('\n') ? '...' : ''}`);
}

/**
 * Generate PR body
 */
function generatePRBody(results) {
  const lines = [];

  lines.push('## Content Generation Summary\n');
  lines.push(`**Files Validated:** ${results.validated.length}`);
  lines.push(`**Errors:** ${results.errors.length}`);
  lines.push(`**Warnings:** ${results.warnings.length}\n`);

  lines.push('### Files\n');
  for (const item of results.validated) {
    const status = results.errors.find((e) => e.file === item.file)
      ? '❌'
      : results.warnings.find((w) => w.file === item.file)
        ? '⚠️'
        : '✅';
    lines.push(`- ${status} \`${item.file}\` (${item.contentType})`);
  }

  if (results.warnings.length > 0) {
    lines.push('\n### Warnings\n');
    for (const warning of results.warnings) {
      lines.push(`- **${warning.file}:** ${warning.warnings.join(', ')}`);
    }
  }

  if (results.errors.length > 0) {
    lines.push('\n### Errors\n');
    for (const error of results.errors) {
      lines.push(`- **${error.file}:** ${error.errors.join(', ')}`);
    }
  }

  lines.push('\n---\n');
  lines.push('*This PR was automatically generated by the Content Generation Pipeline.*');

  return lines.join('\n');
}

// Run
main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});

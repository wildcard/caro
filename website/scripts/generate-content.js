#!/usr/bin/env node

/**
 * Content Generation Orchestrator
 *
 * Main entry point for automated content generation pipeline.
 * Coordinates sub-agents to plan, write, and validate content.
 *
 * Usage:
 *   ANTHROPIC_API_KEY=xxx CONTENT_TYPE=daily-pick node generate-content.js
 *
 * Environment Variables:
 *   ANTHROPIC_API_KEY - Claude API key (required)
 *   CONTENT_TYPE - Type of content: daily-pick, command, story, all
 *   DRY_RUN - If true, don't commit or create PR
 *   ITEM_COUNT - Number of items to generate (default: 1)
 */

import { execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync, readFileSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// ES module __dirname equivalent
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Import sub-agents
import { planContent } from './agents/content-planner.js';
import { writeContent } from './agents/content-writer.js';
import { validateContent } from './agents/qa-validator.js';
import { cacheSocialContent } from './agents/social-poster.js';
import { loadConfig, getContentQuota, determineContentType } from './lib/config.js';
import { createBranch, commitChanges, getExistingContent } from './lib/git-utils.js';
import { generateReport } from './lib/reporter.js';

// Configuration
const CONFIG = {
  contentDir: join(__dirname, '../src/content'),
  outputDir: join(__dirname, 'output'),
  maxRetries: 3,
  retryDelay: 2000,
};

/**
 * Main orchestration function
 */
async function main() {
  console.log('🚀 Starting Content Generation Pipeline\n');

  // Setup
  const startTime = Date.now();
  const report = {
    startTime: new Date().toISOString(),
    contentType: process.env.CONTENT_TYPE || 'daily-pick',
    itemCount: parseInt(process.env.ITEM_COUNT || '1', 10),
    dryRun: process.env.DRY_RUN === 'true',
    generated: [],
    errors: [],
    warnings: [],
  };

  // Ensure output directory exists
  if (!existsSync(CONFIG.outputDir)) {
    mkdirSync(CONFIG.outputDir, { recursive: true });
  }

  try {
    // Step 1: Load configuration and determine what to generate
    console.log('📋 Step 1: Loading configuration...');
    const config = await loadConfig();
    const contentType = determineContentType(report.contentType);
    const quota = getContentQuota(contentType);

    console.log(`   Content Type: ${contentType}`);
    console.log(`   Items to Generate: ${report.itemCount}`);
    console.log(`   Dry Run: ${report.dryRun}\n`);

    // Step 2: Analyze existing content for gaps
    console.log('🔍 Step 2: Analyzing existing content...');
    const existingContent = await getExistingContent(CONFIG.contentDir, contentType);
    console.log(`   Found ${existingContent.length} existing ${contentType} items\n`);

    // Step 3: Plan content (determine what to write)
    console.log('📝 Step 3: Planning content...');
    const contentBriefs = await planContent({
      contentType,
      existingContent,
      count: report.itemCount,
      config,
    });

    if (contentBriefs.length === 0) {
      console.log('   ⚠️ No content briefs generated. Exiting.');
      report.warnings.push('No content briefs generated');
      await generateReport(report, CONFIG.outputDir);
      return;
    }

    console.log(`   Generated ${contentBriefs.length} content briefs:\n`);
    contentBriefs.forEach((brief, i) => {
      console.log(`   ${i + 1}. ${brief.title} (${brief.difficulty || 'N/A'})`);
    });
    console.log('');

    // Step 4: Generate content for each brief
    console.log('✍️ Step 4: Generating content...');
    const generatedContent = [];

    for (const brief of contentBriefs) {
      console.log(`\n   Writing: ${brief.title}...`);

      try {
        const content = await writeContent({
          brief,
          contentType,
          config,
        });

        // Validate the generated content
        const validation = await validateContent({
          content,
          contentType,
          config,
        });

        if (validation.errors.length > 0) {
          console.log(`   ❌ Validation failed: ${validation.errors.join(', ')}`);
          report.errors.push({
            title: brief.title,
            errors: validation.errors,
          });
          continue;
        }

        if (validation.warnings.length > 0) {
          console.log(`   ⚠️ Warnings: ${validation.warnings.join(', ')}`);
          report.warnings.push(...validation.warnings);
        }

        generatedContent.push({
          brief,
          content,
          validation,
        });

        console.log(`   ✅ Generated successfully`);

      } catch (error) {
        console.error(`   ❌ Error: ${error.message}`);
        report.errors.push({
          title: brief.title,
          error: error.message,
        });
      }
    }

    // Step 5: Write content to files
    if (generatedContent.length > 0 && !report.dryRun) {
      console.log('\n💾 Step 5: Writing files...');

      // Create feature branch
      const branchName = createBranch(contentType);
      console.log(`   Created branch: ${branchName}`);

      for (const item of generatedContent) {
        const filename = item.content.filename;
        const filepath = join(CONFIG.contentDir, getContentSubdir(contentType), filename);

        writeFileSync(filepath, item.content.markdown);
        console.log(`   Wrote: ${filepath}`);

        report.generated.push({
          title: item.brief.title,
          filename,
          filepath,
        });
      }

      // Cache social content for dashboard
      console.log(`\n📱 Caching social content...`);
      for (const item of generatedContent) {
        await cacheSocialContent({
          contentType,
          title: item.brief.title,
          description: item.brief.description,
          command: item.brief.command,
          slug: item.content.filename.replace(/\.mdx?$/, ''),
          hashtags: item.brief.hashtags || ['unix', 'cli', 'terminal', 'caro'],
        });
        console.log(`   Cached: ${item.brief.title}`);
      }

      // Commit changes
      const commitMessage = generateCommitMessage(contentType, generatedContent);
      await commitChanges(commitMessage);
      console.log(`   Committed: ${commitMessage}`);

    } else if (report.dryRun) {
      console.log('\n🔬 Step 5: Dry run - skipping file writes');

      // Save generated content to output directory for review
      for (const item of generatedContent) {
        const filename = item.content.filename;
        const filepath = join(CONFIG.outputDir, filename);
        writeFileSync(filepath, item.content.markdown);
        console.log(`   Preview saved: ${filepath}`);

        report.generated.push({
          title: item.brief.title,
          filename,
          filepath,
          preview: true,
        });
      }
    }

    // Step 6: Generate report
    report.endTime = new Date().toISOString();
    report.duration = Date.now() - startTime;
    report.success = report.errors.length === 0;

    await generateReport(report, CONFIG.outputDir);

    console.log('\n' + '='.repeat(50));
    console.log('📊 Generation Complete');
    console.log('='.repeat(50));
    console.log(`   Duration: ${(report.duration / 1000).toFixed(1)}s`);
    console.log(`   Generated: ${report.generated.length} items`);
    console.log(`   Errors: ${report.errors.length}`);
    console.log(`   Warnings: ${report.warnings.length}`);

    // Exit with error code if there were failures
    if (report.errors.length > 0) {
      process.exit(1);
    }

  } catch (error) {
    console.error('\n❌ Pipeline failed:', error.message);
    report.errors.push({ fatal: error.message });
    report.endTime = new Date().toISOString();
    report.duration = Date.now() - startTime;
    await generateReport(report, CONFIG.outputDir);
    process.exit(1);
  }
}

/**
 * Get content subdirectory based on type
 */
function getContentSubdir(contentType) {
  const map = {
    'daily-pick': 'daily-picks',
    'command': 'commands',
    'story': 'stories',
  };
  return map[contentType] || contentType;
}

/**
 * Generate commit message for the content
 */
function generateCommitMessage(contentType, items) {
  if (items.length === 1) {
    const item = items[0];
    return `Add ${contentType}: ${item.brief.title}`;
  }

  const titles = items.map(i => i.brief.title).join(', ');
  return `Add ${items.length} ${contentType} items: ${titles}`;
}

// Run main function
main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});

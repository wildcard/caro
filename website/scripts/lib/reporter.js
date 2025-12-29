/**
 * Report Generator
 *
 * Generates detailed reports of content generation runs.
 * Outputs both JSON and markdown formats.
 */

import { writeFileSync } from 'fs';
import { join } from 'path';

/**
 * Generate comprehensive report
 */
export async function generateReport(report, outputDir) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');

  // Generate JSON report
  const jsonReport = generateJsonReport(report);
  const jsonPath = join(outputDir, `report-${timestamp}.json`);
  writeFileSync(jsonPath, JSON.stringify(jsonReport, null, 2));

  // Generate markdown report
  const markdownReport = generateMarkdownReport(report);
  const mdPath = join(outputDir, `report-${timestamp}.md`);
  writeFileSync(mdPath, markdownReport);

  // Generate summary for GitHub Actions
  if (process.env.GITHUB_STEP_SUMMARY) {
    writeFileSync(process.env.GITHUB_STEP_SUMMARY, markdownReport, { flag: 'a' });
  }

  return { jsonPath, mdPath };
}

/**
 * Generate JSON report structure
 */
function generateJsonReport(report) {
  return {
    metadata: {
      startTime: report.startTime,
      endTime: report.endTime,
      duration: report.duration,
      contentType: report.contentType,
      itemCount: report.itemCount,
      dryRun: report.dryRun,
      success: report.success,
    },
    results: {
      generated: report.generated.map((item) => ({
        title: item.title,
        filename: item.filename,
        filepath: item.filepath,
        preview: item.preview || false,
      })),
      errors: report.errors,
      warnings: report.warnings,
    },
    summary: {
      totalGenerated: report.generated.length,
      totalErrors: report.errors.length,
      totalWarnings: report.warnings.length,
      successRate:
        report.itemCount > 0
          ? ((report.generated.length / report.itemCount) * 100).toFixed(1) + '%'
          : 'N/A',
    },
  };
}

/**
 * Generate markdown report
 */
function generateMarkdownReport(report) {
  const lines = [];

  // Header
  lines.push('# Content Generation Report');
  lines.push('');
  lines.push(`**Generated:** ${new Date(report.startTime).toLocaleString()}`);
  lines.push(`**Content Type:** ${report.contentType}`);
  lines.push(`**Mode:** ${report.dryRun ? 'Dry Run' : 'Production'}`);
  lines.push('');

  // Summary
  lines.push('## Summary');
  lines.push('');
  lines.push('| Metric | Value |');
  lines.push('|--------|-------|');
  lines.push(`| Items Requested | ${report.itemCount} |`);
  lines.push(`| Items Generated | ${report.generated.length} |`);
  lines.push(`| Errors | ${report.errors.length} |`);
  lines.push(`| Warnings | ${report.warnings.length} |`);
  lines.push(`| Duration | ${formatDuration(report.duration)} |`);
  lines.push(
    `| Success Rate | ${report.itemCount > 0 ? ((report.generated.length / report.itemCount) * 100).toFixed(1) : 0}% |`
  );
  lines.push('');

  // Generated Content
  if (report.generated.length > 0) {
    lines.push('## Generated Content');
    lines.push('');

    for (const item of report.generated) {
      const status = item.preview ? '(preview)' : '';
      lines.push(`### ${item.title} ${status}`);
      lines.push('');
      lines.push(`- **Filename:** \`${item.filename}\``);
      lines.push(`- **Path:** \`${item.filepath}\``);
      lines.push('');
    }
  }

  // Errors
  if (report.errors.length > 0) {
    lines.push('## Errors');
    lines.push('');

    for (const error of report.errors) {
      if (error.fatal) {
        lines.push(`- **Fatal:** ${error.fatal}`);
      } else if (error.title) {
        lines.push(`- **${error.title}:** ${error.errors?.join(', ') || error.error}`);
      } else {
        lines.push(`- ${JSON.stringify(error)}`);
      }
    }
    lines.push('');
  }

  // Warnings
  if (report.warnings.length > 0) {
    lines.push('## Warnings');
    lines.push('');

    for (const warning of report.warnings) {
      lines.push(`- ${warning}`);
    }
    lines.push('');
  }

  // Status badge for README
  lines.push('---');
  lines.push('');
  if (report.success) {
    lines.push('![Status](https://img.shields.io/badge/status-success-green)');
  } else if (report.errors.length > 0) {
    lines.push('![Status](https://img.shields.io/badge/status-failed-red)');
  } else {
    lines.push('![Status](https://img.shields.io/badge/status-warning-yellow)');
  }

  return lines.join('\n');
}

/**
 * Format duration in human-readable format
 */
function formatDuration(ms) {
  if (!ms) return 'N/A';

  if (ms < 1000) {
    return `${ms}ms`;
  }

  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) {
    return `${seconds}s`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}

/**
 * Generate GitHub Actions annotation
 */
export function generateAnnotation(type, message, file = null, line = null) {
  let annotation = `::${type}`;

  if (file || line) {
    const params = [];
    if (file) params.push(`file=${file}`);
    if (line) params.push(`line=${line}`);
    annotation += ` ${params.join(',')}`;
  }

  annotation += `::${message}`;

  console.log(annotation);
}

/**
 * Generate summary statistics for content database
 */
export function generateContentStats(analysis) {
  return {
    commands: {
      total: analysis.commands.covered.length,
      byDifficulty: analysis.commands.byDifficulty,
      coverage: calculateCoverage(analysis.commands.covered.length, 100), // Assume 100 total commands
    },
    stories: {
      total: analysis.stories.covered.length,
      byCategory: analysis.stories.byCategory,
      byEra: analysis.stories.byEra,
    },
    dailyPicks: {
      total: analysis.dailyPicks.total,
      byType: analysis.dailyPicks.byType,
      daysOfContent: analysis.dailyPicks.total, // 1 per day
    },
  };
}

/**
 * Calculate coverage percentage
 */
function calculateCoverage(covered, total) {
  if (total === 0) return '0%';
  return ((covered / total) * 100).toFixed(1) + '%';
}

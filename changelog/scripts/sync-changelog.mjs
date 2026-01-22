#!/usr/bin/env node
/**
 * Sync changelog content from CHANGELOG.md to Astro content files
 *
 * This script parses the root CHANGELOG.md file and generates individual
 * release markdown files for the Astro changelog website.
 *
 * Usage: node scripts/sync-changelog.mjs
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = join(__dirname, '..', '..');
const CHANGELOG_PATH = join(ROOT_DIR, 'CHANGELOG.md');
const RELEASES_DIR = join(__dirname, '..', 'src', 'content', 'releases');

// Ensure releases directory exists
if (!existsSync(RELEASES_DIR)) {
  mkdirSync(RELEASES_DIR, { recursive: true });
}

// Read the CHANGELOG.md file
const changelog = readFileSync(CHANGELOG_PATH, 'utf-8');

// Parse version entries
const versionRegex = /^## \[(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})$/gm;
const sections = changelog.split(/^## \[/gm).filter(s => s.trim());

// Skip the header section (before first version)
const releases = [];

for (const section of sections) {
  const versionMatch = section.match(/^(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})/);
  if (!versionMatch) continue;

  const version = versionMatch[1];
  const date = versionMatch[2];

  // Get the content after the version line
  const content = section.replace(/^(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})\n*/, '').trim();

  // Determine release characteristics
  const isBreaking = content.toLowerCase().includes('breaking change') ||
                     content.includes('BREAKING CHANGE');
  const isSecurity = content.toLowerCase().includes('### security') ||
                     section.toLowerCase().includes('security audit');

  // Generate title from first major heading or content summary
  let title = generateTitle(version, content);
  let description = generateDescription(content);

  releases.push({
    version,
    date,
    title,
    description,
    content,
    isBreaking,
    isSecurity
  });
}

function generateTitle(version, content) {
  // Try to extract a meaningful title from the content
  if (version === '1.0.0') return 'First Stable Release';

  // Look for major feature additions
  const addedMatch = content.match(/### Added\n+(?:.*\n)*?-\s*\*\*([^*]+)\*\*/);
  if (addedMatch) {
    return addedMatch[1].replace(/:$/, '').trim();
  }

  // Look for major fixes
  const fixedMatch = content.match(/### Fixed\n+(?:.*\n)*?(?:####\s*)?([^\n]+)/);
  if (fixedMatch) {
    return fixedMatch[1].replace(/^[#\s]+/, '').trim();
  }

  // Look for major changes
  const changedMatch = content.match(/### Changed\n+(?:.*\n)*?(?:####\s*)?([^\n]+)/);
  if (changedMatch) {
    return changedMatch[1].replace(/^[#\s]+/, '').trim();
  }

  return `Version ${version} Release`;
}

function generateDescription(content) {
  // Extract first meaningful paragraph or list item
  const lines = content.split('\n').filter(l => l.trim() && !l.startsWith('#'));
  const firstMeaningful = lines.find(l => l.trim().length > 20);

  if (firstMeaningful) {
    let desc = firstMeaningful
      .replace(/^[-*]\s*/, '')
      .replace(/\*\*/g, '')
      .replace(/`/g, '')
      .trim();

    // Truncate if too long
    if (desc.length > 200) {
      desc = desc.substring(0, 197) + '...';
    }
    return desc;
  }

  return `Release notes for version ${content.match(/(\d+\.\d+\.\d+)/)?.[1] || 'unknown'}`;
}

// Write individual release files
for (const release of releases) {
  const frontmatter = `---
title: "${release.title}"
description: "${release.description.replace(/"/g, '\\"')}"
versionNumber: "${release.version}"
date: ${release.date}
breaking: ${release.isBreaking}
security: ${release.isSecurity}
---

`;

  const fileContent = frontmatter + release.content;
  // Use underscores instead of dots for Astro content collection IDs
  const filename = release.version.replace(/\./g, '_');
  const filePath = join(RELEASES_DIR, `${filename}.md`);

  writeFileSync(filePath, fileContent);
  console.log(`Generated: ${filename}.md (v${release.version})`);
}

console.log(`\nSync complete! Generated ${releases.length} release files.`);

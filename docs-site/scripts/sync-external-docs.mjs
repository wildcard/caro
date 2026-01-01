#!/usr/bin/env node
/**
 * Sync External Docs Script
 *
 * Generates Starlight-compatible documentation pages from external
 * markdown files in the /docs directory.
 *
 * Usage: node scripts/sync-external-docs.mjs
 *
 * This script:
 * 1. Reads the docs.config.mjs configuration
 * 2. Processes each file in the include list
 * 3. Adds/updates frontmatter for Starlight compatibility
 * 4. Writes output to src/content/docs/external/
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const projectRoot = path.resolve(rootDir, '..');

// Load config
const configPath = path.join(rootDir, 'docs.config.mjs');
const config = (await import(configPath)).default;

const sourceDir = path.resolve(rootDir, config.sourceDir);
const outputDir = path.resolve(rootDir, config.outputDir);

console.log('Syncing external docs...');
console.log(`  Source: ${sourceDir}`);
console.log(`  Output: ${outputDir}`);

// Ensure output directory exists
fs.mkdirSync(outputDir, { recursive: true });

// Track processed files for cleanup
const processedFiles = new Set();

/**
 * Extract frontmatter from markdown content
 */
function extractFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  if (match) {
    const frontmatter = {};
    match[1].split('\n').forEach(line => {
      const [key, ...valueParts] = line.split(':');
      if (key && valueParts.length) {
        frontmatter[key.trim()] = valueParts.join(':').trim().replace(/^["']|["']$/g, '');
      }
    });
    return { frontmatter, content: match[2] };
  }
  return { frontmatter: {}, content };
}

/**
 * Generate frontmatter string from object
 */
function generateFrontmatter(fm) {
  const lines = ['---'];
  for (const [key, value] of Object.entries(fm)) {
    if (typeof value === 'string') {
      // Escape quotes in string values
      const escaped = value.includes(':') || value.includes('"') ? `"${value.replace(/"/g, '\\"')}"` : value;
      lines.push(`${key}: ${escaped}`);
    } else if (typeof value === 'boolean') {
      lines.push(`${key}: ${value}`);
    } else if (typeof value === 'number') {
      lines.push(`${key}: ${value}`);
    }
  }
  lines.push('---\n');
  return lines.join('\n');
}

/**
 * Clean up markdown content for Starlight compatibility
 */
function cleanContent(content, sourcePath) {
  let cleaned = content;

  // Remove duplicate h1 if it matches the title
  // (Many docs have # Title that duplicates the frontmatter title)
  cleaned = cleaned.replace(/^#\s+[^\n]+\n+/, '');

  // Fix relative links to point to correct location
  // Links like [text](../FILENAME.md) need to be updated
  cleaned = cleaned.replace(/\[([^\]]+)\]\(\.\.\/([^)]+)\)/g, (match, text, href) => {
    // If it's a link to another doc that will be included, update it
    const found = config.include.find(inc => inc.source === href || inc.source.endsWith(href));
    if (found) {
      const newHref = '/' + found.target.replace(/\.md$/, '/');
      return `[${text}](${newHref})`;
    }
    return match;
  });

  // Fix image paths
  cleaned = cleaned.replace(/!\[([^\]]*)\]\(\.\.\/([^)]+)\)/g, '![$1](/images/$2)');

  return cleaned;
}

/**
 * Get group from target path
 */
function getGroup(targetPath) {
  const parts = targetPath.split('/');
  if (parts.length > 1) {
    return parts[0];
  }
  return 'guides';  // Default group
}

/**
 * Process a single file
 */
function processFile(item) {
  const sourcePath = path.join(sourceDir, item.source);
  const targetPath = path.join(outputDir, item.target);

  // Check if source exists
  if (!fs.existsSync(sourcePath)) {
    console.warn(`  Warning: Source file not found: ${item.source}`);
    return false;
  }

  // Read source content
  let content = fs.readFileSync(sourcePath, 'utf-8');

  // Handle special case for DCO.txt
  if (item.source.endsWith('.txt')) {
    // Wrap in code block for text files
    content = '```\n' + content + '\n```';
  }

  // Extract existing frontmatter
  const { frontmatter: existingFm, content: bodyContent } = extractFrontmatter(content);

  // Build new frontmatter
  const newFm = {
    title: item.title || existingFm.title || path.basename(item.source, path.extname(item.source)),
    description: existingFm.description || `Documentation: ${item.title || item.source}`,
    ...config.defaultFrontmatter,
    ...existingFm,
  };

  // Override title if specified in config
  if (item.title) {
    newFm.title = item.title;
  }

  // Clean and transform content
  let finalContent = cleanContent(bodyContent, item.source);

  // Apply custom transform if defined
  if (config.transform) {
    finalContent = config.transform(finalContent, item.source);
  }

  // Generate final file
  const output = generateFrontmatter(newFm) + finalContent;

  // Ensure target directory exists
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });

  // Write output file
  fs.writeFileSync(targetPath, output, 'utf-8');
  processedFiles.add(targetPath);

  console.log(`  Synced: ${item.source} -> ${item.target}`);
  return true;
}

// Process all included files
let successCount = 0;
let failCount = 0;

for (const item of config.include) {
  if (processFile(item)) {
    successCount++;
  } else {
    failCount++;
  }
}

// Clean up orphaned files (files in output dir that weren't just generated)
function cleanOrphans(dir) {
  if (!fs.existsSync(dir)) return;

  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      cleanOrphans(fullPath);
      // Remove empty directories
      if (fs.readdirSync(fullPath).length === 0) {
        fs.rmdirSync(fullPath);
      }
    } else if (entry.isFile() && !processedFiles.has(fullPath)) {
      fs.unlinkSync(fullPath);
      console.log(`  Removed orphan: ${path.relative(outputDir, fullPath)}`);
    }
  }
}

cleanOrphans(outputDir);

console.log(`\nSync complete: ${successCount} files synced, ${failCount} failed`);

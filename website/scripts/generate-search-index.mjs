#!/usr/bin/env node
/**
 * Generate Search Index
 *
 * Scans all pages and resources to create a comprehensive search index
 * that includes every piece of content available on the website.
 *
 * Usage: node scripts/generate-search-index.mjs
 *
 * This script is run at build time to generate the search index.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEBSITE_ROOT = path.resolve(__dirname, '..');

// Categories based on path
function getCategoryFromPath(filePath) {
  if (filePath.includes('/blog/')) return 'blog';
  if (filePath.includes('/compare/')) return 'compare';
  if (filePath.includes('/use-cases/')) return 'use-cases';
  if (filePath.includes('/explore/')) return 'explore';
  if (filePath.includes('/docs/')) return 'docs';
  return 'main';
}

// Get icon based on category and path
function getIconFromPath(urlPath, category) {
  const iconMap = {
    blog: '📝',
    compare: '⚖️',
    'use-cases': '📋',
    explore: '🔍',
    docs: '📚',
    main: '📄',
  };

  // Special icons for specific pages
  if (urlPath.includes('sre')) return '🚨';
  if (urlPath.includes('air-gapped')) return '🔐';
  if (urlPath.includes('devops')) return '🔧';
  if (urlPath.includes('tech-lead')) return '👥';
  if (urlPath.includes('developer')) return '💻';
  if (urlPath.includes('safety') || urlPath.includes('security')) return '🛡️';
  if (urlPath.includes('roadmap')) return '🗺️';
  if (urlPath.includes('support')) return '💬';
  if (urlPath.includes('credits')) return '🙏';
  if (urlPath.includes('github') || urlPath.includes('copilot')) return '🐙';
  if (urlPath.includes('warp')) return '🚀';
  if (urlPath.includes('amazon')) return '📦';
  if (urlPath.includes('claude')) return '🤖';
  if (urlPath === '/' || urlPath === '') return '🏠';

  return iconMap[category] || '📄';
}

// Clean HTML and extract text
function extractTextContent(content) {
  let text = content
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/---[\s\S]*?---/g, '') // Remove frontmatter
    .replace(/<[^>]+>/g, ' ') // Remove HTML tags
    .replace(/\{[^}]+\}/g, ' ') // Remove JSX expressions
    .replace(/&[a-z]+;/gi, ' ') // Remove HTML entities
    .replace(/\s+/g, ' ') // Normalize whitespace
    .trim();

  return text;
}

// Extract specific content sections
function extractSections(content) {
  const sections = [];

  // Extract headings
  const headingMatches = content.matchAll(/<h[1-6][^>]*>([\s\S]*?)<\/h[1-6]>/gi);
  for (const match of headingMatches) {
    const text = extractTextContent(match[1]);
    if (text.length > 2) sections.push({ type: 'heading', text });
  }

  // Extract paragraphs
  const paragraphMatches = content.matchAll(/<p[^>]*>([\s\S]*?)<\/p>/gi);
  for (const match of paragraphMatches) {
    const text = extractTextContent(match[1]);
    if (text.length > 10) sections.push({ type: 'paragraph', text });
  }

  // Extract list items
  const listMatches = content.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/gi);
  for (const match of listMatches) {
    const text = extractTextContent(match[1]);
    if (text.length > 5) sections.push({ type: 'list', text });
  }

  return sections;
}

// Extract frontmatter metadata
function extractFrontmatter(content) {
  const frontmatterMatch = content.match(/---\n([\s\S]*?)\n---/);
  if (!frontmatterMatch) return {};

  const frontmatter = frontmatterMatch[1];
  const titleMatch = frontmatter.match(/title:\s*["']?([^"'\n]+)["']?/i);
  const descriptionMatch = frontmatter.match(/description:\s*["']?([^"'\n]+)["']?/i);

  return {
    title: titleMatch ? titleMatch[1].trim() : null,
    description: descriptionMatch ? descriptionMatch[1].trim() : null,
  };
}

// Convert file path to URL path
function filePathToUrl(filePath, pagesDir) {
  let url = filePath
    .replace(pagesDir, '')
    .replace(/\.astro$/, '')
    .replace(/\.md$/, '')
    .replace(/\.mdx$/, '')
    .replace(/\/index$/, '');

  return url || '/';
}

// Generate title from path
function generateTitleFromPath(urlPath) {
  if (urlPath === '/' || urlPath === '') return 'Home';

  const segments = urlPath.split('/').filter(Boolean);
  const lastSegment = segments[segments.length - 1];

  return lastSegment
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

// Scan a single page file
function scanPage(filePath, pagesDir) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const urlPath = filePathToUrl(filePath, pagesDir);
  const category = getCategoryFromPath(filePath);
  const frontmatter = extractFrontmatter(content);
  const sections = extractSections(content);
  const fullText = extractTextContent(content);

  // Generate keywords from content
  const keywords = new Set();

  // Add words from title
  const title = frontmatter.title || generateTitleFromPath(urlPath);
  if (title) {
    title.toLowerCase().split(/\s+/).forEach(w => {
      if (w.length > 2) keywords.add(w);
    });
  }

  // Add words from sections
  sections.forEach(section => {
    section.text.toLowerCase().split(/\s+/).forEach(w => {
      if (w.length > 3 && !['the', 'and', 'for', 'with', 'that', 'this', 'from'].includes(w)) {
        keywords.add(w);
      }
    });
  });

  // Limit keywords
  const keywordsArray = Array.from(keywords).slice(0, 20);

  const description = frontmatter.description ||
    sections.find(s => s.type === 'paragraph')?.text?.slice(0, 150) ||
    '';

  return {
    title: title || 'Untitled',
    path: urlPath,
    description,
    category,
    keywords: keywordsArray,
    icon: getIconFromPath(urlPath, category),
    // Full content for deep search
    content: {
      headings: sections.filter(s => s.type === 'heading').map(s => s.text),
      paragraphs: sections.filter(s => s.type === 'paragraph').map(s => s.text).slice(0, 10),
      listItems: sections.filter(s => s.type === 'list').map(s => s.text).slice(0, 20),
    },
    // Full text for fuzzy search (limited for performance)
    fullText: fullText.slice(0, 2000),
  };
}

// Scan components for additional content
function scanComponents(componentsDir) {
  const additionalContent = [];

  const scanDir = (dir) => {
    if (!fs.existsSync(dir)) return;

    const files = fs.readdirSync(dir);
    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);

      if (stat.isDirectory()) {
        scanDir(filePath);
      } else if (file.endsWith('.astro') || file.endsWith('.tsx')) {
        try {
          const content = fs.readFileSync(filePath, 'utf-8');
          const sections = extractSections(content);

          // Only include components with meaningful content
          if (sections.length > 0) {
            additionalContent.push({
              source: filePath.replace(componentsDir, ''),
              sections,
            });
          }
        } catch (e) {
          // Skip files that can't be read
        }
      }
    }
  };

  scanDir(componentsDir);
  return additionalContent;
}

// Main function
function generateSearchIndex() {
  console.log('🔍 Generating comprehensive search index...\n');

  const pagesDir = path.join(WEBSITE_ROOT, 'src/pages');
  const componentsDir = path.join(WEBSITE_ROOT, 'src/components');
  const outputPath = path.join(WEBSITE_ROOT, 'src/config/search-index.json');

  const pages = [];

  // Scan all pages
  const scanDir = (dir) => {
    const files = fs.readdirSync(dir);
    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);

      if (stat.isDirectory()) {
        scanDir(filePath);
      } else if (file.endsWith('.astro') || file.endsWith('.md') || file.endsWith('.mdx')) {
        try {
          const pageData = scanPage(filePath, pagesDir);
          pages.push(pageData);
          console.log(`  ✓ ${pageData.path} - ${pageData.title}`);
        } catch (e) {
          console.error(`  ✗ Error scanning ${filePath}:`, e.message);
        }
      }
    }
  };

  console.log('📄 Scanning pages...');
  scanDir(pagesDir);

  console.log('\n🧩 Scanning components for additional content...');
  const componentContent = scanComponents(componentsDir);
  console.log(`  Found ${componentContent.length} components with content\n`);

  // Create the search index
  const searchIndex = {
    version: '1.0.0',
    generated: new Date().toISOString(),
    totalPages: pages.length,
    pages: pages.map(p => ({
      ...p,
      // Pre-compute search strings
      _searchText: [
        p.title || '',
        p.description || '',
        p.path || '',
        ...(p.keywords || []),
        ...(p.content?.headings || []),
        ...(p.content?.paragraphs || []).slice(0, 3),
      ].join(' ').toLowerCase(),
      _words: [
        p.title || '',
        p.description || '',
        ...(p.keywords || []),
        ...(p.content?.headings || []),
      ].join(' ').toLowerCase().split(/\s+/).filter(w => w.length > 2),
    })),
    // Include component content for deep search
    componentContent: componentContent.slice(0, 50), // Limit for performance
  };

  // Write the index
  fs.writeFileSync(outputPath, JSON.stringify(searchIndex, null, 2));

  console.log('✅ Search index generated successfully!');
  console.log(`   Output: ${outputPath}`);
  console.log(`   Pages indexed: ${pages.length}`);
  console.log(`   Components with content: ${componentContent.length}`);
}

// Run
generateSearchIndex();

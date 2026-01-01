#!/usr/bin/env node
/**
 * Generate Search Index
 *
 * Scans all pages and resources to create a comprehensive search index
 * that includes EVERY piece of text content available on the website.
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

// Common stop words to exclude from keywords (but keep in full text)
const STOP_WORDS = new Set([
  'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
  'of', 'with', 'by', 'from', 'as', 'is', 'was', 'are', 'were', 'been',
  'be', 'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'could',
  'should', 'may', 'might', 'must', 'shall', 'can', 'need', 'dare', 'ought',
  'used', 'it', 'its', 'this', 'that', 'these', 'those', 'i', 'you', 'he',
  'she', 'we', 'they', 'what', 'which', 'who', 'whom', 'whose', 'where',
  'when', 'why', 'how', 'all', 'each', 'every', 'both', 'few', 'more',
  'most', 'other', 'some', 'such', 'no', 'nor', 'not', 'only', 'own',
  'same', 'so', 'than', 'too', 'very', 'just', 'also', 'now', 'here',
]);

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

// Extract ALL text content from HTML/Astro content
function extractAllTextContent(content) {
  let text = content
    // Remove code blocks first (they contain non-searchable content)
    .replace(/<code[\s\S]*?<\/code>/gi, ' ')
    .replace(/<pre[\s\S]*?<\/pre>/gi, ' ')
    // Remove script and style tags
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    // Remove frontmatter
    .replace(/---[\s\S]*?---/g, '')
    // Remove import statements
    .replace(/import\s+.*?from\s+['"][^'"]+['"]\s*;?/g, '')
    // Remove Astro/JSX expressions but keep text content
    .replace(/\{`([^`]*)`\}/g, '$1') // Template literals
    .replace(/\{['"]([^'"]*)['"]\}/g, '$1') // String literals
    .replace(/\{[^}]+\}/g, ' ') // Other expressions
    // Extract text from HTML tags
    .replace(/<[^>]+>/g, ' ')
    // Decode common HTML entities
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&quot;/gi, '"')
    .replace(/&#39;/gi, "'")
    .replace(/&[a-z]+;/gi, ' ')
    // Normalize whitespace
    .replace(/\s+/g, ' ')
    .trim();

  return text;
}

// Extract text from specific elements for structured content
function extractStructuredContent(content) {
  const result = {
    headings: [],
    paragraphs: [],
    listItems: [],
    spans: [],
    buttons: [],
    links: [],
    labels: [],
    allText: [],
  };

  // Extract headings (h1-h6)
  const headingMatches = content.matchAll(/<h[1-6][^>]*>([\s\S]*?)<\/h[1-6]>/gi);
  for (const match of headingMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
      result.headings.push(text);
      result.allText.push(text);
    }
  }

  // Extract paragraphs
  const paragraphMatches = content.matchAll(/<p[^>]*>([\s\S]*?)<\/p>/gi);
  for (const match of paragraphMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 3) {
      result.paragraphs.push(text);
      result.allText.push(text);
    }
  }

  // Extract list items
  const listMatches = content.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/gi);
  for (const match of listMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 2) {
      result.listItems.push(text);
      result.allText.push(text);
    }
  }

  // Extract spans with text content
  const spanMatches = content.matchAll(/<span[^>]*>([\s\S]*?)<\/span>/gi);
  for (const match of spanMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 2 && !text.match(/^[\d\s\-\+]+$/)) {
      result.spans.push(text);
      result.allText.push(text);
    }
  }

  // Extract button text
  const buttonMatches = content.matchAll(/<button[^>]*>([\s\S]*?)<\/button>/gi);
  for (const match of buttonMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
      result.buttons.push(text);
      result.allText.push(text);
    }
  }

  // Extract link text
  const linkMatches = content.matchAll(/<a[^>]*>([\s\S]*?)<\/a>/gi);
  for (const match of linkMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
      result.links.push(text);
      result.allText.push(text);
    }
  }

  // Extract div text (for content divs)
  const divMatches = content.matchAll(/<div[^>]*class="[^"]*(?:text|content|desc|title|label|message)[^"]*"[^>]*>([\s\S]*?)<\/div>/gi);
  for (const match of divMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 3) {
      result.allText.push(text);
    }
  }

  // Extract label text
  const labelMatches = content.matchAll(/<label[^>]*>([\s\S]*?)<\/label>/gi);
  for (const match of labelMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
      result.labels.push(text);
      result.allText.push(text);
    }
  }

  // Extract alt text from images
  const altMatches = content.matchAll(/alt=["']([^"']+)["']/gi);
  for (const match of altMatches) {
    const text = match[1].trim();
    if (text.length > 1) {
      result.allText.push(text);
    }
  }

  // Extract title attributes
  const titleMatches = content.matchAll(/title=["']([^"']+)["']/gi);
  for (const match of titleMatches) {
    const text = match[1].trim();
    if (text.length > 1) {
      result.allText.push(text);
    }
  }

  // Extract aria-label attributes
  const ariaLabelMatches = content.matchAll(/aria-label=["']([^"']+)["']/gi);
  for (const match of ariaLabelMatches) {
    const text = match[1].trim();
    if (text.length > 1) {
      result.allText.push(text);
    }
  }

  // Extract placeholder text
  const placeholderMatches = content.matchAll(/placeholder=["']([^"']+)["']/gi);
  for (const match of placeholderMatches) {
    const text = match[1].trim();
    if (text.length > 1) {
      result.allText.push(text);
    }
  }

  return result;
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

// Extract unique meaningful words for keyword generation
function extractKeywords(text, limit = 50) {
  const words = text.toLowerCase()
    .replace(/[^a-z0-9\s]/g, ' ')
    .split(/\s+/)
    .filter(w => w.length > 2 && !STOP_WORDS.has(w));

  // Count word frequency
  const wordCount = {};
  for (const word of words) {
    wordCount[word] = (wordCount[word] || 0) + 1;
  }

  // Sort by frequency and return top words
  return Object.entries(wordCount)
    .sort((a, b) => b[1] - a[1])
    .slice(0, limit)
    .map(([word]) => word);
}

// Scan a single page file
function scanPage(filePath, pagesDir) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const urlPath = filePathToUrl(filePath, pagesDir);
  const category = getCategoryFromPath(filePath);
  const frontmatter = extractFrontmatter(content);
  const structured = extractStructuredContent(content);
  const fullText = extractAllTextContent(content);

  // Generate title
  const title = frontmatter.title || generateTitleFromPath(urlPath);

  // Combine all text for comprehensive search
  const allTextContent = [
    title,
    frontmatter.description || '',
    ...structured.allText,
    fullText,
  ].join(' ');

  // Generate keywords from all content
  const keywords = extractKeywords(allTextContent, 50);

  // Get first paragraph for description if not in frontmatter
  const description = frontmatter.description ||
    structured.paragraphs[0]?.slice(0, 200) ||
    '';

  return {
    title: title || 'Untitled',
    path: urlPath,
    description,
    category,
    keywords,
    icon: getIconFromPath(urlPath, category),
    // Structured content for display
    content: {
      headings: structured.headings,
      paragraphs: structured.paragraphs.slice(0, 20),
      listItems: structured.listItems.slice(0, 30),
    },
    // FULL text content for comprehensive search - no limit!
    fullText: fullText,
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
          const structured = extractStructuredContent(content);
          const fullText = extractAllTextContent(content);

          // Only include components with meaningful content
          if (structured.allText.length > 0 || fullText.length > 50) {
            additionalContent.push({
              source: filePath.replace(componentsDir, ''),
              headings: structured.headings,
              paragraphs: structured.paragraphs.slice(0, 10),
              fullText: fullText,
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
          const textLength = pageData.fullText?.length || 0;
          console.log(`  ✓ ${pageData.path} - ${pageData.title} (${textLength} chars)`);
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

  // Create the search index with FULL text content
  const searchIndex = {
    version: '2.0.0', // Version bump for full-text search
    generated: new Date().toISOString(),
    totalPages: pages.length,
    pages: pages.map(p => ({
      ...p,
      // Pre-compute comprehensive search string including ALL text
      _searchText: [
        p.title || '',
        p.description || '',
        p.path || '',
        ...(p.keywords || []),
        ...(p.content?.headings || []),
        ...(p.content?.paragraphs || []),
        ...(p.content?.listItems || []),
        p.fullText || '',
      ].join(' ').toLowerCase(),
      // All unique words for fuzzy matching
      _words: extractKeywords([
        p.title || '',
        p.description || '',
        ...(p.keywords || []),
        ...(p.content?.headings || []),
        p.fullText || '',
      ].join(' '), 100),
    })),
    // Include component content for deep search
    componentContent: componentContent.map(c => ({
      ...c,
      _searchText: [
        ...c.headings,
        ...c.paragraphs,
        c.fullText,
      ].join(' ').toLowerCase(),
    })),
  };

  // Write the index
  fs.writeFileSync(outputPath, JSON.stringify(searchIndex, null, 2));

  // Calculate total indexed content
  const totalChars = pages.reduce((sum, p) => sum + (p.fullText?.length || 0), 0);
  const totalWords = pages.reduce((sum, p) => sum + (p._searchText?.split(/\s+/).length || 0), 0);

  console.log('✅ Search index generated successfully!');
  console.log(`   Output: ${outputPath}`);
  console.log(`   Pages indexed: ${pages.length}`);
  console.log(`   Components indexed: ${componentContent.length}`);
  console.log(`   Total content: ${totalChars.toLocaleString()} characters`);
}

// Run
generateSearchIndex();

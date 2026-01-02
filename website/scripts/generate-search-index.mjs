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
  if (urlPath.includes('opencode')) return '💡';
  if (urlPath.includes('claude')) return '🤖';
  if (urlPath === '/' || urlPath === '') return '🏠';

  return iconMap[category] || '📄';
}

// Extract ALL text content from HTML/Astro content
function extractAllTextContent(content) {
  let text = content
    // Remove code blocks first (they contain non-searchable content)
    // Use word boundary and flexible closing tags to handle </code > variants
    .replace(/<code\b[^>]*>[\s\S]*?<\/code\b[^>]*>/gi, ' ')
    .replace(/<pre\b[^>]*>[\s\S]*?<\/pre\b[^>]*>/gi, ' ')
    // Remove script and style tags (handle forgiving end-tag syntax like </script >)
    .replace(/<script\b[^>]*>[\s\S]*?<\/script\b[^>]*>/gi, '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style\b[^>]*>/gi, '')
    // Remove frontmatter completely (including its contents)
    .replace(/---[\s\S]*?---/g, '')
    // Remove ALL import statements (various formats)
    .replace(/^\s*import\s+.*$/gm, '')
    .replace(/import\s+\{[^}]*\}\s+from\s+['"][^'"]+['"]\s*;?/g, '')
    .replace(/import\s+[\w,\s{}*]+\s+from\s+['"][^'"]+['"]\s*;?/g, '')
    .replace(/import\s+['"][^'"]+['"]\s*;?/g, '')
    // Remove Astro/JSX expressions but keep text content
    .replace(/\{`([^`]*)`\}/g, '$1') // Template literals
    .replace(/\{['"]([^'"]*)['"]\}/g, '$1') // String literals
    .replace(/\{[^}]+\}/g, ' ') // Other expressions
    // Extract text from HTML tags
    .replace(/<[^>]+>/g, ' ')
    // Decode common HTML entities
    // IMPORTANT: Decode &amp; LAST to prevent double-unescaping
    // (e.g., &amp;lt; -> &lt; -> < if &amp; is decoded first)
    .replace(/&nbsp;/gi, ' ')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&quot;/gi, '"')
    .replace(/&#39;/gi, "'")
    .replace(/&amp;/gi, '&') // Decode &amp; LAST
    .replace(/&[a-z]+;/gi, ' ')
    // Normalize whitespace
    .replace(/\s+/g, ' ')
    .trim();

  // Safety net: iteratively remove any remaining HTML-like tags
  // in case earlier replacements (e.g., entity decoding) reintroduced them
  let previous;
  do {
    previous = text;
    text = text
      .replace(/<script\b[^>]*>[\s\S]*?<\/script\b[^>]*>/gi, '')
      .replace(/<style\b[^>]*>[\s\S]*?<\/style\b[^>]*>/gi, '')
      .replace(/<[^>]+>/g, ' ')
      .replace(/\s+/g, ' ')
      .trim();
  } while (text !== previous);

  return text;
}

// Extract title and description from Layout or LandingPage component props
function extractComponentProps(content) {
  let title = null;
  let description = null;

  // Match Layout or LandingPage component with props
  // Handle multi-line props with various quote styles
  const layoutMatch = content.match(/<(?:Layout|LandingPage)\s+([\s\S]*?)>/);
  if (layoutMatch) {
    const propsStr = layoutMatch[1];

    // Extract title prop - handle double-quoted strings (may contain single quotes)
    const titleMatchDouble = propsStr.match(/title\s*=\s*"([^"]+)"/);
    const titleMatchSingle = propsStr.match(/title\s*=\s*'([^']+)'/);
    if (titleMatchDouble) {
      title = titleMatchDouble[1];
    } else if (titleMatchSingle) {
      title = titleMatchSingle[1];
    }

    if (title) {
      // Clean up title (remove "| Caro" and " - Caro" suffixes for cleaner display)
      title = title
        .replace(/\s*\|\s*Caro.*$/, '')
        .replace(/\s*-\s*Caro$/, '')
        .trim();
    }

    // Extract description prop - handle double-quoted strings (may contain single quotes)
    const descMatchDouble = propsStr.match(/description\s*=\s*"([^"]+)"/);
    const descMatchSingle = propsStr.match(/description\s*=\s*'([^']+)'/);
    if (descMatchDouble) {
      description = descMatchDouble[1];
    } else if (descMatchSingle) {
      description = descMatchSingle[1];
    }
  }

  return { title, description };
}

// Extract all string values from JavaScript data in frontmatter
function extractFrontmatterData(content) {
  const frontmatterMatch = content.match(/---\n([\s\S]*?)\n---/);
  if (!frontmatterMatch) return { strings: [], title: null, description: null };

  const frontmatter = frontmatterMatch[1];
  const strings = [];

  // Extract title and description from frontmatter comments or data
  let title = null;
  let description = null;

  // Look for title in page comment
  const titleComment = frontmatter.match(/\*\s*(?:Use Case|Page|Blog):\s*([^\n*]+)/i);
  if (titleComment) {
    title = titleComment[1].trim();
  }

  // Look for competitor name in comparison pages
  const competitorMatch = frontmatter.match(/name:\s*['"]([^'"]+)['"]/);
  if (competitorMatch && frontmatter.includes('competitor')) {
    title = `Caro vs ${competitorMatch[1]}`;
  }

  // Extract all string literals from the frontmatter data
  // Match single-quoted strings - but SKIP import paths and code artifacts
  const singleQuoteStrings = frontmatter.matchAll(/'([^'\\]|\\.)*'/g);
  for (const match of singleQuoteStrings) {
    const str = match[0].slice(1, -1); // Remove quotes
    // Skip import paths, component names, code artifacts, and short strings
    if (str.length > 3 &&
        str.length < 300 && // Skip very long strings (likely malformed)
        !str.includes('/') &&
        !str.includes('.astro') &&
        !str.includes('.tsx') &&
        !str.includes('\n') && // Skip strings with newlines (likely code)
        !str.includes('import ') && // Skip strings containing import statements
        !str.includes('from ') && // Skip strings containing from clauses
        !str.match(/^[A-Z][A-Za-z]+$/) && // Skip PascalCase component names
        !str.startsWith('../') &&
        !str.startsWith('./')) {
      strings.push(str);
    }
  }

  // Match double-quoted strings - but SKIP import paths and code artifacts
  const doubleQuoteStrings = frontmatter.matchAll(/"([^"\\]|\\.)*"/g);
  for (const match of doubleQuoteStrings) {
    const str = match[0].slice(1, -1); // Remove quotes
    // Skip import paths, component names, code artifacts, and short strings
    if (str.length > 3 &&
        str.length < 300 && // Skip very long strings (likely malformed)
        !str.includes('/') &&
        !str.includes('.astro') &&
        !str.includes('.tsx') &&
        !str.includes('\n') && // Skip strings with newlines (likely code)
        !str.includes('import ') && // Skip strings containing import statements
        !str.includes('from ') && // Skip strings containing from clauses
        !str.match(/^[A-Z][A-Za-z]+$/) && // Skip PascalCase component names
        !str.startsWith('../') &&
        !str.startsWith('./')) {
      strings.push(str);
    }
  }

  // Match template literals with content
  const templateStrings = frontmatter.matchAll(/`([^`]*)`/g);
  for (const match of templateStrings) {
    const str = match[1];
    // Clean HTML from template strings
    const cleaned = extractAllTextContent(str);
    if (cleaned.length > 10) {
      strings.push(cleaned);
    }
  }

  // Look for explicit title/description props
  const titleMatch = frontmatter.match(/title:\s*["']([^"']+)["']/i);
  if (titleMatch) title = titleMatch[1];

  const descMatch = frontmatter.match(/description:\s*["']([^"']+)["']/i);
  if (descMatch) description = descMatch[1];

  return { strings, title, description };
}

// Extract text from specific elements for structured content
function extractStructuredContent(content) {
  const result = {
    headings: [],
    paragraphs: [],
    listItems: [],
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
      result.allText.push(text);
    }
  }

  // Extract button text
  const buttonMatches = content.matchAll(/<button[^>]*>([\s\S]*?)<\/button>/gi);
  for (const match of buttonMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
      result.allText.push(text);
    }
  }

  // Extract link text
  const linkMatches = content.matchAll(/<a[^>]*>([\s\S]*?)<\/a>/gi);
  for (const match of linkMatches) {
    const text = extractAllTextContent(match[1]).trim();
    if (text.length > 1) {
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

// Generate better title based on URL path for specific page types
function generateSmartTitle(urlPath, frontmatterData, structured) {
  // For compare pages, always use predefined titles (these take priority)
  if (urlPath.includes('/compare/') && urlPath !== '/compare') {
    const pageName = urlPath.split('/').pop();
    const titleMap = {
      'amazon-q-cli': 'Caro vs Amazon Q CLI',
      'github-copilot-cli': 'Caro vs GitHub Copilot CLI',
      'warp': 'Caro vs Warp',
      'opencode': 'Caro vs OpenCode',
    };
    if (titleMap[pageName]) return titleMap[pageName];
  }

  // For use-cases pages, use predefined titles (these take priority)
  if (urlPath.includes('/use-cases/') && urlPath !== '/use-cases') {
    const pageName = urlPath.split('/').pop();
    const titleMap = {
      'sre': 'SRE & On-Call Engineers',
      'air-gapped': 'Air-Gapped Security Environments',
      'devops': 'DevOps & Platform Engineers',
      'tech-lead': 'Tech Leads & Engineering Managers',
      'developer': 'Developers',
    };
    if (titleMap[pageName]) return titleMap[pageName];
  }

  // Use frontmatter title if available (but not generic ones)
  if (frontmatterData.title &&
      !['TL;DR', 'Overview', 'Introduction'].includes(frontmatterData.title)) {
    return frontmatterData.title;
  }

  // Use first heading if available and not generic
  if (structured.headings.length > 0) {
    const firstHeading = structured.headings[0];
    if (!['TL;DR', 'Overview', 'Introduction'].includes(firstHeading)) {
      return firstHeading;
    }
  }

  // Fall back to path-based title
  return generateTitleFromPath(urlPath);
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

  // Extract from component props (Layout/LandingPage) - highest priority
  const componentProps = extractComponentProps(content);
  const frontmatterData = extractFrontmatterData(content);
  const structured = extractStructuredContent(content);
  const fullText = extractAllTextContent(content);

  // Priority for title: component props > frontmatter > smart detection
  const title = componentProps.title ||
    frontmatterData.title ||
    generateSmartTitle(urlPath, frontmatterData, structured);

  // Priority for description: component props > frontmatter > first paragraph
  const description = componentProps.description ||
    frontmatterData.description ||
    structured.paragraphs[0]?.slice(0, 200) ||
    frontmatterData.strings.find(s => s.length > 30 && s.length < 200) ||
    '';

  // Combine all text for comprehensive search including frontmatter data
  const allTextContent = [
    title,
    description,
    ...frontmatterData.strings, // Include all extracted string data
    ...structured.allText,
    fullText,
  ].join(' ');

  // Generate keywords from all content
  const keywords = extractKeywords(allTextContent, 50);

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
    // FULL text content for comprehensive search - includes frontmatter data
    fullText: allTextContent,
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
    version: '2.1.0', // Version bump for improved extraction
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

  console.log('✅ Search index generated successfully!');
  console.log(`   Output: ${outputPath}`);
  console.log(`   Pages indexed: ${pages.length}`);
  console.log(`   Components indexed: ${componentContent.length}`);
  console.log(`   Total content: ${totalChars.toLocaleString()} characters`);
}

// Run
generateSearchIndex();

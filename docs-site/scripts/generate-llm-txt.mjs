#!/usr/bin/env node
/**
 * Generate LLM.txt Files for Caro Docs
 *
 * Creates two versions of llm.txt for LLM consumption:
 * - llm.txt: Lean version with overview and links
 * - llm-full.txt: Comprehensive version with full content from all docs
 *
 * Based on the llmstxt specification: https://llmstxt.org/
 *
 * Usage: node scripts/generate-llm-txt.mjs
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const DOCS_ROOT = path.resolve(__dirname, '..');

const SITE_URL = 'https://docs.caro.sh';

// Read version from Cargo.toml
function getVersion() {
  try {
    const cargoPath = path.resolve(DOCS_ROOT, '../Cargo.toml');
    const cargoContent = fs.readFileSync(cargoPath, 'utf-8');
    const versionMatch = cargoContent.match(/^version\s*=\s*"([^"]+)"/m);
    return versionMatch ? versionMatch[1] : 'unknown';
  } catch {
    return 'unknown';
  }
}

// Parse frontmatter from markdown/mdx files
function parseFrontmatter(content) {
  const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
  if (!frontmatterMatch) return {};

  const frontmatter = frontmatterMatch[1];
  const result = {};

  // Extract title
  const titleMatch = frontmatter.match(/^title:\s*(.+)$/m);
  if (titleMatch) {
    result.title = titleMatch[1].replace(/^["']|["']$/g, '').trim();
  }

  // Extract description
  const descMatch = frontmatter.match(/^description:\s*(.+)$/m);
  if (descMatch) {
    result.description = descMatch[1].replace(/^["']|["']$/g, '').trim();
  }

  return result;
}

// Extract markdown content without frontmatter and components
function extractMarkdownContent(content) {
  return content
    // Remove frontmatter
    .replace(/^---\n[\s\S]*?\n---\n?/, '')
    // Remove import statements
    .replace(/^import\s+.*$/gm, '')
    // Remove JSX components (Starlight components)
    .replace(/<[A-Z][^>]*>[\s\S]*?<\/[A-Z][^>]*>/g, '')
    .replace(/<[A-Z][^/>]*\/>/g, '')
    // Remove HTML divs with classes (feature cards etc)
    .replace(/<div[^>]*>[\s\S]*?<\/div>/g, '')
    // Keep code blocks but mark them
    .replace(/```(\w+)?\n([\s\S]*?)```/g, (_, lang, code) => {
      return `\n[Code block${lang ? ` (${lang})` : ''}]\n`;
    })
    // Remove inline HTML
    .replace(/<[^>]+>/g, '')
    // Clean up multiple newlines
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

// Generate URL path from file path
function filePathToUrl(filePath, contentDir) {
  let urlPath = filePath
    .replace(contentDir, '')
    .replace(/\.mdx?$/, '')
    .replace(/\/index$/, '');

  return urlPath || '/';
}

// Scan docs content directory
function scanDocs() {
  const contentDir = path.join(DOCS_ROOT, 'src/content/docs');
  const docs = [];

  function scanDir(dir, basePath = '') {
    const files = fs.readdirSync(dir);

    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);

      if (stat.isDirectory()) {
        scanDir(filePath, `${basePath}/${file}`);
      } else if (file.endsWith('.md') || file.endsWith('.mdx')) {
        try {
          const content = fs.readFileSync(filePath, 'utf-8');
          const frontmatter = parseFrontmatter(content);
          const markdownContent = extractMarkdownContent(content);
          const urlPath = filePathToUrl(filePath, contentDir);

          // Determine category from path
          let category = 'main';
          if (urlPath.includes('getting-started')) category = 'getting-started';
          else if (urlPath.includes('guides')) category = 'guides';
          else if (urlPath.includes('development')) category = 'development';
          else if (urlPath.includes('reference')) category = 'reference';

          docs.push({
            path: urlPath,
            title: frontmatter.title || file.replace(/\.mdx?$/, ''),
            description: frontmatter.description || '',
            content: markdownContent,
            category,
          });

          console.log(`  Processed: ${urlPath}`);
        } catch (e) {
          console.error(`  Error processing ${filePath}:`, e.message);
        }
      }
    }
  }

  scanDir(contentDir);

  // Sort by category and importance
  const categoryOrder = ['main', 'getting-started', 'guides', 'reference', 'development'];

  return docs.sort((a, b) => {
    const aCategory = categoryOrder.indexOf(a.category);
    const bCategory = categoryOrder.indexOf(b.category);

    if (aCategory !== bCategory) return aCategory - bCategory;
    return a.path.localeCompare(b.path);
  });
}

// Generate lean llm.txt
function generateLeanVersion(version, docs) {
  const sections = {
    'getting-started': [],
    'guides': [],
    'reference': [],
    'development': [],
  };

  for (const doc of docs) {
    if (doc.category !== 'main' && sections[doc.category]) {
      sections[doc.category].push(doc);
    }
  }

  return `# Caro Documentation

> Technical documentation for Caro - Natural language to shell commands CLI

Caro is an open-source Rust CLI tool that transforms natural language into safe, POSIX-compliant shell commands. This documentation covers installation, configuration, safety features, and development.

## Quick Links

- [Main Docs Site](${SITE_URL}/)
- [Main Website](https://caro.sh/)
- [GitHub Repository](https://github.com/wildcard/caro)

## Getting Started

${sections['getting-started'].map(d => `- [${d.title}](${SITE_URL}${d.path}/) - ${d.description}`).join('\n')}

## Guides

${sections['guides'].map(d => `- [${d.title}](${SITE_URL}${d.path}/) - ${d.description}`).join('\n')}

## Reference

${sections['reference'].map(d => `- [${d.title}](${SITE_URL}${d.path}/) - ${d.description}`).join('\n')}

## Development

${sections['development'].map(d => `- [${d.title}](${SITE_URL}${d.path}/) - ${d.description}`).join('\n')}

## Project Info

- **Version**: ${version}
- **License**: AGPL-3.0
- **Language**: Rust
- **Minimum Rust Version**: 1.75+

## Optional

For comprehensive documentation including full page content, see: ${SITE_URL}/llm-full.txt
`;
}

// Generate full llm.txt with complete content
function generateFullVersion(version, docs) {
  let content = `# Caro Documentation - Complete Reference

> Technical documentation for Caro - Natural language to shell commands CLI

**Version**: ${version}
**License**: AGPL-3.0
**Repository**: https://github.com/wildcard/caro
**Docs Site**: ${SITE_URL}
**Main Website**: https://caro.sh

---

## Overview

Caro is an open-source Rust CLI tool that transforms natural language into safe, POSIX-compliant shell commands. It runs 100% locally by default, validates commands for safety before execution, and supports multiple LLM backends.

### Key Features

- **Privacy-First**: All inference runs locally on your machine
- **Safety Validation**: Pattern-based detection blocks dangerous commands (rm -rf /, fork bombs, etc.)
- **Multiple Backends**: Supports MLX (Apple Silicon), Ollama, vLLM
- **Cross-Platform**: Works on macOS, Linux, and Windows (WSL)
- **POSIX Compliant**: Generated commands work across all Unix systems

### Quick Start

\`\`\`bash
# Install caro
cargo install caro

# Convert natural language to shell commands
caro "find all rust files modified in the last week"
# Output: find . -name "*.rs" -mtime -7

caro "show disk usage sorted by size"
# Output: du -sh * | sort -hr
\`\`\`

---

`;

  // Add each doc's content
  for (const doc of docs) {
    content += `## ${doc.title}\n\n`;
    content += `**URL**: ${SITE_URL}${doc.path}/\n`;
    content += `**Category**: ${doc.category}\n\n`;

    if (doc.description) {
      content += `${doc.description}\n\n`;
    }

    if (doc.content) {
      // Add the actual content, truncated if very long
      const docContent = doc.content.length > 5000
        ? doc.content.substring(0, 5000) + '\n\n[Content truncated - see full page for more]'
        : doc.content;

      content += `${docContent}\n\n`;
    }

    content += `---\n\n`;
  }

  // Add quick reference section
  content += `## Quick Reference

### Installation

\`\`\`bash
# Via Cargo (recommended)
cargo install caro

# From source
git clone https://github.com/wildcard/caro.git
cd caro && cargo install --path .
\`\`\`

### Configuration Location

| Platform | Config Path |
|----------|-------------|
| macOS | ~/Library/Application Support/caro/config.toml |
| Linux | ~/.config/caro/config.toml |
| Windows | %APPDATA%\\caro\\config.toml |

### Common Commands

\`\`\`bash
caro "your natural language query"    # Generate command
caro --backend ollama "query"          # Use specific backend
caro --yes "query"                     # Skip confirmation
caro --verbose "query"                 # Verbose output
caro cache info                        # Show cache info
caro cache clear                       # Clear model cache
\`\`\`

### Risk Levels

| Level | Description | Examples |
|-------|-------------|----------|
| Safe | Normal read operations | ls, cat, find, grep |
| Moderate | File modifications | mv, cp, chmod |
| High | System-level changes | sudo, chown |
| Critical | Blocked - dangerous | rm -rf /, fork bombs |

---

## Links

- [GitHub Repository](https://github.com/wildcard/caro)
- [Issue Tracker](https://github.com/wildcard/caro/issues)
- [Discussions](https://github.com/wildcard/caro/discussions)
- [Main Website](https://caro.sh/)

---

*Generated on ${new Date().toISOString().split('T')[0]}*
`;

  return content;
}

// Main function
function generateLlmTxt() {
  console.log('Generating llm.txt files for docs-site...\n');

  const version = getVersion();
  console.log(`Version: ${version}\n`);

  // Scan docs
  console.log('Scanning documentation...');
  const docs = scanDocs();
  console.log(`\nFound ${docs.length} documentation pages\n`);

  // Generate lean version
  const leanContent = generateLeanVersion(version, docs);
  const leanPath = path.join(DOCS_ROOT, 'public/llm.txt');
  fs.writeFileSync(leanPath, leanContent);
  console.log(`Created: ${leanPath} (${leanContent.length} bytes)`);

  // Generate full version
  const fullContent = generateFullVersion(version, docs);
  const fullPath = path.join(DOCS_ROOT, 'public/llm-full.txt');
  fs.writeFileSync(fullPath, fullContent);
  console.log(`Created: ${fullPath} (${fullContent.length} bytes)`);

  console.log('\nllm.txt generation complete!');
}

// Run
generateLlmTxt();

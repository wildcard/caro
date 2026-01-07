#!/usr/bin/env node
/**
 * Generate LLM.txt Files
 *
 * Creates two versions of llm.txt for LLM consumption:
 * - llm.txt: Lean version with overview and links
 * - llm-full.txt: Comprehensive version with full content
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
const WEBSITE_ROOT = path.resolve(__dirname, '..');

const SITE_URL = 'https://caro.sh';

// Read version from Cargo.toml
function getVersion() {
  try {
    const cargoPath = path.resolve(WEBSITE_ROOT, '../Cargo.toml');
    const cargoContent = fs.readFileSync(cargoPath, 'utf-8');
    const versionMatch = cargoContent.match(/^version\s*=\s*"([^"]+)"/m);
    return versionMatch ? versionMatch[1] : 'unknown';
  } catch {
    return 'unknown';
  }
}

// Clean text content - remove template artifacts
function cleanText(text) {
  return text
    // Remove common Astro/JSX artifacts
    .replace(/class="[^"]*"/g, '')
    .replace(/className="[^"]*"/g, '')
    .replace(/style="[^"]*"/g, '')
    .replace(/\b(const|let|var|function|import|export|default|return|if|else|for|while|switch|case|break|continue|new|this|null|undefined|true|false)\b/g, ' ')
    // Remove common patterns
    .replace(/\(\s*\)/g, '')
    .replace(/->/g, '')
    .replace(/=>/g, '')
    .replace(/\[\s*\]/g, '')
    .replace(/\{\s*\}/g, '')
    .replace(/\(\s*\w+\s*\)/g, '')
    // Remove URLs that aren't part of text
    .replace(/https?:\/\/[^\s<>"]+/g, match => match.includes('caro.sh') || match.includes('github.com') ? match : '')
    // Clean up whitespace
    .replace(/\s+/g, ' ')
    .trim();
}

// Extract text content from Astro/HTML - improved version
function extractTextContent(content) {
  // First, extract meaningful content sections
  let text = content
    // Remove frontmatter
    .replace(/---[\s\S]*?---/g, '')
    // Remove code blocks
    .replace(/<code\b[^>]*>[\s\S]*?<\/code\b[^>]*>/gi, '')
    .replace(/<pre\b[^>]*>[\s\S]*?<\/pre\b[^>]*>/gi, '')
    // Remove script and style
    .replace(/<script\b[^>]*>[\s\S]*?<\/script\b[^>]*>/gi, '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style\b[^>]*>/gi, '')
    // Remove all import statements
    .replace(/^\s*import\s+.*$/gm, '')
    .replace(/import\s+[\s\S]*?from\s+['"][^'"]+['"]\s*;?/g, '')
    // Remove Astro expressions but keep readable text
    .replace(/\{`([^`]*)`\}/g, '$1')
    .replace(/\{['"]([^'"]*)['"]\}/g, '$1')
    // Remove complex expressions
    .replace(/\{[^{}]*\{[^{}]*\}[^{}]*\}/g, '') // Nested braces
    .replace(/\{[\s\S]*?\}/g, ' ') // Remaining braces
    // Extract text from common HTML elements
    .replace(/<(h[1-6]|p|li|td|th|span|a|button|label|strong|em|b|i)[^>]*>([^<]*)<\/\1>/gi, '$2 ')
    // Remove remaining HTML tags
    .replace(/<[^>]+>/g, ' ')
    // Decode HTML entities
    .replace(/&nbsp;/gi, ' ')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&quot;/gi, '"')
    .replace(/&#39;/gi, "'")
    .replace(/&amp;/gi, '&')
    .replace(/&[a-z]+;/gi, ' ');

  return cleanText(text);
}

// Extract title and description from Layout/LandingPage props
function extractMetadata(content) {
  let title = null;
  let description = null;

  const layoutMatch = content.match(/<(?:Layout|LandingPage|ComparisonPageLayout|BlogPost)\s+([\s\S]*?)>/);
  if (layoutMatch) {
    const propsStr = layoutMatch[1];

    const titleMatchDouble = propsStr.match(/title\s*=\s*"([^"]+)"/);
    const titleMatchSingle = propsStr.match(/title\s*=\s*'([^']+)'/);
    title = titleMatchDouble?.[1] || titleMatchSingle?.[1];

    if (title) {
      title = title
        .replace(/\s*\|\s*Caro.*$/, '')
        .replace(/\s*-\s*Caro$/, '')
        .replace(/\s*-\s*Feature Comparison.*$/, '')
        .trim();
    }

    const descMatchDouble = propsStr.match(/description\s*=\s*"([^"]+)"/);
    const descMatchSingle = propsStr.match(/description\s*=\s*'([^']+)'/);
    description = descMatchDouble?.[1] || descMatchSingle?.[1];
  }

  return { title, description };
}

// Extract structured data from frontmatter (for FAQ, data-driven pages)
function extractFrontmatterStrings(content) {
  const frontmatterMatch = content.match(/---\n([\s\S]*?)\n---/);
  if (!frontmatterMatch) return [];

  const frontmatter = frontmatterMatch[1];
  const strings = [];

  // Extract question/answer pairs for FAQ
  const questionMatches = frontmatter.matchAll(/question:\s*["']([^"']+)["']/g);
  const answerMatches = frontmatter.matchAll(/answer:\s*["']([^"']+)["']/g);

  for (const match of questionMatches) {
    strings.push(`Q: ${match[1]}`);
  }

  for (const match of answerMatches) {
    // Clean HTML from answers
    const cleanAnswer = match[1]
      .replace(/<[^>]+>/g, '')
      .replace(/&[a-z]+;/gi, ' ');
    strings.push(`A: ${cleanAnswer}`);
  }

  // Extract title strings
  const titleMatches = frontmatter.matchAll(/title:\s*["']([^"']+)["']/g);
  for (const match of titleMatches) {
    strings.push(match[1]);
  }

  // Extract description strings
  const descMatches = frontmatter.matchAll(/description:\s*["']([^"']+)["']/g);
  for (const match of descMatches) {
    strings.push(match[1]);
  }

  return strings;
}

// Generate lean llm.txt
function generateLeanVersion(version) {
  return `# Caro

> Your loyal shell companion - A privacy-first AI CLI assistant that converts natural language to safe shell commands

Caro is an open-source Rust CLI tool that helps you generate POSIX-compliant shell commands from natural language descriptions. It runs 100% locally, validates commands for safety before execution, and supports multiple LLM backends including Ollama, MLX (Apple Silicon), and cloud providers.

## Quick Start

Install via cargo:
\`\`\`bash
cargo install caro
\`\`\`

Or use the install script:
\`\`\`bash
curl -sSL https://caro.sh/install.sh | bash
\`\`\`

Basic usage:
\`\`\`bash
caro "find all Python files modified in the last week"
\`\`\`

## Key Features

- **Privacy-First**: Runs 100% locally - commands never leave your machine
- **Safety Validation**: Pattern-based detection of dangerous commands before execution
- **Multiple Backends**: Supports Ollama, MLX, llama.cpp, Anthropic Claude, OpenAI GPT
- **Cross-Platform**: Works on macOS, Linux, and Windows (WSL)
- **POSIX Compliant**: Generates shell-agnostic commands that work everywhere

## Documentation

- [Homepage](${SITE_URL}/)
- [FAQ](${SITE_URL}/faq)
- [AI Command Safety](${SITE_URL}/ai-command-safety)
- [Safe Shell Commands](${SITE_URL}/safe-shell-commands)
- [Roadmap](${SITE_URL}/roadmap)
- [Telemetry](${SITE_URL}/telemetry)
- [Credits](${SITE_URL}/credits)

## Use Cases

- [SRE & On-Call Engineers](${SITE_URL}/use-cases/sre)
- [Air-Gapped Security Environments](${SITE_URL}/use-cases/air-gapped)
- [DevOps & Platform Engineers](${SITE_URL}/use-cases/devops)
- [Tech Leads](${SITE_URL}/use-cases/tech-lead)
- [Developers](${SITE_URL}/use-cases/developer)

## Comparisons

- [Caro vs GitHub Copilot CLI](${SITE_URL}/compare/github-copilot-cli)
- [Caro vs Warp AI](${SITE_URL}/compare/warp)
- [Caro vs Kiro CLI](${SITE_URL}/compare/kiro-cli)
- [Caro vs OpenCode](${SITE_URL}/compare/opencode)

## Blog

- [Announcing Caro](${SITE_URL}/blog/announcing-caro)
- [Why Caro?](${SITE_URL}/blog/why-caro)
- [Security Practices](${SITE_URL}/blog/security-practices)
- [Batteries Included](${SITE_URL}/blog/batteries-included)
- [Claude Skill Launch](${SITE_URL}/blog/claude-skill-launch)
- [Modern Unix Tools in Rust](${SITE_URL}/blog/rust-unix-tools)

## Resources

- [GitHub Repository](https://github.com/wildcard/caro)
- [Full Documentation](${SITE_URL}/llm-full.txt)
- [Modern Unix Tools Reference](${SITE_URL}/modern-unix-tools)
- [Terminal Glossary](${SITE_URL}/glossary)

## Project Info

- **Version**: ${version}
- **License**: AGPL-3.0
- **Language**: Rust
- **Minimum Rust Version**: 1.70+

## Optional

For comprehensive documentation including full page content, see: ${SITE_URL}/llm-full.txt
`;
}

// Generate full llm.txt with complete content
function generateFullVersion(version, pages) {
  let content = `# Caro - Complete Documentation

> Your loyal shell companion - A privacy-first AI CLI assistant that converts natural language to safe shell commands

**Version**: ${version}
**License**: AGPL-3.0
**Repository**: https://github.com/wildcard/caro
**Website**: ${SITE_URL}

---

## Overview

Caro is an open-source Rust CLI tool that helps you generate POSIX-compliant shell commands from natural language descriptions. It runs 100% locally by default, validates commands for safety before execution, and supports multiple LLM backends.

### Key Features

- **Privacy-First**: Runs 100% locally - your commands, file paths, and data never leave your machine
- **Safety Validation**: Pattern-based detection of dangerous commands (rm -rf /, fork bombs, disk wipers) before execution
- **Multiple Backends**: Supports Ollama, MLX (Apple Silicon), llama.cpp, Anthropic Claude, OpenAI GPT
- **Cross-Platform**: Works on macOS, Linux, and Windows (WSL)
- **POSIX Compliant**: Generates shell-agnostic commands that work everywhere
- **AI Agent Integration**: Available as MCP server for Claude and other AI agents

### Installation

Install via cargo:
\`\`\`bash
cargo install caro
\`\`\`

Or use the install script:
\`\`\`bash
curl -sSL https://caro.sh/install.sh | bash
\`\`\`

### Basic Usage

\`\`\`bash
# Generate a command from natural language
caro "find all Python files modified in the last week"

# Auto-execute safe commands
caro -e "list files sorted by size"

# Use a specific backend
caro --backend ollama "compress all logs older than 30 days"

# JSON output for scripting
caro --json "find large files"
\`\`\`

### Configuration

Caro uses a config file at \`~/.config/caro/config.toml\`:

\`\`\`toml
# Default backend (ollama, mlx, anthropic, openai)
backend = "ollama"

# Default model for the backend
model = "llama3"

# Safety validation level (strict, normal, permissive)
safety_level = "normal"

# Enable/disable telemetry
telemetry = false
\`\`\`

### System Requirements

- macOS, Linux, or Windows (WSL)
- Rust 1.70+ for building from source
- 8GB+ RAM recommended for local LLM inference
- Apple Silicon Macs get hardware acceleration with MLX

---

`;

  // Add each page content
  for (const page of pages) {
    if (!page.description && !page.content) continue;

    content += `## ${page.title}\n\n`;
    content += `**URL**: ${SITE_URL}${page.path}\n\n`;

    if (page.description) {
      content += `${page.description}\n\n`;
    }

    // Add frontmatter data (like FAQ Q&A pairs)
    if (page.frontmatterData && page.frontmatterData.length > 0) {
      for (const item of page.frontmatterData) {
        content += `${item}\n\n`;
      }
    }

    content += `---\n\n`;
  }

  // Add comprehensive FAQ section
  content += `## Frequently Asked Questions

### Getting Started

**What is Caro?**
Caro is a privacy-first AI shell assistant that converts natural language into safe, validated shell commands. It runs 100% locally on your machine - your commands, file paths, and data never leave your computer.

**How do I install Caro?**
The easiest way is via cargo: \`cargo install caro\`. You can also use the install script: \`curl -sSL https://caro.sh/install.sh | bash\`. For other options, see our installation guide.

**What are the system requirements?**
Caro runs on macOS, Linux, and Windows (WSL). You need Rust 1.70+ for building from source. For local LLM inference, we recommend 8GB+ RAM. Apple Silicon Macs get hardware acceleration with MLX.

**Do I need an API key to use Caro?**
It depends on your backend choice. Local backends (Ollama, MLX, llama.cpp) require no API keys. Cloud backends (Anthropic, OpenAI) require API keys. Caro defaults to trying local inference first.

### Safety & Security

**How does Caro's safety validation work?**
Caro uses pattern-based command validation, not AI-based filtering. Every generated command is checked against known dangerous patterns (rm -rf /, fork bombs, disk wipers) before you run it. This is deterministic, not probabilistic - the same command always gets the same safety assessment.

**What happens when Caro detects a dangerous command?**
Caro shows a clear warning explaining why the command is dangerous and what it would do. You can still proceed if you intend to run it - Caro is a seatbelt, not a straitjacket. For truly destructive commands, you'll need to explicitly confirm.

**Does Caro send my data to the cloud?**
No. Caro runs 100% locally by default. Your commands, file paths, server names, and directory structures never leave your machine. If you choose to use a cloud backend (like Anthropic or OpenAI), only the natural language prompt is sent - not your command history or file system data.

**Is Caro safe to use with AI agents?**
Yes - Caro is designed with AI agents in mind. We recommend defense in depth: run agents as unprivileged users, sandbox to specific directories, use container isolation, and let Caro validate commands. Each layer catches what others miss.

### Backends & Models

**What LLM backends does Caro support?**
Caro supports multiple backends:
- **Local**: Ollama, MLX (Apple Silicon), llama.cpp
- **Cloud**: Anthropic Claude, OpenAI GPT
Caro automatically selects the best available backend or you can specify one explicitly.

**Which backend should I use?**
For privacy and offline use, choose local backends. On Apple Silicon Macs, MLX offers the best performance with hardware acceleration. Ollama is great cross-platform. For best quality responses, cloud backends like Claude typically perform better but require API keys.

**How do I set up Ollama with Caro?**
Install Ollama from ollama.ai, then run \`ollama pull llama3\` (or another model). Caro will automatically detect Ollama when it's running. You can verify with \`caro --backend ollama "list files"\`.

**How do I use MLX on my Mac?**
MLX is built into Caro for Apple Silicon Macs. Ensure you have the mlx Python package installed. Caro will automatically use MLX when available on M1/M2/M3 Macs for hardware-accelerated local inference.

**Can I use my own fine-tuned models?**
Yes! With Ollama, you can use any GGUF model. With MLX, you can use any MLX-compatible model. Point Caro to your model with the \`--model\` flag or set it in your config file.

### Usage & Features

**How do I use Caro?**
Simply run \`caro "your request in natural language"\`. For example: \`caro "find all Python files modified in the last week"\`. Caro generates the command, shows it to you with a safety assessment, and asks for confirmation before running.

**Can Caro execute commands automatically?**
By default, Caro shows you the command and waits for confirmation. You can use \`--execute\` or \`-e\` to auto-execute safe commands, but dangerous commands always require explicit confirmation.

**How does Caro handle different shells?**
Caro detects your shell ($SHELL) at runtime and generates appropriate syntax. It works with bash, zsh, fish, and POSIX sh. On macOS, it knows you're using BSD tools; on Linux, it adjusts for GNU syntax.

**Can I use Caro in scripts?**
Yes! Use \`caro --quiet\` for scripting. You can pipe output, use in CI/CD, or integrate with other tools. The \`--json\` flag outputs structured data for programmatic use.

### Troubleshooting

**Caro says 'no backend available' - what do I do?**
This means Caro couldn't find a local LLM or API key. Either: (1) Install and start Ollama with a model, (2) Set up MLX if on Apple Silicon, or (3) Set an API key for a cloud provider with \`export ANTHROPIC_API_KEY=...\`.

**Commands are generating slowly - how do I speed things up?**
Local inference speed depends on your hardware and model size. Try a smaller model (7B instead of 70B), use MLX on Apple Silicon for acceleration, or switch to a cloud backend for faster responses.

**The generated command is wrong - what should I do?**
You can: (1) Rephrase your request to be more specific, (2) Add context like "on macOS" or "using find command", (3) Try a different backend/model, or (4) Report the issue on GitHub so we can improve.

**How do I report a bug or request a feature?**
Open an issue on our GitHub repository at github.com/wildcard/caro/issues. Include your OS, Caro version (\`caro --version\`), backend, and steps to reproduce.

### Privacy & Telemetry

**What data does Caro collect?**
By default, Caro collects minimal, anonymous usage metrics to help us improve the product - things like command success rates and which backends are used. No commands, file paths, or personal data are ever collected.

**Can I disable telemetry?**
Yes. Set \`telemetry = false\` in your config file or use \`export CARO_TELEMETRY=false\`. Caro will work exactly the same with telemetry disabled.

**Is Caro GDPR compliant?**
Yes. We don't collect personal data. All telemetry is anonymous and aggregated. You have full control over what data is sent (if any), and can disable telemetry entirely.

### Contributing & Community

**How can I contribute to Caro?**
We welcome contributions! You can: submit bug reports, propose features, contribute code (it's Rust!), add safety patterns, improve documentation, or help with translations. See our contributing guide on GitHub.

**Is Caro open source?**
Yes! Caro is licensed under AGPL-3.0. You can read, modify, and distribute the source code. The full codebase is available on GitHub.

---

## Links

- [GitHub Repository](https://github.com/wildcard/caro)
- [Issue Tracker](https://github.com/wildcard/caro/issues)
- [Discussions](https://github.com/wildcard/caro/discussions)
- [Contributing Guide](https://github.com/wildcard/caro/blob/main/CONTRIBUTING.md)

---

*Generated on ${new Date().toISOString().split('T')[0]}*
`;

  return content;
}

// Scan pages and extract content
function scanPages() {
  const pagesDir = path.join(WEBSITE_ROOT, 'src/pages');
  const pages = [];

  function scanDir(dir, basePath = '') {
    const files = fs.readdirSync(dir);

    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);

      if (stat.isDirectory()) {
        scanDir(filePath, `${basePath}/${file}`);
      } else if (file.endsWith('.astro')) {
        try {
          const content = fs.readFileSync(filePath, 'utf-8');
          const metadata = extractMetadata(content);
          const frontmatterData = extractFrontmatterStrings(content);

          let urlPath = `${basePath}/${file}`
            .replace('/index.astro', '')
            .replace('.astro', '');

          if (urlPath === '') urlPath = '/';

          // Skip certain pages that don't have meaningful content for LLMs
          if (urlPath.includes('explore')) continue;

          const title = metadata.title || urlPath.split('/').pop()?.replace(/-/g, ' ') || 'Home';

          pages.push({
            path: urlPath,
            title: title.charAt(0).toUpperCase() + title.slice(1),
            description: metadata.description || '',
            frontmatterData: frontmatterData.slice(0, 50) // Limit to first 50 items
          });

          console.log(`  Processed: ${urlPath}`);
        } catch (e) {
          console.error(`  Error processing ${filePath}:`, e.message);
        }
      }
    }
  }

  scanDir(pagesDir);

  // Sort pages by importance and path
  const priorityOrder = [
    '/',
    '/faq',
    '/ai-command-safety',
    '/safe-shell-commands',
    '/ai-agent-safety',
    '/roadmap',
    '/telemetry',
    '/credits',
    '/glossary',
    '/modern-unix-tools'
  ];

  return pages.sort((a, b) => {
    const aIndex = priorityOrder.indexOf(a.path);
    const bIndex = priorityOrder.indexOf(b.path);

    // Priority pages first
    if (aIndex !== -1 && bIndex !== -1) return aIndex - bIndex;
    if (aIndex !== -1) return -1;
    if (bIndex !== -1) return 1;

    // Then by category
    const categoryOrder = ['/', '/use-cases', '/compare', '/blog'];
    const aCategory = categoryOrder.find(c => a.path.startsWith(c)) || '/zzz';
    const bCategory = categoryOrder.find(c => b.path.startsWith(c)) || '/zzz';

    if (aCategory !== bCategory) {
      return categoryOrder.indexOf(aCategory) - categoryOrder.indexOf(bCategory);
    }

    return a.path.localeCompare(b.path);
  });
}

// Main function
function generateLlmTxt() {
  console.log('Generating llm.txt files...\n');

  const version = getVersion();
  console.log(`Version: ${version}\n`);

  // Scan pages
  console.log('Scanning pages...');
  const pages = scanPages();
  console.log(`\nFound ${pages.length} pages\n`);

  // Generate lean version
  const leanContent = generateLeanVersion(version);
  const leanPath = path.join(WEBSITE_ROOT, 'public/llm.txt');
  fs.writeFileSync(leanPath, leanContent);
  console.log(`Created: ${leanPath} (${leanContent.length} bytes)`);

  // Generate full version
  const fullContent = generateFullVersion(version, pages);
  const fullPath = path.join(WEBSITE_ROOT, 'public/llm-full.txt');
  fs.writeFileSync(fullPath, fullContent);
  console.log(`Created: ${fullPath} (${fullContent.length} bytes)`);

  console.log('\nllm.txt generation complete!');
}

// Run
generateLlmTxt();

# Website OmniSearch Page Indexing

**Created**: January 1, 2026
**Last Updated**: January 1, 2026

## Overview

The caro website uses OmniSearch (Cmd+/ or Ctrl+/) for site-wide search functionality. This search feature indexes all pages and content to provide comprehensive search results.

## IMPORTANT: Adding New Pages

**When creating any new page on the website, you MUST add it to the search index.**

### Step 1: Add to Pages Index

Add the new page to `website/src/config/pages.ts`:

```typescript
{
  title: 'Page Title',
  path: '/your-page-path',
  description: 'Brief description for search results',
  category: 'main' | 'use-cases' | 'compare' | 'blog' | 'docs',
  keywords: ['relevant', 'search', 'terms'],
  icon: '🔍', // Choose an appropriate emoji
},
```

### Step 2: Choose the Correct Category

| Category | When to Use |
|----------|-------------|
| `main` | Core pages (home, features, roadmap, support, credits) |
| `use-cases` | Persona-specific use case pages |
| `compare` | Comparison pages vs other tools |
| `blog` | Blog posts and articles |
| `docs` | Documentation pages |

### Step 3: Add Meaningful Keywords

Include keywords that users might search for:
- Primary function/purpose
- Synonyms and related terms
- Technical terms if applicable
- Persona or audience keywords

## Example: Adding a New Blog Post

```typescript
// In website/src/config/pages.ts, add to PAGES_INDEX:
{
  title: 'How to Use Shell Aliases',
  path: '/blog/shell-aliases',
  description: 'Learn how to create and manage shell aliases with Caro',
  category: 'blog',
  keywords: ['aliases', 'bash', 'zsh', 'shortcuts', 'productivity', 'tutorial'],
  icon: '📝',
},
```

## Automated Index Generation

The build process runs `scripts/generate-search-index.mjs` which:
1. Scans all pages in `src/pages/`
2. Extracts content from headings, paragraphs, and lists
3. Generates a comprehensive search index
4. Stores it in `src/config/search-index.json`

**Manual page entries in `pages.ts` take precedence** and provide better control over search presentation.

## Search Index Structure

Each indexed page includes:
- `title` - Display name in search results
- `path` - URL path
- `description` - Summary shown in results
- `category` - For grouping results
- `keywords` - Additional search terms
- `icon` - Visual indicator
- `_search` - Pre-computed search string (auto-generated)
- `_words` - Tokenized words for fuzzy matching (auto-generated)

## Hotkey Visibility

The search feature is discoverable via:
1. **Search button in navigation** - Shows `⌘/` hotkey hint
2. **Tip inside search dialog** - "Use ⌘/ or Ctrl+/ anywhere to open search"
3. **Footer hints** - Keyboard navigation instructions

## Verification

After adding a new page:
1. Run `npm run dev` in the website directory
2. Press `Cmd+/` or `Ctrl+/` to open search
3. Type keywords related to your new page
4. Verify it appears in results with correct title, description, and icon

## Files Involved

| File | Purpose |
|------|---------|
| `src/config/pages.ts` | Manual page index entries |
| `src/config/search-index.json` | Auto-generated full index |
| `src/components/OmniSearch.astro` | Search component |
| `scripts/generate-search-index.mjs` | Index generator script |

## See Also

- `website/src/config/pages.ts` - Full page index with all entries
- `website/scripts/generate-search-index.mjs` - Index generation script
- `.claude/memory/website-centralized-config.md` - Related config patterns

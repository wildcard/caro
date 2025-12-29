# Content Management

This directory contains all content for the Caro Learn section, managed by **Keystatic CMS**.

## Directory Structure

```
content/
├── commands/           # Command tutorials (Terminus-style)
│   ├── find.md
│   ├── grep.md
│   └── xargs.md
├── stories/            # Unix history narratives
│   ├── creat-missing-e.md
│   └── pipes-connected-world.md
├── daily-picks/        # Daily tips and trivia
│   ├── 2025-01-15-lsof.md
│   ├── 2025-01-16-history.md
│   └── 2025-01-17-epoch.md
├── settings/           # Site-wide settings
│   └── site.yaml
└── config.ts           # Astro Content Collections schema
```

## Content Types

### Commands

Terminus-style practical command tutorials.

**Schema:**
- `title`: Full title (e.g., "xargs: Transform Input into Arguments")
- `command`: Command name (used as slug)
- `description`: Brief SEO description
- `difficulty`: beginner | intermediate | advanced
- `platforms`: Array of linux, macos, bsd, posix, unix
- `tags`: Array of topic tags
- `publishedAt`: Date (YYYY-MM-DD)
- `featured`: Boolean (show on homepage)
- `relatedCommands`: Array of related command names
- `caroPrompt`: Example prompt for Caro

### Stories

Unix history narratives.

**Schema:**
- `title`: Story title
- `subtitle`: Optional tagline
- `category`: history | people | technology | culture | platform
- `era`: Time period (e.g., "1970s")
- `publishedAt`: Date
- `featured`: Boolean
- `readingTime`: Minutes to read
- `author`: Author name (default: "Caro Team")
- `sources`: Array of {title, url} for references
- `tags`: Array of topic tags

### Daily Picks

Short-form content for social media.

**Schema:**
- `title`: Pick title
- `type`: command | tip | trivia | quote | error
- `publishedAt`: Date
- `socialText`: Pre-formatted Twitter text (max 280 chars)
- `hashtags`: Array of hashtags (without #)
- `source`: Attribution

## Using the CMS

### Development Mode (with CMS admin UI)

```bash
npm run dev:cms
```

Then visit `http://localhost:4321/keystatic` to access the admin UI.

### Regular Development (without CMS)

```bash
npm run dev
```

### Production Build

```bash
npm run build
```

The CMS is disabled for production builds. Content is managed through the admin UI in development and committed to Git.

## Adding New Content

### Via CMS (Recommended)

1. Run `npm run dev:cms`
2. Open `http://localhost:4321/keystatic`
3. Navigate to the content type
4. Click "Create" and fill in the form
5. Save and commit the changes

### Via File System

1. Create a new `.md` file in the appropriate directory
2. Add frontmatter matching the schema above
3. Write content in Markdown/MDX format

## Content Guidelines

### Commands
- Focus on 3-5 "commands you'll actually use"
- Include real-world examples
- Add platform notes for differences
- Link to related commands

### Stories
- Start with a compelling hook
- Include historical context and dates
- Use code examples where relevant
- End with modern relevance or "try it yourself"

### Daily Picks
- Keep social text under 280 characters
- Use 3-5 relevant hashtags
- Include a quick tip or example
- Make it shareable

## Keystatic Configuration

The CMS schema is defined in `/keystatic.config.ts`. This includes:
- Field types and validation
- UI configuration
- Content organization

Changes to the schema should be reflected in both:
- `keystatic.config.ts` (CMS schema)
- `src/content/config.ts` (Astro collections schema)

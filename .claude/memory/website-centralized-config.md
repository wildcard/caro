# Website Centralized Configuration

**Created**: December 29, 2025
**Last Updated**: December 29, 2025

## Overview

The caro website uses a centralized TypeScript configuration file to maintain consistency across all pages and components. This prevents version mismatches, incorrect URLs, and other configuration errors.

## Configuration File

**Location**: `website/src/config/site.ts`

```typescript
export const SITE_CONFIG = {
  name: 'Caro',
  tagline: 'Your loyal shell companion',
  version: '1.0.2',
  domain: 'caro.sh',
  github: {
    org: 'wildcard',
    repo: 'caro',
    url: 'https://github.com/wildcard/caro',
  },
  downloads: {
    baseUrl: 'https://github.com/wildcard/caro/releases/download',
  },
} as const;
```

## What This Replaces

Before centralized config, these values were hardcoded across multiple files:

| Value | Old Locations | Issues Found |
|-------|---------------|--------------|
| Version number | 6+ places (Download.astro, explore pages) | explore showed v0.1.0 when actual was v1.0.2 |
| GitHub URL | 3+ places | Some used `caro-sh/caro`, correct is `wildcard/caro` |
| Product name | 20+ places | All hardcoded, no single source of truth |
| Domain | 5+ places | Repeated `caro.sh` strings |

## Usage Pattern

### Import the config

```typescript
import { SITE_CONFIG } from '../config/site';
```

### Use in Astro components

```astro
<!-- Version display -->
<div>v{SITE_CONFIG.version}</div>

<!-- GitHub link -->
<a href={SITE_CONFIG.github.url}>GitHub</a>

<!-- Download URL -->
<a href={`${SITE_CONFIG.downloads.baseUrl}/v${SITE_CONFIG.version}/caro-${SITE_CONFIG.version}-macos-silicon`}>
  Download macOS
</a>
```

## Components Updated

As of December 2025, the following components use `SITE_CONFIG`:

1. **`website/src/components/Download.astro`**
   - Version text: `(v{SITE_CONFIG.version})`
   - 5 download URLs using `SITE_CONFIG.downloads.baseUrl` and `SITE_CONFIG.version`

2. **`website/src/pages/explore/index.astro`**
   - Version badge: `v{SITE_CONFIG.version}`
   - 3 GitHub URLs using `SITE_CONFIG.github.url`

## Release Process Integration

When releasing a new version, update **TWO** files:

1. **`Cargo.toml`** (Rust package version)
   ```toml
   [package]
   version = "1.0.3"
   ```

2. **`website/src/config/site.ts`** (Website version)
   ```typescript
   version: '1.0.3',
   ```

**The `/caro.release.version` skill should automatically update both files.**

## Future Development

### Adding New Pages

When creating new pages or components that need product info:

1. **DO**: Import and use `SITE_CONFIG`
   ```astro
   import { SITE_CONFIG } from '../config/site';
   <h1>{SITE_CONFIG.name}</h1>
   ```

2. **DON'T**: Hardcode values
   ```astro
   <h1>Caro</h1>  <!-- ❌ Don't do this -->
   ```

### Extending the Config

To add new centralized values:

1. Add to `site.ts`:
   ```typescript
   export const SITE_CONFIG = {
     // ... existing ...
     social: {
       twitter: '@caro_cli',
       discord: 'https://discord.gg/caro',
     },
   } as const;
   ```

2. Use TypeScript autocomplete to discover available fields

## Benefits

- ✅ **Consistency**: All pages show the same version/info
- ✅ **Type Safety**: TypeScript catches typos and missing fields
- ✅ **DRY Principle**: Don't Repeat Yourself
- ✅ **Easy Updates**: Change once, updates everywhere
- ✅ **Release Safety**: No more version mismatches

## Historical Context

This centralization was created on December 29, 2025 after discovering:
- `/explore` page showed v0.1.0 (wrong)
- `/explore` page linked to `github.com/caro-sh/caro` (wrong org)
- Download component hardcoded v1.0.2 in 6 places
- Actual version was v1.0.2 in `Cargo.toml`

The fix involved:
1. Creating `website/src/config/site.ts`
2. Updating `Download.astro` (6 replacements)
3. Updating `explore/index.astro` (4 replacements)
4. Updating `RELEASE_PROCESS.md` documentation

## See Also

- `/docs/RELEASE_PROCESS.md` - Section "Website Version Management"
- `/.claude/commands/caro.release.version.md` - Version bump skill

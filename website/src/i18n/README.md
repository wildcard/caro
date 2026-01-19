# Caro Internationalization (i18n) System

Complete documentation for the Caro website internationalization infrastructure.

## Overview

The Caro website supports **15 locales** with a comprehensive i18n system that includes:

- ✅ Locale detection and persistence
- ✅ Localized routing and links
- ✅ Language switcher UI component
- ✅ SEO-optimized hreflang tags
- ✅ RTL layout support (Hebrew, Arabic, Urdu)
- ✅ Automated translation workflows
- ✅ Quality validation and coverage reporting
- ✅ Cultural context guidelines

## Supported Locales

| Tier | Locales | Target Coverage |
|------|---------|-----------------|
| **Tier 1** | es, fr, de, pt, ja | 95%+ |
| **Tier 2** | ko, he, ar, hi | 85%+ |
| **Tier 3** | ru, uk, ur, fil, id | 75%+ |

Current status: **61.8% average coverage**

## Architecture

### Core Modules

| Module | Path | Purpose |
|--------|------|---------|
| **Translation Config** | `config.ts` | Core translation function `t()` and locale metadata |
| **Locale Manager** | `lib/locale-manager.ts` | Detection, persistence, browser integration |
| **Localized Links** | `lib/localized-links.ts` | URL generation with locale prefixes |
| **Cultural Contexts** | `cultural-contexts.ts` | Metro-based cultural guidelines |
| **Translation Rules** | `translation-rules.ts` | NEVER/ALWAYS translate rules |

### Components

| Component | Path | Usage |
|-----------|------|-------|
| **LanguageSwitcher** | `components/LanguageSwitcher.astro` | Dropdown UI for language selection |
| **HreflangMeta** | `components/HreflangMeta.astro` | SEO meta tags for alternate locales |

### Scripts

| Script | Path | Purpose |
|--------|------|---------|
| **status.mjs** | `scripts/i18n/status.mjs` | Coverage report |
| **sync-keys.mjs** | `scripts/i18n/sync-keys.mjs` | Ensure all locales have same keys |
| **validate.mjs** | `scripts/i18n/validate.mjs` | Quality validation |

## Usage

### 1. Using Translations in Components

```astro
---
import { t } from '../i18n/config';
import type { Locale } from '../i18n/config';

interface Props {
  lang?: Locale;
}

const { lang = 'en' } = Astro.props;
---

<h1>{t(lang, 'landing.hero.title')}</h1>
<p>{t(lang, 'landing.hero.description')}</p>
```

### 2. Creating Localized Links

```astro
---
import { localizedHref } from '../lib/localized-links';

const lang = Astro.params.lang || 'en';
---

<nav>
  <a href={localizedHref('/', lang)}>Home</a>
  <a href={localizedHref('/features', lang)}>Features</a>
  <a href={localizedHref('/blog', lang)}>Blog</a>
</nav>
```

### 3. Adding Language Switcher

```astro
---
import LanguageSwitcher from '../components/LanguageSwitcher.astro';

const currentPath = Astro.url.pathname;
const currentLocale = Astro.params.lang || 'en';
---

<header>
  <LanguageSwitcher currentLocale={currentLocale} currentPath={currentPath} />
</header>
```

### 4. Adding SEO Meta Tags

```astro
---
import HreflangMeta from '../components/HreflangMeta.astro';
---

<head>
  <HreflangMeta currentPath="/features" />
</head>
```

## Translation Workflow

### Manual Translation

1. **Edit locale files** in `src/i18n/locales/{locale}/*.json`
2. **Sync keys** to ensure consistency:
   ```bash
   node scripts/i18n/sync-keys.mjs
   ```
3. **Validate translations**:
   ```bash
   node scripts/i18n/validate.mjs --locale es
   ```
4. **Check coverage**:
   ```bash
   node scripts/i18n/status.mjs
   ```

### Automated Translation (GitHub Actions)

#### Trigger Mass Translation

```bash
gh workflow run translate-all.yml \
  -f locales="es,fr,de" \
  -f backend="claude" \
  -f batch_size="30"
```

This will:
1. Translate missing keys for specified locales
2. Create PRs with translation updates
3. Run validation checks
4. Generate coverage report

#### Scheduled Translation

- **Weekly**: Every Sunday 3am UTC
- **Auto-triggers**: All locales
- **Creates**: Individual PRs per locale

### On-Demand Translation (CLI)

The existing translation script can be used directly:

```bash
node .github/scripts/translate-multi-backend.js \
  --locale es \
  --backend claude \
  --batch-size 50
```

## Translation Guidelines

### NEVER Translate

- **Brand names**: Caro, Claude, GitHub, Anthropic
- **Technical terms**: POSIX, CLI, API, JSON, BSD, GNU
- **Code/commands**: `rm -rf /`, `caro "query"`
- **Placeholders**: `{count}`, `{name}`, `{{variable}}`
- **Version numbers**: v1.1.3, 52+
- **File paths**: `~/.config/caro/`

### ALWAYS Translate

- **Headlines and titles**
- **Descriptions and body text**
- **CTAs (call-to-action buttons)**
- **Error messages**
- **Navigation labels**
- **User-facing guidance**

### Cultural Adaptation

Each locale has a **metro context** defining its cultural voice:

| Locale | Metro | Tone |
|--------|-------|------|
| es | Madrid | Passionate, direct, confident |
| fr | Paris | Sophisticated, elegant, precise |
| pt | São Paulo | Warm, expressive, welcoming |
| de | Berlin | Precise, efficient, straightforward |
| he | Tel Aviv | Direct, informal (dugri), startup culture |
| ar | Dubai | Respectful, hospitable, elegant |
| ja | Tokyo | Polite, precise, harmonious (wa) |
| ko | Seoul | Community-focused, innovative |
| ... | ... | ... |

Full contexts available in `cultural-contexts.ts`.

## RTL Support

Hebrew, Arabic, and Urdu use **Right-to-Left** text direction.

### Layout Considerations

- `dir="rtl"` on `<html>` element
- Navigation mirrors horizontally
- Noto font families loaded
- Text alignment reversed
- Punctuation follows RTL rules

### In Components

```astro
---
import { isRtl } from '../i18n/config';

const lang = Astro.params.lang || 'en';
const direction = isRtl(lang) ? 'rtl' : 'ltr';
---

<html dir={direction} lang={lang}>
```

## Quality Validation

### CI/CD Integration

- **On PR**: Validates changed locale files
- **On push to main**: Full validation
- **Blocks merge**: If critical errors found
- **Warns**: If coverage below threshold

### Validation Rules

1. **JSON syntax**: Must be valid
2. **Placeholder preservation**: `{count}` → `{count}`
3. **Protected terms**: Brand names must be preserved
4. **No empty translations**: All values must have content
5. **Coverage threshold**: Tier 1 ≥ 80%

### Running Validation Locally

```bash
# Validate all locales
node scripts/i18n/validate.mjs

# Validate specific locale
node scripts/i18n/validate.mjs --locale es

# Strict mode (warnings = errors)
node scripts/i18n/validate.mjs --strict
```

## Coverage Reporting

### Check Current Status

```bash
node scripts/i18n/status.mjs
```

Output:
```
┌─────────┬──────────┬──────────┬─────────────┐
│ Locale  │ Keys     │ Translated│ Coverage   │
├─────────┼──────────┼──────────┼─────────────┤
│ es      │ 278      │ 231       │ 83.1%      │
│ fr      │ 278      │ 173       │ 62.2%      │
│ ...     │ ...      │ ...       │ ...        │
└─────────┴──────────┴──────────┴─────────────┘
```

### Tier-Specific Reporting

```bash
# Check only Tier 1 locales
node scripts/i18n/status.mjs --tier1-only

# Set minimum coverage threshold
node scripts/i18n/status.mjs --min-coverage 80 --tier1-only
```

## Adding New Locales

To add a new locale (e.g., `zh` for Chinese):

1. **Update config**:
   ```typescript
   // src/i18n/config.ts
   export type Locale = 'en' | 'es' | ... | 'zh';

   export const languages: Record<Locale, LocaleConfig> = {
     // ... existing locales
     zh: {
       code: 'zh',
       nativeName: '中文',
       englishName: 'Chinese',
       direction: 'ltr',
       isDefault: false
     }
   };
   ```

2. **Add cultural context**:
   ```typescript
   // src/i18n/cultural-contexts.ts
   zh: {
     metro: 'Shanghai',
     tone: '...',
     slang: '...',
     notes: '...'
   }
   ```

3. **Create locale directory**:
   ```bash
   mkdir src/i18n/locales/zh
   ```

4. **Sync keys**:
   ```bash
   node scripts/i18n/sync-keys.mjs
   ```

5. **Add to workflows**:
   - Update `LOCALES` array in `.github/workflows/translate-all.yml`
   - Update locale list in scripts

6. **Test**:
   ```bash
   node scripts/i18n/status.mjs
   node scripts/i18n/validate.mjs --locale zh
   ```

## Troubleshooting

### Issue: Locale not detecting from URL

**Solution**: Ensure URL pattern matches `/{locale}/path`
- ✅ `/es/features`
- ❌ `/features?lang=es`

### Issue: Translation not appearing

**Debug checklist**:
1. Check key exists in English: `src/i18n/locales/en/*.json`
2. Verify key exists in target locale
3. Ensure `t()` function called with correct key
4. Check browser console for errors
5. Verify locale is valid in `config.ts`

### Issue: RTL layout broken

**Debug checklist**:
1. Check `dir="rtl"` on `<html>` element
2. Verify Noto font loaded
3. Check CSS doesn't override `direction`
4. Test with browser DevTools RTL emulation

### Issue: Language switcher not persisting

**Debug**:
1. Check localStorage in DevTools
2. Verify `setLocalePreference()` called on selection
3. Test in incognito mode (localStorage may be blocked)

## Performance Considerations

### Bundle Size

- **Translations**: Loaded on-demand per locale (~30-50KB per locale)
- **Components**: Minimal overhead (~5KB)
- **Scripts**: Run at build time (no runtime cost)

### Caching

- **localStorage**: Persists user locale preference
- **CDN**: Static locale files cached
- **Build-time**: Translations baked into static HTML

## Roadmap

- [ ] Complete Tier 1 locales to 95%+ coverage
- [ ] Implement locale-specific date/time formatting
- [ ] Add currency localization
- [ ] Create translation memory system
- [ ] Build visual diff tool for translation changes
- [ ] Implement A/B testing for translations

## Resources

- **Translation Dashboard**: `/i18n/dashboard` (planned)
- **Coverage Report**: `npm run i18n:status`
- **Validation**: `npm run i18n:validate`
- **Sync Keys**: `npm run i18n:sync`

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for general guidelines.

For i18n contributions:
1. Follow translation rules in `translation-rules.ts`
2. Respect cultural contexts in `cultural-contexts.ts`
3. Run validation before submitting PR
4. Include coverage impact in PR description
5. Test in browser with target locale

---

**Questions?** Ask in #i18n channel or open an issue with `i18n` label.

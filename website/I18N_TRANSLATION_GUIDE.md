# Website Translation Guide (i18n)

How the caro website translates content into 15 languages using Astro's i18n routing, JSON translation files, and automated GitHub Actions workflows.

## Architecture Overview

```
website/
├── astro.config.mjs          # i18n routing config (locales, fallbacks)
├── src/
│   ├── i18n/
│   │   ├── config.ts         # t() function, Locale type, language metadata
│   │   ├── index.ts          # Merges all JSON files per locale (English = base)
│   │   ├── locales/
│   │   │   ├── en/           # English source files (7 JSON sections)
│   │   │   │   ├── common.json
│   │   │   │   ├── hero.json
│   │   │   │   ├── features.json
│   │   │   │   ├── download.json
│   │   │   │   ├── navigation.json
│   │   │   │   ├── landing.json
│   │   │   │   └── compare.json
│   │   │   ├── es/           # Spanish (common.json + landing.json)
│   │   │   ├── fr/           # French (common.json)
│   │   │   ├── ...           # 12 more locales
│   │   │   └── id/           # Indonesian
│   │   ├── translation-rules.ts   # What to translate vs protect
│   │   └── cultural-contexts.ts   # Per-locale tone, metro, slang
│   ├── lib/
│   │   ├── localized-links.ts     # localizedHref(), switchLocale()
│   │   └── locale-manager.ts      # detectLocale(), localStorage persistence
│   ├── components/
│   │   ├── LanguageSwitcher.astro # Dropdown UI for locale selection
│   │   └── HreflangMeta.astro     # <link rel="alternate"> SEO tags
│   └── pages/
│       ├── [lang]/                # Dynamic locale routes (/es/, /fr/, etc.)
│       │   ├── index.astro
│       │   ├── credits.astro
│       │   └── compare/
│       └── ...                    # English pages at / root
├── scripts/i18n/
│   ├── status.mjs            # Coverage report (per-locale %)
│   ├── validate.mjs          # Placeholder, protected-term, empty checks
│   └── sync-keys.mjs         # Ensure all locales match English key structure
└── .github/
    ├── workflows/
    │   ├── translate.yml          # Auto-translate on EN change (weekly cron)
    │   ├── translate-all.yml      # Manual mass translation
    │   └── validate-translations.yml  # PR validation (JSON + coverage)
    └── scripts/
        └── translate-multi-backend.js  # OpenAI / Claude / LibreTranslate
```

## Supported Locales (15)

| Code | Language        | Direction | Tier |
|------|-----------------|-----------|------|
| en   | English         | LTR       | Default |
| es   | Español         | LTR       | 1 |
| fr   | Français        | LTR       | 1 |
| pt   | Português       | LTR       | 1 |
| de   | Deutsch         | LTR       | 1 |
| ja   | 日本語           | LTR       | 1 |
| ko   | 한국어           | LTR       | 2 |
| he   | עברית           | **RTL**   | 2 |
| ar   | العربية          | **RTL**   | 2 |
| hi   | हिन्दी           | LTR       | 2 |
| ru   | Русский         | LTR       | 3 |
| uk   | Українська      | LTR       | 3 |
| ur   | اردو            | **RTL**   | 3 |
| fil  | Filipino        | LTR       | 3 |
| id   | Bahasa Indonesia| LTR       | 3 |

Tier 1 targets 80%+ coverage. Tier 2 targets 60%+. Tier 3 is best-effort.

## How Translation Works

### 1. English Source Files

All translatable strings live in `src/i18n/locales/en/*.json`. English is the **single source of truth**. Other locales override keys from English and fall back to English for missing keys.

Example `en/common.json`:
```json
{
  "common": {
    "buttons": {
      "copy": "Copy",
      "getStarted": "Get Started"
    },
    "nav": {
      "features": "Features",
      "compare": "Compare"
    }
  }
}
```

### 2. Locale Merging (src/i18n/index.ts)

Each locale starts as a copy of English, then locale-specific JSON files override matching keys:

```typescript
export const en = { ...navigationEn, ...heroEn, ...featuresEn, ... };
export const es = { ...en, ...commonEs, ...landingEs };  // EN base + ES overrides
export const fr = { ...en, ...commonFr };
```

If a key is missing from a locale, it automatically falls back to the English value. No 404s, no blank strings.

### 3. The `t()` Function (src/i18n/config.ts)

Components use `t(locale, 'dot.notation.key')` to get translated strings:

```astro
---
import { t, type Locale } from '../i18n/config';
const lang: Locale = 'es';
---
<h1>{t(lang, 'hero.title')}</h1>
<p>{t(lang, 'common.buttons.getStarted')}</p>
```

**Signature:** `t(locale: Locale, key: string): string`

**Fallback chain:** locale key → English key → raw key string (never empty).

For complex data (arrays/objects), use `getLocalizedData(locale, key)` instead.

### 4. Locale Detection (src/lib/locale-manager.ts)

Detection waterfall on page load:
1. **URL path** — `/es/features` → `es` (highest priority)
2. **localStorage** — `caro-locale` key persists user's manual selection
3. **Browser language** — `navigator.language` first match
4. **Default** — English

### 5. Localized Routing

Astro's `i18n` config in `astro.config.mjs`:
- `prefixDefaultLocale: false` — English stays at `/`, other locales get `/es/`, `/fr/`, etc.
- All non-English locales fall back to English
- Pages live in `src/pages/[lang]/` using `getStaticPaths()` to generate one page per locale

Localized link helpers (`src/lib/localized-links.ts`):
- `localizedHref('/features', 'es')` → `/es/features`
- `switchLocale('/es/features', 'fr')` → `/fr/features`
- `removeLocalePrefix('/es/features')` → `/features`

### 6. RTL Support

Three locales use right-to-left text: Hebrew (`he`), Arabic (`ar`), Urdu (`ur`).

The `isRtl(locale)` function from `src/i18n/config.ts` returns `true` for RTL locales. RTL locales also specify a `fontFamily` override (e.g., `'Noto Sans Arabic'`).

## Adding New Translated Strings

### Step 1: Add to English

Edit the appropriate JSON file in `src/i18n/locales/en/`:

```bash
# Example: adding a new button label
vim website/src/i18n/locales/en/common.json
```

### Step 2: Use in Components

```astro
---
import { t } from '../i18n/config';
---
<button>{t(lang, 'common.buttons.newAction')}</button>
```

### Step 3: Automated Translation

The `translate.yml` workflow automatically runs when English JSON files change on `main`. It:
1. Detects which files changed
2. Runs each locale through the translation backend (OpenAI/Claude/LibreTranslate)
3. Opens a PR per locale on branch `i18n/auto-translate-{locale}`

Manual trigger:
```bash
gh workflow run translate.yml -f backend=claude -f force_retranslate=true
```

### Step 4: Review and Merge

Each auto-generated PR includes a checklist:
- Review translated strings for accuracy
- Check placeholders like `{count}`, `{name}` are preserved
- Verify brand names like "Caro" remain unchanged
- Test RTL rendering for Hebrew/Arabic/Urdu

## Adding a New Locale

1. **Create locale directory**: `src/i18n/locales/{code}/common.json`
2. **Add to `Locale` type** in `src/i18n/config.ts`
3. **Add language metadata** to the `languages` record (code, nativeName, direction, fontFamily if RTL)
4. **Add to `astro.config.mjs`** `i18n.locales` array and `i18n.fallback` map
5. **Import in `src/i18n/index.ts`**: `import common{Code} from './locales/{code}/common.json'` and add `export const {code} = { ...en, ...common{Code} }`
6. **Add to LanguageSwitcher** options
7. **Add to translate.yml** matrix locale list
8. **Add to translate-multi-backend.js** cultural context (if using cultural-contexts.ts)
9. **Create `[lang]` pages** in `src/pages/[lang]/` using `getStaticPaths()`

## Translation Rules (src/i18n/translation-rules.ts)

### NEVER Translate
- Brand names: `Caro`, `Claude`, `POSIX`
- Technical terms: `CLI`, `API`, `BSD`, `GNU`, `MLX`
- Placeholders: `{count}`, `{name}`, `{version}`
- Code blocks, file paths, environment variables
- Version numbers

### ALWAYS Translate
- UI labels, button text, navigation
- Descriptions, help text, error messages
- Marketing copy, blog excerpts

### Cultural Context

Each locale has cultural context in `src/i18n/cultural-contexts.ts`:
- Metro city reference (e.g., Mexico City for `es`)
- Tone guidance (formal vs casual)
- Slang and pop culture references
- Code-switching notes (e.g., Hinglish for Hindi)

## Validation & Quality Assurance

### Scripts

```bash
# Check translation coverage
node website/scripts/i18n/status.mjs

# CI mode (exits 1 if Tier 1 < 80%)
node website/scripts/i18n/status.mjs --ci --min-coverage 80

# Validate placeholders, protected terms, empty strings
node website/scripts/i18n/validate.mjs --strict

# Sync missing keys from English to all locales
node website/scripts/i18n/sync-keys.mjs
```

### CI Workflows

| Workflow | Trigger | What it checks |
|----------|---------|----------------|
| `validate-translations.yml` | PR touching `locales/**/*.json` | JSON syntax, placeholder preservation, coverage threshold |
| `translate.yml` | Push to `main` changing `en/**/*.json`, weekly cron | Auto-generates translation PRs |
| `translate-all.yml` | Manual dispatch | Mass retranslate all locales |

## Adding Translated Pages

Only 3 of 8 planned localized page routes exist. To add a new one:

1. Create `src/pages/[lang]/your-page.astro`
2. Add `getStaticPaths()` filtering non-English locales
3. Import `t()` and use translated strings
4. The page generates at `/es/your-page`, `/fr/your-page`, etc.

Existing localized pages:
- `src/pages/[lang]/index.astro` → `/es/`, `/fr/`, etc.
- `src/pages/[lang]/credits.astro` → `/es/credits`, etc.
- `src/pages/[lang]/compare/index.astro` → `/es/compare`, etc.

Missing pages: `faq`, `glossary`, `roadmap`, `blog`, `support`.

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/i18n/config.ts` | `t()`, `isRtl()`, `Locale` type, language metadata |
| `src/i18n/index.ts` | Merges JSON files, exports per-locale translation objects |
| `src/i18n/locales/en/*.json` | English source strings (7 sections) |
| `src/lib/localized-links.ts` | URL generation helpers |
| `src/lib/locale-manager.ts` | Client-side locale detection + localStorage |
| `src/i18n/translation-rules.ts` | What to translate vs protect |
| `src/i18n/cultural-contexts.ts` | Per-locale cultural guidance for AI translators |
| `astro.config.mjs` | Astro i18n routing config |
| `.github/workflows/translate.yml` | Automated translation CI |
| `scripts/i18n/status.mjs` | Coverage report generator |
| `scripts/i18n/validate.mjs` | Translation validation |

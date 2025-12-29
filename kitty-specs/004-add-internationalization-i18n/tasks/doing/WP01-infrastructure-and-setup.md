---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
  - "T006"
title: "Infrastructure & Setup"
phase: "Phase 1 - Infrastructure"
lane: "doing"
assignee: "Claude Sonnet 4.5"
agent: "claude"
shell_pid: "68286"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
  - timestamp: "2025-12-29T01:00:00Z"
    lane: "doing"
    agent: "claude"
    shell_pid: "68286"
    action: "Started infrastructure and setup implementation"
---

# Work Package Prompt: WP01 – Infrastructure & Setup

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````astro`, ````typescript`, ````bash`

---

## Objectives & Success Criteria

**Goal**: Establish core i18n infrastructure with Astro config, translation utilities, RTL CSS foundation, and locale directory structure.

**Success Criteria**:
- Astro builds successfully with i18n configuration loaded
- `Layout.astro` renders with `dir` attribute computed from locale
- Translation utility functions (`t()`, `isRtl()`, `getLocaleConfig()`, `getAllLocales()`) are importable and functional
- All 15 locale directories exist and are empty (ready for Phase 2)
- RTL CSS logical properties replace all physical properties in global styles
- Google Fonts CDN links for Noto fonts (Hebrew, Arabic, Urdu) are present in `<head>`

**Independent Test**:
```bash
# Build test
cd website && npm run build

# Verify i18n config
grep -A 20 "i18n:" website/astro.config.mjs

# Verify translation utilities exist
ls website/src/i18n/config.ts

# Verify locale directories
ls website/src/i18n/locales/

# Test RTL rendering
npm run dev
# Visit http://localhost:4321/ and inspect <html dir="ltr">
```

---

## Context & Constraints

**Prerequisites**:
- `website/astro.config.mjs` exists (verified in research.md)
- Astro 4.16.17 is installed (confirmed in package.json)
- `website/src/layouts/Layout.astro` exists with holiday theme system

**Constraints**:
- Must maintain existing holiday theme system (Christmas/Hanukkah detection via `isFromIsrael()`)
- Must keep English at `/` root (use `prefixDefaultLocale: false`)
- Must use CSS logical properties throughout (no `margin-left`, `padding-right`, etc.)
- Must support all 15 locales: en, es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id

**Reference Documents**:
- `.specify/memory/constitution.md` - No over-engineering principle
- `kitty-specs/004-add-internationalization-i18n/plan.md` - Phase 1 details
- `kitty-specs/004-add-internationalization-i18n/contracts/translation-api.ts` - Type definitions
- `kitty-specs/004-add-internationalization-i18n/quickstart.md` - Implementation examples
- `kitty-specs/004-add-internationalization-i18n/research.md` - Decision rationale (native Astro i18n)

---

## Subtasks & Detailed Guidance

### Subtask T001 – Update Astro Config with i18n

**Purpose**: Enable Astro's native i18n routing system for all 15 locales.

**Steps**:
1. Open `website/astro.config.mjs`
2. Add `i18n` configuration object to `defineConfig()`:
   ```javascript
   export default defineConfig({
     site: 'https://caro.sh',
     i18n: {
       defaultLocale: 'en',
       locales: ['en', 'es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'],
       routing: {
         prefixDefaultLocale: false,  // Keep English at / root
         redirectToDefaultLocale: false,
       },
       fallback: {
         es: 'en', fr: 'en', pt: 'en', de: 'en',
         he: 'en', ar: 'en', uk: 'en', ru: 'en',
         ja: 'en', ko: 'en', hi: 'en', ur: 'en',
         fil: 'en', id: 'en'
       }
     }
   });
   ```
3. Verify Astro builds: `cd website && npm run build`

**Files**: `website/astro.config.mjs`

**Parallel?**: No (blocks all other i18n work)

**Notes**:
- `prefixDefaultLocale: false` keeps English URLs unchanged (`/about`, not `/en/about`)
- `fallback` ensures missing translations default to English
- Order of locales array doesn't matter, but keep alphabetical for readability

---

### Subtask T002 – Create Translation Utilities

**Purpose**: Provide type-safe translation functions for all components.

**Steps**:
1. Create `website/src/i18n/config.ts`
2. Define `Locale` type union matching `contracts/translation-api.ts`:
   ```typescript
   export type Locale =
     | 'en' | 'es' | 'fr' | 'pt' | 'de'
     | 'he' | 'ar' | 'uk' | 'ru' | 'ja'
     | 'ko' | 'hi' | 'ur' | 'fil' | 'id';

   export type TextDirection = 'ltr' | 'rtl';

   export interface LocaleConfig {
     code: Locale;
     nativeName: string;
     englishName: string;
     direction: TextDirection;
     fontFamily?: string;
     isDefault: boolean;
   }
   ```
3. Create `languages` configuration object:
   ```typescript
   export const languages: Record<Locale, LocaleConfig> = {
     en: { code: 'en', nativeName: 'English', englishName: 'English', direction: 'ltr', isDefault: true },
     es: { code: 'es', nativeName: 'Español', englishName: 'Spanish', direction: 'ltr', isDefault: false },
     fr: { code: 'fr', nativeName: 'Français', englishName: 'French', direction: 'ltr', isDefault: false },
     // ... (continue for all 15 locales)
     he: { code: 'he', nativeName: 'עברית', englishName: 'Hebrew', direction: 'rtl', fontFamily: "'Noto Sans Hebrew', sans-serif", isDefault: false },
     ar: { code: 'ar', nativeName: 'العربية', englishName: 'Arabic', direction: 'rtl', fontFamily: "'Noto Sans Arabic', sans-serif", isDefault: false },
     ur: { code: 'ur', nativeName: 'اردو', englishName: 'Urdu', direction: 'rtl', fontFamily: "'Noto Nastaliq Urdu', serif", isDefault: false },
   };
   ```
4. Implement utility functions:
   ```typescript
   export function t(locale: Locale, key: string): string {
     // Placeholder for WP02 - will load from JSON files
     return key; // For now, just return the key
   }

   export function isRtl(locale: Locale): boolean {
     return languages[locale]?.direction === 'rtl';
   }

   export function getLocaleConfig(locale: Locale): LocaleConfig {
     return languages[locale];
   }

   export function getAllLocales(): LocaleConfig[] {
     return Object.values(languages);
   }

   export function isValidLocale(code: string): code is Locale {
     return code in languages;
   }
   ```
5. Test import: Create a test file and verify functions are accessible

**Files**: `website/src/i18n/config.ts`

**Parallel?**: No (blocks component refactoring)

**Notes**:
- `t()` function is a placeholder - actual translation loading happens in WP02
- RTL locales: he, ar, ur (3 total)
- Font families are optional - only specified for RTL languages needing special fonts
- Native names use proper Unicode characters (Hebrew, Arabic, etc.)

---

### Subtask T003 – Update Layout with RTL Support

**Purpose**: Make `Layout.astro` locale-aware with proper `dir` attribute for RTL languages.

**Steps**:
1. Open `website/src/layouts/Layout.astro`
2. Update frontmatter to accept `lang` prop:
   ```astro
   ---
   import { isRtl, type Locale } from '../i18n/config';

   interface Props {
     title: string;
     lang?: Locale;
   }

   const { title, lang = 'en' } = Astro.props;
   const dir = isRtl(lang) ? 'rtl' : 'ltr';
   ---
   ```
3. Update `<html>` tag to include `lang` and `dir`:
   ```astro
   <html lang={lang} dir={dir}>
   ```
4. Preserve existing holiday theme logic (don't remove `isFromIsrael()`)
5. Test: Build and verify `<html dir="rtl">` when using RTL locale

**Files**: `website/src/layouts/Layout.astro`

**Parallel?**: No (foundation for T004, T005)

**Notes**:
- Default `lang='en'` ensures backward compatibility
- `dir` attribute is automatically computed from locale
- Holiday theme system should still work (Christmas/Hanukkah detection)
- Don't remove existing meta tags or scripts

---

### Subtask T004 – Add Noto Fonts CDN

**Purpose**: Load Google Fonts for RTL languages (Hebrew, Arabic, Urdu).

**Steps**:
1. In `Layout.astro` `<head>` section, add conditional font loading:
   ```astro
   <head>
     <!-- Existing meta tags... -->

     {lang === 'he' && (
       <link rel="preconnect" href="https://fonts.googleapis.com" />
       <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
       <link href="https://fonts.googleapis.com/css2?family=Noto+Sans+Hebrew:wght@400;700&display=swap" rel="stylesheet" />
     )}

     {lang === 'ar' && (
       <link rel="preconnect" href="https://fonts.googleapis.com" />
       <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
       <link href="https://fonts.googleapis.com/css2?family=Noto+Sans+Arabic:wght@400;700&display=swap" rel="stylesheet" />
     )}

     {lang === 'ur' && (
       <link rel="preconnect" href="https://fonts.googleapis.com" />
       <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
       <link href="https://fonts.googleapis.com/css2?family=Noto+Nastaliq+Urdu:wght@400;700&display=swap" rel="stylesheet" />
     )}
   </head>
   ```
2. Use `display=swap` for better performance (prevents FOUC)
3. Test: Verify fonts load in browser Network tab when accessing RTL locales

**Files**: `website/src/layouts/Layout.astro`

**Parallel?**: Yes (can be done alongside T005 after T003 completes)

**Notes**:
- Only load fonts for RTL locales (don't load for all languages)
- `preconnect` improves font loading performance
- Google Fonts CDN is free and reliable
- `display=swap` shows fallback font immediately, then swaps when Noto loads

---

### Subtask T005 – Add RTL CSS Logical Properties

**Purpose**: Replace all physical CSS properties with logical equivalents for RTL support.

**Steps**:
1. In `Layout.astro` `<style is:global>` section, replace physical properties:

   **Old (physical)**:
   ```css
   .container {
     margin-left: 20px;
     padding-right: 10px;
     border-left: 1px solid;
     text-align: left;
   }
   ```

   **New (logical)**:
   ```css
   .container {
     margin-inline-start: 20px;
     padding-inline-end: 10px;
     border-inline-start: 1px solid;
     text-align: start;
   }
   ```

2. Common replacements:
   - `margin-left` → `margin-inline-start`
   - `margin-right` → `margin-inline-end`
   - `padding-left` → `padding-inline-start`
   - `padding-right` → `padding-inline-end`
   - `border-left` → `border-inline-start`
   - `border-right` → `border-inline-end`
   - `left: 0` → `inset-inline-start: 0`
   - `right: 0` → `inset-inline-end: 0`
   - `text-align: left` → `text-align: start`
   - `text-align: right` → `text-align: end`

3. Add RTL font stack:
   ```css
   :lang(he) { font-family: 'Noto Sans Hebrew', sans-serif; }
   :lang(ar) { font-family: 'Noto Sans Arabic', sans-serif; letter-spacing: 0; }
   :lang(ur) { font-family: 'Noto Nastaliq Urdu', serif; }
   ```

4. Grep check for remaining physical properties:
   ```bash
   grep -E "margin-(left|right)|padding-(left|right)|text-align: (left|right)" website/src/layouts/Layout.astro
   ```
   Should return no matches.

**Files**: `website/src/layouts/Layout.astro`

**Parallel?**: Yes (can be done alongside T004 after T003 completes)

**Notes**:
- Arabic requires `letter-spacing: 0` (connected script)
- Don't change vertical properties (margin-top, padding-bottom) - those stay physical
- Flexbox `flex-direction` doesn't need changes - CSS handles RTL automatically
- Some properties like `position: absolute` with `left`/`right` may need manual RTL overrides later

---

### Subtask T006 – Create Locale Directory Structure

**Purpose**: Prepare empty locale directories for translation files in Phase 2.

**Steps**:
1. Create directories for all 15 locales:
   ```bash
   mkdir -p website/src/i18n/locales/{en,es,fr,pt,de,he,ar,uk,ru,ja,ko,hi,ur,fil,id}
   ```
2. Verify directory structure:
   ```bash
   ls website/src/i18n/locales/
   # Should show: ar  de  en  es  fil  fr  he  hi  id  ja  ko  pt  ru  uk  ur
   ```
3. Leave directories empty (no JSON files yet - that's WP02)

**Files**: `website/src/i18n/locales/*` (15 directories)

**Parallel?**: No (needed for WP02)

**Notes**:
- Directory names match ISO 639-1 language codes exactly
- Lowercase only (e.g., `fil` not `Fil`)
- Order doesn't matter (alphabetical for convenience)
- No files created yet - just empty directories

---

## Test Strategy

**Manual Testing**:
1. **Build Test**:
   ```bash
   cd website
   npm run build
   ```
   Should complete without errors.

2. **Development Server Test**:
   ```bash
   npm run dev
   # Visit http://localhost:4321/
   ```
   - Inspect HTML: `<html lang="en" dir="ltr">`
   - Verify no console errors
   - Check Network tab: No 404s for missing files

3. **RTL Test** (will work fully after WP06, but structure should be ready):
   - Manually change Layout.astro to `const { lang = 'he' }`
   - Rebuild and verify `<html lang="he" dir="rtl">`
   - Check if Noto Sans Hebrew font loads in Network tab

4. **Import Test**:
   ```typescript
   // Create test file: website/src/test-i18n.ts
   import { t, isRtl, getLocaleConfig, getAllLocales } from './i18n/config';

   console.log(isRtl('he')); // Should log: true
   console.log(isRtl('en')); // Should log: false
   console.log(getAllLocales().length); // Should log: 15
   ```
   Run: `node website/src/test-i18n.ts` (or use TypeScript compiler)

---

## Risks & Mitigations

**Risk 1: Astro i18n routing breaking existing URLs**
- **Impact**: English pages become inaccessible or redirect incorrectly
- **Mitigation**: Use `prefixDefaultLocale: false` in config - this keeps `/about` working, not `/en/about`
- **Test**: Visit existing URLs after config change

**Risk 2: Holiday theme system breaking**
- **Impact**: Christmas/Hanukkah theme detection stops working
- **Mitigation**: Preserve `isFromIsrael()` function and all holiday theme logic in Layout.astro
- **Test**: Verify holiday themes still toggle correctly (may need to mock timezone)

**Risk 3: RTL CSS conflicts with existing styles**
- **Impact**: Layout breaks on RTL pages
- **Mitigation**: Use CSS logical properties exclusively - they work bidirectionally
- **Test**: Visual inspection of `/he/` route after WP06 (routes don't exist yet)

**Risk 4: Font loading delays (FOUC - Flash of Unstyled Content)**
- **Impact**: RTL text shows in wrong font briefly on page load
- **Mitigation**: Use `display=swap` in Google Fonts URL + preconnect for faster loading
- **Test**: Check Network waterfall - fonts should load within 100-200ms

---

## Definition of Done Checklist

- [ ] `website/astro.config.mjs` contains i18n configuration with 15 locales
- [ ] `website/src/i18n/config.ts` exists with all utility functions
- [ ] `website/src/layouts/Layout.astro` accepts `lang` prop and computes `dir` attribute
- [ ] Google Fonts CDN links for Noto fonts are present in `<head>`
- [ ] All physical CSS properties replaced with logical properties in global styles
- [ ] All 15 locale directories exist: `website/src/i18n/locales/{en,es,fr,...}`
- [ ] `npm run build` completes successfully
- [ ] No console errors in development mode
- [ ] Import test for translation utilities passes
- [ ] Grep check confirms no physical properties remain (`margin-left`, `padding-right`, etc.)
- [ ] `tasks.md` updated with WP01 status change

---

## Review Guidance

**Key Acceptance Checkpoints**:
1. **Configuration Correctness**: Verify `astro.config.mjs` has all 15 locales and correct routing settings
2. **Type Safety**: Check `config.ts` types match `contracts/translation-api.ts` exactly
3. **RTL Foundation**: Confirm `dir` attribute logic works (`isRtl()` function)
4. **CSS Logical Properties**: Grep for physical properties - should find zero matches
5. **No Breaking Changes**: Existing English pages still load at `/` root

**Context to Revisit**:
- `quickstart.md` - Examples of translation usage patterns
- `research.md` - Why we chose native Astro i18n (no external libraries)
- `plan.md` Phase 1 - Full technical context for infrastructure setup

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created via /spec-kitty.tasks

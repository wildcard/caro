# Implementation Plan: Website Internationalization with 15 Languages

**Branch**: `004-add-internationalization-i18n` | **Date**: 2025-12-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/kitty-specs/004-add-internationalization-i18n/spec.md`

## Summary

Implement comprehensive internationalization (i18n) for the Caro marketing website, supporting 15 languages (including 3 RTL: Hebrew, Arabic, Urdu) with automated OpenAI GPT-4 translations. Uses Astro's native i18n routing, component-based JSON translation files, and GitHub Actions for automation. Scope limited to marketing pages (homepage, landing pages, comparison pages, navigation/footer) - excludes blog, docs, and CLI.

**Key Technical Decisions**:
- Native Astro i18n (no external libraries)
- Component-based translation structure (~10 JSON files per locale)
- Official OpenAI Node.js SDK for reliability
- CSS logical properties for RTL support
- Google Fonts CDN for Noto font families

## Technical Context

**Language/Version**: TypeScript/JavaScript (Astro 4.16.17), Node.js 20+
**Primary Dependencies**:
  - Astro 4.0+ with native i18n support
  - OpenAI Node.js SDK (~2.x) for GitHub Action
  - Google Fonts CDN (Noto Sans Hebrew, Arabic, Nastaliq Urdu)

**Storage**: JSON files on disk (`website/src/i18n/locales/{locale}/{section}.json`), version-controlled via git

**Testing**: Manual testing for each locale, visual regression testing for RTL layouts, spot-check review of automated translations

**Target Platform**: Static website (Astro SSG), deployed to Vercel/GitHub Pages

**Project Type**: Web application (Astro frontend)

**Performance Goals**:
  - Page load time increase <100ms for localized vs English pages (SC-007)
  - Build time <15 seconds for all 15 locales
  - Translation generation <10 minutes per GitHub Action run

**Constraints**:
  - Budget: ~$6 per full translation run (OpenAI GPT-4 API)
  - 95% translation accuracy via spot-check (SC-004)
  - All 15 languages launch together (no phased rollout)
  - Marketing pages only (no blog/docs/CLI)

**Scale/Scope**:
  - 15 locales (1 default + 14 translated)
  - ~200 translatable strings across 10 translation files
  - ~30 Astro components to refactor
  - 5 main pages (homepage + 4 landing/comparison pages)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Project Constitution**: Template constitution in `.kittify/memory/constitution.md` is unpopulated. Using CLAUDE.md principles instead.

### CLAUDE.md Alignment

**✅ Simplicity Principle** ("Avoid over-engineering. Only make changes that are directly requested or clearly necessary.")
- Using native Astro i18n (built-in) vs external library
- Component-based structure (proven pattern) vs complex nested hierarchy
- No premature abstractions - straightforward `t(locale, key)` function

**✅ Security-First** ("Be careful not to introduce security vulnerabilities")
- API keys in GitHub secrets (not hardcoded)
- No user-generated translations (only GPT-4 automated)
- Input validation for locale codes (TypeScript types)

**✅ No Panics** ("No panics in production code - use Result types")
- Graceful fallback to English if translation missing
- Error handling in GitHub Action (retry logic, fail states)

**✅ Quality Standards** ("All public APIs must have documentation")
- Translation API contract defined (`contracts/translation-api.ts`)
- Quickstart guide for developers (`quickstart.md`)
- Inline JSDoc for all exported functions

**Post-Design Re-Check** (after Phase 1):
- ✅ No new violations introduced
- ✅ Translation utility functions follow TypeScript best practices
- ✅ GitHub Action follows existing workflow patterns

## Project Structure

### Documentation (this feature)

```
kitty-specs/004-add-internationalization-i18n/
├── spec.md              # Feature specification
├── plan.md              # This file (implementation plan)
├── research.md          # Phase 0 research decisions
├── data-model.md        # Entity definitions
├── quickstart.md        # Developer guide
├── contracts/           # API contracts
│   ├── translation-api.ts          # Translation function signatures
│   └── github-action-workflow.yml  # GitHub Action contract
├── checklists/
│   └── requirements.md  # Spec quality validation
└── research/
    ├── evidence-log.csv        # Research findings
    └── source-register.csv     # Source tracking
```

### Source Code (repository root)

```
website/
├── astro.config.mjs     # [MODIFY] Add i18n configuration
├── src/
│   ├── layouts/
│   │   └── Layout.astro         # [MODIFY] Add lang/dir props, RTL CSS
│   ├── components/
│   │   ├── Navigation.astro     # [MODIFY] Refactor for i18n
│   │   ├── Hero.astro           # [MODIFY] Refactor for i18n
│   │   ├── Features.astro       # [MODIFY] Refactor for i18n
│   │   ├── Download.astro       # [MODIFY] Refactor for i18n
│   │   ├── Footer.astro         # [MODIFY] Refactor for i18n
│   │   └── LanguageSwitcher.astro  # [NEW] Language selection UI
│   ├── pages/
│   │   ├── index.astro          # [MODIFY] Pass lang to components
│   │   └── [lang]/              # [NEW] Localized routes
│   │       ├── index.astro      # [NEW] Homepage for non-English
│   │       ├── credits.astro    # [NEW] Credits page
│   │       └── compare/         # [NEW] Comparison pages
│   └── i18n/                    # [NEW] Translation infrastructure
│       ├── config.ts            # [NEW] Translation utilities
│       ├── index.ts             # [NEW] Export all translations
│       └── locales/
│           ├── en/              # [NEW] English (source)
│           │   ├── common.json
│           │   ├── navigation.json
│           │   ├── hero.json
│           │   ├── features.json
│           │   ├── download.json
│           │   ├── faq.json
│           │   ├── landing.json
│           │   └── compare.json
│           ├── es/              # [NEW] Spanish (auto-generated)
│           ├── fr/              # [NEW] French (auto-generated)
│           ├── pt/              # [NEW] Portuguese (auto-generated)
│           ├── de/              # [NEW] German (auto-generated)
│           ├── he/              # [NEW] Hebrew (auto-generated, RTL)
│           ├── ar/              # [NEW] Arabic (auto-generated, RTL)
│           ├── uk/              # [NEW] Ukrainian (auto-generated)
│           ├── ru/              # [NEW] Russian (auto-generated)
│           ├── ja/              # [NEW] Japanese (auto-generated)
│           ├── ko/              # [NEW] Korean (auto-generated)
│           ├── hi/              # [NEW] Hindi (auto-generated)
│           ├── ur/              # [NEW] Urdu (auto-generated, RTL)
│           ├── fil/             # [NEW] Filipino (auto-generated)
│           └── id/              # [NEW] Indonesian (auto-generated)

.github/
├── workflows/
│   └── translate.yml            # [NEW] Translation automation
└── scripts/
    └── translate.js             # [NEW] OpenAI translation script
```

**Structure Decision**: Web application structure selected. Astro SSG frontend with translation files stored in `website/src/i18n/locales/`. GitHub Action automation in `.github/workflows/`. No backend required (static translations).

## Complexity Tracking

*No violations - all decisions align with constitution principles.*

## Parallel Work Analysis

### Dependency Graph

```
Phase 1: Infrastructure (Day 1 - Sequential)
  ├── Astro config.mjs i18n setup
  ├── Layout.astro RTL support
  └── Translation utility (config.ts)

Phase 2: Translation Files (Day 2 - Sequential)
  ├── Extract English strings to JSON
  └── Create translation file structure

Phase 3: Component Refactoring (Days 3-4 - Parallel possible)
  ├── Stream A: Navigation, Footer (shared components)
  ├── Stream B: Hero, Features, Download (page sections)
  └── Stream C: Landing page components (12 LP components)

Phase 4: Localized Routes (Day 5 - Sequential)
  ├── Create [lang] directory structure
  └── Implement getStaticPaths()

Phase 5: Automation (Day 6 - Sequential)
  ├── GitHub Action workflow
  ├── Translation script
  └── Initial translation run

Phase 6: Polish (Day 7 - Sequential)
  ├── Language switcher component
  ├── RTL testing and fixes
  └── Performance optimization
```

### Work Distribution

- **Sequential work** (must be done first):
  - Phase 1: Infrastructure (Astro config, Layout, translation utility)
  - Phase 2: English translation files (source of truth)

- **Parallel streams** (can be done simultaneously):
  - **Stream A**: Navigation.astro, Footer.astro refactoring
  - **Stream B**: Hero.astro, Features.astro, Download.astro refactoring
  - **Stream C**: 12 landing page components (LPHero, LPFeatures, etc.)

- **Agent assignments** (if using multiple agents):
  - Agent 1: Core infrastructure + Navigation/Footer
  - Agent 2: Page sections (Hero, Features, Download)
  - Agent 3: Landing page components
  - Agent 4: GitHub Action automation

### Coordination Points

- **Sync schedule**: End of Day 2 (after translation files complete), end of Day 4 (after component refactoring)
- **Integration tests**: Manual testing of `/es/`, `/he/`, `/ja/` routes after each sync
- **Merge strategy**: Sequential merges to avoid conflicts in shared files (config.ts, Layout.astro)

## Implementation Phases

### Phase 1: Infrastructure (Priority: Critical)

**Goal**: Set up Astro i18n configuration, RTL support, and translation utility functions.

**Files Modified**:
- `website/astro.config.mjs` - Add i18n config
- `website/src/layouts/Layout.astro` - Add lang/dir props, RTL CSS
- `website/src/i18n/config.ts` (new) - Translation utilities

**Implementation Steps**:

1. **Update Astro Config**
   ```javascript
   // website/astro.config.mjs
   export default defineConfig({
     site: 'https://caro.sh',
     integrations: [sitemap()],
     i18n: {
       defaultLocale: 'en',
       locales: ['en', 'es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'],
       routing: {
         prefixDefaultLocale: false,
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

2. **Update Layout for RTL**
   ```astro
   ---
   // website/src/layouts/Layout.astro
   interface Props {
     title: string;
     description: string;
     lang?: string;  // Add lang prop
   }

   const { title, description, lang = 'en' } = Astro.props;
   const dir = ['he', 'ar', 'ur'].includes(lang) ? 'rtl' : 'ltr';
   ---

   <!doctype html>
   <html lang={lang} dir={dir}>
     <!-- Rest of layout -->
   </html>

   <style is:global>
     /* Add RTL CSS */
     [dir="rtl"] .nav-links {
       flex-direction: row-reverse;
     }

     :lang(he) {
       font-family: 'Noto Sans Hebrew', sans-serif;
     }

     :lang(ar) {
       font-family: 'Noto Sans Arabic', sans-serif;
       letter-spacing: 0;
     }

     :lang(ur) {
       font-family: 'Noto Nastaliq Urdu', sans-serif;
     }
   </style>
   ```

3. **Create Translation Utility**
   - Implement `config.ts` per `contracts/translation-api.ts` specification
   - Export: `t()`, `getTranslations()`, `isRtl()`, `getAllLocales()`

**Acceptance Criteria**:
- Astro builds successfully with i18n config
- Layout accepts `lang` prop and sets `dir` attribute
- Translation utility exports all required functions

---

### Phase 2: Translation Files (Priority: Critical)

**Goal**: Extract all English strings from components and create JSON translation file structure.

**Files Created**:
- `website/src/i18n/locales/en/*.json` (8 files: common, navigation, hero, features, download, faq, landing, compare)

**Implementation Steps**:

1. **Audit Components for Strings**
   - Read Navigation.astro, Hero.astro, Features.astro, etc.
   - Extract all hardcoded text
   - Categorize by section (navigation, hero, features, etc.)

2. **Create English JSON Files**
   ```json
   // locales/en/navigation.json
   {
     "brand": "Caro",
     "links": {
       "features": "Features",
       "compare": "Compare",
       "play": "Play",
       "blog": "Blog",
       "getStarted": "Get Started",
       "github": "GitHub"
     },
     "theme": {
       "toggleDarkMode": "Toggle dark mode",
       "holidayTheme": "Holiday Theme"
     }
   }
   ```

3. **Create Directory Structure**
   ```bash
   mkdir -p website/src/i18n/locales/{en,es,fr,pt,de,he,ar,uk,ru,ja,ko,hi,ur,fil,id}
   ```

**Acceptance Criteria**:
- All English strings extracted from components
- 8 JSON files created in `locales/en/`
- Key naming follows dot-notation convention
- Placeholders documented (e.g., `{count}`, `{name}`)

---

### Phase 3: Component Refactoring (Priority: High)

**Goal**: Refactor Astro components to use translation functions instead of hardcoded strings.

**Priority Order** (implement in this sequence):
1. Navigation.astro (shared, high visibility)
2. Footer.astro (shared, high visibility)
3. Hero.astro (homepage)
4. Features.astro (homepage)
5. Download.astro (homepage)
6. 12 LP components (landing pages)

**Implementation Pattern** (apply to each component):

```astro
---
// Before
---
<nav>
  <a href="/">🐕 Caro</a>
  <a href="/#features">Features</a>
</nav>

// After
---
import { t, type Locale } from '../i18n/config';

interface Props {
  lang?: Locale;
}

const { lang = 'en' } = Astro.props;
---
<nav>
  <a href="/">🐕 {t(lang, 'navigation.brand')}</a>
  <a href="/#features">{t(lang, 'navigation.links.features')}</a>
</nav>
```

**Acceptance Criteria** (per component):
- Props interface includes `lang?: Locale`
- All hardcoded strings replaced with `t(lang, 'key')` calls
- Component renders correctly with `lang="en"`
- Component renders correctly with `lang="es"` (test translation)

---

### Phase 4: Localized Routes (Priority: High)

**Goal**: Create dynamic locale routes using Astro's `[lang]` directory pattern.

**Files Created**:
- `website/src/pages/[lang]/index.astro` (homepage)
- `website/src/pages/[lang]/credits.astro`
- `website/src/pages/[lang]/compare/*.astro` (5 comparison pages)

**Implementation Steps**:

1. **Create [lang] Directory**
   ```bash
   mkdir -p website/src/pages/[lang]/compare
   ```

2. **Implement Dynamic Route**
   ```astro
   ---
   // website/src/pages/[lang]/index.astro
   import Layout from '../../layouts/Layout.astro';
   import { languages, type Locale } from '../../i18n/config';
   import Navigation from '../../components/Navigation.astro';
   import Hero from '../../components/Hero.astro';

   export function getStaticPaths() {
     return Object.keys(languages)
       .filter(lang => lang !== 'en')
       .map(lang => ({ params: { lang } }));
   }

   const { lang } = Astro.params as { lang: Locale };
   ---

   <Layout title="Caro" lang={lang}>
     <Navigation lang={lang} />
     <Hero lang={lang} />
     <!-- Pass lang to all components -->
   </Layout>
   ```

3. **Add hreflang Meta Tags**
   ```astro
   // In Layout.astro <head>
   {getAllLocales().map(locale => (
     <link rel="alternate" hreflang={locale.code} href={`https://caro.sh/${locale.code === 'en' ? '' : locale.code + '/'}`} />
   ))}
   ```

**Acceptance Criteria**:
- Build generates routes for all 14 non-English locales
- `/es/`, `/fr/`, `/he/` etc. all render correctly
- `hreflang` tags present for SEO
- Internal links maintain locale (e.g., clicking Features from `/es/` goes to `/es/#features`)

---

### Phase 5: GitHub Action Automation (Priority: High)

**Goal**: Automate translation generation using OpenAI GPT-4 when English files change.

**Files Created**:
- `.github/workflows/translate.yml`
- `.github/scripts/translate.js`

**Implementation Steps**:

1. **Create Workflow** (per `contracts/github-action-workflow.yml`)
   - Trigger on changes to `website/src/i18n/locales/en/**`
   - Matrix strategy for 14 locales
   - Artifact upload/download
   - PR creation with labels

2. **Create Translation Script**
   ```javascript
   // .github/scripts/translate.js
   const OpenAI = require('openai');
   const fs = require('fs');

   const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });

   async function translateString(text, locale, context) {
     const systemPrompt = `You are a professional translator for a software product called Caro.
     Translate from English to ${localeInfo[locale].name}.
     Preserve placeholders like {count}, {name}.
     Keep brand name "Caro" untranslated.
     Maintain technical terms (POSIX, shell, CLI).`;

     const response = await openai.chat.completions.create({
       model: 'gpt-4-turbo-preview',
       messages: [
         { role: 'system', content: systemPrompt },
         { role: 'user', content: text }
       ],
       temperature: 0.3,
     });

     return response.choices[0].message.content.trim();
   }

   // Implement file processing, error handling, retry logic
   ```

3. **Configure Secret**
   - Add `OPENAI_API_KEY` to GitHub repository secrets

**Acceptance Criteria**:
- Workflow triggers on English file changes
- Generates translations for all 14 locales
- Creates PR with `i18n` label
- PR includes review checklist
- Estimated run time <10 minutes
- Estimated cost ~$4-6 per run

---

### Phase 6: Language Switcher & Polish (Priority: Medium)

**Goal**: Add language switcher UI, optimize performance, finalize RTL styling.

**Files Created/Modified**:
- `website/src/components/LanguageSwitcher.astro` (new)
- `website/src/layouts/Layout.astro` (modify - add switcher)

**Implementation Steps**:

1. **Create Language Switcher Component**
   ```astro
   ---
   import { getAllLocales, type Locale } from '../i18n/config';

   const allLocales = getAllLocales();
   const currentLang = Astro.params.lang || 'en';
   ---

   <div class="lang-switcher">
     <select id="lang-select">
       {allLocales.map(locale => (
         <option value={locale.code} selected={locale.code === currentLang}>
           {locale.nativeName}
         </option>
       ))}
     </select>
   </div>

   <script>
     // Redirect logic with localStorage persistence
   </script>
   ```

2. **RTL Testing**
   - Test `/he/`, `/ar/`, `/ur/` in Chrome/Firefox/Safari
   - Verify text direction, layout mirroring, font rendering
   - Fix any CSS issues

3. **Performance Optimization**
   - Verify lazy-loading of translation files
   - Test page load times (target: <100ms increase)
   - Optimize build time (<15 seconds)

**Acceptance Criteria**:
- Language switcher shows all 15 languages
- Selection persists in localStorage
- RTL languages render correctly without layout breaks
- SC-007 met: Page load time increase <100ms

---

## Testing Strategy

### Manual Testing Checklist

**Per Locale**:
- [ ] Navigate to `/[locale]/` (e.g., `/es/`, `/fr/`)
- [ ] Verify all UI strings translated
- [ ] Test navigation links (maintain locale)
- [ ] Check placeholders preserved (e.g., `{count}`)

**RTL Specific** (Hebrew, Arabic, Urdu):
- [ ] Text flows right-to-left
- [ ] Navigation items reversed
- [ ] `<html dir="rtl">` attribute present
- [ ] Font renders correctly
- [ ] No layout breaks

**Language Switcher**:
- [ ] Dropdown shows all 15 languages in native scripts
- [ ] Selection changes URL and content
- [ ] Selection persists across page loads
- [ ] Works from any page

**GitHub Action**:
- [ ] Triggers on English file changes
- [ ] Generates translations for 14 locales
- [ ] Creates PR with review checklist
- [ ] Spot-check 2-3 strings per language
- [ ] Merge PR and verify deployment

### Success Criteria Validation

- **SC-001**: All 15 languages accessible ✓ (test `/[locale]/` for all)
- **SC-002**: RTL languages render correctly ✓ (test `/he/`, `/ar/`, `/ur/`)
- **SC-003**: Translations available in <24 hours ✓ (measure GitHub Action time)
- **SC-004**: 95% translation accuracy ✓ (spot-check review process)
- **SC-005**: Language preference persists ✓ (test localStorage)
- **SC-006**: Placeholders preserved ✓ (verify `{count}`, `{name}` unchanged)
- **SC-007**: Page load <100ms increase ✓ (measure with Lighthouse)
- **SC-008**: hreflang tags present ✓ (inspect page source)

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| OpenAI API costs exceed budget | Medium | Set concurrency limits, cache translations, monitor spending | Planned |
| Translation quality issues | Medium | Spot-check review, community feedback, iterative fixes | Planned |
| RTL layout breaks | Low | Test with existing themes, use logical properties | Planned |
| Performance degradation | Low | Lazy-load translation files, Astro code splitting | Planned |
| Merge conflicts | Low | Component-based structure (10 files vs 1) | Planned |

## Open Questions

None - all planning questions resolved during discovery phase.

## Next Steps

1. Run `/spec-kitty.tasks` to generate work packages from this plan
2. Begin Phase 1 (Infrastructure) implementation
3. Test each phase before proceeding to next
4. Run `/spec-kitty.review` for code quality checks
5. Run `/spec-kitty.accept` for final validation
6. Run `/spec-kitty.merge` to integrate into main

---

**Plan Status**: Ready for task generation
**Last Updated**: 2025-12-28
**Reviewed By**: Claude (AI Agent)

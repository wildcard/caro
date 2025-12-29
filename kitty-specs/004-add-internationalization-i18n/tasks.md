---
description: "Work package task list for Website Internationalization (i18n) with 15 Languages"
---

# Work Packages: Website Internationalization (i18n)

**Feature**: Add full internationalization support with 15 languages (3 RTL, 12 LTR) and automated GitHub Action for OpenAI GPT-4 translations

**Inputs**: Design documents from `/kitty-specs/004-add-internationalization-i18n/`
**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/, quickstart.md

**Organization**: 38 fine-grained subtasks (`T001`-`T038`) rolled up into 8 work packages (`WP01`-`WP08`). Each work package is independently deliverable and testable.

**Prompt Files**: Each work package references a matching prompt file in `/tasks/planned/` for detailed implementation guidance.

## Subtask Format: `[Txxx] [P?] Description`
- **[P]** indicates the subtask can proceed in parallel (different files/components).
- All file paths are relative to repository root.

## Path Conventions
- **Website**: `website/src/`, `website/astro.config.mjs`
- **i18n**: `website/src/i18n/`, `website/src/i18n/locales/{locale}/`
- **GitHub Action**: `.github/workflows/`, `.github/scripts/`

---

## Work Package WP01: Infrastructure & Setup (Priority: P0 - Critical)

**Goal**: Establish core i18n infrastructure with Astro config, translation utilities, RTL CSS foundation, and locale directory structure.
**Independent Test**: Astro builds successfully with i18n config loaded. Layout.astro renders with `dir` attribute. Translation utility functions are importable.
**Prompt**: `/tasks/planned/WP01-infrastructure-and-setup.md`

### Included Subtasks
- [ ] T001 Update `website/astro.config.mjs` with i18n configuration (15 locales, routing, fallback)
- [ ] T002 Create `website/src/i18n/config.ts` with translation utilities (t(), isRtl(), getLocaleConfig(), getAllLocales())
- [ ] T003 Update `website/src/layouts/Layout.astro` with `lang` prop and `dir` attribute for RTL support
- [ ] T004 Add Google Fonts CDN links for Noto fonts (Hebrew, Arabic, Urdu) in Layout.astro `<head>`
- [ ] T005 Add RTL CSS logical properties to Layout.astro global styles (margin-inline, padding-inline, text-align: start)
- [ ] T006 Create directory structure `website/src/i18n/locales/{en,es,fr,pt,de,he,ar,uk,ru,ja,ko,hi,ur,fil,id}/`

### Implementation Notes
1. Start with Astro config to enable i18n routing globally
2. Create translation utility with TypeScript types matching `contracts/translation-api.ts`
3. Update Layout.astro to accept `lang` prop and compute `dir` from RTL locale list
4. Add Noto font CDN links conditionally based on locale
5. Replace all physical CSS properties (margin-left, padding-right) with logical properties
6. Create empty locale directories to prepare for Phase 2 extraction

### Parallel Opportunities
- T004 (fonts) and T005 (CSS) can be done in parallel after T003 (Layout.astro structure) is complete

### Dependencies
- None (starting package)

### Risks & Mitigations
- **Risk**: Astro i18n routing breaking existing URLs
  - **Mitigation**: Use `prefixDefaultLocale: false` to keep English at `/` root
- **Risk**: RTL CSS conflicts with holiday theme system
  - **Mitigation**: Use CSS logical properties throughout - they work with existing theme CSS

---

## Work Package WP02: Translation File Extraction (Priority: P0 - Critical)

**Goal**: Extract all English strings from existing components to JSON translation files, organized by section (navigation, hero, features, download, common, landing, compare).
**Independent Test**: All JSON files exist in `locales/en/` with complete string coverage. No hardcoded strings remain in targeted components (verified by grep). JSON is valid and parseable.
**Prompt**: `/tasks/planned/WP02-translation-file-extraction.md`

### Included Subtasks
- [ ] T007 [P] Extract Navigation component strings to `locales/en/navigation.json`
- [ ] T008 [P] Extract Hero section strings to `locales/en/hero.json`
- [ ] T009 [P] Extract Features section strings to `locales/en/features.json`
- [ ] T010 [P] Extract Download section strings to `locales/en/download.json`
- [ ] T011 [P] Extract Footer component strings to `locales/en/navigation.json` (or create `footer.json` if warranted)
- [ ] T012 [P] Extract common UI strings (buttons, labels, status) to `locales/en/common.json`
- [ ] T013 [P] Extract landing page strings to `locales/en/landing.json`
- [ ] T014 [P] Extract comparison page strings to `locales/en/compare.json`
- [ ] T015 Create `website/src/i18n/index.ts` to export all translation objects

### Implementation Notes
- Use hierarchical dot-notation keys: `section.subsection.key`
- Preserve placeholders like `{count}`, `{name}` for dynamic content
- Group related strings logically (e.g., all nav links under `navigation.links`)
- Common strings should be truly shared across multiple components
- Reference existing components to find all hardcoded strings

### Parallel Opportunities
- **All subtasks T007-T014 are fully parallel** - each targets a different JSON file
- T015 (index.ts export) must wait for all others to complete

### Dependencies
- Depends on WP01 (directory structure must exist)

### Risks & Mitigations
- **Risk**: Missing strings during extraction
  - **Mitigation**: Use grep to verify no hardcoded strings remain: `grep -r "Caro" website/src/components/`
- **Risk**: Key naming inconsistencies across files
  - **Mitigation**: Follow pattern from `quickstart.md` examples

---

## Work Package WP03: Component Refactoring - Navigation & Core (Priority: P1)

**Goal**: Refactor critical navigation and layout components to use translation functions. Add hreflang meta tags for SEO.
**Independent Test**: Navigation and Footer render correctly in all 15 languages. Hreflang tags present in page `<head>`. No hardcoded strings remain in these components.
**Prompt**: `/tasks/planned/WP03-component-refactoring-navigation-core.md`

### Included Subtasks
- [ ] T016 Refactor `website/src/components/Navigation.astro` to use `t(lang, 'navigation.links.*')` function
- [ ] T017 Refactor `website/src/components/Footer.astro` to use `t()` function for footer links
- [ ] T018 Add `hreflang` meta tags to `Layout.astro` `<head>` section linking to all 15 language versions

### Implementation Notes
1. Import `t` and `Locale` type from `../i18n/config`
2. Add `lang?: Locale` to Props interface with default `'en'`
3. Replace all hardcoded strings with `t(lang, 'key.path')` calls
4. Pass `lang` prop to all child components
5. For hreflang, generate `<link rel="alternate" hreflang="es" href="/es/" />` for each locale

### Parallel Opportunities
- T016 (Navigation) and T017 (Footer) can be done in parallel
- T018 (hreflang) can be done in parallel with T016/T017

### Dependencies
- Depends on WP02 (translation JSON files must exist)

### Risks & Mitigations
- **Risk**: Breaking holiday theme system (Christmas/Hanukkah detection)
  - **Mitigation**: Preserve existing `isFromIsrael()` logic, ensure it works with RTL Hebrew
- **Risk**: Navigation layout breaking in RTL
  - **Mitigation**: Use flexbox with CSS logical properties (set in WP01)

---

## Work Package WP04: Component Refactoring - Content Sections (Priority: P1)

**Goal**: Refactor homepage content sections (Hero, Features, Download) to use translation functions.
**Independent Test**: All homepage sections render correctly in all 15 languages. Page content dynamically updates when `lang` prop changes.
**Prompt**: `/tasks/planned/WP04-component-refactoring-content-sections.md`

### Included Subtasks
- [ ] T019 [P] Refactor `website/src/components/Hero.astro` to use `t(lang, 'hero.*')` function
- [ ] T020 [P] Refactor `website/src/components/Features.astro` to use `t(lang, 'features.*')` function
- [ ] T021 [P] Refactor `website/src/components/Download.astro` to use `t(lang, 'download.*')` function

### Implementation Notes
- Same pattern as WP03: add `lang?: Locale` prop, import `t()`, replace strings
- Pay attention to CTA buttons - these are critical conversion points
- Preserve existing Astro slot usage and component structure
- Test placeholder replacement (e.g., `{count} downloads`)

### Parallel Opportunities
- **All three subtasks are fully parallel** - each targets a different component file

### Dependencies
- Depends on WP02 (translation JSON files must exist)

### Risks & Mitigations
- **Risk**: CTA button text too long in some languages (German, Indonesian)
  - **Mitigation**: Use CSS `white-space: nowrap` or `overflow: hidden` with ellipsis if needed
- **Risk**: RTL layout breaking hero image positioning
  - **Mitigation**: Use CSS logical properties for image placement

---

## Work Package WP05: Component Refactoring - Landing Pages (Priority: P1)

**Goal**: Refactor all 12 landing page components (LP01-LP12) to use translation functions.
**Independent Test**: All landing pages render correctly in all 15 languages. Each component accepts and uses `lang` prop.
**Prompt**: `/tasks/planned/WP05-component-refactoring-landing-pages.md`

### Included Subtasks
- [ ] T022 [P] Refactor LP01-LP12 landing page components to use `t(lang, 'landing.*')` function (12 components)

### Implementation Notes
- This is a single subtask covering 12 similar components for efficiency
- Each LP component follows identical refactoring pattern:
  1. Add `lang?: Locale` to Props interface
  2. Import `t` from `../i18n/config`
  3. Replace hardcoded strings with `t(lang, 'landing.lpXX.*')` calls
- Use a consistent naming convention: `landing.lp01.title`, `landing.lp01.description`, etc.
- If components share common copy, consider extracting to `common.json` instead

### Parallel Opportunities
- **Highly parallelizable**: Each of the 12 LP components can be refactored independently
- Consider splitting across multiple agents/sessions if available

### Dependencies
- Depends on WP02 (translation JSON files must exist)

### Risks & Mitigations
- **Risk**: Inconsistent refactoring across 12 components
  - **Mitigation**: Create a template/checklist for first component, then replicate
- **Risk**: Missing strings due to component complexity
  - **Mitigation**: Use grep to verify no hardcoded strings remain after refactoring

---

## Work Package WP06: Localized Routes & RTL Polish (Priority: P1)

**Goal**: Create locale-based URL routing for all pages (`/es/`, `/ja/`, `/he/`, etc.) and refine RTL layout for Hebrew, Arabic, Urdu.
**Independent Test**: Navigate to `/es/`, `/ja/`, `/he/`, `/ar/`, `/ur/` and verify all pages render correctly. RTL languages show mirrored navigation and right-aligned text.
**Prompt**: `/tasks/planned/WP06-localized-routes-rtl-polish.md`

### Included Subtasks
- [ ] T023 Create `website/src/pages/[lang]/index.astro` with `getStaticPaths()` generating routes for 14 non-English locales
- [ ] T024 Create `website/src/pages/[lang]/credits.astro` with same routing pattern
- [ ] T025 Create `website/src/pages/[lang]/compare/` directory structure with dynamic locale routes
- [ ] T026 Test and refine RTL layouts for Hebrew (`/he/`), Arabic (`/ar/`), Urdu (`/ur/`)
- [ ] T027 Add RTL-specific overrides for directional icons/arrows using `[dir="rtl"]` selectors

### Implementation Notes
1. Use `getStaticPaths()` to filter out English (`lang !== 'en'`) and generate locale routes
2. Import all refactored components with `lang` prop passed through
3. For RTL polish:
   - Test on actual RTL URLs (`/he/`, `/ar/`, `/ur/`)
   - Verify navigation flows right-to-left
   - Check text alignment (should be `text-align: start`, not hardcoded right)
   - Add `transform: scaleX(-1)` for directional arrows/chevrons
   - Verify Noto fonts load correctly

### Parallel Opportunities
- T023, T024, T025 (route creation) can be done in parallel
- T026, T027 (RTL polish) can be done in parallel after routes exist

### Dependencies
- Depends on WP03, WP04, WP05 (all components must be refactored first)
- Depends on WP01 (Layout.astro RTL support must exist)

### Risks & Mitigations
- **Risk**: RTL layout breaking on complex pages
  - **Mitigation**: Test incrementally on each route, fix issues before moving to next
- **Risk**: Font loading delays causing FOUC (Flash of Unstyled Content)
  - **Mitigation**: Use `font-display: swap` in Google Fonts URL

---

## Work Package WP07: GitHub Action Automation (Priority: P2)

**Goal**: Automated translation workflow that triggers on English content changes and creates PRs with GPT-4 translations for all 14 non-English locales.
**Independent Test**: Update `locales/en/navigation.json`, push to main, verify GitHub Action runs successfully and creates PR with translations within 1 hour. PR includes translations for all 14 locales with preserved placeholders.
**Prompt**: `/tasks/planned/WP07-github-action-automation.md`

### Included Subtasks
- [ ] T028 Create `.github/workflows/translate.yml` workflow file with triggers and permissions
- [ ] T029 Create `.github/scripts/translate.js` Node.js translation script
- [ ] T030 Configure OpenAI API integration in translation script (official Node.js SDK)
- [ ] T031 Add matrix strategy for 14 locales with `max-parallel: 3` for rate limiting
- [ ] T032 Add translation cache using `actions/cache@v5` with hash of English files as cache key
- [ ] T033 Add PR creation using `peter-evans/create-pull-request@v6` with labels `i18n` and `automated`
- [ ] T034 Test workflow with sample English content change (add test string to `locales/en/common.json`)

### Implementation Notes
1. Follow `contracts/github-action-workflow.yml` contract exactly
2. Use official OpenAI Node.js SDK (not direct fetch)
3. Translation script should:
   - Read changed English JSON files from git diff
   - For each changed file, for each target locale:
     - Load existing translations (if any)
     - Call GPT-4 with context-aware prompt (preserve placeholders, brand names, technical terms)
     - Merge new translations with existing translations
     - Write to `locales/{locale}/{section}.json`
   - Handle rate limiting with retries and exponential backoff
4. Test locally before pushing to GitHub

### Parallel Opportunities
- T028 (workflow YAML) and T029 (translation script) can be started in parallel
- T030, T031, T032, T033 must be sequential (building on T028/T029 foundation)

### Dependencies
- Depends on WP02 (English translation files must exist as source)
- Does NOT depend on WP03-WP06 (can be done in parallel with component refactoring)

### Risks & Mitigations
- **Risk**: OpenAI API quota exhaustion or rate limiting
  - **Mitigation**: Use `max-parallel: 3`, add retry logic with exponential backoff, cache translations
- **Risk**: GPT-4 not preserving placeholders like `{count}`
  - **Mitigation**: Explicit prompt instruction: "PRESERVE all placeholders like {variable} unchanged"
- **Risk**: Workflow failing silently
  - **Mitigation**: Add error notification to workflow (GitHub Actions built-in failure alerts)

---

## Work Package WP08: Language Switcher & Polish (Priority: P3)

**Goal**: Language selection UI component with localStorage persistence, performance optimization, and SEO enhancements.
**Independent Test**: Language switcher appears in navigation with all 15 languages. Selecting a language persists across sessions. Page load time is within 100ms of English baseline.
**Prompt**: `/tasks/planned/WP08-language-switcher-polish.md`

### Included Subtasks
- [ ] T035 Create `website/src/components/LanguageSwitcher.astro` component with dropdown of all 15 languages
- [ ] T036 Add client-side localStorage persistence logic (save/restore language preference)
- [ ] T037 Performance optimization: verify lazy loading of translations per route (Astro should handle automatically)
- [ ] T038 Add hreflang sitemap to `public/sitemap.xml` for SEO (all language versions linked)

### Implementation Notes
1. LanguageSwitcher component:
   - Display native language names (e.g., "Español", "日本語", "עברית")
   - Current language highlighted/selected
   - On change: redirect to corresponding locale URL (e.g., `/es/`, `/ja/`)
   - Use client-side script tag for interactivity
2. localStorage persistence:
   - Save as `caroPreferredLanguage`
   - On page load, check localStorage and redirect if different from current URL
   - Respect manual URL navigation (don't auto-redirect if user explicitly visits `/es/`)
3. Performance:
   - Verify Astro's code splitting works (use build output to check)
   - Measure page load time with Lighthouse for baseline comparison
4. SEO:
   - Generate sitemap with all locale URLs
   - Include `<url>` entries for each language version of each page

### Parallel Opportunities
- T035 (component) and T037 (performance) can be done in parallel
- T036 (localStorage) depends on T035 (component must exist)
- T038 (sitemap) can be done in parallel with all others

### Dependencies
- Depends on WP06 (localized routes must exist for switcher to navigate to)
- Depends on WP01 (translation config must exist)

### Risks & Mitigations
- **Risk**: Language switcher causing layout shifts (CLS)
  - **Mitigation**: Reserve space for dropdown, use CSS to prevent reflow
- **Risk**: localStorage conflicts with server-side rendering
  - **Mitigation**: Only read localStorage on client-side, use SSR for initial render
- **Risk**: Sitemap generation missing new pages
  - **Mitigation**: Use Astro's built-in sitemap integration, verify all URLs included

---

## Dependency & Execution Summary

**Critical Path**:
1. **WP01** (Infrastructure) → **WP02** (Translation Files) → **WP03/WP04/WP05** (Component Refactoring) → **WP06** (Routes & RTL) → **WP08** (Switcher & Polish)

**Parallel Path**:
2. **WP01** (Infrastructure) → **WP02** (Translation Files) → **WP07** (GitHub Action)

**Sequence**:
- **Start**: WP01 (no dependencies)
- **After WP01**: WP02 can start
- **After WP02**: WP03, WP04, WP05, WP07 can all start in parallel
- **After WP03/WP04/WP05**: WP06 can start
- **After WP06**: WP08 can start

**Parallelization Opportunities**:
- **Phase 1**: WP03, WP04, WP05 can run concurrently (different components)
- **Phase 2**: WP07 (GitHub Action) can run alongside WP03-WP06 (independent work)
- **Within WP02**: All 9 subtasks (T007-T015) are parallel
- **Within WP04**: All 3 subtasks (T019-T021) are parallel
- **Within WP05**: 12 landing page components are parallel

**MVP Scope (Minimum Release)**:
- **MVP = WP01 + WP02 + WP03 + WP04 + WP05 + WP06** (omit WP07, WP08 for initial release)
- This delivers User Story 1 (View Website in Native Language) and User Story 4 (RTL Support)
- Allows manual translation updates until WP07 automation is complete
- Defers language switcher polish (WP08) to post-MVP

---

## Subtask Index (Reference)

| Subtask ID | Summary | Work Package | Priority | Parallel? | Files |
|------------|---------|--------------|----------|-----------|-------|
| T001 | Astro i18n config | WP01 | P0 | No | `website/astro.config.mjs` |
| T002 | Translation utilities | WP01 | P0 | No | `website/src/i18n/config.ts` |
| T003 | Layout RTL support | WP01 | P0 | No | `website/src/layouts/Layout.astro` |
| T004 | Noto fonts CDN | WP01 | P0 | Yes | `website/src/layouts/Layout.astro` |
| T005 | RTL CSS logical properties | WP01 | P0 | Yes | `website/src/layouts/Layout.astro` |
| T006 | Locale directory structure | WP01 | P0 | No | `website/src/i18n/locales/*` |
| T007 | Extract navigation strings | WP02 | P0 | Yes | `locales/en/navigation.json` |
| T008 | Extract hero strings | WP02 | P0 | Yes | `locales/en/hero.json` |
| T009 | Extract features strings | WP02 | P0 | Yes | `locales/en/features.json` |
| T010 | Extract download strings | WP02 | P0 | Yes | `locales/en/download.json` |
| T011 | Extract footer strings | WP02 | P0 | Yes | `locales/en/navigation.json` |
| T012 | Extract common strings | WP02 | P0 | Yes | `locales/en/common.json` |
| T013 | Extract landing page strings | WP02 | P0 | Yes | `locales/en/landing.json` |
| T014 | Extract compare strings | WP02 | P0 | Yes | `locales/en/compare.json` |
| T015 | i18n index exports | WP02 | P0 | No | `website/src/i18n/index.ts` |
| T016 | Refactor Navigation.astro | WP03 | P1 | Yes | `website/src/components/Navigation.astro` |
| T017 | Refactor Footer.astro | WP03 | P1 | Yes | `website/src/components/Footer.astro` |
| T018 | Add hreflang meta tags | WP03 | P1 | Yes | `website/src/layouts/Layout.astro` |
| T019 | Refactor Hero.astro | WP04 | P1 | Yes | `website/src/components/Hero.astro` |
| T020 | Refactor Features.astro | WP04 | P1 | Yes | `website/src/components/Features.astro` |
| T021 | Refactor Download.astro | WP04 | P1 | Yes | `website/src/components/Download.astro` |
| T022 | Refactor LP01-LP12 | WP05 | P1 | Yes | `website/src/components/LP*.astro` (12 files) |
| T023 | Create [lang]/index.astro | WP06 | P1 | Yes | `website/src/pages/[lang]/index.astro` |
| T024 | Create [lang]/credits.astro | WP06 | P1 | Yes | `website/src/pages/[lang]/credits.astro` |
| T025 | Create [lang]/compare/ routes | WP06 | P1 | Yes | `website/src/pages/[lang]/compare/*` |
| T026 | Test RTL layouts | WP06 | P1 | Yes | (verification task) |
| T027 | RTL icon overrides | WP06 | P1 | Yes | `website/src/layouts/Layout.astro` |
| T028 | GitHub workflow YAML | WP07 | P2 | No | `.github/workflows/translate.yml` |
| T029 | Translation script | WP07 | P2 | Yes | `.github/scripts/translate.js` |
| T030 | OpenAI API integration | WP07 | P2 | No | `.github/scripts/translate.js` |
| T031 | Matrix strategy | WP07 | P2 | No | `.github/workflows/translate.yml` |
| T032 | Translation cache | WP07 | P2 | No | `.github/workflows/translate.yml` |
| T033 | PR creation | WP07 | P2 | No | `.github/workflows/translate.yml` |
| T034 | Test workflow | WP07 | P2 | No | (verification task) |
| T035 | LanguageSwitcher component | WP08 | P3 | Yes | `website/src/components/LanguageSwitcher.astro` |
| T036 | localStorage persistence | WP08 | P3 | No | `website/src/components/LanguageSwitcher.astro` |
| T037 | Performance optimization | WP08 | P3 | Yes | (verification task) |
| T038 | Hreflang sitemap | WP08 | P3 | Yes | `public/sitemap.xml` |

---

**Total**: 38 subtasks across 8 work packages
**MVP Scope**: WP01-WP06 (30 subtasks)
**Post-MVP**: WP07-WP08 (8 subtasks)

> All work packages have detailed prompt files in `/tasks/planned/WPxx-<slug>.md` generated by `/spec-kitty.tasks`.

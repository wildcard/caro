# Feature Specification: Website Internationalization with 15 Languages

**Feature Branch**: `004-add-internationalization-i18n`
**Created**: 2025-12-28
**Status**: Draft
**Input**: User description: "Add internationalization (i18n) support to website with 15 languages (Hebrew, Arabic, Urdu for RTL; Spanish, French, Portuguese, German, Ukrainian, Russian, Japanese, Korean, Hindi, Filipino, Indonesian for LTR) and automated GitHub Action for OpenAI GPT translations"

## User Scenarios & Testing

### User Story 1 - View Website in Native Language (Priority: P1)

A non-English speaker visits the Caro website and can immediately access all marketing content (homepage, landing pages, comparison pages, navigation) in their native language with accurate translations and proper text rendering.

**Why this priority**: Core value proposition of i18n - enables global audience to understand the product. Without this, 15 language markets are inaccessible.

**Independent Test**: Can be fully tested by navigating to `/[locale]/` URLs (e.g., `/es/`, `/ja/`, `/he/`) and verifying all UI strings, navigation, and content are translated correctly. Delivers immediate value by making the site accessible to target language speakers.

**Acceptance Scenarios**:

1. **Given** a Spanish speaker visits caro.sh, **When** they change language to Spanish via language switcher, **Then** all navigation links, hero text, features, and CTA buttons display in Spanish
2. **Given** a Japanese user navigates to `/ja/`, **When** they browse the homepage, **Then** all marketing copy including comparison pages renders in Japanese with proper character encoding
3. **Given** a Hebrew speaker accesses `/he/`, **When** the page loads, **Then** text displays right-to-left with proper RTL layout (navigation reversed, text aligned right)
4. **Given** any user on a localized page, **When** they click internal links, **Then** navigation maintains the selected language (e.g., clicking Features from `/fr/` goes to `/fr/#features`)

---

### User Story 2 - Automated Translation Updates (Priority: P2)

When English marketing content is updated (new features added, copy improvements, blog posts), translations are automatically generated for all 15 languages within 24 hours without manual intervention, maintaining translation consistency across the site.

**Why this priority**: Enables sustainable i18n at scale - without automation, translations become stale and maintenance becomes prohibitive with 15 languages.

**Independent Test**: Can be tested by updating an English JSON translation file in `website/src/i18n/locales/en/`, pushing to main, and verifying a PR is created with translations for all 14 non-English locales within 24 hours. Delivers value by eliminating manual translation workflows.

**Acceptance Scenarios**:

1. **Given** a developer updates `locales/en/navigation.json` with a new menu item, **When** changes are pushed to main branch, **Then** GitHub Action triggers and creates a PR with the new menu item translated to all 15 languages within 1 hour
2. **Given** the marketing team updates hero copy in `locales/en/hero.json`, **When** the change is committed, **Then** automated translation PR includes updated hero translations for es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id
3. **Given** a translation PR is created, **When** a reviewer spot-checks 2-3 strings per language, **Then** translations are contextually accurate and preserve placeholders like `{count}` and `{name}`
4. **Given** translation PR passes spot-check, **When** PR is merged to main, **Then** all localized pages reflect new translations on next deployment

---

### User Story 3 - Language Preference Persistence (Priority: P3)

Users' language selection is remembered across sessions via localStorage, so returning visitors automatically see content in their preferred language without re-selecting each visit.

**Why this priority**: Improves UX for returning users - eliminates friction of re-selecting language. Lower priority than P1/P2 as it's a convenience feature, not core functionality.

**Independent Test**: Can be tested by selecting a language (e.g., German), leaving the site, returning later, and verifying German is still active. Delivers value through saved user preferences.

**Acceptance Scenarios**:

1. **Given** a user selects French from the language switcher, **When** they close the browser and return to caro.sh 3 days later, **Then** the site automatically loads in French
2. **Given** a user's browser language is Spanish (es) but they select Japanese manually, **When** they visit any page, **Then** Japanese takes precedence over browser language detection
3. **Given** a user clears browser data, **When** they return to the site, **Then** language defaults to English (or browser language detection if implemented)

---

### User Story 4 - RTL Language Support (Priority: P1)

Users accessing RTL languages (Hebrew, Arabic, Urdu) see proper right-to-left text rendering with mirrored layouts - navigation flows right-to-left, text aligns right, and all UI elements respect RTL conventions.

**Why this priority**: Critical for 3 target markets (Israel, Arabic-speaking countries, Urdu speakers). Without RTL support, these languages are unusable despite being translated.

**Independent Test**: Can be tested by accessing `/he/`, `/ar/`, or `/ur/` and verifying text flows right-to-left, navigation is reversed, and layout mirrors correctly. Delivers value by making the site usable for RTL language speakers.

**Acceptance Scenarios**:

1. **Given** a Hebrew speaker visits `/he/`, **When** the page loads, **Then** the `<html>` element has `dir="rtl"` attribute and all text aligns right
2. **Given** an Arabic user browses `/ar/`, **When** they view navigation, **Then** menu items flow right-to-left with logo on right side
3. **Given** Urdu content on `/ur/`, **When** paragraphs render, **Then** Arabic script displays correctly with proper font stack (Noto Nastaliq Urdu) and letter-spacing is zero
4. **Given** RTL page with mixed content, **When** English words appear in RTL text (e.g., "Caro"), **Then** bidirectional text handling prevents layout breaks

---

### Edge Cases

- **What happens when a translation is missing for a specific string?** System falls back to English translation for that string only, preventing broken UI
- **How does system handle new languages added later?** Translation workflow supports adding new locales by extending config arrays and running GitHub Action
- **What happens if OpenAI API is unavailable during translation?** GitHub Action retries 3 times with exponential backoff, then fails gracefully with error notification
- **How are placeholders preserved in translations?** Translation script detects patterns like `{variable}` and instructs GPT-4 to preserve them unchanged
- **What happens if a language switcher is used on a page that doesn't exist in that locale?** User is redirected to homepage in selected language (e.g., `/blog/post-1` → `/es/` if post-1 has no Spanish version)
- **How does SEO handle duplicate content across languages?** `hreflang` meta tags in `<head>` indicate alternate language versions to search engines
- **What happens if RTL CSS conflicts with holiday themes (Christmas/Hanukkah)?** RTL logical properties work with existing theme system without conflicts

## Requirements

### Functional Requirements

- **FR-001**: System MUST support 15 languages: English (en, default), Spanish (es), French (fr), Portuguese (pt), German (de), Hebrew (he), Arabic (ar), Ukrainian (uk), Russian (ru), Japanese (ja), Korean (ko), Hindi (hi), Urdu (ur), Filipino (fil), Indonesian (id)
- **FR-002**: System MUST implement RTL (right-to-left) layout and text direction for Hebrew, Arabic, and Urdu using CSS logical properties
- **FR-003**: System MUST provide URL-based locale routing with English as default (`/about`) and locale prefixes for other languages (`/es/about`, `/ja/about`)
- **FR-004**: System MUST store translations in JSON format organized by locale and component/section (`website/src/i18n/locales/{locale}/{section}.json`)
- **FR-005**: System MUST include a language switcher component in navigation showing all 15 available languages with native names (e.g., "Español", "日本語", "עברית")
- **FR-006**: System MUST persist user language selection in browser localStorage
- **FR-007**: System MUST provide translation utility function `t(locale, key)` that retrieves translated strings with automatic English fallback
- **FR-008**: System MUST trigger GitHub Action when English translation files in `website/src/i18n/locales/en/**` are modified
- **FR-009**: GitHub Action MUST call OpenAI GPT-4 API to generate translations for all 14 non-English locales
- **FR-010**: GitHub Action MUST create a pull request with generated translations labeled `i18n` and `automated`
- **FR-011**: Translation prompts MUST instruct GPT-4 to preserve placeholders (e.g., `{count}`, `{name}`), brand names ("Caro"), and technical terms (POSIX, shell, CLI)
- **FR-012**: System MUST use CSS logical properties for all spacing and layout (e.g., `margin-inline-start` instead of `margin-left`)
- **FR-013**: System MUST apply RTL-specific font stacks: Noto Sans Hebrew for Hebrew, Noto Sans Arabic for Arabic (with `letter-spacing: 0`), Noto Nastaliq Urdu for Urdu
- **FR-014**: System MUST include `hreflang` meta tags in page `<head>` linking to all language versions for SEO
- **FR-015**: System MUST maintain existing holiday theme system (Christmas/Hanukkah) compatibility with RTL languages

### Key Entities

- **Locale**: Represents a language-region combination (e.g., "es" for Spanish, "he" for Hebrew). Contains: code (2-letter ISO 639-1), native name, text direction (ltr/rtl), font family preferences
- **Translation**: Key-value pairs mapping English source strings to target language strings. Contains: locale, section (navigation, hero, features, etc.), key path, translated value
- **Translation Job**: GitHub Action workflow run that processes English content changes. Contains: trigger commit, changed files list, target locales, GPT-4 API calls, output PR reference

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can access the entire marketing website (homepage, landing pages, comparison pages, navigation, footer) in any of 15 supported languages
- **SC-002**: RTL languages (Hebrew, Arabic, Urdu) render with correct text direction and mirrored layout without broken UI elements
- **SC-003**: When English translation files are updated, automated translations for all 14 non-English locales are available in a PR within 24 hours
- **SC-004**: 95% of automated translations pass spot-check validation (2-3 strings reviewed per language per PR)
- **SC-005**: Language selection persists across browser sessions for returning users
- **SC-006**: All placeholder variables (e.g., `{count}`, `{name}`) are preserved unchanged in translated strings
- **SC-007**: Page load time increases by no more than 100ms when loading localized versions compared to English version
- **SC-008**: Search engines can discover and index all language versions via `hreflang` tags in sitemap

## Assumptions

- **Assumption 1**: OpenAI GPT-4 API will maintain current quality standards for translation accuracy (spot-check target: 95%+)
- **Assumption 2**: Project has budget for ~$6 per full translation run (14 languages × ~200 strings × GPT-4 pricing)
- **Assumption 3**: `OPENAI_API_KEY` secret will be configured in GitHub repository settings before GitHub Action is enabled
- **Assumption 4**: Google Noto fonts (Sans Hebrew, Sans Arabic, Nastaliq Urdu) will be loaded via CDN (Google Fonts) for RTL language support
- **Assumption 5**: Astro framework's built-in i18n routing (available since Astro 4.0) provides sufficient functionality for URL-based locale routing
- **Assumption 6**: Marketing content structure remains primarily static (not CMS-driven), allowing JSON translation files to work effectively
- **Assumption 7**: Language switcher will be positioned in navigation component, consistent with existing UI patterns
- **Assumption 8**: Translation quality issues discovered after merge can be fixed incrementally via subsequent PRs (not blocking deployment)

## Dependencies

- **External**: OpenAI GPT-4 API availability and access token
- **External**: Google Fonts CDN for Noto font families (Hebrew, Arabic, Urdu)
- **Internal**: Astro 4.0+ framework with built-in i18n support
- **Internal**: GitHub Actions with workflow permissions to create pull requests
- **Internal**: Existing website deployment pipeline to handle localized routes

## Out of Scope

- Blog post content translation (markdown files in `website/src/pages/blog/`)
- Documentation translation (`docs/**/*.md` files)
- CLI help text translation (Rust strings in `src/`)
- User-generated content translation
- Real-time translation or dynamic translation selection
- Translation memory or glossary management system
- Professional human translator review (only automated GPT-4 + spot-checks)
- A/B testing different translations
- Analytics tracking per-language traffic

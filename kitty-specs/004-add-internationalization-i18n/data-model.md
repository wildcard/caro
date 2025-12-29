# Data Model: Website Internationalization

**Feature**: 004-add-internationalization-i18n
**Date**: 2025-12-28
**Status**: Phase 0 Research Output

## Overview

This document defines the core entities and their relationships for the i18n feature. These entities represent the conceptual data model - implementation details (TypeScript types, JSON schemas) will be determined in Phase 1.

## Entity Definitions

### 1. Locale

Represents a language-region combination supported by the website.

**Attributes**:
- **code** (string, primary key): 2-letter ISO 639-1 language code (e.g., "es", "he", "ja")
- **nativeName** (string): Language name in its native script (e.g., "Español", "עברית", "日本語")
- **englishName** (string): Language name in English (e.g., "Spanish", "Hebrew", "Japanese")
- **direction** (enum): Text direction - "ltr" or "rtl"
- **fontFamily** (string, optional): Preferred font stack for this locale (e.g., "Noto Sans Hebrew")
- **isDefault** (boolean): Whether this is the default locale (true only for "en")

**Validation Rules**:
- code must be unique
- code must be exactly 2 characters (ISO 639-1)
- direction must be either "ltr" or "rtl"
- exactly one locale must have isDefault=true

**Example Instances**:
```
Locale { code: "en", nativeName: "English", direction: "ltr", isDefault: true }
Locale { code: "he", nativeName: "עברית", direction: "rtl", fontFamily: "Noto Sans Hebrew" }
Locale { code: "ar", nativeName: "العربية", direction: "rtl", fontFamily: "Noto Sans Arabic" }
```

**Relationships**:
- One Locale has many Translations (1:N)
- One Locale has many TranslationJobs (1:N)

---

### 2. Translation

Represents a key-value pair mapping an English source string to a target language string.

**Attributes**:
- **locale** (string, foreign key): References Locale.code (e.g., "es", "fr")
- **section** (string): Component or page section (e.g., "navigation", "hero", "features")
- **key** (string): Dot-notation path to the translation (e.g., "navigation.links.features", "hero.tagline")
- **value** (string): Translated text in the target language
- **sourceValue** (string, optional): Original English text for reference
- **lastUpdated** (timestamp): When this translation was last modified

**Validation Rules**:
- (locale, section, key) must be unique (composite key)
- locale must reference an existing Locale.code
- section must match one of the predefined translation file names
- value must not be empty
- value must preserve any placeholders from sourceValue (e.g., `{count}`, `{name}`)

**Example Instances**:
```
Translation {
  locale: "es",
  section: "navigation",
  key: "links.features",
  value: "Características",
  sourceValue: "Features",
  lastUpdated: "2025-12-28T10:30:00Z"
}

Translation {
  locale: "he",
  section: "hero",
  key: "tagline",
  value: "חבר הנאמן שלך לשורת הפקודה",
  sourceValue: "Your loyal shell companion",
  lastUpdated: "2025-12-28T10:31:00Z"
}
```

**Relationships**:
- Many Translations belong to one Locale (N:1)
- Many Translations belong to one TranslationJob (N:1)

**State Transitions**:
1. **New**: Translation created by GitHub Action
2. **Reviewed**: Translation spot-checked and approved
3. **Updated**: Translation modified due to source change
4. **Deprecated**: Translation marked for removal (source deleted)

---

### 3. TranslationJob

Represents a GitHub Action workflow run that processes English content changes and generates translations.

**Attributes**:
- **jobId** (string, primary key): GitHub Actions run ID
- **triggerCommit** (string): Git commit SHA that triggered the job
- **changedFiles** (array of strings): List of English translation files modified (e.g., ["locales/en/navigation.json"])
- **targetLocales** (array of strings): List of locales to translate (e.g., ["es", "fr", "de", ...])
- **status** (enum): "pending", "in-progress", "completed", "failed"
- **startedAt** (timestamp): When the job started
- **completedAt** (timestamp, optional): When the job finished
- **prNumber** (number, optional): Pull request number created by this job
- **totalTokens** (number, optional): OpenAI API tokens consumed
- **estimatedCost** (number, optional): USD cost for this translation run

**Validation Rules**:
- jobId must be unique
- status must be one of: "pending", "in-progress", "completed", "failed"
- targetLocales must contain only valid Locale codes
- completedAt must be after startedAt (if present)

**Example Instances**:
```
TranslationJob {
  jobId: "gh-run-12345678",
  triggerCommit: "a1b2c3d4",
  changedFiles: ["locales/en/hero.json", "locales/en/features.json"],
  targetLocales: ["es", "fr", "pt", "de", "he", "ar", ...],
  status: "completed",
  startedAt: "2025-12-28T10:00:00Z",
  completedAt: "2025-12-28T10:15:00Z",
  prNumber: 234,
  totalTokens: 25000,
  estimatedCost: 5.50
}
```

**Relationships**:
- One TranslationJob generates many Translations (1:N)
- One TranslationJob targets many Locales (1:N)

**State Transitions**:
1. **pending**: Job queued, waiting to start
2. **in-progress**: OpenAI API calls in progress
3. **completed**: All translations generated, PR created
4. **failed**: Error occurred (API failure, rate limit, etc.)

---

## Entity Relationships Diagram

```
┌─────────────┐
│   Locale    │
│─────────────│         1:N
│ code (PK)   │◄─────────────┐
│ nativeName  │              │
│ direction   │              │
│ fontFamily  │              │
└─────────────┘              │
                             │
                             │
┌──────────────────┐         │
│  TranslationJob  │         │
│──────────────────│     1:N │
│ jobId (PK)       │◄────────┤
│ triggerCommit    │         │
│ changedFiles     │         │
│ targetLocales    │         │
│ status           │         │
│ prNumber         │         │
└──────────────────┘         │
        │                    │
        │ 1:N                │
        │                    │
        ▼                    │
┌─────────────────────┐      │
│    Translation      │      │
│─────────────────────│      │
│ locale (FK)         │──────┘
│ section             │
│ key                 │
│ value               │
│ lastUpdated         │
└─────────────────────┘
```

## Derived Data

These are not stored entities but calculated/aggregated views:

### TranslationCoverage
- **locale** (string): Locale code
- **section** (string): Translation section
- **totalKeys** (number): Total keys in English source
- **translatedKeys** (number): Keys with translations
- **coveragePercent** (number): (translatedKeys / totalKeys) × 100
- **missingKeys** (array): Keys without translations (fallback to English)

### TranslationQualityMetrics
- **jobId** (string): TranslationJob reference
- **locale** (string): Locale code
- **spotCheckSample** (number): Number of strings reviewed
- **passedReview** (number): Number of approved strings
- **qualityScore** (number): (passedReview / spotCheckSample) × 100
- **issues** (array): List of quality problems found

## Implementation Notes

**Storage Format**: JSON files on disk
- Path pattern: `website/src/i18n/locales/{locale}/{section}.json`
- No database required - version control via git

**Access Patterns**:
- Read: `t(locale, key)` function loads JSON file and navigates key path
- Write: GitHub Action writes entire JSON file atomically
- Update: PR merge updates JSON files in main branch

**Performance Considerations**:
- JSON files lazy-loaded per route (not all files loaded upfront)
- Astro's build-time optimization pre-renders localized pages
- No runtime API calls for translations (all static)

## Open Questions

None - all entity definitions validated during Phase 0 research.

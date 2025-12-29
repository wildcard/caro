---
work_package_id: "WP02"
subtasks:
  - "T007"
  - "T008"
  - "T009"
  - "T010"
  - "T011"
  - "T012"
  - "T013"
  - "T014"
  - "T015"
title: "Translation File Extraction"
phase: "Phase 2 - Translation Files"
lane: "doing"
assignee: "Claude Sonnet 4.5"
agent: "claude"
shell_pid: "71083"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
  - timestamp: "2025-12-29T02:15:00Z"
    lane: "doing"
    agent: "claude"
    shell_pid: "71083"
    action: "Started translation file extraction - extracting English strings to JSON"
---

# Work Package Prompt: WP02 – Translation File Extraction

## Objectives & Success Criteria

**Goal**: Extract all English strings from existing components to JSON translation files, organized by section (navigation, hero, features, download, common, landing, compare).

**Success Criteria**:
- All 8 JSON files exist in `locales/en/` with complete string coverage
- No hardcoded strings remain in targeted components (verified by grep)
- JSON is valid and parseable
- Keys use hierarchical dot-notation (`section.subsection.key`)
- Placeholders preserved (`{count}`, `{name}`)

---

## Subtasks & Detailed Guidance

### T007-T014 – Extract Component Strings (PARALLEL)

**Pattern for each file**:
1. Open target component (e.g., `website/src/components/Navigation.astro`)
2. Find all hardcoded strings
3. Create corresponding JSON file: `website/src/i18n/locales/en/{section}.json`
4. Use hierarchical structure:
   ```json
   {
     "navigation": {
       "brand": "Caro",
       "links": {
         "features": "Features",
         "download": "Get Started",
         "compare": "Compare"
       }
     }
   }
   ```
5. Preserve placeholders: `"{count} downloads"` not `"10 downloads"`

**Files to create**:
- T007: `locales/en/navigation.json` (from Navigation.astro, Footer.astro)
- T008: `locales/en/hero.json` (from Hero.astro)
- T009: `locales/en/features.json` (from Features.astro)
- T010: `locales/en/download.json` (from Download.astro)
- T011: Include footer in `navigation.json` or create separate `footer.json`
- T012: `locales/en/common.json` (buttons, labels, status messages)
- T013: `locales/en/landing.json` (LP01-LP12 components)
- T014: `locales/en/compare.json` (comparison pages)

**Parallel?**: YES - all 8 files are independent

### T015 – Create i18n Index

**Purpose**: Export all translation objects for import by components.

**Steps**:
1. Create `website/src/i18n/index.ts`:
   ```typescript
   import en from './locales/en/navigation.json';
   import heroEn from './locales/en/hero.json';
   // ... import all sections

   export { en, heroEn, ... };
   ```
2. Update `config.ts` `t()` function to actually load from JSON (replace placeholder)

**Depends on**: T007-T014 must complete first

---

## Test Strategy

**Validation**:
```bash
# JSON syntax check
for file in website/src/i18n/locales/en/*.json; do
  jq . "$file" > /dev/null && echo "$file OK" || echo "$file INVALID"
done

# Verify no hardcoded strings (example for Navigation)
grep -E '"(Features|Download|Compare)"' website/src/components/Navigation.astro
# Should return NO matches after refactoring
```

---

## Definition of Done

- [ ] All 8 JSON files exist and are valid JSON
- [ ] All hardcoded strings extracted to appropriate files
- [ ] Keys use dot-notation (`navigation.links.features`)
- [ ] Placeholders preserved unchanged
- [ ] `index.ts` exports all translation objects
- [ ] `config.ts` `t()` function loads from JSON (not placeholder)

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created

---
work_package_id: WP03
title: Component Refactoring - Navigation & Core
dependencies: []
subtasks:
- T016
- T017
- T018
phase: Phase 3 - Component Refactoring
history:
- timestamp: '2025-12-29T00:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
- timestamp: '2025-12-29T02:40:00Z'
  lane: doing
  agent: claude
  shell_pid: '78552'
  action: Started component refactoring - Navigation, Footer, hreflang tags
- timestamp: '2025-12-29T02:50:00Z'
  lane: for_review
  agent: claude
  shell_pid: '78552'
  action: Completed all 3 subtasks - Navigation, Footer refactored, hreflang tags added (commit 1933c75)
- timestamp: '2025-12-29T02:55:00Z'
  lane: done
  agent: claude
  shell_pid: '80406'
  action: Approved by user - proceeding to WP04
authoritative_surface: src/components/Navigation.astro/
execution_mode: code_change
mission_id: 01KQ6BCQV2SS4PHX6NGR5YG3Z0
owned_files:
- src/components/Navigation.astro
wp_code: WP03
---

# Work Package Prompt: WP03 – Component Refactoring - Navigation & Core

## Objectives

Refactor Navigation, Footer, and add hreflang meta tags to use translation functions.

**Success**: Navigation and Footer render in all 15 languages. Hreflang tags present.

---

## Subtasks

### T016 – Refactor Navigation.astro

1. Import: `import { t, type Locale } from '../i18n/config';`
2. Props: `interface Props { lang?: Locale; } const { lang = 'en' } = Astro.props;`
3. Replace strings: `<a>{t(lang, 'navigation.links.features')}</a>`
4. Preserve holiday theme logic

### T017 – Refactor Footer.astro

Same pattern as T016 for footer links.

### T018 – Add Hreflang Meta Tags

In `Layout.astro` `<head>`:
```astro
{getAllLocales().map(locale => (
  <link rel="alternate" hreflang={locale.code} href={`/${locale.code === 'en' ? '' : locale.code + '/'}`} />
))}
```

**Parallel**: T016/T017/T018 can all be done concurrently.

---

## Test

```bash
# Verify no hardcoded strings
grep -E '"Features|Download"' website/src/components/Navigation.astro
# Should return NO matches

# Verify hreflang
npm run dev
curl -s http://localhost:4321/ | grep 'hreflang'
# Should show 15 <link> tags
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created

---
work_package_id: WP04
title: Component Refactoring - Content Sections
dependencies: []
subtasks:
- T019
- T020
- T021
phase: Phase 3 - Component Refactoring
history:
- timestamp: '2025-12-29T00:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
- timestamp: '2025-12-29T03:00:00Z'
  lane: doing
  agent: claude
  shell_pid: '80406'
  action: Started component refactoring - Hero, Features, Download
authoritative_surface: src/
execution_mode: code_change
mission_id: 01KQ6BCQV2SS4PHX6NGR5YG3Z0
owned_files:
- src/**
wp_code: WP04
---

# Work Package Prompt: WP04 – Component Refactoring - Content Sections

## Objectives

Refactor Hero, Features, Download components to use translation functions.

**Success**: Homepage sections render in all 15 languages.

---

## Subtasks

### T019 – Refactor Hero.astro
### T020 – Refactor Features.astro
### T021 – Refactor Download.astro

**Pattern for each**:
1. Add `lang?: Locale` prop
2. Import `t` function
3. Replace strings: `t(lang, '{section}.key')`
4. Test placeholder replacement if present

**Parallel**: All 3 can be done concurrently.

---

## Test

```bash
# Build and verify
npm run build
# Should succeed

# Visual test (after WP06 routes exist)
npm run dev
# Visit http://localhost:4321/ - should show English
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created
- 2025-12-29T10:31:21Z – claude – shell_pid=83371 – lane=for_review – Completed all 3 subtasks: Hero, Features, Download refactored (commit 446f85c)
- 2025-12-29T10:31:32Z – claude – shell_pid=83420 – lane=done – Approved by user - all 3 content sections support i18n

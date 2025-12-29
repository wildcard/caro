---
work_package_id: "WP04"
subtasks:
  - "T019"
  - "T020"
  - "T021"
title: "Component Refactoring - Content Sections"
phase: "Phase 3 - Component Refactoring"
lane: "doing"
assignee: "Claude Sonnet 4.5"
agent: "claude"
shell_pid: "80406"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
  - timestamp: "2025-12-29T03:00:00Z"
    lane: "doing"
    agent: "claude"
    shell_pid: "80406"
    action: "Started component refactoring - Hero, Features, Download"
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

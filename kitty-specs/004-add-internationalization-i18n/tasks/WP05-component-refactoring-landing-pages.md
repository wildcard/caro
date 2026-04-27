---
work_package_id: "WP05"
subtasks:
  - "T022"
title: "Component Refactoring - Landing Pages"
phase: "Phase 3 - Component Refactoring"
lane: "done"
assignee: ""
agent: "claude"
shell_pid: "84844"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP05 – Component Refactoring - Landing Pages

## Objectives

Refactor all 12 LP components (LP01-LP12) to use translation functions.

**Success**: All landing pages render in all 15 languages.

---

## Subtask T022 – Refactor LP01-LP12

**Highly Parallel**: Each component is independent.

**Pattern (apply to all 12)**:
1. Add `lang?: Locale` prop
2. Import `t` from `../i18n/config`
3. Replace strings: `t(lang, 'landing.lpXX.key')`
4. Use consistent naming: `landing.lp01.title`, `landing.lp01.description`

**Files**: `website/src/components/LP*.astro` (12 files)

**Tip**: Create template for LP01, then replicate pattern to LP02-LP12.

---

## Test

```bash
# Verify refactoring complete
grep -E '"[A-Z]' website/src/components/LP*.astro
# Should show minimal matches (only preserved brand names)
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created
- 2025-12-29T10:32:16Z – claude – shell_pid=83593 – lane=doing – Starting LP component refactoring
- 2025-12-29T10:37:26Z – claude – shell_pid=84745 – lane=for_review – Completed all 12 LP components with i18n support (commit b27be7d)
- 2025-12-29T10:37:40Z – claude – shell_pid=84844 – lane=done – Approved - all LP components support i18n

---
work_package_id: "WP06"
subtasks:
  - "T023"
  - "T024"
  - "T025"
  - "T026"
  - "T027"
title: "Localized Routes & RTL Polish"
phase: "Phase 4 - Localized Routes"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "93043"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP06 – Localized Routes & RTL Polish

## Objectives

Create locale-based routing (`/es/`, `/ja/`, `/he/`) and refine RTL layouts.

**Success**: All 14 non-English locales accessible via URLs. RTL languages render correctly.

---

## Subtasks

### T023 – Create [lang]/index.astro

```astro
---
import Layout from '../../layouts/Layout.astro';
import { languages, type Locale } from '../../i18n/config';
import Hero from '../../components/Hero.astro';
// ... import all components

export function getStaticPaths() {
  return Object.keys(languages)
    .filter(lang => lang !== 'en')  // English stays at /
    .map(lang => ({ params: { lang } }));
}

const { lang } = Astro.params as { lang: Locale };
---

<Layout title="Caro" lang={lang}>
  <Hero lang={lang} />
  <Features lang={lang} />
  <!-- Pass lang to all components -->
</Layout>
```

### T024 – Create [lang]/credits.astro

Same pattern as T023.

### T025 – Create [lang]/compare/ Routes

Same pattern for comparison pages.

### T026 – Test RTL Layouts

**Manual testing for `/he/`, `/ar/`, `/ur/`**:
- Text flows right-to-left
- Navigation reversed (logo on right)
- Text aligned right
- Noto fonts load

### T027 – RTL Icon Overrides

Add to `Layout.astro`:
```css
[dir="rtl"] .icon-arrow {
  transform: scaleX(-1);  /* Flip horizontal */
}
```

---

## Test

```bash
npm run build
# Should generate routes for 14 locales

npm run dev
# Visit http://localhost:4321/es/ → Spanish
# Visit http://localhost:4321/he/ → Hebrew (RTL)
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created
- 2025-12-29T10:45:54Z – claude – shell_pid=86061 – lane=doing – Starting localized routes and RTL polish
- 2025-12-29T10:53:28Z – claude – shell_pid=93043 – lane=for_review – Completed all 5 subtasks - localized routes and RTL support (commit 9f251fb)

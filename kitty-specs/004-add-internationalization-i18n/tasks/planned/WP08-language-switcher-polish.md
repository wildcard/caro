---
work_package_id: "WP08"
subtasks:
  - "T035"
  - "T036"
  - "T037"
  - "T038"
title: "Language Switcher & Polish"
phase: "Phase 6 - Polish"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP08 – Language Switcher & Polish

## Objectives

Language selection UI, localStorage persistence, performance, SEO.

**Success**: Language switcher works, preferences persist, hreflang sitemap exists.

---

## Subtasks

### T035 – Create LanguageSwitcher.astro

```astro
---
import { getAllLocales, type Locale } from '../i18n/config';
const currentLang = Astro.params.lang || 'en';
const allLocales = getAllLocales();
---

<select id="lang-switcher">
  {allLocales.map(locale => (
    <option value={locale.code} selected={locale.code === currentLang}>
      {locale.nativeName}
    </option>
  ))}
</select>

<script>
  document.getElementById('lang-switcher')?.addEventListener('change', (e) => {
    const newLang = (e.target as HTMLSelectElement).value;
    const path = window.location.pathname.replace(/^\/[a-z]{2,3}/, '');
    window.location.href = newLang === 'en' ? path : `/${newLang}${path}`;
  });
</script>
```

### T036 – Add localStorage Persistence

```javascript
// On page load
const saved = localStorage.getItem('caroPreferredLanguage');
if (saved && saved !== currentLang) {
  // Redirect to saved language
}

// On change
localStorage.setItem('caroPreferredLanguage', newLang);
```

### T037 – Performance Verification

- Check build output for code splitting
- Measure page load with Lighthouse
- Target: <100ms overhead vs English baseline

### T038 – Hreflang Sitemap

Add to `public/sitemap.xml`:
```xml
<url>
  <loc>https://caro.sh/</loc>
  <xhtml:link rel="alternate" hreflang="es" href="https://caro.sh/es/"/>
  <!-- ... repeat for all locales -->
</url>
```

---

## Test

```bash
# Visual test
npm run dev
# Verify switcher appears, changes language on select

# Performance
npm run build
npx lighthouse http://localhost:4321/ --only-categories=performance
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created

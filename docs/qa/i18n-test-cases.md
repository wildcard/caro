# i18n Manual QA Test Cases

## Overview
Standard Test Document (STD) for verifying internationalization functionality on the Caro website.

**Supported Locales:** en, es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id
**RTL Locales:** he (Hebrew), ar (Arabic), ur (Urdu)

---

## Test Environment Setup

```bash
# Local testing
cd website && npm run dev
# Open http://localhost:4321

# Staging (after deploy)
# https://caro-preview-*.vercel.app
```

---

## TC-001: Locale Route Navigation

**Objective:** Verify localized routes work correctly

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Navigate to `/es/` | Spanish homepage loads |
| 2 | Navigate to `/fr/` | French homepage loads |
| 3 | Navigate to `/he/` | Hebrew homepage loads with RTL layout |
| 4 | Navigate to `/fil/` | Filipino homepage loads (3-char locale) |
| 5 | Navigate to `/invalid/` | 404 or redirect to English |

**Pass Criteria:** All locale routes return 200, content displays

---

## TC-002: Translation Loading

**Objective:** Verify translations display correctly (not English fallback)

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/es/` | See "Copiar" button (not "Copy") |
| 2 | Go to `/fr/` | See "Copier" button (not "Copy") |
| 3 | Go to `/de/` | See "Kopieren" button (not "Copy") |
| 4 | Go to `/ja/` | See "コピー" button (not "Copy") |
| 5 | Go to `/he/` | See "העתק" button (not "Copy") |

**Pass Criteria:** Common UI strings display in target language

---

## TC-003: RTL Layout (Hebrew, Arabic, Urdu)

**Objective:** Verify RTL languages display correctly

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/he/` | Page has `dir="rtl"` attribute |
| 2 | Check navigation | Menu items flow right-to-left |
| 3 | Check text alignment | Body text right-aligned |
| 4 | Check Noto Sans Hebrew font | Hebrew text renders with Noto font |
| 5 | Repeat for `/ar/` | Arabic with Noto Sans Arabic |
| 6 | Repeat for `/ur/` | Urdu with Noto Nastaliq Urdu |

**Pass Criteria:** RTL layout correct, appropriate fonts loaded

---

## TC-004: English Fallback

**Objective:** Verify untranslated content falls back to English

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/es/` | Hero section shows English (not translated yet) |
| 2 | Check FAQ section | English content (common.json only translated) |
| 3 | Check footer | English content |
| 4 | No broken keys visible | No `landing.hero.title` raw keys shown |

**Pass Criteria:** Untranslated sections show English, no raw keys

---

## TC-005: Hreflang Tags (SEO)

**Objective:** Verify SEO language tags present

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | View page source on `/` | Find `<link rel="alternate" hreflang="es" href="...">` |
| 2 | Count hreflang tags | 15 tags (one per locale) |
| 3 | Verify href values | Each points to correct locale path |

**Pass Criteria:** All 15 hreflang tags present with correct URLs

---

## TC-006: Dark Mode + Locale

**Objective:** Verify dark mode works across locales

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/es/` | Default theme loads |
| 2 | Toggle dark mode | Dark theme applies |
| 3 | Refresh page | Dark mode persists |
| 4 | Navigate to `/fr/` | Dark mode still active |

**Pass Criteria:** Theme preference persists across locale navigation

---

## TC-007: Build Verification

**Objective:** Verify website builds without errors

```bash
cd website && npm run build
```

| Check | Expected |
|-------|----------|
| Exit code | 0 |
| Pages built | 47 |
| No TypeScript errors | Clean build |
| Sitemap generated | sitemap-index.xml created |

**Pass Criteria:** Build completes successfully with 47 pages

---

## TC-008: Compare Page Localized Routes

**Objective:** Verify compare page works in all locales

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/compare/` | English compare page |
| 2 | Go to `/es/compare/` | Compare page with Spanish layout |
| 3 | Go to `/he/compare/` | Compare page with RTL layout |

**Pass Criteria:** Compare pages accessible in all locales

---

## TC-009: Credits Page Localized Routes

**Objective:** Verify credits page works in all locales

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Go to `/credits` | English credits page |
| 2 | Go to `/es/credits` | Credits page loads |
| 3 | Go to `/ar/credits` | Credits with RTL layout |

**Pass Criteria:** Credits pages accessible in all locales

---

## Smoke Test Checklist

Quick verification for CI/CD or release:

- [ ] `npm run build` succeeds (47 pages)
- [ ] `/es/` loads with Spanish "Copiar" button
- [ ] `/he/` loads with RTL direction
- [ ] `/fil/` loads (3-char locale)
- [ ] Dark mode toggle works on `/fr/`
- [ ] No console errors on any locale page

---

## Known Limitations

1. **Partial translations:** Only `common.json` translated; hero, features, FAQ show English
2. **Page metadata:** Title/description hardcoded in English (tracked: #473)
3. **Navigation brand:** Links to `/` not locale-aware (tracked: #474)
4. **No language switcher:** UI component deferred (tracked: #466)

---

## Reporting Issues

File bugs with:
- Browser and version
- Locale tested
- Screenshot of issue
- Console errors (if any)

Label: `i18n`, `website`, `bug`

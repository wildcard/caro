/**
 * Translation Index
 *
 * Central export point for all translation JSON files.
 * Import this file to access translations for any locale.
 *
 * Every JSON file under `locales/<locale>/` is loaded automatically via
 * `import.meta.glob`. Previously each locale was wired up by hand and only
 * `common.json` + `landing.json` were imported, so `hero.json`,
 * `features.json`, `download.json`, `navigation.json` and `compare.json`
 * (plus the `ai_safety` / `blog` / `use_cases` files that exist for some
 * locales) were translated but never shipped — those sections silently
 * rendered English on all 14 non-English locales. Globbing also keeps new
 * locale files wired up by default.
 */

/**
 * Eagerly load every locale JSON file at build time.
 * Keys look like `./locales/he/hero.json`.
 */
const localeModules = import.meta.glob<Record<string, unknown>>(
  './locales/*/*.json',
  { eager: true, import: 'default' }
);

/**
 * Merge every JSON file belonging to one locale into a single object.
 *
 * Glob keys are returned in a stable (sorted) order, so the merge is
 * deterministic. Locale files use disjoint top-level sections (`hero`,
 * `features`, `footer` + `navigation`, …), so a shallow merge is sufficient
 * and no file can clobber another's section.
 */
function collectLocale(locale: string): Record<string, unknown> {
  const prefix = `./locales/${locale}/`;
  const merged: Record<string, unknown> = {};

  for (const [path, mod] of Object.entries(localeModules)) {
    if (path.startsWith(prefix)) {
      Object.assign(merged, mod);
    }
  }

  return merged;
}

/**
 * English translation object (base/fallback).
 * Combines all English section translations into a single object.
 */
export const en = collectLocale('en');

/**
 * Build a locale by spreading English as the base, then overriding with the
 * locale's own sections.
 *
 * The merge is shallow, so a locale that translates only part of a section
 * replaces that whole section. That is safe because `t()` in `config.ts`
 * resolves keys one segment at a time and falls back to English per key, so
 * partially-translated sections still render English for missing leaves.
 */
function buildLocale(locale: string): Record<string, unknown> {
  return { ...en, ...collectLocale(locale) };
}

/**
 * Localized translations
 */
export const es = buildLocale('es');
export const fr = buildLocale('fr');
export const pt = buildLocale('pt');
export const de = buildLocale('de');
export const he = buildLocale('he');
export const ar = buildLocale('ar');
export const uk = buildLocale('uk');
export const ru = buildLocale('ru');
export const ja = buildLocale('ja');
export const ko = buildLocale('ko');
export const hi = buildLocale('hi');
export const ur = buildLocale('ur');
export const fil = buildLocale('fil');
export const id = buildLocale('id');

/**
 * Map of all translations by locale code
 */
export const translations = {
  en,
  es,
  fr,
  pt,
  de,
  he,
  ar,
  uk,
  ru,
  ja,
  ko,
  hi,
  ur,
  fil,
  id
};

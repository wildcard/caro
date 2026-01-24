/**
 * Localized Link Helpers
 *
 * Ensures internal navigation maintains locale persistence by prepending
 * the locale code to paths when not in English.
 */

import type { Locale } from '../i18n/config';

/**
 * Convert a path to its localized version
 *
 * @param path - The base path (e.g., '/features', '/blog/post-1')
 * @param locale - Target locale code
 * @returns Localized path with locale prefix if not English
 *
 * @example
 * localizedHref('/features', 'en')  → '/features'
 * localizedHref('/features', 'es')  → '/es/features'
 * localizedHref('/blog/', 'fr')     → '/fr/blog/'
 */
export function localizedHref(path: string, locale: Locale): string {
  // English is the default, no prefix needed
  if (locale === 'en') {
    return path;
  }

  // Ensure path starts with /
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;

  // Add locale prefix
  return `/${locale}${normalizedPath}`;
}

/**
 * Generate all localized versions of a path
 * Useful for building sitemap.xml or creating alternate links
 *
 * @param path - The base path
 * @param locales - Array of locale codes to generate
 * @returns Map of locale → localized path
 *
 * @example
 * getAllLocalizedHrefs('/features', ['en', 'es', 'fr'])
 * → { en: '/features', es: '/es/features', fr: '/fr/features' }
 */
export function getAllLocalizedHrefs(
  path: string,
  locales: readonly Locale[]
): Record<string, string> {
  const result: Record<string, string> = {};

  for (const locale of locales) {
    result[locale] = localizedHref(path, locale);
  }

  return result;
}

/**
 * Check if a path is already localized
 * Returns the locale code if found, null otherwise
 *
 * @param path - The path to check
 * @returns Locale code if path is localized, null if not
 *
 * @example
 * getLocaleFromHref('/es/features') → 'es'
 * getLocaleFromHref('/features')    → null
 */
export function getLocaleFromHref(path: string): Locale | null {
  const pathParts = path.split('/').filter(Boolean);
  if (pathParts.length === 0) return null;

  const firstSegment = pathParts[0];
  const possibleLocales = ['es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'];

  return possibleLocales.includes(firstSegment) ? (firstSegment as Locale) : null;
}

/**
 * Remove locale prefix from a path
 * Useful for getting the canonical/base path
 *
 * @param path - Possibly localized path
 * @returns Path without locale prefix
 *
 * @example
 * removeLocalePrefix('/es/features') → '/features'
 * removeLocalePrefix('/features')    → '/features'
 */
export function removeLocalePrefix(path: string): string {
  const locale = getLocaleFromHref(path);
  if (!locale) return path;

  return path.replace(`/${locale}`, '') || '/';
}

/**
 * Switch a localized path to a different locale
 * Preserves the page structure while changing language
 *
 * @param currentPath - Current localized path
 * @param newLocale - Target locale to switch to
 * @returns Path in the new locale
 *
 * @example
 * switchLocale('/es/features', 'fr') → '/fr/features'
 * switchLocale('/features', 'es')    → '/es/features'
 * switchLocale('/es/blog/post', 'en') → '/blog/post'
 */
export function switchLocale(currentPath: string, newLocale: Locale): string {
  const basePath = removeLocalePrefix(currentPath);
  return localizedHref(basePath, newLocale);
}

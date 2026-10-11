/**
 * One source of truth for the per-page SEO files.
 *
 * The <head> (src/components/SEO.astro) and the post-build step
 * (src/integrations/agent-seo.ts) both derive a page's share card, its
 * Markdown twin and its card text from the page's pathname and title with
 * these functions. The head can then point at files the build writes later,
 * and the two never drift.
 *
 * Pure: no Astro, no fs. Covered by src/lib/seo/routes.test.ts.
 */
import type { Locale } from '../../i18n/config';

export const SITE_NAME = 'Caro';
export const X_HANDLE = '@CaroDaShellShib';

/** Every locale the site routes, default first. Typed against `Locale`. */
export const LOCALES = [
  'en', 'es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id',
] as const satisfies readonly Locale[];

export const DEFAULT_LOCALE: Locale = 'en';

/**
 * Paths that have a real translated page under every locale prefix.
 *
 * Every other `/<locale>/...` URL is an i18n fallback: a `noindex`
 * meta-refresh stub that redirects to the English page. Pointing hreflang or
 * the sitemap at those stubs tells search engines to index redirects, so only
 * these paths get alternates. The post-build check fails the build if an
 * hreflang target turns out to be a redirect stub.
 */
export const LOCALIZED_PATHS: readonly string[] = ['/'];

/** Real pages that must stay out of search indexes and the sitemap. */
export const NOINDEX_PATHS: readonly string[] = ['/occasions'];

/** `og:locale` values, in the language_TERRITORY form Open Graph expects. */
export const OG_LOCALES: Record<Locale, string> = {
  en: 'en_US',
  es: 'es_ES',
  fr: 'fr_FR',
  pt: 'pt_BR',
  de: 'de_DE',
  he: 'he_IL',
  ar: 'ar_AR',
  uk: 'uk_UA',
  ru: 'ru_RU',
  ja: 'ja_JP',
  ko: 'ko_KR',
  hi: 'hi_IN',
  ur: 'ur_PK',
  fil: 'tl_PH',
  id: 'id_ID',
};

/** Section labels, by first path segment, in llms.txt order. Used on cards and in breadcrumbs. */
export const SECTION_LABELS: Record<string, string> = {
  docs: 'Docs',
  blog: 'Blog',
  compare: 'Compare',
  'use-cases': 'Use cases',
  product: 'Product',
  explore: 'Explore',
  integrations: 'Integrations',
};

/** The eyebrow on cards for pages outside a section. */
export const DEFAULT_EYEBROW = 'Your loyal shell companion';

/** `/docs/cli/` → `/docs/cli`; `/` stays `/`. The canonical path form. */
export function normalizePath(pathname: string): string {
  const path = `/${pathname.replace(/^\/+/, '')}`.replace(/\/index\.html$/, '/');
  return path === '/' ? '/' : path.replace(/\/+$/, '');
}

/** The file key shared by a page's card and its Markdown twin: `docs/cli`, or `index` for `/`. */
export function routeKey(pathname: string): string {
  const path = normalizePath(pathname);
  return path === '/' ? 'index' : path.slice(1);
}

/** Where the build writes a page's share card. */
export function ogImagePath(pathname: string): string {
  return `/og/${routeKey(pathname)}.png`;
}

/** Where the build writes a page's Markdown twin. */
export function markdownPath(pathname: string): string {
  return `/${routeKey(pathname)}.md`;
}

/** The locale prefix of a path, or the default locale. */
export function localeOf(pathname: string): Locale {
  const first = normalizePath(pathname).split('/')[1] ?? '';
  return (LOCALES as readonly string[]).includes(first) && first !== DEFAULT_LOCALE
    ? (first as Locale)
    : DEFAULT_LOCALE;
}

/** The path without its locale prefix: `/es/` → `/`, `/es/docs` → `/docs`. */
export function unlocalizedPath(pathname: string): string {
  const path = normalizePath(pathname);
  const locale = localeOf(path);
  if (locale === DEFAULT_LOCALE) return path;
  return normalizePath(path.slice(locale.length + 1) || '/');
}

/** `path` under `locale`: `('/', 'es')` → `/es`. */
export function localizePath(path: string, locale: Locale): string {
  const base = normalizePath(path);
  if (locale === DEFAULT_LOCALE) return base;
  return normalizePath(`/${locale}${base === '/' ? '' : base}`);
}

/** The hreflang alternates for a page, or none when it has no translations. */
export function hreflangAlternates(pathname: string): { hreflang: string; path: string }[] {
  const base = unlocalizedPath(pathname);
  if (!LOCALIZED_PATHS.includes(base)) return [];
  return [
    ...LOCALES.map((locale) => ({ hreflang: locale, path: localizePath(base, locale) })),
    { hreflang: 'x-default', path: base },
  ];
}

/**
 * Whether a built page belongs in the sitemap: not a locale fallback stub and
 * not a `noindex` page.
 */
export function isIndexablePath(pathname: string): boolean {
  const path = normalizePath(pathname);
  if (NOINDEX_PATHS.includes(unlocalizedPath(path))) return false;
  if (localeOf(path) === DEFAULT_LOCALE) return true;
  return LOCALIZED_PATHS.includes(unlocalizedPath(path));
}

/**
 * The title a card prints: the page title without the site name that tabs
 * and search results need, e.g. `CLI Reference - Caro Docs` → `CLI Reference`
 * and `AI Agent Safety | Caro - Hallucination-Resistant …` → `AI Agent Safety`.
 * A spaced-dash subtitle becomes a colon: `Caro vs Warp - Feature Comparison`
 * → `Caro vs Warp: Feature Comparison`.
 */
export function cardTitle(title: string): string {
  const raw = title.replace(/\s+/g, ' ').trim();
  let text = raw
    .replace(/\s+[-–—|]\s+Caro(?:\s+(?:Blog|Docs))?$/i, '')
    .replace(/^Caro\s+[-–—|:]\s+/i, '');
  const [head] = text.split(/\s+\|\s+/);
  text = (head ?? text).trim();
  // A spaced dash introduces a subtitle; a colon lets the card break there.
  if (!text.includes(':')) text = text.replace(/\s+[-–—]\s+/, ': ');
  return text || raw;
}

const CARD_DATE = new Intl.DateTimeFormat('en-US', {
  year: 'numeric',
  month: 'short',
  day: 'numeric',
  timeZone: 'UTC',
});

/** The section a path belongs to, e.g. `Docs`, or undefined outside a section. */
export function sectionOf(pathname: string): string | undefined {
  const first = unlocalizedPath(pathname).split('/')[1];
  return first ? SECTION_LABELS[first] : undefined;
}

/**
 * The short labels over a card's title: the section, and for articles the
 * publish date (UTC, so every build machine prints the same card) and the
 * reading time.
 */
export function cardEyebrow({
  pathname,
  publishedTime,
  readingTime,
}: {
  pathname: string;
  publishedTime?: string;
  readingTime?: string;
}): string[] {
  const items = [sectionOf(pathname) ?? DEFAULT_EYEBROW];
  const date = publishedTime ? new Date(publishedTime) : undefined;
  if (date && !Number.isNaN(date.getTime())) items.push(CARD_DATE.format(date));
  if (readingTime) items.push(readingTime);
  return items;
}

/** The URL label printed at the bottom of a card: `caro.sh/docs/cli`. */
export function cardUrlLabel(pathname: string, host = 'caro.sh'): string {
  const path = normalizePath(pathname);
  return path === '/' ? host : `${host}${path}`;
}

/** Breadcrumb trail for structured data: Home › Docs › CLI Reference. */
export function breadcrumbs(pathname: string, title: string): { name: string; path: string }[] {
  const path = normalizePath(pathname);
  if (path === '/' || localeOf(path) !== DEFAULT_LOCALE) return [];
  const segments = path.split('/').filter(Boolean);
  return [
    { name: 'Home', path: '/' },
    ...segments.map((segment, i) => {
      const crumbPath = `/${segments.slice(0, i + 1).join('/')}`;
      const last = i === segments.length - 1;
      const name = last ? cardTitle(title) : (SECTION_LABELS[segment] ?? humanize(segment));
      return { name, path: crumbPath };
    }),
  ];
}

function humanize(segment: string): string {
  const words = segment.replace(/[-_]+/g, ' ').trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

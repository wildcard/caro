import { describe, expect, it } from 'vitest';
import {
  breadcrumbs,
  cardEyebrow,
  cardTitle,
  cardUrlLabel,
  hreflangAlternates,
  isIndexablePath,
  LOCALES,
  localeOf,
  markdownPath,
  normalizePath,
  ogImagePath,
  routeKey,
  unlocalizedPath,
} from './routes';

describe('page paths', () => {
  it.each([
    ['/', '/'],
    ['', '/'],
    ['/docs/cli/', '/docs/cli'],
    ['docs/cli/', '/docs/cli'],
    ['/docs/cli/index.html', '/docs/cli'],
    ['/es/', '/es'],
  ])('normalizes %j to %j', (input, expected) => {
    expect(normalizePath(input)).toBe(expected);
  });

  it('gives each page one key for its card and its Markdown twin', () => {
    expect(routeKey('/')).toBe('index');
    expect(ogImagePath('/')).toBe('/og/index.png');
    expect(markdownPath('/')).toBe('/index.md');
    expect(ogImagePath('/blog/why-caro/')).toBe('/og/blog/why-caro.png');
    expect(markdownPath('/blog/why-caro/')).toBe('/blog/why-caro.md');
  });

  it('reads the locale prefix', () => {
    expect(localeOf('/es/')).toBe('es');
    expect(localeOf('/docs/cli')).toBe('en');
    expect(localeOf('/espanol')).toBe('en');
    expect(unlocalizedPath('/es/')).toBe('/');
    expect(unlocalizedPath('/fil/credits/')).toBe('/credits');
    expect(unlocalizedPath('/docs')).toBe('/docs');
  });
});

describe('hreflangAlternates', () => {
  it('lists every locale and x-default on a translated page', () => {
    const alternates = hreflangAlternates('/es/');
    expect(alternates).toHaveLength(LOCALES.length + 1);
    expect(alternates).toContainEqual({ hreflang: 'en', path: '/' });
    expect(alternates).toContainEqual({ hreflang: 'es', path: '/es' });
    expect(alternates).toContainEqual({ hreflang: 'x-default', path: '/' });
  });

  it('lists nothing on a page whose locale URLs are only redirect stubs', () => {
    // /es/docs/cli is an i18n fallback that redirects to /docs/cli.
    expect(hreflangAlternates('/docs/cli/')).toEqual([]);
    expect(hreflangAlternates('/credits')).toEqual([]);
  });
});

describe('isIndexablePath', () => {
  it.each([
    ['/', true],
    ['/docs/cli/', true],
    ['/es/', true],
    ['/es/docs/', false],
    ['/ar/de/', false],
    ['/occasions/', false],
  ])('%s → %s', (path, expected) => {
    expect(isIndexablePath(path)).toBe(expected);
  });
});

describe('cardTitle', () => {
  it.each([
    ['CLI Reference - Caro Docs', 'CLI Reference'],
    ['Why Caro? The Story Behind Your Terminal Companion - Caro Blog', 'Why Caro? The Story Behind Your Terminal Companion'],
    ['Caro - Terminal mastery, unlocked', 'Terminal mastery, unlocked'],
    ['Caro - Tu fiel compañero de shell', 'Tu fiel compañero de shell'],
    ['AI Agent Safety | Caro - Hallucination-Resistant Command Validation', 'AI Agent Safety'],
    ['Modern Unix Tools - Next-Gen CLI Tools in Rust & Go | Caro', 'Modern Unix Tools: Next-Gen CLI Tools in Rust & Go'],
    ['Caro vs Warp - Feature Comparison | Terminal AI Tools', 'Caro vs Warp: Feature Comparison'],
    ['Integrations — Caro', 'Integrations'],
    ['Blog - Caro', 'Blog'],
    ["Caro's founder arc: adopting Anthropic's playbook - Caro Blog", "Caro's founder arc: adopting Anthropic's playbook"],
    ['Caro', 'Caro'],
  ])('%j → %j', (title, expected) => {
    expect(cardTitle(title)).toBe(expected);
  });
});

describe('cardEyebrow', () => {
  it('labels a page by its section', () => {
    expect(cardEyebrow({ pathname: '/docs/cli/' })).toEqual(['Docs']);
    expect(cardEyebrow({ pathname: '/use-cases/sre' })).toEqual(['Use cases']);
    expect(cardEyebrow({ pathname: '/faq' })).toEqual(['Your loyal shell companion']);
  });

  it('dates an article in UTC, whatever the machine time zone', () => {
    const eyebrow = cardEyebrow({
      pathname: '/blog/why-caro/',
      publishedTime: '2025-12-17T00:00:00.000Z',
      readingTime: '8 min read',
    });
    expect(eyebrow).toEqual(['Blog', 'Dec 17, 2025', '8 min read']);
  });

  it('ignores a date it cannot parse', () => {
    expect(cardEyebrow({ pathname: '/blog/x', publishedTime: 'soon' })).toEqual(['Blog']);
  });
});

describe('cardUrlLabel', () => {
  it('prints the host, then the path', () => {
    expect(cardUrlLabel('/')).toBe('caro.sh');
    expect(cardUrlLabel('/docs/cli/')).toBe('caro.sh/docs/cli');
  });
});

describe('breadcrumbs', () => {
  it('builds Home › section › page', () => {
    expect(breadcrumbs('/docs/cli/', 'CLI Reference - Caro Docs')).toEqual([
      { name: 'Home', path: '/' },
      { name: 'Docs', path: '/docs' },
      { name: 'CLI Reference', path: '/docs/cli' },
    ]);
  });

  it('has no trail on the home page or a translated page', () => {
    expect(breadcrumbs('/', 'Caro')).toEqual([]);
    expect(breadcrumbs('/es/', 'Caro')).toEqual([]);
  });
});

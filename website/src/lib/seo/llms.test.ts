import { describe, expect, it } from 'vitest';
import { buildLlmsFullTxt, buildLlmsTxt, type LlmsPage } from './llms';
import { buildRss } from './rss';

const site = new URL('https://caro.sh');

const page = (path: string, title: string, locale = 'en'): LlmsPage => ({
  path,
  url: new URL(path, site).href,
  markdownUrl: new URL(`${path === '/' ? '/index' : path}.md`, site).href,
  title,
  description: `About ${title}.`,
  locale,
  markdown: `---\ntitle: "${title}"\n---\n\n# ${title}\n\nBody of ${path}.\n`,
});

const pages = [
  page('/blog/why-caro', 'Why Caro? - Caro Blog'),
  page('/', 'Caro - Terminal mastery, unlocked'),
  page('/docs/cli', 'CLI Reference - Caro Docs'),
  page('/docs', 'Documentation - Caro Docs'),
  page('/faq', 'FAQ - Caro Shell Assistant'),
  page('/es', 'Caro - Tu fiel compañero de shell', 'es'),
];

describe('buildLlmsTxt', () => {
  const txt = buildLlmsTxt('# Caro\n\n> Summary.', pages, site);

  it('starts with the hand-written header', () => {
    expect(txt.startsWith('# Caro\n\n> Summary.\n\n## How to read this site')).toBe(true);
  });

  it('lists every page by its Markdown twin, docs first, section index first', () => {
    const sections = [...txt.matchAll(/^## (.+)$/gm)].map((m) => m[1]);
    expect(sections).toEqual(['How to read this site', 'Docs', 'Blog', 'Pages', 'Optional']);
    expect(txt).toContain(
      '## Docs\n\n- [Documentation](https://caro.sh/docs.md): About Documentation - Caro Docs.\n- [CLI Reference](https://caro.sh/docs/cli.md)',
    );
    expect(txt).toContain('- [Terminal mastery, unlocked](https://caro.sh/index.md)');
  });

  it('puts translations under Optional', () => {
    expect(txt).toContain('## Optional\n\n- [Tu fiel compañero de shell (es)](https://caro.sh/es.md)');
  });
});

describe('buildLlmsFullTxt', () => {
  const full = buildLlmsFullTxt('# Caro', pages);

  it('holds every primary-locale page, each under its source URL', () => {
    expect(full).toContain('# CLI Reference - Caro Docs\n\nSource: https://caro.sh/docs/cli\n\nBody of /docs/cli.');
    expect(full).not.toContain('Body of /es.');
    expect(full).not.toContain('title: "');
  });
});

describe('buildRss', () => {
  it('lists posts newest first, escaped', () => {
    const rss = buildRss(
      [
        { url: 'https://caro.sh/blog/a', title: 'Old & gold - Caro Blog', description: '<b>a</b>', publishedTime: '2025-01-01T00:00:00Z' },
        { url: 'https://caro.sh/blog/b', title: 'New', description: 'b', publishedTime: '2026-01-01T00:00:00Z' },
      ],
      site,
    );
    expect(rss.indexOf('blog/b')).toBeLessThan(rss.indexOf('blog/a'));
    expect(rss).toContain('<title>Old &amp; gold</title>');
    expect(rss).toContain('<description>&lt;b&gt;a&lt;/b&gt;</description>');
    expect(rss).toContain('<pubDate>Thu, 01 Jan 2026 00:00:00 GMT</pubDate>');
  });
});

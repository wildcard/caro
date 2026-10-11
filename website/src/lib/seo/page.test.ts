import { describe, expect, it } from 'vitest';
import { extractMeta, isRedirectStub, pageToMarkdown, parseHtml, stripFrontMatter } from './page';

const page = (body: string, head = '') => `<!doctype html>
<html lang="en"><head>
<title>CLI Reference - Caro Docs</title>
<meta name="description" content="Every flag.">
<link rel="canonical" href="https://caro.sh/docs/cli">
<link rel="alternate" type="text/markdown" href="https://caro.sh/docs/cli.md">
<link rel="alternate" hreflang="es" href="https://caro.sh/es">
<meta property="og:title" content="CLI Reference - Caro Docs">
<meta property="og:image" content="https://caro.sh/og/docs/cli.png">
<meta property="og:type" content="article">
<meta property="article:published_time" content="2025-12-17T00:00:00.000Z">
<meta name="twitter:label1" content="Reading time"><meta name="twitter:data1" content="3 min read">
<meta name="robots" content="index, follow">
${head}
</head><body>
<nav><a href="/">Home</a></nav>
${body}
<footer>Footer links</footer>
</body></html>`;

const options = {
  url: 'https://caro.sh/docs/cli',
  markdownUrl: 'https://caro.sh/docs/cli.md',
  title: 'CLI Reference',
};

async function markdown(body: string) {
  const tree = parseHtml(page(body));
  return pageToMarkdown(tree, extractMeta(tree), options);
}

describe('isRedirectStub', () => {
  it('spots Astro i18n fallback pages', () => {
    expect(
      isRedirectStub('<!doctype html><title>Redirecting to: /docs/</title><meta http-equiv="refresh" content="2;url=/docs/">'),
    ).toBe(true);
    expect(isRedirectStub(page('<main>Hi</main>'))).toBe(false);
  });
});

describe('extractMeta', () => {
  it('reads the SEO tags the build checks', () => {
    const meta = extractMeta(parseHtml(page('<main></main>')));
    expect(meta).toMatchObject({
      title: 'CLI Reference - Caro Docs',
      description: 'Every flag.',
      canonical: 'https://caro.sh/docs/cli',
      lang: 'en',
      ogImage: 'https://caro.sh/og/docs/cli.png',
      ogType: 'article',
      publishedTime: '2025-12-17T00:00:00.000Z',
      readingTime: '3 min read',
      markdownHref: 'https://caro.sh/docs/cli.md',
      hreflang: [{ hreflang: 'es', href: 'https://caro.sh/es' }],
      robots: 'index, follow',
      isRedirect: false,
    });
  });
});

describe('pageToMarkdown', () => {
  it('writes front matter, then the content of <main> only', async () => {
    const md = await markdown('<main><h1>CLI Reference</h1><p>Use <code>caro</code>.</p></main>');
    expect(md).toBe(`---
title: "CLI Reference"
description: "Every flag."
url: "https://caro.sh/docs/cli"
markdown_url: "https://caro.sh/docs/cli.md"
---

# CLI Reference

Use \`caro\`.
`);
  });

  it('leaves out chrome: nav, footer, scripts, buttons, forms, SVG and opted-out blocks', async () => {
    const md = stripFrontMatter(
      await markdown(`<main><h1>T</h1>
        <script>alert(1)</script><style>p{}</style>
        <button>Copy</button><form><label>Email</label><input></form>
        <svg><text>icon</text></svg>
        <div aria-hidden="true">decor</div>
        <aside data-md-skip>Share on X</aside>
        <p>Kept.</p></main>`),
    );
    expect(md).toBe('# T\n\nKept.\n');
    expect(md).not.toMatch(/Home|Footer/);
  });

  it('without a <main>, uses <body> minus its header, nav, aside and footer', async () => {
    const html = `<!doctype html><html><head><title>T</title></head><body>
      <header>Site header</header><nav>Menu</nav>
      <h1>Body page</h1><p>Kept.</p>
      <aside>Sidebar</aside><footer>Footer links</footer></body></html>`;
    const tree = parseHtml(html);
    const md = stripFrontMatter(await pageToMarkdown(tree, extractMeta(tree), options));
    expect(md).toBe('# Body page\n\nKept.\n');
  });

  it('makes links and images absolute and drops script links', async () => {
    const md = await markdown(
      '<main><h1>T</h1><p><a href="/docs/safety">Safety</a> <a href="#flags">Flags</a> <a href="javascript:void(0)">X</a></p><img src="/caro-pixel.png" alt="Caro"></main>',
    );
    expect(md).toContain('[Safety](https://caro.sh/docs/safety)');
    expect(md).toContain('[Flags](https://caro.sh/docs/cli#flags)');
    expect(md).not.toContain('javascript:');
    const spaced = await markdown('<main><h1>T</h1><p><a href=" javascript:alert(1)">X</a> <img src=" data:image/png;base64,AA" alt="D"></p></main>');
    expect(spaced).not.toMatch(/javascript:|data:/);
    expect(md).toContain('![Caro](https://caro.sh/caro-pixel.png)');
  });

  it('keeps code blocks fenced, with their language', async () => {
    const md = await markdown('<main><h1>T</h1><pre><code class="language-bash">caro "list files"\n</code></pre></main>');
    expect(md).toContain('```bash\ncaro "list files"\n```');
  });

  it('links a card heading once instead of every line of the card', async () => {
    const md = stripFrontMatter(
      await markdown('<main><h1>T</h1><a href="/docs"><h3>Docs</h3><p>Read them.</p></a></main>'),
    );
    expect(md).toBe('# T\n\n### [Docs](https://caro.sh/docs)\n\nRead them.\n');
  });

  it('writes a heading with a line break on one # line', async () => {
    const md = stripFrontMatter(await markdown('<main><h1>Your 3 AM<br>Incident <span>Companion<br></span></h1></main>'));
    expect(md.split('\n')[0]).toMatch(/^# Your 3 AM Incident Companion\s*$/);
  });

  it('adds the page title as H1 when the content has none', async () => {
    const md = stripFrontMatter(await markdown('<main><p>No heading here.</p></main>'));
    expect(md).toBe('# CLI Reference\n\nNo heading here.\n');
  });

  it('adds the H1 even when a code block has a "# comment" line', async () => {
    const md = stripFrontMatter(
      await markdown('<main><h2>Usage</h2><pre><code># Generate a command\ncaro "list"\n</code></pre></main>'),
    );
    expect(md.startsWith('# CLI Reference\n\n## Usage')).toBe(true);
  });

  it('keeps tables as GFM tables', async () => {
    const md = await markdown(
      '<main><h1>T</h1><table><thead><tr><th>Flag</th><th>Use</th></tr></thead><tbody><tr><td>-s</td><td>shell</td></tr></tbody></table></main>',
    );
    expect(md).toMatch(/\| Flag \| Use +\|\n\| -+ \| -+ \|\n\| -s +\| shell \|/);
  });
});

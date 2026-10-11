/**
 * caro:agent-seo — the per-page files that make every page shareable and
 * readable by agents, written after the build from the HTML that shipped:
 *
 *   /og/<page>.png       a 1200 × 630 share card (src/lib/seo/og-card.ts)
 *   /<page>.md           the page as Markdown with front matter
 *   /llms.txt            an index of every page's Markdown twin
 *   /llms-full.txt       every page's Markdown in one file
 *   /rss.xml             the blog feed, from the posts that shipped
 *   .vercel/output/config.json   `Accept: text/markdown` negotiation and
 *                        `Link` headers (src/lib/seo/vercel-routes.ts)
 *
 * SEO.astro points each page's <head> at these paths with the same helpers
 * (src/lib/seo/routes.ts). The step then checks every page and fails the
 * build when a page lacks a title, description, canonical URL, share card or
 * Markdown twin, or has an hreflang alternate that is a redirect stub.
 */
import { existsSync } from 'node:fs';
import { mkdir, readFile, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { AstroIntegration, AstroIntegrationLogger } from 'astro';
import { buildLlmsFullTxt, buildLlmsTxt, type LlmsPage } from '../lib/seo/llms';
import { type CardInput, planCard, renderCard } from '../lib/seo/og-card';
import { buildRss } from '../lib/seo/rss';
import {
  extractMeta,
  isRedirectStub,
  type PageMeta,
  pageToMarkdown,
  parseHtml,
} from '../lib/seo/page';
import {
  cardEyebrow,
  cardTitle,
  cardUrlLabel,
  DEFAULT_LOCALE,
  localeOf,
  markdownPath,
  normalizePath,
  ogImagePath,
  unlocalizedPath,
} from '../lib/seo/routes';
import { markdownRoutes, withMarkdownRoutes } from '../lib/seo/vercel-routes';

interface BuiltPage {
  path: string;
  meta: PageMeta;
  markdown: string;
}

/** Attribute values this long are inlined data (e.g. the search index), never content. */
const HUGE_ATTRIBUTE = /\s[\w:-]+="[^"]{65536,}"/g;

const CONCURRENCY = 8;

async function eachLimit<T>(items: T[], limit: number, run: (item: T) => Promise<void>) {
  let next = 0;
  const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (next < items.length) await run(items[next++] as T);
  });
  await Promise.all(workers);
}

function outFile(dir: URL, publicPath: string): string {
  return path.join(fileURLToPath(dir), publicPath.replace(/^\//, ''));
}

async function write(file: string, data: string | Buffer) {
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, data);
}

/**
 * The card for a page. A translated page whose title or eyebrow the Latin
 * card fonts can't draw uses the English page's text, under its own URL.
 */
async function cardFor(
  page: BuiltPage,
  byPath: Map<string, BuiltPage>,
  host: string,
  markFile: string,
  logger: AstroIntegrationLogger,
): Promise<CardInput> {
  const input = (source: BuiltPage): CardInput => ({
    title: cardTitle(source.meta.ogTitle || source.meta.title),
    eyebrow: cardEyebrow({
      pathname: source.path,
      publishedTime: source.meta.publishedTime,
      readingTime: source.meta.readingTime,
    }),
    url: cardUrlLabel(page.path, host),
  });
  const own = input(page);
  const plan = await planCard(own, markFile);
  if (plan.missingGlyphs.length === 0) {
    if (plan.title.overflow) logger.warn(`${page.path}: card title is cut: "${own.title}"`);
    return own;
  }
  const english = byPath.get(unlocalizedPath(page.path));
  if (localeOf(page.path) !== DEFAULT_LOCALE && english) return input(english);
  logger.warn(`${page.path}: card fonts lack ${plan.missingGlyphs.join(' ')}`);
  return own;
}

function verify(pages: BuiltPage[], dir: URL, site: URL): string[] {
  const problems: string[] = [];
  const paths = new Set(pages.map((page) => page.path));
  const localFile = (href: string) => {
    const url = new URL(href, site);
    return url.host === site.host ? outFile(dir, decodeURIComponent(url.pathname)) : undefined;
  };
  for (const { path: p, meta } of pages) {
    const fail = (message: string) => problems.push(`${p}: ${message}`);
    if (!meta.title) fail('no <title>');
    if (!meta.description) fail('no meta description');
    if (!meta.canonical) fail('no canonical URL');
    else if (new URL(meta.canonical, site).origin !== site.origin) fail(`canonical is off-site: ${meta.canonical}`);
    if (!meta.ogImage) fail('no og:image');
    else {
      const file = localFile(meta.ogImage);
      if (file && !existsSync(file)) fail(`og:image ${meta.ogImage} was not generated`);
    }
    const twin = markdownPath(p);
    if (!meta.markdownHref) fail('no <link rel="alternate" type="text/markdown">');
    else if (new URL(meta.markdownHref, site).pathname !== twin) {
      fail(`Markdown link ${meta.markdownHref} should be ${twin}`);
    }
    for (const { hreflang, href } of meta.hreflang) {
      if (!paths.has(normalizePath(new URL(href, site).pathname))) {
        fail(`hreflang ${hreflang} points at ${href}, which is not a page (a redirect stub?)`);
      }
    }
  }
  return problems;
}

/**
 * The release version from the repo's Cargo.toml (read directly: the
 * generated src/config/version.ts is untracked and may not exist yet).
 */
async function caroVersion(root: URL): Promise<string> {
  const cargo = await readFile(new URL('../Cargo.toml', root), 'utf8');
  const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1];
  if (!version) throw new Error('caro:agent-seo could not read the version from Cargo.toml');
  return version;
}

export default function agentSeo(): AstroIntegration {
  let site: URL;
  let root: URL;
  let publicDir: URL;

  return {
    name: 'caro:agent-seo',
    hooks: {
      'astro:config:done': ({ config }) => {
        if (!config.site) throw new Error('caro:agent-seo needs `site` in astro.config.mjs');
        site = new URL(config.site);
        root = config.root;
        publicDir = config.publicDir;
      },

      'astro:build:done': async ({ dir, pages: routes, logger }) => {
        const started = Date.now();
        const pages: BuiltPage[] = [];

        // 1. Read every built page; skip i18n redirect stubs and endpoints.
        for (const { pathname } of routes) {
          const file = path.join(fileURLToPath(dir), pathname, 'index.html');
          if (!existsSync(file)) continue;
          const raw = await readFile(file, 'utf8');
          if (isRedirectStub(raw)) continue;
          const tree = parseHtml(raw.replace(HUGE_ATTRIBUTE, ''));
          const meta = extractMeta(tree);
          if (meta.isRedirect) continue;
          const p = normalizePath(pathname);
          const url = meta.canonical || new URL(p, site).href;
          const markdown = await pageToMarkdown(tree, meta, {
            url,
            markdownUrl: new URL(markdownPath(p), site).href,
            title: cardTitle(meta.ogTitle || meta.title) || meta.title,
            extra: {
              lang: meta.lang || undefined,
              type: meta.ogType || undefined,
              published: meta.publishedTime,
              reading_time: meta.readingTime,
              image: meta.ogImage || undefined,
            },
          });
          pages.push({ path: p, meta, markdown });
        }
        const byPath = new Map(pages.map((page) => [page.path, page]));

        // 2. Markdown twins.
        for (const page of pages) await write(outFile(dir, markdownPath(page.path)), page.markdown);

        // 3. Share cards, for pages whose og:image is the conventional card path.
        const markFile = fileURLToPath(new URL('mark-caro-smooth.png', publicDir));
        const carded = pages.filter(
          (page) => page.meta.ogImage && new URL(page.meta.ogImage, site).pathname === ogImagePath(page.path),
        );
        await eachLimit(carded, CONCURRENCY, async (page) => {
          const input = await cardFor(page, byPath, site.host, markFile, logger);
          await write(outFile(dir, ogImagePath(page.path)), await renderCard(input, markFile));
        });

        // 4. llms.txt and llms-full.txt.
        const headerFile = new URL('../lib/seo/llms-header.md', import.meta.url);
        const header = (await readFile(headerFile, 'utf8')).replaceAll(
          '{{version}}',
          await caroVersion(root),
        );
        const llmsPages: LlmsPage[] = pages
          .filter((page) => !/noindex/i.test(page.meta.robots))
          .map((page) => ({
            path: page.path,
            url: page.meta.canonical || new URL(page.path, site).href,
            markdownUrl: new URL(markdownPath(page.path), site).href,
            title: page.meta.title,
            description: page.meta.description,
            locale: localeOf(page.path),
            markdown: page.markdown,
          }));
        const llmsTxt = buildLlmsTxt(header, llmsPages, site);
        const llmsFullTxt = buildLlmsFullTxt(header, llmsPages);
        await write(outFile(dir, '/llms.txt'), llmsTxt);
        await write(outFile(dir, '/llms-full.txt'), llmsFullTxt);
        // The names an older generator used; kept so existing links still work.
        await write(outFile(dir, '/llm.txt'), llmsTxt);
        await write(outFile(dir, '/llm-full.txt'), llmsFullTxt);

        // 5. The blog feed, from every post that shipped.
        const posts = pages.filter(
          (page) => page.path.startsWith('/blog/') && page.meta.publishedTime,
        );
        await write(
          outFile(dir, '/rss.xml'),
          buildRss(
            await Promise.all(
              posts.map(async (page) => {
                const image = page.meta.ogImage || undefined;
                const file = image && new URL(image, site).host === site.host
                  ? outFile(dir, new URL(image, site).pathname)
                  : undefined;
                return {
                  url: page.meta.canonical || new URL(page.path, site).href,
                  title: page.meta.ogTitle || page.meta.title,
                  description: page.meta.description,
                  publishedTime: page.meta.publishedTime as string,
                  image,
                  imageLength: file && existsSync(file) ? (await stat(file)).size : undefined,
                };
              }),
            ),
            site,
          ),
        );

        // 6. Every page must carry the full set.
        const problems = verify(pages, dir, site);
        if (problems.length > 0) {
          throw new Error(
            `caro:agent-seo found ${problems.length} problem(s):\n- ${problems.join('\n- ')}`,
          );
        }

        // 7. Vercel routes for Markdown negotiation and Link headers.
        const configFile = new URL('.vercel/output/config.json', root);
        if (existsSync(configFile)) {
          const config = JSON.parse(await readFile(configFile, 'utf8'));
          const next = withMarkdownRoutes(config, markdownRoutes(pages.map((p) => p.path), site));
          await writeFile(configFile, `${JSON.stringify(next, null, '\t')}\n`);
        } else {
          logger.warn('no .vercel/output/config.json; skipped the Markdown negotiation routes');
        }

        logger.info(
          `${pages.length} pages: ${carded.length} share cards, ${pages.length} Markdown twins, llms.txt in ${((Date.now() - started) / 1000).toFixed(1)}s`,
        );
      },
    },
  };
}

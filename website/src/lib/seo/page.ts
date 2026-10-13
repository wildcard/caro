/**
 * Reads a built HTML page: its SEO metadata, and its content as Markdown.
 *
 * The post-build step (src/integrations/agent-seo.ts) runs this over every
 * page in the build output, so the Markdown twin is always the page an agent
 * would see in a browser, minus the chrome: no navigation, scripts, styles,
 * forms or decorative SVG.
 */
import type { Element, ElementContent, Root, RootContent } from 'hast';
import rehypeParse from 'rehype-parse';
import rehypeRemark from 'rehype-remark';
import remarkGfm from 'remark-gfm';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';

export interface PageMeta {
  title: string;
  description: string;
  canonical: string;
  lang: string;
  ogTitle: string;
  ogImage: string;
  ogType: string;
  publishedTime?: string;
  /** Twitter's `label1`/`data1` pair: the reading time on blog posts. */
  readingTime?: string;
  robots: string;
  /** `rel=alternate type=text/markdown` href, as written in the page. */
  markdownHref?: string;
  /** hreflang alternates, as written in the page. */
  hreflang: { hreflang: string; href: string }[];
  /** True for i18n fallback pages: a meta-refresh to another URL. */
  isRedirect: boolean;
}

/** Fast check on raw HTML: Astro's i18n fallback stubs are meta refreshes. */
export function isRedirectStub(html: string): boolean {
  return /<meta\s+http-equiv="refresh"/i.test(html.slice(0, 2048));
}

export function parseHtml(html: string): Root {
  return unified().use(rehypeParse).parse(html);
}

function isElement(node: RootContent | ElementContent | Root): node is Element {
  return node.type === 'element';
}

function walk(node: Root | Element, visit: (el: Element) => void): void {
  for (const child of node.children) {
    if (isElement(child)) {
      visit(child);
      walk(child, visit);
    }
  }
}

function find(node: Root | Element, test: (el: Element) => boolean): Element | undefined {
  for (const child of node.children) {
    if (!isElement(child)) continue;
    if (test(child)) return child;
    const found = find(child, test);
    if (found) return found;
  }
  return undefined;
}

function textOf(node: Element | Root): string {
  let text = '';
  for (const child of node.children) {
    if (child.type === 'text') text += child.value;
    else if (isElement(child)) text += textOf(child);
  }
  return text;
}

function attr(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (value === undefined || value === null || value === false) return undefined;
  return Array.isArray(value) ? value.join(' ') : String(value);
}

function hasClass(el: Element, name: string): boolean {
  const value = el.properties?.className;
  return Array.isArray(value) && value.includes(name);
}

export function extractMeta(tree: Root): PageMeta {
  const meta: PageMeta = {
    title: '',
    description: '',
    canonical: '',
    lang: '',
    ogTitle: '',
    ogImage: '',
    ogType: '',
    robots: '',
    hreflang: [],
    isRedirect: false,
  };
  let twitterLabel = '';
  let twitterData = '';
  walk(tree, (el) => {
    if (el.tagName === 'html') meta.lang = attr(el, 'lang') ?? '';
    if (el.tagName === 'title' && !meta.title) meta.title = textOf(el).trim();
    if (el.tagName === 'meta') {
      const key = attr(el, 'name') ?? attr(el, 'property') ?? '';
      const content = attr(el, 'content') ?? '';
      if (attr(el, 'httpEquiv')?.toLowerCase() === 'refresh') meta.isRedirect = true;
      if (key === 'description') meta.description = content;
      else if (key === 'og:title') meta.ogTitle = content;
      else if (key === 'og:image') meta.ogImage = content;
      else if (key === 'og:type') meta.ogType = content;
      else if (key === 'article:published_time') meta.publishedTime = content;
      else if (key === 'robots') meta.robots = content;
      else if (key === 'twitter:label1') twitterLabel = content;
      else if (key === 'twitter:data1') twitterData = content;
    }
    if (el.tagName === 'link') {
      const rel = attr(el, 'rel') ?? '';
      const href = attr(el, 'href') ?? '';
      if (rel === 'canonical') meta.canonical = href;
      if (rel === 'alternate' && attr(el, 'type') === 'text/markdown') meta.markdownHref = href;
      const hreflang = attr(el, 'hrefLang');
      if (rel === 'alternate' && hreflang) meta.hreflang.push({ hreflang, href });
    }
  });
  if (/reading time/i.test(twitterLabel) && twitterData) meta.readingTime = twitterData;
  return meta;
}

/** Elements that are page chrome or interactive, never content. */
const DROP_TAGS = new Set([
  'script', 'style', 'noscript', 'template', 'svg', 'canvas', 'iframe', 'object', 'embed',
  'video', 'audio', 'nav', 'button', 'form', 'input', 'select', 'textarea', 'dialog', 'link',
  'meta',
]);

function isChrome(el: Element): boolean {
  if (DROP_TAGS.has(el.tagName)) return true;
  if (attr(el, 'ariaHidden') === 'true' || el.properties?.hidden) return true;
  if (attr(el, 'dataMdSkip') !== undefined) return true;
  return hasClass(el, 'skip-link');
}

const BLOCK_TAGS = new Set([
  'address', 'article', 'aside', 'blockquote', 'div', 'dl', 'fieldset', 'figure', 'footer',
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hr', 'li', 'ol', 'p', 'pre', 'section', 'table',
  'ul',
]);
const HEADING = /^h[1-6]$/;
const SAFE_PROTOCOLS = new Set(['http:', 'https:', 'mailto:']);
/** Site chrome directly under <body>, dropped when a page has no <main>. */
const BODY_CHROME = new Set(['header', 'footer', 'nav', 'aside']);

/**
 * A card link (`<a>` around a whole block) would become one link per line in
 * Markdown. Keep the link on the card's heading only, or drop it when the
 * card has none.
 */
function unwrapBlockLink(a: Element): ElementContent[] {
  let linked = false;
  const relink = (el: Element): Element => ({
    ...el,
    children: el.children.map((child) => {
      if (!isElement(child)) return child;
      if (!linked && HEADING.test(child.tagName)) {
        linked = true;
        return { ...child, children: [{ ...a, children: child.children }] };
      }
      return relink(child);
    }),
  });
  return relink({ ...a, children: a.children }).children;
}

function hasBlockChild(el: Element): boolean {
  return el.children.some(
    (child) => isElement(child) && (BLOCK_TAGS.has(child.tagName) || hasBlockChild(child)),
  );
}

/** A copy of `el` without chrome, with links and images made absolute. */
function clean(el: Element, base: URL, inHeading = HEADING.test(el.tagName)): Element {
  const children: ElementContent[] = [];
  for (const child of el.children) {
    if (isElement(child)) {
      if (isChrome(child)) continue;
      // A break inside a heading forces an underlined (setext) heading; a
      // space keeps it a one-line `#` heading.
      if (child.tagName === 'br' && inHeading) {
        children.push({ type: 'text', value: ' ' });
        continue;
      }
      const cleaned = clean(child, base, inHeading || HEADING.test(child.tagName));
      if (cleaned.tagName === 'a' && hasBlockChild(cleaned)) {
        children.push(...unwrapBlockLink(cleaned));
      } else {
        children.push(cleaned);
      }
    } else if (child.type !== 'comment') {
      children.push(child);
    }
  }
  const properties = { ...el.properties };
  for (const key of ['href', 'src'] as const) {
    const value = attr(el, key);
    if (!value) continue;
    // Judge the parsed URL, not the raw text: URL() strips leading spaces,
    // so " javascript:…" would otherwise slip through.
    let url: URL | undefined;
    try {
      url = new URL(value, base);
    } catch {
      url = undefined;
    }
    if (url && SAFE_PROTOCOLS.has(url.protocol)) properties[key] = url.href;
    else delete properties[key];
  }
  return { ...el, properties, children };
}

function yamlString(value: string): string {
  // JSON strings are valid YAML double-quoted scalars.
  return JSON.stringify(value);
}

export interface MarkdownOptions {
  /** Absolute canonical URL of the page. */
  url: string;
  /** Absolute URL of the Markdown twin. */
  markdownUrl: string;
  /** Title for the front matter and the fallback heading. */
  title: string;
  /** Extra front matter fields, written in order. */
  extra?: Record<string, string | undefined>;
}

/**
 * The page's content as a Markdown document with YAML front matter. Content
 * is the first `<main>` (or `<body>` when a page has none), without chrome.
 */
export async function pageToMarkdown(
  tree: Root,
  meta: PageMeta,
  options: MarkdownOptions,
): Promise<string> {
  const main = find(tree, (el) => el.tagName === 'main');
  const bodyElement = find(tree, (el) => el.tagName === 'body');
  // Without a <main>, use <body> minus its top-level site chrome.
  const root =
    main ??
    (bodyElement && {
      ...bodyElement,
      children: bodyElement.children.filter(
        (child) => !(isElement(child) && BODY_CHROME.has(child.tagName)),
      ),
    });
  const base = new URL(options.url);
  const content: Root = { type: 'root', children: root ? [clean(root, base)] : [] };

  const processor = unified()
    .use(rehypeRemark)
    .use(remarkGfm)
    .use(remarkStringify, {
      bullet: '-',
      fences: true,
      rule: '-',
      listItemIndent: 'one',
      setext: false,
    });
  const mdast = await processor.run(content);
  // Read the H1 from the tree: a `# comment` line in a code block is not one.
  const hasH1 = (mdast as { children: { type: string; depth?: number }[] }).children.some(
    (node) => node.type === 'heading' && node.depth === 1,
  );
  let body = String(processor.stringify(mdast as Parameters<typeof processor.stringify>[0]));
  body = body
    .replace(/ /g, ' ')
    .replace(/[ \t]+$/gm, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
  if (!hasH1) body = `# ${options.title}\n\n${body}`;

  const fields: Record<string, string | undefined> = {
    title: options.title,
    description: meta.description || undefined,
    url: options.url,
    markdown_url: options.markdownUrl,
    ...options.extra,
  };
  const frontMatter = Object.entries(fields)
    .filter((entry): entry is [string, string] => Boolean(entry[1]))
    .map(([key, value]) => `${key}: ${yamlString(value)}`)
    .join('\n');
  return `---\n${frontMatter}\n---\n\n${body}\n`;
}

/** The Markdown body without its front matter. */
export function stripFrontMatter(markdown: string): string {
  return markdown.replace(/^---\n[\s\S]*?\n---\n+/, '');
}

/**
 * /llms.txt and /llms-full.txt (https://llmstxt.org), built from the pages in
 * the build output: an index of every page's Markdown twin, grouped by
 * section, and every page's Markdown in one file.
 */
import { cardTitle, DEFAULT_LOCALE, SECTION_LABELS, sectionOf } from './routes';
import { stripFrontMatter } from './page';

export interface LlmsPage {
  /** Normalized path, e.g. `/docs/cli`. */
  path: string;
  /** Absolute canonical URL. */
  url: string;
  /** Absolute URL of the Markdown twin. */
  markdownUrl: string;
  title: string;
  description: string;
  locale: string;
  /** The Markdown twin, front matter included. */
  markdown: string;
}

const OTHER = 'Pages';

function byPathThenTitle(a: LlmsPage, b: LlmsPage): number {
  const depth = (p: LlmsPage) => p.path.split('/').length;
  return depth(a) - depth(b) || cardTitle(a.title).localeCompare(cardTitle(b.title));
}

function groupBySection(pages: LlmsPage[]): [string, LlmsPage[]][] {
  const order = [...Object.values(SECTION_LABELS), OTHER];
  const groups = new Map<string, LlmsPage[]>(order.map((label) => [label, []]));
  for (const page of pages) groups.get(sectionOf(page.path) ?? OTHER)?.push(page);
  return [...groups.entries()]
    .filter(([, list]) => list.length > 0)
    .map(([label, list]) => [label, [...list].sort(byPathThenTitle)]);
}

function entry(page: LlmsPage): string {
  const description = page.description.replace(/\s+/g, ' ').trim();
  return `- [${cardTitle(page.title)}](${page.markdownUrl})${description ? `: ${description}` : ''}`;
}

/**
 * The index. `header` is the hand-written top of the file (title, summary,
 * key facts); the page list below it is generated.
 */
export function buildLlmsTxt(header: string, pages: LlmsPage[], site: URL): string {
  const primary = pages.filter((page) => page.locale === DEFAULT_LOCALE);
  const translated = pages.filter((page) => page.locale !== DEFAULT_LOCALE);
  const origin = site.origin;
  const parts = [
    header.trim(),
    [
      '## How to read this site',
      '',
      `- Every page has a Markdown twin: add \`.md\` to its path (\`${origin}/docs/cli\` → \`${origin}/docs/cli.md\`). The home page is \`${origin}/index.md\`.`,
      '- A request with `Accept: text/markdown` to any page URL gets the Markdown twin.',
      `- \`${origin}/llms-full.txt\` holds every page below in one file.`,
      `- Each page links its twin with \`<link rel="alternate" type="text/markdown">\`, and \`${origin}/sitemap-index.xml\` lists every page.`,
    ].join('\n'),
    ...groupBySection(primary).map(
      ([label, list]) => `## ${label}\n\n${list.map(entry).join('\n')}`,
    ),
  ];
  if (translated.length > 0) {
    const list = [...translated].sort((a, b) => a.path.localeCompare(b.path));
    parts.push(
      `## Optional\n\n${list.map((page) => `- [${cardTitle(page.title)} (${page.locale})](${page.markdownUrl})`).join('\n')}`,
    );
  }
  return `${parts.join('\n\n')}\n`;
}

/** Every primary-locale page's Markdown, in index order, each under its source URL. */
export function buildLlmsFullTxt(header: string, pages: LlmsPage[]): string {
  const primary = pages.filter((page) => page.locale === DEFAULT_LOCALE);
  const sections = groupBySection(primary).flatMap(([, list]) => list);
  const bodies = sections.map((page) => {
    const body = stripFrontMatter(page.markdown).trim();
    // Put the source right under the page's own H1.
    const withSource = body.replace(/^(# .*)$/m, `$1\n\nSource: ${page.url}`);
    return withSource === body ? `Source: ${page.url}\n\n${body}` : withSource;
  });
  return `${[header.trim(), ...bodies].join('\n\n---\n\n')}\n`;
}

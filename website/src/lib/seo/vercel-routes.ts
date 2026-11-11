/**
 * Vercel routes for the Markdown twins (Build Output API v3, config.json).
 *
 * - Content negotiation: a page request with `Accept: text/markdown` is
 *   rewritten to the page's `.md` twin.
 * - Every page answers with `Link: <…md>; rel="alternate"; type="text/markdown"`
 *   and `Vary: Accept`, so agents find the twin from headers alone and caches
 *   keep the HTML and Markdown answers apart.
 * - Every twin answers as `text/markdown` with a canonical `Link` to its page,
 *   so search engines fold it into the HTML page.
 *
 * The routes go into the adapter's own .vercel/output/config.json, ahead of
 * the filesystem handler. They are scoped to this build, so the Storybook
 * project, which shares the `website/` root, is not affected.
 */
import { routeKey } from './routes';

export interface VercelRoute {
  src?: string;
  dest?: string;
  headers?: Record<string, string>;
  has?: { type: 'header'; key: string; value?: string }[];
  missing?: { type: 'header'; key: string; value?: string }[];
  continue?: boolean;
  check?: boolean;
  handle?: string;
  status?: number;
}

const ACCEPTS_MARKDOWN = [{ type: 'header' as const, key: 'accept', value: '.*text/markdown.*' }];
const MARKDOWN_TYPE = 'text/markdown; charset=utf-8';

function escape(segment: string): string {
  return segment.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Routes for `paths` (normalized page paths) on `site`. Header routes come
 * first and `continue`; the negotiation rewrite comes last, since a rewrite
 * ends route matching.
 */
export function markdownRoutes(paths: string[], site: URL): VercelRoute[] {
  const keys = [...new Set(paths.map(routeKey))].filter((key) => key !== 'index').sort();
  const hasRoot = paths.some((path) => routeKey(path) === 'index');
  const routes: VercelRoute[] = [];

  // [page pattern, Markdown twin, canonical page URL] for the root and the rest.
  const groups: [string, string, string][] = [];
  if (hasRoot) groups.push(['', '/index.md', new URL('/', site).href]);
  if (keys.length > 0) {
    groups.push([`(?<page>${keys.map(escape).join('|')})`, '/$page.md', `${site.origin}/$page`]);
  }

  for (const [page, twin, canonical] of groups) {
    const pageSrc = page ? `^/${page}/?$` : '^/$';
    const twinSrc = page ? `^/${page}\\.md$` : '^/index\\.md$';
    const twinHeaders = {
      'content-type': MARKDOWN_TYPE,
      link: `<${canonical}>; rel="canonical"`,
      vary: 'Accept',
    };
    routes.push(
      {
        src: pageSrc,
        missing: ACCEPTS_MARKDOWN,
        headers: { link: `<${twin}>; rel="alternate"; type="text/markdown"`, vary: 'Accept' },
        continue: true,
      },
      { src: twinSrc, headers: twinHeaders, continue: true },
      { src: pageSrc, has: ACCEPTS_MARKDOWN, dest: twin, headers: twinHeaders, check: true },
    );
  }
  return routes;
}

/**
 * `config` with `routes` placed ahead of the filesystem handler, replacing
 * routes an earlier run inserted (marked by their `has` or `.md` sources).
 */
export function withMarkdownRoutes<T extends { routes?: VercelRoute[] }>(
  config: T,
  routes: VercelRoute[],
): T {
  const existing = (config.routes ?? []).filter((route) => !isMarkdownRoute(route));
  const at = existing.findIndex((route) => route.handle === 'filesystem');
  const index = at === -1 ? 0 : at;
  return { ...config, routes: [...existing.slice(0, index), ...routes, ...existing.slice(index)] };
}

function isMarkdownRoute(route: VercelRoute): boolean {
  return Boolean(
    route.headers?.link?.includes('text/markdown') ||
      route.headers?.['content-type'] === MARKDOWN_TYPE ||
      route.has?.some((h) => h.key === 'accept' && h.value?.includes('text/markdown')),
  );
}

import { describe, expect, it } from 'vitest';
import { markdownRoutes, type VercelRoute, withMarkdownRoutes } from './vercel-routes';

const site = new URL('https://caro.sh');
const routes = markdownRoutes(['/', '/docs/cli', '/docs', '/es'], site);

/** What Vercel's router would do for `path` with these routes, in order. */
function resolve(path: string, accept = 'text/html') {
  const headers: Record<string, string> = {};
  for (const route of routes) {
    const match = route.src ? new RegExp(route.src).exec(path) : null;
    if (!match) continue;
    const accepts = (h: VercelRoute['has']) => h?.every((c) => new RegExp(`^${c.value}$`).test(accept)) ?? true;
    if (route.has && !accepts(route.has)) continue;
    if (route.missing && accepts(route.missing)) continue;
    const fill = (value: string) => value.replace('$page', match.groups?.page ?? '');
    for (const [key, value] of Object.entries(route.headers ?? {})) headers[key] = fill(value);
    if (route.dest) return { path: fill(route.dest), headers };
    if (!route.continue) break;
  }
  return { path, headers };
}

describe('markdownRoutes', () => {
  it('serves the Markdown twin to Accept: text/markdown', () => {
    expect(resolve('/docs/cli', 'text/markdown').path).toBe('/docs/cli.md');
    expect(resolve('/docs/cli/', 'text/markdown, text/html;q=0.9').path).toBe('/docs/cli.md');
    expect(resolve('/', 'text/markdown').path).toBe('/index.md');
    const { headers } = resolve('/docs', 'text/markdown');
    expect(headers['content-type']).toBe('text/markdown; charset=utf-8');
    expect(headers.link).toBe('<https://caro.sh/docs>; rel="canonical"');
  });

  it('serves HTML otherwise, with a Link to the twin', () => {
    const { path, headers } = resolve('/docs/cli');
    expect(path).toBe('/docs/cli');
    expect(headers.link).toBe('</docs/cli.md>; rel="alternate"; type="text/markdown"');
    expect(headers.vary).toBe('Accept');
    expect(resolve('/').headers.link).toBe('</index.md>; rel="alternate"; type="text/markdown"');
  });

  it('marks each twin as Markdown with a canonical link to its page', () => {
    const { headers } = resolve('/docs/cli.md');
    expect(headers['content-type']).toBe('text/markdown; charset=utf-8');
    expect(headers.link).toBe('<https://caro.sh/docs/cli>; rel="canonical"');
    expect(resolve('/index.md').headers.link).toBe('<https://caro.sh/>; rel="canonical"');
  });

  it('leaves other paths alone', () => {
    for (const path of ['/docs/clix', '/api/waitlist', '/og/docs/cli.png', '/docs/cli/extra']) {
      expect(resolve(path, 'text/markdown')).toEqual({ path, headers: {} });
    }
  });
});

describe('withMarkdownRoutes', () => {
  const adapter = { version: 3, routes: [{ handle: 'filesystem' }, { src: '^/.*$', dest: '_render', status: 404 }] };

  it('puts the routes ahead of the filesystem handler', () => {
    const config = withMarkdownRoutes(adapter, routes);
    expect(config.routes.slice(0, routes.length)).toEqual(routes);
    expect(config.routes[routes.length]).toEqual({ handle: 'filesystem' });
    expect(config.version).toBe(3);
  });

  it('replaces its own routes when it runs again', () => {
    const twice = withMarkdownRoutes(withMarkdownRoutes(adapter, routes), routes);
    expect(twice.routes).toHaveLength(adapter.routes.length + routes.length);
  });
});

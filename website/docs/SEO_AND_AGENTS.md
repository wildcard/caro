# SEO, share cards and agent access

Every page on caro.sh ships with a share card, a Markdown twin and a full
SEO head. Nothing is registered per page: a new page gets all of it from its
path and its `<title>`.

```
 page.astro ─▶ Layout / DocsLayout / LandingPage ─▶ <SEO>          (render time)
                                                     │ points at
                                                     ▼
 astro build ─▶ dist/client/**/index.html ─▶ caro:agent-seo        (after the build)
                                              ├─ og/<page>.png      share card
                                              ├─ <page>.md          Markdown twin
                                              ├─ llms.txt, llms-full.txt
                                              ├─ rss.xml
                                              ├─ verify: fail the build on any gap
                                              └─ .vercel/output/config.json routes
```

`<SEO>` and the build step derive paths from the same helpers in
`src/lib/seo/routes.ts`, so the head always points at files that exist.

## What a page gets

| Item | Where | Source |
|---|---|---|
| `<title>`, description, canonical, robots | `<head>` | `src/components/SEO.astro` |
| Open Graph and X/Twitter tags | `<head>` | `SEO.astro` |
| Share card, 1200 × 630 PNG | `/og/<page>.png` (`/og/index.png` for `/`) | `src/lib/seo/og-card.ts` |
| Markdown twin | `/<page>.md` (`/index.md` for `/`) | `src/lib/seo/page.ts` |
| `<link rel="alternate" type="text/markdown">` | `<head>` | `SEO.astro` |
| JSON-LD: Organization, WebSite, WebPage, BreadcrumbList; SoftwareApplication on `/`; BlogPosting on posts | `<head>` | `SEO.astro` |
| hreflang | `<head>`, only on paths in `LOCALIZED_PATHS` | `routes.ts` |

Site-wide files: `/llms.txt` (index of every twin), `/llms-full.txt` (every
twin in one file), `/rss.xml` (blog posts), `/robots.txt` (open to all
crawlers and named AI agents), `/sitemap-index.xml`. `/llm.txt` and
`/llm-full.txt` are aliases kept for old links.

## Agent access

- Add `.md` to any page path: `https://caro.sh/docs/cli.md`.
- Or send `Accept: text/markdown` to the page URL. Vercel rewrites the
  request to the twin. Each HTML page answers with
  `Link: </docs/cli.md>; rel="alternate"; type="text/markdown"` and
  `Vary: Accept`. Each twin answers as `text/markdown` with a canonical
  `Link` to its HTML page, so search engines fold the twin into the page.
- The routes go into the adapter's `.vercel/output/config.json`
  (`src/lib/seo/vercel-routes.ts`), not a `vercel.json`, so the Storybook
  project that shares `website/` is not affected.

Try it on a deployment:

```bash
curl -s https://caro.sh/docs/cli.md | head
curl -s -H 'Accept: text/markdown' https://caro.sh/docs/cli | head
curl -sI https://caro.sh/docs/cli | grep -i '^link'
```

## How to…

**Change a page's card text.** The card prints the page title without the
site name (`cardTitle` in `routes.ts`). Change the page's `title`.

**Use a custom image for one page.** Pass `image="/my-image.png"` (1200 × 630)
to the layout. The build then skips that page's card.

**Keep a block out of the Markdown twin.** Add `data-md-skip` to it. Nav,
scripts, styles, buttons, forms, SVG and `aria-hidden` elements are left out
already.

**Translate a page for real.** Add its path to `LOCALIZED_PATHS` in
`routes.ts`. The head then emits hreflang for it, and the sitemap lists its
locale URLs. The build fails if any of those URLs is still an i18n fallback
redirect.

**Edit the top of llms.txt.** Edit `src/lib/seo/llms-header.md`. `{{version}}`
becomes the version in `Cargo.toml`. The page list below it is generated.

## Checks

- `npm test` runs the unit tests in `src/lib/seo/*.test.ts` and
  `src/__tests__/seo-coverage.test.ts` (every layout that renders `<html>`
  must render `<SEO>`).
- `astro build` fails when a page has no title, description, canonical URL,
  generated share card or Markdown twin, or has an hreflang alternate that is a
  redirect stub.
- CI: the `Build website (caro.sh)` job in
  `.github/workflows/static-sites-build.yml` runs both.

The share card system follows the one on kobi.kadosh.me (`og-text.ts` is
ported from it): text outlined to SVG paths with opentype.js, rasterized with
sharp, so a card renders the same on any build machine.

## Known limits

- Card fonts cover Latin only. Locale homepages in Hebrew, Arabic, Urdu,
  Russian, Ukrainian, Japanese, Korean and Hindi use the English page's card
  text.
- The dev server (`npm run dev`) does not serve cards or twins. They exist
  only after `astro build`.

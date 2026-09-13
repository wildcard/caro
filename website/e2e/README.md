# caro.sh structural QA (Kitesurf)

Nightly structural checks of the production site, driven through
[Cloudflare Browser Run's Kitesurf engine](https://developers.cloudflare.com/browser-run/kitesurf/)
over CDP — Playwright connects to a remote agent-first browser, so CI needs
**no local browser install** and the Kitesurf lane is free while in beta.
Part of ADR-017 phase P3; dormant until CF secrets exist.

Suites (structural only — Kitesurf is not pixel-perfect; brand/visual audits
stay on the claude-design Chromium flow):

- `i18n-leaks.spec.ts` — raw translation keys visible on any key page or
  locale home (the documented L1 regression class).
- `links.spec.ts` — internal links on key pages resolve.
- `structure.spec.ts` — nav/footer/h1 present; `lang`/RTL plumbing wired.
- `screenshot-smoke.spec.ts` — page rasterizes to a non-blank frame.

## Run

```bash
cd website/e2e && npm ci
CARO_CF_ACCOUNT_ID=… CARO_CF_API_TOKEN=… npm test
# real Chromium via the same endpoint (paid; manual runs only):
BROWSER_MODE=chromium CARO_CF_ACCOUNT_ID=… CARO_CF_API_TOKEN=… npm test
# against a preview deploy:
CARO_QA_BASE_URL=https://deploy-preview.example npm test
npm run list   # validates config with no credentials
```

CI: `.github/workflows/website-qa-kitesurf.yml` (nightly, non-blocking,
skips green without secrets).

This package is deliberately separate from `website/package.json` so Vercel
production installs are unaffected.

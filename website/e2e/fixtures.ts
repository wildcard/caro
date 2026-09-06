// Shared fixture: routes every test's browser through Cloudflare Browser Run
// over CDP. Default engine is Kitesurf (free beta); BROWSER_MODE=chromium
// drops the query parameter and the exact same endpoint serves real Chromium
// (paid) — the trivial fallback ADR-017 relies on.
import { test as base, chromium, type Browser } from "@playwright/test";

export function credentials(): { accountId: string; apiToken: string } | null {
  const accountId = process.env.CARO_CF_ACCOUNT_ID;
  const apiToken = process.env.CARO_CF_API_TOKEN;
  return accountId && apiToken ? { accountId, apiToken } : null;
}

export const laneDormant =
  "Kitesurf lane dormant: CARO_CF_ACCOUNT_ID / CARO_CF_API_TOKEN not set (see ADR-017 P3)";

function wsEndpoint(accountId: string): string {
  const engine = process.env.BROWSER_MODE === "chromium" ? "" : "?browser=kitesurf";
  return `wss://api.cloudflare.com/client/v4/accounts/${accountId}/browser-run/devtools/browser${engine}`;
}

export const test = base.extend<object, { browser: Browser }>({
  browser: [
    // eslint-disable-next-line no-empty-pattern
    async ({}, use) => {
      const creds = credentials();
      if (!creds) throw new Error(laneDormant);
      const browser = await chromium.connectOverCDP(wsEndpoint(creds.accountId), {
        headers: { Authorization: `Bearer ${creds.apiToken}` },
        timeout: 30_000,
      });
      await use(browser);
      await browser.close();
    },
    { scope: "worker" },
  ],
});

export const expect = test.expect;

/** Key routes checked in every suite (EN lives at the root). */
export const KEY_ROUTES = [
  "/",
  "/pricing",
  "/faq",
  "/roadmap",
  "/try-caro",
  "/telemetry",
  "/glossary",
];

/** Non-default locales served under /[lang]/ (mirror of src/i18n/locales/). */
export const LOCALES = [
  "ar",
  "de",
  "es",
  "fil",
  "fr",
  "he",
  "hi",
  "id",
  "ja",
  "ko",
  "pt",
  "ru",
  "uk",
  "ur",
];

// Raw-i18n-key leak scan — the documented L1 regression class: a missing EN
// key makes EVERY locale render the literal key path (e.g.
// "landing.hero.headline.line1" visible on the homepage). See
// .claude/rules/astro-esbuild-shell-syntax.md "Lessons Learned" L1.
//
// The namespace list is read from src/i18n/locales/en/*.json at runtime, so
// new translation namespaces are covered without touching this file.
import { readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { test, expect, credentials, laneDormant, KEY_ROUTES, LOCALES } from "./fixtures";

test.skip(() => !credentials(), laneDormant);

const localesDir = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "i18n", "locales", "en");
const namespaces = readdirSync(localesDir)
  .filter((f) => f.endsWith(".json"))
  .map((f) => f.replace(/\.json$/, ""));

// e.g. "landing.hero.headline.line1" — a known namespace followed by ≥2
// dotted segments. Anchored to word boundaries so prose and URLs don't match.
const leakPattern = new RegExp(
  `\\b(?:${namespaces.join("|")})(?:\\.[A-Za-z0-9_-]+){2,}\\b`,
  "g",
);

const pages: { route: string; label: string }[] = [
  ...KEY_ROUTES.map((route) => ({ route, label: `en ${route}` })),
  ...LOCALES.map((lang) => ({ route: `/${lang}/`, label: `${lang} home` })),
];

for (const { route, label } of pages) {
  test(`no raw translation keys visible: ${label}`, async ({ page }) => {
    const response = await page.goto(route, { waitUntil: "domcontentloaded" });
    expect(response, `no response for ${route}`).toBeTruthy();
    expect(response!.status(), `${route} should be served`).toBeLessThan(400);

    const visibleText = await page.evaluate(() => document.body?.innerText ?? "");
    const leaks = [...new Set(visibleText.match(leakPattern) ?? [])];
    expect(leaks, `raw i18n keys rendered on ${route}: ${leaks.join(", ")}`).toEqual([]);
  });
}

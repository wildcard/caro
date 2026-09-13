// Internal link integrity: collect same-origin hrefs from key pages via the
// Kitesurf DOM, then verify each target serves (status < 400). Status checks
// go straight from the runner (plain fetch) — only DOM extraction needs the
// browser engine.
import { test, expect, credentials, laneDormant, KEY_ROUTES } from "./fixtures";

test.skip(() => !credentials(), laneDormant);

test("internal links on key pages resolve", async ({ page, baseURL }) => {
  test.setTimeout(240_000);
  const origin = new URL(baseURL!).origin;
  const targets = new Map<string, string>(); // url -> first page referencing it

  for (const route of KEY_ROUTES) {
    await page.goto(route, { waitUntil: "domcontentloaded" });
    const hrefs = await page.evaluate(() =>
      Array.from(document.querySelectorAll("a[href]"), (a) => (a as HTMLAnchorElement).href),
    );
    for (const href of hrefs) {
      try {
        const url = new URL(href);
        if (url.origin !== origin) continue; // external links are out of scope
        url.hash = "";
        if (!targets.has(url.toString())) targets.set(url.toString(), route);
      } catch {
        // ignore unparseable hrefs (mailto:, javascript:)
      }
    }
  }

  expect(targets.size, "expected to discover internal links").toBeGreaterThan(5);

  const broken: string[] = [];
  for (const [url, referrer] of targets) {
    const res = await fetch(url, { method: "GET", redirect: "follow" });
    if (res.status >= 400) broken.push(`${url} -> ${res.status} (linked from ${referrer})`);
  }
  expect(broken, `broken internal links:\n${broken.join("\n")}`).toEqual([]);
});

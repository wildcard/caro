// Structural DOM assertions: the page skeleton users depend on exists and the
// locale plumbing is wired (lang / dir attributes). Deliberately not visual —
// Kitesurf rendering is not pixel-perfect and brand audits stay on Chromium.
import { test, expect, credentials, laneDormant } from "./fixtures";

test.skip(() => !credentials(), laneDormant);

test("homepage has nav, footer, and a headline", async ({ page }) => {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.locator("nav").first()).toBeAttached();
  await expect(page.locator("footer").first()).toBeAttached();
  const h1 = (await page.locator("h1").first().textContent())?.trim() ?? "";
  expect(h1.length, "h1 should carry a real headline").toBeGreaterThan(3);
});

test("html lang attribute matches the served locale", async ({ page }) => {
  await page.goto("/es/", { waitUntil: "domcontentloaded" });
  const lang = await page.evaluate(() => document.documentElement.lang);
  expect(lang.toLowerCase()).toContain("es");
});

test("hebrew locale is right-to-left", async ({ page }) => {
  await page.goto("/he/", { waitUntil: "domcontentloaded" });
  const dir = await page.evaluate(
    () => document.documentElement.dir || getComputedStyle(document.documentElement).direction,
  );
  expect(dir).toBe("rtl");
});

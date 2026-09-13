// Renders-at-all screenshot smoke: proves Kitesurf can rasterize the page and
// the page isn't blank. NOT a visual-regression check — no pixel comparisons
// here, ever (Kitesurf is explicitly not pixel-perfect; ADR-017 P3).
import { mkdirSync, writeFileSync } from "node:fs";
import { test, expect, credentials, laneDormant } from "./fixtures";

test.skip(() => !credentials(), laneDormant);

test("homepage rasterizes to a non-trivial screenshot", async ({ page }) => {
  await page.goto("/", { waitUntil: "load" });
  const shot = await page.screenshot({ type: "png" });
  // Save the frame BEFORE asserting, so a blank frame — the exact failure this
  // test exists to catch — is preserved on disk for debugging.
  mkdirSync("test-results", { recursive: true });
  writeFileSync("test-results/homepage-kitesurf.png", shot);
  // A blank/failed frame compresses to almost nothing.
  expect(shot.byteLength, "screenshot suspiciously small — blank frame?").toBeGreaterThan(10_000);
});

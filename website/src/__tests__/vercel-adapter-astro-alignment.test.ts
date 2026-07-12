/**
 * Vercel Adapter ↔ Astro Major Alignment — Regression Guard
 *
 * Prevents the deploy-breaking bug fixed alongside the Flox use-case PR
 * (#1308 / issue #1309): the site ran astro 6 but pinned
 * `@astrojs/vercel@^9`, whose astro peer is `^5.0.0`. npm nested an
 * astro-5 copy under the adapter; the adapter's serverless polyfill then
 * imported `applyPolyfills` (astro-6 only) against astro 5 and every
 * Vercel deploy crashed:
 *
 *   "applyPolyfills" is not exported by astro/dist/.../node.js
 *
 * The `@astrojs/vercel` major tracks the Astro major one-to-one:
 *   astro 5 → @astrojs/vercel 9   (peer astro ^5)
 *   astro 6 → @astrojs/vercel 10  (peer astro ^6)
 *   astro 7 → @astrojs/vercel 11  (peer astro ^7)
 *
 * If someone bumps astro without bumping the adapter (or vice versa),
 * this test fails locally/in CI instead of only surfacing as a red
 * Vercel deploy after merge.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const WEBSITE_ROOT = resolve(HERE, '..', '..'); // website/

/** Extract the leading major integer from a semver range like "^10.0.8". */
function majorOf(range: string): number {
  const m = range.match(/(\d+)/);
  expect(m, `could not parse a major from "${range}"`).not.toBeNull();
  return Number(m![1]);
}

// astro major -> required @astrojs/vercel major (from the adapter's astro peer)
const ADAPTER_FOR_ASTRO: Record<number, number> = {
  5: 9,
  6: 10,
  7: 11,
};

describe('Vercel adapter ↔ Astro major alignment', () => {
  const pkg = JSON.parse(
    readFileSync(resolve(WEBSITE_ROOT, 'package.json'), 'utf-8'),
  );
  const astroRange: string =
    pkg.dependencies?.astro ?? pkg.devDependencies?.astro;
  const adapterRange: string =
    pkg.dependencies?.['@astrojs/vercel'] ??
    pkg.devDependencies?.['@astrojs/vercel'];

  it('declares both astro and the vercel adapter', () => {
    expect(astroRange, 'website must depend on astro').toBeTruthy();
    expect(
      adapterRange,
      'website must depend on @astrojs/vercel',
    ).toBeTruthy();
  });

  it('pins an adapter major compatible with the astro major', () => {
    const astroMajor = majorOf(astroRange);
    const adapterMajor = majorOf(adapterRange);
    const expected = ADAPTER_FOR_ASTRO[astroMajor];
    expect(
      expected,
      `no known @astrojs/vercel major mapped for astro ${astroMajor}; ` +
        `update ADAPTER_FOR_ASTRO in this test when adopting a new astro major`,
    ).toBeDefined();
    expect(
      adapterMajor,
      `astro ${astroMajor} needs @astrojs/vercel ${expected}, but ` +
        `package.json pins ${adapterMajor} (${adapterRange}). ` +
        `Mismatch reintroduces the applyPolyfills deploy crash (issue #1309).`,
    ).toBe(expected);
  });

  it('the installed adapter, if present, peers the declared astro major', () => {
    // In CI/local with node_modules installed, cross-check against the
    // adapter's actual declared peer range rather than the static map.
    const adapterPkgPath = resolve(
      WEBSITE_ROOT,
      'node_modules/@astrojs/vercel/package.json',
    );
    if (!existsSync(adapterPkgPath)) return; // no install — static map covers it
    const adapterPkg = JSON.parse(readFileSync(adapterPkgPath, 'utf-8'));
    const peer: string | undefined = adapterPkg.peerDependencies?.astro;
    expect(peer, 'adapter must declare an astro peer').toBeTruthy();
    expect(
      majorOf(peer!),
      `installed @astrojs/vercel@${adapterPkg.version} peers astro ${peer}, ` +
        `but website declares astro ${astroRange}`,
    ).toBe(majorOf(astroRange));
  });
});

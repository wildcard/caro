/**
 * Regression guard: competitor comparison claims stay sourced and scoped.
 *
 * PR #1583 corrected the comparison pages against vendor docs. These checks
 * keep the retired claims from coming back:
 * - Retired phrases must not appear in the comparison sources or in the
 *   search index (the index generator also reads quoted strings in comments).
 * - A cell without a primary source is 'unknown' ("Not documented"), never a
 *   negative, and an 'unknown' row picks no winner. The expected unknown
 *   cells are pinned below (from the source ledger), so turning one into a
 *   negative fails the test instead of silently leaving the filter.
 * - Each vs page carries a dated verification block with its sources.
 * - Every locale has the comparison legend and value labels that English has.
 */
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const src = fileURLToPath(new URL('..', import.meta.url));
const read = (path: string) => readFileSync(join(src, path), 'utf8');

const vsPages = ['warp', 'github-copilot-cli', 'kiro-cli', 'opencode'].map(
  (name) => `pages/compare/${name}.astro`,
);
const claimSources = [
  ...vsPages,
  'pages/compare/index.astro',
  'pages/use-cases/index.astro',
  'components/Comparison.astro',
  'components/gtm/GTMComparison.astro',
  'data/gtm-comparison.ts',
  'data/compare-pages.ts',
];

// Cells with no primary source (docs/research/2026-10-11-competitive-refresh/
// source-ledger.md, status "unknown"). Each must stay 'unknown' until a source
// exists; then update the ledger and this list together.
const unknownOnVsPage: Record<string, string[]> = {
  'github-copilot-cli': ['Built-in dangerous-command patterns', 'Risk level for each command'],
  'kiro-cli': ['Built-in dangerous-command patterns', 'Risk level for each command'],
  opencode: ['Risk level for each command'],
  warp: ['Risk level for each command'],
};
const unknownOnHub: Record<string, string[]> = {
  'Built-in dangerous-command patterns': ['copilot', 'kiro'],
  'Risk level for each command': ['copilot', 'warp', 'kiro', 'opencode'],
  'GNU vs BSD syntax awareness': ['warp'],
};
const unknownOnLanding: Record<string, string[]> = {
  riskLevel: ['copilot-cli', 'warp', 'kiro', 'opencode'],
  crossPlatform: ['copilot-cli', 'kiro', 'opencode'],
  platformAware: ['warp'],
};

// Row parsers for the object literals in the page sources. They read fields by
// name, so spacing, line breaks and key order do not matter.
const unquote = (value: string) => value.trim().replace(/^(['"])(.*)\1$/, '$2');
function field(object: string, key: string): string | undefined {
  const match = object.match(new RegExp(`(?:^|[{,\\s])['"]?${key}['"]?\\s*:\\s*('[^']*'|"[^"]*"|[\\w-]+)`));
  return match ? unquote(match[1]) : undefined;
}
function rows(text: string, nameKey: string): Map<string, string> {
  const found = new Map<string, string>();
  for (const [object] of text.matchAll(/\{[^{}]*\}/g)) {
    const name = field(object, nameKey);
    if (name) found.set(name, object);
  }
  return found;
}
function landingRows(text: string): Map<string, string> {
  const found = new Map<string, string>();
  for (const match of text.matchAll(/featureKey:\s*['"](\w+)['"]\s*,\s*values:\s*(\{[^{}]*\})/g)) {
    found.set(match[1], match[2]);
  }
  return found;
}

// Claims retired in PR #1583, with the reason each one is wrong.
const retired = [
  ['every Warp AI request', 'Warp routing is documented only for the personal endpoint and BYOK flows'],
  ['CRITICAL, HIGH, MEDIUM', 'RiskLevel is Safe, Moderate, High, Critical'],
  ['only AI shell tool', 'no exclusivity claims'],
  ['nothing else can', 'no exclusivity claims'],
  ['checks every command against', 'the validator checks the commands Caro generates'],
  ['Zero Telemetry', 'telemetry is opt-in, not absent'],
  ['free tier with 50 credits', 'Kiro Free-tier CLI coverage is not documented'],
];

describe('comparison claims', () => {
  it.each(claimSources)('%s has no retired claim', (file) => {
    const text = read(file);
    const found = retired.filter(([phrase]) => text.includes(phrase));
    expect(found, file).toEqual([]);
  });

  it('the search index has no retired Warp routing claim', () => {
    expect(read('config/search-index.json')).not.toContain('every Warp AI request');
  });

  it.each(Object.entries(unknownOnVsPage))('%s: undocumented rows stay unknown with no winner', (page, features) => {
    const parsed = rows(read(`pages/compare/${page}.astro`), 'feature');
    expect(parsed.size, 'row parser self-check').toBeGreaterThan(5);
    for (const feature of features) {
      const row = parsed.get(feature);
      expect(row, `${page}: row "${feature}" is missing`).toBeDefined();
      expect(field(row!, 'competitor'), `${page}: ${feature}`).toBe('unknown');
      expect(field(row!, 'winner'), `${page}: ${feature}`).toBe('none');
    }
  });

  it.each(vsPages)('%s: every unknown competitor cell picks no winner', (file) => {
    for (const row of rows(read(file), 'feature').values()) {
      if (field(row, 'competitor') === 'unknown') expect(field(row, 'winner'), file).toBe('none');
    }
  });

  it('the /compare hub keeps undocumented cells unknown', () => {
    const parsed = rows(read('pages/compare/index.astro'), 'name');
    for (const [feature, tools] of Object.entries(unknownOnHub)) {
      const row = parsed.get(feature);
      expect(row, `hub: row "${feature}" is missing`).toBeDefined();
      for (const tool of tools) expect(field(row!, tool), `hub: ${feature} / ${tool}`).toBe('unknown');
    }
  });

  it('the landing matrix keeps undocumented cells unknown', () => {
    const parsed = landingRows(read('components/Comparison.astro'));
    for (const [featureKey, tools] of Object.entries(unknownOnLanding)) {
      const values = parsed.get(featureKey);
      expect(values, `landing: row "${featureKey}" is missing`).toBeDefined();
      for (const tool of tools) expect(field(values!, tool), `landing: ${featureKey} / ${tool}`).toBe('unknown');
    }
  });

  it.each(vsPages)('%s has a dated verification block with sources', (file) => {
    const text = read(file);
    expect(text).toMatch(/verification:\s*\{\s*date:\s*'\d{4}-\d{2}-\d{2}'/);
    expect(text.slice(text.indexOf('verification:'))).toMatch(/https:\/\//);
  });
});

describe('comparison labels in every locale', () => {
  type Labels = { legend: Record<string, string>; values: Record<string, string> };
  const labels = (locale: string): Labels =>
    JSON.parse(read(`i18n/locales/${locale}/landing.json`)).landing.comparison;
  const english = labels('en');
  const locales = readdirSync(join(src, 'i18n/locales')).filter((locale) => locale !== 'en');

  it('English has the "Not documented" labels', () => {
    expect(english.values.unknown).toBeTruthy();
    expect(english.legend.notDocumented).toBeTruthy();
  });

  it.each(locales)('%s has the same legend and value keys as English', (locale) => {
    const own = labels(locale);
    expect(Object.keys(own.legend).sort()).toEqual(Object.keys(english.legend).sort());
    expect(Object.keys(own.values).sort()).toEqual(Object.keys(english.values).sort());
  });
});

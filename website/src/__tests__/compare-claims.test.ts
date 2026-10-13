/**
 * Regression guard: competitor comparison claims stay sourced and scoped.
 *
 * PR #1583 corrected the comparison pages against vendor docs. These checks
 * keep the retired claims from coming back:
 * - Retired phrases must not appear in the comparison sources or in the
 *   search index (the index generator also reads quoted strings in comments).
 * - A cell without a primary source is 'unknown' ("Not documented"), never a
 *   negative, and an 'unknown' row picks no winner.
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

  it.each(vsPages)('%s: an unknown competitor cell picks no winner', (file) => {
    const rows = read(file)
      .split('\n')
      .filter((line) => line.includes("competitor: 'unknown'"));
    for (const row of rows) expect(row, file).toContain("winner: 'none'");
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

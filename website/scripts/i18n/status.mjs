#!/usr/bin/env node
/**
 * Translation Status Report
 *
 * Generates a coverage report for all supported locales.
 * Shows how many keys are translated vs total keys.
 *
 * Usage:
 *   node scripts/i18n/status.mjs
 *   node scripts/i18n/status.mjs --ci              # CI mode (exit code 1 on low coverage)
 *   node scripts/i18n/status.mjs --min-coverage 80 # Set minimum coverage threshold
 *   node scripts/i18n/status.mjs --tier1-only      # Only check tier 1 locales
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const LOCALES_DIR = path.join(__dirname, '../../src/i18n/locales');
const LOCALES = ['es', 'fr', 'pt', 'de', 'ja', 'ko', 'he', 'ar', 'ru', 'uk', 'hi', 'ur', 'fil', 'id'];
const TIER1_LOCALES = ['es', 'fr', 'de', 'pt', 'ja'];

// Parse CLI args
const args = process.argv.slice(2);
const ciMode = args.includes('--ci');
const minCoverage = args.includes('--min-coverage')
  ? parseInt(args[args.indexOf('--min-coverage') + 1], 10)
  : 0;
const tier1Only = args.includes('--tier1-only');

/**
 * Recursively flatten nested JSON object into dot-notation keys
 */
function flattenKeys(obj, prefix = '') {
  const result = {};

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      Object.assign(result, flattenKeys(value, fullKey));
    } else {
      result[fullKey] = value;
    }
  }

  return result;
}

/**
 * Load and merge all JSON files for a locale
 */
function loadLocaleTranslations(locale) {
  const localeDir = path.join(LOCALES_DIR, locale);
  if (!fs.existsSync(localeDir)) {
    return {};
  }

  const merged = {};
  const files = fs.readdirSync(localeDir).filter(f => f.endsWith('.json'));

  for (const file of files) {
    try {
      const content = JSON.parse(fs.readFileSync(path.join(localeDir, file), 'utf8'));
      Object.assign(merged, content);
    } catch (error) {
      console.error(`Error reading ${locale}/${file}:`, error.message);
    }
  }

  return merged;
}

/**
 * Calculate translation coverage for a locale
 */
function calculateCoverage(enTranslations, localeTranslations) {
  const enKeys = flattenKeys(enTranslations);
  const localeKeys = flattenKeys(localeTranslations);

  const total = Object.keys(enKeys).length;
  let translated = 0;

  for (const key of Object.keys(enKeys)) {
    const localeValue = localeKeys[key];
    const enValue = enKeys[key];

    // Count as translated if:
    // - Key exists in locale
    // - Value is not empty
    // - Value is different from English (actually translated)
    if (
      localeValue &&
      String(localeValue).trim() !== '' &&
      String(localeValue) !== String(enValue)
    ) {
      translated++;
    }
  }

  return {
    translated,
    total,
    percentage: total > 0 ? ((translated / total) * 100).toFixed(1) : '0.0'
  };
}

/**
 * Generate status report
 */
function generateReport() {
  const enTranslations = loadLocaleTranslations('en');
  const results = [];

  const targetLocales = tier1Only ? TIER1_LOCALES : LOCALES;

  for (const locale of targetLocales) {
    const localeTranslations = loadLocaleTranslations(locale);
    const coverage = calculateCoverage(enTranslations, localeTranslations);

    results.push({
      locale,
      ...coverage
    });
  }

  return results;
}

/**
 * Display report in terminal
 */
function displayReport(results) {
  console.log('\n┌─────────┬──────────┬──────────┬─────────────┐');
  console.log('│ Locale  │ Keys     │ Translated│ Coverage   │');
  console.log('├─────────┼──────────┼──────────┼─────────────┤');

  for (const result of results) {
    const { locale, total, translated, percentage } = result;
    const coverageNum = parseFloat(percentage);
    const coverageColor = coverageNum >= 90 ? '\x1b[32m' : coverageNum >= 70 ? '\x1b[33m' : '\x1b[31m';

    console.log(
      `│ ${locale.padEnd(7)} │ ${String(total).padEnd(8)} │ ${String(translated).padEnd(9)} │ ${coverageColor}${percentage}%\x1b[0m${' '.repeat(9 - percentage.length)} │`
    );
  }

  console.log('└─────────┴──────────┴──────────┴─────────────┘\n');

  // Calculate average
  const avgCoverage = (
    results.reduce((sum, r) => sum + parseFloat(r.percentage), 0) / results.length
  ).toFixed(1);

  console.log(`Average coverage: ${avgCoverage}%`);

  // Tier breakdown
  if (!tier1Only) {
    const tier1Results = results.filter(r => TIER1_LOCALES.includes(r.locale));
    const tier1Avg = (
      tier1Results.reduce((sum, r) => sum + parseFloat(r.percentage), 0) / tier1Results.length
    ).toFixed(1);

    console.log(`Tier 1 average (es, fr, de, pt, ja): ${tier1Avg}%\n`);
  }

  return avgCoverage;
}

/**
 * Main execution
 */
function main() {
  console.log('📊 Translation Coverage Report\n');

  if (tier1Only) {
    console.log('🎯 Tier 1 locales only (es, fr, de, pt, ja)\n');
  }

  const results = generateReport();
  const avgCoverage = displayReport(results);

  // CI mode checks
  if (ciMode && minCoverage > 0) {
    const failedLocales = results.filter(r => parseFloat(r.percentage) < minCoverage);

    if (failedLocales.length > 0) {
      console.error(`\n❌ ${failedLocales.length} locale(s) below minimum coverage (${minCoverage}%):`);
      failedLocales.forEach(l => {
        console.error(`   ${l.locale}: ${l.percentage}%`);
      });
      process.exit(1);
    } else {
      console.log(`\n✅ All locales meet minimum coverage threshold (${minCoverage}%)`);
    }
  }
}

main();

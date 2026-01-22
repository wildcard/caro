#!/usr/bin/env node
/**
 * Translation Coverage Status Report
 *
 * Generates coverage statistics for all locales
 *
 * Usage:
 *   node scripts/i18n/status.mjs
 *   node scripts/i18n/status.mjs --ci
 *   node scripts/i18n/status.mjs --tier1-only
 *   node scripts/i18n/status.mjs --min-coverage 80
 *   node scripts/i18n/status.mjs --json
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Locale tiers from README
const LOCALE_TIERS = {
  tier1: ['es', 'fr', 'de', 'pt', 'ja'],
  tier2: ['ko', 'he', 'ar', 'hi'],
  tier3: ['ru', 'uk', 'ur', 'fil', 'id']
};

// Parse CLI arguments
const args = process.argv.slice(2);
const flags = {
  ci: args.includes('--ci'),
  tier1Only: args.includes('--tier1-only'),
  json: args.includes('--json'),
  minCoverage: parseInt(args.find(a => a.startsWith('--min-coverage'))?.split('=')[1] || '0')
};

/**
 * Recursively flatten nested JSON object into dot-notation keys
 */
function flattenKeys(obj, prefix = '') {
  const result = {};

  for (const [key, value] of Object.entries(obj)) {
    const newKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      Object.assign(result, flattenKeys(value, newKey));
    } else {
      result[newKey] = value;
    }
  }

  return result;
}

/**
 * Load all JSON files from a locale directory
 */
function loadLocaleTranslations(locale) {
  const localeDir = path.join(__dirname, '../../src/i18n/locales', locale);

  if (!fs.existsSync(localeDir)) {
    return {};
  }

  const translations = {};
  const files = fs.readdirSync(localeDir).filter(f => f.endsWith('.json'));

  for (const file of files) {
    const filePath = path.join(localeDir, file);
    const content = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    const namespace = file.replace('.json', '');
    translations[namespace] = content;
  }

  return translations;
}

/**
 * Calculate coverage statistics for a locale
 */
function calculateCoverage(enTranslations, localeTranslations) {
  const enKeys = flattenKeys(enTranslations);
  const localeKeys = flattenKeys(localeTranslations);

  const total = Object.keys(enKeys).length;
  let translated = 0;
  let untranslated = [];

  for (const key of Object.keys(enKeys)) {
    const enValue = enKeys[key];
    const localeValue = localeKeys[key];

    // Count as translated if:
    // 1. Key exists in locale
    // 2. Value is not empty
    // 3. Value is different from English (actually translated)
    if (localeValue &&
        localeValue.toString().trim() !== '' &&
        localeValue !== enValue) {
      translated++;
    } else {
      untranslated.push(key);
    }
  }

  return {
    total,
    translated,
    untranslated,
    percentage: total > 0 ? (translated / total * 100).toFixed(1) : '0.0'
  };
}

/**
 * Get tier for a locale
 */
function getLocaleTier(locale) {
  for (const [tier, locales] of Object.entries(LOCALE_TIERS)) {
    if (locales.includes(locale)) {
      return tier;
    }
  }
  return 'unknown';
}

/**
 * Generate status report
 */
function generateReport() {
  const localesDir = path.join(__dirname, '../../src/i18n/locales');
  const allLocales = fs.readdirSync(localesDir)
    .filter(name => fs.statSync(path.join(localesDir, name)).isDirectory())
    .filter(name => name !== 'en');  // Exclude English (source)

  // Filter to tier 1 if requested
  const targetLocales = flags.tier1Only
    ? allLocales.filter(l => LOCALE_TIERS.tier1.includes(l))
    : allLocales;

  // Load English translations (source)
  const enTranslations = loadLocaleTranslations('en');

  // Calculate coverage for each locale
  const results = [];

  for (const locale of targetLocales) {
    const localeTranslations = loadLocaleTranslations(locale);
    const coverage = calculateCoverage(enTranslations, localeTranslations);

    results.push({
      locale,
      tier: getLocaleTier(locale),
      ...coverage
    });
  }

  // Sort by coverage percentage (descending)
  results.sort((a, b) => parseFloat(b.percentage) - parseFloat(a.percentage));

  return results;
}

/**
 * Format report as table
 */
function formatTable(results) {
  console.log('\n┌─────────┬──────┬──────────┬─────────────┬────────────┐');
  console.log('│ Locale  │ Tier │ Keys     │ Translated  │ Coverage   │');
  console.log('├─────────┼──────┼──────────┼─────────────┼────────────┤');

  for (const result of results) {
    const locale = result.locale.padEnd(7);
    const tier = result.tier.padEnd(4);
    const total = result.total.toString().padStart(8);
    const translated = result.translated.toString().padStart(11);
    const coverage = `${result.percentage}%`.padStart(10);

    console.log(`│ ${locale} │ ${tier} │ ${total} │ ${translated} │ ${coverage} │`);
  }

  console.log('└─────────┴──────┴──────────┴─────────────┴────────────┘');

  // Calculate average
  const avgCoverage = (results.reduce((sum, r) => sum + parseFloat(r.percentage), 0) / results.length).toFixed(1);
  console.log(`\nAverage coverage: ${avgCoverage}%`);

  // Tier breakdown
  const tierStats = {
    tier1: results.filter(r => r.tier === 'tier1'),
    tier2: results.filter(r => r.tier === 'tier2'),
    tier3: results.filter(r => r.tier === 'tier3')
  };

  for (const [tier, locales] of Object.entries(tierStats)) {
    if (locales.length > 0) {
      const avg = (locales.reduce((sum, r) => sum + parseFloat(r.percentage), 0) / locales.length).toFixed(1);
      console.log(`${tier.toUpperCase()}: ${avg}% average (${locales.length} locales)`);
    }
  }
}

/**
 * Format report as JSON
 */
function formatJSON(results) {
  console.log(JSON.stringify(results, null, 2));
}

/**
 * Check minimum coverage threshold
 */
function checkMinimumCoverage(results, minCoverage) {
  const failing = results.filter(r => parseFloat(r.percentage) < minCoverage);

  if (failing.length > 0) {
    console.error(`\n❌ ${failing.length} locale(s) below ${minCoverage}% coverage:`);
    for (const result of failing) {
      console.error(`   - ${result.locale}: ${result.percentage}%`);
    }
    return false;
  }

  console.log(`\n✅ All locales meet ${minCoverage}% coverage threshold`);
  return true;
}

// Main execution
try {
  const results = generateReport();

  if (flags.json) {
    formatJSON(results);
  } else {
    formatTable(results);
  }

  // Check minimum coverage if specified
  if (flags.minCoverage > 0) {
    const passed = checkMinimumCoverage(results, flags.minCoverage);
    if (flags.ci && !passed) {
      process.exit(1);
    }
  }

} catch (error) {
  console.error('Error generating status report:', error.message);
  if (flags.ci) {
    process.exit(1);
  }
}

#!/usr/bin/env node

/**
 * Translation Status Report
 * Shows translation coverage statistics for all locales
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const LOCALES_DIR = join(__dirname, '../../src/i18n/locales');
const SUPPORTED_LOCALES = ['es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'];

// Tier classification (from plan)
const TIER1 = ['es', 'fr', 'pt', 'de', 'ja'];
const TIER2 = ['ko', 'he', 'ar', 'hi'];
const TIER3 = ['ru', 'uk', 'ur', 'fil', 'id'];

// Parse command line arguments
const args = process.argv.slice(2);
const CI_MODE = args.includes('--ci');
const MIN_COVERAGE = parseInt(args.find(arg => arg.startsWith('--min-coverage='))?.split('=')[1] || '0');
const TIER1_ONLY = args.includes('--tier1-only');

/**
 * Count keys recursively in a JSON object
 */
function countKeys(obj, prefix = '') {
  let count = 0;

  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      count += countKeys(value, `${prefix}${key}.`);
    } else {
      count++;
    }
  }

  return count;
}

/**
 * Get translation stats for a locale
 */
function getLocaleStats(locale) {
  const localeDir = join(LOCALES_DIR, locale);
  const enDir = join(LOCALES_DIR, 'en');

  if (!existsSync(localeDir)) {
    return { files: 0, keys: 0, enKeys: 0, coverage: 0 };
  }

  const files = readdirSync(localeDir).filter(f => f.endsWith('.json'));
  const enFiles = readdirSync(enDir).filter(f => f.endsWith('.json'));

  let totalKeys = 0;
  let totalEnKeys = 0;

  for (const file of enFiles) {
    const enPath = join(enDir, file);
    const localePath = join(localeDir, file);

    try {
      const enData = JSON.parse(readFileSync(enPath, 'utf-8'));
      const enKeyCount = countKeys(enData);
      totalEnKeys += enKeyCount;

      if (existsSync(localePath)) {
        const localeData = JSON.parse(readFileSync(localePath, 'utf-8'));
        const localeKeyCount = countKeys(localeData);
        totalKeys += localeKeyCount;
      }
    } catch (err) {
      console.error(`Error processing ${file} for ${locale}:`, err.message);
    }
  }

  const coverage = totalEnKeys > 0 ? (totalKeys / totalEnKeys * 100) : 0;

  return {
    files: files.length,
    enFiles: enFiles.length,
    keys: totalKeys,
    enKeys: totalEnKeys,
    coverage: coverage.toFixed(1)
  };
}

/**
 * Get tier for a locale
 */
function getTier(locale) {
  if (TIER1.includes(locale)) return 'TIER1';
  if (TIER2.includes(locale)) return 'TIER2';
  if (TIER3.includes(locale)) return 'TIER3';
  return 'UNKNOWN';
}

/**
 * Main function
 */
function main() {
  const results = [];
  const locales = TIER1_ONLY ? TIER1 : SUPPORTED_LOCALES;

  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('🌍 Translation Coverage Status');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  // Header
  console.log('┌─────────┬──────┬──────────┬──────────┬─────────────┐');
  console.log('│ Locale  │ Tier │ Keys     │ Files    │ Coverage    │');
  console.log('├─────────┼──────┼──────────┼──────────┼─────────────┤');

  let failedLocales = [];

  for (const locale of locales) {
    const stats = getLocaleStats(locale);
    const tier = getTier(locale);
    const coverage = parseFloat(stats.coverage);

    // Status indicator
    let status = '✓';
    if (MIN_COVERAGE > 0 && coverage < MIN_COVERAGE) {
      status = '✗';
      failedLocales.push({ locale, coverage });
    }

    console.log(
      `│ ${status} ${locale.padEnd(5)} │ ${tier.padEnd(4)} │ ${stats.keys.toString().padStart(3)}/${stats.enKeys.toString().padStart(3)} │ ${stats.files}/${stats.enFiles}      │ ${stats.coverage.padStart(5)}%     │`
    );

    results.push({ locale, tier, ...stats, coverageNum: coverage });
  }

  console.log('└─────────┴──────┴──────────┴──────────┴─────────────┘\n');

  // Calculate tier averages
  const tier1Avg = results.filter(r => TIER1.includes(r.locale)).reduce((sum, r) => sum + r.coverageNum, 0) / TIER1.filter(l => locales.includes(l)).length;
  const tier2Avg = results.filter(r => TIER2.includes(r.locale)).reduce((sum, r) => sum + r.coverageNum, 0) / TIER2.filter(l => locales.includes(l)).length;
  const tier3Avg = results.filter(r => TIER3.includes(r.locale)).reduce((sum, r) => sum + r.coverageNum, 0) / TIER3.filter(l => locales.includes(l)).length;
  const overallAvg = results.reduce((sum, r) => sum + r.coverageNum, 0) / results.length;

  console.log('Tier Averages:');
  console.log(`  TIER1 (es, fr, pt, de, ja): ${tier1Avg.toFixed(1)}%`);
  console.log(`  TIER2 (ko, he, ar, hi):     ${tier2Avg.toFixed(1)}%`);
  console.log(`  TIER3 (ru, uk, ur, fil, id): ${tier3Avg.toFixed(1)}%`);
  console.log(`  Overall:                    ${overallAvg.toFixed(1)}%\n`);

  // CI mode checks
  if (CI_MODE && MIN_COVERAGE > 0 && failedLocales.length > 0) {
    console.error(`❌ ${failedLocales.length} locale(s) below ${MIN_COVERAGE}% coverage threshold:\n`);
    for (const { locale, coverage } of failedLocales) {
      console.error(`  • ${locale}: ${coverage.toFixed(1)}% (need ${MIN_COVERAGE}%)`);
    }
    console.error('');
    process.exit(1);
  }

  if (CI_MODE) {
    console.log('✅ All locales meet coverage requirements\n');
  }
}

main();

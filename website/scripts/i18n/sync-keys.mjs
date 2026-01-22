#!/usr/bin/env node
/**
 * Sync Translation Keys
 *
 * Ensures all locales have the same key structure as English (source)
 * Missing keys are added with English values as placeholders
 *
 * Usage:
 *   node scripts/i18n/sync-keys.mjs
 *   node scripts/i18n/sync-keys.mjs --dry-run
 *   node scripts/i18n/sync-keys.mjs --locale es
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Parse CLI arguments
const args = process.argv.slice(2);
const flags = {
  dryRun: args.includes('--dry-run'),
  locale: args.find(a => a.startsWith('--locale'))?.split('=')[1]
};

/**
 * Recursively merge missing keys from source into target
 * Returns the updated target object
 */
function mergeKeys(source, target) {
  const result = { ...target };

  for (const [key, value] of Object.entries(source)) {
    if (!(key in result)) {
      // Key missing - add from source
      result[key] = value;
    } else if (value && typeof value === 'object' && !Array.isArray(value)) {
      // Nested object - recurse
      if (typeof result[key] === 'object' && !Array.isArray(result[key])) {
        result[key] = mergeKeys(value, result[key]);
      } else {
        // Type mismatch - overwrite with source
        result[key] = value;
      }
    }
    // Else: key exists and is a primitive - leave it alone
  }

  return result;
}

/**
 * Count differences between source and target
 */
function countDifferences(source, target, prefix = '') {
  let missing = 0;
  const missingKeys = [];

  for (const [key, value] of Object.entries(source)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (!(key in target)) {
      missing++;
      missingKeys.push(fullKey);
    } else if (value && typeof value === 'object' && !Array.isArray(value)) {
      if (typeof target[key] === 'object' && !Array.isArray(target[key])) {
        const nested = countDifferences(value, target[key], fullKey);
        missing += nested.missing;
        missingKeys.push(...nested.missingKeys);
      } else {
        missing++;
        missingKeys.push(fullKey);
      }
    }
  }

  return { missing, missingKeys };
}

/**
 * Process a single locale
 */
function syncLocale(locale) {
  const enDir = path.join(__dirname, '../../src/i18n/locales/en');
  const localeDir = path.join(__dirname, '../../src/i18n/locales', locale);

  if (!fs.existsSync(localeDir)) {
    console.log(`\n⚠️  Locale directory not found: ${locale}`);
    return { totalMissing: 0, files: [] };
  }

  const enFiles = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));
  let totalMissing = 0;
  const processedFiles = [];

  for (const file of enFiles) {
    const enPath = path.join(enDir, file);
    const localePath = path.join(localeDir, file);

    // Load English (source)
    const enContent = JSON.parse(fs.readFileSync(enPath, 'utf8'));

    // Load locale (target) or create empty object
    let localeContent = {};
    if (fs.existsSync(localePath)) {
      localeContent = JSON.parse(fs.readFileSync(localePath, 'utf8'));
    }

    // Count missing keys
    const { missing, missingKeys } = countDifferences(enContent, localeContent);

    if (missing > 0) {
      totalMissing += missing;
      processedFiles.push({ file, missing, missingKeys });

      if (!flags.dryRun) {
        // Merge and write
        const merged = mergeKeys(enContent, localeContent);
        fs.writeFileSync(localePath, JSON.stringify(merged, null, 2) + '\n', 'utf8');
      }
    }
  }

  return { totalMissing, files: processedFiles };
}

/**
 * Main execution
 */
try {
  const localesDir = path.join(__dirname, '../../src/i18n/locales');
  const allLocales = fs.readdirSync(localesDir)
    .filter(name => fs.statSync(path.join(localesDir, name)).isDirectory())
    .filter(name => name !== 'en');  // Exclude English (source)

  const targetLocales = flags.locale
    ? [flags.locale]
    : allLocales;

  if (flags.dryRun) {
    console.log('\n🔍 DRY RUN - No files will be modified\n');
  } else {
    console.log('\n🔄 Syncing translation keys...\n');
  }

  let grandTotal = 0;
  const results = [];

  for (const locale of targetLocales) {
    const result = syncLocale(locale);
    grandTotal += result.totalMissing;

    if (result.totalMissing > 0) {
      results.push({ locale, ...result });
    }
  }

  // Print summary
  if (results.length === 0) {
    console.log('✅ All locales are in sync - no missing keys');
  } else {
    console.log('Summary:\n');
    for (const { locale, totalMissing, files } of results) {
      console.log(`${locale}: ${totalMissing} missing key(s)`);
      for (const { file, missing } of files) {
        console.log(`  - ${file}: ${missing} key(s)`);
      }
    }

    console.log(`\nTotal: ${grandTotal} missing key(s) across ${results.length} locale(s)`);

    if (flags.dryRun) {
      console.log('\n💡 Run without --dry-run to apply changes');
    } else {
      console.log('\n✅ Keys synchronized');
    }
  }

  process.exit(0);

} catch (error) {
  console.error('Error syncing keys:', error.message);
  console.error(error.stack);
  process.exit(1);
}

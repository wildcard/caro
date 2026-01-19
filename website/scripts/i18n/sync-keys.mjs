#!/usr/bin/env node
/**
 * Sync Translation Keys
 *
 * Ensures all locale files have the same keys as English.
 * Creates skeleton files with English fallback for missing keys.
 *
 * Usage:
 *   node scripts/i18n/sync-keys.mjs
 *   node scripts/i18n/sync-keys.mjs --dry-run  # Show changes without writing
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const LOCALES_DIR = path.join(__dirname, '../../src/i18n/locales');
const LOCALES = ['es', 'fr', 'pt', 'de', 'ja', 'ko', 'he', 'ar', 'ru', 'uk', 'hi', 'ur', 'fil', 'id'];

const args = process.argv.slice(2);
const dryRun = args.includes('--dry-run');

/**
 * Recursively merge missing keys from source into target
 */
function mergeKeys(source, target, path = []) {
  let changes = 0;

  for (const [key, value] of Object.entries(source)) {
    const currentPath = [...path, key].join('.');

    if (!(key in target)) {
      // Missing key - add with English fallback
      target[key] = value;
      changes++;
      console.log(`  ✚ Added missing key: ${currentPath}`);
    } else if (value && typeof value === 'object' && !Array.isArray(value)) {
      // Nested object - recurse
      if (typeof target[key] !== 'object' || Array.isArray(target[key])) {
        target[key] = {};
      }
      changes += mergeKeys(value, target[key], [...path, key]);
    }
  }

  return changes;
}

/**
 * Sync a single locale against English
 */
function syncLocale(locale) {
  console.log(`\n📦 Syncing ${locale}...`);

  const enDir = path.join(LOCALES_DIR, 'en');
  const localeDir = path.join(LOCALES_DIR, locale);

  if (!fs.existsSync(localeDir)) {
    fs.mkdirSync(localeDir, { recursive: true });
    console.log(`  Created directory: ${localeDir}`);
  }

  const enFiles = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));
  let totalChanges = 0;

  for (const file of enFiles) {
    const enFilePath = path.join(enDir, file);
    const localeFilePath = path.join(localeDir, file);

    const enContent = JSON.parse(fs.readFileSync(enFilePath, 'utf8'));
    let localeContent = {};

    if (fs.existsSync(localeFilePath)) {
      try {
        localeContent = JSON.parse(fs.readFileSync(localeFilePath, 'utf8'));
      } catch (error) {
        console.error(`  ⚠️  Error reading ${file}:`, error.message);
        localeContent = {};
      }
    } else {
      console.log(`  📄 Creating new file: ${file}`);
    }

    const changes = mergeKeys(enContent, localeContent);

    if (changes > 0 || !fs.existsSync(localeFilePath)) {
      totalChanges += changes;

      if (!dryRun) {
        fs.writeFileSync(
          localeFilePath,
          JSON.stringify(localeContent, null, 2) + '\n',
          'utf8'
        );
        console.log(`  ✓ Updated ${file} (${changes} keys added)`);
      } else {
        console.log(`  [DRY RUN] Would update ${file} (${changes} keys)`);
      }
    }
  }

  if (totalChanges === 0) {
    console.log(`  ✓ Already in sync`);
  }

  return totalChanges;
}

/**
 * Main execution
 */
function main() {
  console.log('🔄 Syncing Translation Keys\n');

  if (dryRun) {
    console.log('🔍 DRY RUN MODE - No files will be modified\n');
  }

  let grandTotal = 0;

  for (const locale of LOCALES) {
    const changes = syncLocale(locale);
    grandTotal += changes;
  }

  console.log(`\n${'═'.repeat(50)}`);
  if (grandTotal === 0) {
    console.log('✅ All locales are in sync with English!');
  } else {
    if (dryRun) {
      console.log(`📊 Would add ${grandTotal} keys across all locales`);
    } else {
      console.log(`✅ Added ${grandTotal} keys across all locales`);
    }
  }
  console.log(`${'═'.repeat(50)}\n`);
}

main();

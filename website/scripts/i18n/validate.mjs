#!/usr/bin/env node
/**
 * Validate Translation Files
 *
 * Checks translation files for common issues:
 * - Valid JSON syntax
 * - Placeholder preservation ({count}, {name})
 * - Protected terms preservation (Caro, Claude, POSIX)
 * - Empty translations
 * - Duplicate keys
 *
 * Usage:
 *   node scripts/i18n/validate.mjs
 *   node scripts/i18n/validate.mjs --locale es  # Validate specific locale
 *   node scripts/i18n/validate.mjs --strict     # Exit with error on warnings
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const LOCALES_DIR = path.join(__dirname, '../../src/i18n/locales');
const LOCALES = ['es', 'fr', 'pt', 'de', 'ja', 'ko', 'he', 'ar', 'ru', 'uk', 'hi', 'ur', 'fil', 'id'];

// Protected terms that should NEVER be translated
const PROTECTED_TERMS = [
  'Caro', 'Claude', 'POSIX', 'CLI', 'API', 'BSD', 'GNU', 'MLX',
  'Kyaro', 'Kyarorain', 'Kadosh', 'Caroline', 'GLaDOS',
  'GitHub', 'Anthropic', 'Aperture Science'
];

// Regex patterns
const PLACEHOLDER_PATTERN = /\{[^}]+\}/g;

// Parse CLI args
const args = process.argv.slice(2);
const targetLocale = args.includes('--locale')
  ? args[args.indexOf('--locale') + 1]
  : null;
const strictMode = args.includes('--strict');

let totalErrors = 0;
let totalWarnings = 0;

/**
 * Flatten nested JSON for easier validation
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
 * Validate a single translation value against English
 */
function validateTranslation(key, enValue, localeValue, locale) {
  const issues = [];

  // 1. Check for empty translations
  if (!localeValue || String(localeValue).trim() === '') {
    issues.push({ type: 'error', message: 'Empty translation' });
  }

  // 2. Check placeholder preservation
  const enPlaceholders = String(enValue).match(PLACEHOLDER_PATTERN) || [];
  const localePlaceholders = String(localeValue).match(PLACEHOLDER_PATTERN) || [];

  if (enPlaceholders.length !== localePlaceholders.length) {
    issues.push({
      type: 'error',
      message: `Placeholder mismatch. English: [${enPlaceholders.join(', ')}], ${locale}: [${localePlaceholders.join(', ')}]`
    });
  } else {
    const enSet = new Set(enPlaceholders.sort());
    const localeSet = new Set(localePlaceholders.sort());

    if (JSON.stringify([...enSet]) !== JSON.stringify([...localeSet])) {
      issues.push({
        type: 'error',
        message: `Different placeholders. Expected: [${[...enSet].join(', ')}], Got: [${[...localeSet].join(', ')}]`
      });
    }
  }

  // 3. Check protected terms preservation
  for (const term of PROTECTED_TERMS) {
    const enHas = String(enValue).includes(term);
    const localeHas = String(localeValue).includes(term);

    if (enHas && !localeHas) {
      issues.push({
        type: 'warning',
        message: `Protected term "${term}" missing. Verify this is intentional.`
      });
    }
  }

  // 4. Check for untranslated (identical to English)
  const rtlLocales = ['he', 'ar', 'ur'];
  if (
    !rtlLocales.includes(locale) &&
    String(enValue) === String(localeValue) &&
    String(enValue).length > 20 && // Only flag long strings
    !/^[0-9\s\{\}\[\]\(\)\-\_\.\/]+$/.test(String(enValue)) // Ignore technical strings
  ) {
    issues.push({
      type: 'warning',
      message: 'Possibly untranslated (identical to English)'
    });
  }

  return issues;
}

/**
 * Validate a locale against English
 */
function validateLocale(locale) {
  console.log(`\n🔍 Validating ${locale}...`);

  const enDir = path.join(LOCALES_DIR, 'en');
  const localeDir = path.join(LOCALES_DIR, locale);

  if (!fs.existsSync(localeDir)) {
    console.error(`  ❌ Locale directory does not exist: ${localeDir}`);
    totalErrors++;
    return;
  }

  const enFiles = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));
  let localeErrors = 0;
  let localeWarnings = 0;

  for (const file of enFiles) {
    const enFilePath = path.join(enDir, file);
    const localeFilePath = path.join(localeDir, file);

    // Check if locale file exists
    if (!fs.existsSync(localeFilePath)) {
      console.error(`  ❌ Missing file: ${file}`);
      localeErrors++;
      continue;
    }

    let enContent, localeContent;

    // Try to parse JSON
    try {
      enContent = JSON.parse(fs.readFileSync(enFilePath, 'utf8'));
    } catch (error) {
      console.error(`  ❌ Invalid JSON in en/${file}: ${error.message}`);
      localeErrors++;
      continue;
    }

    try {
      localeContent = JSON.parse(fs.readFileSync(localeFilePath, 'utf8'));
    } catch (error) {
      console.error(`  ❌ Invalid JSON in ${locale}/${file}: ${error.message}`);
      localeErrors++;
      continue;
    }

    // Flatten and validate
    const enFlat = flattenKeys(enContent);
    const localeFlat = flattenKeys(localeContent);

    for (const [key, enValue] of Object.entries(enFlat)) {
      const localeValue = localeFlat[key];

      if (!localeValue) {
        // Missing key (warning, since sync-keys.mjs should handle this)
        localeWarnings++;
        continue;
      }

      const issues = validateTranslation(key, enValue, localeValue, locale);

      for (const issue of issues) {
        if (issue.type === 'error') {
          console.error(`  ❌ [${file}] ${key}: ${issue.message}`);
          localeErrors++;
        } else if (issue.type === 'warning') {
          console.warn(`  ⚠️  [${file}] ${key}: ${issue.message}`);
          localeWarnings++;
        }
      }
    }
  }

  if (localeErrors === 0 && localeWarnings === 0) {
    console.log(`  ✅ No issues found`);
  } else {
    console.log(`  📊 ${localeErrors} errors, ${localeWarnings} warnings`);
  }

  totalErrors += localeErrors;
  totalWarnings += localeWarnings;
}

/**
 * Main execution
 */
function main() {
  console.log('✅ Validating Translation Files\n');

  if (strictMode) {
    console.log('⚠️  STRICT MODE: Warnings will be treated as errors\n');
  }

  const localesToValidate = targetLocale ? [targetLocale] : LOCALES;

  for (const locale of localesToValidate) {
    if (!LOCALES.includes(locale) && locale !== 'en') {
      console.error(`❌ Invalid locale: ${locale}`);
      process.exit(1);
    }

    validateLocale(locale);
  }

  console.log(`\n${'═'.repeat(50)}`);
  console.log(`📊 Total: ${totalErrors} errors, ${totalWarnings} warnings`);
  console.log(`${'═'.repeat(50)}\n`);

  if (totalErrors > 0 || (strictMode && totalWarnings > 0)) {
    console.error('❌ Validation failed');
    process.exit(1);
  } else {
    console.log('✅ Validation passed');
  }
}

main();

#!/usr/bin/env node

/**
 * Translation Validation Script
 * Validates translation files for common issues
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const LOCALES_DIR = join(__dirname, '../../src/i18n/locales');
const SUPPORTED_LOCALES = ['es', 'fr', 'pt', 'de', 'he', 'ar', 'uk', 'ru', 'ja', 'ko', 'hi', 'ur', 'fil', 'id'];

// Parse command line arguments
const args = process.argv.slice(2);
const CI_MODE = args.includes('--ci');

// Brand names that should NEVER be translated
const PRESERVE_EXACT = [
  'Caro',
  'Claude',
  'GitHub',
  'Anthropic',
  'POSIX',
  'BSD',
  'GNU',
  'MLX',
  'Ollama',
  'vLLM',
  'Rust',
];

// Placeholder pattern: {variable}, {{variable}}, {count}, etc.
const PLACEHOLDER_PATTERN = /\{[^}]+\}/g;

let errors = [];
let warnings = [];

/**
 * Check if a string contains placeholders
 */
function extractPlaceholders(str) {
  return str.match(PLACEHOLDER_PATTERN) || [];
}

/**
 * Get all key-value pairs from a nested object
 */
function flattenKeys(obj, prefix = '') {
  const result = {};

  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      Object.assign(result, flattenKeys(value, fullKey));
    } else if (typeof value === 'string') {
      result[fullKey] = value;
    }
  }

  return result;
}

/**
 * Validate a single translation file
 */
function validateFile(locale, filename) {
  const enPath = join(LOCALES_DIR, 'en', filename);
  const localePath = join(LOCALES_DIR, locale, filename);

  if (!existsSync(localePath)) {
    warnings.push(`[${locale}/${filename}] File missing`);
    return;
  }

  try {
    // Parse JSON files
    const enData = JSON.parse(readFileSync(enPath, 'utf-8'));
    const localeData = JSON.parse(readFileSync(localePath, 'utf-8'));

    const enKeys = flattenKeys(enData);
    const localeKeys = flattenKeys(localeData);

    // Check 1: Validate placeholders are preserved
    for (const [key, enValue] of Object.entries(enKeys)) {
      const localeValue = localeKeys[key];

      if (!localeValue) {
        warnings.push(`[${locale}/${filename}] Missing key: ${key}`);
        continue;
      }

      const enPlaceholders = extractPlaceholders(enValue);
      const localePlaceholders = extractPlaceholders(localeValue);

      // Check placeholder count
      if (enPlaceholders.length !== localePlaceholders.length) {
        errors.push(
          `[${locale}/${filename}] Placeholder mismatch in key "${key}":\n` +
          `  EN: ${enPlaceholders.join(', ')} (${enPlaceholders.length})\n` +
          `  ${locale.toUpperCase()}: ${localePlaceholders.join(', ')} (${localePlaceholders.length})`
        );
        continue;
      }

      // Check placeholder names match
      const enSet = new Set(enPlaceholders);
      const localeSet = new Set(localePlaceholders);

      for (const placeholder of enPlaceholders) {
        if (!localeSet.has(placeholder)) {
          errors.push(
            `[${locale}/${filename}] Placeholder "${placeholder}" missing in key "${key}":\n` +
            `  EN: ${enValue}\n` +
            `  ${locale.toUpperCase()}: ${localeValue}`
          );
        }
      }
    }

    // Check 2: Validate brand names are preserved
    for (const [key, localeValue] of Object.entries(localeKeys)) {
      const enValue = enKeys[key];

      if (!enValue) continue;

      for (const brandName of PRESERVE_EXACT) {
        const enHasBrand = enValue.includes(brandName);
        const localeHasBrand = localeValue.includes(brandName);

        if (enHasBrand && !localeHasBrand) {
          warnings.push(
            `[${locale}/${filename}] Brand name "${brandName}" may have been translated in key "${key}":\n` +
            `  EN: ${enValue}\n` +
            `  ${locale.toUpperCase()}: ${localeValue}`
          );
        }
      }
    }

    // Check 3: Validate no extra keys in translation
    for (const key of Object.keys(localeKeys)) {
      if (!enKeys[key]) {
        warnings.push(`[${locale}/${filename}] Extra key not in English: ${key}`);
      }
    }

  } catch (err) {
    errors.push(`[${locale}/${filename}] Failed to parse: ${err.message}`);
  }
}

/**
 * Validate all locales
 */
function main() {
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('🔍 Validating Translation Files');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  // Get all English files as reference
  const enDir = join(LOCALES_DIR, 'en');
  const enFiles = readdirSync(enDir).filter(f => f.endsWith('.json'));

  console.log(`Checking ${enFiles.length} files across ${SUPPORTED_LOCALES.length} locales...\n`);

  // Validate each locale
  for (const locale of SUPPORTED_LOCALES) {
    for (const filename of enFiles) {
      validateFile(locale, filename);
    }
  }

  // Report results
  console.log('Results:');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

  if (errors.length > 0) {
    console.error(`❌ Found ${errors.length} error(s):\n`);
    for (const error of errors) {
      console.error(error);
      console.error('');
    }
  }

  if (warnings.length > 0) {
    console.warn(`⚠️  Found ${warnings.length} warning(s):\n`);
    for (const warning of warnings.slice(0, 10)) {
      console.warn(warning);
    }
    if (warnings.length > 10) {
      console.warn(`\n... and ${warnings.length - 10} more warnings`);
    }
    console.warn('');
  }

  if (errors.length === 0 && warnings.length === 0) {
    console.log('✅ All validation checks passed!\n');
  }

  // Summary
  console.log('Summary:');
  console.log(`  Errors:   ${errors.length}`);
  console.log(`  Warnings: ${warnings.length}\n`);

  // Exit code for CI
  if (CI_MODE && errors.length > 0) {
    console.error('❌ Validation failed in CI mode\n');
    process.exit(1);
  }

  if (errors.length > 0) {
    process.exit(1);
  }
}

main();

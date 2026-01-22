#!/usr/bin/env node
/**
 * Translation Quality Validation
 *
 * Validates translation files for quality issues:
 * - JSON syntax errors
 * - Placeholder preservation
 * - Protected terms preservation
 * - Empty translations
 * - Untranslated content
 *
 * Usage:
 *   node scripts/i18n/validate.mjs
 *   node scripts/i18n/validate.mjs --locale es
 *   node scripts/i18n/validate.mjs --strict
 *   node scripts/i18n/validate.mjs --ci
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Parse CLI arguments
const args = process.argv.slice(2);
const flags = {
  strict: args.includes('--strict'),
  ci: args.includes('--ci'),
  locale: args.find(a => a.startsWith('--locale'))?.split('=')[1]
};

// Protected terms that should NEVER be translated
const PROTECTED_TERMS = [
  'Caro', 'Claude', 'GitHub', 'Anthropic',
  'POSIX', 'CLI', 'API', 'JSON', 'YAML',
  'BSD', 'GNU', 'MLX', 'Kyaro',
  'GPT-4', 'OpenAI', 'Llama'
];

// Placeholder pattern (e.g., {count}, {name}, {{variable}})
const PLACEHOLDER_PATTERN = /\{[^}]+\}/g;

/**
 * Recursively flatten nested JSON object into key-value pairs
 */
function flattenTranslations(obj, prefix = '') {
  const result = {};

  for (const [key, value] of Object.entries(obj)) {
    const newKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      Object.assign(result, flattenTranslations(value, newKey));
    } else {
      result[newKey] = value;
    }
  }

  return result;
}

/**
 * Validate a single translation value against source
 */
function validateTranslation(key, sourceValue, targetValue, locale) {
  const errors = [];

  // 1. Check for empty/whitespace-only translations
  if (!targetValue || targetValue.toString().trim() === '') {
    errors.push({
      severity: 'error',
      code: 'EMPTY_TRANSLATION',
      message: `Translation is empty or whitespace-only`
    });
    return errors; // Skip other checks if empty
  }

  // 2. Check placeholder preservation
  const sourceStr = String(sourceValue);
  const targetStr = String(targetValue);
  const sourcePlaceholders = (sourceStr.match(PLACEHOLDER_PATTERN) || []).sort();
  const targetPlaceholders = (targetStr.match(PLACEHOLDER_PATTERN) || []).sort();

  if (JSON.stringify(sourcePlaceholders) !== JSON.stringify(targetPlaceholders)) {
    errors.push({
      severity: 'error',
      code: 'PLACEHOLDER_MISMATCH',
      message: `Placeholders don't match. Source: [${sourcePlaceholders.join(', ')}], Target: [${targetPlaceholders.join(', ')}]`
    });
  }

  // 3. Check protected terms preservation
  for (const term of PROTECTED_TERMS) {
    const sourceHas = sourceStr.includes(term);
    const targetHas = targetStr.includes(term);

    if (sourceHas && !targetHas) {
      errors.push({
        severity: 'warning',
        code: 'PROTECTED_TERM_MISSING',
        message: `Protected term "${term}" missing in translation`
      });
    }
  }

  // 4. Check for potentially untranslated content (identical to source)
  const rtlLocales = ['he', 'ar', 'ur'];
  if (!rtlLocales.includes(locale) &&
      targetStr === sourceStr &&
      sourceStr.length > 15 &&
      !sourceStr.match(/^[0-9\s\p{P}]+$/u)) { // Not just numbers/punctuation
    errors.push({
      severity: 'warning',
      code: 'POSSIBLY_UNTRANSLATED',
      message: `Value identical to English source (might be untranslated)`
    });
  }

  // 5. Check for suspicious patterns (common machine translation artifacts)
  if (targetStr.includes('[[') || targetStr.includes(']]')) {
    errors.push({
      severity: 'warning',
      code: 'SUSPICIOUS_PATTERN',
      message: `Contains suspicious brackets [[ ]] - possible machine translation artifact`
    });
  }

  return errors;
}

/**
 * Validate a single locale's translation files
 */
function validateLocale(locale) {
  const enDir = path.join(__dirname, '../../src/i18n/locales/en');
  const localeDir = path.join(__dirname, '../../src/i18n/locales', locale);

  if (!fs.existsSync(localeDir)) {
    return {
      locale,
      success: false,
      error: `Locale directory not found: ${locale}`
    };
  }

  const enFiles = fs.readdirSync(enDir).filter(f => f.endsWith('.json'));
  const results = {
    locale,
    success: true,
    files: [],
    totalErrors: 0,
    totalWarnings: 0
  };

  for (const file of enFiles) {
    const enPath = path.join(enDir, file);
    const localePath = path.join(localeDir, file);

    const fileResult = {
      file,
      errors: [],
      warnings: []
    };

    // Check if file exists
    if (!fs.existsSync(localePath)) {
      fileResult.errors.push({
        key: null,
        severity: 'error',
        code: 'FILE_MISSING',
        message: `Translation file missing for locale ${locale}`
      });
      results.files.push(fileResult);
      results.totalErrors++;
      continue;
    }

    // Validate JSON syntax
    let enContent, localeContent;
    try {
      enContent = JSON.parse(fs.readFileSync(enPath, 'utf8'));
    } catch (err) {
      fileResult.errors.push({
        key: null,
        severity: 'error',
        code: 'INVALID_JSON',
        message: `English source has invalid JSON: ${err.message}`
      });
      results.files.push(fileResult);
      results.totalErrors++;
      continue;
    }

    try {
      localeContent = JSON.parse(fs.readFileSync(localePath, 'utf8'));
    } catch (err) {
      fileResult.errors.push({
        key: null,
        severity: 'error',
        code: 'INVALID_JSON',
        message: `Invalid JSON syntax: ${err.message}`
      });
      results.files.push(fileResult);
      results.totalErrors++;
      continue;
    }

    // Flatten and validate each key
    const enFlat = flattenTranslations(enContent);
    const localeFlat = flattenTranslations(localeContent);

    for (const [key, enValue] of Object.entries(enFlat)) {
      const localeValue = localeFlat[key];

      if (!localeValue) {
        fileResult.warnings.push({
          key,
          severity: 'warning',
          code: 'KEY_MISSING',
          message: `Key not found in ${locale} translation`
        });
        continue;
      }

      const validationErrors = validateTranslation(key, enValue, localeValue, locale);

      for (const error of validationErrors) {
        const issue = { key, ...error };
        if (error.severity === 'error') {
          fileResult.errors.push(issue);
        } else {
          fileResult.warnings.push(issue);
        }
      }
    }

    results.totalErrors += fileResult.errors.length;
    results.totalWarnings += fileResult.warnings.length;

    // Only include files with issues (or in strict mode, all files)
    if (fileResult.errors.length > 0 || fileResult.warnings.length > 0 || flags.strict) {
      results.files.push(fileResult);
    }
  }

  return results;
}

/**
 * Format validation results
 */
function formatResults(allResults) {
  const hasErrors = allResults.some(r => r.totalErrors > 0);
  const hasWarnings = allResults.some(r => r.totalWarnings > 0);

  if (!hasErrors && !hasWarnings) {
    console.log('✅ All translations valid - no issues found');
    return true;
  }

  console.log('\n📋 Validation Results\n');

  for (const result of allResults) {
    if (result.error) {
      console.log(`❌ ${result.locale}: ${result.error}`);
      continue;
    }

    if (result.totalErrors === 0 && result.totalWarnings === 0 && !flags.strict) {
      continue; // Skip clean locales unless strict mode
    }

    const status = result.totalErrors > 0 ? '❌' : (result.totalWarnings > 0 ? '⚠️' : '✅');
    console.log(`${status} ${result.locale}: ${result.totalErrors} error(s), ${result.totalWarnings} warning(s)`);

    for (const fileResult of result.files) {
      if (fileResult.errors.length === 0 && fileResult.warnings.length === 0) {
        if (flags.strict) {
          console.log(`  ✅ ${fileResult.file}`);
        }
        continue;
      }

      console.log(`\n  📄 ${fileResult.file}`);

      // Show errors first
      for (const issue of fileResult.errors) {
        console.log(`    ❌ ${issue.code}${issue.key ? ` [${issue.key}]` : ''}`);
        console.log(`       ${issue.message}`);
      }

      // Show warnings if not in CI mode (CI only cares about errors)
      if (!flags.ci) {
        for (const issue of fileResult.warnings) {
          console.log(`    ⚠️  ${issue.code}${issue.key ? ` [${issue.key}]` : ''}`);
          console.log(`       ${issue.message}`);
        }
      }
    }

    console.log('');
  }

  // Summary
  const totalErrors = allResults.reduce((sum, r) => sum + (r.totalErrors || 0), 0);
  const totalWarnings = allResults.reduce((sum, r) => sum + (r.totalWarnings || 0), 0);

  console.log('═══════════════════════════════════════');
  console.log(`Total: ${totalErrors} error(s), ${totalWarnings} warning(s)`);

  if (totalErrors > 0) {
    console.log('\n❌ Validation failed - fix errors before committing');
    return false;
  } else if (totalWarnings > 0 && flags.strict) {
    console.log('\n⚠️  Validation passed with warnings (strict mode)');
    return false;
  } else {
    console.log('\n✅ Validation passed');
    return true;
  }
}

// Main execution
try {
  const localesDir = path.join(__dirname, '../../src/i18n/locales');
  const allLocales = fs.readdirSync(localesDir)
    .filter(name => fs.statSync(path.join(localesDir, name)).isDirectory())
    .filter(name => name !== 'en');

  const targetLocales = flags.locale
    ? [flags.locale]
    : allLocales;

  const results = targetLocales.map(locale => validateLocale(locale));
  const success = formatResults(results);

  process.exit(success ? 0 : 1);

} catch (error) {
  console.error('Error during validation:', error.message);
  if (flags.ci) {
    process.exit(1);
  }
}

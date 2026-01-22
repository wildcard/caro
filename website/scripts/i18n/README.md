# i18n Automation Scripts

Automation tools for managing translations across 14 supported locales.

## Overview

This directory contains three scripts for translation management:

- **status.mjs** - Coverage reporting and statistics
- **sync-keys.mjs** - Key synchronization across locales  
- **validate.mjs** - Translation quality validation

All scripts are written in Node.js ESM and can be run directly.

---

## status.mjs - Coverage Reporting

Generates translation coverage statistics for all locales.

### Usage

```bash
# Show coverage for all locales
node scripts/i18n/status.mjs

# Show only Tier 1 locales (es, fr, de, pt, ja)
node scripts/i18n/status.mjs --tier1-only

# Enforce minimum coverage threshold (exits 1 if any locale below threshold)
node scripts/i18n/status.mjs --min-coverage 80

# CI mode (combines with --min-coverage for build gates)
node scripts/i18n/status.mjs --ci --min-coverage 80

# Output as JSON instead of table
node scripts/i18n/status.mjs --json
```

### Output Format

**Table mode** (default):
```
┌─────────┬──────┬──────────┬─────────────┬────────────┐
│ Locale  │ Tier │ Keys     │ Translated  │ Coverage   │
├─────────┼──────┼──────────┼─────────────┼────────────┤
│ ar      │ tier2 │      207 │         188 │      90.8% │
│ he      │ tier2 │      207 │         184 │      88.9% │
...
└─────────┴──────┴──────────┴─────────────┴────────────┘

Average coverage: 79.8%
TIER1: 73.1% average (5 locales)
TIER2: 88.0% average (4 locales)
TIER3: 79.9% average (5 locales)
```

**JSON mode** (`--json`):
```json
[
  {
    "locale": "ar",
    "tier": "tier2",
    "total": 207,
    "translated": 188,
    "percentage": "90.8"
  },
  ...
]
```

### Locale Tiers

| Tier | Locales | Target Coverage |
|------|---------|-----------------|
| Tier 1 | es, fr, de, pt, ja | 95%+ |
| Tier 2 | ko, he, ar, hi | 85%+ |
| Tier 3 | ru, uk, ur, fil, id | 75%+ |

### Exit Codes

- `0` - Success (all checks passed)
- `1` - Failure (error occurred or coverage below threshold)

---

## sync-keys.mjs - Key Synchronization

Ensures all locales have the same key structure as English (source locale).

### Usage

```bash
# Preview missing keys without making changes
node scripts/i18n/sync-keys.mjs --dry-run

# Sync all locales
node scripts/i18n/sync-keys.mjs

# Sync a specific locale
node scripts/i18n/sync-keys.mjs --locale=pt

# Combine with dry-run to preview one locale
node scripts/i18n/sync-keys.mjs --dry-run --locale=es
```

### What It Does

1. **Loads English** (source) translations as the reference
2. **Compares** each locale's keys against English
3. **Identifies** missing keys at any nesting level
4. **Adds** missing keys with English values as placeholders
5. **Preserves** existing translations (never overwrites)

### Output

**Dry-run mode**:
```
🔍 DRY RUN - No files will be modified

Summary:

pt: 3 missing key(s)
  - common.json: 2 key(s)
  - landing.json: 1 key(s)

Total: 3 missing key(s) across 1 locale(s)

💡 Run without --dry-run to apply changes
```

**Normal mode**:
```
🔄 Syncing translation keys...

Summary:

pt: 3 missing key(s)
  - common.json: 2 key(s)
  - landing.json: 1 key(s)

Total: 3 missing key(s) across 1 locale(s)

✅ Keys synchronized
```

### When to Use

- After adding new keys to English translations
- Before starting translation work on a locale
- To verify all locales have complete key structure
- In CI to catch missing keys before merge

---

## validate.mjs - Quality Validation

Validates translation files for common quality issues.

### Usage

```bash
# Validate all locales
node scripts/i18n/validate.mjs

# Validate a specific locale
node scripts/i18n/validate.mjs --locale es

# Strict mode (warnings also cause failure)
node scripts/i18n/validate.mjs --strict

# CI mode (only shows errors, not warnings)
node scripts/i18n/validate.mjs --ci
```

### Validation Rules

#### Errors (block commit)

1. **INVALID_JSON** - JSON syntax errors
2. **EMPTY_TRANSLATION** - Empty or whitespace-only values
3. **PLACEHOLDER_MISMATCH** - Placeholders don't match source
   - Example: Source has `{count}`, translation missing it
4. **FILE_MISSING** - Translation file doesn't exist

#### Warnings (informational)

1. **PROTECTED_TERM_MISSING** - Brand names should not be translated
   - Protected terms: Caro, Claude, GitHub, Anthropic, POSIX, CLI, API, etc.
2. **POSSIBLY_UNTRANSLATED** - Value identical to English (length > 15)
3. **SUSPICIOUS_PATTERN** - Contains `[[` or `]]` (machine translation artifacts)
4. **KEY_MISSING** - Key exists in English but not in locale

### Output Format

```
📋 Validation Results

❌ es: 2 error(s), 5 warning(s)

  📄 common.json
    ❌ PLACEHOLDER_MISMATCH [common.buttons.download]
       Placeholders don't match. Source: [{platform}], Target: []
    
    ⚠️  PROTECTED_TERM_MISSING [common.meta.description]
       Protected term "Claude" missing in translation

═══════════════════════════════════════
Total: 2 error(s), 5 warning(s)

❌ Validation failed - fix errors before committing
```

### Protected Terms

Never translate these terms:
- **Brand names**: Caro, Claude, GitHub, Anthropic
- **Technical terms**: POSIX, CLI, API, JSON, YAML, BSD, GNU, MLX
- **Proper names**: Kyaro, GPT-4, OpenAI, Llama

### Exit Codes

- `0` - Success (no errors, or only warnings in non-strict mode)
- `1` - Failure (errors found, or warnings in strict mode)

---

## Integration Examples

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating translations..."
cd website
node scripts/i18n/validate.mjs --ci

if [ $? -ne 0 ]; then
  echo "❌ Translation validation failed"
  echo "Run: node scripts/i18n/validate.mjs"
  exit 1
fi

echo "✅ Translations valid"
```

### CI Workflow

```yaml
# .github/workflows/i18n-check.yml
name: i18n Quality Check

on:
  pull_request:
    paths:
      - 'website/src/i18n/locales/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      
      - name: Validate translations
        run: |
          cd website
          node scripts/i18n/validate.mjs --ci
      
      - name: Check Tier 1 coverage
        run: |
          cd website
          node scripts/i18n/status.mjs --tier1-only --min-coverage 80
```

### Development Workflow

```bash
# 1. Add new keys to English
vim website/src/i18n/locales/en/features.json

# 2. Sync keys to all locales
node scripts/i18n/sync-keys.mjs

# 3. Check which locales need translation
node scripts/i18n/status.mjs

# 4. Translate specific locale
# (use /translator skill or manual editing)

# 5. Validate quality before commit
node scripts/i18n/validate.mjs --locale es

# 6. Commit if validation passes
git add website/src/i18n/locales/
git commit -m "feat(i18n): Add feature descriptions"
```

---

## Troubleshooting

### Script fails with "Cannot find module"

Ensure you're running from the website directory:
```bash
cd website
node scripts/i18n/status.mjs
```

### Coverage looks wrong

The coverage calculation counts a key as "translated" only if:
1. Key exists in the locale
2. Value is not empty/whitespace
3. Value differs from English (actually translated)

Keys with English fallback values count as "untranslated".

### Validation shows false positives

Some brand names may appear in different scripts (Arabic, Hebrew, etc.) and look "missing" but are actually present. These are warnings, not errors.

Use `--ci` mode to suppress warnings in automated checks.

---

## Development

### Adding New Validation Rules

Edit `validate.mjs` and add to the `validateTranslation()` function:

```javascript
// Example: Check for excessive length
if (targetValue.length > sourceValue.length * 2) {
  errors.push({
    severity: 'warning',
    code: 'EXCESSIVE_LENGTH',
    message: `Translation much longer than source (${targetValue.length} vs ${sourceValue.length} chars)`
  });
}
```

### Testing Scripts Locally

```bash
# Test on a copy to avoid modifying files
cp -r website/src/i18n/locales website/src/i18n/locales.backup

# Run script
node scripts/i18n/sync-keys.mjs

# Compare results
diff -r website/src/i18n/locales website/src/i18n/locales.backup

# Restore if needed
rm -rf website/src/i18n/locales
mv website/src/i18n/locales.backup website/src/i18n/locales
```

---

## Related Documentation

- [i18n System Overview](../../src/i18n/README.md)
- [Translation Guidelines](../../docs/i18n-guidelines.md)
- [Locale Tiers and Coverage](../../docs/i18n-tiers.md)

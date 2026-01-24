# i18n Scripts

Translation validation and coverage scripts for the Caro website.

## Scripts

### `status.mjs`

Shows translation coverage statistics for all locales.

**Usage:**

```bash
# Show coverage for all locales
node scripts/i18n/status.mjs

# CI mode with minimum coverage threshold
node scripts/i18n/status.mjs --ci --min-coverage=80

# Check only Tier 1 locales (es, fr, pt, de, ja)
node scripts/i18n/status.mjs --tier1-only

# Combine flags for CI validation
node scripts/i18n/status.mjs --ci --min-coverage=80 --tier1-only
```

**Exit codes:**
- `0`: All locales meet coverage requirements
- `1`: One or more locales below threshold (in CI mode)

**Output:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌍 Translation Coverage Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────┬──────┬──────────┬──────────┬─────────────┐
│ Locale  │ Tier │ Keys     │ Files    │ Coverage    │
├─────────┼──────┼──────────┼──────────┼─────────────┤
│ ✓ es    │ TIER1 │ 327/313 │ 10/10      │ 104.5%     │
│ ✓ fr    │ TIER1 │ 327/313 │ 10/10      │ 104.5%     │
...
```

### `validate.mjs`

Validates translation files for common issues:
- JSON syntax errors
- Placeholder preservation (`{variable}` placeholders)
- Brand name preservation (Caro, Claude, etc.)
- Missing or extra keys

**Usage:**

```bash
# Run validation with warnings
node scripts/i18n/validate.mjs

# CI mode (exit 1 on errors)
node scripts/i18n/validate.mjs --ci
```

**Exit codes:**
- `0`: No validation errors (warnings allowed)
- `1`: Validation errors found

**What it checks:**

1. **Placeholders**: Ensures `{count}`, `{name}`, etc. are preserved in translations
2. **Brand names**: Warns if "Caro", "Claude", "GitHub" etc. are missing
3. **Key consistency**: Detects missing or extra translation keys
4. **JSON validity**: Ensures all files parse correctly

## Tier System

Locales are grouped by priority:

- **TIER1** (es, fr, pt, de, ja): Target 95% coverage - Primary markets
- **TIER2** (ko, he, ar, hi): Target 90% coverage - Secondary markets
- **TIER3** (ru, uk, ur, fil, id): Target 80% coverage - Emerging markets

## Integration

These scripts are used by:

1. **`.github/workflows/validate-translations.yml`**
   - Runs on every PR that modifies locale files
   - Validates translations and checks coverage
   - Blocks merge if Tier 1 locales below 80%

2. **`.github/workflows/translate.yml`**
   - Automated translation workflow
   - Triggers weekly (Sunday 3am UTC)
   - Creates PRs with updated translations

## Development

### Adding New Validation Rules

Edit `validate.mjs` to add custom checks:

```javascript
// Example: Check for untranslated English words
for (const [key, localeValue] of Object.entries(localeKeys)) {
  if (localeValue.includes('TODO')) {
    warnings.push(`[${locale}/${filename}] TODO found in: ${key}`);
  }
}
```

### Changing Coverage Thresholds

Thresholds are set at the workflow level:

```yaml
# .github/workflows/validate-translations.yml
- name: Check Tier 1 Coverage
  run: node scripts/i18n/status.mjs --ci --min-coverage=80 --tier1-only
```

## Troubleshooting

### "Cannot find module"

Make sure you're in the `website/` directory:

```bash
cd website
node scripts/i18n/status.mjs
```

### High warning count

Warnings are normal during development. They indicate:
- Missing translations (not yet completed)
- Extra keys (old translations not removed)
- Brand name variations

Only **errors** block CI/CD pipelines.

### Coverage over 100%

This happens when translations have more keys than English (extra translations). Not an error, but indicates extra keys that could be removed.

## See Also

- [Phase 8 Implementation Plan](../../../.claude/docs/i18n-phase-8-plan.md)
- [Translation Rules](../../src/i18n/translation-rules.ts)
- [Cultural Contexts](../../src/i18n/cultural-contexts.ts)

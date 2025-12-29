---
work_package_id: "WP07"
subtasks:
  - "T028"
  - "T029"
  - "T030"
  - "T031"
  - "T032"
  - "T033"
  - "T034"
title: "GitHub Action Automation"
phase: "Phase 5 - GitHub Action"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "98542"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2025-12-29T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP07 – GitHub Action Automation

## Objectives

Automated translation workflow via OpenAI GPT-4.

**Success**: Changing English JSON triggers GitHub Action that creates PR with translations for all 14 locales.

---

## Subtasks

### T028 – Create Workflow YAML

Follow `contracts/github-action-workflow.yml`:

```yaml
name: Translate Website Content
on:
  push:
    branches: [main]
    paths:
      - 'website/src/i18n/locales/en/**/*.json'
  workflow_dispatch:
    inputs:
      force_retranslate:
        type: boolean
        default: false

permissions:
  contents: write
  pull-requests: write

jobs:
  translate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        locale: [es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id]
      max-parallel: 3
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-node@v6
        with:
          node-version: 20
      - run: npm install openai
      - run: node .github/scripts/translate.js
        env:
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
          TARGET_LOCALE: ${{ matrix.locale }}
```

### T029 – Create Translation Script

`.github/scripts/translate.js`:

```javascript
const OpenAI = require('openai');
const fs = require('fs');

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const targetLocale = process.env.TARGET_LOCALE;

async function translate() {
  // Read English JSON
  const enFiles = fs.readdirSync('website/src/i18n/locales/en');

  for (const file of enFiles) {
    const enContent = JSON.parse(fs.readFileSync(`locales/en/${file}`));

    // Call GPT-4
    const response = await openai.chat.completions.create({
      model: 'gpt-4',
      messages: [{
        role: 'system',
        content: `Translate JSON to ${targetLocale}. PRESERVE placeholders like {count}, {name}. PRESERVE brand names like "Caro". Technical terms: POSIX, shell, CLI.`
      }, {
        role: 'user',
        content: JSON.stringify(enContent)
      }]
    });

    const translated = JSON.parse(response.choices[0].message.content);
    fs.writeFileSync(`locales/${targetLocale}/${file}`, JSON.stringify(translated, null, 2));
  }
}

translate().catch(console.error);
```

### T030-T033 – Add Features

- T030: OpenAI SDK integration (done in T029)
- T031: Matrix strategy (done in T028)
- T032: Add caching for translations
- T033: PR creation with `peter-evans/create-pull-request@v6`

### T034 – Test Workflow

1. Add test string to `locales/en/common.json`
2. Push to main
3. Verify GitHub Action runs
4. Verify PR created with translations

---

## Test

```bash
# Local test (requires OPENAI_API_KEY)
export OPENAI_API_KEY="sk-..."
export TARGET_LOCALE="es"
node .github/scripts/translate.js
# Verify locales/es/*.json created
```

---

## Activity Log

- 2025-12-29T00:00:00Z – system – lane=planned – Prompt created
- 2025-12-29T10:54:50Z – claude – shell_pid=93883 – lane=doing – Starting GitHub Action automation for translations
- 2025-12-29T11:02:27Z – claude – shell_pid=98542 – lane=for_review – Completed all 7 subtasks - GitHub Action workflow, translation script with caching, and testing guide

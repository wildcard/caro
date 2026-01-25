# Visual Regression Testing - Design Requirements Specification

> **Document Type**: DRS
> **Version**: 1.0.0
> **Status**: Active
> **Parent**: [AUTOMATED_DEV_FLOW_DRS.md](./AUTOMATED_DEV_FLOW_DRS.md)
> **Pack**: Technical

---

## 1. Overview

Visual Regression Testing captures screenshots of key website pages and compares them against baselines to detect unintended visual changes.

### 1.1 Objectives

1. **Catch Visual Regressions**: Detect layout breaks, missing elements, styling issues
2. **Maintain Visual Quality**: Ensure consistent user experience
3. **Block Problematic PRs**: Prevent visual regressions from reaching production
4. **Track Visual Evolution**: Maintain history of visual changes

---

## 2. System Design

### 2.1 Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   VISUAL REGRESSION SYSTEM                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TRIGGERS                                                        │
│  ────────                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Nightly      │  │ PR Opened    │  │ Manual       │           │
│  │ (2 AM)       │  │              │  │ (/visual)    │           │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘           │
│         │                 │                 │                    │
│         └─────────────────┼─────────────────┘                    │
│                           │                                      │
│                           ▼                                      │
│                   ┌───────────────┐                              │
│                   │ Test Runner   │                              │
│                   │ (Playwright)  │                              │
│                   └───────┬───────┘                              │
│                           │                                      │
│         ┌─────────────────┼─────────────────┐                    │
│         ▼                 ▼                 ▼                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ Website     │  │ Docs Site   │  │ Storybook   │              │
│  │ (port 4321) │  │ (port 4322) │  │ (port 6006) │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │ Screenshot    │                               │
│                  │ Capture       │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │ Comparison    │                               │
│                  │ Engine        │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│              ┌───────────┴───────────┐                           │
│              ▼                       ▼                           │
│       ┌───────────┐           ┌───────────┐                      │
│       │ PASS      │           │ FAIL      │                      │
│       │ No diff   │           │ Diff > n% │                      │
│       └───────────┘           └─────┬─────┘                      │
│                                     │                            │
│                                     ▼                            │
│                             ┌───────────────┐                    │
│                             │ Report        │                    │
│                             │ Generator     │                    │
│                             └───────────────┘                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Test Matrix

```yaml
# .claude/automation/config/visual_test_matrix.yaml
test_matrix:
  sites:
    website:
      url: "http://localhost:4321"
      build_cmd: "npm run build && npm run preview"
      pages:
        - path: "/"
          name: "homepage"
          wait_for: ".hero-section"
        - path: "/roadmap"
          name: "roadmap"
          wait_for: ".timeline"
        - path: "/faq"
          name: "faq"
          wait_for: ".faq-list"
        - path: "/glossary"
          name: "glossary"
          wait_for: ".glossary-grid"
        - path: "/credits"
          name: "credits"
          wait_for: ".credits-section"

    docs:
      url: "http://localhost:4322"
      build_cmd: "npm run build && npm run preview"
      pages:
        - path: "/"
          name: "docs-home"
          wait_for: ".sidebar"
        - path: "/getting-started/installation"
          name: "installation"
          wait_for: ".content"

  viewports:
    desktop:
      width: 1920
      height: 1080
    tablet:
      width: 768
      height: 1024
    mobile:
      width: 375
      height: 812

  themes:
    - light
    - dark

  # Total screenshots = sites.pages * viewports * themes
  # = (5 + 2) * 3 * 2 = 42 screenshots
```

---

## 3. Execution Flow

### 3.1 Step-by-Step Process

```
1. SETUP
   │
   ├── Start dev servers (website, docs)
   ├── Wait for servers to be ready
   └── Initialize Playwright browser

2. CAPTURE SCREENSHOTS
   │
   For each site:
     For each page:
       For each viewport:
         For each theme:
           │
           ├── Navigate to page
           ├── Set viewport
           ├── Set color scheme
           ├── Wait for page load
           ├── Wait for animations
           └── Capture screenshot
               └── Save to: screenshots/current/{site}/{page}-{viewport}-{theme}.png

3. COMPARE WITH BASELINE
   │
   For each screenshot:
     │
     ├── Load baseline from: screenshots/baseline/{...}.png
     ├── If no baseline exists:
     │   └── Create baseline (first run)
     └── If baseline exists:
         │
         ├── Compare using pixelmatch
         ├── Calculate diff percentage
         └── If diff > threshold:
             ├── Save diff image
             └── Add to failures list

4. GENERATE REPORT
   │
   ├── Create HTML report with:
   │   ├── Summary (pass/fail counts)
   │   ├── For each failure:
   │   │   ├── Side-by-side comparison
   │   │   ├── Diff overlay
   │   │   └── Diff percentage
   │   └── Approve/reject buttons
   └── Save report to: visual-report/index.html

5. HANDLE RESULTS
   │
   ├── If all pass:
   │   └── Exit 0, no action needed
   └── If any fail:
       ├── Exit 1 (fail CI)
       ├── Post PR comment with summary
       └── Upload report as artifact
```

### 3.2 Screenshot Comparison Algorithm

```typescript
interface ComparisonResult {
  match: boolean;
  diffPixels: number;
  diffPercentage: number;
  diffImagePath?: string;
}

function compareScreenshots(
  current: Buffer,
  baseline: Buffer,
  options: {
    threshold: number;      // Per-pixel threshold (0-1)
    maxDiffPercentage: number;  // Max allowed diff %
  }
): ComparisonResult {
  // Use pixelmatch for comparison
  const diff = pixelmatch(current, baseline, diffOutput, width, height, {
    threshold: options.threshold,
    alpha: 0.1,
    diffColor: [255, 0, 0],
    aaColor: [255, 255, 0],
  });

  const diffPercentage = (diff / (width * height)) * 100;

  return {
    match: diffPercentage <= options.maxDiffPercentage,
    diffPixels: diff,
    diffPercentage,
    diffImagePath: diffPercentage > 0 ? saveDiffImage(diffOutput) : undefined,
  };
}
```

---

## 4. Configuration

### 4.1 Playwright Configuration

```typescript
// website/playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/visual',
  snapshotDir: './screenshots/baseline',
  outputDir: './visual-report',

  // Fail fast on first error
  maxFailures: process.env.CI ? 10 : 0,

  expect: {
    toHaveScreenshot: {
      // Per-pixel color difference threshold
      threshold: 0.2,
      // Max different pixels allowed
      maxDiffPixels: 100,
      // Max percentage difference allowed
      maxDiffPixelRatio: 0.01, // 1%
    },
  },

  use: {
    // Base URL for navigation
    baseURL: 'http://localhost:4321',
    // Capture screenshot on failure
    screenshot: 'only-on-failure',
    // Record trace on failure
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'desktop-light',
      use: {
        ...devices['Desktop Chrome'],
        colorScheme: 'light',
      },
    },
    {
      name: 'desktop-dark',
      use: {
        ...devices['Desktop Chrome'],
        colorScheme: 'dark',
      },
    },
    {
      name: 'tablet-light',
      use: {
        ...devices['iPad'],
        colorScheme: 'light',
      },
    },
    {
      name: 'tablet-dark',
      use: {
        ...devices['iPad'],
        colorScheme: 'dark',
      },
    },
    {
      name: 'mobile-light',
      use: {
        ...devices['iPhone 14'],
        colorScheme: 'light',
      },
    },
    {
      name: 'mobile-dark',
      use: {
        ...devices['iPhone 14'],
        colorScheme: 'dark',
      },
    },
  ],

  webServer: [
    {
      command: 'npm run preview',
      port: 4321,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
```

### 4.2 Automation Configuration

```yaml
# .claude/automation/config/visual_regression.yaml
visual_regression:
  enabled: true
  schedule: "0 2 * * *"  # Nightly 2 AM

  thresholds:
    per_pixel: 0.2        # Color difference threshold
    max_diff_pixels: 100  # Absolute pixel count
    max_diff_percent: 1.0 # Percentage threshold

  on_failure:
    block_pr: true
    create_issue: false  # Only create issue for nightly runs
    post_comment: true

  baseline:
    storage: "git"  # git or git-lfs
    auto_update: false  # Require manual approval

  retention:
    keep_reports: 30  # days
    keep_artifacts: 7  # days
```

---

## 5. Output Artifacts

### 5.1 Directory Structure

```
website/
├── screenshots/
│   ├── baseline/              # Git-tracked baseline images
│   │   ├── homepage-desktop-light.png
│   │   ├── homepage-desktop-dark.png
│   │   └── ...
│   └── current/               # Current run (gitignored)
│       └── ...
├── visual-report/             # HTML report (gitignored)
│   ├── index.html
│   └── assets/
│       ├── diff-homepage-desktop-light.png
│       └── ...
└── tests/
    └── visual/
        ├── homepage.spec.ts
        ├── roadmap.spec.ts
        └── ...
```

### 5.2 HTML Report Format

```html
<!DOCTYPE html>
<html>
<head>
  <title>Visual Regression Report - {date}</title>
</head>
<body>
  <h1>Visual Regression Report</h1>
  <p>Generated: {timestamp}</p>

  <section class="summary">
    <h2>Summary</h2>
    <p>Passed: {passed} | Failed: {failed} | Total: {total}</p>
    <p>Status: {status}</p>
  </section>

  <section class="failures">
    <h2>Failures</h2>
    <!-- For each failure -->
    <article class="failure">
      <h3>{page} - {viewport} - {theme}</h3>
      <p>Diff: {diff_percent}% ({diff_pixels} pixels)</p>
      <div class="comparison">
        <div class="baseline">
          <h4>Baseline</h4>
          <img src="baseline/{name}.png" />
        </div>
        <div class="current">
          <h4>Current</h4>
          <img src="current/{name}.png" />
        </div>
        <div class="diff">
          <h4>Difference</h4>
          <img src="diff/{name}.png" />
        </div>
      </div>
      <div class="actions">
        <button onclick="approve('{name}')">Approve Change</button>
        <button onclick="reject('{name}')">Reject</button>
      </div>
    </article>
  </section>
</body>
</html>
```

---

## 6. CI/CD Integration

### 6.1 GitHub Actions Workflow

```yaml
# .github/workflows/visual-regression.yml
name: Visual Regression

on:
  pull_request:
    paths:
      - 'website/**'
      - 'docs-site/**'
  schedule:
    - cron: '0 2 * * *'  # Nightly 2 AM
  workflow_dispatch:

jobs:
  visual-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: |
          cd website && npm ci
          cd ../docs-site && npm ci

      - name: Install Playwright browsers
        run: cd website && npx playwright install chromium

      - name: Build sites
        run: |
          cd website && npm run build
          cd ../docs-site && npm run build

      - name: Run visual tests
        run: cd website && npm run test:visual

      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-regression-report
          path: website/visual-report/

      - name: Comment on PR
        if: failure() && github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '## Visual Regression Failed\n\nScreenshots differ from baseline. [View Report](${artifact_url})'
            })
```

---

## 7. Skill Interface

### 7.1 Commands

```
/visual-regression-test run          # Run visual tests
/visual-regression-test update       # Update baselines
/visual-regression-test report       # Open last report
/visual-regression-test compare <pr> # Compare PR branch vs main
```

---

## 8. Related Documents

- [VISUAL_REGRESSION_TEST.md](../tests/VISUAL_REGRESSION_TEST.md) - Test cases
- [Playwright Documentation](https://playwright.dev/docs/test-snapshots)

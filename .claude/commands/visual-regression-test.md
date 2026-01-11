# Visual Regression Testing

Automated visual regression testing for website and documentation. Captures screenshots across pages, viewports, and themes, then compares with baselines to detect unintended visual changes.

## Usage

```
/visual-regression-test <command> [options]
```

## Commands

### run - Run visual tests

```
/visual-regression-test run [--pages <list>] [--update-baseline]
```

### update - Update baseline images

```
/visual-regression-test update [--all|--failed]
```

### report - View last report

```
/visual-regression-test report
```

### compare - Compare branch vs main

```
/visual-regression-test compare <branch>
```

## Process

### 1. Setup

Start development servers:

```bash
# Website
cd website && npm run build && npm run preview &
# Wait for port 4321

# Docs site (optional)
cd docs-site && npm run build && npm run preview &
# Wait for port 4322
```

### 2. Configure Test Matrix

```yaml
# Pages to test
pages:
  - path: "/"
    name: "homepage"
    wait_for: ".hero-section"
  - path: "/roadmap"
    name: "roadmap"
  - path: "/faq"
    name: "faq"

# Viewports
viewports:
  - desktop: 1920x1080
  - tablet: 768x1024
  - mobile: 375x812

# Themes
themes:
  - light
  - dark

# Total: 5 pages × 3 viewports × 2 themes = 30 screenshots
```

### 3. Capture Screenshots

Using Playwright:

```typescript
for (const page of pages) {
  for (const viewport of viewports) {
    for (const theme of themes) {
      await page.goto(url);
      await page.setViewportSize(viewport);
      await page.emulateMedia({ colorScheme: theme });
      await page.waitForSelector(page.waitFor);
      await page.screenshot({
        path: `screenshots/current/${name}-${viewport}-${theme}.png`,
        fullPage: true
      });
    }
  }
}
```

### 4. Compare with Baseline

For each screenshot:

```typescript
const diff = pixelmatch(current, baseline, diffOutput, width, height, {
  threshold: 0.2,
  alpha: 0.1,
});

const diffPercent = (diff / (width * height)) * 100;

if (diffPercent > 1.0) {
  failures.push({
    name,
    diffPercent,
    diffImage: saveDiff(diffOutput)
  });
}
```

### 5. Generate Report

Create HTML report at `website/visual-report/index.html`:

```html
<h1>Visual Regression Report</h1>
<p>Date: 2026-01-11</p>
<p>Status: 2 failures</p>

<section class="failures">
  <article>
    <h3>homepage-mobile-dark</h3>
    <p>Diff: 2.3%</p>
    <div class="comparison">
      <img src="baseline/..." alt="Baseline" />
      <img src="current/..." alt="Current" />
      <img src="diff/..." alt="Difference" />
    </div>
  </article>
</section>
```

### 6. Handle Results

**All Pass:**
- Log success
- Update state

**Failures:**
- Generate diff images
- Create HTML report
- If in CI: fail the build
- If on PR: post comment with summary

## Example Session

```
> /visual-regression-test run

Visual Regression Testing
═════════════════════════

Starting servers...
  ✓ Website: http://localhost:4321
  ✓ Docs: http://localhost:4322

Capturing screenshots...
  [1/30] homepage-desktop-light ✓
  [2/30] homepage-desktop-dark ✓
  [3/30] homepage-tablet-light ✓
  ...
  [30/30] faq-mobile-dark ✓

Comparing with baseline...
  homepage-desktop-light: ✓ Match
  homepage-desktop-dark: ✓ Match
  homepage-tablet-light: ⚠ Diff 0.5% (below threshold)
  roadmap-mobile-dark: ✗ Diff 2.3% (above threshold)
  ...

Results:
  Passed: 29
  Failed: 1
  Threshold: 1.0%

Failed Screenshots:
  roadmap-mobile-dark (2.3% diff)

Report: file://./website/visual-report/index.html

> /visual-regression-test update --failed

Updating baseline for failed screenshots...
  ✓ roadmap-mobile-dark updated

Baseline updated. Commit these changes:
  git add website/screenshots/baseline/
  git commit -m "chore: update visual regression baselines"
```

## CI Integration

### GitHub Actions

```yaml
# .github/workflows/visual-regression.yml
name: Visual Regression

on:
  pull_request:
    paths:
      - 'website/**'

jobs:
  visual-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup
        run: |
          cd website && npm ci
          npx playwright install chromium

      - name: Run visual tests
        run: npx playwright test tests/visual/

      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-report
          path: website/visual-report/
```

## Configuration

### Playwright Config

```typescript
// website/playwright.config.ts
export default defineConfig({
  snapshotDir: './screenshots/baseline',
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01,  // 1% threshold
    },
  },
  projects: [
    { name: 'desktop-light', use: { colorScheme: 'light' } },
    { name: 'desktop-dark', use: { colorScheme: 'dark' } },
    // ...
  ],
});
```

### Automation Config

```yaml
# .claude/automation/config/visual_regression.yaml
visual_regression:
  enabled: true
  schedule: "0 2 * * *"  # Nightly

  thresholds:
    max_diff_percent: 1.0

  on_failure:
    block_pr: true
    create_issue: false
```

## File Structure

```
website/
├── screenshots/
│   ├── baseline/          # Git-tracked baselines
│   │   ├── homepage-desktop-light.png
│   │   └── ...
│   └── current/           # gitignored
│       └── ...
├── visual-report/         # gitignored
│   ├── index.html
│   └── diff/
│       └── ...
└── tests/
    └── visual/
        ├── homepage.spec.ts
        └── roadmap.spec.ts
```

## Related

- [VISUAL_REGRESSION_DRS.md](../.claude/automation/specs/VISUAL_REGRESSION_DRS.md)
- [Playwright Visual Comparisons](https://playwright.dev/docs/test-snapshots)

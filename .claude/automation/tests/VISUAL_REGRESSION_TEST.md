# Visual Regression Testing - Test Document

> **Document Type**: Test Specification
> **DRS Reference**: [VISUAL_REGRESSION_DRS.md](../specs/VISUAL_REGRESSION_DRS.md)
> **Skill**: `/visual-regression-test`

---

## 1. Test Categories

### 1.1 Unit Tests

| ID | Test Case | Input | Expected Output |
|----|-----------|-------|-----------------|
| VR-U01 | Image comparison (match) | Identical images | diff = 0 |
| VR-U02 | Image comparison (diff) | Different images | diff > 0 |
| VR-U03 | Threshold check | 0.5% diff, 1% threshold | Pass |
| VR-U04 | Threshold exceed | 2% diff, 1% threshold | Fail |
| VR-U05 | Missing baseline | No baseline file | Create baseline |

### 1.2 Integration Tests

| ID | Test Case | Setup | Expected |
|----|-----------|-------|----------|
| VR-I01 | Server startup | Website built | Port 4321 responds |
| VR-I02 | Screenshot capture | Page loaded | PNG file created |
| VR-I03 | Viewport switching | Multiple viewports | Correct dimensions |
| VR-I04 | Theme switching | Light/dark | Correct color scheme |
| VR-I05 | Report generation | Failures exist | HTML report created |

### 1.3 End-to-End Tests

| ID | Test Case | Scenario | Validation |
|----|-----------|----------|------------|
| VR-E01 | Full matrix run | All pages, viewports, themes | 30 screenshots |
| VR-E02 | Baseline update | Update flag set | Baselines updated |
| VR-E03 | CI failure mode | Diff exceeds threshold | Exit code 1 |
| VR-E04 | Report accuracy | Multiple failures | All shown in report |

---

## 2. Test Pages

| Page | URL | Wait For | Notes |
|------|-----|----------|-------|
| Homepage | `/` | `.hero-section` | Above fold content |
| Roadmap | `/roadmap` | `.timeline` | Dynamic content |
| FAQ | `/faq` | `.faq-list` | Accordion elements |
| Glossary | `/glossary` | `.glossary-grid` | Grid layout |
| Credits | `/credits` | `.credits-section` | Static content |

---

## 3. Viewport Matrix

| Name | Width | Height | Type |
|------|-------|--------|------|
| desktop | 1920 | 1080 | Large screen |
| tablet | 768 | 1024 | iPad |
| mobile | 375 | 812 | iPhone 14 |

---

## 4. Theme Matrix

| Theme | CSS Property | Expected |
|-------|--------------|----------|
| light | color-scheme | light |
| dark | color-scheme | dark |

---

## 5. Validation Checklist

- [ ] All pages load without error
- [ ] All viewports render correctly
- [ ] Both themes apply correctly
- [ ] Screenshots are consistent (no animation artifacts)
- [ ] Diff algorithm is accurate
- [ ] Report is navigable
- [ ] Baselines can be updated

---

## 6. Manual Testing Steps

1. **Setup**
   ```bash
   cd website
   npm run build
   npm run preview &
   ```

2. **Run Tests**
   ```bash
   npx playwright test tests/visual/
   ```

3. **View Report**
   ```bash
   open visual-report/index.html
   ```

4. **Update Baselines**
   ```bash
   npx playwright test --update-snapshots
   ```

---

## 7. Performance Requirements

| Metric | Target | Maximum |
|--------|--------|---------|
| Total execution time | < 5 min | 15 min |
| Screenshot capture | < 5 sec each | 15 sec |
| Comparison | < 1 sec each | 5 sec |

# Automation Tests Index

> **Purpose**: Index of all test documents for automated development flows
> **Last Updated**: 2026-01-11

---

## Test Documents by Pack

### Technical Pack

| Flow | Test Document | Status |
|------|---------------|--------|
| QA Automation Loop | [QA_LOOP_TEST.md](./QA_LOOP_TEST.md) | ✅ Complete |
| Visual Regression | [VISUAL_REGRESSION_TEST.md](./VISUAL_REGRESSION_TEST.md) | ✅ Complete |
| Sync Engine | Uses existing `/caro.sync` tests | ✅ Existing |

### Content Pack

| Flow | Test Document | Status |
|------|---------------|--------|
| Idea Sourcing | [IDEA_SOURCING_TEST.md](./IDEA_SOURCING_TEST.md) | 📋 Planned |
| Social Queue | [SOCIAL_QUEUE_TEST.md](./SOCIAL_QUEUE_TEST.md) | 📋 Planned |

### Management Pack

| Flow | Test Document | Status |
|------|---------------|--------|
| PR Management | [PR_MANAGEMENT_TEST.md](./PR_MANAGEMENT_TEST.md) | 📋 Planned |
| Stale Revival | [STALE_REVIVAL_TEST.md](./STALE_REVIVAL_TEST.md) | 📋 Planned |
| Metrics Collection | [METRICS_TEST.md](./METRICS_TEST.md) | 📋 Planned |

---

## Test Execution

### Run All Tests

```bash
# From automation directory
/automation/orchestrate test all
```

### Run by Pack

```bash
/automation/orchestrate test technical
/automation/orchestrate test content
/automation/orchestrate test management
```

### Run Specific Test

```bash
/automation/orchestrate test qa_loop
```

---

## Test Coverage Goals

| Pack | Coverage Target | Current |
|------|-----------------|---------|
| Technical | 80% | TBD |
| Content | 70% | TBD |
| Management | 75% | TBD |

---

## CI Integration

Tests run in GitHub Actions:

```yaml
# .github/workflows/automation-tests.yml
name: Automation Tests

on:
  push:
    paths:
      - '.claude/automation/**'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run automation tests
        run: /automation/orchestrate test all --ci
```

---

## Test Data Location

```
.claude/automation/tests/
├── fixtures/           # Test fixtures
│   ├── profiles/       # Mock profiles
│   ├── findings/       # Sample findings
│   └── screenshots/    # Test screenshots
├── mocks/              # API mocks
│   ├── github.json     # GitHub API responses
│   └── rss.xml         # RSS feed mocks
└── *.md                # Test documents
```

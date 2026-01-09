# Keeble: Caro Dogfooding Framework

This directory contains the infrastructure for Keeble - our internal dogfooding program where we use Caro to build Caro.

## Directory Contents

| File | Purpose |
|------|---------|
| `friction-log.yaml` | Track suboptimal command generations |
| `success-log.yaml` | Track excellent command generations |
| `README.md` | This file |

## Quick Links

- [Full Keeble Strategy](/docs/KEEBLE_DOGFOODING.md) - Comprehensive dogfooding plan
- [Quickstart Guide](/docs/KEEBLE_QUICKSTART.md) - Get started with dogfooding

## How to Use

### Reporting Friction

When Caro generates a suboptimal command:

1. Add entry to `friction-log.yaml`
2. Include: intent, expected, actual, severity
3. Continue working (don't block on fixing)

### Celebrating Success

When Caro generates an excellent command:

1. Add entry to `success-log.yaml`
2. Include: intent, generated, quality level
3. Flag for potential harvest to static patterns

### Weekly Review

Every Friday:

1. Review friction log entries
2. Prioritize fixes
3. Harvest successful patterns
4. Update metrics

## Schema Versions

Both YAML files follow schema version 1.0. Future changes will increment the version for backward compatibility.

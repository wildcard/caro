# Modulization Skill

> Cadence-driven work moderation and recomposition

Modulization is a sophisticated agent that runs on a regular cadence to discover scattered, incomplete, and drifting work across your project, then recomposes it into coherent **modules** — units of work that can be scheduled, spec'd, and delivered together.

## Quick Start

```bash
# Run modulization scan
/modulization

# Or trigger specific phrases
"Run weekly work moderation"
"What work is floating?"
"Prepare for milestone v1.1.0"
```

## Directory Structure

```
modulization/
├── SKILL.md              # Main skill definition
├── README.md             # This file
├── references/
│   ├── module-schema.md          # JSON/YAML schema for modules
│   ├── decision-heuristics.md    # Classification logic
│   ├── signal-collectors.md      # Collection strategies
│   └── spec-kitty-integration.md # Spec-driven workflow
├── examples/
│   └── sample-report.md          # Real example from Caro
└── scripts/
    └── (automation utilities)
```

## Configuration

Configuration lives at `.modulization/config.yaml`. See the config file for all options.

## GitHub Action

The workflow at `.github/workflows/modulization.yml` runs Modulization on a weekly cadence:

- **Schedule:** Every Monday at 9 AM UTC
- **Manual Trigger:** Available via workflow_dispatch
- **Post-Release:** Can trigger after releases

## Outputs

- **Reports:** `.modulization/reports/YYYY-MM-DD-weekly.md`
- **Spec Seeds:** `.modulization/spec-seeds/MOD-YYYY-NNN.yaml`
- **Module Data:** `.modulization/modules.yaml`

## Key Concepts

### Module
A coherent, self-contained unit of unfinished work grouped by intent and surface area.

### Classification
- **Integrate Now:** Aligned with current milestone, low cost
- **Schedule:** Important but needs planning
- **Ice:** Valid idea, wrong time
- **Archive:** Obsolete or superseded

### Spec Seeds
Pre-formatted inputs for `/spec-kitty.specify` that preserve context and accelerate specification.

## References

- [Module Schema](./references/module-schema.md)
- [Decision Heuristics](./references/decision-heuristics.md)
- [Signal Collectors](./references/signal-collectors.md)
- [Spec-Kitty Integration](./references/spec-kitty-integration.md)

## Related Skills

- `continuity_ledger` — State preservation
- `create_handoff` — Work transfer
- `quality-engineer-manager` — Release validation
- `beta-test-cycles` — Testing improvement

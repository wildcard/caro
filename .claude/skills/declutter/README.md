# Declutter Skill

Organize your project structure following open-source best practices.

## Quick Start

Say: **"Declutter this project"**

The skill will:
1. Survey your current structure
2. Identify files that should be moved
3. Consult specialized skills for guidance
4. Present a reorganization proposal
5. Execute moves with your approval

## Philosophy

A well-organized project should have a **clean, hand-sized root** that immediately communicates what the project is and how it's structured.

```
project/
├── docs/       # Documentation
├── scripts/    # Automation
├── src/        # Source code
├── tests/      # Tests
├── README.md   # Entry point
├── LICENSE     # Legal
└── Cargo.toml  # Config
```

## What It Does

- **Identifies** misplaced files
- **Classifies** everything by type
- **Researches** best practices via agents
- **Proposes** reorganization
- **Delegates** refactoring to specialized skills

## What It Does NOT Do

- **Does not** refactor code (delegates to `refactor` skill)
- **Does not** delete without asking
- **Does not** override project conventions
- **Does not** move without explanation

## Structure

```
declutter/
├── SKILL.md                    # Full workflow documentation
├── README.md                   # This file
├── references/
│   ├── oss-conventions.md      # OSS project structure patterns
│   └── skill-delegation.md     # When/how to delegate
└── examples/
    ├── cluttered-rust-project.md   # Rust cleanup example
    └── monorepo-declutter.md       # Mono-repo example
```

## Target Root Count

| Count | Status |
|-------|--------|
| ≤10 | Excellent |
| 10-20 | Good |
| 20-30 | Needs attention |
| 30+ | Critical |

## Trigger Phrases

- "Declutter this project"
- "Organize the file structure"
- "Clean up the root"
- "This project is messy"
- "Where should this file go?"

## Dependencies

**Required**: None (standalone capable)

**Recommended**:
- `refactor` skill - For code reference updates
- `technical-writer` agent - For docs restructuring
- `Explore` agent - For convention research

---

*A decluttered project is a welcoming project.*

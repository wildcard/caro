# cmdai Documentation

This directory contains organized documentation for the cmdai project. Standard files (README, CONTRIBUTING, LICENSE, etc.) remain in the project root, while specialized documentation is organized here by category.

## Directory Structure

```
docs/
├── README.md              # This file
├── legal/                 # Legal and contributor agreements
│   ├── CLA.md            # Individual Contributor License Agreement
│   ├── DCO.txt           # Developer Certificate of Origin
│   └── CLA_RESEARCH.md   # Research on CLA/DCO for AGPL projects
├── development/           # Development processes and guidelines
│   ├── CLAUDE.md         # AI agent instructions for Claude Code
│   ├── AGENTS.md         # Multi-agent development guidelines
│   ├── TDD-WORKFLOW.md   # Test-Driven Development workflow
│   └── TECH_DEBT.md      # Technical debt tracking and good first issues
└── qa-test-cases.md      # Quality assurance test cases
```

## Quick Links

### For Contributors

- **Start Here**: [CONTRIBUTING.md](../CONTRIBUTING.md) in project root
- **Contributor Agreements**: Choose [CLA](legal/CLA.md) or [DCO](legal/DCO.txt)
- **Development Workflow**: [TDD-WORKFLOW.md](development/TDD-WORKFLOW.md)
- **Good First Issues**: [TECH_DEBT.md](development/TECH_DEBT.md)

### For Maintainers

- **Agent Guidelines**: [AGENTS.md](development/AGENTS.md)
- **AI Development**: [CLAUDE.md](development/CLAUDE.md)
- **Test Cases**: [qa-test-cases.md](qa-test-cases.md)

### Legal & Licensing

- **Project License**: [LICENSE](../LICENSE) (AGPL-3.0)
- **CLA Documentation**: [legal/CLA.md](legal/CLA.md)
- **CLA Research**: [legal/CLA_RESEARCH.md](legal/CLA_RESEARCH.md)
- **DCO Alternative**: [legal/DCO.txt](legal/DCO.txt)

## Project Root Files (Standard OSS)

The following standard open-source files remain in the project root:

- **README.md** - Project overview and getting started
- **LICENSE** - AGPL-3.0 license text
- **CONTRIBUTING.md** - Contribution guidelines
- **CODE_OF_CONDUCT.md** - Community standards
- **SECURITY.md** - Security policy and vulnerability reporting
- **CHANGELOG.md** - Version history and release notes

## Additional Documentation

- **Specifications**: `/specs/` directory contains feature specs, plans, and contracts
- **Test Documentation**: Integration and contract test documentation in `/tests/`
- **Architecture**: See [CLAUDE.md](development/CLAUDE.md) for project structure details

## Contributing to Documentation

Documentation improvements are always welcome! When adding new documentation:

1. **Standard files** (README, CONTRIBUTING, etc.) → Project root
2. **Legal/licensing** → `docs/legal/`
3. **Development processes** → `docs/development/`
4. **Feature specifications** → `/specs/[feature-id]/`
5. **Test documentation** → Alongside relevant tests

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the full contribution process.

---

**Last Updated**: December 19, 2025

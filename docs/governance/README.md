# Governance Documentation

This directory contains governance-related documentation for the cmdai project, including policies, guidelines, and governance templates.

## Overview

Governance in cmdai operates at two levels:

1. **Community Governance**: How the cmdai project itself is governed (contributor guidelines, decision-making processes, code of conduct)
2. **User Governance**: Templates and examples for users to govern their own cmdai usage (safety policies, command patterns)

## Directory Structure

```
governance/
├── README.md                    # This file
├── community/                   # Community governance (future)
│   ├── decision-process.md     # How we make decisions
│   ├── contributor-roles.md    # Roles and responsibilities
│   └── roadmap-planning.md     # How roadmap is planned
└── templates/                   # Governance policy templates (future)
    ├── startup-friendly.yaml   # Low-friction policies for startups
    ├── enterprise-strict.yaml  # High-security enterprise policies
    ├── fintech-baseline.yaml   # Financial services compliance
    └── healthcare-hipaa.yaml   # HIPAA-compliant policies
```

## Community Governance (Future)

**Purpose**: Define how the cmdai project is governed as an open-source community

**Topics to cover**:
- Decision-making process (RFCs, voting, consensus)
- Contributor roles (maintainer, committer, contributor)
- Code review and merge guidelines
- Release process and versioning
- Community advisory board
- Conflict resolution

**Status**: To be developed as community grows

## User Governance Templates

**Purpose**: Provide starting points for users to define their own cmdai safety and governance policies

### Template Categories

**1. Risk-Based Templates**:
- `startup-friendly.yaml`: High velocity, medium risk tolerance
- `enterprise-strict.yaml`: Low risk, high compliance, conservative
- `balanced-security.yaml`: Moderate risk, good for most organizations

**2. Industry-Specific Templates**:
- `fintech-baseline.yaml`: Financial services (SOC2, PCI-DSS)
- `healthcare-hipaa.yaml`: Healthcare (HIPAA compliance)
- `government-security.yaml`: Government/defense (strict controls)
- `saas-standard.yaml`: SaaS companies (cloud-native)

**3. Use-Case Specific Templates**:
- `ai-agent-supervision.yaml`: Policies for autonomous agents
- `production-access.yaml`: High-risk production environment access
- `developer-sandbox.yaml`: Permissive policies for dev environments
- `ci-cd-pipeline.yaml`: Automated pipeline safety

### Template Format

All templates follow the same YAML schema:

```yaml
# Template metadata
template:
  name: "Template Name"
  version: "1.0"
  author: "author-name"
  license: "MIT"
  description: "What this template is for and who should use it"
  compliance_frameworks: ["SOC2", "ISO27001"]
  risk_tolerance: "low | medium | high"

# Safety rules
safety:
  risk_tolerance: "low"
  require_approval_for:
    - destructive_commands
    - privilege_escalation
  blocked_patterns:
    - pattern: "dangerous pattern"
      reason: "Why this is blocked"

# Tool allowlist
allowed_tools:
  - name: "tool-name"
    versions: [">=x.y.z"]
    reason: "Why this version is required"

# Additional sections...
```

### Using Templates

**Community Edition**:
1. Browse templates in `docs/governance/templates/`
2. Copy template to your local config directory
3. Customize for your needs
4. cmdai reads policy from `~/.cmdai/policy.yaml`

**Enterprise Edition** (Future):
1. Start with community template
2. Customize in CISO dashboard
3. Provision to all organization machines
4. Centralized enforcement and updates

### Contributing Templates

**We welcome community contributions!**

To contribute a governance template:

1. Fork the repository
2. Create your template in `docs/governance/templates/`
3. Follow the template format
4. Include clear description and use cases
5. Test template with cmdai community edition
6. Submit pull request with:
   - Template file
   - Example usage documentation
   - Any special considerations

**Quality Guidelines**:
- Clear, descriptive names
- Comprehensive comments explaining rules
- Tested with real-world usage
- No overly permissive policies (err on side of safety)
- No overly restrictive policies (must be usable)

### Template License

All governance templates are licensed under **MIT License** unless otherwise specified. This allows:
- Free use in commercial and non-commercial projects
- Modification and redistribution
- No warranty or liability

## Enterprise vs Community Governance

### Community Edition
- **User-owned**: Users define and manage their own policies
- **Opt-in**: Templates are suggestions, not requirements
- **Local**: Policies stored locally on user's machine
- **Flexible**: Users can modify or ignore templates

### Enterprise Edition (Future)
- **Organization-owned**: CISO defines organization-wide policies
- **Enforced**: Policies are mandatory, provisioned centrally
- **Global**: Policies distributed to all organization machines
- **Controlled**: Updates managed by security team

**Key Difference**: Community templates are educational and opt-in. Enterprise provisioning is mandatory and enforced.

## Future Enhancements

**Planned features**:

1. **Template Validation**: CLI tool to validate policy syntax
2. **Template Testing**: Framework to test policies against command sets
3. **Template Registry**: Searchable catalog of community templates
4. **Template Ratings**: Community voting and feedback
5. **Template Versioning**: Track template updates and changes
6. **Template Migration**: Tools to upgrade templates to new schema versions

## Questions?

- **For community governance**: Open an issue or discussion
- **For template contributions**: Submit a pull request
- **For enterprise governance**: See [Enterprise Documentation](../enterprise/)

---

*Last Updated: 2025-11-29*
*Status: Initial structure, templates to be added in future releases*

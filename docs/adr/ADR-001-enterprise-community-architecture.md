# ADR-001: Enterprise vs Community Architecture

**Status**: Proposed

**Date**: 2025-11-29

**Authors**: cmdai core team

**Target**: Hybrid (defines both Community and Enterprise)

## Context

cmdai has established itself as a powerful community-driven tool that empowers individual developers, system engineers, SREs, and DevOps professionals to leverage AI for safe command generation. The tool's core value proposition centers on:

- **User empowerment**: Helping terminal users work better and learn new tools
- **Safety-first approach**: Local governance and safety policies owned by individual users
- **Community-driven**: Open-source development for the greater good

However, enterprise organizations face different challenges when deploying AI-powered tools:

- **CISO requirements**: Need centralized governance and audit capabilities
- **Compliance mandates**: Must monitor and control what runs on provisioned machines
- **Risk management**: Cannot rely solely on individual user judgment
- **Scale challenges**: Difficult to govern hundreds or thousands of developers
- **Agent deployment**: Increasing use of autonomous agents requiring oversight

### Current Situation

Today's cmdai is optimized for individual users making their own safety decisions. There is no mechanism for:

- Centralized policy enforcement across an organization
- Audit trails for command execution and approval
- Provisioning of organization-wide governance rules
- Monitoring of developer machines for compliance
- Differentiation between approved and unapproved tools/versions

### Business Drivers

1. **Market opportunity**: Enterprise security and governance tooling market
2. **Revenue model**: Sustainable funding for continued open-source development
3. **Investor appeal**: Clear path to monetization while preserving community value
4. **Customer demand**: Organizations want to deploy cmdai but need governance
5. **Competitive positioning**: First-mover advantage in AI-powered command governance

### Constraints

- **Community preservation**: Must not compromise the open-source community edition
- **Plugin architecture**: Enterprise features should be separate, not invasive
- **Backward compatibility**: Community edition must continue to function independently
- **Open governance**: Community can define and share governance rules (non-provisioned)
- **Transparent pricing**: Clear value differentiation between Community and Enterprise

## Decision

We will architect cmdai as a **dual-offering system** with clear separation between Community and Enterprise editions:

### Community Edition (Open Source)
- Remains fully open-source under current license
- User-centric safety validation and governance
- User-owned policies and preferences
- Community-shared governance templates (opt-in)
- Full CLI functionality with all backends
- No provisioning or centralized monitoring
- Continues independent development and evolution

### Enterprise Edition (Premium Plugin/Suite)
- Developed and distributed as a for-profit offering
- Plugin or suite architecture that extends Community edition
- Centralized governance provisioning capability
- Organization-wide policy enforcement
- Audit trail and monitoring systems
- Machine and device correlation
- IT rollout integration
- CISO dashboard and controls
- Agent monitoring and rogue detection

### Integration Model

The Enterprise edition operates as a **non-invasive extension**:

```
┌─────────────────────────────────────┐
│     cmdai Community Edition         │
│  (Core CLI + Safety + Backends)     │
└──────────────┬──────────────────────┘
               │
               │ Plugin Interface
               │
┌──────────────▼──────────────────────┐
│   Enterprise Governance Plugin      │
│ - Policy Enforcement                │
│ - Audit Logging                     │
│ - Centralized Provisioning          │
│ - Monitoring Dashboard              │
└─────────────────────────────────────┘
```

### Key Architectural Principles

1. **Separation of Concerns**: Community and Enterprise code live in separate repositories/modules
2. **Opt-out by Default**: Enterprise features require explicit provisioning
3. **Plugin Architecture**: Enterprise capabilities injected via well-defined interfaces
4. **Non-breaking Changes**: Community edition evolution doesn't break Enterprise
5. **Open Standards**: Governance policy format is open and documented
6. **Community Contribution**: Community can contribute governance templates (non-provisioned)

## Rationale

### Why Dual Architecture?

1. **Sustainability**: Provides revenue stream to fund continued open-source development
2. **Community trust**: Keeps community edition fully independent and open
3. **Market fit**: Addresses legitimate enterprise security requirements
4. **Developer experience**: Enterprises can deploy cmdai with confidence
5. **Innovation space**: Each edition can evolve for its specific audience

### Why Plugin Model?

1. **Clean separation**: No enterprise code polluting community codebase
2. **Flexible deployment**: Enterprises only pay for and deploy what they need
3. **Testing isolation**: Changes in one edition don't break the other
4. **License clarity**: Clear boundary between open-source and proprietary code
5. **Extensibility**: Opens door for other plugins (e.g., industry-specific compliance)

### Alignment with Mission

This architecture preserves cmdai's mission to empower individual developers while acknowledging that enterprises need different tooling. The community edition remains the foundation, with enterprise features as optional extensions for organizations with compliance requirements.

## Consequences

### Benefits

- **Clear value proposition**: Community gets powerful free tool, enterprises get governance
- **Revenue generation**: Sustainable funding model for project growth
- **Community preservation**: Open-source edition remains fully functional and independent
- **Market expansion**: Can serve both individual developers and large enterprises
- **Innovation velocity**: Two parallel tracks of development
- **Talent attraction**: For-profit entity can hire full-time maintainers
- **Investor confidence**: Clear path to monetization and scale

### Trade-offs

- **Complexity**: Maintaining two editions requires coordination
- **Support burden**: Different support models for Community vs Enterprise
- **Feature parity**: Some features may exist only in Enterprise
- **Communication**: Must clearly articulate differences without alienating community
- **Development overhead**: Plugin interface must be well-designed and maintained
- **Testing matrix**: Both editions need comprehensive testing
- **Documentation**: Separate documentation for Community and Enterprise

### Risks

1. **Community backlash**: Fear of "rug pull" or feature gating
   - **Mitigation**: Clear communication, community governance, transparent roadmap

2. **Technical debt**: Plugin interface becomes limiting factor
   - **Mitigation**: Design plugin interface with extensibility in mind, iterate based on needs

3. **Market adoption**: Enterprises may not adopt or pay for premium features
   - **Mitigation**: Validate with early design partners, clear ROI documentation

4. **Competitive response**: Others may fork or create alternatives
   - **Mitigation**: Strong community relationships, continuous innovation, superior UX

5. **Talent retention**: Open-source contributors may feel excluded from Enterprise development
   - **Mitigation**: Offer contributor access to Enterprise features, transparent development process

## Alternatives Considered

### Alternative 1: Fully Open Source with Enterprise Support Model
- **Description**: Keep everything open-source, charge only for support/SLA
- **Pros**: Maximum community trust, no code separation needed
- **Cons**: Limited revenue potential, doesn't address provisioning needs, enterprises want owned features
- **Why not chosen**: Doesn't solve the core enterprise governance problem

### Alternative 2: Open Core with Premium Features in Main Repo
- **Description**: Add premium features to main codebase behind license checks
- **Pros**: Single codebase, easier to maintain
- **Cons**: Pollutes community code with enterprise concerns, license complexity, community perception issues
- **Why not chosen**: Violates clean separation principle, creates community trust issues

### Alternative 3: Fully Proprietary Fork
- **Description**: Create entirely separate proprietary version
- **Pros**: Total freedom for enterprise development
- **Cons**: Community loses governance innovations, duplicated effort, splits ecosystem
- **Why not chosen**: Abandons community, creates fragmentation

### Alternative 4: SaaS-Only Enterprise Offering
- **Description**: Host enterprise features as cloud service only
- **Pros**: Easier deployment, recurring revenue, no on-prem complexity
- **Cons**: Many enterprises require on-prem, data sovereignty issues, latency concerns
- **Why not chosen**: Market demands on-prem option, conflicts with local-first philosophy

## Implementation Notes

### Phase 1: Plugin Interface Design (Q1 2026)
- Define hook points for governance injection
- Create policy schema and evaluation engine interface
- Design audit event structure
- Establish plugin loading and lifecycle management

### Phase 2: Core Enterprise Features (Q2 2026)
- Build governance provisioning system
- Implement audit logging and event collection
- Create policy enforcement engine
- Develop monitoring agent for terminal injection

### Phase 3: CISO Dashboard and Management (Q3 2026)
- Build centralized dashboard for policy management
- Implement device and machine correlation
- Create alert and notification system
- Add reporting and analytics capabilities

### Phase 4: Enterprise Deployment Tools (Q4 2026)
- IT rollout automation and scripting
- Configuration management integration (Ansible, Chef, Puppet)
- MDM/EDM integration for policy distribution
- SSO and identity provider integration

### Testing Strategy
- **Community**: Continue current test suite, ensure no regression from plugin interface
- **Enterprise**: Separate test suite covering provisioning, monitoring, policy enforcement
- **Integration**: Test Enterprise plugin with various Community versions
- **Security**: Penetration testing, audit trail integrity verification

### Rollout Strategy
1. Announce architectural decision with clear communication
2. Engage design partners for early feedback
3. Beta program with select enterprise customers
4. General availability with tiered pricing
5. Continuous iteration based on customer feedback

## Success Metrics

### Community Edition
- **Adoption growth**: Maintain or increase user growth rate (target: 20% YoY)
- **Contributor retention**: No drop in active contributors (target: maintain current levels)
- **Community satisfaction**: Positive sentiment in surveys and discussions (target: >80% positive)

### Enterprise Edition
- **Revenue**: Achieve $1M ARR by end of Year 1 (target: 50 enterprise customers)
- **Customer satisfaction**: NPS score of 40+ for enterprise customers
- **Deployment scale**: Average 500+ seats per enterprise customer
- **Renewal rate**: 90%+ annual renewal rate

### Technical Metrics
- **Performance overhead**: Enterprise plugin adds <5% overhead to command execution
- **Reliability**: 99.9% uptime for enterprise services
- **Security**: Zero critical vulnerabilities in enterprise components
- **Compatibility**: Support latest 3 major versions of Community edition

## Business Implications

### Revenue Potential
- **Pricing model**: Tiered pricing based on seats and features
  - **Starter**: 10-50 seats, basic governance - $5K/year
  - **Professional**: 51-500 seats, full governance + monitoring - $50K/year
  - **Enterprise**: 500+ seats, custom deployment + premium support - $250K+/year
- **Market size**: Large enterprises (Fortune 2000) with developer teams
- **TAM**: Estimated $500M+ in enterprise developer tooling governance market

### Market Differentiation
- **First mover**: No existing AI command governance solution at enterprise scale
- **Trust advantage**: Built on proven open-source foundation
- **Safety heritage**: Industry-leading safety validation already built-in
- **Local-first**: Aligns with data sovereignty and security requirements
- **Multi-backend**: Works with various LLM providers, not locked to one

### Customer Pain Points Addressed
1. **Uncontrolled AI usage**: Developers using AI tools without oversight
2. **Compliance gaps**: No audit trail for AI-generated commands
3. **Security risks**: Malicious or accidental destructive commands
4. **Tool sprawl**: Ungoverned installation of development tools
5. **Agent supervision**: No monitoring of autonomous AI agents
6. **Incident response**: Can't trace back what happened on developer machines

### Competitive Positioning
- **vs. Generic MDM**: Purpose-built for developer workflow, not just policy enforcement
- **vs. ChatGPT Enterprise**: Focused on command safety, not general AI chat
- **vs. GitHub Copilot**: Terminal-focused, governance-first design
- **vs. Build-it-yourself**: Proven solution, faster time-to-value, continuous innovation

### Sales Enablement Considerations
- **Champion persona**: CISO, VP Engineering, DevOps Director
- **Economic buyer**: CIO, CFO
- **End user**: Developers, SREs, DevOps engineers (must not alienate)
- **Proof points**: Case studies showing reduced security incidents
- **ROI calculation**: Cost of one security incident vs. annual licensing cost

## References

- [cmdai Community Edition Repository](https://github.com/wildcard/cmdai)
- [CNCF Survey on Developer Tool Governance 2024](https://www.cncf.io/reports/)
- [Gartner: AI Governance Market 2025](https://www.gartner.com/)
- Related ADRs:
  - ADR-002: Governance and Provisioning System
  - ADR-003: Monitoring and Audit Trail System

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-11-29 | cmdai core team | Initial draft |

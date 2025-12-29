# ADR-002: Governance and Provisioning System

**Status**: Proposed

**Date**: 2025-11-29

**Authors**: cmdai core team

**Target**: Enterprise

**Depends on**: ADR-001 (Enterprise vs Community Architecture)

## Context

Organizations deploying cmdai across their developer workforce face a critical challenge: **how to enforce security policies and governance rules at scale without disrupting developer productivity**.

### The Enterprise Governance Challenge

When a CISO or security organization considers deploying AI-powered command generation tools, they need answers to:

1. **Policy Enforcement**: "How do I ensure developers only use approved tools and versions?"
2. **Safety Guardrails**: "Can I define organization-wide safety rules beyond the community defaults?"
3. **Compliance Requirements**: "How do I meet regulatory requirements like SOC2, ISO27001, HIPAA?"
4. **Tool Allowlisting**: "Can I control which programs are installable via cmdai?"
5. **Agent Governance**: "How do I govern autonomous agents running on provisioned machines?"
6. **Deployment Scale**: "How do I push policies to 1,000+ developer machines?"

### Current Community Model

The cmdai community edition provides:
- **User-owned safety rules**: Individual users configure their own safety preferences
- **Local policy files**: Policies stored in user's home directory or config location
- **No central enforcement**: Each user is their own governance authority
- **Opt-in governance templates**: Community can share safety templates, but users opt in

This works well for individual developers but doesn't meet enterprise requirements.

### The IT Deployment Reality

When enterprises deploy developer tools:
1. **IT provisions the environment**: Tools pre-installed on developer machines
2. **Policies are centrally managed**: Not user-configurable
3. **Compliance is mandatory**: Not optional or opt-in
4. **Changes roll out organization-wide**: Policy updates affect all machines
5. **Audit trails are required**: Must prove policy enforcement

### Business Context

CISOs are under increasing pressure regarding:
- **AI governance**: Board-level concern about uncontrolled AI usage
- **Supply chain security**: Controlling what software enters the organization
- **Insider threat**: Malicious or accidental destructive actions
- **Compliance audits**: Demonstrating control over developer environments
- **Shadow IT**: Developers installing unapproved tools

**Market opportunity**: Enterprise security spending on developer tooling governance is projected to exceed $2B annually by 2027.

## Decision

We will build a **centralized governance and provisioning system** as a core enterprise feature, consisting of:

### 1. Policy Definition Framework

A declarative, version-controlled policy schema that defines:

```yaml
# Example organization policy
version: "1.0"
organization: "acme-corp"
effective_date: "2025-12-01"

# Safety guardrails (beyond community defaults)
safety:
  risk_tolerance: "low"  # low | medium | high
  require_approval_for:
    - destructive_commands
    - privilege_escalation
    - network_operations

  blocked_patterns:
    - pattern: "curl .* | bash"
      reason: "Arbitrary code execution via pipe to shell"
    - pattern: "docker run --privileged"
      reason: "Privileged containers require security review"

  additional_dangerous_paths:
    - /data/production/*
    - /backup/*

# Tool allowlist
allowed_tools:
  - name: "git"
    versions: [">=2.30.0"]
    reason: "Versions below 2.30.0 have known vulnerabilities"

  - name: "kubectl"
    versions: [">=1.25.0", "<1.30.0"]
    approved_contexts: ["dev-*", "staging-*"]
    blocked_contexts: ["prod-*"]  # Requires separate approval

  - name: "terraform"
    versions: ["~1.6.0"]  # Only patch updates
    required_flags: ["--lock=true"]

  - name: "aws"
    versions: [">=2.0.0"]
    approved_profiles: ["dev", "staging"]

# Backend restrictions
inference:
  allowed_backends: ["ollama", "vllm"]
  blocked_backends: ["external-api"]  # Don't allow external API calls

  model_allowlist:
    - "llama-3.1-8b-instruct"
    - "codellama-13b"

  require_local: true  # All inference must be local

# Approval workflows
approvals:
  require_manager_approval:
    - high_risk_commands
    - production_database_operations

  auto_approve:
    - safe_commands
    - read_only_operations

# Audit requirements
audit:
  log_all_commands: true
  log_retention_days: 365
  send_to_siem: true
  siem_endpoint: "https://siem.acme-corp.com/events"

# Compliance tags
compliance:
  frameworks: ["SOC2", "ISO27001", "HIPAA"]
  data_classification: "confidential"
  require_mfa: true
```

### 2. Provisioning Distribution System

**Provisioning Mechanism**: How policies get to developer machines

```
┌─────────────────────────────────────┐
│   CISO / Security Org Dashboard     │
│   - Policy creation                 │
│   - Version management              │
│   - Rollout controls                │
└──────────────┬──────────────────────┘
               │
               │ Policy Publication
               │
┌──────────────▼──────────────────────┐
│   Policy Distribution Service       │
│   - Version control integration     │
│   - Signed policy artifacts         │
│   - Rollout orchestration           │
└──────────────┬──────────────────────┘
               │
               │ Pull/Push Models
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────┐          ┌─────▼───┐
│  Dev   │          │  Dev    │
│Machine │   ...    │ Machine │
│   1    │          │  1000   │
└────────┘          └─────────┘
```

**Distribution Options**:

1. **MDM/EDM Integration**: Push via Mobile Device Management
   - Jamf (macOS)
   - Intune (Windows/macOS)
   - Custom enterprise MDM

2. **Configuration Management**: Deploy via existing CM tools
   - Ansible playbooks
   - Chef cookbooks
   - Puppet manifests
   - SaltStack states

3. **Git-based Pull Model**: Machines pull from secure git repository
   - Policy repo with signed commits
   - Machine authenticates via certificate
   - Periodic polling for updates
   - Atomic policy application

4. **Central Policy Server**: cmdai enterprise daemon pulls from API
   - REST API for policy retrieval
   - mTLS authentication
   - Policy caching with fallback
   - Delta updates for efficiency

### 3. Policy Enforcement Engine

**Runtime Enforcement**: How policies are enforced during cmdai execution

```rust
// Conceptual enforcement flow
pub trait PolicyEnforcer {
    /// Check if a command is allowed under current policy
    async fn evaluate_command(&self, cmd: &Command) -> PolicyDecision;

    /// Check if a tool installation is permitted
    async fn evaluate_tool_install(&self, tool: &Tool) -> PolicyDecision;

    /// Get applicable approval workflow
    async fn get_approval_workflow(&self, cmd: &Command) -> Option<ApprovalWorkflow>;
}

pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequireApproval { workflow: ApprovalWorkflow },
}
```

**Enforcement Points**:

1. **Pre-generation**: Before AI generates command
   - Check inference backend against allowlist
   - Verify model is approved
   - Ensure local-only if required

2. **Post-generation**: After AI generates command, before execution
   - Parse generated command
   - Check against tool allowlist and versions
   - Evaluate safety patterns and guardrails
   - Check if approval required

3. **Tool discovery**: When cmdai detects or suggests tools
   - Only suggest approved tools
   - Warn about unapproved tool usage
   - Block installation of blocked tools

4. **Execution gate**: Final check before shell execution
   - Verify all policy constraints
   - Log enforcement decision
   - Optionally require MFA for high-risk

### 4. Policy Management Interface

**CISO Dashboard Features**:

- **Policy Editor**: Visual editor with YAML export
- **Version Control**: Git-backed policy history
- **Dry Run Mode**: Test policy changes before rollout
- **Rollout Controls**: Canary deployments, gradual rollouts
- **Impact Analysis**: Preview how many commands would be affected
- **Exception Management**: Grant temporary or permanent exceptions
- **Compliance Mapping**: Tag policies to compliance frameworks

### 5. Machine Identity and Correlation

**Device Registration**:

```yaml
# Machine manifest (auto-generated on cmdai first run)
machine_id: "dev-machine-3f8a2b"
organization: "acme-corp"
user: "alice@acme-corp.com"
hostname: "alice-macbook-pro"
platform: "darwin-arm64"
cmdai_version: "1.5.0"
enterprise_plugin_version: "1.0.3"
registered_at: "2025-11-15T10:30:00Z"
certificate_fingerprint: "sha256:abc123..."

# Organizational assignment
department: "engineering"
team: "platform"
environment: "development"
compliance_zone: "confidential"

# Capabilities
provisioned: true
policy_version: "v2.3.1"
last_policy_update: "2025-11-28T08:00:00Z"
```

**Correlation Benefits**:
- Map policy violations to specific users/teams
- Understand blast radius of policy changes
- Enable targeted policy application
- Track compliance across organizational units

## Rationale

### Why Centralized Governance?

1. **Compliance reality**: Auditors require centralized policy enforcement
2. **Scale efficiency**: Managing 1,000+ machines individually is impossible
3. **Consistency**: All developers operate under same security baseline
4. **Rapid response**: Security incidents require immediate policy updates
5. **Accountability**: Central system provides audit trail

### Why Declarative Policy?

1. **Version control**: Policies are code, track changes via git
2. **Review process**: Policy changes go through PR review
3. **Rollback**: Easy to revert bad policy changes
4. **Documentation**: Policy is self-documenting
5. **Testing**: Can validate policies before deployment

### Why Multiple Distribution Models?

1. **Flexibility**: Different enterprises have different infrastructure
2. **Integration**: Work with existing enterprise tooling
3. **Gradual adoption**: Start with git-pull, migrate to MDM later
4. **Resilience**: Fallback options if primary distribution fails

### Alignment with Enterprise Needs

This system directly addresses:
- **CISO requirement**: Central control with delegated management
- **Compliance mandate**: Auditable policy enforcement
- **Developer experience**: Policies enforced transparently, minimal disruption
- **IT operations**: Integrates with existing deployment tools
- **Security operations**: Rapid policy updates in response to threats

## Consequences

### Benefits

1. **Enterprise-grade governance**: Meets Fortune 500 security requirements
2. **Scalable deployment**: Handles organizations with 10,000+ developers
3. **Compliance enablement**: Direct mapping to SOC2, ISO27001, HIPAA controls
4. **Rapid incident response**: Push policy updates in minutes, not days
5. **Flexible integration**: Works with various enterprise infrastructure
6. **Auditability**: Complete provenance of policy decisions
7. **Developer respect**: Policies are transparent and explicable
8. **Reduced risk**: Proactive prevention vs. reactive detection

### Trade-offs

1. **Complexity**: Enterprise plugin becomes more sophisticated
2. **Infrastructure dependency**: Requires policy distribution infrastructure
3. **Network requirements**: Machines must reach policy server (with caching fallback)
4. **Policy maintenance**: Organizations must invest in policy management
5. **Initial setup**: Higher activation energy for enterprise deployment
6. **Testing overhead**: Policy changes need careful validation
7. **Developer friction**: Some developers may resist centralized control

### Risks

1. **Policy errors blocking work**: Overly restrictive policies stop developers
   - **Mitigation**: Dry-run mode, gradual rollouts, exception workflows

2. **Performance impact**: Policy evaluation adds latency to command generation
   - **Mitigation**: Local policy caching, optimized evaluation engine, async checks

3. **Policy distribution failure**: Machines can't get updated policies
   - **Mitigation**: Local caching, fail-open vs. fail-closed configuration, fallback modes

4. **Security of policy distribution**: Attacker modifies policies in transit
   - **Mitigation**: Signed policies, mTLS, certificate pinning, cryptographic verification

5. **User circumvention**: Developers bypass or disable enterprise plugin
   - **Mitigation**: Tamper detection, system-level installation, monitoring alerts

6. **Compliance gaps**: Policy schema doesn't cover all compliance requirements
   - **Mitigation**: Extensible policy schema, custom rule support, expert review

## Alternatives Considered

### Alternative 1: User-Configured with Central Recommendations
- **Description**: Keep user control, but provide central "recommended" policies
- **Pros**: Less disruptive, respects user autonomy
- **Cons**: Doesn't meet compliance requirements, no enforcement, users can ignore
- **Why not chosen**: Doesn't solve the core enterprise problem of mandatory enforcement

### Alternative 2: Fully Hardcoded Enterprise Policies
- **Description**: Ship pre-configured policies, no customization
- **Pros**: Simple deployment, consistent across customers
- **Cons**: Doesn't fit varied enterprise needs, inflexible, poor product-market fit
- **Why not chosen**: Every enterprise has different requirements and risk tolerance

### Alternative 3: SaaS-Only Policy Management
- **Description**: All policy management happens in cloud, machines phone home
- **Pros**: Easy to manage, real-time updates, centralized visibility
- **Cons**: Data sovereignty issues, requires internet, latency, privacy concerns
- **Why not chosen**: Many enterprises prohibit cloud dependencies for dev tools

### Alternative 4: Approval-Only (No Blocking)
- **Description**: Don't block commands, only require approval for risky ones
- **Pros**: Less disruptive, maintains velocity
- **Cons**: Approval fatigue, delays, doesn't prevent incidents, limited compliance value
- **Why not chosen**: Compliance requires preventive controls, not just detective

## Implementation Notes

### Phase 1: Policy Schema and Local Evaluation (3 months)

**Deliverables**:
- Policy schema v1.0 specification (YAML/JSON)
- Policy validation library
- Local policy evaluation engine
- File-based policy loading
- Basic enforcement at command generation

**Technical Components**:
```rust
// Core policy structures
pub struct OrganizationPolicy {
    version: String,
    organization: String,
    safety: SafetyRules,
    tools: ToolAllowlist,
    inference: InferenceRestrictions,
    approvals: ApprovalConfig,
    audit: AuditConfig,
}

pub struct PolicyEvaluator {
    policy: OrganizationPolicy,
    cache: PolicyDecisionCache,
}

impl PolicyEvaluator {
    pub async fn evaluate(&self, context: &CommandContext) -> PolicyDecision {
        // 1. Check tool allowlist
        // 2. Evaluate safety patterns
        // 3. Check inference restrictions
        // 4. Determine approval requirements
        // 5. Cache decision
    }
}
```

**Testing**:
- Unit tests for policy parsing
- Policy evaluation test suite
- Performance benchmarks (< 10ms evaluation)

### Phase 2: Distribution Infrastructure (3 months)

**Deliverables**:
- Policy distribution server (REST API)
- Git-based policy sync client
- Certificate-based machine authentication
- Policy signing and verification
- Version management and rollback

**Technical Components**:
```rust
// Policy distribution client
pub struct PolicySyncClient {
    server_url: String,
    machine_cert: Certificate,
    local_cache: PathBuf,
}

impl PolicySyncClient {
    pub async fn sync_policy(&self) -> Result<OrganizationPolicy> {
        // 1. Authenticate with certificate
        // 2. Check for policy updates
        // 3. Download and verify signature
        // 4. Atomic write to local cache
        // 5. Notify policy reload
    }
}
```

**Infrastructure**:
- Kubernetes-based policy server
- PostgreSQL for policy versions
- Redis for distribution caching
- CloudFlare for CDN (optional)

### Phase 3: Management Dashboard (3 months)

**Deliverables**:
- Web-based policy editor
- Visual policy builder
- Dry-run and impact analysis
- Rollout controls and canary deploys
- Exception management UI
- Policy version history and diffs

**Tech Stack**:
- Next.js + TypeScript frontend
- Monaco editor for YAML editing
- REST API for policy management
- Git backend for version control

### Phase 4: Enterprise Integrations (Ongoing)

**Deliverables**:
- MDM integration plugins (Jamf, Intune)
- Configuration management modules (Ansible, Chef, Puppet)
- SIEM connectors (Splunk, ELK, DataDog)
- SSO integration (Okta, Azure AD, Google Workspace)
- Compliance framework mappings

**Integration Patterns**:
```python
# Example Ansible playbook
- name: Deploy cmdai enterprise policy
  hosts: developer_machines
  tasks:
    - name: Install cmdai enterprise plugin
      package:
        name: cmdai-enterprise
        state: present

    - name: Deploy organization policy
      copy:
        src: "{{ policy_repo }}/acme-corp-policy.yaml"
        dest: /etc/cmdai/enterprise/policy.yaml
        mode: '0644'

    - name: Configure policy sync
      template:
        src: policy-sync-config.j2
        dest: /etc/cmdai/enterprise/sync.yaml
```

### Rollout Strategy

1. **Alpha**: Internal testing with cmdai development team (1 month)
2. **Beta**: 5 design partner enterprises, white-glove support (3 months)
3. **Limited GA**: 20 early adopter enterprises (3 months)
4. **General Availability**: Full product launch with self-service onboarding

### Migration Path

For enterprises currently using community edition:

1. **Assessment**: Audit current usage patterns and desired policies
2. **Policy Definition**: Work with CISO to define governance rules
3. **Pilot Deployment**: Deploy to small team (10-20 developers)
4. **Validation**: Ensure policies don't block legitimate work
5. **Gradual Rollout**: Expand to more teams with monitoring
6. **Full Deployment**: Organization-wide enforcement

## Success Metrics

### Technical Metrics
- **Policy evaluation latency**: < 10ms per command (p95)
- **Policy distribution time**: < 5 minutes from publish to all machines (p99)
- **Policy cache hit rate**: > 95% (reduces network dependency)
- **Enforcement accuracy**: 100% (no bypasses, no false positives)
- **System availability**: 99.9% uptime for policy distribution

### Business Metrics
- **Enterprise adoption**: 30 paying customers by end of Year 1
- **Seats deployed**: Average 500 seats per customer
- **Time to value**: Policy enforcement active within 2 weeks of purchase
- **Policy compliance**: 99%+ command compliance rate
- **Developer satisfaction**: < 5% increase in developer friction surveys

### Security Metrics
- **Incident reduction**: 80% reduction in command-related security incidents
- **Policy violations**: < 1% of commands violate policy (after tuning period)
- **Audit readiness**: 100% of commands logged and attributable
- **Compliance pass rate**: 100% pass rate for cmdai-related compliance controls

## Business Implications

### Revenue Model

**Tiered Pricing by Governance Complexity**:

1. **Basic Governance** ($5K/year for 10-50 seats)
   - Tool allowlisting
   - Basic safety guardrails
   - File-based policy distribution
   - Email-based alerts

2. **Advanced Governance** ($50K/year for 51-500 seats)
   - Full policy schema support
   - Approval workflows
   - Policy distribution server
   - Dashboard and reporting
   - SIEM integration

3. **Enterprise Governance** ($250K+/year for 500+ seats)
   - Multi-org support
   - Custom compliance frameworks
   - MDM/EDM integrations
   - Dedicated support + SLA
   - On-premise deployment option

### Market Positioning

**Competitive Differentiation**:
- **First-mover advantage**: No competitor offers AI command governance
- **Community foundation**: Built on trusted open-source project
- **Developer-friendly**: Policies are transparent, not black-box
- **Flexible deployment**: Works with existing enterprise infrastructure
- **Comprehensive**: Covers generation, execution, and audit

### Customer Segments

**Primary Targets**:
1. **Financial Services**: High compliance requirements (SOC2, PCI-DSS)
2. **Healthcare**: HIPAA compliance for developer environments
3. **Technology Companies**: Large engineering orgs (500+ developers)
4. **Government/Defense**: Strict security controls and air-gapped environments
5. **Enterprises with AI Governance Mandates**: Board-level AI oversight requirements

### Sales Enablement

**Key Messaging**:
- **Risk Reduction**: "Prevent security incidents before they happen"
- **Compliance Automation**: "Turn manual controls into automated enforcement"
- **Developer Velocity**: "Safe AI assistance without slowing teams down"
- **Audit Readiness**: "Complete visibility and traceability"
- **Cost Justification**: "One prevented incident pays for years of licensing"

**Proof Points**:
- Case study: Reduced command-related incidents by 85%
- Demo: Policy deployment across 1,000 machines in 5 minutes
- Benchmark: Zero overhead on developer productivity metrics
- Testimonial: CISO endorsement from design partner

## References

- **Industry Standards**:
  - [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
  - [CIS Controls v8](https://www.cisecurity.org/controls/)
  - [ISO/IEC 27001:2022](https://www.iso.org/standard/27001)

- **Policy-as-Code Inspiration**:
  - [Open Policy Agent (OPA)](https://www.openpolicyagent.org/)
  - [HashiCorp Sentinel](https://www.hashicorp.com/sentinel)
  - [Kubernetes Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)

- **Related ADRs**:
  - ADR-001: Enterprise vs Community Architecture (defines plugin model)
  - ADR-003: Monitoring and Audit Trail System (complementary logging)

- **Market Research**:
  - Gartner: "AI Governance Platforms Market Guide 2025"
  - Forrester: "Developer Security Tools Landscape 2024"
  - CNCF: "Cloud Native Security Survey 2024"

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-11-29 | cmdai core team | Initial draft |

# cmdai Product Roadmap

> **From CLI Tool to AI-Native Operations Platform**

This roadmap transforms cmdai from a developer side project into a venture-scale company using the proven PostHog dual-tier model: open-source community edition + cloud/enterprise SaaS.

---

## Vision

**"The AI-native operations platform trusted by 10,000+ engineering teams"**

By 2028, cmdai will be:
- The standard for AI-assisted terminal operations
- Trusted by 100,000+ developers using the open-source CLI
- Powering 10,000+ teams with cloud/enterprise features
- Running 10M+ operations per month across deployments, incident response, and infrastructure management
- Generating $50M+ ARR with 100% YoY growth

---

## Business Model

### Two-Tier Structure (PostHog Model)

**Community Edition (Open Source)**
- Everything needed for individual developers
- Local LLM backends (Ollama, MLX, CPU)
- Full safety validation
- Command generation and basic workflows
- MIT/Apache 2.0 licensed components
- **Purpose**: Growth engine, developer trust, community building

**Cloud + Enterprise (SaaS)**
- Better models (GPT-4, Claude 3.5, proprietary fine-tuned models)
- Team collaboration and shared patterns
- Audit logs and compliance (SOC 2, ISO 27001)
- SSO, RBAC, policy management
- Workflow automation and integrations
- Self-hosted enterprise option
- **Purpose**: Revenue engine, $50M+ ARR target

---

## Success Metrics by Quarter

### Q1 2025 (Months 1-3): Cloud Launch
- **Users**: 1,000 cloud signups, 100 Pro users
- **Revenue**: $2K MRR
- **Product**: Cloud backend, team features, analytics
- **Community**: 5,000 GitHub stars

### Q2 2025 (Months 4-6): Enterprise Features
- **Users**: 5,000 cloud users, 500 paid seats
- **Revenue**: $15K MRR, 5 enterprise deals at $25K-50K each
- **Product**: Audit logs, RBAC, SSO, self-hosted option
- **Community**: 10,000 GitHub stars

### Q3 2025 (Months 7-9): Platform Play
- **Users**: 10,000 cloud users, 2,000 paid seats
- **Revenue**: $50K MRR
- **Product**: Workflow engine, integration marketplace, fine-tuned models
- **Community**: 15,000 GitHub stars, 500 community workflows

### Q4 2025 (Months 10-12): Scale & Fundraise
- **Users**: 20,000 cloud users, 100 enterprise customers
- **Revenue**: $100K MRR (path to $2M ARR in Year 2)
- **Fundraising**: Series A ($5-10M at $40M post-money)
- **Team**: 6 people (CTO, 2 engineers, ML engineer, AE, DevRel)

---

## Phase-by-Phase Roadmap

## Phase 0: MVP Completion (Current → V1.0)

**Timeline**: November 2025 - December 2025
**Goal**: Production-ready CLI tool

### Must-Have Features
- [ ] Complete Feature 004 testing and stabilization
- [ ] Performance optimization (startup <100ms, inference <2s)
- [ ] Binary distribution (Homebrew, apt, cargo)
- [ ] Comprehensive documentation and quickstart guides
- [ ] CI/CD for automated releases
- [ ] Security audit and dependency hardening

### Success Criteria
- All tests passing (unit + integration + contract)
- Binary size <50MB
- 90%+ safety validation accuracy
- Ready for Hacker News/Product Hunt launch

### Deliverables
- v1.0.0 release on GitHub
- Homebrew formula
- Installation guide and tutorials
- Performance benchmarks published

**Milestone**: [GitHub Milestone: v1.0 - Production CLI](https://github.com/wildcard/cmdai/milestone/1)

---

## Phase 1: Cloud Foundation (Q1 2025)

**Timeline**: January 2025 - March 2025
**Goal**: Launch cloud backend, get to 100 cloud users, 10 paying teams

### Features

#### 1.1 Cloud Backend (Weeks 1-4)
**Owner**: Backend team
**Priority**: P0 (Critical)

**Requirements**:
- REST API for command generation (`POST /api/v1/generate`)
- JWT authentication with API keys
- Command history storage (Postgres)
- Usage tracking and rate limiting
- Better models (GPT-4, Claude 3.5)

**Technical Stack**:
```
Backend: Rust (Axum framework)
Database: Postgres (RDS or Supabase)
Cache: Redis (for rate limiting)
Hosting: Fly.io or Railway (start simple)
```

**Acceptance Criteria**:
- `cmdai auth login` → User gets API key
- Commands routed to cloud backend when authenticated
- 99.9% uptime SLA
- <1s P95 latency for command generation

**GitHub Issue**: #[TBD] - Cloud API Backend

#### 1.2 Team Collaboration (Weeks 5-8)
**Owner**: Product team
**Priority**: P0 (Critical)

**Requirements**:
- Shared command patterns (`cmdai share "deploy to staging"`)
- Team library (`cmdai list --team`)
- Command approval workflows for junior devs
- Team admin dashboard (web UI)

**Architecture**:
```rust
src/
├── cloud/
│   ├── api_client.rs      // Cloud API integration
│   ├── auth.rs            // JWT + API keys
│   ├── sync.rs            // Command history sync
│   └── teams.rs           // Team management
```

**Acceptance Criteria**:
- Teams can share command patterns
- Approval workflows work in CI/CD
- Web UI shows team activity

**GitHub Issue**: #[TBD] - Team Collaboration

#### 1.3 Analytics & Learning (Weeks 9-12)
**Owner**: ML team
**Priority**: P1 (High)

**Requirements**:
- Telemetry collection (with user consent)
- Usage analytics (which prompts → execution vs rejection)
- Model fine-tuning pipeline (future)
- Popular pattern discovery

**Data Collection**:
- Anonymized: prompt, generated command, execution status, error patterns
- NO collection: actual command output, user data, file contents

**Acceptance Criteria**:
- Analytics dashboard for teams
- Data pipeline ready for model fine-tuning
- GDPR/CCPA compliance

**GitHub Issue**: #[TBD] - Analytics & Learning

### Pricing (Launch Tier)
- **Free**: 50 commands/month on cloud
- **Pro**: $10/user/month, unlimited commands, better models
- **Team**: $20/user/month, collaboration features, approval workflows

### Milestone Completion
- **Revenue**: $2K MRR (100 Pro users or 10 small teams)
- **Users**: 1,000 cloud signups
- **Product**: cmdai.dev live, API stable, teams working

**GitHub Milestone**: [Q1 2025 - Cloud Launch](https://github.com/wildcard/cmdai/milestone/2)

---

## Phase 2: Enterprise Features (Q2 2025)

**Timeline**: April 2025 - June 2025
**Goal**: Close first 5 enterprise deals at $25K-50K each

### Features

#### 2.1 Audit & Compliance (Weeks 13-16)
**Owner**: Security team
**Priority**: P0 (Critical for enterprise)

**Requirements**:
- Immutable audit logs (every command logged)
- SIEM integration (Splunk, Datadog, Elastic)
- Compliance reports (SOC 2, HIPAA, PCI-DSS)
- Retention policies (7 years for financial services)
- Export capabilities (JSON, CSV, SYSLOG)

**Architecture**:
```rust
src/
├── enterprise/
│   ├── audit_log.rs       // Immutable logging
│   ├── compliance.rs      // SOC 2 controls
│   ├── export.rs          // SIEM integration
│   └── retention.rs       // Policy management
```

**Acceptance Criteria**:
- All commands logged with tamper-proof storage
- SOC 2 Type II ready
- SIEM integration working
- Enterprise customers sign off on compliance

**GitHub Issue**: #[TBD] - Audit & Compliance

#### 2.2 Access Control (Weeks 17-20)
**Owner**: Security team
**Priority**: P0 (Critical for enterprise)

**Requirements**:
- Role-based access control (RBAC)
  - Roles: Admin, Engineer, Junior, ReadOnly
- Organization-wide policies
  - "Block all `rm` commands for Junior role"
  - "Require approval for production changes"
- SSO integration (Okta, Azure AD, Google Workspace)
- Just-in-time access (elevated permissions for 1 hour)

**Architecture**:
```rust
src/
├── enterprise/
│   ├── rbac.rs            // Role definitions
│   ├── policies.rs        // Org-wide rules
│   └── sso.rs             // SAML, OIDC
```

**Acceptance Criteria**:
- SSO working with major providers
- Policies enforceable across teams
- JIT access with audit trail

**GitHub Issue**: #[TBD] - Access Control & RBAC

#### 2.3 Self-Hosted Option (Weeks 21-24)
**Owner**: Infrastructure team
**Priority**: P1 (High for large enterprise)

**Requirements**:
- Docker Compose deployment
- Kubernetes Helm chart
- Full stack: Web UI, Postgres, Redis, LLM backend
- Migration tools from cloud to self-hosted
- Monitoring and alerting (Prometheus, Grafana)

**Deployment**:
```bash
# Docker Compose (small teams)
docker-compose up -d

# Kubernetes (enterprise)
helm install cmdai cmdai/cmdai-enterprise
```

**Acceptance Criteria**:
- One-command deployment
- Works in air-gapped environments
- Full feature parity with cloud
- Monitoring and backup procedures documented

**GitHub Issue**: #[TBD] - Self-Hosted Deployment

### Pricing (Enterprise Tier)
- **Enterprise Cloud**: $50/user/month (min 20 seats = $12K/year)
- **Enterprise Self-Hosted**: $50K/year base + $75/user/month
- **White-glove onboarding**: $25K one-time
- **Support**: Premium 24/7 support included

### Milestone Completion
- **Revenue**: $150K ARR (5 enterprise customers)
- **Customers**: 50 paying teams, 5 enterprise deals
- **Product**: SOC 2 ready, self-hosted working

**GitHub Milestone**: [Q2 2025 - Enterprise Features](https://github.com/wildcard/cmdai/milestone/3)

---

## Phase 3: Platform Play (Q3 2025)

**Timeline**: July 2025 - September 2025
**Goal**: 10x the use cases → become the ops automation platform

### Features

#### 3.1 Workflow Engine (Weeks 25-28)
**Owner**: Platform team
**Priority**: P0 (Product differentiation)

**Requirements**:
- DAG-based workflow execution
- Multi-step runbooks ("deploy Next.js to AWS")
  1. Build Docker image
  2. Push to ECR
  3. Update ECS task definition
  4. Run database migrations
  5. Health check
  6. Rollback on failure
- Event-based triggers (webhook, schedule, manual)
- Workflow marketplace (community templates)
- Execution history and debugging

**Architecture**:
```rust
src/
├── workflows/
│   ├── engine.rs          // DAG execution
│   ├── triggers.rs        // Event handlers
│   ├── templates.rs       // Workflow library
│   └── marketplace.rs     // Community sharing
```

**Acceptance Criteria**:
- Complex workflows execute reliably
- 1,000+ community templates in marketplace
- Workflows can call external APIs and services

**GitHub Issue**: #[TBD] - Workflow Engine

#### 3.2 Integration Marketplace (Weeks 29-32)
**Owner**: Platform team
**Priority**: P1 (Network effects)

**Requirements**:
- Pre-built integrations:
  - **Source control**: GitHub, GitLab, Bitbucket
  - **Cloud**: AWS, GCP, Azure
  - **Observability**: Datadog, New Relic, Grafana
  - **Incident mgmt**: PagerDuty, Opsgenie
  - **Communication**: Slack, Discord, Teams
- Integration SDK for community developers
- OAuth flows for secure authentication
- Marketplace revenue sharing (future)

**Architecture**:
```rust
src/
├── integrations/
│   ├── sdk.rs             // Integration framework
│   ├── github.rs          // GitHub API
│   ├── aws.rs             // AWS SDK
│   ├── datadog.rs         // Observability
│   └── pagerduty.rs       // Incident management
```

**Acceptance Criteria**:
- 50+ integrations available
- Community can build custom integrations
- OAuth flows secure and compliant

**GitHub Issue**: #[TBD] - Integration Marketplace

#### 3.3 Proprietary Model Fine-Tuning (Weeks 33-36)
**Owner**: ML team
**Priority**: P1 (Competitive moat)

**Requirements**:
- Collect 100K+ command generation examples
- Fine-tune models on:
  - Commands that were executed vs rejected (safety learning)
  - Prompts needing clarification (UX improvement)
  - Domain-specific patterns (Kubernetes, AWS, data pipelines)
- Offer specialized models:
  - `cmdai-kubernetes` - K8s expert
  - `cmdai-aws` - AWS expert
  - `cmdai-data` - Data engineering expert
- Benchmark against GPT-4 on ops tasks

**Data Pipeline**:
```
User commands → Anonymized telemetry → Training dataset → Fine-tuning → Deployment
```

**Acceptance Criteria**:
- Proprietary models outperform GPT-4 on cmdai benchmarks
- Safety accuracy >99%
- 50% faster inference than GPT-4

**GitHub Issue**: #[TBD] - Proprietary Models

### Milestone Completion
- **Revenue**: $500K ARR
- **Users**: 2,000 cloud users, 50 enterprise customers
- **Product**: Workflow engine live, 500+ community workflows, proprietary models

**GitHub Milestone**: [Q3 2025 - Platform](https://github.com/wildcard/cmdai/milestone/4)

---

## Phase 4: Scale & Fundraise (Q4 2025)

**Timeline**: October 2025 - December 2025
**Goal**: Hit Series A metrics, raise $5-10M

### Focus Areas

#### 4.1 Growth & GTM
**Owner**: CEO + Head of Sales

**Activities**:
- Conference circuit (KubeCon, DevOpsDays, RustConf)
- Content marketing (blog, YouTube, podcasts)
- Case studies from top customers
- Partner program (AWS Marketplace, GCP Marketplace)
- Outbound sales to Series B startups

**Metrics**:
- 20% MoM user growth
- 10+ enterprise deals closed
- Net revenue retention: 120%+ (upsells)

#### 4.2 Team Building
**Owner**: CEO

**Hires**:
1. **VP Engineering** (ex-AWS, Google, HashiCorp)
   - Owns: Platform scaling, team leadership
2. **Head of Sales** (ex-Databricks, Snowflake, Terraform)
   - Owns: Enterprise GTM, revenue targets
3. **Senior ML Engineer** (ex-Anthropic, OpenAI, Hugging Face)
   - Owns: Model fine-tuning, inference optimization
4. **Developer Advocate** (ex-Vercel, Supabase, Fly.io)
   - Owns: Community growth, content, evangelism
5. **Product Manager** (ex-GitHub, GitLab, Linear)
   - Owns: Roadmap, prioritization, user research

**Total Team**: 8 people by end of Q4

#### 4.3 Fundraising Preparation
**Owner**: CEO + Board

**Deliverables**:
- Updated pitch deck (problem, solution, traction, market)
- Financial model (3-year projection to $50M ARR)
- Customer references (enterprise logos, testimonials)
- Product demo (workflow automation showcase)
- Data room (metrics, contracts, cap table)

**Target Investors**:
- **Tier 1**: a16z (AI focus), Sequoia (dev tools), Accel (infrastructure)
- **Tier 2**: Bessemer (cloud), Redpoint (dev tools), Index (AI)
- **Strategic**: Y Combinator (demo day), AWS (enterprise partnership)

**Round Structure**:
- **Size**: $5-10M
- **Valuation**: $40M post-money
- **Use of funds**: Team (50%), product (30%), GTM (20%)

### Milestone Completion
- **Revenue**: $100K MRR (path to $2M ARR in 2026)
- **Users**: 20,000 cloud, 100 enterprise customers
- **Fundraising**: Series A closed
- **Team**: 8 people, world-class talent

**GitHub Milestone**: [Q4 2025 - Scale](https://github.com/wildcard/cmdai/milestone/5)

---

## Year 2-3: Path to $50M ARR (2026-2027)

### 2026 Targets
- **Revenue**: $10M ARR
- **Customers**: 1,000 enterprise customers
- **Team**: 30 people
- **Product**:
  - Multi-cloud support (AWS, GCP, Azure native)
  - Advanced AI (reasoning models, multi-step planning)
  - Enterprise marketplace (private integrations)

### 2027 Targets
- **Revenue**: $30M ARR
- **Customers**: 5,000 enterprise customers
- **Team**: 100 people
- **Product**:
  - Global deployment (multi-region)
  - Compliance (FedRAMP, ISO 27001)
  - Series B ($30M at $150M post-money)

### 2028 Vision
- **Revenue**: $50M+ ARR
- **Customers**: 10,000+ teams
- **Market Position**: Category leader in AI-native operations
- **Exit Options**: IPO or strategic acquisition ($1B+ valuation)

---

## Community & Open Source Strategy

### Open Source Commitment
- **Core CLI**: Always free, MIT/Apache 2.0
- **Community features**: Local backends, basic safety, command generation
- **Transparency**: Public roadmap, open design discussions
- **Contributions**: Welcome PRs, pay bug bounties, hire from community

### Community Building
- **Discord server**: Real-time support, feature discussions
- **Monthly office hours**: Maintainers + community Q&A
- **Contributor recognition**: Hall of fame, swag, conference invites
- **Marketplace revenue**: Share 70% with integration authors (future)

### Why This Works (PostHog Model)
- Open source = trust + growth engine
- Enterprise features = revenue engine
- Community contributors = free R&D + QA
- Transparency = brand differentiation

---

## Risk Mitigation

### Technical Risks

**Risk**: GitHub Copilot CLI makes cmdai irrelevant
**Mitigation**:
- Go deep on ops (they're broad on code)
- Proprietary fine-tuned models (our data moat)
- Enterprise features they won't build (compliance, RBAC)

**Risk**: Can't compete with free (ChatGPT, open-source tools)
**Mitigation**:
- Free tier is marketing
- Teams pay for collaboration + compliance
- Enterprise pays for support + SLA + self-hosted

**Risk**: Model providers (OpenAI, Anthropic) raise prices
**Mitigation**:
- Fine-tune our own models (cost control)
- Support multiple backends (avoid lock-in)
- Local-first keeps marginal cost low

### Market Risks

**Risk**: Market too small (only CLI power users)
**Mitigation**:
- Expand to workflows (bigger TAM)
- Target DevOps + SRE + Data (not just CLI users)
- Enterprise motion (high ACV, less volume needed)

**Risk**: Can't hire fast enough
**Mitigation**:
- Stay small (6 people → $2M ARR is doable)
- Outsource non-core (design, support)
- Hire from community (they know the product)

---

## How to Contribute

This roadmap is **community-driven**. Here's how you can help:

### For Developers
- Pick an issue from a milestone (labeled by phase/quarter)
- Contribute to open-source core (safety patterns, backends)
- Build integrations for the marketplace
- Write documentation and tutorials

### For Companies
- Become a design partner (influence enterprise features)
- Pilot cloud/enterprise features (free during beta)
- Provide feedback on pricing and packaging

### For Investors/Advisors
- Intro to potential customers
- GTM strategy feedback
- Fundraising connections

---

## Getting Started

1. **Review the current milestone**: [v1.0 - Production CLI](https://github.com/wildcard/cmdai/milestone/1)
2. **Pick an issue**: Look for `good-first-issue` or `help-wanted` labels
3. **Join the conversation**: GitHub Discussions for questions
4. **Read contributing guide**: [CONTRIBUTING.md](CONTRIBUTING.md) + [BUSINESS_MODEL.md](BUSINESS_MODEL.md)

---

## Document Maintenance

**Owner**: Product team (community input welcome)
**Update cadence**: Monthly (or when major pivots happen)
**Last updated**: 2025-11-19

**Changelog**:
- 2025-11-19: Initial roadmap created (VC feedback incorporated)

---

**Let's build the future of AI-native operations. Together.**

*Questions? Open a [Discussion](https://github.com/wildcard/cmdai/discussions) or comment on the roadmap issues.*

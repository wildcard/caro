# cmdai Business Model

> **The PostHog Playbook: Open Source + Cloud + Enterprise**

This document explains how cmdai will build a venture-scale business ($50M+ ARR) while staying true to open-source values.

---

## Table of Contents

1. [The Dual-Tier Model](#the-dual-tier-model)
2. [Pricing Strategy](#pricing-strategy)
3. [Revenue Projections](#revenue-projections)
4. [Unit Economics](#unit-economics)
5. [Go-To-Market Strategy](#go-to-market-strategy)
6. [Competitive Moats](#competitive-moats)
7. [Why This Works](#why-this-works)

---

## The Dual-Tier Model

### Tier 1: Community Edition (Open Source)

**What's Included:**
- Full CLI tool (`cmdai` binary)
- Local LLM backends (Ollama, MLX, CPU)
- Remote backend support (bring your own API)
- Complete safety validation
- Command generation and execution
- Configuration management
- All core features

**License:**
- Core CLI: MIT or Apache 2.0 (permissive)
- Cloud connector: AGPL-3.0 (copyleft for network use)
- Dual-licensing option for enterprise (future)

**Purpose:**
- **Growth engine**: Attract developers, build trust
- **Marketing**: Word-of-mouth, GitHub stars, community
- **Innovation**: Community contributions, free R&D
- **Talent pipeline**: Hire the best contributors

**Target Users:**
- Individual developers
- Students and learners
- Open-source projects
- Small teams (<5 people)
- Security researchers (local-only use)

---

### Tier 2: Cloud + Enterprise (SaaS)

**What's Included:**
- Everything in Community Edition, plus:
- **Cloud-hosted inference** (better, faster models)
- **Team collaboration** (shared patterns, approval workflows)
- **Analytics** (usage insights, learning from patterns)
- **Audit logging** (compliance, security)
- **SSO & RBAC** (enterprise access control)
- **Self-hosted option** (for regulated industries)
- **Workflow automation** (multi-step runbooks)
- **Integration marketplace** (GitHub, AWS, Datadog, etc.)
- **Priority support** (SLA, dedicated Slack channel)
- **Proprietary models** (fine-tuned on cmdai data)

**Purpose:**
- **Revenue engine**: Path to $50M+ ARR
- **Sustainability**: Fund open-source development
- **Value delivery**: Features enterprises actually pay for

**Target Users:**
- **Cloud (SMB)**: Startups with 5-50 engineers
- **Enterprise**: Companies with 100+ engineers, compliance needs
- **Industries**: FinTech, HealthTech, SaaS, any regulated industry

---

## Pricing Strategy

### Community Edition

```
Free
├── All CLI features
├── Local backends
├── Safety validation
├── Community support (GitHub Discussions)
└── No limits
```

**Why free?**
- Acquisition cost = $0
- Developers try before teams buy
- Trust through transparency
- Community contributions = free value

---

### Cloud Tier (Product-Led Growth)

```
FREE TIER
├── 50 commands/month
├── Cloud-hosted inference (GPT-3.5 equivalent)
├── Command history (7 days)
├── Community support
└── Purpose: Try before you buy

PRO ($10/user/month)
├── Unlimited commands
├── Better models (GPT-4, Claude 3.5)
├── Command history (30 days)
├── Email support
├── Team sharing (up to 5 users)
└── Purpose: Individual power users, small teams

TEAM ($20/user/month)
├── Everything in Pro
├── Unlimited team size
├── Approval workflows (for junior devs)
├── Advanced analytics
├── Shared pattern library
├── Slack/Discord support
├── 99.9% uptime SLA
└── Purpose: Growing engineering teams (5-50 people)
```

**Annual Discount**: 20% off (2 months free)
- Pro: $96/year ($8/month effective)
- Team: $192/year ($16/month effective)

---

### Enterprise Tier (Sales-Led)

```
ENTERPRISE CLOUD ($50/user/month, min 20 seats)
├── Everything in Team
├── Audit logs (immutable, exportable)
├── SSO (Okta, Azure AD, Google)
├── RBAC (custom roles, policies)
├── Compliance (SOC 2, ISO 27001, HIPAA)
├── Advanced workflows (multi-step automation)
├── Integration marketplace
├── Dedicated success manager
├── 99.95% uptime SLA
├── 24/7 premium support
└── Minimum: $12,000/year (20 seats)

ENTERPRISE SELF-HOSTED ($50K/year base + $75/user/month)
├── Everything in Enterprise Cloud
├── On-premises deployment (Docker, Kubernetes)
├── Air-gapped environment support
├── Custom integrations
├── Private model hosting
├── White-glove onboarding ($25K one-time)
├── Annual security audit included
└── Minimum: $50,000/year + ($75 × seats)
```

**Volume Discounts** (Enterprise Cloud):
- 50-99 seats: 10% off
- 100-249 seats: 20% off
- 250+ seats: 30% off (custom pricing)

**Example Enterprise Pricing**:
- 50 seats: $50 × 50 × 0.9 = $2,250/month = $27K/year
- 100 seats: $50 × 100 × 0.8 = $4,000/month = $48K/year
- 500 seats: $50 × 500 × 0.7 = $17,500/month = $210K/year

---

## Revenue Projections

### Year 1 (2025): $0 → $100K MRR

| Quarter | Cloud Users | Pro Users | Team Seats | Enterprise Deals | MRR | ARR |
|---------|-------------|-----------|------------|------------------|-----|-----|
| Q1 | 1,000 | 100 | 0 | 0 | $1K | $12K |
| Q2 | 5,000 | 300 | 200 | 5 @ $2K each | $15K | $180K |
| Q3 | 10,000 | 600 | 800 | 15 @ $3K each | $50K | $600K |
| Q4 | 20,000 | 1,000 | 2,000 | 30 @ $4K each | $100K | $1.2M |

**Revenue Mix (Q4)**:
- Pro: $10K MRR (1,000 × $10)
- Team: $40K MRR (2,000 × $20)
- Enterprise: $120K MRR (30 deals × $4K avg)
- **Total**: $170K MRR (conservative, showing $100K as milestone)

---

### Year 2 (2026): $100K → $800K MRR

| Quarter | Cloud Users | Paid Seats | Enterprise Deals | MRR | ARR |
|---------|-------------|------------|------------------|-----|-----|
| Q1 | 30,000 | 5,000 | 50 | $200K | $2.4M |
| Q2 | 50,000 | 8,000 | 100 | $400K | $4.8M |
| Q3 | 80,000 | 12,000 | 200 | $600K | $7.2M |
| Q4 | 100,000 | 15,000 | 300 | $800K | $9.6M |

**Growth Drivers**:
- Product-led growth (PLG) motion working
- Enterprise sales team scaling (5+ AEs)
- Workflow marketplace creating lock-in
- Proprietary models launching (competitive advantage)

---

### Year 3 (2027): $800K → $3M MRR

| Quarter | Enterprise Customers | Avg Deal Size | MRR from Enterprise | Total MRR | ARR |
|---------|---------------------|---------------|---------------------|-----------|-----|
| Q1 | 500 | $6K | $3M | $3.5M | $42M |
| Q2 | 750 | $7K | $5.25M | $6M | $72M |
| Q3 | 1,000 | $8K | $8M | $9M | $108M |
| Q4 | 1,500 | $10K | $15M | $15M | $180M |

**Note**: These are aggressive but achievable with proper execution.

---

## Unit Economics

### Customer Acquisition Cost (CAC)

**Product-Led Growth (Cloud)**:
- Marketing spend: $50K/month
- New paying users: 500/month
- **CAC**: $100/user

**Sales-Led (Enterprise)**:
- Sales team cost: $200K/month (4 AEs @ $50K/month loaded)
- Enterprise deals closed: 10/month
- **CAC**: $20K/deal

### Lifetime Value (LTV)

**Pro User**:
- Monthly price: $10
- Average lifespan: 24 months
- Gross margin: 85%
- **LTV**: $10 × 24 × 0.85 = $204

**Team Seat**:
- Monthly price: $20
- Average lifespan: 36 months (higher retention)
- Gross margin: 85%
- **LTV**: $20 × 36 × 0.85 = $612

**Enterprise Deal**:
- Average deal size: $60K/year (100 seats @ $50/month)
- Average contract length: 3 years
- Gross margin: 90%
- **LTV**: $60K × 3 × 0.9 = $162K

### LTV/CAC Ratios

- Pro users: $204 / $100 = **2.0x** (acceptable, not great)
- Team users: $612 / $100 = **6.1x** (excellent)
- Enterprise: $162K / $20K = **8.1x** (outstanding)

**Interpretation**:
- Enterprise is the profit engine (high LTV/CAC)
- Cloud tiers drive volume and pipeline
- Focus on upselling Pro → Team → Enterprise

### CAC Payback Period

- Pro: 10 months ($100 CAC / $10 monthly)
- Team: 5 months ($100 CAC / $20 monthly)
- Enterprise: 4 months ($20K CAC / $5K monthly)

**Target**: <12 months for all tiers ✅

### Gross Margin

**Cloud Tier**:
- Revenue: $20/user/month
- Infrastructure cost: $2/user/month (inference, hosting, bandwidth)
- Support cost: $1/user/month (amortized)
- **Gross margin**: 85%

**Enterprise Tier**:
- Revenue: $50/user/month
- Infrastructure cost: $3/user/month (dedicated resources)
- Support cost: $2/user/month (dedicated CSM)
- **Gross margin**: 90%

**Self-Hosted**:
- Revenue: $50K base + $75/seat
- COGS: Engineering support, updates, security patches (~10%)
- **Gross margin**: 90%

### Net Revenue Retention (NRR)

**Target**: 120%+

This means if you start with $100K ARR from a cohort:
- $10K churns (10% annual churn)
- $30K expands (upsells, more seats)
- **End of year**: $120K from same cohort

**How to achieve**:
- Land with Team, expand to Enterprise
- Start with 50 seats, grow to 200 seats
- Add workflow automation as add-on
- Marketplace integrations drive lock-in

---

## Go-To-Market Strategy

### Bottom-Up (Product-Led Growth)

**Stage 1: Individual Adoption**
1. Developer finds cmdai on GitHub / Hacker News
2. Installs open-source CLI (`brew install cmdai`)
3. Loves the experience (fast, safe, local)
4. Tells teammates

**Stage 2: Team Adoption**
1. Multiple developers using cmdai
2. Hit limits of free tier or want collaboration
3. One person signs up for Pro/Team
4. Share with team via `cmdai teams invite`

**Stage 3: Organic Expansion**
1. Team grows from 5 → 20 → 50 users
2. Need compliance, audit logs, SSO
3. Inbound request to sales team
4. Upgrade to Enterprise

**Metrics**:
- Time to value: <5 minutes (install → first command)
- Activation rate: 40%+ (signups → first command)
- Free → Paid conversion: 3-5%
- Monthly active users (MAU): 30%+ of signups

### Top-Down (Sales-Led for Enterprise)

**Target ICP (Ideal Customer Profile)**:
- **Company stage**: Series B-D startups (100-1000 employees)
- **Industry**: FinTech, HealthTech, SaaS, E-commerce
- **Team size**: 50+ engineers, 10+ SREs/DevOps
- **Pain points**:
  - Incident response taking hours
  - Compliance requirements (SOC 2, HIPAA)
  - Junior engineers afraid of terminal
  - Knowledge silos (only senior engineers know commands)

**Sales Motion**:
1. **Outbound**: LinkedIn, warm intros from investors
2. **Discovery call**: Understand pain points
3. **Pilot**: 10-20 users, 30-day trial
4. **Business case**: Show time saved, incidents resolved faster
5. **Contract**: Annual or multi-year deal

**Sales Team Structure**:
- **Year 1**: 1 founding AE (closes first 10 deals)
- **Year 2**: 4 AEs + 1 Sales Engineer + 1 CSM
- **Year 3**: 10 AEs + 3 SEs + 5 CSMs

**Quota**:
- Each AE carries $1M annual quota
- Close 16 deals @ $60K avg = $960K

### Channels

**Direct**:
- cmdai.dev website with free trial
- Sales team for enterprise (demos, POCs)

**Partnerships**:
- **AWS Marketplace**: List cmdai Enterprise
- **GCP Marketplace**: Same
- **Resellers**: Partner with DevOps consulting firms

**Community**:
- **Open source**: GitHub stars → website visits → signups
- **Content**: Blog posts, YouTube tutorials, conference talks
- **Developer advocacy**: Hire DevRel from community

---

## Competitive Moats

### 1. Data Moat (Months 6-12)

**The Flywheel**:
```
More users → More commands generated → Better training data →
Better models → Better commands → More users
```

**Data assets**:
- 100K+ prompt → command pairs
- Success/failure rates for safety validation
- Domain-specific patterns (K8s, AWS, data pipelines)

**Result**: Proprietary models that outperform GPT-4 for ops tasks

**Defensibility**: No one else has this data. Takes 12+ months to replicate.

### 2. Integration Moat (Months 9-12)

**Network effects**:
- 50+ pre-built integrations (GitHub, AWS, Datadog, PagerDuty)
- 500+ community workflows in marketplace
- Switching cost: Reintegrating with 50 tools is painful

**Defensibility**: Integrations create lock-in. The more you use, the harder to leave.

### 3. Community Moat (Ongoing)

**Open source advantages**:
- 10,000+ GitHub stars = credibility
- Contributors = free evangelists
- Workflow marketplace = content moat
- Brand trust in security-conscious industries

**Defensibility**: Community takes years to build, can't be bought.

### 4. Compliance Moat (Q2 2025)

**Enterprise barriers**:
- SOC 2 Type II certification (6-12 months to get)
- HIPAA, PCI-DSS, ISO 27001 compliance
- Audit logs, RBAC, SSO (table stakes for enterprise)

**Defensibility**: Competitors need 6-12 months + $200K to match.

### 5. Platform Moat (Q3 2025)

**Workflow marketplace**:
- 1,000+ community workflows
- Integration ecosystem (50+ tools)
- Proprietary models (fine-tuned on cmdai data)

**Defensibility**: Platform businesses are winner-take-most markets.

---

## Why This Works

### 1. Proven Model (PostHog, GitLab, Supabase)

**PostHog**:
- Open-source analytics
- $40M ARR in 3 years
- Same model: free core + cloud + enterprise

**GitLab**:
- Open-source dev platform
- IPO'd at $11B
- Same model: free community edition + enterprise

**Supabase**:
- Open-source Firebase alternative
- $100M ARR in 3 years
- Same model: free tier + cloud + enterprise

**Pattern**: Dev tools + open source + cloud + enterprise = $B outcomes

### 2. Developer Trust Through Transparency

**Why developers trust open source**:
- Can read the code (no black boxes)
- No vendor lock-in (can self-host)
- Community contributions (aligned incentives)
- Free forever tier (no rug pull)

**Why this leads to revenue**:
- Individual developers adopt → Teams follow → Enterprises buy
- Trust accelerates sales cycles (no "security review" blockers)
- Open source = free marketing (GitHub stars, word of mouth)

### 3. Unit Economics Work

**Efficient customer acquisition**:
- CAC payback: 4-10 months
- LTV/CAC: 2x-8x depending on tier
- Gross margins: 85-90%

**Capital efficient growth**:
- $100K ARR/employee in Year 1 (6 people → $600K ARR)
- $150K ARR/employee in Year 2 (20 people → $3M ARR)
- $300K ARR/employee in Year 3 (50 people → $15M ARR)

### 4. Venture Scale TAM

**Total Addressable Market**:
- 30M developers worldwide
- 10M use CLI heavily
- 1M work at companies with compliance needs
- 100K companies with 50+ engineers

**Pricing assumptions**:
- Average company: 100 engineers @ $50/month = $60K/year
- 10,000 companies = $600M annual revenue potential

**Realistic capture**: 5-10% of TAM = $30-60M ARR

### 5. Network Effects Kick In

**Workflow marketplace**:
- More users → More workflows → More value → More users
- Winner-take-most dynamic (like GitHub Actions)

**Integration ecosystem**:
- More integrations → More use cases → More users
- Switching costs increase over time

**Data flywheel**:
- More usage → Better models → Better output → More usage

---

## Risks & Mitigation

### Risk: "Why would anyone pay for this?"

**Mitigation**:
- Free tier validates demand (if no free users, no paid users)
- Enterprise pays for compliance, not features
- Teams pay for collaboration, not just better models

**Validation approach**:
- Launch cloud in Q1, measure free → paid conversion
- If <1%, we have a product problem (not a pricing problem)
- Target: 3-5% conversion (industry standard for PLG)

### Risk: "GitHub Copilot CLI will crush you"

**Mitigation**:
- They're horizontal (code + CLI), we're vertical (ops only)
- We go deeper: workflows, integrations, enterprise features
- Open source = trust in regulated industries (banks won't use Copilot)

### Risk: "Open source cannibalizes paid"

**Mitigation**:
- True for individuals (that's the point)
- Teams/enterprises need collaboration + compliance (can't self-host)
- PostHog proves this works: 90% revenue from paid, 10% self-hosted

### Risk: "Can't monetize fast enough to fund growth"

**Mitigation**:
- Raise venture capital (this is a VC-backed business model)
- Start with $2M seed → Get to $100K MRR → Raise $10M Series A
- Unit economics work (not a money-losing business)

---

## Success Criteria by Quarter

### Q1 2025
- [ ] 1,000 cloud signups
- [ ] 100 Pro users ($1K MRR)
- [ ] 10 Team pilots
- [ ] Free → Paid conversion: 3%+

### Q2 2025
- [ ] 5,000 cloud signups
- [ ] 500 paid seats ($15K MRR)
- [ ] 5 enterprise deals closed
- [ ] NRR: 100%+ (no churn yet)

### Q3 2025
- [ ] 10,000 cloud signups
- [ ] 2,000 paid seats ($50K MRR)
- [ ] 20 enterprise customers
- [ ] 500+ community workflows

### Q4 2025
- [ ] 20,000 cloud signups
- [ ] 5,000 paid seats ($100K MRR)
- [ ] 50 enterprise customers
- [ ] NRR: 120%+ (expansion revenue working)

---

## Document Maintenance

**Owner**: CEO + CFO (community input welcome)
**Update cadence**: Quarterly (after metrics review)
**Last updated**: 2025-11-19

---

## Further Reading

- [ROADMAP.md](ROADMAP.md) - Product roadmap by quarter
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture for cloud/enterprise
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute

---

**Questions?**
- Open a [Discussion](https://github.com/wildcard/cmdai/discussions)
- Email: founders@cmdai.dev (when we set it up)

---

*This is the playbook. Let's execute.* 🚀

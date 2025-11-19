# cmdai Executive Summary

> **The 5-minute version for VCs, employees, and community members**

Last updated: 2025-11-19

---

## The 30-Second Pitch

**cmdai is GitHub Copilot for your terminal.** We convert natural language to safe shell commands using local AI. Open-source CLI, cloud SaaS for teams, enterprise for compliance.

**Think:** PostHog for DevOps. GitLab for command-line automation.

**We're building:** The AI-native operations platform trusted by 10,000+ engineering teams.

**Target:** $50M ARR by 2028, Series A in Q4 2025.

---

## The Problem

**Every DevOps engineer faces this daily:**

1. **Time wasted:** 5 minutes searching Stack Overflow for command syntax
2. **Fear of mistakes:** One typo = production outage
3. **Knowledge silos:** Only senior engineers know the "magic commands"
4. **Slow incident response:** Fumbling with commands during critical outages

**Market size:** 30M developers worldwide, 10M use CLI heavily, $600M+ annual opportunity

---

## The Solution

**What we built:**
- Rust CLI that converts `"deploy to AWS"` → correct, safe commands
- Local AI (no cloud dependency, works offline)
- Safety validation (blocks `rm -rf /`, dangerous operations)
- Multi-backend (MLX for Apple Silicon, CPU fallback, Ollama, vLLM)

**Why it's better:**
- **10x faster:** 5 minutes → 5 seconds to find the right command
- **100% safer:** AI-powered validation prevents destructive commands
- **Zero cloud dependency:** Works offline, no API keys needed
- **Open source:** Community trust, no vendor lock-in

---

## The Business Model

### Dual-Tier (PostHog Model)

```
┌─────────────────────────────────────────────────┐
│  OPEN SOURCE (Free Forever)                    │
│  • CLI tool for individuals                    │
│  • Local AI backends                           │
│  • Full safety validation                      │
│  • Purpose: Growth engine (trust, adoption)    │
└─────────────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│  CLOUD + ENTERPRISE (Revenue)                  │
│  • Better models (GPT-4, Claude)               │
│  • Team collaboration                          │
│  • Audit logs, SSO, RBAC                       │
│  • Purpose: Revenue engine ($50M ARR)          │
└─────────────────────────────────────────────────┘
```

### Pricing

| Tier | Price | Target |
|------|-------|--------|
| **Free** | $0 | Individuals, students |
| **Pro** | $10/user/month | Power users |
| **Team** | $20/user/month | Small teams (5-50) |
| **Enterprise Cloud** | $50/user/month (min 20) | Large companies |
| **Enterprise Self-Hosted** | $50K/year + $75/user | Regulated industries |

### Unit Economics (Healthy)

- **Gross Margin:** 85-90%
- **LTV/CAC:** 2-8x depending on tier
- **CAC Payback:** 4-10 months
- **NRR:** 120%+ target (expansion revenue)

---

## The Traction

### Current Status (MVP)
- ✅ Working CLI with embedded AI
- ✅ Apple Silicon optimization (MLX)
- ✅ Remote backends (Ollama, vLLM)
- ✅ Comprehensive safety validation
- ✅ 44 tests passing, contract-based testing
- ✅ AGPL-3.0 licensed (open source)

### GitHub
- **Stars:** [Current count]
- **Contributors:** [Current count]
- **Commits:** [Current count]

### Next Milestone: V1.0 (Dec 2025)
- Binary <50MB, startup <100ms
- Homebrew/apt/cargo distribution
- Production-ready for launch

---

## The Roadmap

### Year 1 (2025): MVP → $100K MRR

| Quarter | Focus | Revenue Target | Key Metrics |
|---------|-------|----------------|-------------|
| **Q1** | Cloud launch | $2K MRR | 1,000 users, 100 paid |
| **Q2** | Enterprise | $15K MRR | 5 enterprise deals |
| **Q3** | Platform | $50K MRR | Workflow engine |
| **Q4** | Series A | $100K MRR | 100 enterprise customers |

### Year 2-3 (2026-2027): Scale to $50M ARR

- **2026:** $10M ARR, 1,000 enterprise customers
- **2027:** $30M ARR, 5,000 enterprise customers
- **2028:** $50M+ ARR, category leader

---

## The Competitive Moats

### 1. Data Moat (Q3 2025)
- 100K+ command examples from real usage
- Fine-tuned models outperform GPT-4 for ops tasks
- **No one else has this training data**

### 2. Integration Moat (Q3 2025)
- 50+ pre-built integrations (AWS, GitHub, Datadog)
- Community marketplace with 1,000+ workflows
- High switching costs

### 3. Community Moat (Ongoing)
- 10,000+ GitHub stars = credibility
- Open source = trust in regulated industries
- Contributors = free evangelists

### 4. Compliance Moat (Q2 2025)
- SOC 2 Type II ($200K + 6 months to replicate)
- Audit logs, RBAC, SSO = enterprise table-stakes
- Competitors need 12+ months to catch up

---

## The Market

### TAM (Total Addressable Market)
- **Developers worldwide:** 30M
- **CLI power users:** 10M
- **At companies with compliance:** 1M
- **Target capture:** 10,000 teams × $60K/year = **$600M opportunity**

### Comparable Companies (Validation)

| Company | Model | Outcome |
|---------|-------|---------|
| **PostHog** | Open-source analytics, dual-tier | $40M ARR in 3 years |
| **GitLab** | Open-source DevOps, dual-tier | $11B IPO |
| **Supabase** | Open-source Firebase, dual-tier | $100M ARR in 3 years |
| **HashiCorp** | Open-source infra tools | $14B IPO (Terraform, Vault) |

**Pattern:** Dev tools + open source + cloud + enterprise = $B outcomes

---

## The Team

### Current
- **Maintainer:** [Your name/background]
- **Contributors:** [X active contributors]
- **Community:** Growing open-source community

### Hiring Plan (Year 1)

| Role | When | Why |
|------|------|-----|
| **Co-founder/CTO** | Q1 | Platform scaling, team leadership |
| **Senior Backend Eng** | Q1 | Cloud infrastructure |
| **Senior ML Eng** | Q2 | Model fine-tuning |
| **Founding AE (Sales)** | Q2 | Enterprise GTM |
| **Developer Advocate** | Q3 | Community growth |

**Target team size:** 6-8 people by Q4 2025

---

## Why Now?

### 1. AI is Eating Software
- ChatGPT proved AI can generate code
- GitHub Copilot: $100M ARR in 18 months
- **Ops is the next frontier**

### 2. Developer Tools Market is Hot
- VCs invested $10B+ in dev tools (2024)
- a16z, Sequoia, Accel actively seeking AI dev tools
- Y Combinator funded 20+ AI dev tools in W24

### 3. Remote Work = Terminal Renaissance
- More developers working in terminals (SSH, cloud shells)
- DevOps/SRE roles growing 20%+ YoY
- Kubernetes adoption = complex CLI operations

### 4. Compliance Tailwind
- SOC 2 now required for any B2B SaaS
- Audit logs for CLI operations = greenfield opportunity
- We're early to "compliance for terminal operations"

---

## The Comparables

### Who We're NOT
- ❌ **Not** GitHub Copilot CLI (they're broad on code, we're deep on ops)
- ❌ **Not** ChatGPT wrapper (we have local AI, safety validation, team features)
- ❌ **Not** bash scripting tool (we're AI-native, not script-based)

### Who We ARE
- ✅ **GitHub Copilot** for terminal (AI-assisted operations)
- ✅ **PostHog** business model (open source + cloud + enterprise)
- ✅ **Terraform** category (infrastructure automation)

---

## The Ask

### For VCs (Series Seed/A)

**Round:** $2-5M seed (Q1 2025) or $5-10M Series A (Q4 2025)

**Use of funds:**
- 50% Team (hire 5-6 people)
- 30% Product (cloud infrastructure, security)
- 20% GTM (conferences, content, sales)

**Metrics we're targeting:**
- Seed: $10K MRR, 1,000 cloud users, strong team
- Series A: $100K MRR, 20,000 users, 100 enterprise customers

**Why invest:**
- Proven model (PostHog, GitLab, Supabase)
- Hot category (AI + dev tools)
- Clear path to $50M+ ARR
- Founder executing (documentation, roadmap, community)

**Investor profile:**
- Tier 1: a16z (AI), Sequoia (dev tools), Accel (infrastructure)
- Tier 2: Bessemer, Redpoint, Index
- Strategic: YC, AWS/GCP partnerships

---

### For Employees

**Why join cmdai:**

1. **Mission:** Make terminal operations 10x faster and safer for every developer
2. **Opportunity:** Ground floor of potential $B outcome (like GitLab, PostHog)
3. **Model:** Open source = aligned with community, not just VC returns
4. **Ownership:** Early equity, significant impact, shape the product
5. **Team:** Work with world-class Rust, ML, and DevOps engineers

**What you'll get:**
- Competitive salary (market rate)
- Equity (0.5-2% for early hires)
- Remote-first (work from anywhere)
- Learn from the best (open-source community, YC mentors)
- Build in public (open roadmap, transparent company)

**Open roles (2025):**
- Senior Backend Engineer (Rust, Axum, Postgres)
- Senior ML Engineer (LLM fine-tuning, inference optimization)
- Founding AE (enterprise sales, SaaS GTM)
- Developer Advocate (content, community, evangelism)

---

### For Contributors

**Why contribute:**

1. **Learn:** Work with production Rust, LLMs, DevOps at scale
2. **Impact:** Your code will run on 100,000+ machines
3. **Community:** Collaborate with talented developers worldwide
4. **Resume:** Open-source contributions = hiring signal
5. **Potential:** Top contributors may be hired as employees

**What we need:**
- **Developers:** V1.0 completion, cloud backend, enterprise features
- **Technical writers:** Docs, tutorials, case studies
- **Designers:** Web UI, landing pages, UX
- **Business:** GTM feedback, customer research, partnerships

**How to start:**
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Pick an issue labeled `good-first-issue`
3. Join [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
4. Submit your first PR

---

## One-Page Summary (For Slides)

```
┌─────────────────────────────────────────────────────────────┐
│                         cmdai                               │
│        GitHub Copilot for Your Terminal                     │
├─────────────────────────────────────────────────────────────┤
│ PROBLEM                                                     │
│ • Developers waste 5 min/day searching for CLI syntax      │
│ • Fear of typos causing production outages                 │
│ • Only senior engineers know "magic commands"              │
│                                                             │
│ SOLUTION                                                    │
│ • Natural language → safe shell commands (AI-powered)      │
│ • Local-first (works offline, no API keys)                 │
│ • Safety validation (blocks dangerous operations)          │
│ • Open-source core + Cloud/Enterprise SaaS                 │
│                                                             │
│ BUSINESS MODEL (PostHog Playbook)                          │
│ • Free: Open-source CLI (growth engine)                    │
│ • Pro: $10/user/mo (individual power users)                │
│ • Team: $20/user/mo (small teams)                          │
│ • Enterprise: $50-75/user/mo (compliance, audit, SSO)      │
│                                                             │
│ TRACTION                                                    │
│ • Working MVP with Apple Silicon optimization              │
│ • 44 tests passing, production-ready architecture          │
│ • Active open-source community                             │
│                                                             │
│ ROADMAP                                                     │
│ • Q1 2025: Cloud launch ($2K MRR)                          │
│ • Q2 2025: Enterprise features ($150K ARR)                 │
│ • Q3 2025: Platform (workflows) ($500K ARR)                │
│ • Q4 2025: Series A ($100K MRR)                            │
│ • 2028: $50M+ ARR, 10,000+ teams                           │
│                                                             │
│ MOATS                                                       │
│ • Data: 100K+ command examples (proprietary training)      │
│ • Integration: 50+ tools, 1,000+ workflows (lock-in)       │
│ • Community: 10,000+ stars, contributor network            │
│ • Compliance: SOC 2, audit logs (6-12 mo to replicate)     │
│                                                             │
│ COMPARABLES                                                 │
│ • PostHog: $40M ARR in 3 yrs (same model)                  │
│ • GitLab: $11B IPO (same model)                            │
│ • Supabase: $100M ARR in 3 yrs (same model)                │
│                                                             │
│ THE ASK                                                     │
│ • VCs: $2-5M seed or $5-10M Series A                       │
│ • Employees: Join as founding engineer/AE/DevRel           │
│ • Contributors: Help build V1.0, shape the roadmap         │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Metrics to Track

### North Star Metric
**Active teams using cmdai weekly** (leading indicator of revenue)

### Growth Metrics (Weekly)
- [ ] GitHub stars (community growth)
- [ ] Cloud signups (funnel top)
- [ ] Active users (engagement)
- [ ] Free → Paid conversion (monetization)

### Revenue Metrics (Monthly)
- [ ] MRR (monthly recurring revenue)
- [ ] New customers (volume)
- [ ] Churn rate (retention)
- [ ] NRR (net revenue retention / expansion)

### Product Metrics
- [ ] Command generation success rate
- [ ] Safety validation accuracy
- [ ] P95 latency (performance)
- [ ] CLI startup time

---

## FAQ (Quick Answers)

**Q: Why open source if you want to make money?**
A: PostHog, GitLab, Supabase prove you can. Open source = growth engine (trust, adoption). Cloud/enterprise = revenue engine.

**Q: Won't GitHub Copilot CLI kill you?**
A: They're horizontal (code + CLI), we're vertical (deep on ops). We have workflows, enterprise features, compliance they won't build.

**Q: Why would anyone pay for this?**
A: Individuals might not. Teams pay for collaboration. Enterprises pay for compliance (audit logs, SSO, RBAC).

**Q: What's your unfair advantage?**
A: Data moat. After 100K commands, we fine-tune models that outperform GPT-4 for ops tasks. No one else has this data.

**Q: How do you compete with free (ChatGPT)?**
A: ChatGPT is generic. We're domain-specific (ops), safety-first (validation), integrated (terminal native), and team-oriented (collaboration).

**Q: What if a big company (AWS, Google) copies you?**
A: Open source = community trust they can't replicate. Compliance moat (SOC 2) = 12 mo head start. Data moat = proprietary training.

**Q: How capital efficient can you be?**
A: Target $100K ARR/employee in Year 1. PostHog did $40M ARR with 40 people. We'll stay lean.

---

## Next Steps

### If you're a VC
1. Read [BUSINESS_MODEL.md](BUSINESS_MODEL.md) for full economics
2. Schedule a demo (see the product in action)
3. Meet the team and community
4. Intro to design partners (early customers)

### If you're considering joining as employee
1. Read [ROADMAP.md](ROADMAP.md) to see where we're going
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) for technical depth
3. Talk to current contributors (get real feedback)
4. Schedule a call with founder

### If you're a contributor
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Pick an issue from [V1.0 milestone](https://github.com/wildcard/cmdai/milestone/1)
3. Join [Discussions](https://github.com/wildcard/cmdai/discussions)
4. Submit your first PR

---

## Contact

- **GitHub:** https://github.com/wildcard/cmdai
- **Email:** [Maintainer email]
- **Discussions:** https://github.com/wildcard/cmdai/discussions

---

## Detailed Documentation

For those who want depth:

- [ROADMAP.md](ROADMAP.md) - Complete quarterly roadmap
- [BUSINESS_MODEL.md](BUSINESS_MODEL.md) - Full business plan and economics
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [MVP_TO_V1.md](MVP_TO_V1.md) - Immediate next steps
- [GITHUB_SETUP.md](GITHUB_SETUP.md) - How we organize work

---

**The elevator pitch:** cmdai is GitHub Copilot for your terminal. Open-source CLI + Cloud/Enterprise SaaS. PostHog model. $50M ARR by 2028.

**The ask:** Help us build the AI-native operations platform for 10,000+ teams.

**Let's do this.** 🚀

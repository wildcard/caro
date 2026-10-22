# Interview script: `enterprise-dashboard` hypothesis

Calibrated extension of [`interview-template.md`](./interview-template.md)
for the `enterprise-dashboard` hypothesis specifically — the
CISO / IT-leader / security-team buyer for the Caro Enterprise
plugin suite per [ADR-001](../adr/ADR-001-enterprise-community-architecture.md).

**Hypothesis under test**: Organizations that already deploy AI
coding agents need an execution-safety layer with fleet-wide
governance, centralized policy distribution, audit-trail
forwarding, and machine correlation — and they will pay for it
through standard enterprise procurement.

**Target cohort**: Decision-makers and influencers for
developer-tooling procurement at organizations with ≥50 engineers
who already use AI coding assistants. Roles: CISO, Director of
Security Engineering, Head of Developer Platform, VP of
Engineering, IT operations lead.

**Time budget**: 30 minutes synchronous (this cohort doesn't do
async chat well). Get on their calendar.

## Pre-interview setup

- [ ] Confirm the participant's role and org size from public
  signals (LinkedIn, conference talks, prior posts). Note in
  `anon_handle`: e.g. `ciso-fintech-mid`, `eng-platform-bigco`.
- [ ] Get consent explicitly: "May I log this as an anonymized
  discovery transcript? Your name, company, and any specific
  internal tool names get generalized. The transcript lives in
  a public OSS repo."
- [ ] Use a recording tool the participant trusts (their choice,
  ideally — not yours). If recording is refused, take careful notes
  but DO NOT pretend the transcript is verbatim.
- [ ] Calibrate yourself: you are not pitching Caro Enterprise
  today. You are validating whether the *category* exists. If
  they ask "how much does it cost?", deflect to "we're scoping
  pricing after these conversations" — quoting a number turns the
  interview into a negotiation.

## Question 1 — Their AI-coding-agent landscape (3–5 min)

> "Tell me about the AI coding tools your engineering org uses
> today — Cursor, Claude Code, GitHub Copilot, others. How many
> seats, who pays, who decided to adopt?"

**Listen for**: maturity of the deployment, governance posture
("we have a policy" vs "I think they use it"), procurement shape
(team-by-team vs centralized).

**Why this question first**: anchors the conversation in their
actual deployment, not in theoretical Caro adoption.

## Question 2 — The execution-safety gap (5–7 min)

> "When one of those AI tools generates a shell command and an
> engineer runs it, how do you currently know it was safe? Who
> owns the question if something destructive happens?"

**Listen for**: silence here is the most valuable answer. If
they don't have an answer, the gap is real. If they have one
("we trust the engineer", "we have rules in our CI", "we don't"),
push on the next question: what would they want instead.

**Push probes**:
- "Has an AI-generated command ever caused an incident here?"
- "If one did, would you find out? How?"
- "What does your audit team think about AI-generated commands
  hitting prod systems?"

## Question 3 — Current alternatives (3–5 min)

> "What tools do you evaluate (or have evaluated) for this gap —
> Wiz, Bridgewater Code Security, Snyk, the AI-vendor's own
> safety claims, internal rules, nothing?"

**Listen for**: the competitor set they actually consider. This
shapes the [`docs/enterprise/MOAT.md`](../enterprise/MOAT.md)
narrative. If the answer is "the AI vendor's own safety claims",
that's the playbook's "single provider dependency" failure mode
showing up in the customer.

## Question 4 — Procurement reality (3–5 min)

> "If you wanted to roll out a tool like this across your eng
> org, what's the actual process? Security review, procurement,
> IT deployment? How long does it take? What kills tools at
> each step?"

**Listen for**: the SOC 2 / ISO / GDPR / HIPAA gates, the
existing vendor-management system (Vanta, Drata, Anecdotes), the
MDM / package-manager / image-bake step they'd ship Caro
through. This calibrates the
[`docs/enterprise/ENTERPRISE-VALUE-PROPOSITION.md`](../enterprise/ENTERPRISE-VALUE-PROPOSITION.md)
ROI math against their real procurement friction.

**Push probes**:
- "Where do small-vendor tools die in your procurement?"
- "What's the smallest contract size your finance team will
  process?"
- "Who has to say yes — you, your boss, security, finance, IT,
  legal?"

## Question 5 — Willingness signal calibrated for enterprise (5 min)

> "If a tool gave you per-engineer execution audit trails,
> centralized safety-pattern distribution, and a CISO dashboard
> for AI-generated command activity across your fleet — what's
> the budget shape? Per-seat-per-month? Annual contract? What
> tier of approval does it need?"

**Listen for**: an actual range AND the approval-tier signal.
"$X per dev per month" alone isn't enough; "we'd buy it
out-of-pocket from my team budget" vs "this would need board-
level approval" tells you whether the deal cycle is months or
quarters.

**Be honest**: if they say "we don't have a budget for this", that
is the most valuable answer you'll get all day. It means the
hypothesis is wrong about *this cohort* — and you can ask why
without taking it personally.

## Question 6 — Enterprise dealbreakers (5–7 min) ⭐ most important

> "What would a tool in this space do that would make your
> security team refuse to deploy it, or make you decommission it
> in year 2?"

**Listen for** — these are the demoware-trap inputs for the
Enterprise spec:
- "Phones home with anything sensitive." (Telemetry posture)
- "Vendor lock-in to one model." (Multi-backend resilience)
- "Can't be air-gapped." (Air-gap mode requirements)
- "Doesn't integrate with our SSO / SCIM / RBAC." (Identity
  integration scope)
- "No SOC 2 / no SBOM / no security questionnaire response."
  (Compliance baseline)
- "Auto-updates without our approval." (Update governance)
- "Built by a single founder with no enterprise support story."
  (Vendor risk)

**This question is the most important question in the interview.**
Question 5 is what they would say yes to in theory; question 6 is
what makes them say no in practice. The dealbreakers determine
which Enterprise features are load-bearing.

## Optional — Beta interest (2 min)

> "If we had an early-access version of this Enterprise plugin
> ready in Q3, would you want to be a pilot customer? What would
> 'pilot' mean for you — paid trial, free trial, contract with
> termination clauses, a single team rollout?"

**Listen for**: enthusiasm vs. politeness. A pilot commitment
from one of these conversations is a paid-conversion signal,
which is the Stage-3 exit criterion you're working toward.

## After the interview

- [ ] Save the transcript to
  `docs/discovery/transcripts/YYYY-MM-DD-<anon-handle>-enterprise-dashboard.md`
- [ ] Frontmatter must include `hypothesis: enterprise-dashboard`
- [ ] Anonymize per `README.md` — generalize company name to
  `<industry>-co` unless they explicitly consented
- [ ] Tag 1–3 `pain_patterns` slugs in frontmatter
- [ ] Update [`hypothesis-ledger.md`](./hypothesis-ledger.md)
  row for `enterprise-dashboard` — transcript count +=1
- [ ] After every 5 transcripts, run the synthesis pass

## Calibration notes specific to this hypothesis

- **The cohort is selection-biased toward security-paranoid orgs.**
  That's fine — it's the right cohort for an Enterprise audit-trail
  product. Just be honest about it in synthesis (don't generalize
  to "all eng orgs need this" from CISO-only conversations).
- **CISOs say no by default.** Treat that as the baseline — what
  matters is *why* they would say yes. If you finish 5 interviews
  with zero "yes, this is a real gap" responses, that's a strong
  invalidation signal.
- **The procurement question is load-bearing.** A great product
  that dies in procurement is invalidated. If the answer to "what
  kills tools at each step" reveals friction Caro can't survive,
  the Enterprise GTM strategy has to shift before any features ship.

## Anti-patterns

- **Don't quote a price.** "We're scoping pricing after these
  conversations." If they push, "what's the price you'd expect?"
  becomes Question 5.
- **Don't promise features.** "We're considering that" not "we'll
  build that".
- **Don't push back on objections.** The objections are the data.
  "Tell me more about that" is the right response, not "but
  actually we…".
- **Don't talk for more than 20% of the conversation.** Your job
  is to ask, listen, and capture.

## See also

- [`interview-template.md`](./interview-template.md) — generic
  6-question script this calibrates
- [`hypothesis-ledger.md`](./hypothesis-ledger.md) — the
  `enterprise-dashboard` row receives synthesis from these
  interviews
- [`outreach-templates.md`](./outreach-templates.md) — Template 1
  (waitlist segmented) is the natural outreach surface for this
  cohort once the pricing-enterprise CTA captures signups
- [`docs/enterprise/MOAT.md`](../enterprise/MOAT.md) and
  [`ENTERPRISE-VALUE-PROPOSITION.md`](../enterprise/ENTERPRISE-VALUE-PROPOSITION.md) —
  these documents claim things the interviews must validate
- [`.claude/agents/devils-advocate.md`](../../.claude/agents/devils-advocate.md) —
  audit the resulting spec when the 20-transcript threshold approaches

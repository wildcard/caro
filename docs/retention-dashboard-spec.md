# Retention Dashboard Spec

**Status**: Spec only. Implementation lives in a future PR.
**Owner**: TBD (Stage-3 exit-gate work)
**Source framework**: [Anthropic's Founder's Playbook, Stage 3 — Launch](https://claude.com/blog/the-founders-playbook)

The Stage-3 exit gate from
[`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md) is **retention
curve flattens**. We cannot show that today because we collect the
underlying telemetry events but do not aggregate them. This document
specifies the dashboard we'd build — designed so it can ship without
expanding the data we collect.

## Hard constraints

These constraints are non-negotiable. Any implementation that violates
one fails review.

1. **No new PII.** The retention dashboard uses only events already
   collected per [`docs/TELEMETRY.md`](./TELEMETRY.md): session
   start, session end, command-generation success/failure/timing,
   backend identifier, error category. **No command content**, **no
   file paths**, **no environment variables**, **no natural-language
   prompts**, **no IP addresses beyond what's needed for
   geo-region bucketing**.
2. **Opt-in only.** Users who haven't opted into telemetry contribute
   zero data. The dashboard's denominator is opt-in users, not
   "everyone".
3. **Anonymous session IDs only.** Existing session IDs are hash(machine_id
   + date), rotated daily. The dashboard works on this hash, not on
   stable user IDs. A user who uses Caro on day 1 and day 8 looks
   like two different sessions — and that's intentional.
4. **Aggregate-only published numbers.** Any dashboard view that
   exposes <50 sessions in a bucket is suppressed. This protects
   small cohorts (specific OS+shell+region combinations) from
   re-identification.
5. **Audit-trail of definitions.** Every metric's SQL/PromQL is
   version-controlled in this repo so the methodology is reproducible.

## Metrics

### D1 / D7 / D30 retention (the headline)

**Definition**: of users who opened Caro and ran ≥1 command on day
0, what fraction came back and ran ≥1 command on day 1 / 7 / 30?

**Cohort defense** (per `.claude/rules/validation-discipline.md` Gate
5): the cohort is **users who completed ≥1 successful command on
their first session**, not "everyone who installed Caro". A user who
installed and never ran a command is interesting noise (install-funnel
signal), not retention signal.

**Caveat**: because session IDs rotate daily, "the same user across
days" is reconstructed from a stable machine_id hash that is part of
the session ID. The granularity is therefore "this machine ran a
command on day N". Multi-machine users count as multiple users
(under-counts retention). This is acceptable for the Sean Ellis
check; flag it in the dashboard.

### Activation depth

**Definition**: of users in the retention cohort, fraction who
completed ≥5 successful commands by end of week 1.

**Why**: distinguishes "kicked the tires" from "adopted". The Sean
Ellis defended-cohort definition uses this as the cohort filter for
the qualitative survey.

### Backend-success rate by backend

**Definition**: among generated commands, fraction marked successful
by backend (embedded / MLX / CPU / Ollama / vLLM / OpenRouter).

**Why**: a backend with a 70% success rate while the population
averages 94.8% is a quality regression hiding behind the average.

### Time to First Command (TTFC)

**Definition**: median + p95 seconds from session start to first
successful command in the same session.

**Why**: matches the existing v1.1.0 success-criterion (target <3s)
and is the cleanest install-funnel telemetry we already collect.

### Safety block rate

**Definition**: fraction of generated commands the safety validator
blocked at risk levels CRITICAL/HIGH/MEDIUM. Already informally
tracked; surface in dashboard for trend visibility.

### Error rate by category

**Definition**: fraction of sessions that emitted ≥1 error event,
sliced by error category (model-failure / safety-rejection / cache-
miss / config / network / other). Categories per existing
telemetry schema.

## Sean Ellis instrument

The playbook recommends the Sean Ellis test (>40% "very disappointed
without your product") as the PMF check. The playbook also corrects
the common misuse: the survey is only meaningful with a defended
cohort.

**Trigger condition**: send the survey to a sampled subset of users
who:
- Completed ≥5 successful commands in week 1 (activation depth), AND
- Were on day ≥14 of opt-in telemetry, AND
- Have not been sampled in the prior 90 days

**Sample size target**: ≥200 responses per quarter. With the
cohort filter above, this requires ~1,500 invited users; the
opt-in cohort needs to be at that scale for the survey to even be
runnable. *Until it is, we do not run the survey.* Running it on a
smaller cohort and citing 47% is exactly the misuse the playbook
warns about.

**Delivery**: in-CLI prompt one time, dismissable, with a link to a
form that does NOT require login. Form vendor TBD; the form's
question text is in this spec so it doesn't drift.

**Question text**:

> "Caro is in early-stage development. To help us understand fit:
> how would you feel if you could no longer use Caro?
> [ ] Very disappointed
> [ ] Somewhat disappointed
> [ ] Not disappointed (it isn't really useful)
> [ ] N/A — I no longer use it"

Standard Sean Ellis phrasing. Don't tinker with it.

## Surfaces

| Surface | Audience | Update cadence | Visibility |
| --- | --- | --- | --- |
| **Internal dashboard** (Grafana / similar) | maintainers | hourly | private; auth-gated |
| **Public North-Star** (caro.sh/telemetry or /metrics) | community | weekly | aggregate-only; no per-user data |
| **Weekly briefing** (`.hermes/digests/`) | founder + maintainers | weekly | private; Hermes narrative + numbers |
| **Quarterly retention report** | community + investors | quarterly | aggregate-only; methodology in this spec |

The existing [`website/src/pages/telemetry.astro`](../website/src/pages/telemetry.astro)
page is the natural home for the public-North-Star view.

## Implementation notes

- **Ingest**: the v1.1.0 roadmap mentions `telemetry.caro.sh` as the
  ingest service (deferred post-release). This spec assumes that
  service exists by the time the dashboard ships. If it doesn't, the
  dashboard runs on locally-exported telemetry per `caro telemetry
  export`.
- **Storage**: time-series store (DuckDB / Postgres / ClickHouse —
  TBD) with retention of raw events ≤ 13 months. Aggregates retained
  indefinitely.
- **Compute**: D7 is computed as a 7-day moving window of activation-
  cohort returns. Don't use 7-day rolling totals (different metric).
- **Backfill**: the metrics start with the v1.1.0 GA opt-in
  population. Pre-GA telemetry was opt-out and is deliberately not
  backfilled (different consent shape).
- **Privacy review**: any change to the metric definitions requires a
  re-run of [`docs/TELEMETRY.md`](./TELEMETRY.md)'s privacy audit.

## What this dashboard does NOT show

- **Individual users.** All views are aggregate.
- **Command content.** No prompt strings, no command strings, no
  files. The dashboard cannot answer "what commands are people
  running".
- **Cross-machine identity.** A user with two machines counts as
  two; we don't reconstruct identity across devices.
- **Cohorts smaller than 50 sessions.** Suppressed for re-identification
  resistance.
- **Real-time anything.** Latency is hours, not seconds. There is no
  business reason for real-time.

## How this serves the Stage-3 exit gate

The [`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md) Stage-3 exit
table:

| Playbook criterion | What this dashboard provides |
| --- | --- |
| Retention curve flattens | D7 / D30 over rolling cohorts |
| Proactive user recall | Sean Ellis result on defended cohort |
| Sustainable CAC | (Out of scope until pricing-page demand signal lands; see Phase 5 of the playbook adaptation) |

Two of three exit gates land here. The third (CAC) requires paid
acquisition, which Caro does not run today.

## See also

- [`docs/TELEMETRY.md`](./TELEMETRY.md) — the telemetry contract this
  spec must not violate
- [`.claude/rules/validation-discipline.md`](../.claude/rules/validation-discipline.md) —
  Gate 5 (Sean Ellis with defended cohort) — this dashboard is how
  we clear it
- [`playbook/STAGE_MAP.md`](../playbook/STAGE_MAP.md) — the stage
  this measurement serves
- [`docs/launch-os.md`](./launch-os.md) — the dashboard is one of the
  four named Launch-OS gaps
- [`website/src/pages/telemetry.astro`](../website/src/pages/telemetry.astro) —
  natural home for the public-North-Star view

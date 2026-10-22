# Discovery Outreach Templates

Starting points for the founder to personalize and send. Each
template is **a draft**, not a final message. Per
[`README.md` § Candidate sourcing](./README.md#candidate-sourcing),
the founder (or a delegated human maintainer) sends the actual
outreach.

## Why these are templates, not bulk sends

The validation-discipline rule's Gate 1 says: *"First-hand means
**you** talked to them."* A bulk-send blast is not an interview
recruitment — it's spam, and it'll burn the cohort before they ever
agree to talk. Each template below is calibrated to be:

- **Honest about the ask** — "we're validating a hypothesis", not
  "you have to try our product"
- **Short** — under 200 words so the recipient can read it on a
  phone screen
- **Personalized at minimum** with the recipient's specific
  signal (issue number, star date, post they made, etc.)
- **Easy to decline** — explicit "no worries if not interested"

## Template 1 — Waitlist segmented outreach

**Use when**: contacting a person who joined the waitlist with a
specific `source` value that maps to a hypothesis you're validating
(e.g. `pricing-enterprise` → CISO-track interview for
`enterprise-dashboard` hypothesis).

**Subject**: A quick 20-min chat about your terminal workflow?

> Hi {{anon_first_name_or_handle}},
>
> Thanks for joining the Caro waitlist on {{date}}. You signed up
> via the {{source}} CTA, which is exactly the kind of signal I
> want to dig into before we build the {{feature_or_tier}} side of
> Caro.
>
> I'm running 20 short interviews to validate the hypothesis
> behind that direction before we ship anything. They're 20–30
> minutes, recorded if you consent (anonymized in our public
> discovery repo, your handle hashed), and they're emphatically
> not a sales pitch — Caro's Community edition is and will stay
> free under AGPL-3.0.
>
> If you'd be up for it, here's my calendar: {{calendly_link}}.
> If not, no worries — I won't email you again unprompted.
>
> Either way, your waitlist signup is appreciated and you'll
> get the launch announcement when we hit GA.
>
> Kobi

## Template 2 — GitHub stargazer outreach

**Use when**: contacting a recent GitHub stargazer who has visible
public activity in adjacent tools (shell-AI, dev-tools, Rust CLIs)
that suggests they have shell-command pain.

**Channel**: GitHub Discussions DM, or — if their GitHub profile
links to their preferred channel (Twitter / website email) — there.

> Hi {{anon_handle}},
>
> I noticed you starred [caro](https://github.com/wildcard/caro)
> on {{date}}, and your public {{Rust / dev-tools / shell-AI}}
> activity on {{repo_or_topic}} suggests you might run into the
> exact kind of "I know what I want but not the syntax" friction
> Caro is built around.
>
> I'm running a small batch of structured interviews (20–30 min)
> to validate which v2.0 features are worth building. Specifically
> right now I'm trying to surface real first-hand pain around
> {{hypothesis_in_one_sentence}}.
>
> Would you be willing to do one? It's anonymized in our public
> discovery repo (your handle gets hashed), no sales pitch
> attached — Community Caro is free forever under AGPL-3.0.
>
> Calendar: {{calendly_link}}. Or just reply here with a
> "yes / no / maybe later" and I'll take it from there.
>
> — Kobi

## Template 3 — Direct network / "warm intro" outreach

**Use when**: reaching out to someone in your personal/professional
network — a former colleague, a known power-user, an OSS
collaborator on an adjacent project.

**Channel**: whatever you usually use with them (DM, Signal, email,
Slack).

> Hey {{first_name}} —
>
> Caro shipped 1.4.0 last month and I'm starting structured
> discovery work on v2.0 before committing to features. Specifically
> trying to surface real first-hand pain around
> {{hypothesis_in_one_sentence}}.
>
> Would you do a 20–30 min interview? It's qualitative, no sales
> pitch, and the transcript goes in our public discovery repo
> anonymized — your handle gets hashed, your company gets
> generalized to "{{industry}}-co" unless you tell me otherwise.
>
> Heads up: you're closer to my network than the average
> interviewee, so I'll flag you as "warm cohort" in the synthesis
> so we don't kid ourselves about generalizability — but your
> signal is still useful because you actually use a terminal
> every day.
>
> Up for it? {{calendly_link}} or just pick a time.
>
> — Kobi

## Template 4 — Public reply to a shell-AI complaint

**Use when**: someone on Twitter / Mastodon / Hacker News / Reddit
posts a public complaint about shell-AI tools that matches a
hypothesis you're validating.

**Channel**: reply in the original thread, NOT a DM. DMs from
strangers chasing a complaint feel pushy; public replies on the
same thread are conversational.

> Caro maintainer here — your point about
> {{specific_thing_they_said}} is exactly the kind of signal
> I'm trying to ground v2.0 in. We're doing 20 structured
> interviews per feature direction before committing to build,
> and your shape of pain matches the {{hypothesis_id}} hypothesis
> we're testing.
>
> If you'd be up for a 20-min conversation, my calendar's at
> {{calendly_link}} — no sales pitch, Caro Community is free
> under AGPL-3.0, transcript would be anonymized in our public
> discovery repo.
>
> No pressure if not — your public comment alone is useful
> signal.

## Anti-patterns

- **Mass mail-merge** to the full 247-person waitlist. Spam, will
  burn the list.
- **Multiple unanswered follow-ups.** One ask, one decline-or-silence,
  then move on. Silence is data.
- **Citing internal docs in cold outreach.** "Per ADR-001…" and
  "per validation-discipline.md…" are great in internal contexts
  and confusing in cold outreach. Translate to plain English.
- **Promising features in exchange for interviews.** That biases
  the response — the interview becomes a sales conversation in
  disguise. Caro doesn't trade access for testimonials.
- **AI-drafted outreach sent without personalization.** If the
  template's `{{slot}}` placeholders ship unfilled, the recipient
  knows. Always personalize.

## What goes in the transcript

After the interview, capture per the rules in
[`README.md`](./README.md#how-to-add-a-transcript) and
[`interview-template.md`](./interview-template.md). The outreach
message itself doesn't go in the transcript — only the conversation
that follows.

## See also

- [`README.md`](./README.md) — anonymization rules, candidate sourcing
- [`interview-template.md`](./interview-template.md) — the question
  script the interview itself follows
- [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md) —
  the rule this outreach serves

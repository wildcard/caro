# Brand Audit — Waitlist Surface (PR #599)

**Date:** 2026-05-14
**Auditor:** caro-waitlist-engineer (inline brand-engineering review)
**Scope:** `website/src/components/landing/LPWaitlist.astro` only (the
`Waitlist.astro` standalone component is not mounted on any page in this PR)
**Preview:** https://caro-foss-website-2o47o6e83-kadosh-dev.vercel.app
**Screenshots reviewed:**
- `.claude/audits/2026-05-14/remote-e2e/01-lpwaitlist-before.png`
- `.claude/audits/2026-05-14/remote-e2e/03-lpwaitlist-success.png`
- `.claude/audits/2026-05-14/remote-e2e/04-lpwaitlist-error-validation.png`
- `.claude/audits/2026-05-14/local-e2e/01-local-lpwaitlist-before.png`
- `.claude/audits/2026-05-14/local-e2e/02-local-lpwaitlist-success.png`

---

## 1. Audit summary

The LPWaitlist surface ships in good brand health. Tokens are used
consistently for `--accent`, `--bg-inverse`, `--fg-inverse`, `--radius-md`,
`--shadow-sm/lg`, `--border-strong`. The card sits on `--bg-inverse` (dark)
with the section also on `--bg-inverse`, producing a single dark island
inside the otherwise paper-light landing page — consistent with the 95/5
composition rule (most of the page is paper; this is the 5%).

The component honours the **no-left-accent-stripe** card spec from the
post-pixel-icon brand reset: instead of a vertical accent bar, it uses a
**1px top accent line** via `::before` (lines 176–184). This is a creative
honouring of "kill the stripe" while still signaling brand on the surface.

Form interactions (focus ring, success colour change, hover lift) are
tasteful and aligned with the LPHero gold-standard component.

## 2. Gaps found

| Priority | File / line | Finding |
|---|---|---|
| P2 | LPWaitlist.astro:193 | `background: rgba(239, 51, 51, 0.18)` — hardcoded RGB tied to the legacy `#ef3333` red. Tokens system should expose `--accent-translucent-light` (or similar) for the 18% tint so future palette shifts don't need code touches in three places. |
| P2 | LPWaitlist.astro:280 | `box-shadow: 0 0 0 3px rgba(239, 51, 51, 0.15)` — same hardcoded RGB issue as above for the focus ring. Should be tokenised. |
| P2 | LPWaitlist.astro:233 | `color: #4ade80` for checkmark — hardcoded green, not a `--success` or `--accent-success` token. Acceptable for v1 but worth tokenising. |
| P2 | LPWaitlist.astro:312 | `background: #15803d` on the success state — same as above; should pair with the checkmark token. |
| P2 | LPWaitlist.astro:209, 228, 325 | Inline fallback color literals (`var(--caro-grey-300, #cccccc)`, `var(--caro-grey-400, #999)`) — the fallbacks suggest these tokens may not exist in `tokens.css`. Either define them or use existing `--fg-muted` etc. |
| P3 | LPWaitlist.astro:170-174 | Card hover does a `translateY(-4px)` lift. The post-pixel brand reset removed scale/lift hover from card spec; this is a translate not a scale so it sits in a gray area. Worth flagging to Claude Design for a ruling. |
| P3 | LPWaitlist.astro:67-72 | Code comment references `caro-jac.9` for the dropped honeypot/upstash safeguards — confirms the tracking bead exists and is wired into the code. Good. |

**No P0/P1 findings.** The surface is shippable.

## 3. Decisions

1. **Ship as-is** — no P0/P1 blockers exist.
2. **Tokenise red-translucent and green pair** in a follow-up — file as
   `caro-jac.10` (or similar) under the v2.1 brand-system epic. This is
   a cleanup, not a fix.
3. **`Waitlist.astro` (community standalone)** — the file exists at
   `website/src/components/Waitlist.astro` (397 lines) but is not imported
   by any page. It duplicates the LPWaitlist functionality with a slightly
   different layout. **Recommendation: defer the decision to wildcard.**
   Either (a) delete it to reduce maintenance surface, or (b) wire it to
   `/community` if the original plan was a separate community-page surface.

## 4. Files changed by this audit

None. The audit is read-only; recommendations are deferred to follow-up
beads.

## 5. Beads filed

To be filed under the `caro-jac` epic post-merge:

- **caro-jac.NEW-A** (P2): Tokenise `rgba(239, 51, 51, 0.18)` and the
  paired green `#4ade80`/`#15803d` in `tokens.css`; replace the three
  LPWaitlist hardcoded uses.
- **caro-jac.NEW-B** (P3): Decide fate of unmounted `Waitlist.astro` —
  delete or wire to a `/community` route.
- **caro-jac.NEW-C** (P3): Confirm with Claude Design whether the
  card-hover `translateY(-4px)` is in or out of the brand spec.

(Numbering deferred to the post-merge bead-creation step.)

## 6. Next move

Proceed to Phase 2H — closing acceptance comment + handoff. Surface the
`Waitlist.astro`-unmounted finding and the P2 token gaps in the PR
comment under a "Notes for follow-up" section so wildcard can decide
whether to address them in this PR or defer.

---

## Verdict (1 sentence)

LPWaitlist ships in good brand health with only P2/P3 token-hardcoding
and one unmounted-component question; no P0/P1 blockers, merge is safe.

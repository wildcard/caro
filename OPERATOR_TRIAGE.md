# Caro Operator Triage Playbook

**Created**: March 25, 2026
**Operator**: Anastasia (Nastya)
**Session context**: Project resumed after ~2 months dormancy. v1.2.0 deadline pushed to April 30.

---

## Decisions Made This Session

### Milestone Adjustments
- **v1.2.0 deadline**: Pushed from March 31 → **April 30, 2026**
- Rationale: 47% complete with 45 open items after 2 months dormancy; unrealistic to ship in 6 days

### PR Triage Decisions

#### Batch: Bot PRs (27 PRs) → MERGE ALL
Merge all dependabot dependency bumps and i18n automation PRs:
- Dependabot: #778, #777, #776, #775, #774, #773, #767, #764, #763, #743, #555, #554, #553, #552, #551, #550, #424, #423
- i18n bot: #760, #759, #758, #757, #756, #755, #754, #753, #752, #751, #750, #749, #748, #747

#### Batch: Copilot Drafts → KEEP ALL EXCEPT #206
- **KEEP** #288 (Automated GitHub Releases) — substantive release automation
- **KEEP** #287 (User Manual + Man Pages) — full man page and shell completions
- **KEEP** #286 (Open Source Building Guide) — reusable StaticPage layout + content
- **KEEP** #285 (Investor Pitch Deck PDF) — PDF export, needs cleanup
- **KEEP** #205 (UGC Content Pipeline) — Keystatic CMS foundation
- **CLOSE** #206 (Rebase Conflict Investigation) — diagnostic doc, served its purpose

#### Batch: Old PRs (#80-#184) → KEEP ALL
All old PRs kept open for future work. No closures.

---

## Feature PRs by Milestone — Triage Queue

Work through these PRs in priority order within each milestone. Priority is based on: (1) user impact, (2) v1.2.0 website launch readiness, (3) dependency chains.

### v1.2.0 — Website & Documentation Launch (Due: April 30)

| Priority | PR | Title | Size | Recommendation |
|----------|-----|-------|------|----------------|
| P0 | #681 | Resolve merge conflicts for 23 v1.2.0 PRs | Issue | **BLOCKER**: Fix conflicts before merging other v1.2.0 PRs |
| P0 | #130 | Build interactive terminal landing page | XL | Core deliverable — the main landing page at caro.sh |
| P0 | #639 | Version header, security notes, docs pages | XL | Essential for docs site completeness |
| P1 | #81 | SEO and social media meta tags | L | Needed before any public launch |
| P1 | #620 | Complete Nix support for reproducible builds | L | Distribution channel expansion |
| P1 | #652 | Developer Certificate of Origin (DCO) | XS | Quick merge — governance requirement |
| P1 | #680 | Translation automation scripts | XL | Enables scalable i18n going forward |
| P1 | #573 | Improve installation section (GitHub CLI inspired) | L | First impression for new users |
| P2 | #611 | Instagram feed in website footer | XL | Nice-to-have, not critical for launch |
| P2 | #651 | Reusable prompt for product launch analysis | XS | Quick merge — docs utility |
| P2 | #572 | Update generated search index and llm.txt | XL | Website maintenance |
| P2 | #568 | Update search index with TellYourCTO component | XL | Website component |
| P2 | #599 | Astro DB with Turso for community waitlist | XL | Community feature — can defer |
| P2 | #595 | Homebrew formula with real SHA256 checksums | L | Distribution — important but not blocking |
| P3 | #451 | Multi-channel product announcement plan | XL | Marketing plan doc |
| P3 | #419 | Research blog post ideas | XL | Content planning |
| P3 | #418 | Review SEO project for blog workflow | XS | Quick read |
| P3 | #417 | Agent work patterns research | XS | Research doc |
| P3 | #416 | Swift ecosystem blog post | L | Blog content |
| P3 | #414 | Competitor alternative landing pages | XL | Marketing |
| P3 | #413 | Seattle AI Summit landing page | XL | Event marketing — may be stale |
| P3 | #400 | ASCII art demo in README | XS | Quick win for first impressions |
| P3 | #396 | Buildkite-inspired landing page | XL | Alternative design — may conflict with #130 |
| P3 | #331 | Installation and setup docs pages | XL | Overlaps with #573 |

### v1.3.0 — Core Features (Due: May 31)

| Priority | PR | Title | Size | Recommendation |
|----------|-----|-------|------|----------------|
| P0 | #657 | Resolve all CI build failures and test issues | XL | **BLOCKER**: CI must be green |
| P1 | #636 | Starship crate for enhanced context | XL | Key differentiator — richer environment awareness |
| P1 | #643 | Proactive Suggested Queries | XL | Flagship UX feature |
| P1 | #644 | Capability boundaries — Caro knows limits | XL | Safety/UX improvement |
| P1 | #619 | ShellCheck integration for post-processing | XL | Quality improvement for generated commands |
| P1 | #578 | User feedback system (Phase 1 MVP) | XL | Critical for learning what users need |
| P2 | #648 | Release Please for automated releases | XL | DevOps improvement (overlaps with #288) |
| P2 | #647 | Request memory tracking | XL | Analytics/improvement feature |
| P2 | #645 | Directory context in inference prompts | M | Better command generation from context |
| P2 | #640 | AI code reviewer agent integration | XS | Quick merge — tooling |
| P2 | #638 | Cargo.lock for resource assessment deps | XL | Dependency management |
| P2 | #629 | User feedback + --no-spellcheck flag | XL | UX improvement (overlaps #578?) |
| P2 | #626 | GLM-4.6V-Flash integration tests | L | Backend testing |
| P2 | #608 | Prioritize smaller Qwen models for 100Mbps | XL | **NEW**: Align with Qwen 3.5 Coder plans |
| P2 | #605 | Fix first-run model download with retry | L | Bug fix — critical for new users |
| P2 | #606 | Tests and CI for setup script | XL | Quality |
| P3 | #582 | TLDR client for command context enrichment | XL | Knowledge base expansion |
| P3 | #581 | Project roadmap with gaps and completion path | XL | Planning doc |
| P3 | #580 | Oh My Zsh/Bash plugin support | L | Distribution channel |
| P3 | #575 | Skip pattern matching on Windows | M | Platform compatibility |
| P3 | #574 | Windows PATH setup improvements | L | Platform compatibility |
| P3 | #571 | Intelligent clarification UX | M | UX research |
| P3 | #570 | Strategic vision and Anthropic recognition | XS | Docs |
| P3 | #569 | Standard CLI version output format | S | Quick fix |
| P3 | #567 | UX for long-running commands | XS | Docs/design |
| P3 | #566 | File type categories for smarter commands | M | Pattern expansion |
| P3 | #565 | Agent reasoning mode | XL | Pre-processing feature |
| P3 | #564 | Agentic idea pipeline from live data | XL | Research/automation |
| P3 | #563 | Progress indicators for command execution | XL | UX improvement |
| P3 | #562 | Ask mode for question handling | XL | New interaction mode |
| P3 | #561 | Binary installation docs | S | Quick docs PR |
| P3 | #560 | Explain mode for commands | XL | New interaction mode |
| P3 | #559 | Fix elevator music scroll bug | M | Website bug |
| P3 | #558 | FunctionGemma + LM Studio backend | XS | Backend spec |
| P3 | #557 | Astro development skills for website | XS | Dev tooling |
| P3 | #556 | Debug pydantic validation | L | Website bug |
| P3 | #527 | Separate knowledge integration tests | M | Testing improvement |
| P3 | #526 | CLI subcommands for knowledge index | L | Feature |
| P3 | #515 | Expand static_matcher patterns (#511) | L | Safety pattern expansion |
| P3 | #514 | Safety validation for remote backends | XL | Security feature |
| P3 | #503 | API key redaction in Debug trait (#441) | M | Security fix |
| P3 | #502 | Prompt validation with warnings (#462) | M | UX improvement |
| P3 | #500 | Self-healing for permission errors | L | Resilience feature |
| P3 | #499 | Research Caro keywords | XL | SEO research |
| P3 | #497 | Advanced tool use patterns guide (#168) | XS | Docs |
| P3 | #496 | Knowledge index CLI subcommands (#492) | L | Duplicate of #526? |
| P3 | #468 | File type search patterns | L | Pattern expansion |
| P3 | #273 | Command alias suggestions | XL | Feature |

### v2.0.0 — Advanced Features (Due: June 30)

| Priority | PR | Title | Size | Recommendation |
|----------|-----|-------|------|----------------|
| P1 | #660 | Switch Qdrant → ChromaDB for vector store | XL | Architecture decision |
| P1 | #659 | Azure Foundry backend (enterprise) | XL | Enterprise expansion |
| P1 | #658 | Handy.Computer integration Phase 1 | XL | New integration |
| P1 | #650 | vLLM Jukebox multi-model server | XL | Backend feature |
| P2 | #641 | Interactive TUI welcome screen | XL | UX feature |
| P2 | #628 | P2P distributed networking (Karo) | XL | Core v2 architecture |
| P3 | #609 | Fix install script arguments | M | Bug fix |

### Standalone / No Milestone

| PR | Title | Size | Recommendation |
|----|-------|------|----------------|
| #769 | ADR-015 AgentSH integration | XS | Review and decide — ADR needs accept/reject |
| #744 | Greptile.json for PR review config | M | Quick tooling improvement |
| #597 | CLAUDE.md from habit-tracker patterns | XS | Dev tooling |
| #596 | System capability profile detection | S | Core feature — assign to milestone |
| #401 | Kibble dogfooding strategy | M | Planning doc |

---

## Qwen 3.5 Coder Integration Opportunity

As discussed, the Qwen ecosystem has new models worth integrating:
- **Qwen Coder 3.5** (and the Claude Opus 4.6 fine-tuned variant)
- PR #608 already plans for smaller Qwen models
- This could significantly boost Caro's offline command generation quality
- **Action**: Create a new issue/spec for Qwen 3.5 Coder integration, link to #608

---

## Issue Triage (93 Open Issues)

### By Category (from GitHub scan):
- **i18n**: #746, #745, #690, #689, #688, #687 — translation coverage and localization
- **v1.2.0 blockers**: #681 (merge conflicts for 23 PRs)
- **v1.3.0 features**: #674, #673, #671, #670 — core feature tracking issues
- **v2.0.0 epics**: #672, #661-#668 — advanced feature epics

### Recommended Next Step for Issues:
Run through the full issue list and categorize into: close (stale/duplicate), keep (active), or defer (future milestone).

---

## ADR Decisions Needed

14 ADRs in "Proposed" status need accept/reject decisions:

| ADR | Title | Recommendation |
|-----|-------|---------------|
| ADR-001 | Enterprise vs Community Architecture | **Accept** — dual edition model is the business plan |
| ADR-002 | Governance and Provisioning | **Accept** — needed for enterprise edition |
| ADR-003 | Monitoring and Audit Trail | **Accept** — enterprise compliance requirement |
| ADR-004 | Just-Based Runbook Integration | **Review** — evaluate if Just is still the right choice |
| ADR-005 | Hayagriva Bibliography Integration | **Review** — niche, assess actual need |
| ADR-006 | OLMo 3 Model Support | **Review** — may be superseded by Qwen 3.5 plans |
| ADR-007 | AST Parser for Shell Validation | **Accept** — strong safety improvement |
| ADR-008 | Self-Update Mechanism | **Accept** — standard for CLI tools |
| ADR-009 | Website Claims Verification | Already **Accepted** |
| ADR-010 | Bubblewrap Sandbox | **Accept** — key for agent guardian use case |
| ADR-011 | cmd_lib Evaluation | Already **Rejected** |
| ADR-012 | Honggfuzz for Fuzz Testing | **Accept** — security best practice |
| ADR-013 | Pre-Processing Pipeline | **Accept** — architecture improvement |
| ADR-014 | serde-env for Environment Variables | **Accept** — developer ergonomics |
| ADR-015 | AgentSH Integration (PR #769) | **Review** — new, needs evaluation |

---

## How to Use This Document

### For Nastya (current session):
1. Start with **P0 items** in v1.2.0 — especially #681 (merge conflicts blocker)
2. Work through P1 items, making merge/defer/close decisions
3. Review ADRs and make accept/reject decisions
4. Move to issue triage when PRs are under control

### For any future operator:
1. Run `gh issue list --state open` and `gh pr list --state open` to get current counts
2. Read ROADMAP.md for milestone context
3. Use the priority framework: P0 (blockers) → P1 (high impact) → P2 (nice to have) → P3 (backlog)
4. Make decisions in batches, not one-by-one
5. Update this document with decisions made

---

*Last updated: March 25, 2026 — Session with Nastya*

---

## Session 2 — March 26, 2026

### Work Completed

#### Dependencies Merged (prior session, already done)
- 14 i18n bot PRs (#747–#760) — merged
- 10 dependabot PRs (#424, #743, #763, #764, #767, #773, #775, #776, #777, #778) — merged
- PR #206 (stale Copilot diagnostic) — closed
- PR #780 (ROADMAP date update) — created

#### Rebases Completed This Session

| PR | Branch | Status | Notes |
|----|--------|--------|-------|
| #639 | claude/add-init-setup-wizard-VP8Yh | **MERGEABLE** | Rebased + force-pushed. Telemetry + setup wizard both preserved. |
| #130 | claude/terminal-landing-demo-aJq1s | **MERGEABLE** | Rebased with 6 commits. Conflicts in index.astro, package-lock.json, LPNavigation resolved. LP component conflicts resolved by taking main's versions (newer). |
| #81 | claude/add-seo-meta-tags-ZzD5b | **MERGEABLE** | Rebased. SEO props merged into Layout.astro and BlogPost.astro. |
| #595 | claude/create-homebrew-tap-ghDPp | **MERGEABLE** | Rebased. Homebrew tab UI merged into Download.astro. |
| #573 | claude/improve-cli-installation-xgOUF | **MERGEABLE** | Rebased. PR's GitHub CLI-inspired installation structure kept. |
| #774 | dependabot/cargo/rust-minor-patch-9edf74a7c1 | **Pending recalc** | Head commit is fast-forward on main. GitHub still shows CONFLICTING (cache lag). Should auto-resolve. |
| #680 | 006-i18n-automation | **CONFLICTING** | 9 commits, complex JSON conflicts. Deferred per plan. Needs dedicated session. |

### Disposition Report (P0/P1 PRs)

| PR | Rebased? | Conflict-free? | CI green? | In v1.2.0 scope? | Recommended disposition |
|----|----------|----------------|-----------|------------------|------------------------|
| #639 | ✅ Yes | ✅ Yes (MERGEABLE) | ⚠️ Pre-existing failures (same as main) | ✅ Yes — docs pages | **Merge when CI issues tracked in #657** |
| #130 | ✅ Yes | ✅ Yes (MERGEABLE) | ⚠️ Pre-existing failures (same as main) | ✅ Yes — core landing page | **Merge when CI issues tracked in #657** |
| #81 | ✅ Yes | ✅ Yes (MERGEABLE) | ⚠️ Pre-existing failures (same as main) | ✅ Yes — pre-launch SEO | **Merge when CI issues tracked in #657** |
| #595 | ✅ Yes | ✅ Yes (MERGEABLE) | ⚠️ Pre-existing failures (same as main) | ✅ Yes — distribution | **Merge when CI issues tracked in #657** |
| #573 | ✅ Yes | ✅ Yes (MERGEABLE) | ⚠️ Pre-existing failures (same as main) | ✅ Yes — first impression | **Merge when CI issues tracked in #657** |
| #680 | ❌ No (aborted) | ❌ No (9 commits, JSON conflicts) | — | ✅ Yes — i18n infra | **Defer to dedicated rebase session** |

**CI Note**: All CI failures on rebased PRs are pre-existing failures identical to those on `main` (Lint, SpellCheck, MSRV, ShellCheck, Vercel). These are tracked under v1.3.0 issue #657. The rebases did not introduce new failures.

### Unresolved Blockers

1. **CI is broken on main** — All PRs will show CI failures until #657 (v1.3.0) is addressed. Decision needed: either fix CI before merging v1.2.0 PRs, or merge with known CI failures.
2. **PR #774 conflict** — Dependabot branch was deleted and recreated. GitHub may need up to 30 minutes to recalculate mergeability. Head commit is a fast-forward.
3. **PR #680 rebase** — 9 commits, conflicts in `search-index.json` and other files. Needs ~1 hour dedicated session.
4. **Major dep PRs #550-#555, #423** — Still deferred. Do not merge until after v1.2.0 launch.

### Next Recommended Tasks

1. **Decide on CI policy**: Can v1.2.0 PRs merge with pre-existing CI failures? If yes, merge #639, #130, #81, #595, #573 immediately.
2. **Rebase PR #680** in a dedicated session — 9 commits, complex JSON (search index) conflicts.
3. **Check PR #774** — Should auto-resolve to MERGEABLE within an hour. Merge if so.
4. **Review and merge remaining v1.2.0 P2 PRs** (#651, #652, #400) — all small and likely conflict-free.

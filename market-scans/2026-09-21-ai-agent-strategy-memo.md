# Caro Weekly Strategy Memo — AI Agent Launch Scan

**Window**: 2026-09-13 → 2026-09-21 (one carryover from Sep 13; see caveats)
**Produced by**: `scan-ai-agent-releases-for-caro-opportunities` scheduled task, autonomous run, no user present
**Prior memo**: `market-scans/2026-09-16-ai-agent-strategy-memo.md`
**Baseline**: `caro` 1.4.0, working tree @ `50859b89` (branch `integrator/20260711-postmerge`, HEAD authored 2026-07-16, divergent from `origin/main`)

> **This is a memo, not an implementation.** No branch, no PR, no code committed. This file is written to the working tree only.

**The one-line version:** the Product Hunt board was thin for Caro this week — another hand-rolled approval gate, another sandbox vendor, another tool-call protocol. The consequential finding came from reading around the board. CSA's September 10 note on the DeepSeek Harness sandbox escape links it to **GuardFall** (July 1), in which **ten of eleven popular coding agents were tricked into running commands their own guardrails were built to block, because they inspected raw command text before the shell performed quote removal and expansion**. Caro matches its patterns against the raw command string and applies a quote *heuristic that only ever suppresses detections*. **Verified by extracting all 67 pattern sources and running them: `rm -rf /` is caught; `rm -rf "/"` is not.** On the published failure mode that most precisely describes Caro's product category, Caro is in the failing group — and a second, cheaper bypass exists that needs no obfuscation of the command at all.

---

## 1. Market scan

Product Hunt, Sep 13–21. Nine relevant launches; the board was dominated by speed and consumer productivity, not trust infrastructure.

| Product | One-line | Core problem solved | Why it matters | Signal | Caro relevance |
|---|---|---|---|---|---|
| **Aside** · Sep 16 · ~160▲ · #7 · YC | AI browser that acts inside your logged-in accounts | Real work sits behind authentication — vendor portals, refunds, payments | Agent transacts with the user's live credentials, including **payments**. Makers give the control model in-thread: "permission level control" plus a **"Final confirm"** toggle that pauses before *send, pay, delete, submit*. Another product-specific approval gate, built from scratch. Also claims SOTA on agentic-browsing benchmarks, "outperforms Claude Cowork" — self-reported, benchmark not public. | **High** | **Direct** |
| **jurniti** · ~Sep 14 · 70▲ · #27 | Always-on agents in per-agent Firecracker microVMs | Agents die when the laptop lid closes; shared containers put client keys on a stranger's kernel | Hardware-boundary isolation (KVM, the Lambda primitive) at **$25/mo from a solo maker**. Isolation has finished commoditizing — it is now an indie SKU. Notable gap, raised in-thread and unanswered: an *always-on* agent has no kill-switch or spend cap; the user finds out from the bill. | **High** | **Direct** (threat + gap) |
| **Ruby UTCP** · Sep 20 · 95▲ · #9 | UTCP 1.1 for Ruby — "scalable, secure alternative to MCP" | The wrapper tax and server process MCP imposes on tool calling | 12 native transports (HTTP, CLI, WebSocket, gRPC, GraphQL, **MCP**, WebRTC) behind one manifest, plus **CodeMode** — multi-tool workflows expressed as compact Ruby. The tool-call substrate is fragmenting away from MCP, and CodeMode generates composite actions *between* per-call gate points. A gate wired to MCP covers a shrinking surface. | **Medium–High** | **Direct** |
| **AEXGrid** · Sep 18 · 55▲ · #82 | Coordinate Codex/Cursor/Kiro/OpenCode as a visual team | Context-copying across agents and accounts | Roles are **Explorer / Builder / Reviewer**, connected by "collaboration links and **reviewed handoffs**," with a GPT-6 Astra *manager* assigning work and routing output through review. Review is becoming **agent-to-agent**; the human is leaving the approval loop from the top. Low votes; directional only. | Medium | **Direct** |
| **Axari** · this month | "Assign your security busywork to your AI twin" | Security ops toil | Security review itself is now delegated to an agent. Same recursion as AEXGrid's Reviewer role, in the security domain. | Medium | Adjacent |
| **Accordio AI** · Sep 13 · 81▲ | "The admin tools your Claude is missing" | No governance around someone else's coding agent | Second week on the list (carryover — outside the strict Sep 14 window). Agent-agnostic governance bolt-on remains a viable shape, and the gap it patches is still unpatched by the agent vendor. | Medium | Adjacent |
| **MakersClaw** · this month | "The operating system for a company run by agents" | Running an org on agents | The control-plane shape again, indie edition. Consistent with last week's hob / Cadenya / Salesforce read. | Low–Med | Adjacent |
| **Toone** · this month | AI workspace for agentic workflows on macOS | Local agent orchestration | Local-first agent surface; no stated safety model. | Low | Weak |
| **Bolt Forge** · Sep 21 · 238▲ | Open-source AI agent, "50X faster usage" | Cost and latency of agent execution | **Top-voted agent launch of the window** — on speed, with no safety claim at all. The honest read on PH sentiment: throughput still outsells trust on this board. | Low | Weak |

### Off Product Hunt — more consequential than the board

- **CVE-2026-82533, DeepSeek Harness sandbox escape** (CVSS 9.4; CVE published Sep 8, CSA note Sep 10). An open-source coding-agent runtime at ~215K GitHub stars. A sandboxed agent could disable its own confinement with **a single `curl` on default settings** — the control API authenticated on the client-supplied `Host` header rather than the connection's peer address, and the sandbox restricted file writes while leaving loopback networking open. Routine shell commands needed no approval, so the escalation never surfaced to the user. Patched Aug 27; the *pattern* is not patched.
- **GuardFall** (CSA, Jul 1; re-surfaced by the Sep 10 note as one of three containment failures in one year). **Ten of eleven popular open-source coding agents** could be manipulated into executing shell commands their guardrails were designed to block, **because the agents evaluated raw command text before the shell performed quote removal and variable expansion.** This is the precise failure mode of a regex-over-raw-string validator.
- **Trust handoff** (CSA, Jul 22; Pillar Security across Cursor, Codex CLI, Gemini CLI, Antigravity). Attackers did not break the sandboxes — a confined agent wrote a file that a trusted component *outside* the sandbox later executed without re-validating.
- CSA's framing across all three: *"coding-agent containment is failing at multiple independent layers — command-level guardrails, downstream trust handoffs, and control-plane authentication — within the same year and the same product category."*

---

## 2. Market shifts

**Isolation has finished commoditizing downward, and is failing upward.** A solo maker now sells per-agent Firecracker microVMs for $25/mo. In the same window, the flagship open-source coding-agent runtime — 215K stars — shipped a sandbox a `curl` could switch off. "Runs in a sandbox" is simultaneously table stakes and, per three independent 2026 disclosures, not a boundary that holds. Building isolation is a losing position; knowing what the command *inside* the isolation actually does is the position nobody occupies.

**Text-level guardrails are now a named, published failure class.** Before GuardFall, "we validate commands against dangerous patterns" was a credible claim on its own. After it, the buyer's question is *at what layer* — before or after shell word expansion. Ten of eleven agents answered wrong. The category's differentiator moved this quarter from *whether* you gate to *whether your parser agrees with the shell's parser*, and that is measurable rather than rhetorical.

**Review is going agent-to-agent.** AEXGrid ships a Reviewer role and reviewed handoffs with an LLM manager routing work; Axari sells security review as an AI twin. A model reviewing a model has no floor unless something deterministic sits underneath.

**Every product keeps rebuilding the gate, and none of them can share one.** Aside's "Final confirm," AEXGrid's "reviewed handoffs," Accordio's bolt-on admin controls, and Relaticle's approval-gated writes (last window) are four independent implementations of *pause before the dangerous thing*, each with its own vocabulary and none with a portable payload. The vacuum is unchanged.

**The tool-call substrate is fragmenting, and composite actions are escaping per-call gates.** UTCP now spans 12 transports with MCP as merely one of them, and CodeMode expresses multi-tool workflows as code rather than as a sequence of gated calls. Any gate positioned as "MCP middleware" is betting on a narrowing chokepoint. The durable position is the *execution site* — the moment a command meets a shell — which is transport-agnostic by construction.

**Nobody is selling a kill-switch for always-on agents.** jurniti's own launch thread surfaces it and leaves it open: a persistent agent holding the user's model key with no spend cap or hard stop. Standing authority over time is a different risk shape from per-task authority, and the category has not named it yet.

---

## 3. Caro opportunities

### A. Make the validator parse the way the shell parses — the GuardFall gate
- **Problem.** Caro matches compiled regexes against the raw command string (`src/safety/mod.rs::is_dangerous_in_context`, l.432). There is no `shlex`, no `shell_words`, and no quote-removal or word-expansion pass in the safety path. What *does* exist is worse than nothing: before accepting a match, the function counts unescaped quotes preceding it and **discards the match if the count is odd** (l.432–455). Caro's quote handling is *suppressing*, never *normalizing*. Two verified consequences:
  - **Obfuscation bypass.** `rm -rf /` matches Critical patterns 1 and 2 (`patterns.rs:16`, `:24`). `rm -rf "/"`, `rm -rf '/'`, `rm -r${IFS}f /`, `r""m -rf /`, and `rm  -rf  "/"` match **none of the 67** — and the shell executes every one of them as `rm -rf /`.
  - **Suppression bypass, cheaper.** `echo "hi ; rm -rf /` — the unmatched quote earlier in the line makes the pre-match quote count odd, so the genuine Critical match on `rm -rf /` is discarded and the command is returned safe. No obfuscation of the dangerous token required.
- **Why now.** GuardFall put this failure mode in a named, cited, industry-published class, and the Sep 10 CSA note rebroadcast it as one of three structural containment failures. Every buyer evaluating a command-safety layer now has a specific question to ask, and Caro currently answers it the way the ten failing agents did.
- **User value.** The one claim Caro makes that nothing else in the market makes — deterministic, intent-aware validation of what a POSIX command actually does — becomes true against adversarial input, not only against well-formed input.
- **Market evidence.** GuardFall (10/11 agents); CVE-2026-82533's `curl`-from-inside-the-sandbox path; the `isTrustedApiRequest` root cause is the same species of error — trusting the claimed form of a thing rather than verifying it.
- **Fit.** This is not an adjacent feature. It is the correctness of the shipped product.
- **Priority: Now** · **Complexity: S for the test, M–L for the fix** · **Next step:** write the failing differential test first, per `safety-pattern-developer`. For every built-in, CVE (`src/safety/cve_patterns.rs` + `data/cve_rules/*.yaml`) and custom pattern, generate quoted / expanded / concatenated / `$IFS`-split variants plus an odd-quote-prefix variant, and assert the verdict is unchanged. Expect a large red bar; **that number is the finding, and it is an afternoon's work.** The fix — tokenizing with shell semantics before matching — is the M–L part and should be scoped separately once the number exists.

### B. Make Caro's own documentation true
- **Problem.** Three claims in the front-door documents are not supported by the tree. `README.md:82` and `:946` and `CLAUDE.md:32,108` say **"52+ dangerous command patterns"**; the tree has **67** built-in plus CVE rules plus user patterns. `README.md:34` carries **"zero false positives"** and **"93.1% pass rate"** in one sentence, and `ADR-063` states no false-positive corpus exists. After (A), "zero false positives" is also the wrong metric to lead with — the measured gap is false *negatives*.
- **Why now.** Two competitors this quarter (Harden, and Aside in its own category) lead with self-reported benchmark superiority against no public benchmark. Caro's differentiation is supposed to be that its claims are checkable. Right now Caro's are stale in one direction and unbacked in the other, and a competitor's first move in a bake-off is to run the GuardFall corpus.
- **User value.** A buyer can verify what they are told.
- **Market evidence.** Harden's "beats frontier models on agent-security benchmarks" and Aside's "outperforms Claude Cowork" — both unfalsifiable, both this quarter.
- **Fit.** `.claude/rules/validation-discipline.md` imposes evidence requirements on features; marketing claims should not be held to a lower bar.
- **Priority: Now** · **Complexity: S** · **Next step:** one PR correcting the pattern count and replacing the unbacked accuracy claims with either a measured figure or nothing. Publish the (A) corpus result alongside it, favourable or not.

### C. Ship a decision contract
- **Problem.** No portable decision payload exists, so every product hand-rolls its gate vocabulary.
- **Why now.** Fifth consecutive week as a recommendation. Fifteen ADRs have been written in this space since `ADR-059` declared itself the last one until the module merged; the module does not exist, and **neither do the ADRs, as far as the repository is concerned** — see the process note.
- **User value.** Aside's "Final confirm," AEXGrid's "reviewed handoff," and Accordio's admin gate could render the same verdict instead of inventing three.
- **Market evidence.** Four gates, four vocabularies, zero reuse, in three weeks.
- **Fit.** Structured assessment payloads, policy/reviewer separation. `caro guard`, OTel emission and surface adapters are all blocked on it.
- **Priority: Now** · **Complexity: M** · **Next step:** merge a module, not another ADR — and **pick a name that does not collide with the existing tracked `src/assessment/`**, which is hardware capability detection (`cpu.rs`, `gpu.rs`, `memory.rs`, `recommender.rs`) and has nothing to do with decisions. `src/decision/` or `src/verdict/`.

### D. Tighten the High-risk floor for agent-to-agent review
- **Problem.** `blend_smart_decision` (`src/safety/mod.rs:277`) holds the Critical floor absolutely — a Critical static verdict is returned unchanged regardless of judge confidence. But a judge above `SMART_JUDGE_MIN_CONFIDENCE` (0.7) **can relax High and Moderate**, including unblocking a statically-blocked `High` under `SafetyLevel::Strict`. The judge can never hard-block; escalation only reaches a confirmation gate. So the accurate claim is *the Critical floor holds; the High floor does not.*
- **Why now.** Softer than last memo framed it, and softer than this memo's first draft. Relaxation runs only under `ApprovalMode::Smart` (`src/cli/mod.rs:812`), and `ApprovalMode::Prompt` is `#[default]`, so **default-configuration Caro never relaxes anything.** The AEXGrid agent-to-agent scenario only bites operators who opted into `--approval smart` — which is exactly the population running unattended, but it is a population, not everyone.
- **User value.** Under the one mode where a model is in the approval seat, High-risk verdicts stop being negotiable.
- **Market evidence.** AEXGrid reviewed handoffs; Axari's security-review-as-agent; CSA's escalation-without-surfacing finding.
- **Fit.** Policy/reviewer separation — the reviewer should not be able to rewrite the policy.
- **Priority: Next** · **Complexity: S** · **Next step:** restrict the judge to escalation-only for `High` when `SafetyLevel::Strict` is set. Note that `relax_can_unblock_high_at_strict` is a `#[test] fn` (l.907) documenting the behaviour, **not a flag to flip** — the change is in the function body. The "make it opt-in" half of last memo's version of this recommendation is already shipped via the `ApprovalMode` default.

### E. Position at the execution site, not the protocol
- **Problem.** UTCP's 12 transports and CodeMode show the tool-call chokepoint fragmenting. Gate-as-MCP-middleware bets on a narrowing surface.
- **Why now.** UTCP is not yet a movement, but it is a second credible substrate, and CodeMode specifically generates composite shell actions outside per-call gating.
- **User value.** One integration story that survives whichever protocol wins.
- **Market evidence.** Ruby UTCP (12 transports including MCP; CodeMode). `caro-scope-execution-sites-2026-09-08.md` / `ADR-067` already model this internally.
- **Fit.** Agent-agnostic by construction.
- **Priority: Next** · **Complexity: S** (positioning and docs; the decomposition work exists) · **Next step:** state "we gate the exec, not the protocol" explicitly in `README.md` and on the site, with UTCP/CodeMode as the worked example. Fold into the (B) documentation PR.

---

## 4. Recommendation

**Top 3**

1. **Write the differential obfuscation test (A).** GuardFall is a published, cited, ten-of-eleven failure in Caro's exact category, and the tree says Caro is in the failing group — verified, not inferred. Everything Caro claims is downstream of matching what the shell will actually run. The quote-suppression path is the sharper finding: `echo "hi ; rm -rf /` defeats a Critical pattern with a single unmatched quote and no obfuscation at all.
2. **Correct the documentation (B).** Smallest item on the list. The pattern count is stale by fifteen, "zero false positives" has no artifact, and after (A) it is the wrong metric anyway. A safety product whose front page misstates its own capability has a credibility problem that no feature fixes.
3. **Ship the decision contract (C).** Fifth week. It stays at three — not because it matters less, but because publishing a decision contract out of a validator that a pair of quotes defeats exports the defect rather than fixing it.

**Build or test next — one thing:** the **differential obfuscation test suite**. Extract every pattern, generate obfuscated and odd-quote-prefixed variants, assert the verdict is unchanged. No new architecture, no dependency, no decision required. It produces a number, and that number determines whether the next quarter is a decision contract or a tokenizer. Start with `rm -rf "/"` and `echo "hi ; rm -rf /`.

**Do not build right now: sandboxing, isolation, or a hosted agent runtime.** A solo maker shipped per-agent Firecracker microVMs at $25/mo this week, and CSA published its third 2026 finding that sandboxes in this category do not hold anyway. Building isolation means competing on price with commodity infrastructure while inheriting a failure class you cannot fix. **Secondary avoid: the multi-agent orchestration console** (AEXGrid, MakersClaw, plus hob, Cadenya and Salesforce's control plane from last window). Five entrants and an incumbent in two weeks is a crowding signal, not an opening. Caro's defensible position is being the component inside someone else's gate that they cannot cheaply rebuild — and after GuardFall, that is a validator whose parser agrees with the shell's.

---

## Process note — the finding is worse than "ADRs outpace modules"

Last memo diagnosed an ADR-to-module gap: `ADR-059` declared a moratorium, thirteen ADRs routed around it, no module shipped. Verification for this memo found the diagnosis was too generous.

`docs/adr/` holds ADR-060 through ADR-074 contiguously — fifteen since the moratorium, two of them written during this window. **Twenty ADR files are tracked by git. The rest, including every one of those fifteen, are untracked working-tree files**, not gitignored, never committed, with mtimes on a daily 02:4x cadence (070 = Sep 14, 071 = Sep 15, 072 = Sep 16, 073 = Sep 17, 074 = Sep 18). The same is true of the `caro-scope-*.md` files and of `market-scans/` itself. `git ls-files --others --exclude-standard` returns **450 untracked files**.

So the gap is not that decisions merge while code does not. **Nothing merges.** `HEAD` is `50859b89`, authored **2026-07-16**, on `integrator/20260711-postmerge`, which is not an ancestor of `origin/main`. The checkout has been producing artifacts into a divergent working tree for two months. Last memo's "zero commits since Sep 16" — which this memo's first draft repeated — is close to tautological on a checkout that far behind, and is withdrawn as a finding.

This is not a recommendation for another rule. It is the observation that the daily-cadence artifact generator is the most reliable process in this repository, and it writes to a location git does not see. Recommendation (A) is chosen partly because it is the one item on the list that produces a *failing test* — the artifact type most likely to force the question of where this work is supposed to land.

---

## Caveats on this scan

- **Product Hunt was read directly this week** (category board plus individual product pages), correcting last memo's primary sourcing gap. But only the `ai-agents` category board's "recently launched" view and the Sep 21 `agents-radar` digest were swept — **the daily leaderboards for Sep 14–19 were not paginated**, and the weekly leaderboard URL was not reachable from this session's fetch provenance. Low-vote launches, which is where safety infrastructure starts, remain under-represented. Launches were probably missed.
- **Vote counts and day ranks are as displayed at fetch time** and drift. Dates for Axari, MakersClaw, Toone and jurniti are "launched this week/this month" per the board rather than confirmed launch dates — **treat those rows as approximate on timing.** Accordio is a Sep 13 carryover, outside the nominal Sep 14 window; it is counted among the nine deliberately, and the window heading was widened to match.
- **Aside's "outperforms Claude Cowork" and jurniti's isolation claims are launch-page copy**, unverified. Aside's approval model is quoted from maker replies in its own comment thread, not documentation.
- **The GuardFall figure (10 of 11 agents) is CSA's characterization of its own July 1 note**, read via the September 10 note; the primary GuardFall publication was not fetched. CVE-2026-82533 details come from the CSA note citing OX Security, The Hacker News and DevOps.com; **no primary vendor source was fetched.** CSA itself flags that its three cases come from its own coverage, not an exhaustive survey.
- **Harden's benchmark claim and Relaticle's approval-gated writes are carried over from the Sep 16 memo** and were not re-verified this week. Neither product launched in this window.
- **The bypasses were executed against the extracted pattern sources, not through Caro's full pipeline.** `cargo test` was not run. All 67 pattern strings were extracted from `src/safety/patterns.rs` and matched in Python (equivalent syntax for these patterns) against candidate inputs; the quote-suppression case was derived by reading `is_dangerous_in_context` (l.432–455) rather than executed. `is_dangerous_in_context` can only *subtract* matches, so the real pipeline cannot do better than this result — but CVE and custom patterns were not included in the run, and `get_compiled_patterns_for_shell` (`patterns.rs:568`) filters by shell, so 67 is a ceiling rather than the active count for any given invocation. **Recommendation (A) exists to settle this inside the real pipeline.**
- **Repository state was verified directly** at `50859b89`: 67 `DangerPattern {` entries; `is_dangerous_in_context` matching compiled regexes against the raw `&str`; no `shlex`, `shell_words` or "quote removal" anywhere under `src/`; no `src/safety/assessment.rs`; no `caro.assessment.v1` or `DecisionContract` symbol; `blend_smart_decision` at `src/safety/mod.rs:277` called from `src/cli/mod.rs:822`; `ApprovalMode::Prompt` as `#[default]` at `src/models/mod.rs:288`; fifteen ADRs after `ADR-059`, all untracked; 450 untracked files total.
- **`git status` cannot run in this checkout** — `fatal: not a git repository: .git/worktrees/006-replace-ascii-morph` (dangling worktree registration). Untracked-file counts came from `git ls-files --others --exclude-standard`. An earlier draft of this memo asserted "git status clean"; that assertion was false and has been removed.
- **`origin/*` refs were fetched 2026-09-20** (no-op; `origin/main` tip unchanged at `be07b22b`, 2026-07-18) and carry branch commits through 2026-09-14. An earlier draft claimed the refs were stale since July; that was wrong. **Work may have landed on branches not examined here**, and the process note's scope is this checkout.
- **`ROADMAP.md` is dated May 9, 2026** and was not relied on.
- **This memo was adversarially fact-checked before delivery and materially revised.** Corrections made: the ADR count (sixteen → fifteen); the ADRs' status (described as landed → they are untracked, which changed the process note's conclusion); "git status clean" (removed as unverifiable); the grep caveat (an earlier draft claimed zero matches for `expand`, `normaliz` and `unquote` under `src/` — all three occur outside the safety path, and the caveat now names only the terms that are genuinely absent); the origin-ref staleness claim (withdrawn); three pattern citations that were line numbers presented as ordinals; and §3D, whose severity was **reduced** after reading `blend_smart_decision` — the Critical floor does hold, relaxation is scoped to High/Moderate under a non-default approval mode, and half its proposed next step was already shipped. §3D consequently dropped from the top three, and recommendation (B) was added in its place. The `src/assessment/` namespace collision and the stale "52+ patterns" claim were both found by the fact-check, not the original scan.
- **No user was present.** Which nine launches counted as relevant, the signal and relevance gradings, the priority ordering, and the decision to put a parser defect above the decision contract for a second week were made autonomously and are contestable.

---

## Sources

**Product Hunt** — [AI Agents category, recent launches](https://www.producthunt.com/categories/ai-agents?order=recent_launches&page=1) · [Aside](https://www.producthunt.com/products/aside-6) · [jurniti](https://www.producthunt.com/products/jurniti) · [UTCP / Ruby UTCP](https://www.producthunt.com/products/utcp) · [AEXGrid](https://www.producthunt.com/products/aexgrid) · [Accordio](https://www.producthunt.com/products/accordio) · [MakersClaw](https://www.producthunt.com/products/makersclaw) · [Axari](https://www.producthunt.com/products/axari) · [Toone](https://www.producthunt.com/products/toone) · [agents-radar PH AI digest, Sep 21](https://github.com/duanyytop/agents-radar/issues/3402)

**Security research** — [CSA: DeepSeek Harness Sandbox Escape and Agent Containment (Sep 10)](https://labs.cloudsecurityalliance.org/research/csa-research-note-deepseek-harness-sandbox-escape-20260910-c/) · [CSA: GuardFall — Shell Injection Bypass Defeats AI Coding Agent Guardrails (Jul 1)](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-agent-shell-injection-2026070/) · [CSA: AI Coding Agent Sandbox Escapes — The Trust Handoff Flaw (Jul 22)](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-sandbox-escapes-20260722-c/) · [OX Security: CVE-2026-82533](https://www.ox.security/blog/cve-2026-82533-deepseek-harness-ai-agent-sandbox-escape/)

**Prior memo** — `market-scans/2026-09-16-ai-agent-strategy-memo.md`

**Caro internal** — `src/safety/mod.rs` (`is_dangerous_in_context` l.432, `blend_smart_decision` l.277) · `src/safety/patterns.rs` (l.16, l.24, l.568) · `src/safety/cve_patterns.rs` · `src/cli/mod.rs:812` · `src/models/mod.rs:288` · `src/assessment/` (namespace collision) · `README.md:34,82,946` · `CLAUDE.md:32,108` · `docs/adr/ADR-059` → `ADR-074` · `ADR-063` · `caro-scope-execution-sites-2026-09-08.md` · `.claude/rules/validation-discipline.md`

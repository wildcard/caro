# ADR-071: `caro.posture.v1` — Does This Command Reconfigure the Thing That Would Stop the Next One?

- **Status**: Proposed (implementation ADR — buildable on the tree that ships in 1.4.0; no unlanded
  ADR is a prerequisite)
- **Date**: 2026-09-15
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: the **DeepSeek Harness (`dsh`) process sandbox** —
  [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness), MIT, 215k stars,
  released August 2026, self-described *developer preview*. Architecture read live 2026-09-15 from the
  vendor's own generated reference:
  [Process Sandbox](https://deepseek-harness.github.io/deepseek-harness/en/reference/subsystems/sandbox)
  (`SandboxMode`, `SandboxEnforcement`, `SandboxPolicy`, `ctx.sandboxPolicy.resolve()`),
  [`SAFETY.md`](https://github.com/deepseek-ai/deepseek-harness/blob/master/SAFETY.md), and
  [`README.md`](https://github.com/deepseek-ai/deepseek-harness/blob/master/README.md).
- **Failure-mode corpus** (all read 2026-09-15):
  - **[CVE-2026-82533](https://www.ox.security/blog/cve-2026-82533-deepseek-harness-ai-agent-sandbox-escape/)**,
    CWE-807, **CVSS 9.4**, disclosed by Nir Zadok and Moshe Siman Tov Bustan (OX Research) to
    VulnCheck 2026-08-24, fixed in `0.1.2-alpha.1` on 2026-08-27, published 2026-09-08. Verbatim from
    the disclosure: *"a sandboxed AI agent [could] disable its own confinement with a single shell
    command – on shipped defaults, with no network exposure and no credentials."*
  - **The mechanism, in the vendor's own vocabulary.** The reference doc states plainly that
    `SandboxMode` *"governs filesystem effects only … **Network and process visibility are outside
    this vocabulary**."* The sandbox denied file writes and left loopback open; the harness's control
    API on `127.0.0.1:3080` authenticated callers from the client-supplied `Host` header
    (`isTrustedApiRequest`) rather than the connection's peer address; and routine `bash` calls
    required no approval, because *"approval governed sandbox escalation requests, not routine
    command execution."* One `curl` from inside the sandbox set the session to `danger-full-access`
    with approval `never`.
  - **The attribution failure — the load-bearing one for this ADR.** From the OX write-up: *"the
    policy change is recorded as `source: {kind: 'user'}` — the harness could not distinguish the
    agent's own shell from the human at the keyboard."* No `approval/request` event appears anywhere
    in the session log.
  - **The lifecycle failure.** `ctx.sandboxPolicy.resolve()` is documented as: *"An approved explicit
    mode outranks the session's last `sandbox/mode` event, which outranks the deployment default."*
    The confinement of call *n+1* is resolved from a session event log that the confined process at
    call *n* can append to. **The policy store was inside the blast radius of the policy.**
  - **Known upstream, unresolved.** [Discussion #1516](https://github.com/deepseek-ai/deepseek-harness/discussions/1516)
    (2026-08-14, Ideas, one reply, no label, open): *"Loopback also needs its own rule. #250 shows why
    a sandboxed process reaching the Harness control plane is different from reaching a package
    registry."* Its acceptance criteria include *"`workspace-write` plus `deny` cannot reach public
    addresses or the Harness loopback control plane"* and *"Loopback requires a separate grant."*
    CSA records that the same escape was posted to the discussion board on **Aug 13 and 14**, ten days
    before the formal report — *"discoverable through routine use rather than requiring specialized
    exploit development."*
  - **[CSA AI Safety Initiative research note](https://labs.cloudsecurityalliance.org/research/csa-research-note-deepseek-harness-sandbox-escape-20260910-c/)**,
    2026-09-10, which generalises the shape: *"a sandbox that restricts one channel (the filesystem)
    while leaving another (loopback networking) implicitly trusted — recurs across the AI coding-agent
    ecosystem and is not fully resolved by patching one product."* Same note places it third in a
    2026 series with the [trust-handoff flaw](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-sandbox-escapes-20260722-c/)
    (2026-07-22) and [GuardFall](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-agent-shell-injection-2026070/)
    (2026-07-01).
  - **The vendor's own instruction to the user**, verbatim from `SAFETY.md`: *"Review plugins,
    configuration, and proposed commands before allowing them to run."* That sentence is a product
    specification for a standalone command reviewer, written by a harness vendor.
- **Failure mode found in Caro itself** (Context §3, method and script in the scope doc §5): replaying
  all **67** compiled patterns from `src/safety/patterns.rs` under a faithful re-implementation of
  `SafetyValidator::is_dangerous_in_context` (`src/safety/mod.rs:432`) against a **31-row**
  posture-mutation corpus returns **28 false negatives**. All seven control rows stay clean. The three
  hits are incidental, not posture-aware: `export PATH=` ("Modify PATH environment variable",
  `patterns.rs:231`), `chmod ±x` (`patterns.rs:297`), and `sudo systemctl stop` — none of them
  because the command targets a guard. **Zero of the seven control-plane rows flag. Zero of the nine
  agent-policy-file rows flag.** A `grep` of the whole tree for `.claude/settings`, `codex/config`,
  `bypassPermissions`, `danger-full-access` or `dangerously-skip` returns **nothing** in `src/` or
  `data/`.
- **Depends on**: nothing unlanded. `RiskLevel`, `SafetyLevel`, `ShellType`, `SuggestedRouting`,
  `SafetyValidator`, `clap` 4.5, `serde`, `serde_yaml` 0.9, `schemars` 0.8, `bincode` 1.3,
  `once_cell`, `assert_cmd` 2 all ship in 1.4.0. The `data/cve_rules/*.yaml` →
  `build.rs:compile_cve_ruleset` → bincode → `src/safety/cve_patterns.rs::include_bytes!` pipeline
  ships and is **reused verbatim**, as is `scripts/validate-cve-yaml.ts` as the template for the lint
  gate. No network, no filesystem read at analysis time, no daemon, no new crate, no new top-level
  module.
- **Relates to**:
  - **ADR-064** (`caro.egress.v1`) — the closest neighbour and **not** a superset. Its unit is an
    `EgressPair { source: SourceKind, sink: SinkKind }`; a finding requires a `SourceKind`
    (credential file, credential env var, private key, project file, stdin). The DeepSeek escalation
    carries **no source** — it is inbound to a control plane, not outbound with a secret — so
    ADR-064 returns `Clear` on it *by construction*, and its `SinkKind::NetworkHost` has no loopback
    concept at all (`HostDisposition` is `Allowed | Denied | Unlisted`). Boundary recorded in D13.
  - **ADR-069** (`caro.preflight.v1`) — audits a **directory** for what it will run on its own
    (the read side of ambient config). This ADR audits a **command** for what it will reconfigure
    (the write side). Same files, opposite direction. D14.
  - **ADR-070** (`caro.delegation.v1`) — owns *which argument becomes a program*. This ADR owns
    *which argument names a guard*. Shares the word splitter (D10) and inherits D4's
    required-catalog-identity discipline and the `Unresolved`/`Unparsed` separation.
  - **ADR-067** (`caro.decompose.v1`) — the structural container; `GuardSurface` is a classification
    over its spans, not a second decomposition.
  - **ADR-039** (sandbox-aware verdict tier) — owns *may this run in a sandbox at all*. This ADR does
    not mint a container locus and does not model isolation.
  - **ADR-059** (verb-namespace moratorium) — honoured; no new policy tier, no new routing variant.
  - **Hermes market scan 2026-09-09**, shift **K** (*"the dangerous command is not the dangerous
    string"*) and opportunity **3.10**. This ADR is the *command-side* half of 3.10; ADR-069 is the
    directory-side half.
- **Amends**: nothing. Additive.
- **Numbering note**: highest on disk is ADR-070 (2026-09-14). Next free number is **071**. Per
  [`.claude/rules/adr-numbering.md`](../../.claude/rules/adr-numbering.md), if a competing 071 lands
  first this document renumbers on merge.
- **Full scope document**: [`caro-scope-posture-mutation-2026-09-15.md`](../../caro-scope-posture-mutation-2026-09-15.md)

---

> **Provenance note (autonomous run).** The task template shipped `[FEATURE NAME]` unbound and ran
> with no user present. Target selection is argued in the scope document §0. **This is a scope, not
> an implementation**: no branch, no PR, no code committed
> ([`.claude/rules/git-workflow.md`](../../.claude/rules/git-workflow.md)).
>
> **Validation-discipline note.** This is an architecture scope responding to a disclosed defect
> class with an assigned CVE, a named disclosing party, a patched-version table and a vendor-authored
> statement of the gap. Under [`validation-discipline.md`](../../.claude/rules/validation-discipline.md)
> §"What this rule does NOT do", responses to evidence we already have are outside the five-gate
> regime; Gate 3 ("what breaks at 100 real users") is nonetheless answered explicitly in the scope
> document §6, and Gate 4 (devil's advocate) is owed by the implementation PR, not by this document.
> No PMF claim is made or implied.

---

## Context

### 1. The question no agent harness can answer about its own commands

Every guard in this product category answers *is this command destructive?* None of them answers
*does this command reconfigure the guard?* The two questions have different answers on the same
input, and the second one is strictly more urgent, because a command that weakens the guard makes
every subsequent answer to the first question worthless.

CVE-2026-82533 is that sentence with a CVSS score attached. The escaping command was a `curl` to a
loopback port. It deleted nothing, wrote nothing outside the workspace, contacted no external host,
carried no credential, and matched no destructive pattern in any product's table. It was *the last
command for which any of that mattered*.

### 2. Why the harness could not see it, in four documented pieces

1. **A vocabulary that does not cover the channel.** `SandboxMode` is `read-only |
   workspace-write | danger-full-access` and, per the vendor's own doc, *"governs filesystem effects
   only."* Networking is out of vocabulary — but the mode names (`danger-full-access`) and the word
   "sandbox" promise total confinement. The CVE lives exactly in the gap between the name and the
   vocabulary.
2. **An implicitly trusted channel inside the confinement.** bubblewrap was invoked with
   `--unshare-pid` and not `--unshare-net`; the Seatbelt profile was `(allow default)` with
   `(deny file-write*)` layered on. Loopback was reachable from inside. Discussion #1516 asked for
   loopback to be *"a separate grant"* on 2026-08-14 and is still open.
3. **A policy store the confined process could write.** Policy for call *n+1* resolves from the
   session's last `sandbox/mode` event. The escalation appended one.
4. **An approval gate mounted on the wrong event.** Approval governed *escalation requests*. The
   escalation did not arrive as an escalation request; it arrived as a `bash` call, which needed no
   approval. And the resulting log entry read `source: {kind: 'user'}` — the harness had no way to
   distinguish its own agent's shell from the person at the keyboard.

Items 1–3 are architecture. Item 4 is the one that generalises furthest: **a control plane that
accepts an identity claim from the same channel it is supposed to be guarding cannot audit itself.**

### 3. Caro is not in this lane, measured

Caro's shipped answer to *"does this command touch a guard?"* is: nothing. A 31-row corpus built from
the CVE's own vector and its four sibling classes — loopback control-plane calls, agent policy-file
writes, guard-executable and shell-resolution shadowing, OS confinement facilities, toolchain policy —
replayed against all 67 compiled patterns returns **28 false negatives and 3 incidental hits**, with
**0 false positives on 7 controls**. The three hits fire on `export PATH=`, `chmod ±x` and
`sudo systemctl stop`, none of which is a statement about a guard; `export PATH=/tmp/shim:$PATH`
scores "Modify PATH environment variable" at `Moderate` whether or not `/tmp/shim` contains a fake
`caro`, and a report that cannot tell those apart is not evidence.

The gap is not a missing pattern. It is a missing **noun**. Caro's model has `RiskLevel`,
`SafetyLevel`, `ShellType` and (in paper form) sites, spans, effects, egress pairs and delegated
arguments. It has no type whose instances are *the controls that govern execution*. Adding five
regexes for `setenforce` and `.claude/settings.json` would produce exactly the round-2 patch OX
described in the sibling Black Hat chain — capability keyed on a program name, immediately bypassed
by the next spelling.

### 4. What is actually available to a subprocess, and what is not

Caro sees one string. It does not see the session, the conversation, the caller, the model, or who
typed what. Every prior attempt in this category to answer the posture question has reached for
exactly the thing Caro does not have: caller attribution. DeepSeek reached for it, got
`source: {kind: 'user'}`, and shipped a 9.4.

The finding of this ADR is that **caller attribution is the wrong input**. The capability of a command
to reconfigure a guard is a property of its text, which is forgery-proof in the way an identity claim
is not: a `curl -X POST http://127.0.0.1:3080/api/session/config` is a control-plane mutation
regardless of who typed it, and a schema with no attribution field has nothing for an attacker to
populate. Caro's structural poverty — one string, no session, no state — is the exact shape this
problem wants.

---

## Decision

Ship **`caro posture <command>`**, a read-only verb emitting one `caro.posture.v1` JSON document and
one exit code, that answers: **does this command change the configuration of a control that would
otherwise govern the next command?**

The full type listings, the v1 surface catalog, the file-by-file change set, the exit-code table and
the integration-test matrix are in the scope document. The decisions this ADR is accountable for:

**D1 — The verdict is a total function of the command text, the compiled catalog and the shell
dialect. The schema has no field for caller identity.** There is nothing for an escalation to claim.
This is the direct answer to `source: {kind: 'user'}`, and it is a design property, not a check.

**D2 — No state. The catalog is compiled into the binary at build time and identified by content
hash in every report.** DeepSeek resolves policy per call from a session event log the confined
process can append to; Caro has no policy store an analyzed command can reach, because it has no
policy store. There is no initialization to skip and no cache to poison — the answer to the
template's lifecycle question is the removal of the lifecycle.

**D3 — `Authority::Loopback` is a distinguished schema variant that carries no permissive
semantics.** Loopback gets its own variant *precisely so that* a future contributor who wants to
suppress it has to delete a variant in a reviewed PR rather than add an `if` in a matcher. This is
the DeepSeek asymmetry — filesystem restricted, loopback implicitly trusted — made structurally
unrepresentable. #1516 asked upstream for the same thing in prose.

**D4 — `catalog: CatalogIdentity` is a required, non-defaulted field of every report**, carrying the
content hash, the entry count and the surfaces-known count. A clean report produced by an empty
catalog is distinguishable from a clean report produced by a populated one. This is ADR-070 D4's
rule, and it is independently the right one: DeepSeek's own `SandboxEnforcement = 'full' | 'partial'`
exists because *"enforcement is a reported fact"* — the vendor got this part right and Caro copies it.

**D5 — Five verdicts, and `Observes` is not `Inert`.** Reading a guard surface (`cat
.claude/settings.json`, a `GET` to a loopback port) is named, at exit 0, rather than collapsed into
"nothing here". The whole CVE is the distance between "not modelled" and "safe"; a schema that cannot
say "I saw a guard surface and it was only read" cannot report that distance.

**D6 — A mutation whose surface is Caro's own enforcement path sets `reflexive: true` and floors at
`RiskLevel::Critical`.** `alias caro=true`, `chmod -x $(command -v caro)`, a shim earlier on `PATH`,
`CARO_*` assignments, edits to `patterns.toml`. Every vendor guard in the corpus is outside its own
model; this is the one statement in the category that only a standalone, vendor-neutral tool can
make about itself, and it is the reason this verb is worth more than five regexes.

**D7 — `Unresolved` and `Unparsed` are separate verdicts with separate exit codes, and neither can
coexist with a clean result.** `curl -X POST "$CONTROL_URL"` is not clean. ADR-070's lesson,
restated because it is the lesson GuardFall found ten of eleven agents failing.

**D8 — v1 is read-only: `SafetyValidator` does not consult the catalog and no existing verdict
changes.** Making plain `caro "…"` report posture mutations changes answers on inputs users already
run — the v2.1.259-shipped / v2.1.260-reverted trap ADR-067 §1 documents. That is PR 3, with its own
false-positive corpus.

**D9 — Seven files, no new module.** `src/safety/posture.rs`, `src/safety/posture_catalog.rs`,
`data/posture/*.yaml`, one function in `build.rs`, one `Commands` arm in `src/main.rs`, one lint
script, one integration test file. The delivery mechanism is the `cve-rules` pipeline, reused.

**D10 — The word splitter is shared with ADR-070, not duplicated.** Whichever lands first ships it;
the second imports it. Recorded here so the merge is a deletion, not a reconciliation.

**D11 — No policy vocabulary is minted.** v1 emits facts. "Deny all reflexive mutations" is policy
and is closed by the ADR-059 moratorium until `caro.assessment.v1` merges. `SuggestedRouting` is
reused unchanged and needs the same two-line `JsonSchema` + `Ord` derive fix
(`src/models/mod.rs:187`) that ADR-058 §D-prereq and ADR-069 already require; whichever lands first
pays, the others cite.

**D12 — `direction` is three-valued: `Weakens | Strengthens | Unknown`.** `npm config set
ignore-scripts true` strengthens. A schema that can only say "bad" mislabels hardening as a finding,
and a tool that flags hardening gets turned off.

**D13 — ADR-064 is not extended to cover this, and this does not absorb ADR-064.** Egress is
source→sink: it answers *is something leaving with something*. Posture is sink-only and inbound: it
answers *is something arriving at a control*. Merging them would force a `SourceKind::None`, which
turns every unpaired sink into a finding and detonates the false-positive budget both ADRs depend on.
Two verbs, one boundary sentence, recorded.

**D14 — ADR-069 keeps the directory; ADR-071 keeps the command.** `core.fsmonitor` is the worked
example: 069 owns *"it is set in `.git/config`"*, ADR-070 owns *"it appears as a `git -c` flag"*,
071 owns *"this command sets it"* (`git config --global core.fsmonitor /tmp/p.sh` is a
`ToolchainPolicy` mutation). Three ADRs, three distinct facts, no overlap in what each emits.

---

## Consequences

**Good.**

- Caro acquires a noun for the controls that govern execution, and with it the ability to answer the
  one question the 2026 incident series keeps asking. It closes the command-side half of Hermes
  opportunity 3.10 / shift K.
- The answer is deterministic, model-free, offline, and works as a pure subprocess call inside an
  air-gapped CI runner — which is where this attack class has repeatedly landed.
- `reflexive: true` is a claim no vendor-internal guard can make, and it is directly saleable: a
  buyer asking "what stops the agent from turning your tool off?" gets a schema field, not a
  paragraph.
- The community layer has a second corpus with the same contribution loop as `data/cve_rules/` — one
  YAML file, one linter, build-time compile, `test_cases` that become eval rows. Surface drift
  becomes a PR instead of a vendor release.
- It is small. Seven files, no new module, no new dependency, no new risk tier.

**Bad, and stated as such.**

- **The catalog is a closed set of literal surfaces, and literal surfaces drift.** A product renames
  its settings file and the row goes silently `Inert`. D4's `surfaces_known` count and the
  `--catalog-only` self-check make the drift *visible*; they do not prevent it. This is the honest
  Gate-3 answer and it is elaborated in the scope §6.
- **Coverage is asserted, not measured against an external suite.** Same critique Hermes 3.11 makes
  of "52+ patterns, zero false positives". The 31-row corpus in §5 of the scope is Caro's own; it
  should be mapped onto a public frame (OWASP Agentic Top 10) before any recall number is published.
- **No expansion.** `curl "$URL"` is `Unresolved`, which is correct and unsatisfying. A large share
  of real agent-emitted commands will land there until v2.
- **A fifth exit-code dialect.** ADR-065/066/067/068 use `2` for a finding, colliding with `clap`'s
  usage exit; ADR-070 proposed the reserved range `3..=15` and did not do the consolidation work.
  This ADR adopts 070's proposal, which makes the consolidation PR more overdue, not less.
- **It measures a class nobody has yet reported against Caro users.** The argument is that the class
  is disclosed, CVE'd, vendor-acknowledged and structurally invisible to Caro — not that it has
  happened in the field.

**Neutral.**

- v1 ships facts and no policy, so a host that wants "block reflexive mutations" writes three lines
  of shell against the exit code. That is the intended shape until `caro.assessment.v1` lands.
- The verb never opens a socket, resolves a host, or probes a port. `127.0.0.1:3080` is classified
  from the text whether or not anything is listening — deliberately, because probing would make the
  verb stateful, network-touching and environment-dependent, which is the property that broke the
  thing being researched.

---

## Alternatives considered

**A1 — Add posture regexes to `DANGEROUS_PATTERNS` and ship nothing else.** Rejected. This is
literally the round-2 patch from the sibling Black Hat chain: capability keyed on a program name,
bypassed by the next spelling. It also cannot express `reflexive`, cannot distinguish read from
write, cannot report its own coverage, and changes existing verdicts on day one with no
false-positive corpus (D8). CSA's GuardFall note states the general form of the objection:
*"the deny-list approach is not fixable through iterative list expansion; it requires structural
change at the evaluation layer."* A catalog of typed surfaces is that structural change for this
question; five more regexes are the iterative expansion.

**A2 — Extend ADR-064's `EgressPair` with a `SourceKind::None`.** Rejected, D13. It converts every
unpaired sink into a finding and destroys the precision both verbs need.

**A3 — Probe the host: check whether `127.0.0.1:3080` is listening, read `.claude/settings.json`,
resolve `$PATH`.** Rejected, and this is the load-bearing rejection. It would make the verdict depend
on state the analyzed command can change, which is the exact defect (Context §2.3). A pure function
of the text is weaker in what it can assert and strictly stronger in what it can be made to assert
falsely.

**A4 — Model caller attribution (agent vs human) and gate on it.** Rejected. It is not available to
a subprocess, and the one system in the corpus that tried to synthesize it emitted
`source: {kind: 'user'}` for an agent-issued escalation. Recorded because a reviewer will ask.

**A5 — Wait for ADR-069 and make posture a `preflight` mode.** Rejected, D14. 069's input is a
directory before a session; this verb's input is one command mid-session, on the hook path, under
ADR-062's per-command latency budget. Different input, different cadence, different consumer.

**A6 — Vendor a catalog from an existing corpus (GTFOBins, LOLBAS, Atomic Red Team).** Rejected on
shape, as ADR-070 §1.3 rejected GTFOBins for its own purpose: those corpora catalogue *offensive
capability given you can already run the binary*, keyed on program names, with free-text recipes
rather than decidable predicates, and none of them has a concept of "the guard currently governing
this process". The catalog this verb needs — agent policy paths, harness control ports, confinement
facilities, and the tool's own name — does not exist in any public corpus. That absence is the
opportunity.

**A7 — Do nothing; sandboxes are the industry's answer to this.** Rejected, and the researched
feature is the argument: this *was* a sandbox, on shipped defaults, correctly enforcing its
documented filesystem contract, and it was disabled by one unremarkable shell command. The vendor's
own `SAFETY.md` instructs users to *"review … proposed commands before allowing them to run."*
Something has to do the reviewing.

---

## References

- OX Security — [CVE-2026-82533: DeepSeek Harness Vulnerability Lets AI Agents Escape Their Own Sandbox](https://www.ox.security/blog/cve-2026-82533-deepseek-harness-ai-agent-sandbox-escape/), 2026-09-08
- CSA AI Safety Initiative — [DeepSeek Harness Sandbox Escape and Agent Containment](https://labs.cloudsecurityalliance.org/research/csa-research-note-deepseek-harness-sandbox-escape-20260910-c/), 2026-09-10
- CSA AI Safety Initiative — [AI Coding Agent Sandbox Escapes: The Trust Handoff Flaw](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-sandbox-escapes-20260722-c/), 2026-07-22
- CSA AI Safety Initiative — [GuardFall: Shell Injection Bypass Defeats AI Coding Agent Guardrails](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-agent-shell-injection-2026070/), 2026-07-01
- DeepSeek Harness — [Process Sandbox reference](https://deepseek-harness.github.io/deepseek-harness/en/reference/subsystems/sandbox) · [SAFETY.md](https://github.com/deepseek-ai/deepseek-harness/blob/master/SAFETY.md) · [repository](https://github.com/deepseek-ai/deepseek-harness)
- DeepSeek Harness — [Discussion #1516, "Make network access a separate sandbox permission"](https://github.com/deepseek-ai/deepseek-harness/discussions/1516), 2026-08-14
- The Hacker News — [DeepSeek Harness Flaw Let AI Agents Disable Their Own File Sandbox Without Approval](https://thehackernews.com/2026/09/deepseek-harness-flaw-let-ai-agents.html), 2026-09-09 *(cited via CSA reference [2]; not independently retrieved for this ADR — see scope §8)*
- Hermes market scan, [2026-09-09](../../.hermes/digests/2026-09-09-agent-market-scan.md) — shift K, opportunity 3.10

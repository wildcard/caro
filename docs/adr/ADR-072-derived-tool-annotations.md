# ADR-072: `caro.annotate.v1` — Compute the Four MCP Hints From the Command, Per Call

- **Status**: **Proposed — Gate 1 unmet, Gate 4 pending.** See *Validation discipline* below.
  Buildable on the tree that ships in 1.4.0; no unlanded ADR is a prerequisite.
- **Date**: 2026-09-16
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **MCP `ToolAnnotations`** — the four boolean hints (`readOnlyHint`,
  `destructiveHint`, `idempotentHint`, `openWorldHint`) that shipped in spec revision `2025-03-26`,
  read together with the client behaviour built on them. Schema read 2026-09-16 from the
  [`ToolAnnotations` interface](https://modelcontextprotocol.io/specification/2025-11-25/schema#toolannotations);
  design intent and self-assessed limits from the maintainers' retrospective
  [*Tool Annotations as Risk Vocabulary: What Hints Can and Can't Do*](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/)
  (2026-03-16; Ola Hungerford, Sam Morrow (GitHub), Luca Chang (AWS)); consumer semantics from
  OpenAI's [Codex CLI configuration reference](https://developers.openai.com/codex/config-reference)
  and [agent approvals & security](https://developers.openai.com/codex/agent-approvals-security) via
  [Vaughan, 2026-04-12 (page updated 2026-09-16)](https://codex.danielvaughan.com/2026/04/12/mcp-tool-annotations-risk-vocabulary-codex-cli/).
- **Failure-mode corpus** (all read 2026-09-16):
  - **The specification's own admission**: *"An untrusted server can lie… A server can claim
    `readOnlyHint: true` and delete your files anyway. This is why the spec says clients **MUST**
    treat annotations from untrusted servers as untrusted."*
  - **Codex CLI `apps.<name>.destructive_enabled = false`** — *"a hard block, not a prompt — the tool
    is simply unavailable"* — whose sole input is the server's self-declared `destructiveHint`.
  - **Codex CLI "Allow and remember"** ([PR #10584](https://github.com/openai/codex/pull/10584),
    merged Feb 2026) — session approval cached on `(server, connector_id, tool_name)`, **a key with
    no argument in it**.
  - **Gemini CLI, [CVE-2026-12537](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/)**,
    CVSS 10.0 (Black Hat USA 2026, Elad Meged / Novee) — *"its tool allowlist was checked at
    registration but never enforced at execution."*
  - **Claude Code, CVE-2026-54316** (same disclosure) — a read-only validator *"bypassed by stripping
    quotes"*, carrying *"hardcoded read-only exemptions."*
  - **[LayerX — Claude Desktop extensions RCE](https://layerxsecurity.com/blog/claude-desktop-extensions-rce/)**,
    cited by the MCP maintainers as the demonstrated lethal-trifecta chain, in which a **local
    code-execution tool** is the linchpin.
  - **Five open SEPs, none merged** —
    [#1913](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1913),
    [#1984](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1984),
    [#1561](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1561),
    [#1560](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1560),
    [#1487](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1487) — each of which
    adds another boolean the server declares about itself.
- **Full scope document**: [`caro-scope-tool-annotations-2026-09-16.md`](../../caro-scope-tool-annotations-2026-09-16.md)

---

## Context

### 1. The question the protocol asks a server to answer about itself

MCP gives a host a four-word risk vocabulary so it can route tool calls without prompting for
everything. Each hint is tied to a concrete client behaviour, and the maintainers reject proposed
additions that are not — a discipline worth borrowing:

| Hint | Default | Client behaviour it enables |
|---|---|---|
| `readOnlyHint` | `false` | skip the confirmation dialog |
| `destructiveHint` | `true` | warn — or, in Codex CLI, hard-block |
| `idempotentHint` | `false` | safe to retry on failure |
| `openWorldHint` | `true` | scrutinise output as untrusted; flag a trust-boundary cross |

The defaults are deliberately pessimistic: an unannotated tool is assumed non-read-only, destructive,
non-idempotent and open-world. That direction is correct and this ADR preserves it.

### 2. Why the answer cannot be trusted, in the standard body's own words

Annotations are hints. The spec says clients **MUST** treat them as untrusted from untrusted servers,
and the maintainers state plainly that *"they aren't enforcement"* and that a server *"can claim
`readOnlyHint: true` and delete your files anyway."*

This is not an implementation bug awaiting a patch. It is the documented, intended boundary of a
declaration-based scheme, and every one of the five open SEPs proposes to widen the vocabulary
without changing the trust model.

### 3. Why the gap is load-bearing for exactly one tool family

For `delete_user` the approximation is fine: the tool is destructive whatever its arguments. It
collapses when the argument **is a program**:

| Declared once, on the tool | Actually true, per call |
|---|---|
| `run_command` → one `destructiveHint` | `ls -la` is additive-free; `rm -rf ~` is not |
| `run_command` → one `openWorldHint` | `wc -l f` is closed-world; `curl … \| sh` is not |
| `run_command` → one `idempotentHint` | `mkdir -p x` is idempotent; `echo x >> f` is not |

The server author's three options are all bad: declare permissively and lose the gate on the
dangerous calls; declare pessimistically and have `open_world_enabled = false` hard-block `ls`; or
omit and inherit the pessimistic default, which is the second option renamed. The MCP blog records
both consequences — *"many servers ship without them"*, and *"no MCP client lets users filter tools
by annotation values."*

The second-order failure is sharper. The two strongest consumer behaviours — Codex CLI's **hard
block** and its **remembered approval** — are keyed on identifiers that contain no argument. A
careless or malicious `destructiveHint: false` on a shell tool is not warned about or blocked; it is
*auto-approved*, and the protocol's answer is a sentence instructing the client not to believe the
thing it is acting on.

### 4. What is actually available to a subprocess, and what is not

Caro has no registration step, no tool identity, no session and no cache. It is handed a command and
answers about that command. For this problem that is not a limitation — it is the one input the
protocol structurally cannot have at annotation time.

What is **not** available, and must not be manufactured: the filesystem (`$PATH` resolution, `stat`
on a target), the network, and any notion of *which tool* or *which server* produced the string.
ADR-071 recorded live probing as permanently rejected because a verdict that depends on state the
analysed command can change is the defect the whole series exists to avoid. That decision is
reaffirmed here, not revisited.

---

## Decision

Add a verb, `caro annotate`, that emits a `caro.annotate.v1` document: the four MCP fields **derived
from the command**, per invocation, with the evidence that forced each one, and — when the caller
supplies the server's own declaration — a per-field agreement report.

```
caro annotate [--json] [--declared <ToolAnnotations-json>] [--lexicon-only] <command>
```

### D1 — Three-valued, because `boolean?` conflates two different things

Every field is `Tri { True, False, Undetermined }`. MCP's `boolean?` makes "the server said false"
and "the server said nothing" identical at the point a client acts. Caro must not inherit that.

### D2 — `Provenance` has two variants, and a third must never be added

```rust
pub enum Provenance { Derived, Pessimistic }
```

There is no `Declared`. A declaration supplied via `--declared` is echoed in `declared_input` and
compared in `agreements`; it can reach no other field. This is the same reasoning ADR-071 applied to
its refusal of an attribution field: the schema must make the laundering un-expressible, not merely
discouraged. (**Invariant A1**, test T13.)

### D3 — Asymmetric proof

A permissive value (`read_only: True`, `destructive: False`, `open_world: False`) requires **every**
execution site resolved and classified. A single unresolved site anywhere forces `Undetermined` for
that field. Harmful values may be derived from one site.

Proving harm needs one witness; proving harmlessness needs completeness. CVE-2026-54316 is what the
other rule looks like in production — a read-only determination defeated by quoting, with hardcoded
exemptions patching over the gap. (**Invariant A2**, test T8.)

### D4 — Fail closed to the spec's default, and say so out loud

`Undetermined` projects to the MCP default for that field and is emitted as `Some(default)`, never
`None`, so a client that has not implemented default-resolution cannot get it wrong. The verdict is
`Undetermined` and the exit code is `4` — **not** `0`. A pessimistic answer is not a passing answer.
(**Invariant A3**.)

### D5 — No tool identity in the document

The report carries `command_sha256` and nothing else that could identify a tool, server or
connector. A consumer cannot reconstruct the `(server, connector_id, tool_name)` cache key from it.
This is deliberate: that key is the defect. (**Invariant A5**.)

### D6 — A local splitter, with a named deletion date

Site enumeration uses a local splitter over the same operator set ADR-067 specifies. When ADR-067
lands, the local splitter is deleted in a PR that changes no output, and a conformance test asserts
that. Depending on an unlanded ADR would violate the rule ADR-070 §D14 and ADR-071 established for
this series.

### D7 — The four MCP fields and no fifth

`caro.annotate.v1` emits exactly `read_only`, `destructive`, `idempotent`, `open_world`. Caro does
not invent a fifth annotation, and does not implement the unmerged SEP-1075 vocabulary
(`reads_private_data`, `sees_untrusted_content`, `can_exfiltrate`). Binding v1 to a draft is how a
contract acquires a migration before it acquires a user.

### D8 — The projection is a first-class output

`mcp_projection` is a spec-shaped `{ readOnlyHint?, destructiveHint?, idempotentHint?, openWorldHint? }`
object produced by a total, lossy function from `DerivedAnnotations`. It exists so an ADR-043 gateway
can substitute derived annotations for declared ones on the wire at zero cost to the downstream
client. Doing that substitution is **not** in v1.

### D9 — Extension pressure goes to a namespace, not to the core object

Anything beyond the four fields lands under a namespaced `_meta`-shaped key, following the MCP
maintainers' own guidance (*"ship a namespaced field, see how it holds up in production, and come
back with a proposal backed by actual usage"*). The core object stays at four.

### D10 — Verdict is the contract; the exit code is a shortcut

Stdout is exactly one newline-terminated `caro.annotate.v1` document. `exit_code()` is a total
function of `verdict`, exposed as a method so it is testable without spawning a process.

| Exit | Verdict |
|---|---|
| `0` | `Consistent` |
| `3` | `Contradiction` — a declared hint claims permission the command does not earn |
| `4` | `Undetermined` |
| `5` | `Unparsed` |
| `1` | internal error, no payload |
| `2` | **reserved for `clap` usage errors — never emitted** |

Inside the `3..=15` range ADR-070 §D11 proposed. This verb adds no sixth collision on `2` and does
**not** do the consolidation work owed by ADR-065/066/067/068.

### D11 — JSON only in v1

No human renderer. The consumer is a harness, not a person; a second format is a second contract to
keep in sync. Adding one later is additive.

### D12 — Lexicon as data, compiled in, with its own test cases

`data/annotations/*.yaml`, one file per program family, compiled by `build.rs` (ADR-070's pattern).
Each entry carries `test_cases` that `build.rs` emits into the eval suite, so a drifted or wrong
entry fails a test rather than silently degrading a verdict. Each entry has a stable, suppressible
`entry_id`.

### D13 — `idempotent` may be `Undetermined` while its siblings are `Derived`

It is the hardest field to derive and the only one whose consumer behaviour is a **re-execution**. An
entry that cannot justify an idempotence value omits it; the field falls to the pessimistic default
(`false` — do not retry), which is the direction that cannot cause a duplicate execution.

### D14 — Two PRs, and the risky ones are not in v1

| PR | Contents | Behaviour change |
|---|---|---|
| 1 | types, lexicon, `build.rs`, data, unit tests, `pub(crate)` widening in `side_effects.rs` | **none** — no CLI surface, no verdict change |
| 2 | `caro annotate` verb + contract tests T1–T18 | additive, read-only |
| 3 | ADR-043 gateway substitutes `mcp_projection` on the wire | **out of scope for v1** |
| 4 | `SafetyValidator` consults the lexicon | **out of scope for v1** |

PR 4 is where every false positive lives. Shipping it inside v1 would recreate the
v2.1.259-shipped / v2.1.260-reverted whiplash ADR-067 §1 documents.

---

## Consequences

### Positive

- **The one input the protocol cannot have.** Per-call derivation is structurally impossible inside
  MCP and trivial for a subprocess holding the argument.
- **A lying-server detector, as an exit code.** `caro annotate --declared '{"destructiveHint":false}'
  "rm -rf ~"` → exit 3. That is the missing input to a `destructive_enabled = false` hard block that
  today trusts the declaration it is defending against.
- **Deterministic and offline.** Claude Code's answer to the same problem is a secondary classifier:
  per-call inference cost, a network dependency, a nondeterministic verdict. This is a pure function
  with a test vector, which is the *"be the stage the monitors call"* position
  ([Hermes 2026-09-09 §4](../../.hermes/digests/2026-09-09-agent-market-scan.md)).
- **Zero-cost adoption path.** `mcp_projection` is spec-shaped, so a gateway can substitute without
  the downstream client learning new vocabulary.
- **Mostly assembly.** `has_network`, `has_destructive_fs`, `has_systemwide_write`
  (`src/caroml/validators/side_effects.rs`), the 52-pattern corpus (`src/safety/patterns.rs`), and
  the `Serialize + JsonSchema` derive conventions (`src/models/mod.rs:149`) already exist.

### Negative

- **Coverage is the whole product and it is bounded by a hand-written lexicon.** The long tail of
  program names is unbounded; as coverage falls the verb degenerates toward a constant pessimistic
  answer, which is indistinguishable from a correct one to a consumer that reads only
  `mcp_projection`. Mitigated by making `coverage` a field in every report and the aggregate
  `Undetermined` rate a published release metric — not by monitoring.
- **Flag-predicate drift** across GNU/BSD (`rm --recursive` vs `-rf`; `cp`, `sed`, `date`). The schema
  reserves a `platforms:` key in v1; an entry that needs it and omits it is a lint error.
- **Idempotence will be wrong somewhere.** `git push`, `kubectl apply`, `docker run` without `--rm`.
  D13 is the mitigation and it is a retreat, not a solution.
- **A sixteenth proposed verb against a binary that ships approximately none of them.** The
  maintainers' discipline this ADR praises in *Context §1* — reject any addition that changes no
  concrete consumer behaviour — applies to Caro's own ADR queue at least as sharply. Recorded as a
  cost, not waved away.

### Validation discipline (`.claude/rules/validation-discipline.md`)

This is a new user-facing capability, so all five gates apply.

- **Gate 1 (20 transcripts): NOT MET.** Zero first-hand transcripts exist.
- **Gate 2 (no surveys): met** — none cited.
- **Gate 3 (demoware trap): met** — scope doc §6.
- **Gate 4 (devil's advocate): PENDING.** Not reviewed by the `devils-advocate` agent.
- **Gate 5 (PMF claim): met by making none.**

The rule exempts responses to evidence already in hand — *"broken things, vulnerabilities,
friction."* **This ADR claims that exemption and the claim is contestable.** The honest form: the MCP
specification is not broken, it documents its boundary precisely; what is broken is a shipped host
converting a self-declared boolean into a hard block and a remembered approval, documented in the
vendor's own configuration reference. A reviewer who finds that reading too generous should return
this to Gate 1. **PR 1 must not open until a human resolves scope-doc §8.1.**

---

## Alternatives considered

**1. Wait for the SEPs.** Five are open; none is merged; every one adds another self-declared
boolean. SEP-1075's enforcement rule (*"never allow all three in a single tainted execution path"*)
takes three self-declared inputs. Waiting means waiting for a trust model that is not being changed.
*Rejected.*

**2. Derive the annotations inside an MCP server Caro ships.** Would let Caro annotate its own
`run_command` accurately and nothing else. The value is in annotating *other people's* shell tools,
which requires a subprocess anyone can call. *Rejected — wrong distribution shape.*

**3. Substitute annotations on the wire in the ADR-043 gateway, in v1.** The highest-value form and
the highest-risk one: a proxy silently rewriting a server's declarations is an ecosystem argument
attached to a behaviour change. Deferred to PR 3 with the seam (`mcp_projection`) built in v1.
*Deferred, not rejected.*

**4. Emit a single risk level instead of four booleans.** Smaller, and already nearly free via
`SafetyDecision` (`src/safety/mod.rs:189`). But the four MCP fields are not a function of a risk
level — `curl https://example.com` is `RiskLevel::Safe` and `open_world: True`; `rm -f /tmp/x` is
`Moderate` and `destructive: True`. Collapsing makes the projection unusable to an MCP client.
*Rejected.*

**5. Two-valued fields with a separate `unknown: [field]` list.** Flatter wire format, same
information. Rejected because it puts the two states a consumer most often conflates into two
different places in the document, which is the ergonomic version of the `boolean?` defect. *Rejected.*

**6. Resolve `$PATH` and `stat` the operands to raise coverage.** Would materially reduce the
`Undetermined` rate. Makes the verdict depend on state the analysed command can change, and makes
Caro a process that opens files named by untrusted input — the shape of CVE-2025-41390 that ADR-069
documents. *Permanently rejected*, consistent with ADR-070 §D10 and ADR-071 §7.3.

**7. Ship only three fields, holding `idempotent` permanently `Undetermined`.** A defensible smaller
v1; the pessimistic default (*do not retry*) is what a cautious host wants anyway. Kept open as
scope-doc §8.2 rather than decided here.

---

## References

- Scope document: [`caro-scope-tool-annotations-2026-09-16.md`](../../caro-scope-tool-annotations-2026-09-16.md)
- [MCP — *Tool Annotations as Risk Vocabulary: What Hints Can and Can't Do*](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/) (2026-03-16)
- [MCP specification — `ToolAnnotations`](https://modelcontextprotocol.io/specification/2025-11-25/schema#toolannotations)
- [Codex CLI configuration reference](https://developers.openai.com/codex/config-reference) · [agent approvals & security](https://developers.openai.com/codex/agent-approvals-security)
- [Vaughan — *How Codex CLI Uses Hints to Drive Approval Decisions*](https://codex.danielvaughan.com/2026/04/12/mcp-tool-annotations-risk-vocabulary-codex-cli/) · [*The Writes Mode*](https://codex.danielvaughan.com/2026/07/24/codex-cli-writes-app-approval-mode-mcp-tool-annotations-read-only-hint-approval-flow/)
- [Novee Security — Black Hat USA 2026 disclosure](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/) (CVE-2026-12537, CVE-2026-54316)
- [Simon Willison — *The lethal trifecta for AI agents*](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/)
- In-tree: [ADR-043](./ADR-043-mcp-safety-gateway-proxy.md), [ADR-063](./ADR-063-destructive-command-benchmark-v1.md), [ADR-064](./ADR-064-sandbox-egress-conjunction-gate.md), [ADR-067](./ADR-067-execution-site-decomposition.md), [ADR-069](./ADR-069-ambient-configuration-preflight.md), [ADR-070](./ADR-070-delegated-execution-lexicon.md), [ADR-071](./ADR-071-guard-posture-mutation-classification.md)
- Prior mention of this vocabulary, classified and set aside: `caro-scope-assessment-contract-2026-08-25.md:202-211`

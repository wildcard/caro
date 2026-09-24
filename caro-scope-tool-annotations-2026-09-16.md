# Scope: `caro.annotate.v1` — Compute the Four MCP Hints From the Command, Per Call

> **Provenance.** Produced by the `caro-research--scoping-process` scheduled task,
> autonomous run, **2026-09-16**, no user present. The template's `[FEATURE NAME]`
> shipped unbound; target-selection rationale is §0.
>
> **This is a scope, not an implementation.** No branch, no PR, no code committed
> (`.claude/rules/git-workflow.md`). Companion ADR:
> [`docs/adr/ADR-072-derived-tool-annotations.md`](docs/adr/ADR-072-derived-tool-annotations.md).

---

## 0. Target selection

Four candidates surfaced in the 2026-09-09 → 2026-09-16 window. The chosen one is the fourth.

| Candidate | Why not |
|---|---|
| **CVE-2026-35603** (Cymulate) — `C:\ProgramData\ClaudeCode\managed-settings.json` loaded without checking directory ACLs; Claude Code, Cursor, Codex CLI, Gemini CLI | Real and well-documented, but it is ADR-069's shape with the trust arrow reversed, and the Caro-side answer requires **stat'ing the filesystem** — which ADR-071 §7.3 recorded as *permanently* rejected, not deferred. Scoping it would mean reopening a closed decision on a Windows-only ACL model. |
| **Stop Rogue AI Act** (Gottheimer / Lawler, 2026-09-09) — directs NIST to develop agent-governance standards | Policy, not a feature. Relevant to `COMPANY.md`, not to a scope document. |
| **MS Agent Framework CodeAct** — agents emitting Python that calls `subprocess.run` rather than emitting tool calls | Genuinely interesting and genuinely uncovered, but the unit of analysis is a *program*, not a command, and ADR-067 §7 already names nested-interpreter descent as its own ADR. Scoping it now would produce a Python-AST dependency inside a POSIX-shell product. |
| **MCP `ToolAnnotations` — the four hints, and the clients that hard-block on them** | **Chosen.** |

**Why this one.** The August 25 assessment scope
([`caro-scope-assessment-contract-2026-08-25.md:202-211`](caro-scope-assessment-contract-2026-08-25.md))
looked directly at `readOnlyHint` / `destructiveHint` / `idempotentHint` / `openWorldHint`, quoted
the spec's own disclaimer, classified them as *"a fourth vocabulary but explicitly disclaimed,"* and
moved on. That was the right call for a **vocabulary comparison**. It is the wrong place to leave it
now, because in the twenty-two days since, the thing the disclaimer was supposed to prevent has
shipped anyway: Codex CLI's `apps.<name>.destructive_enabled = false` is a **hard block** — *"not a
prompt — the tool is simply unavailable"* — and its only input is a boolean the MCP server declared
about itself.

Three properties made it the target rather than the fourth interesting CVE of the month:

1. **The gap is structural, published by the standard body itself, and unfixable inside the
   standard.** The MCP blog's own March post states it: *"An untrusted server can lie… A server can
   claim `readOnlyHint: true` and delete your files anyway."* Every proposed fix in the five open
   SEPs is another self-declared boolean. A declaration cannot be made non-self-declared by adding
   declarations.
2. **For exactly one tool family, the declaration is unnecessary, because the answer is computable.**
   When the tool is `run_command` / `execute_bash` / `shell`, the risk is not a property of the tool —
   it is a property of the *string in the argument*. `run_command("ls")` and `run_command("rm -rf /")`
   are the same tool with the same annotations. That family is Caro's lane, exclusively and by
   construction, and it is the family the MCP blog itself calls the linchpin: *"any agent with
   unrestrained shell access sits one injected instruction away from exfiltration."*
3. **The failure mode is one Caro cannot commit.** The defect is *static per-tool declaration
   standing in for dynamic per-call fact*. Caro has no registration step, no tool identity, and no
   cache — it is a subprocess that is handed a command and answers about that command. What looks
   like structural poverty is, for this exact problem, the correct architecture, **provided the
   schema is designed so a declaration can never be laundered through it into something that reads
   as derived.**

Selection rationale in one line: **ADR-071 asked whether a command reconfigures the guard; this asks
whether the label the guard is reading was ever true of the command in the first place.**

---

## 1. Phase 1 — Feature research

### 1.1 The feature: `ToolAnnotations` in the Model Context Protocol

Shipped in spec revision `2025-03-26`; current shape read 2026-09-16 from the
[`ToolAnnotations` schema](https://modelcontextprotocol.io/specification/2025-11-25/schema#toolannotations)
and the maintainers' retrospective,
[*Tool Annotations as Risk Vocabulary: What Hints Can and Can't Do*](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/)
(2026-03-16; Ola Hungerford, Sam Morrow (GitHub), Luca Chang (AWS)).

```typescript
interface ToolAnnotations {
  title?: string;
  readOnlyHint?: boolean;    // default: false
  destructiveHint?: boolean; // default: true
  idempotentHint?: boolean;  // default: false
  openWorldHint?: boolean;   // default: true
}
```

**Problem it solves, and for whom.** A host that gates every tool call drowns the developer in
prompts; a host that gates none destroys the repository. The four hints give the host a coarse risk
vocabulary so it can route: skip the dialog for reads, gate the writes. The user is a developer
running an agent with a mixed set of MCP servers attached.

**Architecture — data flow and separation of concerns.** Worth reading closely, because two of the
three design choices are right:

- **The annotation rides on the tool *definition*,** returned once from `tools/list`. It is
  registration-time metadata, not call-time metadata.
- **Every field is optional, and the absent value is the pessimistic one.** A tool with no
  annotations is *"non-read-only, potentially destructive, non-idempotent, and open-world."* Server
  authors must opt **in** to permissive treatment. This is the right default direction and §2.1
  replicates it.
- **Each hint is justified by naming the client behaviour it changes.** The maintainers' own table:
  `readOnlyHint: true` → skip the confirmation dialog; `destructiveHint: true` → warn before
  executing; `idempotentHint: true` → safe to retry on failure; `openWorldHint: true` → scrutinise
  output for untrusted content. A proposed annotation that changes no client behaviour is rejected.
  This is a discipline Caro's own verb catalogue should borrow and §2.1 says so.

**The structured output contract.** There is no exit code and no envelope — the annotations are four
optional booleans on a JSON object inside a `tools/list` response. The contract that matters is the
**default-resolution** rule, which is client-side and unenforced: absent ⇒ pessimistic.

**Session/context lifecycle — how it avoids redundant initialization.** By caching hard. Annotations
are fetched once per server at `tools/list` and held for the session; `notifications/tools/list_changed`
is the only invalidation. This is exactly the property §1.3 identifies as the defect: the cache key
is the tool, and the risk is in the argument.

### 1.2 The consumers — where a hint became an enforcement decision

The spec's disclaimer is unambiguous: *"clients **MUST** consider tool annotations to be untrusted
unless they come from a trusted server."* Read together with the maintainers' *"They aren't
enforcement"*, the intended posture is: hints drive UX, sandboxes drive safety.

What shipped is stronger than that. From
[*How Codex CLI Uses Hints to Drive Approval Decisions*](https://codex.danielvaughan.com/2026/04/12/mcp-tool-annotations-risk-vocabulary-codex-cli/)
(2026-04-12, **page updated 2026-09-16**, sourced to the vendor's own
[configuration reference](https://developers.openai.com/codex/config-reference) and
[agent approvals doc](https://developers.openai.com/codex/agent-approvals-security)):

```toml
[apps.my_database_app]
destructive_enabled = false
open_world_enabled  = true
```

> "Setting `destructive_enabled = false` on an app prevents Codex CLI from executing *any* tool from
> that app that advertises `destructiveHint: true`. **This is a hard block, not a prompt** — the tool
> is simply unavailable."

Three further consumer behaviours are documented in the same place, each of which converts a
declaration into a decision:

1. **Auto-approval.** `readOnlyHint == true` may be auto-approved under `on-request` / `never`.
2. **Precedence.** `destructiveHint: true` wins over `readOnlyHint: true` when both are set — the
   only place a client repairs an internally inconsistent declaration, and it repairs exactly one of
   the six possible contradictions.
3. **Session-scoped approval memory.** *"Allow and remember"* (Codex PR #10584, merged Feb 2026)
   caches an approval keyed on `(server, connector_id, tool_name)` — **a key with no argument in
   it.** One approved `run_command` call silently approves every later `run_command` call in the
   session.

And in a sibling article the same author records the tier built on top:
[*The Writes Mode*](https://codex.danielvaughan.com/2026/07/24/codex-cli-writes-app-approval-mode-mcp-tool-annotations-read-only-hint-approval-flow/)
(2026-07-24) — *"lets reads fly and gates writes"*, with `readOnlyHint` as the discriminator.

Cross-client, the picture from the same comparison table: Claude Code uses a **classifier** (a
secondary model) rather than the declaration; Gemini CLI has no per-extension annotation filtering
at all. So of the three major hosts, one hard-blocks on an untrusted boolean, one pays for an LLM
call to avoid trusting it, and one ignores it. Nobody computes it.

### 1.3 Why it is limited — the failure mode, stated precisely

> **The annotation is a property of the tool. The risk is a property of the call.**

For most MCP tools this is a tolerable approximation: `delete_user` is destructive whatever its
arguments. It collapses completely for the one family where the argument *is* a program:

| Tool | Declared once | Actual per call |
|---|---|---|
| `run_command` | `destructiveHint: ?` | `ls -la` → additive-free; `rm -rf ~` → destructive |
| `run_command` | `openWorldHint: ?` | `wc -l f` → closed; `curl … \| sh` → open |
| `run_command` | `idempotentHint: ?` | `mkdir -p x` → yes; `echo x >> f` → no |

A server author has exactly three bad options and no good one: declare permissively and lose the
host's gate on the dangerous calls; declare pessimistically and have `open_world_enabled = false`
hard-block `ls`; or omit and inherit the pessimistic default, which is the second option by another
name. The MCP blog records the consequence from the server's side — *"Many servers ship without
them"* — and from the client's — *"no MCP client lets users filter tools by annotation values, and
none surface annotations as context in approval prompts."*

The second-order failure is the one that actually bites. The host's **hard block** and its
**remembered approval** are both keyed on identifiers that do not include the command. A malicious
or merely careless server declaring `destructiveHint: false` on a shell tool does not get warned
about, prompted about, or blocked — it gets *auto-approved*, and the spec's answer to this is a
sentence telling the client not to believe the thing it is about to act on.

**This is not an implementation bug and it will not be patched.** It is the documented, intended
boundary of a declaration-based scheme. That is precisely why it is worth building against rather
than waiting out.

### 1.4 The corpus — where the gap has already been paid for

All read 2026-09-16.

- **The spec's own admission.** *"An untrusted server can lie. A server can claim
  `readOnlyHint: true` and delete your files anyway."* ([MCP blog, 2026-03-16](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/))
- **Five open SEPs, none merged** — [#1913](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1913)
  (trust & sensitivity, co-authored GitHub + OpenAI),
  [#1984](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1984) (governance/UX),
  [#1561](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1561) (`unsafeOutputHint`),
  [#1560](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1560) (`secretHint`),
  [#1487](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1487) (`trustedHint`).
  Every one adds a boolean the server declares about itself. **The vocabulary is growing; the trust
  model is not changing.**
- **SEP-1075** (`reads_private_data` / `sees_untrusted_content` / `can_exfiltrate`) with the
  enforcement rule *"never allow all three in a single tainted execution path"* — a rule whose three
  inputs are, again, self-declared.
- **Gemini CLI, [CVE-2026-12537](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/)**,
  CVSS 10.0 (Black Hat USA 2026, Elad Meged / Novee): *"its tool allowlist was checked at
  registration but never enforced at execution."* The same defect class one layer down — a
  registration-time fact used as an execution-time guarantee.
- **Claude Code, CVE-2026-54316** (same disclosure): a read-only validator *"bypassed by stripping
  quotes"*, carrying *"hardcoded read-only exemptions."* An attempt to *derive* read-only-ness that
  failed on shell quoting — the direct precedent for why §3 puts the derivation behind a parser and
  fails closed when the parse is incomplete.
- **[LayerX / Claude Desktop extensions RCE](https://layerxsecurity.com/blog/claude-desktop-extensions-rce/)**,
  cited by the MCP maintainers themselves as the demonstrated lethal-trifecta chain: a malicious
  calendar event description, an MCP calendar server, and a **local code-execution tool** as the
  linchpin.

### 1.5 Lifecycle, as the feature does it and as Caro must

| | MCP `ToolAnnotations` | What Caro must do |
|---|---|---|
| When computed | once, at `tools/list` | once per call, on the argument |
| Cache key | tool name (+ server) | **none — there is no cache** |
| Invalidation | `list_changed` notification | n/a |
| Trust | server's self-declaration | derived from the input, or `Undetermined` |
| Redundant init avoided by | caching the answer | having no initialization: in-binary lexicon, zero I/O |

The "avoid redundant initialization" requirement in the task template is satisfied here by
*deletion*, not by caching. There is no session, no handshake, and no warm state to reuse: the
lexicon is compiled into the binary by `build.rs` (ADR-070's pattern) and the analysis is a pure
function of the input string. The cost that caching exists to amortise does not exist.

---

## 2. Phase 2 — Competitive differentiation

### 2.1 What they get right, and Caro should replicate

1. **Pessimistic defaults, stated as a direction.** Absent ⇒ dangerous. §3 keeps this as an
   invariant with a name (**A3**) rather than as a convention.
2. **The "what client behaviour changes?" test.** The maintainers reject any proposed annotation
   that does not map to a concrete client decision. Caro's verb catalogue has no such gate and is at
   fifteen verbs. Adopting it costs one line per field in the ADR and is the cheapest quality control
   available. Every field in §3.2 carries its consumer behaviour.
3. **Four fields, held at four for eighteen months.** The interface *"has stayed small since then,
   and that's been intentional."* `caro.annotate.v1` emits **exactly the four MCP fields** and adds
   no fifth of its own invention. Extension pressure goes to a `_meta`-shaped namespace (D9), not
   into the core object.
4. **Naming the combination problem.** `openWorldHint` was designed to let a client reason about
   *sessions*, not tools. Caro emits per-call facts that a session-level reasoner can accumulate; it
   does not attempt the accumulation (§7.4).

### 2.2 The design gaps to avoid by designing the schema first

| Their gap | Caro's schema answer |
|---|---|
| A hint is a `boolean?`, so "unknown" and "false" are the same wire value at the default-resolution step. | **Three-valued `Tri { True, False, Undetermined }`** on every field, and the MCP projection to `Option<bool>` is a *separate, total, lossy* function the consumer can skip. |
| A declaration is indistinguishable from a computation once it is on the wire. | `Provenance` has exactly two variants, `Derived` and `Pessimistic`. **There is no `Declared` variant and one must never be added.** A server's own hints, when supplied, are echoed in a sibling field and never merged. |
| `readOnlyHint: true` + `destructiveHint: true` is expressible; Codex repairs it by precedence, other clients do not. | The four fields are derived from **one** site analysis, so contradictions are not constructible. A conformance test asserts the six impossible pairs (T14). |
| The permissive claim is the unproven one, and the scheme makes it the cheapest to assert. | **Asymmetric proof rule (A2).** `read_only: True` requires *every* site resolved and in the read-only class. One unresolved site anywhere ⇒ `Undetermined` ⇒ pessimistic projection. Proving harm is easy; proving harmlessness requires completeness. |
| Cache keys omit the argument (`(server, connector_id, tool_name)`). | The report has **no tool identity field at all.** It carries `command_sha256`. A consumer that wants to remember an approval can only key it on the command. |

### 2.3 Unique positioning — what Caro can do that they cannot

- **Per-call derivation is impossible inside MCP and trivial outside it.** A server cannot annotate
  per call; the annotation is attached to the definition and returned before any argument exists. A
  subprocess that is handed the argument has the one input the protocol does not.
- **The contradiction report is the product.** `caro annotate --declared '{"destructiveHint":false}'
  "rm -rf ~"` → `verdict: Contradiction`, exit 3. That is a *lying-server detector*, and it is the
  missing input to the `destructive_enabled = false` hard block that currently trusts the liar.
  Nothing in the ecosystem produces it: hosts either believe the declaration (Codex), replace it with
  a model call (Claude Code's classifier), or ignore it (Gemini CLI).
- **Offline, deterministic, no model.** Claude Code's answer to the same problem is a secondary
  classifier — a per-call inference cost, a network dependency, and a nondeterministic verdict. Caro
  is a pure function with a test vector. Against Apollo's Watcher the strategic line from
  [Hermes 2026-09-09 §4](.hermes/digests/2026-09-09-agent-market-scan.md) holds exactly:
  **be the stage the monitors call.** This verb is that stage, with a schema the monitor already
  speaks.
- **The projection makes it droppable in.** Because the output includes a spec-shaped
  `mcp_projection`, a gateway (ADR-043) can substitute derived annotations for declared ones on the
  wire without the downstream client learning a new vocabulary. That is an adoption path that costs
  the client zero.

### 2.4 What already exists in the tree

More than half of this is assembly, not invention.

| Need | Exists | Location |
|---|---|---|
| Destructive-filesystem classification | `has_destructive_fs`, `has_systemwide_write` | `src/caroml/validators/side_effects.rs` |
| Network / open-world classification | `has_network` (curl/wget/ssh/scp/rsync) | `src/caroml/validators/side_effects.rs` |
| Privilege classification | `has_sudo` | `src/caroml/validators/side_effects.rs` |
| Dangerous-pattern corpus (52+) | `SafetyValidator`, `DangerPattern` | `src/safety/mod.rs:155`, `src/safety/patterns.rs` |
| Risk vocabulary + routing | `RiskLevel`, `SuggestedRouting` | `src/models/mod.rs:152`, `src/models/mod.rs:189` |
| Serializable-from-day-one derives | `Serialize, Deserialize, JsonSchema` already on `RiskLevel` | `src/models/mod.rs:149` |
| Execution-site enumeration | ADR-067 `caro.decompose.v1` (paper) | `docs/adr/ADR-067-*` |
| "Which argument is a program" | ADR-070 `caro.delegation.v1` lexicon + `build.rs` catalog pattern (paper) | `docs/adr/ADR-070-*` |
| Egress classification | ADR-064 sandbox-egress conjunction gate (paper) | `docs/adr/ADR-064-*` |

**Dependency posture.** v1 depends on **no unlanded ADR**. Where ADR-067's splitter would be the
right site enumerator, v1 uses a local splitter over the same operator set and marks the seam (D6);
when 067 lands, the local one is deleted in a PR that changes no output. This follows ADR-070 §D14's
rule and ADR-071's: buildable on the tree that ships in 1.4.0, not on the paper stack.

**What this verb is not.** It is not `SafetyValidator` with a new coat of paint. `SafetyValidator`
answers *"should this run?"* on Caro's own risk axis. `caro.annotate.v1` answers *"what are the four
MCP booleans for this string?"* — a different, narrower, and externally-defined question, and the
two disagree on purpose: `rm -f /tmp/x` is `RiskLevel::Moderate` and `destructive: True`; `curl
https://example.com` is `RiskLevel::Safe` and `open_world: True`. Collapsing them would make the
projection unusable to an MCP client.

---

## 3. Phase 3 — Scope definition

### 3.1 The verb

```
caro annotate [--json] [--declared <json>] [--lexicon-only] <command>
```

- `--json` is the default and only output format (D10).
- `--declared` takes the server's own `ToolAnnotations` object verbatim. Supplying it switches the
  verdict axis from *derivation completeness* to *agreement*.
- `--lexicon-only` prints the catalog stamp and exits 0, so CI can assert the lexicon has not
  drifted without running an analysis.

### 3.2 New types

All in `src/models/mod.rs` unless stated. All `#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]` from day one, per the task constraint.

```rust
/// Three-valued, because MCP's `boolean?` conflates "false" with "not known"
/// at exactly the point where a client acts on it.
#[derive(..., Copy, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tri { True, False, Undetermined }

/// Two variants. A third — `Declared` — must never be added; see ADR-072 §A1.
#[derive(..., Copy, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// A lexicon entry matched a resolved execution site in the input.
    Derived,
    /// No entry matched, or the parse was incomplete. Projects to the
    /// MCP-spec pessimistic default.
    Pessimistic,
}

/// One field's answer plus the evidence that forced it.
pub struct Derivation {
    pub value: Tri,
    pub provenance: Provenance,
    /// Empty iff `provenance == Pessimistic`. Never empty otherwise.
    pub basis: Vec<Basis>,
}

pub struct Basis {
    /// Stable catalog id, e.g. "ANN-FS-RM-01". Suppressible by a consumer.
    pub entry_id: String,
    /// Byte offsets into the input command. Half-open.
    pub span: (usize, usize),
    /// Index into `AnnotationReport::sites`.
    pub site: usize,
}

/// The four fields, and only the four.
pub struct DerivedAnnotations {
    pub read_only:   Derivation,  // consumer: skip the confirmation dialog
    pub destructive: Derivation,  // consumer: warn / hard-block
    pub idempotent:  Derivation,  // consumer: safe to retry on failure
    pub open_world:  Derivation,  // consumer: scrutinise output as untrusted
}

/// Spec-shaped, lossy, and derived from `DerivedAnnotations` by a total
/// function. Present so a gateway can substitute on the wire.
pub struct McpToolAnnotations {
    pub read_only_hint:   Option<bool>,
    pub destructive_hint: Option<bool>,
    pub idempotent_hint:  Option<bool>,
    pub open_world_hint:  Option<bool>,
}

pub struct SiteAnnotation {
    pub site: usize,
    pub span: (usize, usize),
    /// Resolved program token, if one could be resolved *from the string alone*.
    pub program: Option<String>,
    pub resolution: SiteResolution, // Resolved | Unresolved | Unparsed
}

/// Emitted only when `--declared` was supplied.
pub struct Agreement {
    pub field: AnnotationField,       // ReadOnly | Destructive | Idempotent | OpenWorld
    pub declared: Option<bool>,       // echoed verbatim, never merged
    pub derived: Tri,
    pub relation: AgreementRelation,  // Agrees | Contradicts | Unprovable
}

pub struct AnnotationReport {
    pub schema: &'static str,          // "caro.annotate.v1"
    pub command_sha256: String,        // the only identity in the document
    pub verdict: AnnotateVerdict,
    pub annotations: DerivedAnnotations,
    pub mcp_projection: McpToolAnnotations,
    pub sites: Vec<SiteAnnotation>,
    pub coverage: Coverage,            // resolved_sites / total_sites
    pub declared_input: Option<McpToolAnnotations>,
    pub agreements: Vec<Agreement>,    // empty unless --declared
    pub lexicon: LexiconStamp,         // { entries: usize, sha256: String }
}

#[derive(..., Copy, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotateVerdict {
    /// Derivation complete; with --declared, every field Agrees or Unprovable.
    Consistent,
    /// --declared claimed something more permissive than the derivation proves.
    Contradiction,
    /// One or more sites unresolved; pessimistic defaults projected.
    Undetermined,
    /// The input did not parse as POSIX shell.
    Unparsed,
}

impl AnnotateVerdict { pub fn exit_code(self) -> u8 { /* total */ } }
```

**Method contracts.**

- `AnnotationReport::analyze(command: &str, lexicon: &Lexicon, declared: Option<&McpToolAnnotations>) -> AnnotationReport`
  — pure, total, no I/O, no allocation of anything the input did not imply. Panics never.
- `DerivedAnnotations::project(&self) -> McpToolAnnotations` — total; `Tri::Undetermined` ⇒ the
  MCP-spec default for that field (`read_only: false`, `destructive: true`, `idempotent: false`,
  `open_world: true`), emitted as `Some(default)` not `None`, so a downstream client that does not
  implement default-resolution cannot get it wrong.
- `AnnotateVerdict::exit_code(self) -> u8` — a method, not a CLI-layer match, so it is testable
  without spawning a process (ADR-070 §D11).

### 3.3 The lexicon

`data/annotations/*.yaml`, compiled by `build.rs` into a static table. One file per family so a
contributed entry is a one-file PR. Entry shape:

```yaml
- id: ANN-FS-MKDIR-01
  program: mkdir
  # Only the flags that change the answer; absence means "any"
  requires_flags: ["-p"]
  read_only:   false
  destructive: false     # additive
  idempotent:  true      # -p makes the second call a no-op
  open_world:  false
  test_cases:
    - command: "mkdir -p /tmp/a/b"
      expect: { read_only: false, destructive: false, idempotent: true, open_world: false }
    - command: "mkdir /tmp/a"
      expect: { idempotent: false }   # without -p, second call fails
```

`test_cases` are emitted by `build.rs` into the eval suite, so a drifted entry fails a test rather
than silently degrading a verdict (the ADR-071 §6 instrumentation pattern).

**v1 coverage target:** the ~80 programs that account for the bulk of the ADR-063 destructive-command
benchmark plus the read-only core (`ls`, `cat`, `grep`, `find`, `wc`, `stat`, `git status|log|diff`,
`ps`, `df`, `du`). Everything else resolves to `Unresolved` → `Undetermined` → pessimistic. **The
`Undetermined` rate is a published release metric, not a hidden one** (§6).

### 3.4 Files that change

No new top-level module.

| File | Change |
|---|---|
| `src/models/mod.rs` | + the types in §3.2 (~180 LOC of definitions + derives) |
| `src/safety/annotations.rs` | **new file, existing module** — lexicon type, site splitter, `analyze` |
| `src/safety/mod.rs` | + `pub mod annotations;` (1 line) |
| `src/caroml/validators/side_effects.rs` | `has_network` / `has_destructive_fs` / `has_systemwide_write` change from private to `pub(crate)`; **no logic change** |
| `src/main.rs` | + `Commands::Annotate { … }` and its dispatch arm |
| `build.rs` | + the `data/annotations/` compile step (mirrors ADR-070's) |
| `data/annotations/*.yaml` | **new data**, ~12 files |
| `tests/annotate_contract.rs` | **new** — the contract tests in §3.6 |

### 3.5 Exit-code / output contract

Stdout is exactly one newline-terminated `caro.annotate.v1` JSON document. A consumer keys on
`verdict`; the exit code is the shortcut.

| Exit | Verdict | Meaning to a script |
|---|---|---|
| `0` | `Consistent` | derivation complete; declared hints (if any) are not more permissive than the facts |
| `3` | `Contradiction` | **the finding.** A declared hint claims permission the command does not earn |
| `4` | `Undetermined` | derivation incomplete; `mcp_projection` carries pessimistic defaults |
| `5` | `Unparsed` | not POSIX shell (PowerShell, Cmd, binary garbage) |
| `1` | — | internal error, **no payload on stdout** |
| `2` | — | reserved for `clap` usage errors, never emitted |

This sits inside the `3..=15` reserved range ADR-070 §D11 proposed and adds no sixth collision on
`2`. It does **not** do the consolidation work owed by ADR-065/066/067/068 (§7.12).

### 3.6 Integration tests — known input → deterministic JSON + exit code

`tests/annotate_contract.rs`. Every case asserts the full document, not a field.

| # | Input | Expect |
|---|---|---|
| T1 | `ls -la` | all four `Derived`; `read_only: True`, `destructive: False`, `idempotent: True`, `open_world: False`; exit 0 |
| T2 | `rm -rf ~/x` | `read_only: False`, `destructive: True`, `idempotent: True` (second `rm -rf` is a no-op), `open_world: False`; exit 0 |
| T3 | `mkdir -p /tmp/a` | `idempotent: True`; exit 0 |
| T4 | `mkdir /tmp/a` | `idempotent: False`; exit 0 |
| T5 | `echo x >> f` | `destructive: False` (additive), `idempotent: False`; exit 0 |
| T6 | `curl https://example.com` | `open_world: True`, `read_only: True`; exit 0 |
| T7 | `curl https://example.com \| sh` | `open_world: True`, `read_only: False`, `destructive: Undetermined` → projection `true`; exit 4 |
| T8 | `ls && frobnicate` | one resolved site, one unresolved ⇒ **whole-command** `read_only: Undetermined`; exit 4 (asserts A2) |
| T9 | `Get-ChildItem` | `Unparsed`; exit 5 |
| T10 | `rm -rf ~` + `--declared '{"destructiveHint":false}'` | `verdict: Contradiction`; the `destructive` agreement is `Contradicts`; exit 3 |
| T11 | `ls` + `--declared '{"destructiveHint":true}'` | `Consistent` — declared is *more* pessimistic than derived, which is never a contradiction; exit 0 |
| T12 | `frobnicate` + `--declared '{"readOnlyHint":true}'` | agreement `Unprovable`, verdict `Undetermined`; exit 4. **Asserts that unprovable ≠ contradiction.** |
| T13 | any input | `declared_input` is byte-identical to what was passed; no field of it appears inside `annotations` |
| T14 | property test over the lexicon | none of the six contradictory pairs (e.g. `read_only: True` ∧ `destructive: True`) is constructible |
| T15 | T1 run 1000× | byte-identical stdout (determinism) |
| T16 | `analyze` over the full ADR-063 corpus | zero panics; `coverage` monotonically ≤ 1.0 |
| T17 | schema | `schemars`-generated JSON Schema matches the checked-in golden file |
| T18 | `--lexicon-only` | prints `LexiconStamp`, exit 0, no analysis performed |

### 3.7 Invariants (the ones the ADR names and the tests enforce)

- **A1 — No laundering.** `Provenance` has two variants. A declaration entering via `--declared`
  can reach `declared_input` and `agreements` and nowhere else. (T13)
- **A2 — Asymmetric proof.** A permissive value (`read_only: True`, `destructive: False`,
  `open_world: False`) requires *every* site resolved. One unresolved site anywhere forces
  `Undetermined` for that field. Harmful values may be derived from a single site. (T8)
- **A3 — Fail closed to the spec, not to silence.** `Undetermined` projects to the MCP default and
  is reported as `Some(default)`, never `None`. Exit 4 is not exit 0.
- **A4 — No I/O.** `analyze` opens no file, resolves no `$PATH`, probes no port. A verdict that
  depends on state the analysed command can change is the defect this document exists to avoid
  (ADR-071 §7.3, permanently rejected, and reaffirmed here).
- **A5 — No tool identity.** The report has no `tool_name`, no `server`, no `connector_id`. A
  consumer cannot build the `(server, connector_id, tool_name)` cache key out of it.

---

## 4. Rollout

| PR | Contents | Behaviour change |
|---|---|---|
| 1 | types, lexicon, `build.rs`, `data/`, unit tests, `pub(crate)` widening | **none** — no CLI surface, no verdict change |
| 2 | `caro annotate` verb + the T1–T18 contract tests | additive, read-only |
| 3 | ADR-043 gateway substitutes `mcp_projection` for declared annotations on the wire | **out of scope for v1** (§7.1) |
| 4 | `SafetyValidator` consults the lexicon so plain `caro "…"` benefits | **out of scope for v1** (§7.2) |

PR 1 and PR 2 are the deliverable. PRs 3 and 4 are where every false positive and every ecosystem
argument lives, and shipping either inside v1 would repeat the pattern ADR-067 §1 records from the
host's own v2.1.259-ship / v2.1.260-revert whiplash.

---

## 5. Validation discipline (`.claude/rules/validation-discipline.md`)

This is a **new user-facing capability**, so the five gates apply. Reporting them honestly:

- **Gate 1 (20 transcripts): NOT MET.** Zero first-hand user transcripts exist for this feature.
- **Gate 2 (no surveys): trivially met** — no survey is cited.
- **Gate 3 (demoware trap): met** — §6.
- **Gate 4 (devil's advocate): pending.** The `devils-advocate` agent has not reviewed this
  document; §8 records the objections a reviewer should press hardest, but that is self-review and
  does not close the gate.
- **Gate 5 (PMF claim): met by making no PMF claim.** Nothing here asserts users want this.

The rule exempts *"responses to evidence we already have — broken things, vulnerabilities,
friction."* **This scope claims that exemption, and the claim is contestable.** The honest form of
the argument: the MCP *specification* is not broken — it documents its boundary precisely. What is
broken is a shipped host converting a self-declared boolean into a hard block and a remembered
approval, which is a defect in the consumer and is documented in the vendor's own configuration
reference. A reviewer who thinks that reading is too generous should send this back to Gate 1, and
§8.1 is written to make that easy. **ADR-072 therefore ships as `Proposed — Gate 1 unmet, Gate 4
pending`, and PR 1 must not open until a human resolves §8.1.**

---

## 6. Gate 3 — what breaks at 100 real users

**The assumption that holds at demo scale.** The lexicon is a closed set of literal program names
with literal flag predicates. At 10 users running `ls`, `git status` and `rm`, coverage is near 1.0.
At 100 users on 4 platforms, three things break:

1. **Coverage collapse.** The long tail of program names (`terraform`, `kubectl`, `just`, `uv`,
   in-repo `./scripts/deploy.sh`) is unbounded. Coverage falls, `Undetermined` dominates, the
   projection is always the pessimistic default, and the verb degenerates into a constant function.
   **This is the dangerous one, because a constant pessimistic answer is indistinguishable from a
   correct one to a consumer that only reads `mcp_projection`.**
2. **Flag-predicate drift.** `rm --recursive` vs `rm -r` vs `rm -rf` vs `rm -fr`; BSD vs GNU flag
   sets for `cp`, `sed`, `date`. A predicate written against GNU silently misses on macOS.
3. **Idempotence is genuinely hard and the lexicon will be wrong.** `git push` is idempotent until it
   isn't; `kubectl apply` is idempotent by design and not in practice; `docker run` depends on
   `--rm`. An overconfident `idempotent: True` tells a host *"safe to retry on failure"* — the one
   hint whose consumer behaviour is a **re-execution**.

**Instrumentation that reveals it.**

- `coverage.resolved_sites / coverage.total_sites` is in **every** report, and the aggregate
  `Undetermined` rate over the ADR-063 corpus is published in `CHANGELOG.md` on every release that
  touches `data/annotations/`. A drifting lexicon shows up as a falling number, not as silence.
- `lexicon.entries` + `lexicon.sha256` in every report, plus `--lexicon-only`, so CI can assert both.
- Every entry carries `test_cases`; `build.rs` emits them into the eval suite. A wrong predicate
  fails a test rather than degrading a verdict.
- `entry_id` is stable and suppressible, so a bad entry is reported as a countable suppression in a
  user's config rather than as a silent workaround.

**Fallback if the failure mode triggers.**

- Coverage collapse: the failure is *visible* (exit 4) and *additive to fix* (a YAML PR). Worst case
  the verb is honest and unhelpful, which is the correct failure direction for a safety tool.
- Flag drift: predicates gain a `platforms: [linux, macos]` key — the schema reserves it in v1 and an
  entry that needs it and omits it is a lint error.
- Idempotence: **`idempotent` is the one field permitted to be `Undetermined` while its three
  siblings are `Derived`.** An entry that cannot justify an idempotence value omits it and the field
  falls to pessimistic (`false` — "not safe to retry"), which is the direction that cannot cause a
  duplicate execution.

**What does not break:** latency (no I/O, in-binary table — ADR-062's per-command budget is not at
risk), concurrency (pure function), determinism (T15).

---

## 7. Explicitly out of scope

Next version, not this one. Each names the seam that admits it without a schema break.

1. **Wire substitution in the ADR-043 gateway.** Replacing a server's declared annotations with
   `mcp_projection` on the `tools/list` response. That is a proxy behaviour change with an ecosystem
   argument attached. **Seam:** `mcp_projection` is already spec-shaped, so the gateway PR is a
   serialisation swap.
2. **`SafetyValidator` consulting the lexicon.** Changes verdicts on inputs users already run and
   needs its own false-positive corpus. **Seam:** `ValidatorContext` (`src/caroml/validators/mod.rs:38`)
   gains one optional field.
3. **Live probing.** Resolving `$PATH`, stat'ing a target, checking whether a URL is reachable.
   **Permanently rejected**, not deferred — same reasoning as ADR-071 §7.3 and ADR-070 §D10. Recorded
   here so it is not reopened as an enhancement.
4. **Session-level trifecta accumulation.** SEP-1075's *"never allow all three in a single tainted
   execution path"* requires state across calls. Caro is stateless by construction; the accumulator
   is the consumer's job and Caro's job is to give it correct per-call inputs. **Seam:** the report is
   append-only to a caller's ledger (ADR-053).
5. **The three SEP-1075 fields** (`reads_private_data`, `sees_untrusted_content`, `can_exfiltrate`).
   None is merged; deriving unmerged vocabulary would bind v1 to a draft. **Seam:** D9's `_meta`-shaped
   namespace, where they can be prototyped without touching the four core fields.
6. **Non-shell MCP tools.** `delete_user`, `send_email`. Caro has no view of them and should not
   pretend to. The verb takes a command, not a tool call.
7. **Non-POSIX dialects.** PowerShell and Cmd get `Unparsed` (T9). LOLBAS is the corresponding corpus
   and is its own ADR — already owed by ADR-071 §7.5.
8. **User-extensible lexicon entries.** Needs a precedence story with `patterns.toml` and the profile
   system. v1 ships a closed set.
9. **Argument-level data-flow.** `tar -xzf x.tar.gz` writes wherever the archive says. v1 annotates
   the *program*, not the archive. Transitive effects need ADR-071 §7.9's trust-handoff model.
10. **A human renderer.** `--json` is the only format in v1; the consumer is a harness, not a person.
11. **Any coverage or accuracy number published externally.** The `Undetermined` rate is a release
    metric in `CHANGELOG.md`; mapping it onto a public suite and claiming recall is Hermes 3.11 and
    belongs with ADR-060/063.
12. **The exit-code consolidation PR.** §3.5 adopts ADR-070's reserved range and does not do the work
    owed by ADR-065/066/067/068.
13. **Replacing the local splitter with ADR-067's.** Deliberate duplication with a named deletion
    date (D6); doing it now would make v1 depend on an unlanded ADR.

---

## 8. Open questions for the human reviewer

**8.1 — Is the validation-discipline exemption legitimate?** §5 claims this is a defect-class
response because a shipped host hard-blocks on an untrusted boolean. The counter-reading is that the
MCP spec documents this boundary, Codex documents its own config semantics, nothing is broken, and
this is a market-driven feature that owes 20 transcripts. **This is the load-bearing question and it
should be answered before PR 1 opens.** ADR-069's first draft was rejected on exactly this axis.

**8.2 — Is `idempotent` worth shipping at all in v1?** It is the hardest of the four to derive, the
only one whose consumer behaviour is a *re-execution*, and §6.3 argues it will be wrong. Shipping it
as permanently `Undetermined` in v1 — deriving only the other three — is a defensible smaller scope
and loses little, since the pessimistic default (`false`, don't retry) is what a cautious host wants
anyway.

**8.3 — Does `--declared` belong in v1, or is it the whole product?** §2.3 argues the contradiction
report is the differentiator. If that is right, the flag is not an add-on and the verb should perhaps
*require* it, with the no-declaration case as the degenerate form. Shipping both modes doubles the
verdict axis (§3.5 has four verdicts because of it).

**8.4 — Sixteen verbs.** `annotate` would be the sixteenth proposed verb in eleven weeks, against a
1.4.0 binary that ships approximately none of them. The MCP maintainers' discipline that §2.1.2
praises — *reject any addition that does not change a concrete consumer behaviour* — applies to
Caro's own ADR queue at least as sharply as it applies to `ToolAnnotations`. **The honest reading of
the scan series is that its bottleneck is no longer scoping.**

**8.5 — Should this supersede rather than extend?** ADR-058's `caro.assessment.v1` is the
decision-contract ADR under moratorium (ADR-059). If assessment lands first, `mcp_projection` is
arguably a *formatter* on an assessment result rather than a verb of its own. That would be a smaller
tree and a worse contract (the four fields are not a function of a risk level), but the question
deserves an explicit answer rather than an implicit one.

---

## 9. Sources

All read 2026-09-16 unless stated.

- [MCP — *Tool Annotations as Risk Vocabulary: What Hints Can and Can't Do*](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/) (2026-03-16)
- [MCP specification — `ToolAnnotations` schema](https://modelcontextprotocol.io/specification/2025-11-25/schema#toolannotations)
- [MCP specification — Server/Tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [Daniel Vaughan — *MCP Tool Annotations as Risk Vocabulary: How Codex CLI Uses Hints to Drive Approval Decisions*](https://codex.danielvaughan.com/2026/04/12/mcp-tool-annotations-risk-vocabulary-codex-cli/) (2026-04-12, page updated 2026-09-16)
- [Daniel Vaughan — *The Writes Mode*](https://codex.danielvaughan.com/2026/07/24/codex-cli-writes-app-approval-mode-mcp-tool-annotations-read-only-hint-approval-flow/) (2026-07-24)
- [OpenAI — Codex CLI configuration reference](https://developers.openai.com/codex/config-reference)
- [OpenAI — Codex CLI agent approvals & security](https://developers.openai.com/codex/agent-approvals-security)
- [Novee Security — *Critical Flaws in Anthropic, Google and OpenAI's Coding Agents*](https://novee.security/blog/critical-flaws-in-anthropic-google-and-openais-coding-agents/) (Black Hat USA 2026; CVE-2026-12537, CVE-2026-54316)
- [Simon Willison — *The lethal trifecta for AI agents*](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/) (2025-06-16)
- [LayerX — Claude Desktop extensions RCE](https://layerxsecurity.com/blog/claude-desktop-extensions-rce/)
- SEPs [#1913](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1913), [#1984](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1984), [#1561](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1561), [#1560](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1560), [#1487](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1487), [#1075](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1075)
- In-tree: `caro-scope-assessment-contract-2026-08-25.md:202-211`; `.hermes/digests/2026-09-09-agent-market-scan.md` §4; `src/models/mod.rs:152`; `src/safety/mod.rs:155`; `src/caroml/validators/mod.rs:38`; `src/caroml/validators/side_effects.rs`; `docs/adr/ADR-063`, `-064`, `-067`, `-070`, `-071`

---

**Quick Actions:**
  `y` = yes | `c` = continue | `ta` = try again | `n` = next | `rp` = recommended plan

**💡 Recommended:** Answer §8.1 (is the validation-discipline exemption legitimate?) before anything
else — it decides whether ADR-072 proceeds to PR 1 or returns to Gate 1.

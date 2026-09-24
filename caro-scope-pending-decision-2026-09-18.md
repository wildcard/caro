# Scope — `caro.pending.v1`: durable pending-decision record

**Companion to** [`docs/adr/ADR-074-pending-decision-record.md`](docs/adr/ADR-074-pending-decision-record.md)
**Date**: 2026-09-18 · **Produced by**: `caro-research--scoping-process` (autonomous run)
**Status**: Proposed. Not implemented, not branched, not committed.

---

## 1. Phase 1 findings, condensed

| | Pizza Bot (AWS, Apache-2.0, 2026-09-10) | Claude Code `defer` (v2.1.89) |
|---|---|---|
| Where the pause lives | server-owned SQLite + files in `~/.pizza-bot-oss`, LangGraph checkpoints | the host's session; process exits, tool call preserved |
| Who declares it | the **skill**, via `interruptOn` per tool, `allowedDecisions` per tool | the **hook**, per call, via `permissionDecision` |
| Human options | approve / edit / reject (skill decides which) | approve / deny at resume |
| Re-evaluation on resume | not described | yes — *"resume with `-p --resume` to have the hook re-evaluate"* |
| Durability requires | a live server (quitting the desktop app ends its runs; *"you lose the step in flight rather than the thread"*) | a resumable session id |
| Documented where | a blog post and the repo | **one changelog line** ([#41791](https://github.com/anthropics/claude-code/issues/41791)) |

**Defects that shape the design.**

- [#89561](https://github.com/anthropics/claude-code/issues/89561) — `"ask"` has *no blocking
  effect* under `permission_mode: "auto"`; *"it appears to silently resolve to an implicit allow
  rather than a safe default."* → **D2**.
- [#39344](https://github.com/anthropics/claude-code/issues/39344) — an `"ask"` decision silently
  overriding `permissions.deny`. → **D1**.
- Pizza Bot `edit` — the human alters the proposed action inside the approval card; the approved
  text is not the assessed text. → **D6**.
- [#41791](https://github.com/anthropics/claude-code/issues/41791) — `defer`'s semantics exist only
  in a changelog sentence, so integrators infer them. → the whole document.

**What they get right and this design copies.** Re-evaluation on resume (Claude Code) — made
mandatory here. An explicit `edit` affordance (Pizza Bot) — kept, with re-assessment attached.
Local-first, user-owned storage in a plain folder (Pizza Bot) — kept, minus the database.

---

## 2. Types

All in **`src/safety/pending.rs`**, re-exported from `crate::safety`. Every type derives
`Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq`; every enum is
`#[serde(rename_all = "snake_case")]` and `#[non_exhaustive]`.

```rust
pub const PENDING_SCHEMA: &str = "caro.pending/1";
pub const DEFAULT_TTL_SECS: u64 = 86_400;   //  24h
pub const MAX_TTL_SECS:     u64 = 604_800;  //   7d

/// Written once, never mutated. `<store>/<record_id>.request.json`
pub struct PendingRecord {
    pub schema_version: String,          // PENDING_SCHEMA, literal-checked on parse
    pub record_id: String,               // UUIDv4 (injectable in tests)
    pub created_at: String,              // RFC 3339 UTC
    pub expires_at: String,              // RFC 3339 UTC = created_at + ttl_secs
    pub ttl_secs: u64,                   // <= MAX_TTL_SECS, else MintError::TtlTooLong
    pub command: String,                 // exact text assessed
    pub command_sha256: String,          // hex sha2 of `command`
    pub shell: ShellType,                // existing type
    pub safety_level: SafetyLevel,       // existing type — the level the mint ran at
    pub assessment: SafetyDecision,      // existing type; D8 upgrade path to caro.assessment.v1
    pub allowed_decisions: Vec<AllowedDecision>,
    pub origin: Option<String>,          // opaque until ADR-073 lands
    pub agent_id: Option<String>,        // ADR-021 attribution of the requester
    pub supersedes: Option<String>,      // set only by D6's Edited path
    pub meta: Option<serde_json::Map<String, serde_json::Value>>,
}

pub enum AllowedDecision { Approve, Deny, Edit }   // caller-narrowable, never widenable

/// Written once, never mutated. `<store>/<record_id>.resolution.json`
pub struct PendingResolution {
    pub schema_version: String,          // PENDING_SCHEMA
    pub record_id: String,               // MUST echo
    pub command_sha256: String,          // MUST echo — content binding (D5)
    pub outcome: ResolutionOutcome,
    pub actor: ResolutionActor,
    pub decided_at: String,              // RFC 3339 UTC
    pub proof: Option<serde_json::Value>,// ADR-045, carried opaquely, presence recorded
    pub meta: Option<serde_json::Map<String, serde_json::Value>>,
}

#[serde(tag = "kind")]
pub enum ResolutionOutcome {
    Approved,
    Denied { reason: Option<String> },
    Edited { new_command: String, new_command_sha256: String },
}

pub struct ResolutionActor { pub id: String, pub surface: String, pub auth: ActorAuthKind }
pub enum ActorAuthKind { None, SurfaceSession, StrongAuth }   // vocabulary shared with ADR-046

/// Computed at read time. NEVER serialized into the request file. (D3)
pub enum PendingState { Open, Expired, Resolved }

/// The `resolve` payload.
pub struct ResolutionReport {
    pub schema_version: String,
    pub record_id: String,
    pub verdict: ResolutionVerdict,
    pub command: String,                 // the text that may now run (post-edit if edited)
    pub reassessed: bool,                // always true in v1 (D7)
    pub verdict_drift: Option<VerdictDrift>,
    pub minted: Option<String>,          // new record_id when D6 re-mints
}

pub enum ResolutionVerdict { Proceed, Refuse { reason: RefuseReason } }

pub enum RefuseReason {
    Denied, Expired, ContentMismatch, VerdictWorsened,
    AlreadyResolved, SchemaMismatch, RecordNotFound, MalformedRecord,
}

pub enum VerdictDrift { Relaxed, Unchanged }   // `Worsened` is unrepresentable: it is a Refuse

/// The `list` payload.
pub struct PendingList {
    pub schema_version: String,
    pub store: String,
    pub records: Vec<PendingListEntry>,   // { record_id, state, expires_at, risk_level, command }
    pub store_scan_ms: u64,               // Gate 3 instrumentation (§7)
    pub unreadable: Vec<UnreadableEntry>, // { path, reason } — never silently skipped
}
```

### Method contracts (pure; no I/O except where named)

| Signature | Contract |
|---|---|
| `PendingRecord::mint(cmd: &str, shell, level, decision: &SafetyDecision, ttl, now, id) -> Result<Self, MintError>` | `Err(NotDeferrable{routing})` unless `decision.suggested_routing == HumanGate` (**D1**); `Err(TtlTooLong)` above `MAX_TTL_SECS`; no I/O |
| `PendingRecord::state(&self, resolved: bool, now) -> PendingState` | pure; `Resolved` > `Expired` > `Open` (**D3**) |
| `PendingStore::open(dir: &Path) -> Result<Self, StoreError>` | creates the directory if absent; never touches any other path (**D4**) |
| `PendingStore::put(&self, rec: &PendingRecord) -> Result<(), StoreError>` | `O_EXCL` create of `<id>.request.json.tmp`, `fsync`, `rename`; `Err(Collision)` on existing id |
| `PendingStore::resolve(&self, id, resp: &PendingResolution, validator: &SafetyValidator, now) -> Result<ResolutionReport, StoreError>` | the only I/O-bearing decision path; order is fixed: **schema → record exists → already-resolved → expired → content binding → outcome branch → re-assess** (**D5, D6, D7**) |
| `PendingStore::list(&self, include_expired: bool, now) -> Result<PendingList, StoreError>` | directory scan; an unparseable file becomes an `unreadable` entry, never a silent skip |

**Fixed ordering matters**: expiry is checked *before* the outcome is read, so an `Approved`
response to an expired record cannot win on a technicality; content binding is checked before the
outcome branch, so an `Edited` payload cannot smuggle a different `record_id`'s text.

---

## 3. Files changed

| # | File | Nature | Notes |
|---|---|---|---|
| 1 | `src/safety/pending.rs` | **new** (~300 LOC + ~150 test LOC) | the only new file in `src/` |
| 2 | `src/safety/mod.rs` | +2 lines (`pub mod` / `pub use`) and `JsonSchema` on `SafetyDecision` (derive at `:188`) | D0 |
| 3 | `src/models/mod.rs` | `JsonSchema` on `SuggestedRouting` (derive at `:187`) | D0; no field change |
| 4 | `src/cli/mod.rs` | `pub const EXIT_CODE_PENDING: i32 = 4;` | near `OutputFormat` (`:105`) |
| 5 | `src/main.rs` | two `Commands` variants (enum `:381`) + two arms (`match cli.command` `:2996`) | follows the existing `Check`/`Export` shape (`:514`, `:588`) |
| 6 | `tests/pending_contract.rs` | **new** | `assert_cmd` + `predicates` + `tempfile`, all existing dev-deps |

**No** new dependency (`sha2` `Cargo.toml:90`, `uuid` v4+serde `:93`, `chrono`+serde `:77`,
`thiserror` `:53`, `schemars` `:46` are present). **No** new config key — `SafetySection`
(`src/safety/mod.rs:135`) is untouched. **No** new module directory.

### CLI surface

```
caro defer <COMMAND> --store <DIR> [--ttl <DURATION>] [--safety <LEVEL>]
                     [--origin <STR>] [--agent-id <STR>]
                     [--allow <approve|deny|edit>...] [-o json|text]

caro pending list    --store <DIR> [--include-expired] [-o json|text]
caro pending resolve <RECORD_ID> --store <DIR> --response <FILE|-> [-o json|text]
caro pending show    <RECORD_ID> --store <DIR> [-o json|text]
```

`--store` is required on every one of them (**D4**). `--allow` can only narrow the default
`[approve, deny, edit]`; there is no flag that widens it.

---

## 4. Worked example

```console
$ caro defer "rm -rf ./build" --store ./.pending --safety strict --agent-id ci-runner-7 -o json
{
  "schema_version": "caro.pending/1",
  "record_id": "0199e1c2-...",
  "created_at": "2026-09-18T02:41:07Z",
  "expires_at": "2026-09-19T02:41:07Z",
  "ttl_secs": 86400,
  "command": "rm -rf ./build",
  "command_sha256": "b1946ac9...",
  "shell": "bash",
  "safety_level": "strict",
  "assessment": { "risk_level": "moderate", "suggested_routing": "human_gate", ... },
  "allowed_decisions": ["approve", "deny", "edit"],
  "agent_id": "ci-runner-7"
}
$ echo $?
4
```

The agent stops. Twelve hours later a person answers from whatever surface the integrator built:

```console
$ caro pending resolve 0199e1c2-... --store ./.pending --response - <<'JSON'
{"schema_version":"caro.pending/1","record_id":"0199e1c2-...",
 "command_sha256":"b1946ac9...","outcome":{"kind":"approved"},
 "actor":{"id":"kobi","surface":"terminal","auth":"surface_session"},
 "decided_at":"2026-09-18T14:02:00Z"}
JSON
{"verdict":"proceed","command":"rm -rf ./build","reassessed":true,"verdict_drift":"unchanged"}
$ echo $?
0
```

Had a pattern landed in the meantime that raised this command to `Block`, the same response would
have produced `{"verdict":{"refuse":{"reason":"verdict_worsened"}}}` and exit 3 (**D7**).

---

## 5. Integration tests

The 14 rows in the ADR, plus:

| # | Setup | Action | Expect |
|---|---|---|---|
| T15 | response with `schema_version: "caro.pending/2"` | `resolve` | `refuse{schema_mismatch}`, exit 3 |
| T16 | request file truncated mid-JSON | `list` | entry in `unreadable`, exit 0; `resolve` → `refuse{malformed_record}`, exit 3 |
| T17 | `--ttl 30d` | `defer` | `TtlTooLong`, exit 2 |
| T18 | `--allow deny` then an `Approved` response | `resolve` | `refuse{denied}` — a decision outside `allowed_decisions` is not honored |
| T19 | two `resolve` processes, same record, concurrent | both | exactly one `proceed`; the other `refuse{already_resolved}` |
| T20 | `defer` twice, same command | — | two distinct `record_id`s, two files; no deduplication (deliberate — dedup is an inbox feature) |
| T21 | store dir is read-only | `defer` | exit 1, stderr names the path; **no record is reported as written** |
| T22 | `-o json` on every verb | — | stdout is exactly one JSON document; all human text on stderr (stdout hygiene, ADR-034) |

Clock injection: `mint`/`state`/`resolve` take `now: DateTime<Utc>`; the CLI passes
`Utc::now()`, tests pass fixtures. No `SystemTime::now()` call inside the module.

---

## 6. Out of scope

As listed in the ADR. Two worth repeating because they will be asked for first: **no notifier**
(nothing watches the store) and **no `allow_always`** (standing permission is ADR-040's job).

---

## 7. What breaks at 100 real users (Gate 3)

- **The assumption**: one JSON file per record in one flat directory, scanned linearly by `list`.
- **Where it fails**: a long-lived CI store accumulating records that are never deleted. At ~10⁴
  files a scan costs tens of milliseconds and at ~10⁶ it is a minute; on a network filesystem,
  sooner. `defer` and `resolve` are unaffected (both are single-file operations by id).
- **Instrumentation**: `PendingList.store_scan_ms` is in the payload from day one, so the
  degradation is visible in the contract rather than discovered in a bug report.
- **Fallback**: date-sharded subdirectories (`<store>/2026-09-18/<id>.request.json`) with a
  compatibility scan of the flat root. This is an additive change to the store layout and does not
  alter any type, which is why the layout is not part of the schema.
- **Not a failure**: concurrent writers. Records are independent, ids are UUIDv4, and creation is
  `O_EXCL`; the only contended resource is a single record's resolution, covered by T19.

---

## 8. Open questions for the human reviewer

1. **Is a caller-named file store acceptable under the "no state" constraint?** The ADR argues yes
   (D4) and states the counter-argument. This is the one decision that should not be made by an
   autonomous run.
2. **Exit code 4** — claimed here, unreconciled with ADR-024's absent table. Confirm or reassign.
3. **`AllowedDecision::Edit` by default.** Pizza Bot's strongest affordance is also the largest
   surface; defaulting it on is a judgment call. `--allow approve,deny` turns it off.
4. **Gate 1.** Nothing here rests on a user interview. If the implementation is cut as a new
   capability rather than a refactor, the 20 transcripts are required first.

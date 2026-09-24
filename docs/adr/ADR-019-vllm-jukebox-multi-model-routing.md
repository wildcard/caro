# ADR-019: vLLM Jukebox — Multi-Model Routing for vLLM Backends

**Status**: Proposed  
**Date**: 2026-06-08  
**Last reviewed**: 2026-06-08  
**Authors**: caro-research--scoping-process (automated)  
**Issue**: [#663](https://github.com/wildcard/caro/issues/663)  
**Target**: Community (no validation gates — extension of caro-core)

---

## Context

### The Problem

The current `VllmBackend` (`src/backends/remote/vllm.rs`) sends every request to a
single, hardcoded model name set at construction time:

```rust
pub struct VllmBackend {
    base_url: Url,
    model_name: String,   // fixed forever after new()
    ...
}
```

A production vLLM server commonly hosts **multiple models simultaneously**: a base
LLM plus one or more LoRA adapters specialised for tasks like SQL generation,
shell scripting, code explanation, or security audit. vLLM exposes each adapter as
a first-class model in the `/v1/models` endpoint, selectable per-request via the
`model` field in `/v1/chat/completions`.

Caro cannot currently exploit this. A user who runs a Jukebox-style vLLM server
gets no benefit — every request hits the base model even when a better-suited
adapter is loaded.

### vLLM's Multi-Model Architecture (Phase 1 Research)

**Static LoRA modules** (declared at server start):

```bash
vllm serve meta-llama/Llama-3.2-3B-Instruct \
    --enable-lora \
    --lora-modules sql-lora=jeeejeee/llama32-3b-text2sql-spider \
                   shell-lora=my-org/shell-command-lora
```

The `/v1/models` endpoint then returns both the base model and all adapters:

```json
{
  "object": "list",
  "data": [
    { "id": "meta-llama/Llama-3.2-3B-Instruct", "parent": null },
    { "id": "sql-lora",   "parent": "meta-llama/Llama-3.2-3B-Instruct" },
    { "id": "shell-lora", "parent": "meta-llama/Llama-3.2-3B-Instruct" }
  ]
}
```

Selecting an adapter is **zero-overhead** — same HTTP endpoint, only `model` field
changes. The vLLM runtime handles adapter swapping.

**Dynamic LoRA loading** (runtime):

```bash
# Load
POST /v1/load_lora_adapter
{ "lora_name": "sql_adapter", "lora_path": "/path/to/adapter" }

# Unload
POST /v1/unload_lora_adapter
{ "lora_name": "sql_adapter" }
```

Requires `VLLM_ALLOW_RUNTIME_LORA_UPDATING=True`. Carries a security warning for
untrusted environments.

### Failure Modes Found in Phase 1 Research

| # | Failure mode | Frequency / Impact |
|---|---|---|
| F1 | User configures `model_name = "my-lora-adapter"` in caro config but the adapter isn't loaded on the server yet → silent 404/model-not-found, no diagnostic | High — any cold-start or deployment drift |
| F2 | User wants SQL queries to use a fine-tuned adapter but caro doesn't know about task routing → base model used for all queries, wasted capability | Medium — power users with Jukebox setups |
| F3 | `/v1/models` response shape changes across vLLM versions → hard crash on deserialisation | Low-medium — vLLM evolves quickly |
| F4 | Dynamic LoRA load succeeds but takes 1-5 s → caro's 30 s timeout is sufficient, but user sees no spinner | Low |

**This ADR solves F1 and F2 by design.** F3 is solved by defensive deserialisation
(unknown fields ignored). F4 is out of scope (spinner is a UI concern).

### Competitive Differentiation (Phase 2)

| Dimension | vLLM client ecosystem | Caro after ADR-019 |
|---|---|---|
| Model discovery | Client must hard-code model names | Auto-discover from `/v1/models` at startup |
| Task routing | Manual (user specifies `model` per request) | Declarative pattern-based routing in `caro.toml` |
| Failure on missing model | 404 with unhelpful JSON | Named error: `ModelNotFound { name, available: Vec<String> }` |
| Offline / no server | N/A | Transparent fallback to embedded backend (existing) |
| Config | OpenAI SDK params | TOML block, validated at startup |

**What vLLM gets right that we replicate**: per-request model selection via the
`model` field — same wire format, zero extra overhead.

**What we improve**: routing is declarative and verified at startup, not implicit
and discovered at runtime failure.

**What existing caro infrastructure already covers**:
- `VllmBackend` — HTTP client, JSON parsing, fallback to embedded
- `GeneratorError` — error taxonomy (add `ModelNotFound` variant)
- `BackendInfo` — augment to include discovered model list
- `remote-backends` feature flag — no new flag needed

---

## Decision

Add a **`VllmJukeboxBackend`** that wraps the existing `VllmBackend` pattern and adds:

1. **Model discovery** — calls `/v1/models` once at construction time and caches
   the list of available model IDs.
2. **Route matching** — evaluates a user-configured `[[vllm.routes]]` list to
   select the best model for each request, falling back to `primary_model` if no
   route matches.
3. **Startup validation** — emits a named error (`ModelNotFound`) if any
   configured route references a model that isn't in the discovered list.

Dynamic LoRA loading (`/v1/load_lora_adapter`) is **out of scope** for this ADR —
it requires server-side configuration changes and carries a security advisory.

### Architecture

```
VllmJukeboxBackend
├── base_url: Url
├── primary_model: String              ← fallback if no route matches
├── client: reqwest::Client
├── api_key: Option<String>
├── discovered_models: Vec<String>     ← populated at new(), from /v1/models
├── routes: Vec<JukeboxRoute>          ← compiled at new(), from config
└── embedded_fallback: Option<Arc<dyn CommandGenerator>>

JukeboxRoute {
    pattern: Regex,                    ← matches against request.input
    model: String,
}

fn select_model(input: &str) -> &str {
    routes.iter()
          .find(|r| r.pattern.is_match(input))
          .map(|r| r.model.as_str())
          .unwrap_or(&primary_model)
}
```

### Config Block (TOML)

```toml
[backend.vllm_jukebox]
base_url = "http://localhost:8000"
primary_model = "meta-llama/Llama-3.2-3B-Instruct"
api_key = ""               # optional

[[backend.vllm_jukebox.routes]]
pattern = "(?i)sql|database|query|select|insert|update"
model   = "sql-lora"

[[backend.vllm_jukebox.routes]]
pattern = "(?i)docker|kubernetes|k8s|helm"
model   = "devops-lora"
```

Routes are evaluated in declaration order; first match wins.

---

## New Types

All new types live in `src/backends/remote/vllm_jukebox.rs`.

### `JukeboxRoute`

```rust
/// A single routing rule: if `pattern` matches the user's input, use `model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukeboxRoute {
    /// Regex pattern compiled at construction time.
    #[serde(skip)]
    pub pattern: Regex,
    /// Raw pattern string (for serialisation and error messages).
    pub pattern_str: String,
    /// Model name to route to (must exist in `discovered_models`).
    pub model: String,
}
```

### `VllmModelsResponse` (deserialisation only)

```rust
/// Deserialised `/v1/models` response. Unknown fields are silently ignored
/// (`deny_unknown_fields` is deliberately absent — solves F3).
#[derive(Debug, Deserialize)]
struct VllmModelsResponse {
    data: Vec<VllmModelEntry>,
}

#[derive(Debug, Deserialize)]
struct VllmModelEntry {
    id: String,
    #[serde(default)]
    parent: Option<String>,   // present for LoRA adapters, null for base models
}
```

### `GeneratorError` extension

Add one variant to the existing enum in `src/backends/mod.rs`:

```rust
#[error("Model '{name}' not found on server; available: {available:?}")]
ModelNotFound {
    name: String,
    available: Vec<String>,
},
```

### `BackendInfo` extension

Add one field to the existing struct in `src/backends/mod.rs`:

```rust
/// Available model IDs discovered at startup (empty for single-model backends).
#[serde(default)]
pub discovered_models: Vec<String>,
```

---

## Files Changed

| # | File | Change | Type |
|---|---|---|---|
| 1 | `src/backends/remote/vllm_jukebox.rs` | New file — `VllmJukeboxBackend`, `JukeboxRoute`, `VllmModelsResponse` | **New** |
| 2 | `src/backends/remote/mod.rs` | Add `pub mod vllm_jukebox; pub use vllm_jukebox::VllmJukeboxBackend;` | Modified |
| 3 | `src/backends/mod.rs` | Add `ModelNotFound` to `GeneratorError`; add `discovered_models` field to `BackendInfo` | Modified |
| 4 | `src/config/mod.rs` | Add `VllmJukeboxConfig` struct for `[backend.vllm_jukebox]` TOML block | Modified |
| 5 | `docs/adr/README.md` | Add ADR-019 row | Modified |

No new modules required beyond the single new file. No new crates — `reqwest`
(HTTP) and `regex` (pattern matching) are already in `Cargo.toml`.

---

## Exit Code / Output Contract

`VllmJukeboxBackend` implements `CommandGenerator` identically to `VllmBackend`.
No new exit codes. The `ModelNotFound` error maps to `GeneratorError` and exits
with the same error path as `BackendUnavailable` in the CLI layer.

Structured output for tools that parse caro JSON (`--output json`):

```json
{
  "error": "ModelNotFound",
  "name": "sql-lora",
  "available": ["meta-llama/Llama-3.2-3B-Instruct", "shell-lora"]
}
```

---

## Integration Tests

All tests use `wiremock` (already in dev-deps). Known inputs → deterministic
outputs with no network dependency.

| # | Test name | Setup | Input | Expected |
|---|---|---|---|---|
| T1 | `test_jukebox_discovers_models` | `/v1/models` mock returns base + 2 adapters | `new()` | `discovered_models.len() == 3` |
| T2 | `test_jukebox_routes_sql_to_lora` | SQL route configured; `/v1/chat/completions` mock asserts `model == "sql-lora"` | `"show me all users"` | `backend_used` contains `"sql-lora"` |
| T3 | `test_jukebox_fallback_to_primary` | No matching route | `"list pdf files"` | request sent to `primary_model` |
| T4 | `test_jukebox_model_not_found_error` | `/v1/models` returns `["base"]`; config routes to `"sql-lora"` | `new()` | `Err(ModelNotFound { name: "sql-lora", available: ["base"] })` |
| T5 | `test_jukebox_unknown_models_fields_ignored` | `/v1/models` response includes `"extra_field": 42` | `new()` | no panic, models parsed correctly (solves F3) |
| T6 | `test_jukebox_empty_routes_behaves_like_vllm` | No `[[routes]]` configured | any input | identical to `VllmBackend` behaviour |
| T7 | `test_jukebox_api_key_forwarded` | API key set; `wiremock` asserts `Authorization: Bearer` header | any input | header present on `/v1/chat/completions` |
| T8 | `test_jukebox_server_unavailable_falls_back` | `/health` returns 503 | any input | embedded fallback invoked (existing fallback path) |

---

## Consequences

### Benefits

1. **Zero-overhead routing** — model selection is a string swap in the JSON body;
   no extra HTTP calls per request.
2. **Startup validation** — misconfigured route names fail loudly at `new()`, not
   silently at the 100th request.
3. **Additive, no regressions** — `VllmBackend` is untouched; existing users see
   no change.
4. **Declarative UX** — routing intent lives in `caro.toml` alongside other
   backend config; no code required.
5. **Discoverable** — `caro doctor` can report available models from
   `BackendInfo.discovered_models`.

### Trade-offs

1. **Extra HTTP call at startup** — one `GET /v1/models` at `new()`. Negligible
   (<5 ms on localhost); tolerable on LAN; bad on high-latency remote servers.
   Mitigation: only `VllmJukeboxBackend::new()` does it — `VllmBackend` is
   unchanged.
2. **Regex compilation overhead** — compiled once at `new()` via `regex::Regex::new()`,
   amortised across all requests. No per-request compile.
3. **Routes are static after construction** — no live reload. Acceptable for v1;
   dynamic reload is a v2 concern.

### Risks

1. **vLLM `/v1/models` shape change** — mitigated by `VllmModelsResponse` using
   permissive deserialisation (no `deny_unknown_fields`); test T5 covers this.
2. **Regex denial-of-service** — user-supplied patterns could be pathological.
   Mitigation: compile at `new()` (not per-request); document that patterns are
   regex, not glob.

---

## Alternatives Considered

### A: Extend `VllmBackend` in place

Add routing directly to the existing struct. Rejected: would break existing users
who depend on single-model behaviour; adds complexity to a well-tested struct.

### B: Config-level model selection only (no `/v1/models` discovery)

Skip the `GET /v1/models` call; just trust whatever the user puts in `[[routes]]`.
Rejected: F1 (silent 404 on missing model) is the most impactful failure mode.
Discovery costs one HTTP call and eliminates it.

### C: Dynamic LoRA loading via `/v1/load_lora_adapter`

Full dynamic loading — caro pushes adapters to the server on demand. Rejected for
v1: requires `VLLM_ALLOW_RUNTIME_LORA_UPDATING=True` (vLLM security warning),
requires adapter paths to be accessible from the server filesystem, adds lifecycle
management complexity. Deferred to v2 as `VllmJukeboxBackend::load_adapter()`.

---

## Out-of-Scope (v2)

- Dynamic LoRA load/unload (`POST /v1/load_lora_adapter`)
- Live config reload (route changes without restart)
- Per-route sampling parameters (temperature, max_tokens overrides)
- Model affinity / sticky routing (same model for a conversation session)
- `caro models` CLI subcommand (list available models from jukebox server)

---

## Implementation Notes

The build-spike rule (`.claude/rules/external-sdk-integration.md`) **does not
apply** — no new external crate is introduced. `reqwest` and `regex` are already
in `[dependencies]`. This ADR can proceed directly to a feature PR without a
spike PR.

`VllmJukeboxBackend::new()` should be `async` (the `/v1/models` call is async).
The constructor signature:

```rust
pub async fn new(
    base_url: Url,
    primary_model: String,
    routes: Vec<JukeboxRouteConfig>,   // raw config, compiled inside new()
    api_key: Option<String>,
) -> Result<Self, GeneratorError>
```

---

## References

- [vLLM LoRA documentation](https://raw.githubusercontent.com/vllm-project/vllm/main/docs/features/lora.md)
- ADR-018 — Azure Foundry Backend (same `reqwest`-only pattern)
- `src/backends/remote/vllm.rs` — source of truth for single-model VllmBackend
- Issue [#663](https://github.com/wildcard/caro/issues/663) — vLLM Jukebox Multi-Model Server
- [#662](https://github.com/wildcard/caro/issues/662) Handy.Computer integration (next candidate for ADR-020)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-08 | caro-research--scoping-process | Initial draft |

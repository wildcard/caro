# ADR-018 — Azure Foundry Backend: Microsoft Foundry Model Inference

**Status**: Proposed

**Date**: 2026-06-05

**Authors**: `caro-research--scoping-process` (automated research/scoping agent)

**Target**: Community / Enterprise

**Roadmap item**: #661 (Azure Foundry Backend Integration — v2.0.0, exempt from validation gates)

**Relates to**: `src/backends/remote/vllm.rs` (reference implementation), ROADMAP.md v2.0.0,
ADR-015 (MCP Safety Server)

---

## Context

As of mid-2026 Microsoft Foundry (formerly Azure AI Foundry / Azure OpenAI Service) is the
dominant enterprise LLM platform in the Microsoft ecosystem. Caro has backends for
Ollama, vLLM, Exo, OpenRouter, and Claude — but no Azure Foundry path. Enterprise
users who deploy models through Azure cannot use Caro as a subprocess in their
toolchains without manual configuration workarounds.

### The Foundry API landscape (2026)

Microsoft Foundry exposes two endpoint flavours:

**Legacy per-deployment URL** (still live, widely used):
```
POST https://{resource-name}.openai.azure.com/openai/deployments/{deployment-name}/chat/completions
     ?api-version=2024-10-21
Headers: api-key: {key}
```

**New Foundry v1 URL** (recommended as of 2025, OpenAI/v1-compatible):
```
POST https://{resource-name}.services.ai.azure.com/openai/v1/chat/completions
     ?api-version=2024-10-21
Headers: api-key: {key}
         OR
         Authorization: Bearer {entra-token}
Body: {"model": "{deployment-name}", "messages": [...]}
```

The new v1 form puts the deployment name in the JSON body `model` field (identical to
OpenAI's format), not in the URL path. The legacy form embeds it in the path.
Both are currently valid. The v1 form is the migration target.

> **CRITICAL (2026-06-05):** The Azure AI Inference beta SDK
> (`azure-ai-inference` Python/JS package) is **deprecated** and retires
> **2026-08-26**. Teams still using the beta SDK are 82 days from a breaking
> change. Caro's implementation targets the stable OpenAI/v1 API only — no
> dependency on the deprecated SDK.

### What Caro already has

| Existing asset | How it helps |
|---|---|
| `src/backends/remote/vllm.rs` | OpenAI-compatible `/v1/chat/completions` HTTP call; exact pattern to extend |
| `reqwest = "0.11"` (in Cargo.toml) | HTTP client already present; no new crate needed |
| `BackendType` enum in `src/models/mod.rs` | Add one variant: `AzureFoundry` |
| `remote-backends` feature flag | `AzureFoundryBackend` slots under this existing flag |
| `wiremock = "0.5"` (dev-deps) | Mock HTTP server for deterministic integration tests |
| `GeneratorError::BackendUnavailable` | 401/403 auth failure mapping already typed |

### Competitor failure modes (research findings)

Three failure modes consistently appear in production Azure Foundry integrations
(source: field reports from Azure AI consulting teams, May 2026):

1. **Hardcoded `api-version` drift**: Code pinned to `2022-12-01` or similar; the
   version was retired; the break was silent until the endpoint returned 404.
   Teams discovered the bug in production, not in CI.

2. **Auth mode confusion**: API key header is `api-key`, not `Authorization: Bearer`.
   Developers copy OpenAI examples, use `Authorization: Bearer {key}`, and receive
   403 errors that look identical to permission problems. Hours lost.

3. **Deployment name ≠ model name**: vLLM/Ollama use a `model` field that matches
   the model's HuggingFace slug. Azure Foundry uses a `model` field that matches
   the *deployment name* the operator chose (e.g. `gpt-4o-prod-v2`, not `gpt-4o`).
   Config that works locally against Ollama silently generates nonsense responses
   when the deployment name is wrong on Azure.

This ADR solves all three by design (see Rationale).

---

## Decision

Add `AzureFoundryBackend` as a new file `src/backends/remote/azure_foundry.rs`,
following the `VllmBackend` pattern exactly. Ship under the existing
`remote-backends` feature flag. No new crates.

**Scope of v1:**

- New Foundry v1 URL format (`services.ai.azure.com/openai/v1/`)
- API key authentication only (`api-key` header)
- `api-version` as a mandatory config field with a default and a staleness warning
- Deterministic integration tests via `wiremock`

**Deferred to v2** (see Out-of-Scope):

- Microsoft Entra ID / `DefaultAzureCredential` auth
- Legacy per-deployment URL format
- Streaming (SSE) responses
- Azure-specific content filtering headers

---

## Architecture & Data Flow

```
caro "list all pods in the cluster"
         │
         ▼
  CommandGenerator::generate_command(&request)
         │
  AzureFoundryBackend
  ├── build_request_url()
  │     https://{resource}.services.ai.azure.com/openai/v1/chat/completions
  │     ?api-version={config.api_version}
  ├── build_headers()
  │     api-key: {config.api_key}
  │     Content-Type: application/json
  ├── build_body()
  │     {"model": "{config.deployment_name}", "messages": [...], ...}
  │     ^^ deployment name, not model slug ^^
  └── parse_response()  ← identical to VllmBackend (OpenAI-compatible shape)
         │
         ▼
  GeneratedCommand { command, explanation, ... }
         │
         ▼
  SafetyValidator (unchanged — no bypass)
```

The data path is a strict subset of `VllmBackend`. The only structural differences
are:
- URL hostname pattern
- `api-key` header instead of `Authorization: Bearer`
- `api-version` query parameter

---

## New Types

All new types live in `src/backends/remote/azure_foundry.rs`.

### `AzureFoundryConfig`

```rust
/// Configuration for the Azure Foundry inference backend.
///
/// The `api_version` field defaults to `DEFAULT_API_VERSION` but MUST be set
/// in user config so teams own the version pin and see staleness warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureFoundryConfig {
    /// Foundry resource name (the subdomain of services.ai.azure.com).
    /// Example: "my-company-ai" → "https://my-company-ai.services.ai.azure.com"
    pub resource_name: String,

    /// Deployment name as configured in Foundry. This is the value of the
    /// `model` field in every request body — NOT the underlying model slug.
    /// Example: "gpt-4o-prod-v2", "deepseek-v3-east"
    pub deployment_name: String,

    /// Azure OpenAI API version string. Format: "YYYY-MM-DD" or "YYYY-MM-DD-preview".
    /// Default: `DEFAULT_API_VERSION`.
    ///
    /// A staleness warning is logged at `WARN` level when this value is older
    /// than 18 months from the current date. This is the primary defence against
    /// silent version-drift failures.
    #[serde(default = "AzureFoundryConfig::default_api_version")]
    pub api_version: String,

    /// API key credential. Use Azure Foundry's "Keys and Endpoint" blade.
    ///
    /// Passed as `api-key` header, NOT `Authorization: Bearer`. This is a
    /// common copy-paste error from OpenAI examples — the field name matters.
    pub api_key: String,

    /// Optional embedded backend to fall back to when Foundry is unreachable.
    /// Mirrors the `embedded_fallback` field on `VllmBackend`.
    #[serde(skip)]
    pub embedded_fallback: Option<Arc<dyn CommandGenerator>>,
}

/// The most recent stable Azure OpenAI API version verified to support
/// chat completions. Update this constant when rotating version pins.
pub const DEFAULT_API_VERSION: &str = "2024-10-21";

/// If api_version is older than this many days, emit WARN at backend construction.
const STALENESS_THRESHOLD_DAYS: i64 = 548; // ~18 months
```

### `AzureFoundryBackend`

```rust
pub struct AzureFoundryBackend {
    config: AzureFoundryConfig,
    client: Client,
    endpoint: Url,  // built once at construction time; fails fast on bad config
}

impl AzureFoundryBackend {
    pub fn new(config: AzureFoundryConfig) -> Result<Self, GeneratorError> {
        // Build endpoint URL once; surface misconfigured resource_name early
        let endpoint = Url::parse(&format!(
            "https://{}.services.ai.azure.com/openai/v1/chat/completions",
            config.resource_name
        ))
        .map_err(|e| GeneratorError::ConfigError { message: e.to_string() })?;

        // Warn on stale api_version to prevent silent version-drift failures
        Self::warn_if_stale_api_version(&config.api_version);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GeneratorError::ConfigError { message: e.to_string() })?;

        Ok(Self { config, client, endpoint })
    }

    /// Emit WARN if the configured api_version date is > STALENESS_THRESHOLD_DAYS old.
    fn warn_if_stale_api_version(api_version: &str) {
        // Parse "YYYY-MM-DD" prefix from version strings like "2024-10-21-preview"
        if let Ok(date) = NaiveDate::parse_from_str(&api_version[..10], "%Y-%m-%d") {
            let age_days = (Local::now().date_naive() - date).num_days();
            if age_days > STALENESS_THRESHOLD_DAYS {
                tracing::warn!(
                    api_version = %api_version,
                    age_days = %age_days,
                    "Azure Foundry api_version is {} days old (threshold: {}). \
                     Check https://learn.microsoft.com/azure/foundry/openai/api-version-lifecycle \
                     for the current stable version.",
                    age_days, STALENESS_THRESHOLD_DAYS
                );
            }
        }
    }
}
```

---

## Files That Change

| File | Change |
|------|--------|
| `src/backends/remote/azure_foundry.rs` | **NEW** — `AzureFoundryConfig`, `AzureFoundryBackend`, impl `CommandGenerator` |
| `src/backends/remote/mod.rs` | Add `pub mod azure_foundry;` + `pub use azure_foundry::AzureFoundryBackend;` |
| `src/models/mod.rs` | Add `AzureFoundry` variant to `BackendType` enum; add `AzureFoundryConfig` to config structs |
| `tests/azure_foundry_backend_contract.rs` | **NEW** — 7 deterministic wiremock tests |
| `docs/adr/README.md` | Add ADR-018 row |

**NOT touched:**

- `Cargo.toml` — no new dependencies; `reqwest` already present
- `src/safety/` — no changes; the backend returns `GeneratedCommand`; safety validator is called by the existing pipeline
- `src/backends/mod.rs` — `CommandGenerator` trait and `BackendInfo` unchanged

---

## Output Contract

No new exit codes or JSON schema changes. `AzureFoundryBackend` implements the
`CommandGenerator` trait identically to `VllmBackend`. Existing callers see no
change.

The `BackendInfo` returned by `backend_info()`:

```rust
BackendInfo {
    backend_type: BackendType::AzureFoundry,
    model_name: self.config.deployment_name.clone(), // deployment name, not model slug
    supports_streaming: false,
    max_tokens: 800,
    typical_latency_ms: 1500,
    memory_usage_mb: 0, // external
    version: self.config.api_version.clone(),
}
```

---

## Integration Tests

`tests/azure_foundry_backend_contract.rs` — all deterministic via `wiremock`;
no live Azure connection required.

| Test | What it verifies | Expected outcome |
|------|-----------------|-----------------|
| `test_url_construction` | `resource_name = "myco-ai"` builds correct base URL | `https://myco-ai.services.ai.azure.com/openai/v1/chat/completions` |
| `test_api_version_query_param` | `api_version = "2024-10-21"` appears in query string | wiremock request matcher: `?api-version=2024-10-21` present |
| `test_api_key_header_not_bearer` | API key sent as `api-key` header | Request header `api-key: sk-test`, `Authorization` header absent |
| `test_deployment_in_model_field` | `deployment_name = "gpt-4o-prod"` → body `model` field | wiremock captures body; `$.model == "gpt-4o-prod"` |
| `test_parse_openai_compatible_response` | Standard OpenAI chat completions response parsed | Returns `GeneratedCommand { command: "kubectl get pods -A", .. }` |
| `test_staleness_warning_old_version` | `api_version = "2022-01-01"` triggers WARN log | `tracing_subscriber` test subscriber records one WARN containing `"api_version"` |
| `test_401_returns_backend_unavailable` | wiremock returns 401 | `Err(GeneratorError::BackendUnavailable { reason: "Authentication failed" })` |
| `test_fallback_on_server_error` | wiremock returns 503; fallback configured | Falls through to embedded backend; `backend_used` contains `"fallback"` |

---

## Competitive Differentiation

### What competitors get right (to replicate)

All Azure-aware LLM tools treat `deployment_name` as a first-class config field
separate from `model_name`. We replicate this: `AzureFoundryConfig` has
`deployment_name` and the config docs call out the distinction explicitly.

### Failure modes solved by design

| Failure mode | Competitor example | Caro solution |
|---|---|---|
| Hardcoded `api-version` drift | Hard-coded in source as `"2022-12-01"` | `api_version` is a named config field; `warn_if_stale_api_version()` fires at construction |
| `Authorization: Bearer` vs `api-key` | Copy-paste from OpenAI examples | Field is named `api_key`; comment in struct doc explicitly names the header; no ambiguity |
| Deployment name ≠ model name | Config `model: "gpt-4o"` but Azure deployment is `"gpt-4o-prod-v2"` | Config field is named `deployment_name`; `BackendInfo.model_name` echoes the deployment |

### What Caro can do that others cannot

- **Offline safety validation**: Azure-generated commands still flow through the
  52-pattern `SafetyValidator`. No Azure-native content filtering required.
- **Embedded fallback**: Enterprise users with unreliable Azure endpoints can
  configure `embedded_fallback` for zero-downtime degradation.
- **AGPL transparency**: No proprietary SDK — just `reqwest` and the public
  Microsoft Foundry REST API.
- **Subprocess-composable**: `caro --backend azure-foundry "..."` works in
  pipelines without any Azure SDK installed on the machine.

---

## Consequences

**Positive:**
- Closes the enterprise Azure gap. Enterprise Caro users can now use their
  existing Foundry deployments without workaround.
- Zero new production dependencies — `reqwest` is already present.
- Clear migration target for the 2026-08-26 Azure AI Inference SDK retirement.
- The staleness-warning mechanism prevents the most common long-term failure mode
  passively, without user action.

**Negative / risks:**
- `api-version = "2024-10-21"` will itself become stale. The constant
  `DEFAULT_API_VERSION` must be updated when a newer stable version ships.
  This is a maintenance obligation, mitigated by the warning mechanism.
- API key auth is less secure than Entra ID. Enterprise teams should be directed
  to v2 (Entra path) once implemented. Document this clearly.
- Rate limit and quota errors from Azure return HTTP 429. `GeneratorError`
  currently has no `RateLimited` variant. First occurrence: handle as
  `GenerationFailed`; add a proper variant in the next iteration.

---

## Alternatives Considered

### A: Wrap the `azure-openai` community Rust crate
Several community crates exist (`azure_ai_inference`, `openai_azure`, etc.). All
are unmaintained or cover only the legacy endpoint. Using one would introduce an
optional dep with unclear MSRV and maintenance risk. **Rejected** — `reqwest` +
the REST API is simpler, more stable, and follows the existing pattern.

### B: Add Azure Foundry as a vLLM configuration preset
Configure `VllmBackend` with an Azure URL + `api-key` header override.
- Pro: zero new code
- Con: `VllmBackend` uses `Authorization: Bearer`, not `api-key`; the URL pattern
  differs; `deployment_name` vs `model_name` semantics cannot be expressed;
  api-version staleness warning cannot be added without polluting `VllmBackend`.
  **Rejected** — the surface area differences justify a separate struct.

### C: Support legacy per-deployment URL as well
Add `endpoint_style: EndpointStyle { New, Legacy }` config field.
- Pro: covers teams that haven't migrated to the v1 endpoint
- Con: doubles the URL construction paths; the legacy endpoint is officially
  deprecated (migration guide exists); adds scope without clear user need.
  **Deferred** — add only if user reports confirm legacy is required.

---

## Implementation Checklist (for the engineer picking this up)

No build spike required — `reqwest` is already in `Cargo.toml` under
`[dependencies]`. The standard PR process applies.

**Feature PR:**
- [ ] `src/backends/remote/azure_foundry.rs` — `AzureFoundryConfig` + `AzureFoundryBackend` + impl `CommandGenerator`
- [ ] `src/backends/remote/mod.rs` — add `pub mod azure_foundry;` + re-export
- [ ] `src/models/mod.rs` — add `BackendType::AzureFoundry`; add `AzureFoundryConfig` to config struct
- [ ] `tests/azure_foundry_backend_contract.rs` — 8 wiremock tests per table above
- [ ] `cargo test --features remote-backends` passes
- [ ] `cargo clippy --features remote-backends -- -D warnings` clean
- [ ] `CHANGELOG.md` entry under `## [Unreleased]`
- [ ] User-facing docs: add `[backend.azure_foundry]` section to the config reference (explain `deployment_name` ≠ model)

---

## Validation Gate Status

Issue #661 is tagged "extends caro-core, exempt" in ROADMAP.md. This feature
adds a new inference backend — same class as Ollama, vLLM, Exo. Validation
gates per `.claude/rules/validation-discipline.md` do not apply.

---

## References

- Microsoft Foundry Endpoints documentation: https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints
- Azure OpenAI API version lifecycle: https://learn.microsoft.com/en-us/azure/foundry/openai/api-version-lifecycle
- Azure AI Inference beta SDK deprecation notice: https://learn.microsoft.com/en-us/azure/foundry/how-to/model-inference-to-openai-migration
- Field guide — Foundry REST APIs: https://team400.ai/blog/2026-05-microsoft-foundry-rest-apis-overview
- `src/backends/remote/vllm.rs` — reference implementation
- ROADMAP.md issue #661

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-05 | `caro-research--scoping-process` | Initial draft — autonomous research/scoping run |

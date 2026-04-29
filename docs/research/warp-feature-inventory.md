# Warp Feature Inventory — for Caro Terminal GUI

**Source:** [warpdotdev/warp](https://github.com/warpdotdev/warp) — open-sourced 2026-04-28.
**Tracking:** [#1009](https://github.com/wildcard/caro/issues/1009) under epic [#1008](https://github.com/wildcard/caro/issues/1008).
**Snapshot date:** 2026-04-28
**Method:** Ralph-loop research pass — one row per crate. This file is the inventory artifact; updates land via incremental commits.

---

## Workspace summary

| Stat | Value |
|---|---|
| Workspace crates | 65 (under `crates/`) |
| Top-level binary | `app/` (the Warp client) |
| Language | ~98% Rust, with Objective-C/Swift in `app/` for macOS-specific bits, Python/Shell for build scripts |
| License | Dual: **MIT** for `warpui` + `warpui_core` (UI framework) — **AGPL-3.0** for everything else |
| Build | Cargo workspace, `resolver = "2"`, `rust-toolchain.toml` pinned |
| VT engine | Their own (not Ghostty) |
| Sponsor note | "OpenAI is the founding sponsor of the new, open-source Warp repository" |

## Feature buckets (taxonomy)

We group crates by which of the original 12 feature buckets they serve. **In-scope-for-v0.1?** is our cut for Caro Terminal GUI v0.1 (per epic [#1008](https://github.com/wildcard/caro/issues/1008)).

| Bucket | Caro v0.1? | Notes |
|---|---|---|
| 1. Block-based UX | ✅ | Headline feature; our differentiator |
| 2. AI integration | ✅ | We use caro-core, not warp's `ai` crate |
| 3. Workflows | ❌ v0.2 | Need UI design |
| 4. Command palette | ✅ | Cheap win |
| 5. Editor-grade input | ⚠️ partial | Multi-line + history; deep editor (vim, LSP) is v0.2+ |
| 6. Sharing | ⚠️ minimal | Block-as-text copy only; URL share is v0.3+ |
| 7. Themes | ✅ | 3 built-ins |
| 8. Splits/tabs/panes | ✅ | Tabs + 1 horizontal split per tab |
| 9. GPU rendering | ❌ deferred | WebView's compositor is fine for v0.1 |
| 10. Settings sync | ❌ v0.3+ | Account model required |
| 11. SSH / remote | ❌ v0.2 | Orthogonal risk |
| 12. Notebooks | ❌ v0.3+ | UX-heavy |

## Per-crate inventory

Complexity tiers: **S** = small (1–3 days), **M** = medium (1–2 weeks), **L** = large (3–6 weeks), **XL** = epic-scale.
"In-scope" means: we *adapt* this crate's idea/code into caro-terminal v0.1. "—" means out of scope.

| # | Crate | Bucket | Lang | Complexity | License | In v0.1? | Notes |
|---|---|---|---|---|---|---|---|
| 1 | `ai` | 2. AI integration | Rust | L | AGPL | — | Warp's AI router (LLM backends, tool-calling). We use **caro-core** instead — already solves NL→cmd + safety. Read for prompt-design ideas only. |
| 2 | `app-installation-detection` | (utility) | Rust | S | AGPL | — | Detects which apps the user has installed. Belongs to **Turi**, not caro-terminal. |
| 3 | `asset_cache` | (utility) | Rust | S | AGPL | maybe | Asset (images/icons) caching. Tauri provides similar via `tauri-plugin-fs`. |
| 4 | `asset_macro` | (utility) | Rust | S | AGPL | — | Compile-time asset embedding macro. Tauri's `include_dir!` is sufficient. |
| 5 | `channel_versions` | (release) | Rust | S | AGPL | — | Stable/beta/dev channel version handling. Caro releases via crates.io; not needed. |
| 6 | `command` | (utility) | Rust | S | AGPL | ✅ | Wraps `std::process::Command` to suppress Windows console flash. **Adapt verbatim** — same problem on Windows for our PTY child processes. |
| 7 | `command-signatures-v2` | 1. Block UX / shell-integration | Rust | M | AGPL | ✅ | Likely the OSC 133 / shell-integration parser side. **Read deeply** for our block parser ([#1015](https://github.com/wildcard/caro/issues/1015)). |
| 8 | `computer_use` | 2. AI integration | Rust | L | AGPL | — | Anthropic-style "computer use" tool. Defer to v0.3+; not a v0.1 caro feature. |
| 9 | `editor` (`warp_editor`) | 5. Editor input | Rust | XL | AGPL | ⚠️ partial | Their full code-editor (multi-cursor, LSP, syntax tree). For v0.1 we want only the *input line* — a thin React `<input>` with multi-line + history is enough. |
| 10 | `field_mask` | (utility) | Rust | S | AGPL | — | Probably gRPC-style FieldMask helper. Not needed. |
| 11 | `firebase` | 10. Settings sync / 6. Sharing | Rust | M | AGPL | — | Cloud backend client. Pure server-coupled; out of v0.1. |
| 12 | `fuzzy_match` | 4. Command palette | Rust | S | AGPL | ✅ | Fuzzy + glob matching. **Adapt** for command-palette ([#1019](https://github.com/wildcard/caro/issues/1019)) — saves us picking between fuse.js/fuzzaldrin. |
| 13 | `graphql` (`warp_graphql`) | (network) | Rust | M | AGPL | — | GraphQL client for their backend. Out of scope. |
| 14 | `handlebars` | (utility) | Rust | S | AGPL | — | Their fork/wrapper of handlebars. Tera/standard handlebars is fine if we ever need templating. |
| 15 | `http_client` | (network) | Rust | S | AGPL | — | Their HTTP client wrapper. `reqwest` direct is fine. |
| 16 | `http_server` | (network) | Rust | M | AGPL | — | Used by `remote_server`. Not a v0.1 thing. |
| 17 | `input_classifier` | 2. AI integration | Rust | M | AGPL | maybe | Likely classifies user input as command vs natural-language. **Worth a read** for our AI prompt bar — could inform the "should this be Cmd+J?" UX. |
| 18 | `integration` | (test) | Rust | — | AGPL | — | Test crate. |
| 19 | `ipc` | (utility) | Rust | M | AGPL | ✅ | Typed IPC request/response protocol. **Adapt** for caro-terminal ↔ Turi JSON-RPC ([#1022](https://github.com/wildcard/caro/issues/1022)). |
| 20 | `isolation_platform` | (utility) | Rust | M | AGPL | — | Likely sandboxing/security. Out of v0.1. |
| 21 | `jsonrpc` | (utility) | Rust | S | AGPL | ✅ | JSON-RPC implementation. **Adapt or use upstream `jsonrpsee`** for caro-terminal ↔ Turi. |
| 22 | `languages` | 5. Editor / syntax | Rust | L | AGPL | — | Tree-sitter language registrations. Not needed for v0.1 (we don't highlight code in the prompt). |
| 23 | `lsp` | 5. Editor | Rust | XL | AGPL | — | LSP client. v0.2+ when we add a real input editor. |
| 24 | `managed_secrets` | (security) | Rust | M | AGPL | — | Encrypted secret storage. Defer; OS keyring crates are sufficient if we ever need this. |
| 25 | `managed_secrets_wasm` | (security) | Rust | S | AGPL | — | WASM build of above. — |
| 26 | `markdown_parser` | (utility) | Rust | S | AGPL | maybe | Renders markdown in AI bar responses. **Adapt** if we want rich AI explanations; otherwise `pulldown-cmark` direct. |
| 27 | `natural_language_detection` | 2. AI integration | Rust | S | AGPL | — | Detects "is this NL or a command?" Caro has its own classification path. |
| 28 | `node_runtime` | 5. Editor / LSP | Rust | M | AGPL | — | Manages a Node runtime (probably for LSP servers like TS). v0.2+. |
| 29 | `onboarding` | UX | Rust | M | AGPL | — | First-run flow. We'll write our own simpler version. |
| 30 | `persistence` | 4. Cmd palette / history | Rust | M | AGPL | ✅ | SQLite-based persistence (history, sessions). **Adapt** for command-palette history ([#1019](https://github.com/wildcard/caro/issues/1019)). |
| 31 | `prevent_sleep` | (utility) | Rust | S | AGPL | maybe | Wakelock when long commands run. Nice polish; v0.2. |
| 32 | `remote_server` | 11. SSH / remote | Rust | XL | AGPL | — | Remote dev server. v0.2+. |
| 33 | `repo_metadata` | UX | Rust | M | AGPL | maybe | Detects git branch / repo state — Warp shows this in its tab title. **Nice-to-have** for tabs ([#1020](https://github.com/wildcard/caro/issues/1020)). |
| 34 | `serve-wasm` | (build helper) | Rust | — | AGPL | — | Helper for serving WASM. Not in default-members. |
| 35 | `settings` | (utility) | Rust | M | AGPL | maybe | Settings infrastructure. We'll likely use `serde` + a JSON file directly, simpler. |
| 36 | `settings_value` | (utility) | Rust | S | AGPL | — | Settings value types. Keep simple in v0.1. |
| 37 | `settings_value_derive` | (utility) | Rust | S | AGPL | — | Derive macro. — |
| 38 | `simple_logger` | (utility) | Rust | S | AGPL | — | Logger. We use `tracing` already. |
| 39 | `string-offset` | 5. Editor | Rust | S | AGPL | — | UTF-16 ↔ UTF-8 offset conversion. v0.2+. |
| 40 | `sum_tree` | (data structure) | Rust | M | AGPL | — | Augmented B-tree (familiar from Zed). Belongs to the editor; not v0.1. |
| 41 | `syntax_tree` | 5. Editor | Rust | L | AGPL | — | Tree-sitter wrapper. Not v0.1. |
| 42 | `ui_components` | 7. Themes / chrome | Rust | L | MIT? | maybe | Reusable UI widgets. We use React; not directly portable. Read for design ideas. |
| 43 | `vim` | 5. Editor | Rust | XL | AGPL | — | Vim mode for the editor. v0.3+ if at all. |
| 44 | `virtual_fs` | (utility) | Rust | M | AGPL | — | Virtual filesystem abstraction. Not needed. |
| 45 | `voice_input` | 2. AI integration | Rust | L | AGPL | — | Speech-to-text. Defer to v0.3+. |
| 46 | `warp_cli` | (utility) | Rust | S | AGPL | — | Warp's `clap` setup. We have our own. |
| 47 | `warp_completer` | 5. Editor / 4. Palette | Rust | M | AGPL | maybe | Inline completions. **Read** for AI prompt bar autocomplete UX. |
| 48 | `warp_core` | (core types) | Rust | L | AGPL | — | Their core domain types. Caro has its own. |
| 49 | `warp_features` | (feature flags) | Rust | S | AGPL | — | Feature-flag system. Not needed in v0.1. |
| 50 | `warp_files` | (utility) | Rust | M | AGPL | — | Files / paths. Standard Rust is fine. |
| 51 | `warp_graphql_schema` | (network) | Rust | M | AGPL | — | GraphQL schema. — |
| 52 | `warp_js` | 5. Editor / scripting | Rust | L | AGPL | — | JS runtime embedding (likely for extensions/scripts). v0.3+. |
| 53 | `warp_logging` | (utility) | Rust | S | AGPL | — | Logging wrapper. — |
| 54 | `warp_ripgrep` | 4. Search | Rust | M | AGPL | maybe | Ripgrep-as-a-library wrapper. **Adapt** if we add "search inside blocks" in v0.2. |
| 55 | `warp_server_client` | (network) | Rust | M | AGPL | — | Backend RPC client. — |
| 56 | `warp_terminal` | **1. Block UX + VT** | Rust | XL | AGPL | ✅ | **Their VT engine + block model.** This is the deepest read of the inventory: we need to understand how they emit/group blocks. Map *their concepts* onto our libghostty-vt + OSC 133 implementation. **Do not** copy the VT — we use libghostty-vt. |
| 57 | `warp_util` | (utility) | Rust | M | AGPL | — | Generic utilities. Cherry-pick as needed. |
| 58 | `warp_web_event_bus` | (network) | Rust | M | AGPL | — | Event bus to web. — |
| 59 | `warpui` | 7. UI framework | Rust | XL | **MIT** | — | Warp's UI framework. **MIT-licensed, more permissive.** We use React/Tauri; not directly applicable, but read for component patterns. |
| 60 | `warpui_core` | 7. UI framework | Rust | XL | **MIT** | — | UI framework core. Same as above. |
| 61 | `warpui_extras` | 7. UI framework | Rust | M | MIT? | — | Optional extras. — |
| 62 | `watcher` | (utility) | Rust | S | AGPL | maybe | File-system watcher. Useful for theme hot-reload ([#1021](https://github.com/wildcard/caro/issues/1021)) — but `notify` crate direct is also fine. |
| 63 | `websocket` | (network) | Rust | S | AGPL | — | WebSocket client. — |
| 64 | `app/` | UX shell | Rust + Obj-C | XL | AGPL | — | The app shell binary. Mac-specific bits in `DockTilePlugin`. We use Tauri; not directly portable. |

## Top deep-reads for caro-terminal v0.1

In priority order, these are the crates we should read end-to-end (vs glance):

1. **`warp_terminal`** — block model + VT integration patterns. Inform [#1015](https://github.com/wildcard/caro/issues/1015) (block parser).
2. **`command-signatures-v2`** — likely their shell-integration / OSC 133 parser. Direct precedent for [#1015](https://github.com/wildcard/caro/issues/1015) and [#1016](https://github.com/wildcard/caro/issues/1016).
3. **`fuzzy_match`** — algorithm + UX for command palette. Inform [#1019](https://github.com/wildcard/caro/issues/1019).
4. **`persistence`** — schema for command history + session persistence. Inform [#1019](https://github.com/wildcard/caro/issues/1019) (history) and [#1020](https://github.com/wildcard/caro/issues/1020) (tab restore).
5. **`ipc` + `jsonrpc`** — wire format for caro-terminal ↔ Turi. Inform [#1022](https://github.com/wildcard/caro/issues/1022).
6. **`input_classifier`** — heuristics for "is this a command or a question?". Could remove the need for the user to press Cmd+J explicitly.
7. **`warp_completer`** — inline completion UX patterns. Inform AI prompt bar ([#1018](https://github.com/wildcard/caro/issues/1018)).
8. **`command`** — the Windows-no-flash hack. Adapt verbatim into our PTY layer ([#1013](https://github.com/wildcard/caro/issues/1013)).

## Open questions — to be answered in subsequent passes

These are things this initial pass *guessed* at and need verification:

- [ ] `command-signatures-v2` — is it actually the OSC 133 parser, or a separate command-introspection system?
- [ ] `warp_terminal` — does it own the VT engine, or is the VT engine in `app/`? What's the API surface?
- [ ] `input_classifier` — what classifier is it? ML model embedded in the binary? Heuristic regex?
- [ ] `warpui` MIT vs `warpui_core` MIT — are *all* UI subcrates MIT, or just these two? Audit `Cargo.toml` license fields per crate.
- [ ] How does Warp emit blocks for shells the user did NOT instrument? (i.e., is there a fallback heuristic?)
- [ ] AI agent loop — what's the prompt template + tool schema in `crates/ai/`?

## Updated v0.1 cut (post-inventory)

The initial cut in epic [#1008](https://github.com/wildcard/caro/issues/1008) holds. Two refinements based on this inventory:

1. **Adapt `command` (crate #6) verbatim** for our PTY layer ([#1013](https://github.com/wildcard/caro/issues/1013)) — saves us a Windows-specific bug.
2. **Adapt `fuzzy_match` (crate #12) and `persistence` (crate #30)** as direct dependencies if their licenses + APIs allow — both AGPL-3.0, which is compatible with caro (also AGPL-3.0). This shrinks [#1019](https://github.com/wildcard/caro/issues/1019) from M to S.

License note: AGPL-3.0 in caro means we can vendor or depend on AGPL-3.0 crates from warp without re-licensing. The MIT-licensed `warpui*` crates are also compatible (MIT into AGPL = AGPL).

---

## Pass log

| Pass | Date | Crates inventoried | Notes |
|---|---|---|---|
| 1 | 2026-04-28 | All 65 crates classified at metadata-level | First pass — descriptions inferred from name + brief `lib.rs` doc-comment. Deep reads pending in passes 2–9. |

Subsequent passes (one per "deep-read" crate above) will append rows here and refine the table.

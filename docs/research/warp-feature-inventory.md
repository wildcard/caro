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
| 1 | `ai` | 2. AI integration | Rust | XL | AGPL | — | **Pass 9 done.** Multi-provider AI agent system: `agent/`, `skills/`, `project_context/`, `diff_validation/`, `index/`. Uses **`rmcp`** (Rust MCP client) + `computer_use`. Has a **skills/ subsystem** parsing YAML-frontmatter skill files — same shape as caro's existing skills. **Reference for caro v0.2+** if we add MCP / multi-turn AI in the terminal. |
| 2 | `app-installation-detection` | (utility) | Rust | S | AGPL | — | Detects which apps the user has installed. Belongs to **Turi**, not caro-terminal. |
| 3 | `asset_cache` | (utility) | Rust | S | AGPL | maybe | Asset (images/icons) caching. Tauri provides similar via `tauri-plugin-fs`. |
| 4 | `asset_macro` | (utility) | Rust | S | AGPL | — | Compile-time asset embedding macro. Tauri's `include_dir!` is sufficient. |
| 5 | `channel_versions` | (release) | Rust | S | AGPL | — | Stable/beta/dev channel version handling. Caro releases via crates.io; not needed. |
| 6 | `command` | (utility) | Rust | S | AGPL | ✅ | **Pass 8 done.** Tiny crate (~couple hundred LOC). Provides `blocking::Command` + `r#async::Command` drop-ins that auto-apply Windows `no_window` flag. **Vendor verbatim** into caro-terminal with NOTICE.md entry. |
| 7 | `command-signatures-v2` | 4. Cmd palette / 5. Editor input | Rust + JS | M | AGPL | maybe | **Pass 2 correction:** NOT the OSC 133 parser. It's a `rust-embed` of a JS bundle (`js/build/`) used for command-line syntax analysis (powering `warp_completer` and inline AI suggestions). v0.2+. |
| 8 | `computer_use` | 2. AI integration | Rust | L | AGPL | — | Anthropic-style "computer use" tool. Defer to v0.3+; not a v0.1 caro feature. |
| 9 | `editor` (`warp_editor`) | 5. Editor input | Rust | XL | AGPL | ⚠️ partial | Their full code-editor (multi-cursor, LSP, syntax tree). For v0.1 we want only the *input line* — a thin React `<input>` with multi-line + history is enough. |
| 10 | `field_mask` | (utility) | Rust | S | AGPL | — | Probably gRPC-style FieldMask helper. Not needed. |
| 11 | `firebase` | 10. Settings sync / 6. Sharing | Rust | M | AGPL | — | Cloud backend client. Pure server-coupled; out of v0.1. |
| 12 | `fuzzy_match` | 4. Command palette | Rust | S | AGPL | depend on parent | **Pass 3 done.** Two-file wrapper around upstream `fuzzy-matcher = "0.3.7"` (skim's algorithm) + wildcard glob support for paths. **Don't vendor — depend directly on upstream `fuzzy-matcher`** for command-palette ([#1019](https://github.com/wildcard/caro/issues/1019)). |
| 13 | `graphql` (`warp_graphql`) | (network) | Rust | M | AGPL | — | GraphQL client for their backend. Out of scope. |
| 14 | `handlebars` | (utility) | Rust | S | AGPL | — | Their fork/wrapper of handlebars. Tera/standard handlebars is fine if we ever need templating. |
| 15 | `http_client` | (network) | Rust | S | AGPL | — | Their HTTP client wrapper. `reqwest` direct is fine. |
| 16 | `http_server` | (network) | Rust | M | AGPL | — | Used by `remote_server`. Not a v0.1 thing. |
| 17 | `input_classifier` | 2. AI integration | Rust | L | AGPL | — | **Pass 6 done.** **Three** plumbed-through-trait backends: `HeuristicClassifier`, `FasttextClassifier`, `OnnxClassifier` (with two runtime libs: `candle` + `ort`). Output: `(p_shell, p_ai)`. **Verdict: skip in v0.1** — explicit Cmd+J replaces auto-detection, and ONNX models would balloon binary size. v0.2+ heuristic only if needed. |
| 18 | `integration` | (test) | Rust | — | AGPL | — | Test crate. |
| 19 | `ipc` | (utility) | Rust | L | AGPL | — | **Pass 5 done.** Bincode-over-`interprocess` (UDS + Windows named pipes). Typed `Service`/`ServiceCaller`. **Verdict: don't vendor — bincode hurts debuggability for Turi. Use `jsonrpc` shape (#21) over `interprocess` transport (the same crate they depend on).** |
| 20 | `isolation_platform` | (utility) | Rust | M | AGPL | — | Likely sandboxing/security. Out of v0.1. |
| 21 | `jsonrpc` | (utility) | Rust | S | AGPL | use upstream | **Pass 5 done.** Tiny JSON-RPC 2.0 implementation (4 files). **Verdict: don't vendor — use upstream `jsonrpsee` (more maintained ecosystem) for Turi RPC. Reference warp's `JsonRpcService` shape for design.** |
| 22 | `languages` | 5. Editor / syntax | Rust | L | AGPL | — | Tree-sitter language registrations. Not needed for v0.1 (we don't highlight code in the prompt). |
| 23 | `lsp` | 5. Editor | Rust | XL | AGPL | — | LSP client. v0.2+ when we add a real input editor. |
| 24 | `managed_secrets` | (security) | Rust | M | AGPL | — | Encrypted secret storage. Defer; OS keyring crates are sufficient if we ever need this. |
| 25 | `managed_secrets_wasm` | (security) | Rust | S | AGPL | — | WASM build of above. — |
| 26 | `markdown_parser` | (utility) | Rust | S | AGPL | maybe | Renders markdown in AI bar responses. **Adapt** if we want rich AI explanations; otherwise `pulldown-cmark` direct. |
| 27 | `natural_language_detection` | 2. AI integration | Rust | S | AGPL | — | Detects "is this NL or a command?" Caro has its own classification path. |
| 28 | `node_runtime` | 5. Editor / LSP | Rust | M | AGPL | — | Manages a Node runtime (probably for LSP servers like TS). v0.2+. |
| 29 | `onboarding` | UX | Rust | M | AGPL | — | First-run flow. We'll write our own simpler version. |
| 30 | `persistence` | 4. Cmd palette / history | Rust | L | AGPL | adopt stack | **Pass 4 done.** `diesel` ORM + `diesel_migrations` + 19+ migration dirs going back to 2021-12. Schema is too warp-specific (AI blocks, cloud refresh). **Verdict: adopt the stack, write narrow caro-terminal schema (commands_history, sessions, tabs).** |
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
| 47 | `warp_completer` | 5. Editor / 4. Palette | Rust | XL | AGPL | — | **Pass 7 done.** Multi-module subsystem (`completer/`, `parsers/`, `signatures/`, `meta`). Depends on `fuzzy_match` + likely consumes `command-signatures-v2` JS for command parsing. **Verdict: out of v0.1 — too large.** v0.2+ defer to shell-native completion (zsh-autosuggestions, fish builtins). |
| 48 | `warp_core` | (core types) | Rust | L | AGPL | — | Their core domain types. Caro has its own. |
| 49 | `warp_features` | (feature flags) | Rust | S | AGPL | — | Feature-flag system. Not needed in v0.1. |
| 50 | `warp_files` | (utility) | Rust | M | AGPL | — | Files / paths. Standard Rust is fine. |
| 51 | `warp_graphql_schema` | (network) | Rust | M | AGPL | — | GraphQL schema. — |
| 52 | `warp_js` | 5. Editor / scripting | Rust | L | AGPL | — | JS runtime embedding (likely for extensions/scripts). v0.3+. |
| 53 | `warp_logging` | (utility) | Rust | S | AGPL | — | Logging wrapper. — |
| 54 | `warp_ripgrep` | 4. Search | Rust | M | AGPL | maybe | Ripgrep-as-a-library wrapper. **Adapt** if we add "search inside blocks" in v0.2. |
| 55 | `warp_server_client` | (network) | Rust | M | AGPL | — | Backend RPC client. — |
| 56 | `warp_terminal` | **1. Block UX + terminal model** | Rust | XL | AGPL | ✅ | **Pass 2 done.** Builds on the **`vte` crate** (external, mature) + their own `model::grid::*` cell/row types + block model (`BlockId`, `BlockIndex`). OSC 133 parser at `model/ansi/control_sequence_parameters.rs:637`. Block IDs come from shell precmd, not from the terminal. **See "Pass 2 deep-read" section below for full details.** |
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

1. **`warp_terminal`** ✅ done in pass 2 — block model + VT integration patterns. Inform [#1015](https://github.com/wildcard/caro/issues/1015) (block parser).
2. ~~`command-signatures-v2`~~ — *demoted by pass 2: it's a JS bundle for command-line syntax, not the OSC 133 parser.* The OSC 133 parser is inside `warp_terminal`.
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
| 2 | 2026-04-30 | `warp_terminal` deep-read + bootstrap scripts + correction to `command-signatures-v2` | See "Pass 2 deep-read" section. |
| 3 | 2026-04-30 | `fuzzy_match` | Tiny wrapper around `fuzzy-matcher = "0.3.7"` (skim's algorithm). Adds wildcard support (`*.rs`, `src/*`). **Verdict: don't vendor — depend on `fuzzy-matcher` directly.** |
| 4 | 2026-04-30 | `persistence` | Uses **`diesel`** (Rust ORM) + `diesel_migrations`. 19+ migration dirs going back to 2021-12. Schema is too warp-specific to vendor. **Verdict: adopt `diesel` as the ORM, write our own narrow schema (commands history, sessions, blocks).** |
| 5 | 2026-04-30 | `ipc` + `jsonrpc` | `ipc` uses `interprocess` (UDS + Windows named pipes — same transport we planned for Turi) + `bincode` serialization. `jsonrpc` is small, separate, JSON-RPC 2.0. **Verdict: for Turi RPC, use `jsonrpc` (debuggable) + `interprocess` (transport). Don't vendor `ipc` — bincode is too opaque.** |
| 6 | 2026-04-30 | `input_classifier` | **3 backends**: heuristic (regex), fasttext (embeddings), **ONNX** (candle or ort runtime). Trait `InputClassifier::detect_input_type → (p_shell, p_ai)`. **Verdict: v0.1 ships heuristic only — no ML model bundled. v0.2+ may add ONNX with a downloadable model.** |
| 7 | 2026-04-30 | `warp_completer` | Inline command completion subsystem. Uses `fuzzy_match` + `parsers` + `signatures`. **Verdict: out of v0.1 — too large.** |
| 8 | 2026-04-30 | `command` | Tiny: Windows `no_window` flag drop-in for `std::process::Command` and `async-process::Command`. **Verdict: vendor verbatim into caro-terminal. Saves a real Windows bug.** |
| 9 | 2026-04-30 | `ai` (bonus) | Large agent/skill/index/diff_validation system. Uses **rmcp** (Rust MCP) → warp talks to MCP servers. Has a `skills/` parser that mirrors Claude Code skills. **Verdict: out of v0.1 (caro-core covers NL→cmd). The `skills/` module is interesting for caro v0.2 if we want skill-loading in the AI bar.** |

---

## Pass 2 deep-read — `warp_terminal` + shell bootstrap

### Architectural correction to pass 1

| Pass-1 guess | Pass-2 reality |
|---|---|
| `command-signatures-v2` is the OSC 133 parser | **Wrong.** It's a `rust-embed` of a JS bundle (`js/build/`) used for command-line syntax analysis (likely powering inline AI completions / `warp_completer`). The OSC 133 parser lives in `warp_terminal/src/model/ansi/control_sequence_parameters.rs:637`. |
| Warp wrote its own VT parser from scratch | **Wrong.** They depend on the well-known **`vte`** crate (same crate alacritty/wezterm use) and build their *terminal model* on top of `vte`'s parser callbacks. |
| Block boundaries come from OSC 133 alone | **Partially wrong.** Warp uses a **two-channel protocol**: standard OSC 133 (interoperable) + their own OSC 9278/9277/9279 (Warp-private, JSON-over-DCS-or-OSC). |

### How Warp actually does block-based UX

```
                    PTY bytes
                       │
                       ▼
                ┌──────────────┐
                │  vte parser  │  (Rust crate, mature)
                └──────┬───────┘
                       │ callbacks
                       ▼
        ┌──────────────────────────────┐
        │  warp_terminal terminal model│
        │  - grid (cells, rows)        │
        │  - block list (BlockId, idx) │
        │  - escape_seq dispatch       │
        └──────────────┬───────────────┘
                       │
       ┌───────────────┼────────────────┐
       │               │                │
       ▼               ▼                ▼
  OSC 133            OSC 9278         OSC 9277
  prompt             warp JSON        in-band cmd
  markers            messages         output
  (interop)          (warp-only)      (warp-only)
```

**OSC 133 (interoperable, public spec):**
- `\e]133;A\a` — prompt start (`PromptKind::Initial`)
- `\e]133;B\a` — prompt end / command-input start
- `\e]133;P;k=r\a` — right-side prompt (Warp extension; spec-compatible)
- **NOT used by Warp:** `OSC 133 ; C` (command-start), `OSC 133 ; D ; exit` (output-end + exit). Warp uses OSC 9278 JSON instead.

**OSC 9278 (Warp-private, JSON-encoded):**
- Carries `{"hook": "CommandFinished", "value": {"exit_code": N, "next_block_id": "precmd-$SESSION-$N"}}` and similar.
- Encoded as either `\e]9278;<hex-encoded JSON>\a` (OSC) or `\eP$d<hex>\x9c` (DCS). DCS is preferred but can't be used on Windows ConPTY.

**OSC 9277 (Warp-private, in-band command output):**
- Wraps stdout of "in-band commands" — commands the *terminal* sends *to* the shell to execute (think AI side-quests). Bracketed by `\e]9277;A\a` ... `\e]9277;B\a`.

**OSC 9279 (Warp-private, grid reset):**
- `\e]9279\a` — clears the rendering grid at block boundaries.

### Block ID generation

Block IDs come from the **shell precmd hook**, not from the terminal:

- **Format:** `precmd-{WARP_SESSION_ID}-{monotonic counter}` (cheap)
- **Or:** `manual-{UUID}` for blocks created in-app (e.g. AI-generated)
- **Why not UUIDs in the precmd?** Quote: *"It is expensive to create a UUID in the bootstrap script."* (from `block_id.rs`). Bootstrap script overhead matters because it runs on every prompt cycle.

### Bootstrap scripts (`app/assets/bundled/bootstrap/`)

| Shell | File | LoC | Hook framework |
|---|---|---|---|
| Bash | `bash_body.sh` | 700+ | `bash-preexec` (vendored) |
| Zsh | `zsh_body.sh` | 600+ | Native `preexec_functions` / `precmd_functions` |
| Fish | `fish.sh` | 700+ | Native `--on-event fish_preexec` / `fish_prompt` |

**Critical compatibility patterns we should adopt:**
1. **p10k coexistence:** Zsh script specifically detects powerlevel10k and preserves its precmd functions. Without this, p10k breaks.
2. **`HISTCONTROL=ignorespace`** trick: prefix internal commands with a space so they don't pollute history. Unset after bootstrap.
3. **Generator commands:** Mechanism for the terminal to ask the shell to run a side command (`OSC 9277`). Ours can skip this for v0.1.
4. **Windows ConPTY transport switch:** Detect `WARP_USING_WINDOWS_CON_PTY` and use OSC instead of DCS — DCS gets mangled.
5. **bash-preexec dependency:** Bash has no native preexec, so they vendor `bash-preexec`. Apache-2.0 licensed. AGPL-compatible.

### Decisions for caro-terminal v0.1

These are concrete updates to the existing issues based on pass-2 findings:

| Issue | Decision | Reason |
|---|---|---|
| **Architecture** | **Use `vte` crate instead of (or alongside) libghostty-vt** | `vte` is mature, pure-Rust, well-tested by alacritty/wezterm. libghostty-vt is newer and Zig-based. **Spike (#1010) should benchmark both.** |
| **#1015 block parser** | Support standard **OSC 133 A/B/C/D** (interoperable with iTerm2/wezterm/ghostty users) | Warp's narrower set is fine for warp-only, but caro-terminal benefits from working with any user's existing OSC-133-emitting shell. |
| **#1016 shell snippets** | Emit standard OSC 133 (A/B/C/D + exit code in D), **NOT** Warp's OSC 9278 | Keeps our snippets ~50 lines each instead of 700, and interoperable. Caro doesn't need warp's RPC — for AI side-commands we'll PTY-spawn a sub-shell. |
| **#1016 shell snippets** | **Must coexist with p10k, starship, oh-my-zsh** | Wrap user's existing precmd/PROMPT_COMMAND chain instead of overwriting. Detect known frameworks and preserve their hooks. |
| **#1015 block parser** | Block ID = monotonic `$SESSION-$N` from shell, OR `manual-{uuid}` | Same scheme as Warp. Avoid UUIDs in the precmd hot path. |
| **#1010 spike** | **Compare `vte` (Rust) vs `libghostty-vt` (Zig FFI)** as alternatives | Pass-1 assumed libghostty-vt is the only option. `vte` may be simpler. Spike should produce a one-page comparison with: build complexity, parsing throughput, screen-state API, license, maintainer activity. |
| **#1018 AI prompt bar** | Don't try to reimplement OSC 9278 / generator-commands | Warp uses these for tight terminal↔AI loop. Caro's NL→cmd is "type, generate, insert" — no in-band feedback. v0.3+ if ever. |

### Open questions (updated)

Closed by pass 2:
- ~~Is `command-signatures-v2` the OSC 133 parser?~~ **No, it's an embedded JS bundle for command-line syntax.**
- ~~Where is the VT engine?~~ **It's `vte` crate (external) + `warp_terminal` model (their own).**
- ~~Block-emit fallback for un-instrumented shells?~~ **No fallback — Warp's bootstrap runs on shell init; if not present, no blocks. Caro should match this behavior in v0.1.**

Closed by passes 3–9:
- ~~`input_classifier` — heuristic or ML?~~ **All three (heuristic + fasttext + ONNX). Trait-based with probabilistic output. Caro v0.1 ships heuristic only.**
- ~~AI agent loop in `crates/ai/`?~~ **Multi-provider, MCP-aware (`rmcp`), has a `skills/` parser mirroring Claude Code skills. Out of v0.1 scope.**
- ~~Does `persistence` use diesel?~~ **Yes — diesel + diesel_migrations + 19 migration dirs. We adopt diesel-the-tool, write our own schema.**
- ~~`ipc` vs `jsonrpc`?~~ **Two different things. `ipc` = bincode-over-UDS/named-pipes for typed services. `jsonrpc` = JSON-RPC 2.0 (small, focused). We'll use `jsonrpc` for Turi.**

Still open (lower priority):
- [ ] How does `command-signatures-v2`'s JS module get invoked? V8? Deno? Lazy WASM? (Doesn't affect v0.1.)
- [ ] Are *all* UI subcrates MIT, or just `warpui` + `warpui_core`? (We use Tauri/React; doesn't affect v0.1.)

---

## Pass 3–9 deep-reads (synthesized)

### Pass 3 — `fuzzy_match`

Two-file crate. Wraps `fuzzy-matcher = "0.3.7"` (skim's well-known algorithm) and adds wildcard glob support optimized for file paths (`*.rs`, `src/*`).

**Decision for [#1019](https://github.com/wildcard/caro/issues/1019):** Don't vendor `fuzzy_match` — directly depend on the upstream `fuzzy-matcher` crate ourselves. Wildcard support is nice-to-have but not v0.1 (we're matching command history strings, not file paths). This shrinks the dep surface vs vendoring warp's wrapper.

### Pass 4 — `persistence`

Stack: `diesel` ORM + `diesel_migrations` for SQL schema migrations. SQLite, edition 2024.

Migration history (selected): `add_ps1_to_restored_block` (2022-02), `add_active_pane` (2021-12), `add_cloud_refresh_table` (2023-09), `create_ignored_suggestions_table` (2025-08), `remove_persisted_ai_blocks` (2025-11). 19+ migration dirs total.

The schema is too warp-specific to vendor (it tracks AI blocks, cloud refresh tokens, suggestion ignores). But the **diesel + diesel_migrations stack is the right baseline** for caro-terminal.

**Decision for [#1019](https://github.com/wildcard/caro/issues/1019) + [#1020](https://github.com/wildcard/caro/issues/1020):** Adopt diesel + diesel_migrations. Write a narrow schema:

```sql
-- caro-terminal v0.1 schema
CREATE TABLE commands_history (
    id INTEGER PRIMARY KEY,
    command TEXT NOT NULL,
    cwd TEXT,
    exit_code INTEGER,
    started_at TIMESTAMP NOT NULL,
    finished_at TIMESTAMP,
    session_id TEXT NOT NULL
);
CREATE INDEX idx_commands_history_command ON commands_history(command);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    started_at TIMESTAMP NOT NULL,
    last_seen_at TIMESTAMP NOT NULL
);

CREATE TABLE tabs (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    cwd TEXT,
    title TEXT,
    closed_at TIMESTAMP
);
```

Schema lives at `caro-terminal/migrations/`.

### Pass 5 — `ipc` + `jsonrpc`

Two distinct crates with different purposes:

| | `ipc` | `jsonrpc` |
|---|---|---|
| Purpose | Typed RPC for Warp ↔ plugin host | LSP-style protocol talking |
| Wire format | `bincode` (compact, opaque) | JSON-RPC 2.0 (debuggable, standard) |
| Transport | `interprocess` crate (UDS + named pipes) | Trait-based (caller provides Transport) |
| WASM target | Yes (planned for Warp on Web) | No |
| Type safety | Service trait, ServiceCaller derive | Caller writes deserialization |
| Lines of code | ~7 modules, sizable | 4 files, small |

**Decision for [#1022](https://github.com/wildcard/caro/issues/1022) (Turi ADR):**

- **Wire format:** JSON-RPC 2.0 (matches what we already proposed; `jsonrpc` crate validates the design)
- **Transport:** `interprocess` crate (UDS on Unix, named pipes on Windows — same as warp `ipc`)
- **Don't vendor `ipc`** — bincode would make Turi protocol dumps unreadable, hurting debuggability for a sibling-app boundary that needs to be inspectable.
- **Don't vendor `jsonrpc`** either — use upstream `jsonrpsee` or `jsonrpc-core` (more maintained ecosystems).

This refines the wire-format decision in epic [#1008](https://github.com/wildcard/caro/issues/1008).

### Pass 6 — `input_classifier`

**Architecture surprise.** Warp doesn't pick *one* classifier — they have three backends plumbed through a common trait:

```rust
trait InputClassifier: Send + Sync {
    async fn detect_input_type(&self, input, context) -> InputType;
    async fn classify_input(&self, input, context) -> Result<ClassificationResult>;
}

struct ClassificationResult {
    p_shell: f32,  // probability it's a shell command
    p_ai: f32,    // probability it's a natural-language AI query
}
```

Backends:
1. **`HeuristicClassifier`** — regex/rules. Fast, no ML.
2. **`FasttextClassifier`** — FastText embeddings (Facebook's lightweight word-embedding model).
3. **`OnnxClassifier`** — ONNX runtime (transformer model). Supports two runtime libs: `candle` (Hugging Face's pure-Rust runtime) and `ort` (ONNX Runtime bindings).

There's also an `evaluate` binary for benchmark accuracy.

**Decision for caro-terminal v0.1:**

- Skip the input classifier entirely in v0.1. **The user opens the AI bar explicitly with Cmd+J** — no need to auto-detect from the input line.
- v0.2+ if we ever want auto-detection: ship `HeuristicClassifier` only (regex). Don't bundle ONNX models — too large for binary size, and caro-core's NL→cmd already has its own logic.
- v0.3+ at most: optional ONNX backend with downloadable model. But caro is not a research project; this is overkill unless real users complain.

This is a meaningful **scope reduction** for [#1018](https://github.com/wildcard/caro/issues/1018) — we don't need a classifier at all.

### Pass 7 — `warp_completer`

Multi-module subsystem: `completer/`, `parsers/`, `signatures/`, `meta`. Depends on `fuzzy_match` (now we know how `fuzzy_match` is used internally) and likely consumes `command-signatures-v2`'s JS bundle for command-line parsing.

**Decision:** Out of v0.1. Inline command completion is large, language-specific, and the warp_completer parses every shell command into typed tokens (massive scope for a v0.1).

For caro v0.2+: if we want inline completion, look at what shells already provide (`zsh-autosuggestions`, fish's built-in suggestions) and forward them — don't reimplement.

### Pass 8 — `command`

Tiny crate. Solves *one* problem: spawning processes on Windows without flashing a console window. Provides:
- `command::blocking::Command` (drop-in for `std::process::Command`)
- `command::r#async::Command` (drop-in for `async-process::Command`)

Both apply the `no_window` flag automatically on Windows. ~couple hundred LOC total.

**Decision for [#1013](https://github.com/wildcard/caro/issues/1013):** Vendor verbatim into `caro-terminal/src-tauri/src/command/`. License attribution + a brief NOTICE.md entry. Saves us a real Windows polish bug we'd otherwise discover only after Windows users complain.

### Pass 9 — `ai` (bonus, partial read)

Large agent system. Modules: `agent/`, `project_context/`, `diff_validation/`, `index/`, `skills/`, `document/`, `aws_credentials.rs`, `api_keys.rs`, `gfm_table.rs` (GitHub Flavored Markdown table parser), `llm_id.rs`.

Notable deps: `rmcp` (Rust MCP — they're an MCP client), `computer_use` (Anthropic's computer-use tool surface), `warp_multi_agent_api` (own crate, not opened — manages multiple concurrent agents).

The **`skills/`** subsystem mirrors Claude Code's skill format: `parse_skill.rs`, `parse_skill_test.rs`, `read_skills.rs`. They parse YAML frontmatter + body. **Caro's existing skills system in `.claude/skills/` is the same idea** — there's a future opportunity to share skill files between caro CLI and caro-terminal.

**Decision:** Out of v0.1. caro-core already does NL→cmd; we don't import warp's `ai`. Note for v0.2+: if we want richer in-terminal AI (multi-turn conversations, MCP tools, project context), warp's `ai` is the reference architecture.

---

## v0.1 cut — final after passes 1–9

| Issue | Original plan | Refined plan after research |
|---|---|---|
| **#1010 spike** | Test libghostty-vt FFI | **Compare `vte` (pure Rust) vs `libghostty-vt` (Zig FFI). Default to `vte`.** |
| **#1013 PTY pool** | portable-pty + per-tab PTY | **+ vendor warp's `command` crate for Windows no_window.** |
| **#1014 FFI bindings** | libghostty-vt-sys wrapper | **Replaced by: `vte` callbacks + own grid model (warp's pattern).** |
| **#1015 block parser** | OSC 133 parser | **Full standard A/B/C/D + interop with non-warp shells.** |
| **#1016 shell snippets** | bash/zsh/fish OSC 133 | **~50 lines each, standard OSC 133 only, p10k/starship coexistence required.** |
| **#1018 AI prompt bar** | Cmd+J → caro NL→cmd | **No input-classifier; explicit Cmd+J only. Scope reduced.** |
| **#1019 command palette** | fuzzy + history | **Use upstream `fuzzy-matcher` directly (don't vendor `fuzzy_match`). Diesel for history.** |
| **#1020 tabs+splits** | Restore on launch | **Diesel persistence layer (sessions + tabs tables).** |
| **#1021 themes** | 3 built-ins | **Unchanged.** |
| **#1022 Turi ADR** | JSON-RPC over UDS/pipes | **+ `interprocess` crate. + use upstream `jsonrpsee`. Don't vendor warp's `ipc` or `jsonrpc`.** |

**Additions to consider for v0.1.5 / v0.2:**
- `input_classifier` heuristic backend (skip the AI bar's Cmd+J requirement).
- A `skills/` module mirroring caro's existing skills format.
- Vendored `vte` integration for kitty graphics protocol if users ask.

# ADR-015: Distributed-LLM Backends (Mesh-LLM, AI-Horde) via a Hybrid Privacy Gateway

| **Status**     | Accepted                            |
|----------------|-------------------------------------|
| **Date**       | June 2026                           |
| **Authors**    | Caro Maintainers                    |
| **Target**     | Community                           |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [The Two Networks Compared](#the-two-networks-compared)
4. [Decision](#decision)
5. [The Hybrid Privacy Gateway](#the-hybrid-privacy-gateway)
6. [What Breaks at 100 Real Users](#what-breaks-at-100-real-users)
7. [Implementation Notes](#implementation-notes)
8. [Consequences](#consequences)

---

## Executive Summary

Caro converts natural language into safe POSIX commands using a spectrum of
inference backends. This ADR adds two **distributed-LLM networks** to that
spectrum and introduces a **Hybrid privacy gateway** so users can benefit from
remote compute without leaking personally identifying information (PII).

- **Mesh-LLM** ([github.com/Mesh-LLM/mesh-llm](https://github.com/Mesh-LLM/mesh-llm),
  Rust, Apache-2.0) — a P2P mesh exposing a single **OpenAI-compatible `/v1`
  API** that pools GPU/RAM across machines. Architecturally identical to Caro's
  existing Exo/vLLM backends.
- **AI-Horde** ([github.com/Haidra-Org/AI-Horde](https://github.com/Haidra-Org/AI-Horde),
  Python, AGPL-3.0) — a crowdsourced volunteer cluster with an **async
  job-queue API** and a free anonymous key.

**Decision:** adopt both as optional remote backends (behind the existing
`remote-backends` feature), make remote endpoint URLs configurable, and wrap
them in a `hybrid` backend that sanitizes PII locally before any prompt leaves
the device.

---

## Context and Problem Statement

Caro's backend spectrum before this change:

```
static_matcher → embedded (local LLM) → ollama/vllm/exo (self-hosted) → claude/openrouter (paid cloud)
```

Two gaps remained:

1. **"My model is too big for one machine, but I want it private."** No backend
   pooled compute across several owned machines.
2. **"I have no capable GPU and won't pay for a cloud API."** The `embedded`
   backend is bounded by local hardware; cloud backends cost money.

Mesh-LLM fills (1); AI-Horde fills (2). But both raise a privacy question: a
shell-command prompt frequently carries PII (cwd, filenames, usernames,
hostnames, IPs). Sending that verbatim to a *public* mesh or to *anonymous
volunteers* is unacceptable for a safety-first tool.

A secondary blocker: the backend factory **hard-coded** every remote URL, and
`UserConfiguration` had no endpoint fields — so even pointing Caro at a mesh on
a non-default port required a recompile.

---

## The Two Networks Compared

| Dimension | Mesh-LLM | AI-Horde |
|---|---|---|
| Topology | P2P mesh you host/join | Public volunteer cluster |
| API shape | OpenAI-compatible **sync** `/v1` | **Async** submit + poll (v2) |
| Language / License | Rust / Apache-2.0 | Python / AGPL-3.0 |
| Setup | Run a mesh node | None (anon key `0000000000`) |
| Latency | Low, direct routing | Variable (queue + kudos priority) |
| Trust model | Your own peers (private) or public | Untrusted anonymous volunteers |
| Privacy default | Private mesh = safe; public = leaks | Prompt always leaves to strangers |
| Caro integration cost | ~copy of `exo.rs` (sync) | New async poll-loop backend |
| Auto-detected? | Yes (probes `:9337`) | No — explicit opt-in only |

Neither project is a linked Rust crate — both are remote HTTP services — so the
`external-sdk-integration.md` build-spike rule does not apply (no `Cargo.toml`
dependency, no license linkage). AI-Horde is AGPL-3.0 (same as Caro); Mesh-LLM
is Apache-2.0 — both fine as remote services.

---

## Decision

1. **Add `BackendType::Mesh`, `BackendType::AiHorde`, `BackendType::Hybrid`.**
2. **`MeshBackend`** (`src/backends/remote/mesh.rs`): OpenAI-compatible mirror
   of the Exo backend; default `http://localhost:9337`; `model=mesh`
   Mixture-of-Agents auto-routing; auto-detected ahead of Exo.
3. **`AiHordeBackend`** (`src/backends/remote/ai_horde.rs`): async
   `POST /v2/generate/text/async` → poll `/v2/generate/text/status/{id}` →
   `generations[0].text`; anonymous key default; bounded `max_wait`; **explicit
   opt-in only** (never auto-detected, since it is a public service).
4. **`[backends]` config section** with per-backend URLs/keys, so endpoints are
   configurable without recompiling. The factory resolves URLs from config with
   localhost fallbacks.
5. **`HybridBackend`** (`src/backends/hybrid/`): the recommended way to use
   either remote network safely (see below).

---

## The Hybrid Privacy Gateway

The `hybrid` backend composes a **local** model (the embedded backend, also the
fallback) with a **remote** enhancer (Mesh-LLM or AI-Horde). Its pipeline, by
default (`[backends].allow_public = false`):

1. A deterministic `ContextSanitizer` redacts PII from the request `input` and
   `context` into **typed, self-describing placeholders** —
   `/Users/alice/secret.txt` → `<REDACTED_FILEPATH_1>` — keeping an in-memory
   placeholder↔value map.
2. A **redaction briefing** is prepended to the prompt sent to the remote. It
   (a) states that Caro's **local model** performed the redaction on the
   harness, (b) lists each placeholder with a **description** of the value it
   stands for ("an absolute filesystem path", "the user's login name", …), and
   (c) instructs the model to reproduce each placeholder **verbatim** and never
   guess the underlying value. The remote thus reasons about command *shape*
   without seeing private data.
3. The **sanitized** request + briefing is sent to the remote enhancer. The
   network never sees the real values.
4. Placeholders in the returned command are **restored** to real values
   locally, so the executed command is correct.
5. If the remote fails, Caro falls back to the local model on the original
   request (which never left the device). The local model is itself an **aware
   participant**: a privacy-layer contract note is attached to its context so it
   knows redaction is active and that it owns the redaction process.

The sanitizer is **rule/regex based, not an LLM call**, guaranteeing
determinism (same input → same placeholders → reproducible, cache-safe output).
Redaction scope ("Broad"): emails, IPv4 addresses, absolute/home paths, the
current username and hostname, and the values of uppercase `ENV=` assignments.
Class ordering redacts broad spans (paths) before narrow ones (usernames) so a
username inside a path is never half-leaked.

Placeholders are **typed and self-describing** (`<REDACTED_FILEPATH_1>`,
`<REDACTED_USERNAME_1>`, …) rather than opaque symbols, and the briefing legend
gives each a plain-language description. This is deliberate: a remote model that
knows a slot is "an absolute filesystem path" produces a correct command
template, whereas an opaque `<X1>` invites it to hallucinate a literal. Tokens
stay space-free and unique so restoration (longest-token-first replacement)
remains unambiguous.

**Opt-in relaxation:** when a user sets `allow_public = true` (e.g. for a
trusted *private* mesh where redaction is unnecessary), sanitization is skipped
and the prompt is sent verbatim — the same "standard remote warning" path that
Claude/OpenRouter already use.

This directly implements the design intent: *the local model is a privacy
gateway; the remote network is an enhancement that never receives PII unless the
user deliberately opts in.*

---

## What Breaks at 100 Real Users

(Per `validation-discipline.md` gate 3.)

- **Assumption that holds at demo scale:** the regex redaction set catches every
  PII form a user's prompt contains. At scale, novel PII shapes (IPv6, UNC
  paths, API tokens in unusual formats) may slip through.
  - **Failure mode:** a token the sanitizer doesn't recognize is sent to a
    public network.
  - **Instrumentation:** log `redaction_count` per request (already surfaced in
    `backend_used`); a sudden drop in average redactions on PII-heavy prompts
    signals a gap.
  - **Fallback:** the `allow_public = false` default means a sanitizer miss is
    the *only* leak vector, and the `[backends].hybrid_remote` + future
    `redact_patterns` escape hatch lets users add custom regexes. Conservative
    users keep the remote enhancer pointed at a private mesh.
- **AI-Horde queue saturation:** at 100 concurrent anonymous users, the shared
  anonymous-key kudos priority is lowest, so latency degrades.
  - **Instrumentation:** `typical_latency_ms` and the poll-loop `Timeout`.
  - **Fallback:** the bounded `max_wait` + embedded fallback means a slow queue
    degrades to local generation, never a hang.

---

## Implementation Notes

- All three backends share the `{"cmd": "..."}` 4-tier parse contract used by
  the existing remote backends.
- `ai::privacy::may_leak_context_offhost` flags `mesh` and `ai-horde` as
  off-host; `hybrid` is intentionally excluded because it sanitizes by default.
- Tests: 7 (mesh) + 6 (AI-Horde, via `wiremock`) + 11 (hybrid + sanitizer),
  including a privacy-proof test asserting the remote never receives PII while
  the restored command is correct.

## Consequences

**Positive:**
- Two new zero/low-cost backend options that extend Caro's reach to users with
  pooled hardware (Mesh-LLM) or no hardware (AI-Horde).
- Remote URLs are finally configurable, removing a recompile-to-relocate wart.
- The hybrid gateway makes "use a remote model" compatible with Caro's
  safety-first, local-first ethos.

**Negative / risks:**
- The redaction set is a moving target (see "What Breaks…").
- More backend permutations to test; mitigated by the per-backend feature gate
  and the shared parse/fallback helpers.

**Neutral:**
- `hybrid` is opt-in (`--backend hybrid`); existing default behavior is
  unchanged.

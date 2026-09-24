# ADR-030: Offline Voice Input (`caro voice`) — Handy.Computer Analog

- **Status**: Proposed
- **Date**: 2026-07-03
- **Authors**: caro-research scoping process (automated scheduled run)
- **Tracking**: [#662](https://github.com/wildcard/caro/issues/662) — Handy.Computer Integration (v2.0.0, "extends caro-core, exempt")
- **Relates to**: ADR-024 (headless JSON contract), `speech-to-text-integration-plan.md` (root; superseded in part by this ADR), `.claude/rules/external-sdk-integration.md` (build-spike gate)

> **Provenance note (autonomous run).** The scheduled task's `[FEATURE NAME]`
> placeholder remains unbound. This run selected **Handy.Computer** (#662)
> as the target: it is the last v2.0.0 roadmap item tagged "extends
> caro-core, exempt" with no existing ADR. Treat the analog choice as a
> reviewable assumption.

---

## Context

### Phase 1 — What Handy is and why it matters to caro

[Handy](https://github.com/cjpais/Handy) (MIT, Rust + Tauri) is an open
source, fully offline speech-to-text desktop app: press a shortcut, speak,
and the transcript is pasted into whatever app has focus. Its author's
framing — "not the best speech-to-text app, the most forkable one" — makes
it the reference OSS architecture for local STT in Rust.

**Architecture** (from README + repo, fetched 2026-07-03):

- `cpal` for cross-platform audio capture; `rubato` for resampling
- Silero VAD (`vad-rs`) filters silence before inference
- Two engine families: `transcribe-cpp` (Whisper GGML/GGUF, GPU-accelerated)
  and **`transcribe-rs`** (Parakeet ONNX int8, CPU-only, ~5x real-time on a
  mid-range i5, automatic language detection)
- `rdev` global shortcuts; Tauri single-instance plugin; text injection via
  enigo/xdotool/wtype/dotool
- Models live in a flat `models/` dir under the app-data path; manual
  installs and custom GGML models are auto-discovered by filename

**Why it is limited / its failure modes:**

1. **No structured output contract.** Handy's only output channel is a
   synthetic paste into the focused window. No stdout payload, no JSON, no
   exit codes, no events. The CLI flags (`--toggle-transcription`,
   `--cancel`) and Unix signals (`SIGUSR1/2`) merely remote-control a
   running GUI instance — they never return the transcript. The Raycast
   integration had to be a third-party extension scraping history.
2. **Daemon-resident lifecycle.** Model warmth is achieved by never
   exiting: a single-instance Tauri app keeps the model in RAM. There is no
   one-shot subprocess mode at all.
3. **Whisper crash class.** Whisper models crash on certain Windows/Linux
   configurations (config-dependent GPU/FFI issues; open "help wanted" in
   their README).
4. **Text-injection fragility.** Wayland needs `wtype`/`dotool`; the
   recording overlay can steal focus and break pasting; GTK layer-shell and
   WebKit DMA-BUF issues cause Linux startup crashes.
5. Their own roadmap admits the settings system is "bloated and messy".

Failure modes 1 and 2 are the ones this ADR must solve **by design**;
3 and 4 are avoided entirely by caro's positioning (below).

### Phase 2 — Competitive differentiation

**Replicate:**

- **Parakeet V3 int8 as the default engine.** CPU-only, broad hardware
  floor (Skylake+), ~5x real-time, auto language detection, and it dodges
  the Whisper crash class entirely.
- **VAD-gated capture** (Silero) so silence is trimmed before inference and
  "no speech" is detectable as a distinct outcome.
- **Flat model dir + checksum'd public download URLs + auto-discovery** —
  maps 1:1 onto caro's existing `src/cache` (download / checksum /
  manifest / progress) and `model_catalog.rs`.

**Avoid by designing our schema first:**

- Handy has no output contract → caro defines `TranscriptionResult` JSON
  (versioned, ADR-024-aligned) before any code lands.
- Handy needs a daemon → caro is a pure subprocess: int8 encoder,
  memory-mapped ONNX, cached model bytes; cold-start budget instead of
  residency.
- Handy must inject text into arbitrary apps → caro never injects text.
  The transcript feeds caro's own pipeline, so `enigo`/`xdotool`/overlay
  code and its whole failure class simply do not exist here.
- Handy links GTK layer-shell / WebKit → caro adds zero GUI deps.

**Our unique positioning:** voice → natural language → **safety-validated**
command. A mis-transcription in Handy types wrong text; a mis-transcription
in a voice-to-shell tool can type `rm -rf` — unless every transcript flows
through the existing 52-pattern `SafetyValidator`, which in caro it does by
construction. Offline like Handy, but scriptable, contract-first, and
composable (`caro voice transcribe --stt-only --output json | jq -r .text`).

**Existing infrastructure that already covers part of this:**

| Need | Already exists |
|---|---|
| Model download, checksum, cache manifest, progress | `src/cache/` |
| Model registry + selection | `src/model_catalog.rs`, `src/model_loader.rs` |
| Versioned JSON output + exit-code registry | ADR-024 headless contract (`schema_version`) |
| Special exit-code precedent | `EXIT_CODE_EDIT: i32 = 201` (`src/main.rs`) |
| Safety validation of the resulting command | `src/safety/` (untouched) |
| Config surface | `src/config/` (`[voice]` block) |
| Deterministic engine mocking pattern | `mock-backend` feature |

---

## Decision

Add an **off-by-default `voice-input` feature** providing offline
speech-to-text as (a) an input modality to the existing pipeline and (b) a
standalone, contract-stable transcription subcommand.

### CLI surface

```bash
caro --voice                        # record → transcribe → confirm → existing NL→command pipeline
caro voice transcribe               # record from mic, print transcript (human format)
caro voice transcribe --audio-file f.wav --stt-only --output json   # pure subprocess contract
caro voice download-model           # fetch parakeet-v3-int8 via src/cache (checksum-verified)
```

`--audio-file <wav>` exists from day one: it makes the engine testable
without a microphone and lets scripts pipe pre-recorded audio.

### Dependencies (Phase 0 build spike REQUIRED first)

Per `.claude/rules/external-sdk-integration.md`, the first PR is a ≤100 LOC
build spike, nothing else:

```toml
transcribe-rs = { version = "0.3", optional = true, default-features = false, features = ["onnx", "vad-silero"] }
cpal = { version = "0.15", optional = true }

[features]
voice-input = ["dep:transcribe-rs", "dep:cpal"]
```

Spike checklist status (verified 2026-07-03 against crates.io):

| Check | Finding |
|---|---|
| License | `transcribe-rs` 0.3.11 is **MIT** — absorbs cleanly into AGPL-3.0 ✓ |
| MSRV | **`rust_version` is undeclared** on all published versions — the spike MUST verify compilation on Rust 1.83 and fail loud if not |
| Optional + feature flag | As above; **not** in `default` |
| Code-ref smoke | `pub fn smoke()` constructing `ParakeetEngine` so `cargo check --features voice-input` actually compiles `ort` |
| Two builds | `--no-default-features --features embedded-cpu` and `…,voice-input`; also `cargo deny check licenses` for `ort`'s transitive tree |

The `whisper-cpp` feature of transcribe-rs is deliberately **not** enabled
(Handy's crash class lives there).

### New types (all `Serialize + Deserialize` from day one)

New file `src/ui/voice.rs` (inside the existing `ui` module — no new
top-level module):

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AudioSource {
    Microphone { max_secs: u16 },      // default 30
    WavFile { path: PathBuf },         // 16 kHz mono PCM; resample otherwise
}

#[derive(Serialize, Deserialize)]
pub struct TranscriptionRequest {
    pub source: AudioSource,
    pub model_id: String,              // "parakeet-v3-int8"
    pub language: Option<String>,      // None => auto-detect
}

#[derive(Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub schema_version: u32,           // 1
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,              // wall-clock inference time
    pub audio_secs: f32,               // post-VAD speech length
    pub model_id: String,
    pub vad_segments: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum VoiceError {
    #[error("model '{model_id}' not downloaded (expected at {expected_path}); run `caro voice download-model`")]
    ModelMissing { model_id: String, expected_path: PathBuf },
    #[error("no audio input device available")]
    NoAudioDevice,
    #[error("no speech detected in input")]
    EmptyTranscript,
    #[error("audio decode/resample failed: {0}")]
    AudioDecode(String),
    #[error("inference failed: {0}")]
    Inference(String),
}

pub trait VoiceEngine {                // enables MockVoiceEngine under mock-backend
    fn transcribe(&mut self, req: &TranscriptionRequest) -> Result<TranscriptionResult, VoiceError>;
}
```

`model_catalog.rs` gains one entry: `parakeet-v3-int8` with the public
archive URL (`https://blob.handy.computer/parakeet-v3-int8.tar.gz`, ~478 MB,
pinned sha256 recorded at implementation time — or a caro-mirrored copy)
downloaded and verified through the existing `src/cache` machinery.

### Exit code / output contract

Machines and scripts depend on:

- **stdout**: in `--output json`, exactly one JSON object
  (`TranscriptionResult` on success, `{"schema_version":1,"error":{code,kind,message}}` on failure). All logs to stderr.
- **Exit codes**: `0` success; `1` general error; and three new
  machine-actionable codes, to be registered in the ADR-024 headless
  contract registry (implementation must confirm no collision):
  - `30` — model missing (remedy: `caro voice download-model`)
  - `31` — no audio input device
  - `32` — empty transcript (VAD found no speech; distinct so scripts can retry)
- In `caro --voice` mode, everything downstream of the transcript keeps the
  existing pipeline's codes (safety block, 201 edit-mode, etc.) unchanged.

### Solving Handy's failure modes by design

1. **No contract → contract-first.** The JSON schema and exit codes above
   are the feature; the mic is plumbing.
2. **Daemon residency → cold-start budget.** Pure subprocess: Parakeet int8
   load ≤ 2 s on the Skylake floor, ONNX memory-mapped, model bytes on disk
   via the cache. No background process, no state between invocations
   beyond the read-only model cache. If the budget is missed, a warm
   sidecar is a v2 discussion — not a workaround now.

### Integration tests (deterministic)

- Checked-in small fixtures: `tests/fixtures/voice/{query.wav,silence.wav}`
  (16 kHz mono, seconds long).
- `MockVoiceEngine` (under `mock-backend`) for CI-fast paths: known request
  → byte-exact `TranscriptionResult` JSON, exit 0.
- Real-model tests marked `#[ignore]`, run in a dedicated CI job with the
  model dir cached: `query.wav → exit 0 + stable text field`;
  `silence.wav → exit 32 + error JSON`; missing model dir → `exit 30 +
  error JSON naming the remedy`.
- `caro voice transcribe --audio-file query.wav --stt-only --output json | jq .schema_version` == `1`.

### Files changed (minimal set)

| File | Change |
|---|---|
| `Cargo.toml` / `Cargo.lock` | optional deps + `voice-input` feature |
| `src/ui/voice.rs` | **new** — types, engine wrapper, VAD gating |
| `src/ui/mod.rs` | `pub mod voice;` (cfg-gated) |
| `src/main.rs` | `--voice` flag + `voice` subcommand wiring |
| `src/model_catalog.rs` | `parakeet-v3-int8` entry |
| `tests/voice_integration.rs` | **new** — contract tests above |

No changes to `src/safety/`, `src/backends/`, `src/cache/` (consumed as-is).

---

## Consequences

**Positive:** first offline voice input in the NL→shell category with a
stable machine contract; zero GUI deps; reuses cache/catalog/safety wholly;
Parakeet path avoids the documented Whisper crash class.

**Negative / accepted:** `ort` (ONNX Runtime) is a heavy transitive tree —
hence off-default and spike-gated for license + size; ~478 MB model
download on first use; mic capture behavior is inherently
platform-variable (mitigated by `--audio-file` being the contract-tested
path); `transcribe-rs` is a 0.x crate by a single maintainer (the Handy
author) — pin minor version, watch releases.

**Validation-discipline note:** #662 is tagged *"extends caro-core,
exempt"* in ROADMAP.md — voice is an input modality to the already-validated
core loop, not a new product line. This is distinct from **voice-synthesis
(#160/#187)**, which remains research-only/unvalidated; nothing here
implements or unblocks TTS.

## Alternatives considered

- **A. `whisper-rs` directly** (original `speech-to-text-integration-plan.md`
  recommendation): mature, but C++ FFI + GPU paths are exactly Handy's
  open crash class; larger integration surface. Rejected for v1; the plan
  doc's recommendation is superseded by the Parakeet-first path.
- **B. Shell out to a running Handy instance** (`handy --toggle-transcription`):
  requires their GUI daemon, returns no transcript, and Handy's brand terms
  restrict redistribution. Rejected.
- **C. OS-native STT** (macOS Speech framework, etc.): not uniform across
  caro's Tier-1 platforms, some routes audio off-device. Rejected.
- **D. Remote STT via OpenAI-compatible endpoint**: privacy regression
  vs. caro's local-first stance; could later ride the hybrid privacy
  gateway (ADR-015) as an opt-in enhancer. Deferred.

## Out of scope (next version)

- Global push-to-talk hotkeys / shell-widget mic binding (`rdev`-class work)
- Streaming partial transcripts (Parakeet EOU streaming exists in the
  ecosystem; needs the ADR-028 token-stream channel)
- Whisper engine option (`transcribe-rs/whisper-cpp`), GPU acceleration
- Post-processing, custom dictionary, transcript history
- Warm-model sidecar / daemon mode
- Any TTS ("voice of Caro") — separate, unvalidated hypothesis

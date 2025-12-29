# Speech-to-Text Integration Plan for cmdai

## Overview

Add offline speech-to-text capabilities to cmdai, allowing users to speak their natural language command descriptions instead of typing them. This aligns with cmdai's local-first, privacy-focused approach while improving accessibility and user experience.

**User Experience:**
```bash
# Current workflow
$ cmdai "find all PDF files larger than 10MB"

# New workflow with speech input
$ cmdai --voice
🎤 Listening... (press Ctrl+C to stop)
[User speaks: "find all PDF files larger than 10MB"]
✓ Transcribed: "find all PDF files larger than 10MB"

Generated command:
  find . -name "*.pdf" -size +10M

Execute this command? (y/N)
```

## Research Findings

### Evaluated Solutions

#### 1. whisper.cpp with whisper-rs bindings
- **Repository**: https://github.com/ggml-org/whisper.cpp
- **Rust Bindings**: https://github.com/tazz4843/whisper-rs
- **Pros**:
  - Most mature and widely adopted (actively maintained in 2025)
  - Pure C++ implementation with excellent performance
  - Multiple Rust binding options (whisper-rs, whisper-rs-sys)
  - Support for CUDA, ROCm, Metal acceleration
  - Minimal dependencies
  - Cross-platform (macOS, Linux, Windows)
  - Model flexibility (Whisper tiny → large)
- **Cons**:
  - Requires downloading Whisper model files (39MB-1.5GB)
  - C++/Rust FFI complexity
  - Binary size increase

#### 2. parakeet-rs (NVIDIA Parakeet via ONNX)
- **Repository**: https://github.com/altunenes/parakeet-rs
- **Pros**:
  - Native Rust implementation (100% Rust)
  - ONNX Runtime integration
  - Multilingual support (25 languages)
  - Streaming ASR with end-of-utterance detection
  - GPU acceleration options
- **Cons**:
  - Requires ONNX Runtime dependency
  - Less mature than whisper.cpp ecosystem
  - Larger model downloads
  - NVIDIA-focused (may not optimize for Apple Silicon)

#### 3. Handy (Desktop Application Reference)
- **Repository**: https://github.com/cjpais/Handy
- **Website**: https://handy.computer/
- **Architecture Reference**:
  - Uses whisper-rs + transcription-rs
  - cpal for audio I/O
  - vad-rs for voice activity detection (Silero)
  - Tauri for desktop app framework
- **Key Learnings**:
  - VAD is crucial for good UX (detect when user stops speaking)
  - Parakeet V3 offers CPU-only alternative
  - Cross-platform audio handling requires careful platform abstraction

### Recommendation: whisper.cpp + whisper-rs

**Rationale**:
1. **Maturity**: Most battle-tested solution with active 2025 maintenance
2. **Performance**: Optimized C++ core with Metal support for Apple Silicon
3. **Rust Ecosystem**: Well-maintained bindings (whisper-rs 0.12+)
4. **Alignment**: Matches cmdai's FFI approach (similar to MLX backend)
5. **Model Flexibility**: Support for tiny models (39MB) to large (1.5GB)
6. **Community**: Large ecosystem, proven in production

## Architecture Design

### Component Structure

```
cmdai/
├── src/
│   ├── audio/                  # NEW: Audio input module
│   │   ├── mod.rs             # Audio trait + platform abstraction
│   │   ├── capture.rs         # Audio capture using cpal
│   │   └── vad.rs             # Voice Activity Detection (optional)
│   ├── transcription/          # NEW: Speech-to-text module
│   │   ├── mod.rs             # Transcription trait
│   │   ├── whisper.rs         # whisper.cpp integration
│   │   └── models.rs          # Model management
│   ├── cli/
│   │   └── mod.rs             # Add --voice flag
│   └── main.rs                # Integrate speech input flow
```

### Trait-Based Design

```rust
/// Platform-agnostic audio input trait
#[async_trait]
pub trait AudioCapture {
    async fn start_recording(&mut self) -> Result<()>;
    async fn stop_recording(&mut self) -> Result<Vec<f32>>;
    fn is_recording(&self) -> bool;
}

/// Speech-to-text transcription trait
#[async_trait]
pub trait Transcriber {
    async fn transcribe(&self, audio: &[f32]) -> Result<String>;
    fn model_info(&self) -> ModelInfo;
    async fn is_available(&self) -> bool;
}

/// Whisper backend implementation
pub struct WhisperTranscriber {
    ctx: WhisperContext,
    model_path: PathBuf,
}
```

### Feature Flags

```toml
[features]
default = ["embedded-cpu"]

# Speech-to-text features
voice = ["dep:whisper-rs", "dep:cpal", "audio"]
voice-vad = ["voice", "dep:vad-rs"]  # Optional VAD
voice-gpu = ["voice", "whisper-rs/cuda"]  # GPU acceleration

# Platform-specific audio
audio = ["dep:cpal"]
```

## Implementation Plan

### Phase 1: Core Audio Infrastructure (Week 1)
**Scope**: Basic audio capture and validation

**Tasks**:
1. Add `cpal` dependency for cross-platform audio input
2. Create `src/audio/mod.rs` with `AudioCapture` trait
3. Implement basic audio recording (16kHz mono, f32 PCM)
4. Add unit tests for audio capture
5. Create platform-specific audio device selection
6. Add `--list-devices` CLI flag for debugging

**Deliverables**:
- Working audio capture on macOS, Linux, Windows
- Unit tests with mock audio input
- CLI flag: `cmdai --list-devices`

**Dependencies**:
```toml
cpal = "0.15"
```

### Phase 2: Whisper Integration (Week 2)
**Scope**: Speech-to-text transcription

**Tasks**:
1. Add `whisper-rs` and `whisper-rs-sys` dependencies
2. Create `src/transcription/whisper.rs` implementation
3. Implement `Transcriber` trait for Whisper
4. Add model download and caching logic (reuse existing cache module)
5. Create model selection logic (tiny → base → small)
6. Add comprehensive error handling for FFI calls
7. Write integration tests with sample audio files

**Deliverables**:
- Working Whisper transcription
- Model caching in `~/.config/cmdai/models/whisper/`
- Integration tests with test audio samples
- Support for multiple model sizes

**Dependencies**:
```toml
whisper-rs = "0.12"
whisper-rs-sys = "0.12"
```

### Phase 3: CLI Integration (Week 3)
**Scope**: End-to-end voice input workflow

**Tasks**:
1. Add `--voice` flag to CLI
2. Implement voice input flow in `main.rs`
3. Add real-time feedback (recording indicator, transcription status)
4. Integrate with existing command generation pipeline
5. Add keyboard interrupt handling (Ctrl+C to stop recording)
6. Create user confirmation for transcribed text
7. Add performance metrics (recording time, transcription time)

**Deliverables**:
- Complete voice input workflow
- User-friendly terminal UI with colored output
- Performance logging
- CLI flag: `cmdai --voice [--model tiny|base|small]`

**Example Flow**:
```bash
$ cmdai --voice --model tiny
🎤 Recording... (press Ctrl+C to stop)
⏸️  Stopped recording (3.2s)
🧠 Transcribing with Whisper (tiny)...
✓ Transcribed (0.8s): "find all PDF files larger than 10MB"

Generated command:
  find . -name "*.pdf" -size +10M

Execute this command? (y/N)
```

### Phase 4: Advanced Features (Week 4)
**Scope**: Voice Activity Detection and optimization

**Tasks**:
1. Add optional VAD integration (`vad-rs` with Silero)
2. Implement auto-stop on silence detection
3. Add configurable VAD thresholds
4. Create model warm-up for faster first inference
5. Add speech-to-text confidence scores
6. Implement retry logic for low-confidence transcriptions
7. Add language detection and multi-language support

**Deliverables**:
- Auto-stop recording on silence
- Faster user experience with VAD
- Multi-language support (optional)
- Configuration: `~/.config/cmdai/config.toml`

**Configuration Example**:
```toml
[voice]
enabled = true
model = "tiny"  # tiny, base, small, medium, large
auto_stop = true
vad_threshold = 0.5
language = "auto"  # auto, en, es, fr, etc.

[voice.vad]
enabled = true
threshold = 0.5
silence_duration_ms = 1500
```

### Phase 5: Testing & Optimization (Week 5)
**Scope**: Quality assurance and performance tuning

**Tasks**:
1. Create comprehensive test suite with sample audio
2. Add benchmarks for transcription performance
3. Optimize model loading and caching
4. Test on all platforms (macOS Intel/ARM, Linux, Windows)
5. Add GPU acceleration testing (CUDA, Metal, ROCm)
6. Create user documentation
7. Add troubleshooting guide

**Deliverables**:
- 90%+ test coverage for audio/transcription modules
- Performance benchmarks
- Cross-platform validation
- User documentation in `docs/VOICE_INPUT.md`

## Technical Considerations

### Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Model load time | < 500ms | Should not slow down startup significantly |
| Transcription (tiny model, 5s audio) | < 1s | Real-time feel |
| Transcription (base model, 5s audio) | < 2s | Acceptable for better accuracy |
| Memory overhead | < 200MB | Keep total binary manageable |
| Binary size increase | < 10MB | Maintain single-binary philosophy |

### Model Selection Strategy

Default model progression:
1. **tiny.en** (39MB) - Default for English, fastest
2. **base.en** (74MB) - Better accuracy, still fast
3. **small.en** (244MB) - High accuracy option

Model auto-selection logic:
- Check available disk space
- Measure first inference time
- Auto-downgrade if performance < target
- User can override with `--model` flag

### Cross-Platform Audio Handling

Platform-specific considerations:
- **macOS**: Use Core Audio via cpal, Metal acceleration for Whisper
- **Linux**: ALSA/PulseAudio via cpal, CUDA/ROCm acceleration optional
- **Windows**: WASAPI via cpal, DirectML acceleration possible

Permissions:
- macOS: Microphone access prompt (Info.plist for .app bundles)
- Linux: PulseAudio permissions
- Windows: Microphone privacy settings

### Safety Integration

Speech input must integrate with existing safety validation:
1. Transcribe audio to text
2. Generate command from text (existing flow)
3. Apply safety validation (existing patterns)
4. User confirmation (existing flow)

No changes needed to safety module - speech is just an input method.

### Error Handling

Robust error handling for:
- Microphone not found/permission denied
- Model download failures
- Transcription errors (low audio quality, background noise)
- FFI failures (whisper.cpp crashes)
- Resource exhaustion (disk space, memory)

All errors should provide helpful user guidance:
```
Error: Microphone not found
Tip: Check audio permissions in System Settings
Run 'cmdai --list-devices' to see available devices
```

## Testing Strategy

### Unit Tests
- Audio capture mock (simulate audio input)
- Transcriber trait implementations
- Model caching logic
- Error handling paths

### Integration Tests
- End-to-end transcription with sample audio files
- Platform-specific audio device handling
- Model download and verification
- Multi-model support

### Manual Testing
- Cross-platform validation (macOS, Linux, Windows)
- Microphone permission flows
- Different audio qualities (clean, noisy, low volume)
- Long recordings (30s+)
- Multiple languages

### Performance Tests
- Model loading benchmarks
- Transcription speed across model sizes
- Memory usage profiling
- Binary size validation

## Security & Privacy

### Privacy Guarantees
- **100% Offline**: All transcription happens locally
- **No Network Calls**: Whisper models run on-device
- **No Audio Storage**: Audio deleted after transcription
- **No Telemetry**: No usage data sent anywhere

### Security Considerations
- Validate audio input buffer sizes (prevent overflow)
- Sanitize transcribed text before command generation
- Existing safety validation applies to voice input
- Model file integrity checks (SHA256 validation)

## Documentation Requirements

### User Documentation
1. **docs/VOICE_INPUT.md** - Comprehensive voice input guide
   - Quick start
   - Model selection
   - Troubleshooting
   - Performance tips

2. **README.md Updates** - Add voice input examples
   ```bash
   # Voice input
   $ cmdai --voice
   🎤 Listening... (speak your command)
   ```

3. **Configuration Guide** - Voice settings in config.toml

### Developer Documentation
1. **Architecture docs** - Audio/transcription module design
2. **FFI Guide** - whisper.cpp integration patterns
3. **Testing Guide** - Audio testing strategies
4. **Platform Notes** - Platform-specific considerations

## Success Criteria

### Functional Requirements
- ✅ Users can speak commands instead of typing
- ✅ Transcription accuracy > 90% for clear English
- ✅ Works offline with no network dependency
- ✅ Cross-platform support (macOS, Linux, Windows)
- ✅ Multiple model size options (tiny, base, small)

### Non-Functional Requirements
- ✅ Transcription time < 2s for 5s audio (base model)
- ✅ Binary size increase < 10MB
- ✅ Memory overhead < 200MB
- ✅ Model load time < 500ms
- ✅ 90%+ test coverage for new modules

### User Experience
- ✅ Clear visual feedback during recording
- ✅ Easy model selection and configuration
- ✅ Helpful error messages
- ✅ Seamless integration with existing workflow
- ✅ Optional VAD for auto-stop convenience

## Dependencies

### Required Crates
```toml
[dependencies]
# Existing dependencies...
whisper-rs = { version = "0.12", optional = true }
cpal = { version = "0.15", optional = true }

[build-dependencies]
whisper-rs-sys = { version = "0.12", optional = true }

[dev-dependencies]
# Audio test utilities
hound = "3.5"  # WAV file I/O for test fixtures
```

### System Dependencies
- **macOS**: Xcode Command Line Tools (for Metal framework)
- **Linux**: ALSA dev packages (`libasound2-dev` on Ubuntu)
- **Windows**: Windows SDK (for WASAPI)

### Model Downloads
- Whisper models from HuggingFace or OpenAI
- Cached in `~/.config/cmdai/models/whisper/`
- SHA256 verification for integrity

## Open Questions

1. **Default Model Size**: Should default be `tiny` (fast) or `base` (accurate)?
   - **Recommendation**: `tiny` for speed, with clear upgrade path

2. **VAD Library**: Use `vad-rs` or implement custom VAD?
   - **Recommendation**: Use `vad-rs` (Silero) for Phase 4

3. **Streaming Transcription**: Support real-time streaming or buffer-then-transcribe?
   - **Recommendation**: Buffer-then-transcribe for Phase 1-3, explore streaming in Phase 4+

4. **Multi-Language**: Support non-English from day 1?
   - **Recommendation**: Start with English, add multi-language in Phase 4

5. **GPU Acceleration**: Make GPU support default or opt-in?
   - **Recommendation**: Opt-in via feature flags (keep binary lean)

## Alternative Approaches Considered

### Approach 1: Cloud-based Speech-to-Text (Rejected)
- **Pros**: No model downloads, always up-to-date
- **Cons**: Privacy concerns, requires network, costs money
- **Verdict**: Conflicts with cmdai's local-first philosophy

### Approach 2: Native Rust Whisper Implementation (Rejected)
- **Pros**: No FFI complexity, pure Rust
- **Cons**: Significant engineering effort, unproven performance
- **Verdict**: whisper.cpp is battle-tested and optimized

### Approach 3: System Speech Recognition APIs (Rejected)
- **Pros**: No dependencies, OS-native
- **Cons**: Platform-specific code, inconsistent quality, macOS only (best quality)
- **Verdict**: whisper.cpp provides consistent cross-platform experience

## Timeline Estimate

**Total Duration**: 5 weeks (medium-sized feature)

- **Week 1**: Audio infrastructure
- **Week 2**: Whisper integration
- **Week 3**: CLI integration
- **Week 4**: Advanced features (VAD)
- **Week 5**: Testing and documentation

**Engineer Effort**: ~1 FTE (full-time equivalent)

## References

- [whisper.cpp GitHub](https://github.com/ggml-org/whisper.cpp)
- [whisper-rs Rust Bindings](https://github.com/tazz4843/whisper-rs)
- [parakeet-rs](https://github.com/altunenes/parakeet-rs)
- [Handy Desktop App](https://github.com/cjpais/Handy)
- [Reddit Discussion](https://www.reddit.com/r/LocalLLaMA/comments/1ldvosh/handy_a_simple_opensource_offline_speechtotext/)
- [C++/Rust FFI in 2025](https://markaicode.com/cpp-rust-ffi-cxx-bridge-2025/)

## Related Issues

- None (initial proposal)

## Labels

`enhancement`, `feature`, `voice-input`, `accessibility`, `medium-priority`

---

**Decision**: Recommend proceeding with whisper.cpp + whisper-rs integration following the 5-phase implementation plan.

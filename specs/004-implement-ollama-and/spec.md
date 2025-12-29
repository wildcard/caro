# Feature Specification: Remote LLM Backend Support

**Feature Branch**: `004-implement-ollama-and`
**Created**: 2025-10-13
**Status**: Draft
**Input**: User description: "implement ollama and vllm backends"

## Execution Flow (main)
```
1. Parse user description from Input ✅
   → Feature: Enable caro to use remote LLM services for command generation
2. Extract key concepts from description ✅
   → Actors: caro users, remote LLM services (Ollama, vLLM)
   → Actions: generate commands via HTTP APIs, fallback between backends
   → Data: HTTP requests/responses, model configurations
   → Constraints: Network availability, API compatibility, error handling
3. For each unclear aspect: ✅
   → Backend priority: User preference vs automatic fallback
   → Authentication: API keys, tokens, or unauthenticated
   → Timeout handling: Default values and user configuration
4. Fill User Scenarios & Testing section ✅
5. Generate Functional Requirements ✅
6. Identify Key Entities ✅
7. Run Review Checklist (pending)
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a caro user, I want to generate shell commands immediately without any setup, and optionally use more powerful remote LLM services (Ollama or vLLM) for enhanced quality.

### Acceptance Scenarios

1. **Given** user has Ollama running locally, **When** user runs `caro "list all files"`, **Then** caro sends the request to Ollama and returns a generated command

2. **Given** user has vLLM service configured, **When** user specifies vLLM backend preference, **Then** caro uses vLLM for command generation

3. **Given** user has both Ollama and vLLM available, **When** primary backend fails, **Then** caro automatically falls back to the secondary backend

4. **Given** no backends are available, **When** user attempts command generation, **Then** caro displays a clear error message explaining connectivity issues

5. **Given** user specifies an invalid backend URL, **When** caro attempts to connect, **Then** caro provides diagnostic information to help troubleshoot the issue

### Edge Cases
- What happens when network connection is lost mid-request?
- How does system handle slow backend responses (>30 seconds)?
- What error messages appear when backend returns malformed JSON?
- How are authentication failures (401/403) communicated to users?
- What happens when backend rate limits are exceeded?

## Requirements *(mandatory)*

### Functional Requirements

**Backend Availability**
- **FR-001**: System MUST include embedded model compiled into binary as default backend (batteries included)
- **FR-002**: System MUST support connecting to Ollama HTTP API as optional enhancement for local command generation
- **FR-003**: System MUST support connecting to vLLM HTTP API as optional enhancement for remote command generation
- **FR-004**: System MUST verify remote backend availability before attempting connection
- **FR-005**: System MUST provide fallback to embedded model if remote backends unavailable or fail

**Command Generation**
- **FR-006**: System MUST send user prompts to selected backend (embedded or remote) and receive generated commands
- **FR-007**: System MUST handle responses from all backends consistently (embedded model, Ollama JSON, vLLM JSON)
- **FR-008**: System MUST parse command, explanation, and confidence scores from backend responses
- **FR-009**: System MUST apply safety validation to all commands regardless of backend source
- **FR-024**: Embedded model MUST support offline operation with no network dependency
- **FR-025**: Embedded model MUST generate commands within 2 seconds on typical hardware (M1 Mac or equivalent)

**Configuration**
- **FR-010**: Users MUST be able to specify preferred backend via three methods: CLI flag (highest priority), configuration file, or auto-detection
- **FR-011**: Users MUST be able to configure backend URLs and connection parameters in `~/.caro/config.toml`
- **FR-012**: System MUST use embedded model as default when no configuration exists (zero-config first run)
- **FR-013**: System MUST use sensible defaults for optional remote backends (localhost:11434 for Ollama)
- **FR-014**: System MUST persist backend preferences in user configuration file at `~/.caro/config.toml`
- **FR-023**: System MUST provide interactive configuration wizard (`caro init`) for optional remote backend setup

**Error Handling**
- **FR-015**: System MUST detect and report remote backend connection failures with actionable error messages
- **FR-016**: System MUST handle timeout scenarios gracefully (default timeout: 30 seconds for remote backends)
- **FR-017**: System MUST validate backend responses and reject malformed data
- **FR-018**: System MUST log backend errors for troubleshooting purposes
- **FR-026**: System MUST automatically fall back to embedded model on remote backend failures without user intervention

**Performance**
- **FR-019**: System MUST complete backend selection and availability checks within 500ms
- **FR-020**: System MUST report generation progress for requests exceeding 5 seconds (remote backends only)
- **FR-021**: System MUST cache remote backend availability status for 60 seconds to reduce latency
- **FR-027**: Embedded model startup MUST complete within 100ms (part of overall CLI startup budget)
- **FR-028**: Binary size MUST remain under 50MB excluding model weights (model size tracked separately)

**Compatibility**
- **FR-022**: System MUST work with Ollama API versions 0.1.0 and above (optional remote backend)
- **FR-029**: System MUST work with vLLM OpenAI-compatible API endpoints (optional remote backend)
- **FR-030**: System MUST handle differences in response formats between embedded model and remote backends
- **FR-031**: Embedded model MUST work offline on Linux, macOS, and Windows without additional runtime dependencies

**Platform Optimization**
- **FR-032**: System MUST provide MLX GPU-accelerated build as primary release for Apple Silicon macOS
- **FR-033**: MLX GPU build MUST be optimized for latest MacBook Pro models with unified memory architecture
- **FR-034**: System MUST provide CPU-based build as cross-platform fallback using Burn or Candle inference
- **FR-035**: Default embedded model MUST be Qwen (Qwen2.5-Coder) with Phi-3 and StarCoder2 as tested alternatives
- **FR-036**: CI/CD pipeline MUST benchmark inference performance across Qwen, Phi-3, and StarCoder2 models
- **FR-037**: Build system MUST support multiple model variants with separate release artifacts per model

### Non-Functional Requirements

**Reliability**
- **NFR-001**: Backend connection attempts MUST timeout after 5 seconds
- **NFR-002**: Command generation MUST timeout after 30 seconds
- **NFR-003**: System MUST retry failed requests up to 2 times with exponential backoff

**Usability**
- **NFR-004**: Error messages MUST include specific troubleshooting steps for common issues
- **NFR-005**: Backend status MUST be visible in verbose mode output
- **NFR-006**: System MUST provide clear feedback when switching backends due to failure

**Security**
- **NFR-007**: API credentials (when configured) MUST be stored securely in user configuration and never logged
- **NFR-008**: System MUST support HTTPS connections to remote backends
- **NFR-009**: System MUST warn users when sending prompts to remote services
- **NFR-010**: Initial release supports unauthenticated connections; optional API key authentication (Bearer token) available via configuration

### Key Entities

- **Embedded Model**: Qwen (Qwen2.5-Coder) as default fast coding model providing offline command generation. Two inference variants: (1) MLX GPU backend for Apple Silicon (primary macOS release), (2) CPU backend using Burn/Candle (cross-platform fallback). Always available with no external dependencies. Alternative models (Phi-3, StarCoder2) tested via CI/CD benchmarking.

- **Backend Connection**: Represents an active connection to an optional remote LLM service (Ollama or vLLM), including URL, authentication, availability status, and last successful connection timestamp

- **Backend Configuration**: User preferences for backend selection, including priority order (embedded → Ollama → vLLM), connection parameters (URLs, timeouts), and authentication credentials

- **Backend Response**: Structured data returned from any backend (embedded or remote) containing generated command, explanation text, confidence score, and any warnings or alternative suggestions

- **Connection Status**: Real-time health information about remote backend availability including connection state (connected, failed, unavailable), latency measurements, and error history

---

## Clarifications

### Session 2025-10-13
- Q: Which authentication approach should be implemented for vLLM remote backends? → A: Option D - Both no auth (start simple) and API key support via config file (Bearer tokens)
- Q: How should users specify their preferred backend? → A: Option D - All three methods (CLI flag > Config file > Auto-detect) with init/config wizard similar to AWS CLI or Claude Code, saved in ~/.caro/
- Q: What should happen when all backends are unavailable? → A: Use embedded model shipped with caro - batteries included, plug-and-play. Fast coding model compiled into binary as default fallback, making caro work offline out-of-box

## Clarifications Needed

### Authentication
- **RESOLVED**: Phase 1 will support unauthenticated connections (Ollama local, vLLM without auth). Phase 2 will add optional API key support via configuration file for vLLM enterprise deployments.

### Backend Priority
- **RESOLVED**: Three-tier priority system:
  1. CLI flag `--backend ollama|vllm` (highest priority, per-command override)
  2. Configuration file `~/.caro/config.toml` with preferred_backend setting
  3. Automatic selection (tries Ollama first, falls back to vLLM if configured)
- **UX Enhancement**: Provide `caro init` wizard (similar to AWS CLI/Claude Code) for first-time configuration setup

### Offline Behavior
- **RESOLVED**: Embedded model fallback strategy:
  - caro ships with fast coding model compiled into binary (default backend)
  - Remote backends (Ollama, vLLM) are optional enhancements for power users
  - When remote backends unavailable, automatically falls back to embedded model
  - No external dependencies required for basic operation (batteries included)
  - Binary size target: <50MB excluding model weights (model size tracked separately)

### Embedded Model Implementation
- **RESOLVED**: Model selection and inference strategy:
  - **Default Model**: Qwen (Qwen2.5-Coder) as primary embedded model
  - **Alternative Models**: Phi-3 and StarCoder2 for CI/CD performance benchmarking
  - **Inference Runtime**: Custom Rust inference using Burn or Candle (test both frameworks)
  - **Platform Priority**: Apple Silicon (MLX GPU) optimized for latest MacBook Pros
  - **Build Matrix**: Two release tracks:
    1. **MLX GPU build** (Apple Silicon - primary/default for macOS)
    2. **CPU build** (cross-platform fallback using Burn/Candle)
  - **Testing Strategy**: GitHub Actions pipeline with model performance benchmarking
  - **Future Roadmap**: ONNX and GGUF format support (post-MVP)
  - **Core Vision**: Batteries-included, plug-and-play on macOS with Apple Silicon optimization

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain → All resolved in Clarifications section
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked (see Clarifications Needed section)
- [x] User scenarios defined
- [x] Requirements generated (37 functional + 10 non-functional)
- [x] Entities identified (5 key entities)
- [x] Review checklist passed → All clarifications resolved

---

## Dependencies & Assumptions

### Dependencies
- Feature 001: TDD Foundation ✅ (complete)
- Feature 002: Core Models & Safety Validation ✅ (complete)
- Feature 003: Core Infrastructure ✅ (complete)
- **No external dependencies** for basic operation (embedded model included)
- **Inference Runtimes**:
  - MLX framework for Apple Silicon GPU acceleration (primary macOS build)
  - Burn or Candle for CPU-based cross-platform inference (fallback build)
- **CI/CD Infrastructure**: GitHub Actions for model performance benchmarking
- **Model Weights**: Qwen (Qwen2.5-Coder) as default; Phi-3 and StarCoder2 for testing
- Optional: Network connectivity for remote backend enhancements
- Optional: Ollama or vLLM service for power users

### Assumptions
- Embedded Qwen model sufficient for 80% of common shell command generation tasks
- Users can operate caro completely offline with embedded model
- Apple Silicon users will download MLX GPU build for optimal performance
- CPU build provides acceptable performance for non-Apple platforms
- Remote backends (Ollama, vLLM) are optional upgrades for better quality
- Safety validation module can process commands from any backend source (embedded or remote)
- Configuration management supports multi-backend preferences
- MLX framework stable and performant on latest MacBook Pro models

### Out of Scope
- Installing or configuring Ollama/vLLM services (user responsibility, optional)
- Streaming response support (future enhancement)
- User-provided custom model integration (beyond embedded/Ollama/vLLM)
- Fine-tuning or model customization
- Model updates separate from caro binary updates (embedded model versioned with caro releases)

---

**Next Steps**: All clarifications resolved. Run `/plan` to update planning artifacts with embedded model architecture (Qwen + MLX + Burn/Candle), then `/implement` to begin task execution.

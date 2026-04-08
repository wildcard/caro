# Glasswing Analysis: Lessons, Improvements, and Future Direction for Caro

## Context

On April 2026, Anthropic announced **Project Glasswing** -- a collaborative cybersecurity initiative with AWS, Apple, Cisco, CrowdStrike, Google, JPMorgan, Linux Foundation, Microsoft, NVIDIA, and Palo Alto Networks. The project introduces **Claude Mythos Preview**, an unreleased frontier model that dramatically outperforms current models at finding and exploiting software vulnerabilities (83.1% vs 66.6% on CyberGym, 93.9% vs 80.8% on SWE-bench Verified). It found zero-days in every major OS and browser, including a 27-year-old OpenBSD vulnerability and a 16-year-old FFmpeg bug.

This analysis examines what Glasswing means for Caro -- a Rust CLI that generates shell commands from natural language using local LLMs, where **safety validation is the core value proposition**.

---

## Part 1: What We Can Learn

### 1.1 Regex-Based Safety Is Necessary but Insufficient

**Glasswing finding**: AI models can now find vulnerabilities that survived "decades of human review and millions of automated tests." If AI can bypass decades of security testing in compiled code, it can certainly craft shell commands that evade 52 regex patterns.

**Caro implication**: Our current safety system (`src/safety/patterns.rs`) uses pattern matching with context-aware quote detection. This is a solid first layer, but Glasswing proves that pattern-based approaches have a fundamental ceiling. ADR-007 (AST Parser) and ADR-010 (Bubblewrap Sandbox) aren't nice-to-haves -- they're essential defense-in-depth layers.

**Key insight**: The documented limitations in `src/safety/mod.rs:155-161` (nested quotes, hex escapes, double-escaped quotes) are exactly the kind of edge cases an AI model would exploit.

### 1.2 Defense in Depth Is Non-Negotiable

**Glasswing philosophy**: The project explicitly frames "defense in depth" as the core architectural principle. Multiple independent safety layers that each catch what others miss.

**Caro's current layers**:
1. Pattern-based regex matching (implemented, 52+ patterns)
2. Context-aware quote detection (implemented)
3. User confirmation prompt (implemented)
4. AST-based semantic validation (ADR-007, proposed)
5. Sandbox execution isolation (ADR-010, proposed)
6. Fuzz testing of safety code (ADR-012, proposed)

**Gap**: Layers 4-6 are all still "Proposed" status. Glasswing validates that shipping without these layers is a risk. The gap between layers 3 and 4 is where adversarial inputs will succeed.

### 1.3 AI Models Will Be Used Offensively Against Tools Like Caro

**Glasswing finding**: Frontier models can autonomously develop exploits. This means:
- Users could prompt-inject Caro's LLM backends to generate commands that look safe but aren't
- Adversarial prompts designed by AI could systematically probe for regex pattern gaps
- The embedded backend's system prompt (`src/prompts/`) could be manipulated

**Caro implication**: We need to think of our safety system not just as protecting against accidental harm, but against **intentional adversarial attack** from AI-augmented threat actors.

### 1.4 The "Claude for Open Source" Program Is a Direct Opportunity

**Glasswing detail**: Anthropic is providing access to security-focused AI tools for open-source maintainers through a "Claude for Open Source" program, plus $2.5M to OpenSSF and $1.5M to Apache Foundation.

**Caro opportunity**: As an AGPL-3.0 open-source security tool, Caro could:
- Apply for the Claude for Open Source program to audit its own safety validation
- Use Mythos Preview (or its derivatives) to fuzz-test safety patterns
- Position Caro as a tool that benefits from Glasswing's defensive capabilities

### 1.5 The 90-Day Disclosure Cycle Sets the Pace

**Glasswing commitment**: Within 90 days, Anthropic will publicly disclose lessons learned, vulnerabilities found, and improvements. This creates a cadence of new threat intelligence that Caro should consume and adapt to.

---

## Part 2: How Caro Should Improve

### Priority 1: Accelerate ADR-007 (AST Parser) -- CRITICAL

**File**: `docs/adr/ADR-007-ast-parser-shell-validation.md`
**Current status**: Proposed
**Why now**: Regex patterns cannot understand command semantics. An AST parser using `yash-syntax` would:
- Distinguish `rm -rf /tmp/safe` from `rm -rf /` (semantic understanding)
- Detect obfuscated commands via variable expansion (`$'\x72\x6d'`)
- Understand command pipelines and subshells structurally
- Catch encoded/escaped evasion techniques that regex misses

**Concrete actions**:
1. Move ADR-007 from "Proposed" to "Accepted"
2. Implement Phase 1: yash-syntax integration for safety validation
3. Run existing safety test suite against AST parser to measure improvement
4. Add tests specifically for evasion techniques (hex encoding, variable expansion, eval obfuscation)

### Priority 2: Implement ADR-010 (Bubblewrap Sandbox) -- HIGH

**File**: `docs/adr/ADR-010-bubblewrap-sandbox-execution.md`
**Current status**: Proposed
**Why now**: Glasswing proves that validation-only approaches fail. The sandbox is the safety net when all validation layers are bypassed.

**Concrete actions**:
1. Move ADR-010 from "Proposed" to "Accepted"
2. Implement basic bwrap integration with filesystem restrictions
3. Make sandbox the default for all command execution (fail-safe: if sandbox unavailable, block execution)
4. Add `--no-sandbox` flag for users who explicitly opt out

### Priority 3: Implement ADR-012 (Honggfuzz Fuzz Testing) -- HIGH

**File**: `docs/adr/ADR-012-honggfuzz-integration.md`
**Current status**: Proposed
**Why now**: If AI models can find vulnerabilities that millions of automated tests missed, our safety validation code needs adversarial testing beyond unit tests and property-based tests.

**Concrete actions**:
1. Create fuzz targets for `SafetyValidator::validate_command()`
2. Create fuzz targets for `is_dangerous_in_context()` quote detection
3. Fuzz all 52+ regex patterns for ReDoS vulnerabilities
4. Add fuzz testing to CI pipeline (nightly)

### Priority 4: Adversarial Prompt Injection Testing -- MEDIUM

**New work**: Create a test suite specifically for adversarial prompt injection against Caro's LLM backends.

**Concrete actions**:
1. Create adversarial prompt test cases that attempt to:
   - Make the LLM generate commands with hidden semicolons/pipes
   - Use Unicode homoglyphs in commands
   - Exploit system prompt context injection
   - Generate commands that look safe but contain encoded danger
2. Add these to the eval framework (`src/evaluation/`)
3. Test across all backends (static, embedded, ollama, vllm)

### Priority 5: Safety Pattern Evasion Test Suite -- MEDIUM

**New work**: Create a systematic evasion test battery informed by Glasswing's findings.

**Evasion categories to test**:
- Hex-encoded characters (`$'\x72\x6d'` for `rm`)
- Variable expansion (`X=rm; $X -rf /`)
- Base64-encoded commands (`echo cm0gLXJmIC8= | base64 -d | sh`)
- Unicode tricks (homoglyphs, RTL override characters)
- Multi-command chaining with obfuscated delimiters
- eval/exec wrappers
- Alias exploitation
- Subshell nesting

### Priority 6: Apply for Claude for Open Source -- LOW (Strategic)

**Action**: Apply to Anthropic's "Claude for Open Source" program to get access to security-focused AI tools for auditing Caro's own safety validation.

---

## Part 3: What's Coming

### 3.1 The Threat Model Is Evolving

**Before Glasswing**: Caro's threat model assumed accidental dangerous commands from well-intentioned users or hallucinating LLMs.

**After Glasswing**: The threat model must include:
- **AI-augmented adversaries** who can systematically probe safety validation
- **Prompt injection attacks** designed by frontier models
- **Evasion techniques** that combine multiple obfuscation layers
- **Supply chain attacks** on model files or configuration

### 3.2 AI-Powered Safety Validation

**Future direction**: As models like Mythos Preview become available (even at $25/$125 per M tokens), Caro could add an **AI-powered safety validation layer** alongside regex + AST:

```
Command → Regex Patterns → AST Analysis → LLM Safety Review → Sandbox Execution
```

This would use a security-focused model to review generated commands before execution. The cost is minimal for individual CLI commands.

### 3.3 The Security Verification Ecosystem

Glasswing's **Cyber Verification Program** and **90-day disclosure cycles** will create a growing body of:
- Publicly disclosed vulnerability patterns
- Best practices for secure-by-design development
- Industry-specific security standards

Caro should consume these as they're published and update safety patterns accordingly.

### 3.4 Competitive Positioning

Glasswing establishes that **AI safety in command execution is a serious concern** backed by major tech companies. This validates Caro's core thesis. Caro should position itself as:
- The tool that takes Glasswing's defensive philosophy seriously
- A reference implementation of defense-in-depth for CLI command generation
- An open-source project that benefits from (and contributes to) the security ecosystem

### 3.5 Model Capability Implications

Mythos Preview's benchmarks show models are getting dramatically better at code understanding. For Caro this means:
- **Better command generation**: Future embedded/remote models will produce more accurate commands
- **Better evasion**: Those same capabilities make adversarial prompts more dangerous
- **Better defense**: AI-powered safety validation becomes more viable and effective

The arms race between generation quality and safety validation will intensify. Caro's multi-layered defense architecture (regex + AST + sandbox + optional AI review) is the right approach.

---

## Recommended ADR Priority (Re-ordered by Glasswing Urgency)

| Priority | ADR | Title | Glasswing Relevance |
|----------|-----|-------|-------------------|
| 1 | ADR-007 | AST Parser for Shell Validation | Directly addresses evasion techniques |
| 2 | ADR-010 | Bubblewrap Sandbox Execution | Safety net when all validation fails |
| 3 | ADR-012 | Honggfuzz Fuzz Testing | Adversarial testing of safety code |
| 4 | ADR-013 | Pre-Processing Pipeline | Input sanitization before validation |
| 5 | ADR-008 | Self-Update Mechanism | Rapid deployment of safety patches |
| 6 | ADR-006 | OLMo 3 Model Support | Better local models for generation |

---

## Summary

Glasswing's core message for Caro: **Your safety system is a good start, but the threat landscape just leveled up.** The regex pattern approach that works against accidental harm is insufficient against AI-augmented adversaries. The three proposed-but-unimplemented ADRs (AST parser, sandbox, fuzz testing) should be treated as urgent priorities, not aspirational improvements. The "Claude for Open Source" program is a concrete opportunity to get AI-powered security auditing for free.

The good news: Caro's architecture already has the right shape (multi-backend, configurable safety levels, evaluation framework). The work is to deepen each defense layer, not to redesign the system.

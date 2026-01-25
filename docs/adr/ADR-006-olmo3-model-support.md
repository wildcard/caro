# ADR-006: OLMo 3 Model Support

**Status**: Proposed

**Date**: 2026-01-03

**Authors**: caro maintainers

**Target**: Community

## Context

caro's model catalog currently supports several models ranging from ultra-tiny (SmolLM 135M) to large (Mistral 7B). Users have expressed interest in models with strong reasoning capabilities, particularly for complex shell command generation that requires multi-step thinking.

OLMo 3 is a new open-source model family from Allen AI that offers:
- Two size variants: 7B (4.5GB) and 32B (19GB)
- Two inference modes: Instruct and Think variants
- State-of-the-art performance on reasoning benchmarks
- Full open-source availability (weights and training data)
- 64K context window for complex command sequences

The model excels at:
- Mathematical reasoning (96.1% on MATH for 32B-Think)
- Coding tasks (91.4% on HumanEvalPlus for 32B-Think)
- Complex logical problems (89.8% on BigBenchHard for 32B-Think)

This aligns with caro's mission to provide high-quality local inference for shell command generation.

## Decision

Add OLMo 3 models to the caro model catalog with the following variants:

1. **OLMo 3 7B Instruct** - Standard instruction-following variant
2. **OLMo 3 7B Think** - Reasoning-enhanced variant for complex commands

The 32B variants will be documented but not included in the default catalog due to size constraints (19GB), with instructions for users who want to use them.

## Rationale

1. **Reasoning Capabilities**: The "Think" variant uses chain-of-thought reasoning, which is valuable for:
   - Complex multi-step shell pipelines
   - Commands requiring understanding of system state
   - Edge cases in command safety validation

2. **Open Source Alignment**: OLMo 3 is fully open (weights + training data), aligning with caro's open-source philosophy

3. **Benchmark Performance**: Strong coding benchmarks suggest good shell command understanding

4. **Size/Quality Balance**: The 7B model at 4.5GB is comparable to our existing Mistral 7B (3.5GB) while offering better reasoning

5. **Ollama Integration**: OLMo 3 is available on Ollama, enabling easy local deployment

## Consequences

### Benefits

- Enhanced reasoning for complex command generation
- Chain-of-thought capability for explaining command steps
- Strong coding task performance translates to shell scripting
- Full open-source model with transparent training
- 64K context window supports complex workflows

### Trade-offs

- 7B model (4.5GB) is larger than current default Qwen 1.5B (1.1GB)
- Think variant may have slower inference due to chain-of-thought
- Not suitable for CI/CD due to size
- Requires Ollama backend (not native GGUF integration initially)

### Risks

- **Model Quality Variance**: Shell command generation is a specific task not directly benchmarked → Mitigation: Extensive testing before promotion to default
- **Ollama Dependency**: Initial integration via Ollama, not direct GGUF → Mitigation: Add GGUF support when available
- **Resource Requirements**: 4.5GB minimum may exclude some users → Mitigation: Keep existing smaller models as alternatives

## Alternatives Considered

### Alternative 1: DeepSeek Coder
- Description: Code-specialized model family
- Pros: Excellent code generation, various sizes
- Cons: Less focus on reasoning, more code-centric than command-centric

### Alternative 2: Llama 3.2
- Description: Meta's latest small models
- Pros: Well-tested, good performance
- Cons: Less reasoning focus, no "think" variant

### Alternative 3: Qwen 2.5 7B
- Description: Larger Qwen variant
- Pros: Same family as current default, proven quality
- Cons: No specialized reasoning variant, less differentiated

## Implementation Notes

1. **Phase 1**: Add OLMo 3 7B Instruct to `model_catalog.rs`
2. **Phase 2**: Add OLMo 3 7B Think as optional reasoning model
3. **Phase 3**: Create benchmarks comparing OLMo 3 vs current models on shell tasks
4. **Phase 4**: Evaluate for potential default model promotion

### Integration Points

- `src/model_catalog.rs`: Add ModelInfo entries
- `docs/MODEL_CATALOG.md`: Update documentation
- Ollama backend: Ensure compatibility with `ollama run olmo3`
- Future: GGUF download support when available

### Testing Approach

- Unit tests for model catalog entries
- Integration tests with Ollama backend
- Command generation quality benchmarks
- Safety validation testing with OLMo 3 outputs

## Success Metrics

- **Metric 1**: Command generation accuracy ≥ current Qwen 1.5B on test suite
- **Metric 2**: User satisfaction ratings for complex command explanations
- **Metric 3**: Successful Think variant chain-of-thought outputs
- **Metric 4**: Download/usage adoption by community

## References

- [OLMo 3 on Ollama](https://ollama.com/library/olmo-3)
- [Allen AI OLMo Project](https://allenai.org/olmo)
- Related: ADR-001-enterprise-community-architecture.md (community model philosophy)
- caro Model Catalog: `docs/MODEL_CATALOG.md`

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-03 | caro maintainers | Initial draft |

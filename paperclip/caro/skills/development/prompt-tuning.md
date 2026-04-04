# Skill: LLM Prompt Tuning

Wraps existing Caro skill: `.claude/skills/prompt-tuner/`

## Purpose

Improve embedded LLM system prompts based on evaluation test failures to increase command generation accuracy.

## When to Use

- Evaluation tests show poor command generation quality
- Specific command categories (find, sed, awk) have low accuracy
- Model generates GNU flags instead of BSD/POSIX flags
- Commands are too verbose or unsafe

## Workflow

1. **Run evaluation**: `cargo run --bin caro-eval`
2. **Analyze failures**: Categorize failure patterns
3. **Modify prompts**: Update `src/prompts/command_templates.rs`
4. **Re-evaluate**: Run eval suite again to measure improvement
5. **Iterate**: Repeat until accuracy target is met

## Invocation

```
skill: prompt-tuner
```

## Key Files

- `src/prompts/command_templates.rs` — LLM prompt templates
- `src/evaluation/` — Evaluation framework
- `src/inference/embedded_backend.rs` — Embedded backend prompts

## Quality Targets

- 93%+ pass rate on comprehensive test suite
- Zero false positives in safety validation
- BSD/POSIX flag compliance
- Concise, single-line commands preferred

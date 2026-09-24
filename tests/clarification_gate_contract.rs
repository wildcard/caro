//! Contract: no backend prompt asks the model to emit a runnable
//! `echo 'Please clarify your request'` command, and every backend maps a
//! clarification onto the typed `GeneratorError::NeedsClarification` (#1462).

use caro::backends::GeneratorError;
use caro::decision::clarification_from_raw;

#[test]
fn no_backend_prompt_asks_for_echo_clarify() {
    // Every prompt-bearing source, remote and embedded. The banned phrase is
    // the runnable command itself, however it is packaged.
    let sources = [
        (
            "remote/ai_horde.rs",
            include_str!("../src/backends/remote/ai_horde.rs"),
        ),
        (
            "remote/claude.rs",
            include_str!("../src/backends/remote/claude.rs"),
        ),
        (
            "remote/exo.rs",
            include_str!("../src/backends/remote/exo.rs"),
        ),
        (
            "remote/mesh.rs",
            include_str!("../src/backends/remote/mesh.rs"),
        ),
        (
            "remote/ollama.rs",
            include_str!("../src/backends/remote/ollama.rs"),
        ),
        (
            "remote/openrouter.rs",
            include_str!("../src/backends/remote/openrouter.rs"),
        ),
        (
            "remote/vllm.rs",
            include_str!("../src/backends/remote/vllm.rs"),
        ),
        (
            "embedded/cpu.rs",
            include_str!("../src/backends/embedded/cpu.rs"),
        ),
        (
            "embedded/mlx.rs",
            include_str!("../src/backends/embedded/mlx.rs"),
        ),
        (
            "embedded/embedded_backend.rs",
            include_str!("../src/backends/embedded/embedded_backend.rs"),
        ),
        (
            "prompts/smollm_prompt.rs",
            include_str!("../src/prompts/smollm_prompt.rs"),
        ),
        (
            "prompts/minimal.rs",
            include_str!("../src/prompts/minimal.rs"),
        ),
        (
            "prompts/command_templates.rs",
            include_str!("../src/prompts/command_templates.rs"),
        ),
    ];
    let offenders: Vec<_> = sources
        .iter()
        .filter(|(_, src)| src.to_lowercase().contains("please clarify your request"))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        offenders.is_empty(),
        "prompt still asks for an echo: {offenders:?}"
    );
}

#[test]
fn legacy_outputs_become_typed_decisions() {
    for raw in [
        "QUESTION: Which directory?",
        r#"{"cmd": "echo 'Please clarify your request'"}"#,
        r#"{"needs_clarification": true, "p": 0.8, "question": "Which port?"}"#,
    ] {
        let c = clarification_from_raw(raw).expect(raw);
        assert!(c.should_ask(), "{raw}");
        let err = GeneratorError::from_clarification(&c);
        assert!(matches!(err, GeneratorError::NeedsClarification { .. }));
    }
    assert!(clarification_from_raw(r#"{"cmd": "ls -la"}"#).is_none());
}

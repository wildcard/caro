//! Contract: no backend prompt asks the model to emit a runnable
//! `echo 'Please clarify your request'` command, and every backend maps a
//! clarification onto the typed `GeneratorError::NeedsClarification` (#1462).

use caro::backends::GeneratorError;
use caro::decision::clarification_from_raw;

#[test]
fn no_backend_prompt_asks_for_echo_clarify() {
    let offenders: Vec<_> = [
        include_str!("../src/backends/remote/ai_horde.rs"),
        include_str!("../src/backends/remote/claude.rs"),
        include_str!("../src/backends/remote/exo.rs"),
        include_str!("../src/backends/remote/mesh.rs"),
        include_str!("../src/backends/remote/ollama.rs"),
        include_str!("../src/backends/remote/openrouter.rs"),
        include_str!("../src/backends/remote/vllm.rs"),
        include_str!("../src/backends/embedded/cpu.rs"),
    ]
    .iter()
    .enumerate()
    .filter(|(_, src)| src.contains("generate \"echo 'Please clarify"))
    .map(|(i, _)| i)
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

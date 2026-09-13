// Remote backend implementations for external model providers

pub mod ai_horde;
pub mod claude;
pub mod exo;
pub mod mesh;
pub mod ollama;
pub mod vllm;

pub use ai_horde::AiHordeBackend;
pub use claude::ClaudeBackend;
pub use exo::ExoBackend;
pub use mesh::MeshBackend;
pub use ollama::OllamaBackend;
pub use vllm::VllmBackend;

/// Context lengths below this floor risk silently truncating even caro's
/// short system prompts. Ollama's own default `num_ctx` (2048) sits right
/// at this boundary, which is why it's worth flagging rather than assuming
/// "the server answered" means "the server answered with enough room".
pub(crate) const MIN_SANE_CONTEXT_LENGTH: u32 = 4096;

/// Decide whether a served context length is worth warning about.
///
/// `served` is the context length actually in effect at generation time
/// (what the inference server reports it will use). `declared` is the
/// model's own catalog/max context length, when the backend can learn it
/// from the same API call — `None` when no such number is available.
///
/// This is a pure, synchronous decision on numbers the caller has already
/// fetched; it makes no network calls and never fails.
pub(crate) fn context_length_warning(served: u32, declared: Option<u32>) -> Option<String> {
    if let Some(declared) = declared {
        if served < declared {
            return Some(format!(
                "model declares a {declared}-token context window but the server is \
                 configured to serve only {served} tokens — prompts may be silently truncated"
            ));
        }
    }
    if served < MIN_SANE_CONTEXT_LENGTH {
        return Some(format!(
            "served context length is only {served} tokens, below the recommended floor \
             of {MIN_SANE_CONTEXT_LENGTH} — prompts may be silently truncated"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_when_served_is_below_floor_and_declared_unknown() {
        assert!(context_length_warning(2048, None).is_some());
    }

    #[test]
    fn silent_when_served_meets_floor_and_declared_unknown() {
        assert!(context_length_warning(8192, None).is_none());
    }

    #[test]
    fn warns_when_served_is_far_below_declared_even_above_floor() {
        // The motivating case: declared 131072, served 8192 — well above the
        // 4096 floor, but still an order of magnitude short of what the
        // model card promises.
        assert!(context_length_warning(8192, Some(131_072)).is_some());
    }

    #[test]
    fn silent_when_served_matches_declared_and_meets_floor() {
        assert!(context_length_warning(8192, Some(8192)).is_none());
    }
}

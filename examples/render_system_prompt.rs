//! Dump the canonical caro system prompt to stdout.
//!
//! Used by `tools/mlx-finetune/build_dataset.py` to guarantee that LoRA
//! fine-tune data embeds the exact same system prompt that the runtime
//! embedded backend ships. If the prompt ever drifts, the model will lose
//! effectiveness on its own benchmark — so we render from the source of
//! truth instead of hard-coding a copy.
//!
//! Usage:
//!   cargo run --quiet --example render_system_prompt > system_prompt.txt
//!
//! Optional env vars:
//!   CARO_PROFILE=gnu-linux|bsd|busybox|hybrid   (default: gnu-linux — matches `ubuntu()`)
//!   CARO_ALLOW_DESTRUCTIVE=1                    (default: 0 — emit QUESTION for rm/mv/dd)
//!   CARO_MAX_PIPELINE_STAGES=4                  (default: 4)

use caro::prompts::{CapabilityProfile, ProfileType, SmolLMPromptBuilder};

fn main() {
    let profile = match std::env::var("CARO_PROFILE").as_deref() {
        Ok("bsd") | Ok("macos") => CapabilityProfile::for_platform(ProfileType::Bsd),
        Ok("busybox") => CapabilityProfile::for_platform(ProfileType::Busybox),
        Ok("hybrid") => CapabilityProfile::for_platform(ProfileType::Hybrid),
        _ => CapabilityProfile::ubuntu(),
    };

    let allow_destructive = std::env::var("CARO_ALLOW_DESTRUCTIVE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let max_stages: usize = std::env::var("CARO_MAX_PIPELINE_STAGES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);

    let builder = SmolLMPromptBuilder::new(profile)
        .allow_destructive(allow_destructive)
        .max_pipeline_stages(max_stages);

    print!("{}", builder.build_system_prompt());
}

//! Per-backend cost estimation for evaluation runs.
//!
//! Inspired by the Fireworks "Open-Source Agents, Frontier Advisors" finding
//! that **cost is a first-class per-configuration metric** — every harness
//! configuration in their report carries a dollar figure, which is what makes
//! "advisor frequency as a cost lever" measurable. caro's eval already reports
//! pass-rate and latency per backend; this module adds an *estimated* USD cost
//! so local and frontier backends can be compared on price-per-passed-task.
//!
//! ## These are estimates
//!
//! Token counts are approximated from text length (~4 chars/token) and priced
//! against a per-million-token table. The harness does not yet surface real
//! token usage from backend responses, so the numbers are directional, not
//! billing-grade — good enough to compare backends and to size a frontier
//! advisor's cost contribution (Phase 3). Refine later by threading real
//! `usage` from remote backend responses into `EvaluationResult`.
//!
//! Local / self-hosted backends (embedded, static, mock, ollama, vllm, exo,
//! mesh) are priced at $0: they cost latency and energy, not per-token fees.

/// Rough characters-per-token ratio for mixed English + shell text.
pub const CHARS_PER_TOKEN: f64 = 4.0;

/// Estimate the token count of a piece of text from its character length.
///
/// Uses the standard ~4 chars/token heuristic, rounded up so that any
/// non-empty text costs at least one token.
pub fn estimate_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    (text.chars().count() as f64 / CHARS_PER_TOKEN).ceil() as u32
}

/// USD price per 1M tokens for a single backend/model, split by direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenPrice {
    /// USD per 1,000,000 input (prompt) tokens.
    pub input_per_mtok: f64,
    /// USD per 1,000,000 output (completion) tokens.
    pub output_per_mtok: f64,
}

impl TokenPrice {
    /// Local / self-hosted backends carry no per-token fee.
    pub const FREE: TokenPrice = TokenPrice {
        input_per_mtok: 0.0,
        output_per_mtok: 0.0,
    };

    /// Estimated USD cost for the given input/output token counts.
    pub fn cost(&self, tokens_in: u32, tokens_out: u32) -> f64 {
        (tokens_in as f64 / 1_000_000.0) * self.input_per_mtok
            + (tokens_out as f64 / 1_000_000.0) * self.output_per_mtok
    }
}

/// Resolve a per-token price for a backend by its registered name.
///
/// Matching is case-insensitive substring against the backend name (the same
/// string the eval harness registers and that lands in
/// `EvaluationResult::backend_name`). Local/self-hosted backends are free;
/// known hosted APIs use illustrative published per-MTok rates; an unknown
/// hosted backend gets a conservative non-zero default so its cost still
/// surfaces rather than silently reading as $0.
///
/// This table is intentionally a code default (KISS); making it config-driven
/// via a `pricing.toml` override is a tracked follow-up.
pub fn price_for(backend_name: &str) -> TokenPrice {
    let n = backend_name.to_ascii_lowercase();

    // Local / self-hosted: no per-token cost (latency/energy only).
    const LOCAL: [&str; 9] = [
        "embedded", "static", "mock", "mlx", "cpu", "ollama", "vllm", "exo", "mesh",
    ];
    if LOCAL.iter().any(|tag| n.contains(tag)) {
        return TokenPrice::FREE;
    }

    // Hosted frontier APIs — illustrative published rates (USD / MTok).
    // Most specific (model family) first so "claude-opus-..." hits the Opus row.
    if n.contains("opus") {
        return TokenPrice {
            input_per_mtok: 15.0,
            output_per_mtok: 75.0,
        };
    }
    if n.contains("sonnet") {
        return TokenPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
        };
    }
    if n.contains("haiku") {
        return TokenPrice {
            input_per_mtok: 0.80,
            output_per_mtok: 4.0,
        };
    }
    if n.contains("claude") || n.contains("openrouter") {
        // Generic hosted Claude/OpenRouter without an identified tier.
        return TokenPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
        };
    }

    // Unknown hosted backend: conservative non-zero default.
    TokenPrice {
        input_per_mtok: 1.0,
        output_per_mtok: 3.0,
    }
}

/// Estimate the cost of one generation given its backend, prompt, and output.
///
/// Returns `(tokens_in, tokens_out, cost_usd)`. `input` is the natural-language
/// request; `output` is the generated command text (empty when the backend
/// produced nothing, e.g. a block or timeout).
pub fn estimate_generation_cost(backend_name: &str, input: &str, output: &str) -> (u32, u32, f64) {
    let tokens_in = estimate_tokens(input);
    let tokens_out = estimate_tokens(output);
    let cost = price_for(backend_name).cost(tokens_in, tokens_out);
    (tokens_in, tokens_out, cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_zero_tokens() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn short_text_rounds_up_to_at_least_one_token() {
        // 2 chars / 4 = 0.5 -> ceil -> 1
        assert_eq!(estimate_tokens("ls"), 1);
    }

    #[test]
    fn token_estimate_uses_four_chars_per_token() {
        // 16 chars / 4 = 4
        assert_eq!(estimate_tokens("0123456789abcdef"), 4);
    }

    #[test]
    fn local_backends_are_free() {
        for name in [
            "embedded",
            "Embedded (MLX)",
            "static_matcher",
            "MockBackend",
            "ollama:codellama",
            "vllm",
            "exo-cluster",
            "mesh-llm",
        ] {
            assert_eq!(price_for(name), TokenPrice::FREE, "{name} should be free");
        }
    }

    #[test]
    fn opus_is_more_expensive_than_sonnet_than_haiku() {
        let opus = price_for("claude-opus-4-7");
        let sonnet = price_for("claude-sonnet-4-6");
        let haiku = price_for("claude-haiku-4-5");
        assert!(opus.output_per_mtok > sonnet.output_per_mtok);
        assert!(sonnet.output_per_mtok > haiku.output_per_mtok);
        assert!(haiku.output_per_mtok > 0.0);
    }

    #[test]
    fn unknown_hosted_backend_has_nonzero_default() {
        let p = price_for("some-new-hosted-llm");
        assert!(p.input_per_mtok > 0.0 && p.output_per_mtok > 0.0);
    }

    #[test]
    fn cost_math_is_direction_weighted() {
        let price = TokenPrice {
            input_per_mtok: 10.0,
            output_per_mtok: 100.0,
        };
        // 1M in @ $10 + 1M out @ $100 = $110
        assert!((price.cost(1_000_000, 1_000_000) - 110.0).abs() < 1e-9);
        // output is weighted 10x input here
        assert!(price.cost(0, 1000) > price.cost(1000, 0));
    }

    #[test]
    fn local_generation_costs_nothing() {
        let (ti, to, cost) = estimate_generation_cost("embedded", "list files", "ls -la");
        assert!(ti > 0 && to > 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn frontier_generation_costs_something() {
        let (_ti, _to, cost) = estimate_generation_cost(
            "claude-opus-4-7",
            "list files recursively",
            "find . -type f",
        );
        assert!(cost > 0.0);
    }
}

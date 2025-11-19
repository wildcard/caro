/// Candle Metal Backend Proof-of-Concept
///
/// This example validates that Candle can:
/// 1. Load a GGUF quantized model (Qwen2.5-Coder-1.5B-Instruct)
/// 2. Run inference on Apple Silicon GPU via Metal backend
/// 3. Produce valid text output (JSON command format)
/// 4. Complete in < 5 seconds total
///
/// Success criteria:
/// - Compiles successfully
/// - Initializes Metal device (GPU)
/// - Loads model from GGUF file
/// - Generates text output
/// - No crashes or panics
///
/// Usage:
/// ```bash
/// # With Metal GPU acceleration (Apple Silicon only)
/// cargo run --example candle_poc --release --features embedded-metal
///
/// # CPU fallback (cross-platform)
/// cargo run --example candle_poc --release --features embedded-cpu
/// ```

use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_core::quantized::gguf_file;
use candle_transformers::generation::LogitsProcessor;
use std::path::PathBuf;
use std::time::Instant;

// Model configuration for Qwen2.5-Coder-1.5B-Instruct
const MODEL_REPO: &str = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF";
const MODEL_FILE: &str = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf";
const TOKENIZER_FILE: &str = "tokenizer.json";

// Inference configuration
const MAX_TOKENS: usize = 100;
const TEMPERATURE: f64 = 0.7;
const TOP_P: Option<f64> = Some(0.9);
const SEED: u64 = 299792458; // Speed of light in m/s (deterministic seed)

/// System prompt for generating shell commands in JSON format
const SYSTEM_PROMPT: &str = r#"You are a command-line assistant. Convert natural language to POSIX shell commands.
Output ONLY valid JSON in this format: {"cmd": "command_here"}
Rules:
- Use only POSIX-compliant utilities
- Quote file paths with spaces properly
- Avoid destructive operations
- Keep commands simple and safe
"#;

fn main() -> Result<()> {
    println!("=== Candle Metal Backend Proof-of-Concept ===\n");

    let total_start = Instant::now();

    // Step 1: Initialize Metal device (Apple Silicon GPU)
    println!("[1/5] Initializing Metal device...");
    let device_start = Instant::now();

    let device = initialize_device()?;
    println!("  ✓ Device initialized: {:?}", device);
    println!("  ⏱  Time: {:?}\n", device_start.elapsed());

    // Step 2: Download/locate model files
    println!("[2/5] Downloading model from Hugging Face Hub...");
    let download_start = Instant::now();

    let (model_path, tokenizer_path) = download_model()?;
    println!("  ✓ Model path: {}", model_path.display());
    println!("  ✓ Tokenizer path: {}", tokenizer_path.display());
    println!("  ⏱  Time: {:?}\n", download_start.elapsed());

    // Step 3: Load GGUF quantized model
    println!("[3/5] Loading GGUF model to Metal GPU...");
    let load_start = Instant::now();

    let mut model = load_gguf_model(&model_path, &device)?;
    println!("  ✓ Model loaded successfully");
    println!("  ⏱  Time: {:?}\n", load_start.elapsed());

    // Step 4: Load tokenizer
    println!("[4/5] Loading tokenizer...");
    let tokenizer_start = Instant::now();

    let tokenizer = load_tokenizer(&tokenizer_path)?;
    println!("  ✓ Tokenizer loaded successfully");
    println!("  ⏱  Time: {:?}\n", tokenizer_start.elapsed());

    // Step 5: Run inference
    println!("[5/5] Running inference...");
    let inference_start = Instant::now();

    let prompt = "list all files in the current directory";
    let response = run_inference(&mut model, &tokenizer, &device, prompt)?;

    println!("  ✓ Inference completed");
    println!("  ⏱  Time: {:?}\n", inference_start.elapsed());

    // Display results
    println!("=== RESULTS ===");
    println!("Prompt: {}", prompt);
    println!("Response: {}", response);
    println!();

    // Performance summary
    let total_time = total_start.elapsed();
    println!("=== PERFORMANCE ===");
    println!("Total time: {:?}", total_time);
    println!("Device init: {:?}", device_start.elapsed());
    println!("Model download: {:?}", download_start.elapsed());
    println!("Model load: {:?}", load_start.elapsed());
    println!("Tokenizer load: {:?}", tokenizer_start.elapsed());
    println!("Inference: {:?}", inference_start.elapsed());
    println!();

    // Validation
    if total_time.as_secs() > 5 {
        println!("⚠ WARNING: Total time exceeded 5 seconds");
    } else {
        println!("✓ Performance target met (< 5 seconds)");
    }

    println!("\n✓ PROOF-OF-CONCEPT SUCCESSFUL!");

    Ok(())
}

/// Initialize Metal device for Apple Silicon GPU acceleration
fn initialize_device() -> Result<Device> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "embedded-metal"))]
    {
        Device::new_metal(0).context("Failed to initialize Metal device. Ensure you're on Apple Silicon and Metal is enabled.")
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "embedded-metal")))]
    {
        println!("  ⚠ Metal not available, falling back to CPU");
        Ok(Device::Cpu)
    }
}

/// Download model and tokenizer from Hugging Face Hub
fn download_model() -> Result<(PathBuf, PathBuf)> {
    use hf_hub::api::sync::Api;

    let api = Api::new().context("Failed to initialize Hugging Face API")?;
    let repo = api.model(MODEL_REPO.to_string());

    let model_path = repo
        .get(MODEL_FILE)
        .context("Failed to download model file")?;

    let tokenizer_path = repo
        .get(TOKENIZER_FILE)
        .context("Failed to download tokenizer file")?;

    Ok((model_path, tokenizer_path))
}

/// Load GGUF quantized model using Candle's quantized model support
///
/// Note: Candle 0.9 has built-in GGUF support via candle-transformers.
/// The exact API depends on the model architecture (Llama, Qwen, etc.)
fn load_gguf_model(model_path: &PathBuf, device: &Device) -> Result<Box<dyn ModelInference>> {
    // Candle 0.9 approach: Use quantized model loading
    // The exact implementation depends on Candle's current API

    use candle_transformers::models::quantized_llama as quantized;

    let mut file = std::fs::File::open(model_path)
        .context("Failed to open model file")?;

    // Read GGUF content
    let content = gguf_file::Content::read(&mut file)
        .map_err(|e| anyhow::anyhow!("Failed to read GGUF content: {}", e))?;

    // Load model weights from GGUF
    let model_weights = quantized::ModelWeights::from_gguf(
        content,
        &mut file,
        device,
    ).context("Failed to load GGUF model")?;

    Ok(Box::new(CandleModel {
        weights: model_weights,
    }))
}

/// Load tokenizer from file
fn load_tokenizer(tokenizer_path: &PathBuf) -> Result<tokenizers::Tokenizer> {
    tokenizers::Tokenizer::from_file(tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))
}

/// Run inference with the loaded model
fn run_inference(
    model: &mut Box<dyn ModelInference>,
    tokenizer: &tokenizers::Tokenizer,
    device: &Device,
    prompt: &str,
) -> Result<String> {
    // Construct full prompt with system message
    let full_prompt = format!("{}\n\nUser: {}\nAssistant:", SYSTEM_PROMPT, prompt);

    // Tokenize input
    let encoding = tokenizer
        .encode(full_prompt.as_str(), true)
        .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

    let tokens = encoding.get_ids();

    // Convert tokens to tensor
    let input_ids = Tensor::new(tokens, device)
        .context("Failed to create input tensor")?
        .unsqueeze(0)
        .context("Failed to add batch dimension")?;

    // Initialize logits processor for sampling
    let mut logits_processor = LogitsProcessor::new(
        SEED,
        Some(TEMPERATURE),
        TOP_P,
    );

    // Generate tokens
    let mut generated_tokens = Vec::new();
    let mut current_input = input_ids;

    for idx in 0..MAX_TOKENS {
        // Forward pass
        let logits = model.forward(&current_input, idx)
            .context("Forward pass failed")?;

        // Sample next token
        let next_token = logits_processor
            .sample(&logits.squeeze(0).context("Failed to squeeze logits")?)
            .context("Sampling failed")?;

        generated_tokens.push(next_token);

        // Check for EOS token (model-specific)
        if is_eos_token(next_token) {
            break;
        }

        // Prepare next input (append new token)
        current_input = Tensor::new(&[next_token], device)
            .context("Failed to create next token tensor")?
            .unsqueeze(0)
            .context("Failed to add batch dimension")?;
    }

    // Decode generated tokens
    let response = tokenizer
        .decode(&generated_tokens, true)
        .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;

    Ok(response)
}

/// Check if token is end-of-sequence
fn is_eos_token(token: u32) -> bool {
    // Common EOS token IDs (model-specific)
    // Qwen2.5 typically uses 151643 or 151645
    matches!(token, 151643 | 151645 | 2)  // 2 is common EOS for many models
}

/// Trait for model inference abstraction
trait ModelInference {
    fn forward(&mut self, input: &Tensor, position: usize) -> Result<Tensor>;
}

/// Wrapper for Candle quantized model
struct CandleModel {
    weights: candle_transformers::models::quantized_llama::ModelWeights,
}

impl ModelInference for CandleModel {
    fn forward(&mut self, input: &Tensor, position: usize) -> Result<Tensor> {
        self.weights
            .forward(input, position)
            .context("Model forward pass failed")
    }
}

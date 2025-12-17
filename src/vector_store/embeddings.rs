//! Embedding generation for vector store
//!
//! This module provides text embedding functionality using local models.
//! Without fastembed, we use a simple hashing-based approach or external service.

use anyhow::{Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{debug, warn};

/// Generate embedding vector for text
///
/// This is a placeholder implementation that creates deterministic
/// vectors based on text hashing. In production, this would use:
/// - sentence-transformers/all-MiniLM-L6-v2
/// - OpenAI embeddings API
/// - Local ONNX runtime
/// - Candle-based transformers
pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    // For now, use a simple deterministic approach
    generate_simple_embedding(text)
}

/// Generate simple hash-based embedding (384 dimensions to match MiniLM)
fn generate_simple_embedding(text: &str) -> Result<Vec<f32>> {
    const DIM: usize = 384;

    // Create a deterministic but distributed embedding
    let mut embedding = vec![0.0f32; DIM];

    // Use multiple hash functions to distribute values
    for (_i, chunk) in text.as_bytes().chunks(16).enumerate() {
        let mut hasher = DefaultHasher::new();
        chunk.hash(&mut hasher);
        let hash_value = hasher.finish();

        // Spread hash across multiple dimensions
        let base_idx = (hash_value % DIM as u64) as usize;
        let value = (hash_value as f32) / (u64::MAX as f32);

        embedding[base_idx] += value;
        if base_idx + 1 < DIM {
            embedding[base_idx + 1] += value * 0.5;
        }
        if base_idx > 0 {
            embedding[base_idx - 1] += value * 0.5;
        }
    }

    // Normalize to unit vector (cosine similarity)
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for val in &mut embedding {
            *val /= magnitude;
        }
    }

    Ok(embedding)
}

/// Generate embedding using external API (future implementation)
#[allow(dead_code)]
async fn generate_api_embedding(text: &str, _api_url: &str) -> Result<Vec<f32>> {
    // TODO: Implement OpenAI-compatible API call
    // Example: POST to api_url with {"input": text}
    warn!("API embedding not implemented, falling back to simple embedding");
    generate_simple_embedding(text)
}

/// Generate embedding using local ONNX model (future implementation)
#[allow(dead_code)]
fn generate_local_embedding(text: &str, _model_path: &str) -> Result<Vec<f32>> {
    // TODO: Implement local ONNX inference
    // - Load tokenizer
    // - Tokenize text
    // - Run inference
    // - Extract embeddings
    warn!("Local ONNX embedding not implemented, falling back to simple embedding");
    generate_simple_embedding(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_embedding() {
        let text = "list all files in directory";
        let embedding = generate_embedding(text).await.unwrap();

        // Check dimensions
        assert_eq!(embedding.len(), 384);

        // Check normalization (should be close to 1.0)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_embedding_deterministic() {
        let text = "find large files";

        let embedding1 = generate_embedding(text).await.unwrap();
        let embedding2 = generate_embedding(text).await.unwrap();

        // Same input should produce same output
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_embedding_different() {
        let text1 = "list files";
        let text2 = "show processes";

        let embedding1 = generate_embedding(text1).await.unwrap();
        let embedding2 = generate_embedding(text2).await.unwrap();

        // Different inputs should produce different outputs
        assert_ne!(embedding1, embedding2);

        // Calculate cosine similarity
        let dot_product: f32 = embedding1.iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();

        // Should be less than perfect similarity
        assert!(dot_product < 0.99);
    }

    #[test]
    fn test_simple_embedding_normalization() {
        let text = "test normalization";
        let embedding = generate_simple_embedding(text).unwrap();

        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6, "Magnitude should be 1.0, got {}", magnitude);
    }
}

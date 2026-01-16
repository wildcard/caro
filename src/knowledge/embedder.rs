//! Text embedding generation using FastEmbed
//!
//! Provides efficient embedding generation for semantic search.

use crate::knowledge::{KnowledgeError, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::path::Path;

/// Text embedder using sentence transformers
pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    /// Create a new embedder, downloading the model if needed
    ///
    /// Uses the all-MiniLM-L6-v2 model (384 dimensions, ~22MB)
    pub fn new(cache_dir: Option<&Path>) -> Result<Self> {
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2);

        if let Some(dir) = cache_dir {
            options = options.with_cache_dir(dir.to_path_buf());
        }

        let model = TextEmbedding::try_new(options)
            .map_err(|e| KnowledgeError::EmbedderInit(e.to_string()))?;

        Ok(Self { model })
    }

    /// Create embedder with quantized model for faster inference
    pub fn new_quantized(cache_dir: Option<&Path>) -> Result<Self> {
        let mut options = InitOptions::new(EmbeddingModel::AllMiniLML6V2Q);

        if let Some(dir) = cache_dir {
            options = options.with_cache_dir(dir.to_path_buf());
        }

        let model = TextEmbedding::try_new(options)
            .map_err(|e| KnowledgeError::EmbedderInit(e.to_string()))?;

        Ok(Self { model })
    }

    /// Generate embeddings for a list of texts
    pub fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.model
            .embed(texts.to_vec(), None)
            .map_err(|e| KnowledgeError::EmbeddingFailed(e.to_string()))
    }

    /// Generate embedding for a single text
    pub fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed(&[text])?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| KnowledgeError::EmbeddingFailed("No embedding generated".to_string()))
    }

    /// Create a combined embedding from request + command
    ///
    /// This creates a semantic representation that captures both
    /// the intent (request) and the solution (command).
    pub fn embed_command(&self, request: &str, command: &str) -> Result<Vec<f32>> {
        // Combine request and command for richer context
        let combined = format!("{} -> {}", request, command);
        self.embed_one(&combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::schema::EMBEDDING_DIM;

    #[test]
    #[ignore = "requires model download"]
    fn test_embedder_creation() {
        let embedder = Embedder::new(None).unwrap();
        let embedding = embedder.embed_one("list all files").unwrap();
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    #[ignore = "requires model download"]
    fn test_embed_multiple() {
        let embedder = Embedder::new(None).unwrap();
        let texts = vec!["list files", "show disk usage", "find large files"];
        let embeddings = embedder.embed(&texts).unwrap();
        assert_eq!(embeddings.len(), 3);
        for emb in embeddings {
            assert_eq!(emb.len(), EMBEDDING_DIM);
        }
    }

    #[test]
    #[ignore = "requires model download"]
    fn test_embed_command() {
        let embedder = Embedder::new(None).unwrap();
        let embedding = embedder.embed_command("list files", "ls -la").unwrap();
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }
}

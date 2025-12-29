/**
 * Image-Heavy Article Demo
 * =========================
 * Example article with many diagrams and visualizations.
 */

import React from 'react';
import { ArticlePage } from '../ArticlePage';

export function ImageHeavyArticle() {
  return (
    <ArticlePage
      title="Understanding LLM Inference Architecture"
      subtitle="A visual guide to how large language models process and generate text."
      category="AI"
      categorySlug="ai"
      author={{
        name: 'Sarah Park',
        avatar: '/images/authors/sarah.jpg',
        bio: 'AI researcher focused on making ML accessible to developers.',
      }}
      publishedAt="2025-12-10"
      readingTime="10 min read"
      coverImage="/images/articles/llm-architecture-cover.jpg"
      coverImageAlt="Neural network visualization"
      variant="image-heavy"
    >
      <p>
        Large Language Models have transformed how we build software. But how
        do they actually work under the hood? In this visual guide, we'll break
        down the architecture of modern LLMs and understand how they process
        and generate text.
      </p>

      <h2>The Transformer Architecture</h2>

      <p>
        At the heart of every modern LLM is the Transformer architecture,
        introduced in the seminal "Attention Is All You Need" paper. Unlike
        earlier recurrent models, Transformers process all tokens in parallel.
      </p>

      <figure>
        <img
          src="/images/articles/transformer-architecture.svg"
          alt="Diagram showing the transformer architecture with encoder and decoder stacks"
        />
        <figcaption>
          Figure 1: The Transformer architecture consists of an encoder and
          decoder, each with multiple layers of attention and feed-forward
          networks.
        </figcaption>
      </figure>

      <h2>Self-Attention Mechanism</h2>

      <p>
        The key innovation is the self-attention mechanism. It allows each
        token to "attend" to all other tokens in the sequence, capturing
        long-range dependencies that were difficult for RNNs.
      </p>

      <figure>
        <img
          src="/images/articles/attention-visualization.svg"
          alt="Visualization of attention weights between tokens"
        />
        <figcaption>
          Figure 2: Attention weights showing how the word "it" attends most
          strongly to "animal" when determining its meaning.
        </figcaption>
      </figure>

      <p>
        The attention mechanism computes three vectors for each token: Query
        (Q), Key (K), and Value (V). The attention score is computed as:
      </p>

      <figure>
        <img
          src="/images/articles/attention-formula.svg"
          alt="Mathematical formula for scaled dot-product attention"
        />
        <figcaption>
          Figure 3: The scaled dot-product attention formula.
        </figcaption>
      </figure>

      <h2>The Inference Pipeline</h2>

      <p>
        When you send a prompt to an LLM, it goes through several stages before
        generating a response:
      </p>

      <figure>
        <img
          src="/images/articles/inference-pipeline.svg"
          alt="Flowchart showing the LLM inference pipeline from input to output"
        />
        <figcaption>
          Figure 4: The complete inference pipeline, from tokenization through
          generation and decoding.
        </figcaption>
      </figure>

      <h3>1. Tokenization</h3>

      <p>
        First, your text is converted into tokens - subword units that the
        model understands. Different models use different tokenization schemes.
      </p>

      <figure>
        <img
          src="/images/articles/tokenization-example.svg"
          alt="Example of how text is split into tokens"
        />
        <figcaption>
          Figure 5: The sentence "Hello, world!" tokenized into subword units.
        </figcaption>
      </figure>

      <h3>2. Embedding</h3>

      <p>
        Each token is converted to a high-dimensional vector (embedding) that
        captures its semantic meaning.
      </p>

      <h3>3. Forward Pass</h3>

      <p>
        The embeddings pass through multiple Transformer layers, each applying
        attention and feed-forward transformations.
      </p>

      <figure>
        <img
          src="/images/articles/forward-pass.svg"
          alt="Visualization of data flowing through transformer layers"
        />
        <figcaption>
          Figure 6: Data flows through 32+ Transformer layers, with each layer
          refining the representations.
        </figcaption>
      </figure>

      <h2>Memory and Performance</h2>

      <p>
        Running LLMs locally requires significant resources. Here's a breakdown
        of memory requirements for different model sizes:
      </p>

      <figure>
        <img
          src="/images/articles/memory-requirements.svg"
          alt="Bar chart comparing memory usage of different model sizes"
        />
        <figcaption>
          Figure 7: Memory requirements scale with model size. Quantization can
          significantly reduce memory usage.
        </figcaption>
      </figure>

      <h2>Optimization Techniques</h2>

      <p>
        Several techniques can make inference faster and more memory-efficient:
      </p>

      <figure>
        <img
          src="/images/articles/optimization-comparison.svg"
          alt="Comparison of different optimization techniques"
        />
        <figcaption>
          Figure 8: KV caching, quantization, and batching can dramatically
          improve inference performance.
        </figcaption>
      </figure>

      <h2>Conclusion</h2>

      <p>
        Understanding LLM architecture helps you make better decisions about
        model selection, optimization, and deployment. As you build AI-powered
        applications, keep these architectural principles in mind.
      </p>

      <p>
        In the next article, we'll dive deeper into quantization techniques
        and how they enable running large models on consumer hardware.
      </p>
    </ArticlePage>
  );
}

export default ImageHeavyArticle;

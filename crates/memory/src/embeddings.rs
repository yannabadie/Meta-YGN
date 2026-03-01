use anyhow::Result;

/// Trait for embedding text into vectors.
/// Implementations can use fastembed, OpenAI, Ollama, or be no-ops.
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn provider_name(&self) -> &str;
}

/// No-op provider: returns empty vectors. Used when embeddings are disabled.
pub struct NoOpProvider;

impl EmbeddingProvider for NoOpProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![])
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![]).collect())
    }

    fn dimension(&self) -> usize {
        0
    }

    fn provider_name(&self) -> &str {
        "none"
    }
}

/// Simple hash-based embedding: converts text to a fixed-dimension vector by
/// hashing individual terms into bucket indices and accumulating counts.
///
/// Not a real neural embedding but provides basic semantic similarity without
/// any external dependencies. Useful as a lightweight fallback when no model
/// is available.
pub struct HashEmbedProvider {
    dimension: usize,
}

impl HashEmbedProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl EmbeddingProvider for HashEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut vec = vec![0.0f32; self.dimension];
        // Simple term-frequency hashing: each word hashes to a dimension,
        // value = count of words that hash to that bucket.
        for word in text.split_whitespace() {
            let word_lower = word.to_lowercase();
            let hash: usize = word_lower.bytes().fold(0usize, |acc, b| {
                acc.wrapping_mul(31).wrapping_add(b as usize)
            });
            let idx = hash % self.dimension;
            vec[idx] += 1.0;
        }
        // L2 normalize
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        Ok(vec)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "hash"
    }
}

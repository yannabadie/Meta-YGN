//! Neural embedding provider backed by the `fastembed` crate.
//!
//! Only compiled when the `embeddings` feature is enabled.

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::embeddings::EmbeddingProvider;

/// Neural embedding provider using fastembed's ONNX-based models.
///
/// Default model: BGE-Small-EN v1.5 (384 dimensions).
pub struct FastEmbedProvider {
    model: TextEmbedding,
}

impl FastEmbedProvider {
    /// Initialise with BGE-Small-EN v1.5, 384 dimensions.
    ///
    /// Download progress is suppressed to keep daemon/CLI output clean.
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_show_download_progress(false),
        )?;
        Ok(Self { model })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.model.embed(vec![text], None)?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("fastembed returned no embeddings"))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let input: Vec<&str> = texts.to_vec();
        let results = self.model.embed(input, None)?;
        Ok(results)
    }

    fn dimension(&self) -> usize {
        384
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }
}

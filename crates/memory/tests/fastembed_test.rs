#![cfg(feature = "embeddings")]

use metaygn_memory::embeddings::EmbeddingProvider;
use metaygn_memory::fastembed_provider::FastEmbedProvider;
use metaygn_memory::graph::cosine_similarity;

#[test]
fn provider_metadata() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");
    assert_eq!(provider.dimension(), 384);
    assert_eq!(provider.provider_name(), "fastembed");
}

#[test]
fn single_embedding_returns_384_dim_non_zero() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");
    let vec = provider.embed("hello world").expect("embed failed");
    assert_eq!(vec.len(), 384);
    // At least some values should be non-zero
    assert!(vec.iter().any(|&v| v != 0.0), "embedding is all zeros");
}

#[test]
fn batch_embedding_returns_correct_count() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");
    let texts: Vec<&str> = vec![
        "first document",
        "second document",
        "third document",
    ];
    let results = provider.embed_batch(&texts).expect("embed_batch failed");
    assert_eq!(results.len(), 3);
    for vec in &results {
        assert_eq!(vec.len(), 384);
    }
}

#[test]
fn similar_texts_higher_cosine_than_unrelated() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");

    let a = provider.embed("the cat sat on the mat").expect("embed a");
    let b = provider.embed("a cat is sitting on a mat").expect("embed b");
    let c = provider
        .embed("quantum computing and cryptography")
        .expect("embed c");

    let sim_ab = cosine_similarity(&a, &b);
    let sim_ac = cosine_similarity(&a, &c);

    assert!(
        sim_ab > sim_ac,
        "similar texts ({sim_ab:.4}) should score higher than unrelated ({sim_ac:.4})"
    );
}

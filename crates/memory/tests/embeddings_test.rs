use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider, NoOpProvider};
use metaygn_memory::graph::cosine_similarity;

// ---------------------------------------------------------------------------
// NoOp provider
// ---------------------------------------------------------------------------

#[test]
fn noop_returns_empty() {
    let provider = NoOpProvider;
    let vec = provider.embed("hello world").unwrap();
    assert!(vec.is_empty());

    let batch = provider.embed_batch(&["a", "b", "c"]).unwrap();
    assert_eq!(batch.len(), 3);
    for v in &batch {
        assert!(v.is_empty());
    }
}

#[test]
fn noop_dimension_is_zero() {
    let provider = NoOpProvider;
    assert_eq!(provider.dimension(), 0);
    assert_eq!(provider.provider_name(), "none");
}

// ---------------------------------------------------------------------------
// Hash embed provider
// ---------------------------------------------------------------------------

#[test]
fn hash_embed_produces_correct_dimension() {
    let provider = HashEmbedProvider::new(128);
    let vec = provider.embed("rust programming language").unwrap();
    assert_eq!(vec.len(), 128);
    assert_eq!(provider.dimension(), 128);
    assert_eq!(provider.provider_name(), "hash");
}

#[test]
fn hash_embed_similar_texts_are_similar() {
    let provider = HashEmbedProvider::new(128);
    let a = provider.embed("rust programming").unwrap();
    let b = provider.embed("rust coding").unwrap();
    let sim = cosine_similarity(&a, &b);
    assert!(
        sim > 0.5,
        "similar texts should have cosine > 0.5, got {sim}"
    );
}

#[test]
fn hash_embed_different_texts_diverge() {
    let provider = HashEmbedProvider::new(128);
    let a = provider.embed("rust programming").unwrap();
    let b = provider.embed("banana recipe").unwrap();
    let sim = cosine_similarity(&a, &b);
    assert!(
        sim < 0.3,
        "different texts should have cosine < 0.3, got {sim}"
    );
}

// ---------------------------------------------------------------------------
// Object safety
// ---------------------------------------------------------------------------

#[test]
fn trait_is_object_safe() {
    // Ensure EmbeddingProvider can be used as a trait object
    let provider: Box<dyn EmbeddingProvider> = Box::new(HashEmbedProvider::new(64));
    let vec = provider.embed("test").unwrap();
    assert_eq!(vec.len(), 64);
    assert_eq!(provider.dimension(), 64);

    let noop: Box<dyn EmbeddingProvider> = Box::new(NoOpProvider);
    let vec2 = noop.embed("test").unwrap();
    assert!(vec2.is_empty());
    assert_eq!(noop.dimension(), 0);
}

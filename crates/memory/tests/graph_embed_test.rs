use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};
use metaygn_memory::graph::{GraphMemory, MemoryNode, NodeType, Scope, cosine_similarity};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_node_with_embedding(
    id: &str,
    label: &str,
    content: &str,
    embedding: Option<Vec<f32>>,
) -> MemoryNode {
    MemoryNode {
        id: id.to_owned(),
        node_type: NodeType::Lesson,
        scope: Scope::Global,
        label: label.to_owned(),
        content: content.to_owned(),
        embedding,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn semantic_search_returns_most_similar() {
    let provider = HashEmbedProvider::new(64);
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    let emb_rust_lang = provider.embed("rust programming language").unwrap();
    let emb_rust_sys = provider.embed("rust systems programming").unwrap();
    let emb_cake = provider.embed("chocolate cake recipe").unwrap();

    let n1 = make_node_with_embedding(
        "n1",
        "Rust lang",
        "rust programming language",
        Some(emb_rust_lang),
    );
    let n2 = make_node_with_embedding(
        "n2",
        "Rust systems",
        "rust systems programming",
        Some(emb_rust_sys),
    );
    let n3 = make_node_with_embedding("n3", "Cake", "chocolate cake recipe", Some(emb_cake));

    gm.insert_node(&n1).await.unwrap();
    gm.insert_node(&n2).await.unwrap();
    gm.insert_node(&n3).await.unwrap();

    let query = provider.embed("rust code").unwrap();
    let results = gm.semantic_search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 3, "should return all 3 nodes");

    // The two rust-related nodes should rank higher than the cake node.
    let ids: Vec<&str> = results.iter().map(|r| r.0.id.as_str()).collect();
    let cake_pos = ids.iter().position(|&id| id == "n3").unwrap();
    let rust1_pos = ids.iter().position(|&id| id == "n1").unwrap();
    let rust2_pos = ids.iter().position(|&id| id == "n2").unwrap();

    assert!(
        rust1_pos < cake_pos,
        "rust lang node (pos {rust1_pos}) should rank higher than cake node (pos {cake_pos})"
    );
    assert!(
        rust2_pos < cake_pos,
        "rust systems node (pos {rust2_pos}) should rank higher than cake node (pos {cake_pos})"
    );

    // Verify scores are in descending order.
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "scores should be descending: {} >= {}",
            results[i - 1].1,
            results[i].1
        );
    }

    // Verify the returned scores are correct cosine similarities.
    let rust_lang_emb = provider.embed("rust programming language").unwrap();
    let expected_sim = cosine_similarity(&query, &rust_lang_emb);
    let actual_sim = results
        .iter()
        .find(|r| r.0.id == "n1")
        .map(|r| r.1)
        .unwrap();
    assert!(
        (actual_sim - expected_sim).abs() < 1e-6,
        "score for n1 should match cosine_similarity: got {actual_sim}, expected {expected_sim}"
    );
}

#[tokio::test]
async fn semantic_search_skips_nodes_without_embedding() {
    let provider = HashEmbedProvider::new(64);
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    let emb = provider.embed("rust programming").unwrap();

    let with_emb = make_node_with_embedding(
        "with",
        "With embedding",
        "rust programming",
        Some(emb.clone()),
    );
    let without_emb = make_node_with_embedding("without", "No embedding", "also about rust", None);

    gm.insert_node(&with_emb).await.unwrap();
    gm.insert_node(&without_emb).await.unwrap();

    let query = provider.embed("rust code").unwrap();
    let results = gm.semantic_search(&query, 10).await.unwrap();

    assert_eq!(
        results.len(),
        1,
        "should return only the node with embedding"
    );
    assert_eq!(results[0].0.id, "with");
}

use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};
use metaygn_memory::graph::{GraphMemory, MemoryNode, NodeType, Scope};

fn make_node(id: &str, label: &str, content: &str, embedding: Vec<f32>) -> MemoryNode {
    MemoryNode {
        id: id.to_string(),
        node_type: NodeType::Lesson,
        scope: Scope::Project,
        label: label.to_string(),
        content: content.to_string(),
        embedding: Some(embedding),
        created_at: "2026-03-01T00:00:00Z".to_string(),
        access_count: 0,
    }
}

#[tokio::test]
async fn adaptive_recall_favors_rewarded_nodes() {
    let provider = HashEmbedProvider::new(64);
    let gm = GraphMemory::open_in_memory().await.unwrap();

    // Two nodes with similar content so their cosine scores are close
    let emb_a = provider.embed("rust async concurrency patterns").unwrap();
    let emb_b = provider.embed("rust async concurrency design").unwrap();

    let node_a = make_node("a", "patterns", "rust async concurrency patterns", emb_a);
    let node_b = make_node("b", "design", "rust async concurrency design", emb_b);

    gm.insert_node(&node_a).await.unwrap();
    gm.insert_node(&node_b).await.unwrap();

    // Reward node A heavily — enough that exploitation dominates exploration
    for _ in 0..50 {
        gm.record_recall_reward("a", 2.0).await.unwrap();
    }
    // Give node B a few low-reward hits so its exploration bonus is modest
    for _ in 0..5 {
        gm.record_recall_reward("b", 0.05).await.unwrap();
    }

    // Query with an embedding close to both
    let query_emb = provider.embed("rust async concurrency").unwrap();
    let results = gm.adaptive_recall(&query_emb, 10).await.unwrap();

    assert!(results.len() >= 2, "expected at least 2 results");

    // Node A (heavily rewarded) should rank first
    assert_eq!(
        results[0].0.id, "a",
        "rewarded node should rank first; got '{}' with score {}, second '{}' with score {}",
        results[0].0.id, results[0].1, results[1].0.id, results[1].1,
    );
}

#[tokio::test]
async fn adaptive_recall_explores_unvisited_nodes() {
    let provider = HashEmbedProvider::new(64);
    let gm = GraphMemory::open_in_memory().await.unwrap();

    // Two nodes with similar embeddings
    let emb_a = provider.embed("memory graph exploration strategy").unwrap();
    let emb_b = provider.embed("memory graph exploration tactics").unwrap();

    let node_a = make_node("a", "strategy", "memory graph exploration strategy", emb_a);
    let node_b = make_node("b", "tactics", "memory graph exploration tactics", emb_b);

    gm.insert_node(&node_a).await.unwrap();
    gm.insert_node(&node_b).await.unwrap();

    // Node A: many hits with low reward => low mean reward, low exploration bonus
    for _ in 0..50 {
        gm.record_recall_reward("a", 0.1).await.unwrap();
    }
    // Node B: never recalled => high exploration bonus (UCB favors unexplored)

    let query_emb = provider.embed("memory graph exploration").unwrap();
    let results = gm.adaptive_recall(&query_emb, 10).await.unwrap();

    assert!(results.len() >= 2, "expected at least 2 results");

    // Node B (unvisited) should get exploration bonus and rank first
    assert_eq!(
        results[0].0.id, "b",
        "unvisited node should rank first due to exploration bonus; got '{}' with score {}, second '{}' with score {}",
        results[0].0.id, results[0].1, results[1].0.id, results[1].1,
    );
}

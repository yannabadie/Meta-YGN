use std::sync::Arc;

use metaygn_memory::fts::{SearchSource, UnifiedSearch};
use metaygn_memory::graph::{GraphMemory, MemoryNode, NodeType, Scope};
use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn search_finds_events() {
    let store = Arc::new(MemoryStore::open_in_memory().await.unwrap());
    let graph = Arc::new(GraphMemory::open_in_memory().await.unwrap());

    store
        .log_event("sess1", "tool_gated", "bash command was denied by risk gate")
        .await
        .unwrap();

    let search = UnifiedSearch::new(store, graph);
    let results = search.search("bash", 10).await.unwrap();

    assert!(!results.is_empty(), "should find the event");
    assert!(matches!(results[0].source, SearchSource::Event));
    assert!(results[0].content.contains("bash"));
}

#[tokio::test]
async fn search_finds_graph_nodes() {
    let store = Arc::new(MemoryStore::open_in_memory().await.unwrap());
    let graph = Arc::new(GraphMemory::open_in_memory().await.unwrap());

    let node = MemoryNode {
        id: "node-1".into(),
        node_type: NodeType::Lesson,
        scope: Scope::Session,
        label: "login fix".into(),
        content: "Fixed the login authentication bug in the user service".into(),
        embedding: None,
        created_at: "2026-03-01T00:00:00Z".into(),
        access_count: 0,
    };
    graph.insert_node(&node).await.unwrap();

    let search = UnifiedSearch::new(store, graph);
    let results = search.search("login", 10).await.unwrap();

    assert!(!results.is_empty(), "should find the graph node");
    assert!(matches!(results[0].source, SearchSource::GraphNode));
    assert!(results[0].content.contains("login"));
}

#[tokio::test]
async fn results_merged() {
    let store = Arc::new(MemoryStore::open_in_memory().await.unwrap());
    let graph = Arc::new(GraphMemory::open_in_memory().await.unwrap());

    // Insert an event containing "deploy"
    store
        .log_event("sess1", "session_started", "deploy pipeline initialized")
        .await
        .unwrap();

    // Insert a graph node containing "deploy"
    let node = MemoryNode {
        id: "node-deploy".into(),
        node_type: NodeType::Task,
        scope: Scope::Project,
        label: "deploy task".into(),
        content: "deploy the application to production".into(),
        embedding: None,
        created_at: "2026-03-01T00:00:00Z".into(),
        access_count: 0,
    };
    graph.insert_node(&node).await.unwrap();

    let search = UnifiedSearch::new(store, graph);
    let results = search.search("deploy", 10).await.unwrap();

    assert!(results.len() >= 2, "should find both event and graph node, got {}", results.len());

    let has_event = results.iter().any(|r| matches!(r.source, SearchSource::Event));
    let has_graph = results.iter().any(|r| matches!(r.source, SearchSource::GraphNode));

    assert!(has_event, "should contain an event result");
    assert!(has_graph, "should contain a graph node result");
}

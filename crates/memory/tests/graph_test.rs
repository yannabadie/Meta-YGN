use metaygn_memory::graph::{
    EdgeType, GraphMemory, MemoryEdge, MemoryNode, NodeType, Scope, cosine_similarity,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_node(
    id: &str,
    node_type: NodeType,
    scope: Scope,
    label: &str,
    content: &str,
) -> MemoryNode {
    MemoryNode {
        id: id.to_owned(),
        node_type,
        scope,
        label: label.to_owned(),
        content: content.to_owned(),
        embedding: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        access_count: 0,
    }
}

fn make_edge(source: &str, target: &str, edge_type: EdgeType) -> MemoryEdge {
    MemoryEdge {
        source_id: source.to_owned(),
        target_id: target.to_owned(),
        edge_type,
        weight: 1.0,
        metadata: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_and_get_node() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    let node = make_node(
        "n1",
        NodeType::Task,
        Scope::Session,
        "My Task",
        "Do something important",
    );
    gm.insert_node(&node).await.expect("insert node");

    let fetched = gm.get_node("n1").await.expect("get node");
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.id, "n1");
    assert_eq!(fetched.node_type, NodeType::Task);
    assert_eq!(fetched.scope, Scope::Session);
    assert_eq!(fetched.label, "My Task");
    assert_eq!(fetched.content, "Do something important");
    assert_eq!(fetched.access_count, 0);

    // Non-existent node returns None
    let missing = gm.get_node("does-not-exist").await.expect("get missing");
    assert!(missing.is_none());
}

#[tokio::test]
async fn insert_edge_and_find_neighbors() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    // Create chain: A -> B -> C
    let a = make_node("a", NodeType::Task, Scope::Project, "Node A", "content A");
    let b = make_node(
        "b",
        NodeType::Decision,
        Scope::Project,
        "Node B",
        "content B",
    );
    let c = make_node(
        "c",
        NodeType::Evidence,
        Scope::Project,
        "Node C",
        "content C",
    );
    gm.insert_node(&a).await.unwrap();
    gm.insert_node(&b).await.unwrap();
    gm.insert_node(&c).await.unwrap();

    gm.insert_edge(&make_edge("a", "b", EdgeType::DependsOn))
        .await
        .unwrap();
    gm.insert_edge(&make_edge("b", "c", EdgeType::Produces))
        .await
        .unwrap();

    // Neighbors of B at depth 1: should include A and C
    let neighbors = gm.find_neighbors("b", 1).await.unwrap();
    assert_eq!(neighbors.len(), 2);
    let ids: Vec<&str> = neighbors.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains(&"a"));
    assert!(ids.contains(&"c"));

    // Neighbors of A at depth 1: should include only B
    let neighbors_a = gm.find_neighbors("a", 1).await.unwrap();
    assert_eq!(neighbors_a.len(), 1);
    assert_eq!(neighbors_a[0].id, "b");

    // Neighbors of A at depth 2: should include B and C
    let neighbors_a2 = gm.find_neighbors("a", 2).await.unwrap();
    assert_eq!(neighbors_a2.len(), 2);
    let ids2: Vec<&str> = neighbors_a2.iter().map(|n| n.id.as_str()).collect();
    assert!(ids2.contains(&"b"));
    assert!(ids2.contains(&"c"));
}

#[tokio::test]
async fn search_content_fts() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    let n1 = make_node(
        "s1",
        NodeType::Lesson,
        Scope::Global,
        "Rust ownership",
        "Rust enforces ownership rules at compile time",
    );
    let n2 = make_node(
        "s2",
        NodeType::Lesson,
        Scope::Global,
        "Python GIL",
        "Python has a Global Interpreter Lock",
    );
    let n3 = make_node(
        "s3",
        NodeType::Lesson,
        Scope::Global,
        "Concurrency",
        "Concurrent programming requires careful synchronization",
    );
    gm.insert_node(&n1).await.unwrap();
    gm.insert_node(&n2).await.unwrap();
    gm.insert_node(&n3).await.unwrap();

    // Search for "ownership" should find only n1
    let results = gm.search_content("ownership", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "s1");

    // Search for "Python" should find only n2
    let results = gm.search_content("Python", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "s2");

    // Search for "Global" should find n2 (label "Python GIL" doesn't match,
    // but content has "Global Interpreter Lock") and n1 label doesn't match.
    // Actually n1 label "Rust ownership" and n3 "Concurrency" don't have "Global".
    // Only n2 content has "Global".
    let results = gm.search_content("Global", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "s2");
}

#[tokio::test]
async fn nodes_by_type_filters() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    gm.insert_node(&make_node(
        "t1",
        NodeType::Task,
        Scope::Session,
        "Task 1",
        "c1",
    ))
    .await
    .unwrap();
    gm.insert_node(&make_node(
        "t2",
        NodeType::Task,
        Scope::Session,
        "Task 2",
        "c2",
    ))
    .await
    .unwrap();
    gm.insert_node(&make_node(
        "d1",
        NodeType::Decision,
        Scope::Session,
        "Dec 1",
        "c3",
    ))
    .await
    .unwrap();
    gm.insert_node(&make_node(
        "e1",
        NodeType::Error,
        Scope::Global,
        "Err 1",
        "c4",
    ))
    .await
    .unwrap();

    let tasks = gm.nodes_by_type(NodeType::Task, 100).await.unwrap();
    assert_eq!(tasks.len(), 2);
    for t in &tasks {
        assert_eq!(t.node_type, NodeType::Task);
    }

    let decisions = gm.nodes_by_type(NodeType::Decision, 100).await.unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].id, "d1");

    let agents = gm.nodes_by_type(NodeType::Agent, 100).await.unwrap();
    assert_eq!(agents.len(), 0);
}

#[tokio::test]
async fn nodes_by_scope_filters() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    gm.insert_node(&make_node("s1", NodeType::Task, Scope::Session, "S1", "c1"))
        .await
        .unwrap();
    gm.insert_node(&make_node(
        "s2",
        NodeType::Decision,
        Scope::Session,
        "S2",
        "c2",
    ))
    .await
    .unwrap();
    gm.insert_node(&make_node("p1", NodeType::Task, Scope::Project, "P1", "c3"))
        .await
        .unwrap();
    gm.insert_node(&make_node(
        "g1",
        NodeType::Lesson,
        Scope::Global,
        "G1",
        "c4",
    ))
    .await
    .unwrap();

    let session_nodes = gm.nodes_by_scope(Scope::Session, 100).await.unwrap();
    assert_eq!(session_nodes.len(), 2);
    for n in &session_nodes {
        assert_eq!(n.scope, Scope::Session);
    }

    let project_nodes = gm.nodes_by_scope(Scope::Project, 100).await.unwrap();
    assert_eq!(project_nodes.len(), 1);
    assert_eq!(project_nodes[0].id, "p1");

    let global_nodes = gm.nodes_by_scope(Scope::Global, 100).await.unwrap();
    assert_eq!(global_nodes.len(), 1);
    assert_eq!(global_nodes[0].id, "g1");
}

#[tokio::test]
async fn cosine_similarity_basic() {
    // Identical vectors -> 1.0
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!(
        (sim - 1.0).abs() < 1e-6,
        "identical vectors should be 1.0, got {sim}"
    );

    // Orthogonal vectors -> 0.0
    let c = vec![1.0, 0.0, 0.0];
    let d = vec![0.0, 1.0, 0.0];
    let sim2 = cosine_similarity(&c, &d);
    assert!(
        sim2.abs() < 1e-6,
        "orthogonal vectors should be 0.0, got {sim2}"
    );

    // Opposite vectors -> -1.0
    let e = vec![1.0, 0.0];
    let f = vec![-1.0, 0.0];
    let sim3 = cosine_similarity(&e, &f);
    assert!(
        (sim3 + 1.0).abs() < 1e-6,
        "opposite vectors should be -1.0, got {sim3}"
    );

    // Empty -> 0.0
    let sim4 = cosine_similarity(&[], &[]);
    assert_eq!(sim4, 0.0);

    // Mismatched lengths -> 0.0
    let sim5 = cosine_similarity(&[1.0, 2.0], &[1.0]);
    assert_eq!(sim5, 0.0);

    // Zero vector -> 0.0
    let sim6 = cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]);
    assert_eq!(sim6, 0.0);
}

#[tokio::test]
async fn node_and_edge_counts() {
    let gm = GraphMemory::open_in_memory().await.expect("open graph");

    assert_eq!(gm.node_count().await.unwrap(), 0);
    assert_eq!(gm.edge_count().await.unwrap(), 0);

    gm.insert_node(&make_node("n1", NodeType::Task, Scope::Session, "N1", "c1"))
        .await
        .unwrap();
    gm.insert_node(&make_node("n2", NodeType::Task, Scope::Session, "N2", "c2"))
        .await
        .unwrap();
    gm.insert_node(&make_node("n3", NodeType::Code, Scope::Global, "N3", "c3"))
        .await
        .unwrap();

    assert_eq!(gm.node_count().await.unwrap(), 3);
    assert_eq!(gm.edge_count().await.unwrap(), 0);

    gm.insert_edge(&make_edge("n1", "n2", EdgeType::DependsOn))
        .await
        .unwrap();
    gm.insert_edge(&make_edge("n2", "n3", EdgeType::Produces))
        .await
        .unwrap();

    assert_eq!(gm.node_count().await.unwrap(), 3);
    assert_eq!(gm.edge_count().await.unwrap(), 2);
}

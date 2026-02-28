//! Graph memory API endpoints.
//!
//! Exposes the [`GraphMemory`] store over HTTP for inserting nodes/edges,
//! full-text search, and statistics.

use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::{get, post}};
use serde::Deserialize;
use serde_json::{json, Value};

use metaygn_memory::graph::{MemoryEdge, MemoryNode};

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for `POST /memory/graph/search`.
#[derive(Deserialize)]
pub struct GraphSearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /memory/nodes -- Insert a MemoryNode.
async fn insert_node(
    State(state): State<AppState>,
    Json(node): Json<MemoryNode>,
) -> Json<Value> {
    match state.graph.insert_node(&node).await {
        Ok(()) => Json(json!({ "ok": true, "id": node.id })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// POST /memory/edges -- Insert a MemoryEdge.
async fn insert_edge(
    State(state): State<AppState>,
    Json(edge): Json<MemoryEdge>,
) -> Json<Value> {
    match state.graph.insert_edge(&edge).await {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// POST /memory/graph/search -- FTS search over graph nodes.
async fn graph_search(
    State(state): State<AppState>,
    Json(req): Json<GraphSearchRequest>,
) -> Json<Value> {
    let limit = req.limit.unwrap_or(10);
    match state.graph.search_content(&req.query, limit).await {
        Ok(nodes) => {
            let items: Vec<Value> = nodes
                .into_iter()
                .map(|n| {
                    json!({
                        "id": n.id,
                        "node_type": n.node_type,
                        "scope": n.scope,
                        "label": n.label,
                        "content": n.content,
                        "created_at": n.created_at,
                        "access_count": n.access_count,
                    })
                })
                .collect();
            Json(json!({ "nodes": items }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /memory/graph/stats -- Return node and edge counts.
async fn graph_stats(State(state): State<AppState>) -> Json<Value> {
    let node_count = state.graph.node_count().await.unwrap_or(0);
    let edge_count = state.graph.edge_count().await.unwrap_or(0);
    Json(json!({
        "node_count": node_count,
        "edge_count": edge_count,
    }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/memory/nodes", post(insert_node))
        .route("/memory/edges", post(insert_edge))
        .route("/memory/graph/search", post(graph_search))
        .route("/memory/graph/stats", get(graph_stats))
}

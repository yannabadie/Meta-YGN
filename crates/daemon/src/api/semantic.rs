//! Semantic search API endpoint.
//!
//! Exposes a `POST /memory/semantic` route that embeds the incoming query
//! via the shared [`EmbeddingProvider`] from [`AppState`] and performs
//! cosine-similarity search over all graph nodes that have stored embeddings.

use axum::Router;
use axum::extract::State;
use axum::response::Json;
use axum::routing::post;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Body for `POST /memory/semantic`.
#[derive(Deserialize)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

/// A single result in the semantic search response.
#[derive(Serialize)]
struct SemanticResult {
    id: String,
    label: String,
    content: String,
    score: f32,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /memory/semantic -- vector-similarity search over graph nodes.
///
/// Uses the shared `EmbeddingProvider` from `AppState`, which is either
/// `FastEmbedProvider` (384d, when the `embeddings` feature is enabled) or
/// `HashEmbedProvider` (128d, fallback).  Nodes are embedded with the same
/// provider at insert time, so dimensions always match.
async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Json<Value> {
    let limit = req.limit.unwrap_or(10);

    let query_emb = match state.embedding.embed(&req.query) {
        Ok(emb) => emb,
        Err(e) => {
            return Json(json!({ "error": format!("embedding failed: {e}") }));
        }
    };

    match state.graph.semantic_search(&query_emb, limit).await {
        Ok(results) => {
            let all_zero = !results.is_empty() && results.iter().all(|(_, s)| *s == 0.0);
            if all_zero {
                tracing::warn!(
                    "semantic search returned all-zero scores — possible embedding dimension mismatch \
                     (query dim={}, stored nodes may use a different provider)",
                    query_emb.len()
                );
            }
            let items: Vec<SemanticResult> = results
                .into_iter()
                .map(|(node, score)| SemanticResult {
                    id: node.id,
                    label: node.label,
                    content: node.content,
                    score,
                })
                .collect();
            Json(json!({
                "results": items,
                "provider": state.embedding.provider_name(),
                "dimension_warning": all_zero,
            }))
        }
        Err(e) => Json(json!({ "error": format!("semantic search failed: {e}") })),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new().route("/memory/semantic", post(semantic_search))
}

//! Semantic search API endpoint.
//!
//! Exposes a `POST /memory/semantic` route that embeds the incoming query
//! via [`HashEmbedProvider`] and performs cosine-similarity search over all
//! graph nodes that have stored embeddings.

use axum::extract::State;
use axum::response::Json;
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};

use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};

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
async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Json<Value> {
    let limit = req.limit.unwrap_or(10);
    let provider = HashEmbedProvider::new(64);

    let query_emb = match provider.embed(&req.query) {
        Ok(emb) => emb,
        Err(e) => {
            return Json(json!({ "error": format!("embedding failed: {e}") }));
        }
    };

    match state.graph.semantic_search(&query_emb, limit).await {
        Ok(results) => {
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
                "provider": provider.provider_name(),
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

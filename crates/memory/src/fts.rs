use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::embeddings::EmbeddingProvider;
use crate::graph::GraphMemory;
use crate::store::MemoryStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSource {
    Event,
    GraphNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source: SearchSource,
    pub id: String,
    pub content: String,
    pub score: f64,
}

pub struct UnifiedSearch {
    store: Arc<MemoryStore>,
    graph: Arc<GraphMemory>,
    embedding: Arc<dyn EmbeddingProvider>,
}

impl UnifiedSearch {
    pub fn new(
        store: Arc<MemoryStore>,
        graph: Arc<GraphMemory>,
        embedding: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            store,
            graph,
            embedding,
        }
    }

    pub async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<SearchResult>> {
        // 1. Search events with real BM25 scores from FTS5
        let event_rows = self.store.search_events_ranked(query, limit).await?;
        // FTS5 rank is negative (closer to 0 = better). Normalize to [0.5, 1.0]
        // so even a single result gets a competitive score (not 0.0).
        let max_abs_rank = event_rows
            .iter()
            .map(|(_, r)| r.abs())
            .fold(0.0_f64, f64::max)
            .max(1.0);
        let mut results: Vec<SearchResult> = event_rows
            .into_iter()
            .map(|(row, rank)| SearchResult {
                source: SearchSource::Event,
                id: row.id,
                content: row.payload,
                score: 0.5 + 0.5 * (1.0 - rank.abs() / max_abs_rank),
            })
            .collect();

        // 2. Try UCB-scored adaptive recall if embedding is available
        let graph_results = match self.embedding.embed(query) {
            Ok(query_emb) if !query_emb.is_empty() => {
                let adaptive = self
                    .graph
                    .adaptive_recall(&query_emb, limit)
                    .await
                    .unwrap_or_default();
                if adaptive.is_empty() {
                    // No nodes had embeddings stored; fall back to FTS5
                    fts_fallback_graph(&self.graph, query, limit).await
                } else {
                    adaptive
                        .into_iter()
                        .map(|(node, score)| SearchResult {
                            source: SearchSource::GraphNode,
                            id: node.id,
                            content: node.content,
                            score: score as f64,
                        })
                        .collect::<Vec<_>>()
                }
            }
            _ => {
                // Embedding provider returned error or empty; FTS5 fallback
                fts_fallback_graph(&self.graph, query, limit).await
            }
        };
        results.extend(graph_results);

        // 3. Sort by score descending, truncate
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit as usize);

        Ok(results)
    }
}

/// Fallback: FTS5 on graph with position-based score (degraded mode).
async fn fts_fallback_graph(
    graph: &GraphMemory,
    query: &str,
    limit: u32,
) -> Vec<SearchResult> {
    graph
        .search_content(query, limit)
        .await
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, node)| SearchResult {
            source: SearchSource::GraphNode,
            id: node.id,
            content: node.content,
            score: 0.5 - (i as f64 * 0.01), // degraded fallback
        })
        .collect()
}

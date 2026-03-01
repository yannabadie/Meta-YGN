use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
}

impl UnifiedSearch {
    pub fn new(store: Arc<MemoryStore>, graph: Arc<GraphMemory>) -> Self {
        Self { store, graph }
    }

    pub async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<SearchResult>> {
        // 1. Search events via store.search_events()
        let event_rows = self.store.search_events(query, limit).await?;
        let mut results: Vec<SearchResult> = event_rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| SearchResult {
                source: SearchSource::Event,
                id: row.id,
                content: row.payload,
                // Events have FTS5 rank scores; approximate with position-based score
                score: 1.0 - (i as f64 * 0.01),
            })
            .collect();

        // 2. Search graph via graph.search_content()
        let graph_nodes = self.graph.search_content(query, limit).await?;
        let graph_results: Vec<SearchResult> = graph_nodes
            .into_iter()
            .enumerate()
            .map(|(i, node)| SearchResult {
                source: SearchSource::GraphNode,
                id: node.id,
                content: node.content,
                // Graph nodes scored after events
                score: 0.5 - (i as f64 * 0.01),
            })
            .collect();

        // 3. Merge results into a single Vec<SearchResult>
        results.extend(graph_results);

        // 4. Sort by relevance (descending score; events first since higher scores)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 5. Truncate to limit
        results.truncate(limit as usize);

        Ok(results)
    }
}

//! Policy traits for pluggable metacognitive strategies.
//!
//! These traits define the extension points where an OpenSage-inspired
//! research engine can propose alternative behaviors under Aletheia control.
//! The current implementations are the stable defaults; future experimental
//! implementations can be swapped in behind feature flags.

use metaygn_shared::state::{RiskLevel, TaskType};

use crate::topology::ExecutionPlan;

/// Policy for selecting execution topology.
///
/// The default implementation is `TopologyPlanner` which uses static rules.
/// An OpenSage implementation might use LLM-driven decomposition.
pub trait TopologyPolicy: Send + Sync {
    fn plan(&self, risk: RiskLevel, difficulty: f32, task_type: TaskType) -> ExecutionPlan;
}

/// Strategy for ranking memory search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingStrategy {
    /// FTS5 BM25 text relevance only.
    TextOnly,
    /// UCB-scored adaptive recall with exploration bonus.
    AdaptiveUcb,
    /// Cosine similarity from embeddings.
    Semantic,
}

/// Policy for memory persistence and retrieval.
pub trait MemoryPolicy: Send + Sync {
    /// Whether a given content string should be persisted to the graph.
    fn should_persist(&self, content: &str, node_type: &str) -> bool;
    /// Which ranking strategy to use for recall.
    fn ranking_strategy(&self) -> RankingStrategy;
}

/// Policy for tool synthesis (Forge) admission control.
pub trait ToolSynthesisPolicy: Send + Sync {
    /// Whether to attempt forging a verification tool for this file.
    fn should_forge(&self, file_path: &str, tool_name: &str) -> bool;
}

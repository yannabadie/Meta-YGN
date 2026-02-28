use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Categorizes the type of task the agent is performing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    Bugfix,
    Feature,
    Refactor,
    Architecture,
    Security,
    Research,
    Release,
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// A signed description of a task, including metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSignature {
    pub id: Uuid,
    pub task_type: TaskType,
    pub risk: RiskLevel,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

/// A 5-dimensional vector representing the agent's metacognitive state.
///
/// All values are expected to be in the range `[0.0, 1.0]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitiveVector {
    pub confidence: f64,
    pub coherence: f64,
    pub grounding: f64,
    pub complexity: f64,
    pub progress: f64,
}

impl MetacognitiveVector {
    /// Computes an overall quality score as:
    ///
    /// `(confidence + coherence + grounding + (1 - complexity) + progress) / 5`
    ///
    /// Complexity is inverted because higher complexity reduces quality.
    pub fn overall_quality(&self) -> f64 {
        (self.confidence + self.coherence + self.grounding + (1.0 - self.complexity) + self.progress)
            / 5.0
    }

    /// Encodes the vector into a compact string representation:
    ///
    /// `"META:c{n}h{n}g{n}x{n}p{n}"` where `n = (value * 9) as u8`.
    pub fn compact_encode(&self) -> String {
        let c = (self.confidence * 9.0) as u8;
        let h = (self.coherence * 9.0) as u8;
        let g = (self.grounding * 9.0) as u8;
        let x = (self.complexity * 9.0) as u8;
        let p = (self.progress * 9.0) as u8;
        format!("META:c{c}h{h}g{g}x{x}p{p}")
    }
}

/// Tracks token/cost/latency budgets for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetState {
    pub max_tokens: u64,
    pub consumed_tokens: u64,
    pub max_latency_ms: u64,
    pub max_cost_usd: f64,
    pub risk_tolerance: RiskLevel,
}

impl BudgetState {
    /// Returns the number of tokens remaining.
    pub fn tokens_remaining(&self) -> u64 {
        self.max_tokens.saturating_sub(self.consumed_tokens)
    }

    /// Returns the fraction of the token budget that has been consumed (0.0 - 1.0).
    pub fn utilization(&self) -> f64 {
        if self.max_tokens == 0 {
            return 0.0;
        }
        self.consumed_tokens as f64 / self.max_tokens as f64
    }
}

/// Reasoning strategy the agent may adopt.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Strategy {
    StepByStep,
    TreeExplore,
    VerifyFirst,
    DivideConquer,
    Analogical,
    Adversarial,
    Rapid,
    Iterative,
}

/// A decision the agent can make about how to proceed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Decision {
    Continue,
    Revise,
    Abstain,
    Escalate,
    Stop,
}

/// Tier of evidence supporting a claim or action.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EvidenceTier {
    Confirmed,
    Experimental,
    Unverified,
}

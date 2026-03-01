use serde::{Deserialize, Serialize};

/// A structured trajectory for RL2F-style fine-tuning data export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rl2fTrajectory {
    pub session_id: String,
    pub task_type: Option<String>,
    pub risk_level: String,
    pub strategy_used: String,
    pub initial_attempt: Option<String>,
    pub verifiable_error: Option<String>,
    pub critique_injected: Option<String>,
    pub revised_attempt: Option<String>,
    pub success: bool,
    pub overconfidence_score: f64,
    pub plasticity_level: String,
    pub confidence: f64,
    pub coherence: f64,
    pub grounding: f64,
    pub timestamp: String,
    pub signature_hash: Option<String>,
}

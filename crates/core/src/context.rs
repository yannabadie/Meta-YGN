use serde::Serialize;

use metaygn_shared::protocol::HookInput;
use metaygn_shared::state::*;

/// Mutable context that flows through all 12 stages of the control loop.
///
/// Each stage reads and/or writes fields on this struct. The runner passes
/// it through the pipeline sequentially so every stage can observe what
/// previous stages decided.
#[derive(Serialize)]
pub struct LoopContext {
    /// The raw hook event that triggered this loop iteration.
    pub input: HookInput,

    /// Classified task type (set by `classify` stage).
    pub task_type: Option<TaskType>,

    /// Risk level for the current operation (set by `assess` stage).
    pub risk: RiskLevel,

    /// Entropy / difficulty estimate in `[0.0, 1.0]` (set by `assess` stage).
    pub difficulty: f32,

    /// Self-assessed competence for this task in `[0.0, 1.0]` (set by `competence` stage).
    pub competence: f32,

    /// Whether a tool invocation is required (set by `tool_need` stage).
    pub tool_necessary: bool,

    /// Token / cost / latency budget (set by `budget` stage).
    pub budget: BudgetState,

    /// Selected reasoning strategy (set by `strategy` stage).
    pub strategy: Strategy,

    /// Final decision (set by `decide` stage, may be overridden by escalation).
    pub decision: Decision,

    /// 5-D metacognitive state vector (updated by `calibrate` stage).
    pub metacog_vector: MetacognitiveVector,

    /// Results from the `verify` stage.
    pub verification_results: Vec<String>,

    /// Lessons learned (populated by `learn` stage and escalation events).
    pub lessons: Vec<String>,
}

impl LoopContext {
    /// Create a new context from a hook input with sensible defaults.
    pub fn new(input: HookInput) -> Self {
        Self {
            input,
            task_type: None,
            risk: RiskLevel::Low,
            difficulty: 0.5,
            competence: 0.7,
            tool_necessary: false,
            budget: BudgetState {
                max_tokens: 5000,
                consumed_tokens: 0,
                max_latency_ms: 30_000,
                max_cost_usd: 0.10,
                risk_tolerance: RiskLevel::Medium,
            },
            strategy: Strategy::StepByStep,
            decision: Decision::Continue,
            metacog_vector: MetacognitiveVector {
                confidence: 0.5,
                coherence: 0.5,
                grounding: 0.5,
                complexity: 0.5,
                progress: 0.0,
            },
            verification_results: Vec::new(),
            lessons: Vec::new(),
        }
    }
}

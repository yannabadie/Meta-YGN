//! Dynamic topology planner for the metacognitive control loop.
//!
//! Selects which subset of the 12 control loop stages to execute based on
//! task characteristics. The key insight is that **skipping** unnecessary
//! stages is what matters -- a trivial task should run 4 stages, not 12.

use metaygn_shared::state::{RiskLevel, TaskType, Topology};

/// The ordered names of all 12 stages in the default pipeline.
///
/// Must match the stage names returned by each `Stage::name()` in `runner.rs`.
pub const ALL_STAGES: [&str; 12] = [
    "classify",
    "assess",
    "competence",
    "tool_need",
    "budget",
    "strategy",
    "act",
    "verify",
    "calibrate",
    "compact",
    "decide",
    "learn",
];

/// An execution plan: ordered subset of the 12 stages.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// The topology category that was selected.
    pub topology: Topology,
    /// Stage names to execute, in order.
    pub stages: Vec<&'static str>,
    /// Human-readable explanation of why this topology was chosen.
    pub rationale: String,
}

/// Plans execution topology based on task characteristics.
///
/// The planner examines risk level, difficulty, and task type to determine
/// the minimal set of stages needed. This transforms the fixed sequential
/// pipeline into a dynamic skip-routing system.
pub struct TopologyPlanner;

impl TopologyPlanner {
    /// Select an execution plan based on task characteristics.
    ///
    /// # Planning rules
    ///
    /// | Condition | Topology | Stages |
    /// |-----------|----------|--------|
    /// | `task_type == Security` (any risk) | Horizontal | all 12 + verify + calibrate (14) |
    /// | `task_type == Research` | Vertical | classify, assess, competence, strategy, act, learn (6) |
    /// | `risk == High` | Horizontal | all 12 + verify + calibrate (14) |
    /// | `risk == Low` AND `difficulty < 0.2` | Single | classify, assess, act, decide (4) |
    /// | otherwise | Vertical | all 12 sequential |
    pub fn plan(risk: RiskLevel, difficulty: f32, task_type: TaskType) -> ExecutionPlan {
        // Security always gets maximum scrutiny, regardless of risk.
        if task_type == TaskType::Security {
            return ExecutionPlan {
                topology: Topology::Horizontal,
                stages: Self::horizontal_stages(),
                rationale:
                    "Security tasks always receive double verification (Horizontal topology)".into(),
            };
        }

        // Research tasks skip heavy verification stages.
        if task_type == TaskType::Research {
            return ExecutionPlan {
                topology: Topology::Vertical,
                stages: vec![
                    "classify",
                    "assess",
                    "competence",
                    "strategy",
                    "act",
                    "learn",
                ],
                rationale:
                    "Research tasks use a slim 6-stage pipeline, skipping verification overhead"
                        .into(),
            };
        }

        // High risk gets double verification pass.
        if risk == RiskLevel::High {
            return ExecutionPlan {
                topology: Topology::Horizontal,
                stages: Self::horizontal_stages(),
                rationale:
                    "High-risk tasks receive double verify+calibrate pass (Horizontal topology)"
                        .into(),
            };
        }

        // Low risk + trivial difficulty => minimal pipeline.
        if risk == RiskLevel::Low && difficulty < 0.2 {
            return Self::trivial_pipeline();
        }

        // Default: full sequential pipeline.
        Self::full_pipeline()
    }

    /// Get the default full pipeline (all 12 stages).
    pub fn full_pipeline() -> ExecutionPlan {
        ExecutionPlan {
            topology: Topology::Vertical,
            stages: ALL_STAGES.to_vec(),
            rationale: "Standard full 12-stage sequential pipeline".into(),
        }
    }

    /// Get a minimal pipeline for trivial tasks (4 stages).
    pub fn trivial_pipeline() -> ExecutionPlan {
        ExecutionPlan {
            topology: Topology::Single,
            stages: vec!["classify", "assess", "act", "decide"],
            rationale: "Trivial task: skip unnecessary overhead, 4 stages only".into(),
        }
    }

    /// Build the horizontal (double-verify) stage list: all 12 + extra verify + calibrate.
    fn horizontal_stages() -> Vec<&'static str> {
        let mut stages = ALL_STAGES.to_vec();
        // Append a second verify + calibrate pass for extra safety.
        stages.push("verify");
        stages.push("calibrate");
        stages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_stages_has_twelve_entries() {
        assert_eq!(ALL_STAGES.len(), 12);
    }

    #[test]
    fn horizontal_stages_has_fourteen_entries() {
        let stages = TopologyPlanner::horizontal_stages();
        assert_eq!(stages.len(), 14);
        // The last two should be the double-pass of verify + calibrate.
        assert_eq!(stages[12], "verify");
        assert_eq!(stages[13], "calibrate");
    }
}

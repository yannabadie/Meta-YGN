use metaygn_shared::state::Decision;

use crate::context::LoopContext;
use crate::stages::*;
use crate::topology::ExecutionPlan;

/// Orchestrates the 12-stage metacognitive control loop.
///
/// Stages run sequentially. Each stage may read/write the shared
/// [`LoopContext`]. A stage can signal early exit via [`StageResult::Skip`]
/// or escalate to a human via [`StageResult::Escalate`].
pub struct ControlLoop {
    stages: Vec<Box<dyn Stage>>,
}

impl ControlLoop {
    /// Build a new control loop with the default 12-stage pipeline.
    pub fn new() -> Self {
        Self {
            stages: vec![
                Box::new(classify::ClassifyStage),     // 1. classify task type
                Box::new(assess::AssessStage),         // 2. assess difficulty + risk
                Box::new(competence::CompetenceStage), // 3. self-assess competence
                Box::new(tool_need::ToolNeedStage),    // 4. determine if tool needed
                Box::new(budget::BudgetStage),         // 5. allocate budget
                Box::new(strategy::StrategyStage),     // 6. select reasoning strategy
                Box::new(act::ActStage),               // 7. execute (no-op)
                Box::new(verify::VerifyStage),         // 8. verify tool output
                Box::new(calibrate::CalibrateStage),   // 9. calibrate metacog vector
                Box::new(compact::CompactStage),       // 10. memory compaction (no-op)
                Box::new(decide::DecideStage),         // 11. make decision
                Box::new(learn::LearnStage),           // 12. collect lessons
            ],
        }
    }

    /// Run the full pipeline on the given context, returning the final decision.
    pub fn run(&self, ctx: &mut LoopContext) -> Decision {
        for stage in &self.stages {
            tracing::trace!(stage = stage.name(), "entering stage");
            match stage.run(ctx) {
                StageResult::Continue => continue,
                StageResult::Skip => {
                    tracing::debug!(stage = stage.name(), "stage requested skip");
                    break;
                }
                StageResult::Escalate(reason) => {
                    ctx.decision = Decision::Escalate;
                    ctx.lessons
                        .push(format!("escalated at stage '{}': {}", stage.name(), reason));
                    tracing::warn!(
                        stage = stage.name(),
                        %reason,
                        "pipeline escalated"
                    );
                    break;
                }
            }
        }
        ctx.decision
    }

    /// Returns the number of stages in the pipeline.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Returns the names of all stages in order.
    pub fn stage_names(&self) -> Vec<&'static str> {
        self.stages.iter().map(|s| s.name()).collect()
    }

    /// Run a subset of stages (0-indexed range) on the given context.
    ///
    /// `start` is inclusive, `end` is exclusive. For example,
    /// `run_range(ctx, 0, 6)` runs stages 1-6 (classify through strategy).
    pub fn run_range(&self, ctx: &mut LoopContext, start: usize, end: usize) -> Decision {
        let end = end.min(self.stages.len());
        for stage in &self.stages[start..end] {
            tracing::trace!(stage = stage.name(), "entering stage (range)");
            match stage.run(ctx) {
                StageResult::Continue => continue,
                StageResult::Skip => {
                    tracing::debug!(stage = stage.name(), "stage requested skip (range)");
                    break;
                }
                StageResult::Escalate(reason) => {
                    ctx.decision = Decision::Escalate;
                    ctx.lessons
                        .push(format!("escalated at stage '{}': {}", stage.name(), reason));
                    tracing::warn!(
                        stage = stage.name(),
                        %reason,
                        "pipeline escalated (range)"
                    );
                    break;
                }
            }
        }
        ctx.decision
    }

    /// Run only the stages specified in an [`ExecutionPlan`].
    ///
    /// Stages are looked up by name and executed in the order listed in
    /// `plan.stages`. A stage name that appears more than once (e.g. the
    /// double verify+calibrate pass in Horizontal topology) will be executed
    /// each time it appears.
    pub fn run_plan(&self, ctx: &mut LoopContext, plan: &ExecutionPlan) -> Decision {
        for stage_name in &plan.stages {
            if let Some(stage) = self.stages.iter().find(|s| s.name() == *stage_name) {
                tracing::trace!(stage = stage.name(), topology = ?plan.topology, "entering stage (plan)");
                match stage.run(ctx) {
                    StageResult::Continue => continue,
                    StageResult::Skip => {
                        tracing::debug!(stage = stage.name(), "stage requested skip (plan)");
                        break;
                    }
                    StageResult::Escalate(reason) => {
                        ctx.decision = Decision::Escalate;
                        ctx.lessons.push(format!(
                            "escalated at stage '{}': {}",
                            stage.name(),
                            reason
                        ));
                        tracing::warn!(
                            stage = stage.name(),
                            %reason,
                            "pipeline escalated (plan)"
                        );
                        break;
                    }
                }
            }
        }
        ctx.decision
    }
}

impl Default for ControlLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_twelve_stages() {
        let cl = ControlLoop::new();
        assert_eq!(cl.stage_count(), 12);
    }

    #[test]
    fn stage_names_are_unique() {
        let cl = ControlLoop::new();
        let names = cl.stage_names();
        let mut deduped = names.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "duplicate stage names found");
    }
}

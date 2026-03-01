use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::TaskType;

/// Stage 3: Self-assess competence for the classified task type.
pub struct CompetenceStage;

impl Stage for CompetenceStage {
    fn name(&self) -> &'static str {
        "competence"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        ctx.competence = base_competence(ctx.task_type);

        // Lower competence when the prompt mentions unfamiliar domains.
        let prompt = ctx.input.prompt.as_deref().unwrap_or("").to_lowercase();
        let unfamiliar = [
            "kernel",
            "driver",
            "gpu",
            "cuda",
            "fpga",
            "assembly",
            "verilog",
            "quantum",
            "blockchain",
        ];
        let penalty = unfamiliar.iter().filter(|kw| prompt.contains(*kw)).count() as f32 * 0.1;

        ctx.competence = (ctx.competence - penalty).max(0.0);

        tracing::debug!(
            stage = self.name(),
            competence = ctx.competence,
            "assessed competence"
        );

        StageResult::Continue
    }
}

/// Default competence by task type. Security and architecture are harder,
/// bugfix and refactor are familiar territory.
fn base_competence(task_type: Option<TaskType>) -> f32 {
    match task_type {
        Some(TaskType::Bugfix) => 0.8,
        Some(TaskType::Feature) => 0.7,
        Some(TaskType::Refactor) => 0.8,
        Some(TaskType::Architecture) => 0.5,
        Some(TaskType::Security) => 0.4,
        Some(TaskType::Research) => 0.6,
        Some(TaskType::Release) => 0.7,
        None => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_lower_competence() {
        let c = base_competence(Some(TaskType::Security));
        assert!(c < 0.5, "security competence should be low, got {c}");
    }

    #[test]
    fn bugfix_higher_competence() {
        let c = base_competence(Some(TaskType::Bugfix));
        assert!(c >= 0.8, "bugfix competence should be high, got {c}");
    }
}

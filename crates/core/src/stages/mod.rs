pub mod act;
pub mod assess;
pub mod budget;
pub mod calibrate;
pub mod classify;
pub mod compact;
pub mod competence;
pub mod decide;
pub mod learn;
pub mod strategy;
pub mod tool_need;
pub mod verify;

use crate::context::LoopContext;

/// Result from running a single stage.
#[derive(Debug, Clone, PartialEq)]
pub enum StageResult {
    /// Proceed to the next stage.
    Continue,
    /// Skip remaining stages (early exit, not an error).
    Skip,
    /// Escalate to a human with the given reason.
    Escalate(String),
}

/// Trait that every pipeline stage implements.
pub trait Stage: Send + Sync {
    /// A unique, human-readable name for this stage (e.g. `"classify"`).
    fn name(&self) -> &'static str;

    /// Execute the stage logic, reading/writing fields on `ctx`.
    fn run(&self, ctx: &mut LoopContext) -> StageResult;
}

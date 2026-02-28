pub mod classify;
pub mod assess;
pub mod competence;
pub mod tool_need;
pub mod budget;
pub mod strategy;
pub mod act;
pub mod verify;
pub mod calibrate;
pub mod compact;
pub mod decide;
pub mod learn;

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
pub trait Stage {
    /// A unique, human-readable name for this stage (e.g. `"classify"`).
    fn name(&self) -> &'static str;

    /// Execute the stage logic, reading/writing fields on `ctx`.
    fn run(&self, ctx: &mut LoopContext) -> StageResult;
}

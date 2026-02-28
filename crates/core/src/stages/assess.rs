use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::RiskLevel;

/// Stage 2: Assess difficulty and risk from the prompt and tool context.
pub struct AssessStage;

impl Stage for AssessStage {
    fn name(&self) -> &'static str {
        "assess"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        let text = prompt_text(ctx);

        // Difficulty: based on prompt length and presence of complexity markers.
        ctx.difficulty = estimate_difficulty(&text);

        // Risk: based on tool name and keywords.
        ctx.risk = estimate_risk(ctx);

        tracing::debug!(
            stage = self.name(),
            difficulty = ctx.difficulty,
            risk = ?ctx.risk,
            "assessed task"
        );

        StageResult::Continue
    }
}

/// Gather all available text for analysis.
fn prompt_text(ctx: &LoopContext) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = &ctx.input.prompt {
        parts.push(String::clone(p));
    }
    if let Some(ti) = &ctx.input.tool_input {
        parts.push(ti.to_string());
    }
    parts.join(" ")
}

/// Estimate difficulty on a `[0.0, 1.0]` scale.
///
/// Heuristic:
/// - Base score from prompt length (longer = harder, capped at 0.6).
/// - Bonus for complexity keywords (each adds 0.1, capped at total 1.0).
fn estimate_difficulty(text: &str) -> f32 {
    let word_count = text.split_whitespace().count();
    let base = (word_count as f32 / 200.0).min(0.6);

    let complexity_keywords = [
        "complex", "concurren", "async", "parallel", "distributed",
        "performance", "optimize", "scale", "migration", "backward",
        "compatibility", "recursive", "cryptograph",
    ];
    let bonus = complexity_keywords
        .iter()
        .filter(|kw| text.to_lowercase().contains(*kw))
        .count() as f32
        * 0.1;

    (base + bonus).min(1.0)
}

/// Estimate risk level from tool name and keywords.
fn estimate_risk(ctx: &LoopContext) -> RiskLevel {
    let tool = ctx.input.tool_name.as_deref().unwrap_or("");
    let prompt = ctx.input.prompt.as_deref().unwrap_or("");
    let combined = format!("{tool} {prompt}").to_lowercase();

    let high_risk = [
        "bash", "write", "delete", "rm ", "drop", "force", "deploy",
        "push", "credential", "secret",
    ];
    if high_risk.iter().any(|kw| combined.contains(kw)) {
        return RiskLevel::High;
    }

    let medium_risk = [
        "edit", "replace", "modify", "update", "install", "create",
    ];
    if medium_risk.iter().any(|kw| combined.contains(kw)) {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_prompt_low_difficulty() {
        let d = estimate_difficulty("hello world");
        assert!(d < 0.2, "got {d}");
    }

    #[test]
    fn complexity_keywords_increase_difficulty() {
        let d = estimate_difficulty("implement a distributed concurrent system with async parallel processing");
        assert!(d > 0.3, "got {d}");
    }
}

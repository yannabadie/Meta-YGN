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
        "complex",
        "concurren",
        "async",
        "parallel",
        "distributed",
        "performance",
        "optimize",
        "scale",
        "migration",
        "backward",
        "compatibility",
        "recursive",
        "cryptograph",
    ];
    let bonus = complexity_keywords
        .iter()
        .filter(|kw| text.to_lowercase().contains(*kw))
        .count() as f32
        * 0.1;

    (base + bonus).min(1.0)
}

/// Estimate risk level from tool name, prompt, and command content.
fn estimate_risk(ctx: &LoopContext) -> RiskLevel {
    let tool = ctx.input.tool_name.as_deref().unwrap_or("");
    let tool_lower = tool.to_lowercase();
    let prompt = ctx.input.prompt.as_deref().unwrap_or("");
    let combined = format!("{tool} {prompt}").to_lowercase();

    // High-risk command patterns (dangerous operations and sensitive domains)
    let high_risk = [
        "delete",
        "rm ",
        "rm\t",
        "drop",
        "force",
        "deploy",
        "push",
        "credential",
        "secret",
        "vulnerability",
        "exploit",
    ];
    if high_risk.iter().any(|kw| combined.contains(kw)) {
        return RiskLevel::High;
    }

    let is_bash = tool_lower == "bash";

    // If the tool is Bash and we have a command (via prompt), check the command
    // for safe patterns — read-only commands are low risk.
    if is_bash && !prompt.is_empty() {
        let safe_prefixes = [
            "ls",
            "cat",
            "head",
            "tail",
            "wc",
            "echo",
            "pwd",
            "date",
            "whoami",
            "uname",
            "find",
            "grep",
            "rg",
            "which",
            "env",
            "printenv",
            "cargo test",
            "cargo check",
            "cargo build",
            "cargo clippy",
            "cargo fmt",
            "npm test",
            "npm run",
            "git status",
            "git log",
            "git diff",
            "git branch",
        ];
        let cmd_lower = prompt.to_lowercase();
        let cmd_trimmed = cmd_lower.trim();
        if safe_prefixes.iter().any(|p| cmd_trimmed.starts_with(p)) {
            return RiskLevel::Low;
        }
        // Unknown bash commands default to medium (guard pipeline handles truly dangerous ones)
        return RiskLevel::Medium;
    }

    // Bash tool with no command prompt available — default to high risk
    // (we don't know what command will run)
    if is_bash {
        return RiskLevel::High;
    }

    // Tool-level risk: Write tool is medium
    let is_write = tool_lower == "write";

    // Medium-risk keywords in prompt/tool context
    let medium_risk_keywords = ["edit", "replace", "modify", "update", "install", "create"];
    if is_write || medium_risk_keywords.iter().any(|kw| combined.contains(kw)) {
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
        let d = estimate_difficulty(
            "implement a distributed concurrent system with async parallel processing",
        );
        assert!(d > 0.3, "got {d}");
    }
}

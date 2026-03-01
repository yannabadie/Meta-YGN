use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::TaskType;

/// Stage 1: Classify the task type from prompt keywords and tool context.
pub struct ClassifyStage;

impl Stage for ClassifyStage {
    fn name(&self) -> &'static str {
        "classify"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        let text = combined_text(ctx);
        let lower = text.to_lowercase();

        ctx.task_type = Some(classify_from_keywords(&lower));

        tracing::debug!(
            stage = self.name(),
            task_type = ?ctx.task_type,
            "classified task"
        );

        StageResult::Continue
    }
}

/// Build a single searchable string from the prompt, tool name, and tool input.
fn combined_text(ctx: &LoopContext) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = &ctx.input.prompt {
        parts.push(String::clone(p));
    }
    if let Some(t) = &ctx.input.tool_name {
        parts.push(String::clone(t));
    }
    if let Some(ti) = &ctx.input.tool_input {
        parts.push(ti.to_string());
    }
    if let Some(msg) = &ctx.input.last_assistant_message {
        parts.push(String::clone(msg));
    }
    parts.join(" ")
}

/// Simple keyword-based classifier. Returns the best-matching `TaskType`.
fn classify_from_keywords(text: &str) -> TaskType {
    // Security keywords take highest priority.
    let security_keywords = [
        "security",
        "vulnerability",
        "cve",
        "auth",
        "permission",
        "secret",
        "credential",
    ];
    if security_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Security;
    }

    // Bug-fix keywords.
    let bugfix_keywords = [
        "fix",
        "bug",
        "error",
        "crash",
        "broken",
        "issue",
        "patch",
        "regression",
    ];
    if bugfix_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Bugfix;
    }

    // Refactor keywords.
    let refactor_keywords = [
        "refactor",
        "cleanup",
        "reorganize",
        "rename",
        "simplify",
        "extract",
        "deduplicate",
    ];
    if refactor_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Refactor;
    }

    // Architecture keywords.
    let arch_keywords = [
        "architecture",
        "design",
        "system",
        "infrastructure",
        "migration",
        "schema",
    ];
    if arch_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Architecture;
    }

    // Release keywords.
    let release_keywords = [
        "release",
        "deploy",
        "publish",
        "version",
        "tag",
        "changelog",
    ];
    if release_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Release;
    }

    // Research keywords.
    let research_keywords = [
        "research",
        "investigate",
        "explore",
        "prototype",
        "spike",
        "experiment",
    ];
    if research_keywords.iter().any(|kw| text.contains(kw)) {
        return TaskType::Research;
    }

    // Default: treat as a new feature.
    TaskType::Feature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_bugfix() {
        assert_eq!(
            classify_from_keywords("fix the login bug"),
            TaskType::Bugfix
        );
    }

    #[test]
    fn classify_security() {
        assert_eq!(
            classify_from_keywords("check for vulnerability"),
            TaskType::Security
        );
    }

    #[test]
    fn classify_refactor() {
        assert_eq!(
            classify_from_keywords("refactor the parser"),
            TaskType::Refactor
        );
    }

    #[test]
    fn classify_default_is_feature() {
        assert_eq!(
            classify_from_keywords("add a new button"),
            TaskType::Feature
        );
    }
}

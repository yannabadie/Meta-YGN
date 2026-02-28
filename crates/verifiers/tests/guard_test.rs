use metaygn_verifiers::guard_pipeline::{
    DefaultGuard, DestructiveGuard, Guard, GuardPipeline, HighRiskGuard, McpGuard, SecretPathGuard,
};

#[test]
fn destructive_guard_blocks_rm_rf() {
    let guard = DestructiveGuard;
    let result = guard.check("bash", "rm -rf /");
    assert!(!result.allowed);
    assert_eq!(result.score, 0);
    assert_eq!(result.guard_name, "destructive");
    assert!(result.reason.is_some());
}

#[test]
fn high_risk_guard_blocks_git_push() {
    let guard = HighRiskGuard;
    let result = guard.check("bash", "git push origin main");
    assert!(!result.allowed);
    assert_eq!(result.score, 30);
    assert_eq!(result.guard_name, "high_risk");
    assert!(result.reason.is_some());
}

#[test]
fn secret_guard_blocks_env_file() {
    let guard = SecretPathGuard;
    let result = guard.check("bash", "cat .env");
    assert!(!result.allowed);
    assert_eq!(result.score, 20);
    assert_eq!(result.guard_name, "secret_path");
    assert!(result.reason.is_some());
}

#[test]
fn mcp_guard_blocks_mcp_tools() {
    let guard = McpGuard;
    let result = guard.check("mcp__foo", "some input");
    assert!(!result.allowed);
    assert_eq!(result.score, 40);
    assert_eq!(result.guard_name, "mcp");
    assert!(result.reason.is_some());
}

#[test]
fn default_guard_allows_safe_commands() {
    let guard = DefaultGuard;
    let result = guard.check("bash", "ls -la");
    assert!(result.allowed);
    assert_eq!(result.score, 100);
    assert_eq!(result.guard_name, "default");
    assert!(result.reason.is_none());
}

#[test]
fn pipeline_blocks_on_first_match() {
    let pipeline = GuardPipeline::new();
    let decision = pipeline.check("bash", "rm -rf /");
    assert!(!decision.allowed);
    assert_eq!(decision.aggregate_score, 0);
    assert_eq!(decision.blocking_guard.as_deref(), Some("destructive"));
}

#[test]
fn pipeline_allows_safe_input() {
    let pipeline = GuardPipeline::new();
    let decision = pipeline.check("bash", "ls -la");
    assert!(decision.allowed);
    assert_eq!(decision.aggregate_score, 100);
    assert!(decision.blocking_guard.is_none());
}

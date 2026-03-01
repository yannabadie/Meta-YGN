use metaygn_core::context::LoopContext;
use metaygn_core::stages::compact::{CompactStage, cluster_lessons};
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};

fn make_context() -> LoopContext {
    LoopContext::new(HookInput {
        hook_event_name: HookEvent::PreCompact,
        session_id: None,
        cwd: None,
        tool_name: None,
        tool_input: None,
        tool_response: None,
        prompt: None,
        error: None,
        last_assistant_message: None,
        source: None,
        reason: None,
        trigger: None,
    })
}

#[test]
fn cluster_merges_similar_lessons() {
    // Three lessons with heavy word overlap (>50% Jaccard)
    let lessons = vec![
        "always check error handling before deploying".to_string(),
        "always check error handling before releasing".to_string(),
        "always check error handling before shipping".to_string(),
    ];
    let result = cluster_lessons(&lessons, 10);
    assert_eq!(
        result.len(),
        1,
        "similar lessons should merge into one cluster"
    );
    assert!(
        result[0].contains("(x3)"),
        "merged cluster should have count suffix (x3), got: {}",
        result[0]
    );
}

#[test]
fn cluster_keeps_different_lessons_separate() {
    // Three completely unrelated lessons
    let lessons = vec![
        "always check error handling before deploying".to_string(),
        "database migrations require backup first".to_string(),
        "unit tests should cover edge cases thoroughly".to_string(),
    ];
    let result = cluster_lessons(&lessons, 10);
    assert_eq!(
        result.len(),
        3,
        "unrelated lessons should stay separate, got: {:?}",
        result
    );
    // None should have a count suffix
    for r in &result {
        assert!(
            !r.contains("(x"),
            "separate lessons should not have count suffix, got: {}",
            r
        );
    }
}

#[test]
fn cluster_respects_max_limit() {
    // 15 genuinely unrelated lessons (no shared non-trivial words)
    let lessons: Vec<String> = vec![
        "always check error handling carefully".into(),
        "database migrations require careful backup".into(),
        "unit tests should cover edge cases".into(),
        "security reviews prevent vulnerabilities early".into(),
        "performance profiling identifies bottleneck issues".into(),
        "documentation helps onboarding new developers".into(),
        "code reviews catch bugs before production".into(),
        "monitoring alerts detect incidents quickly overnight".into(),
        "dependency updates prevent supply chain attacks".into(),
        "configuration management reduces deployment failures".into(),
        "incident response plans minimize customer impact".into(),
        "accessibility standards improve user experience universally".into(),
        "caching strategies reduce latency significantly overall".into(),
        "load balancing distributes traffic across servers".into(),
        "encryption protects sensitive data during transit".into(),
    ];
    let result = cluster_lessons(&lessons, 10);
    assert!(
        result.len() <= 10,
        "should respect max_clusters=10, got {} clusters",
        result.len()
    );
    assert!(
        result.len() >= 10,
        "15 unrelated lessons with max=10 should produce exactly 10, got {}",
        result.len()
    );
}

#[test]
fn compact_stage_produces_summary() {
    let stage = CompactStage;
    let mut ctx = make_context();
    ctx.lessons = vec!["lesson alpha".to_string(), "lesson beta".to_string()];
    ctx.verification_results = vec!["check passed".to_string()];

    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue);

    // The last lesson should be the summary line
    let summary = ctx.lessons.last().expect("should have at least one lesson");
    assert!(
        summary.starts_with("[compact]"),
        "summary should start with [compact], got: {}",
        summary
    );
    assert!(
        summary.contains("lessons"),
        "summary should mention lessons, got: {}",
        summary
    );
    assert!(
        summary.contains("verifications"),
        "summary should mention verifications, got: {}",
        summary
    );
    assert!(
        summary.contains("quality="),
        "summary should mention quality, got: {}",
        summary
    );
}

#[test]
fn empty_lessons_stay_empty() {
    let result = cluster_lessons(&[], 10);
    assert!(
        result.is_empty(),
        "clustering empty lessons should return empty vec"
    );
}

use metaygn_daemon::session::{SessionContext, SessionStore};
use metaygn_shared::state::{RiskLevel, Strategy};

// -----------------------------------------------------------------------
// 1. get_or_create returns the same Arc for the same session ID
// -----------------------------------------------------------------------
#[test]
fn session_store_get_or_create_is_idempotent() {
    let store = SessionStore::new();

    let ctx1 = store.get_or_create("session-abc");
    let ctx2 = store.get_or_create("session-abc");

    // Both should point to the same underlying allocation.
    assert!(
        std::sync::Arc::ptr_eq(&ctx1, &ctx2),
        "expected same Arc for the same session ID"
    );
    assert_eq!(store.count(), 1);
}

// -----------------------------------------------------------------------
// 2. Different session IDs yield different Arcs
// -----------------------------------------------------------------------
#[test]
fn session_store_different_ids_different_contexts() {
    let store = SessionStore::new();

    let ctx1 = store.get_or_create("session-1");
    let ctx2 = store.get_or_create("session-2");

    assert!(
        !std::sync::Arc::ptr_eq(&ctx1, &ctx2),
        "expected different Arcs for different session IDs"
    );
    assert_eq!(store.count(), 2);
}

// -----------------------------------------------------------------------
// 3. remove returns the context and decreases count
// -----------------------------------------------------------------------
#[test]
fn session_store_remove_cleans_up() {
    let store = SessionStore::new();

    store.get_or_create("session-x");
    store.get_or_create("session-y");
    assert_eq!(store.count(), 2);

    let removed = store.remove("session-x");
    assert!(removed.is_some(), "expected Some when removing existing session");
    assert_eq!(store.count(), 1);

    // Removing again should return None.
    let removed_again = store.remove("session-x");
    assert!(removed_again.is_none(), "expected None when removing already-removed session");
    assert_eq!(store.count(), 1);
}

// -----------------------------------------------------------------------
// 4. New SessionContext has sensible defaults
// -----------------------------------------------------------------------
#[test]
fn session_context_defaults() {
    let ctx = SessionContext::new("test-session".to_string());

    assert_eq!(ctx.session_id, "test-session");
    assert!(ctx.task_type.is_none());
    assert_eq!(ctx.risk, RiskLevel::Low);
    assert_eq!(ctx.strategy, Strategy::StepByStep);
    assert!((ctx.difficulty - 0.5).abs() < f32::EPSILON);
    assert!((ctx.competence - 0.7).abs() < f32::EPSILON);
    assert!(ctx.verification_results.is_empty());
    assert!(ctx.lessons.is_empty());
    assert!(ctx.execution_plan.is_none());
    assert_eq!(ctx.tool_calls, 0);
    assert_eq!(ctx.errors, 0);
    assert_eq!(ctx.success_count, 0);
    assert!((ctx.metacog_vector.confidence - 0.5).abs() < f64::EPSILON);
    assert!((ctx.metacog_vector.progress - 0.0).abs() < f64::EPSILON);
    assert!(ctx.entropy_tracker.is_empty());
}

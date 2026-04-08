use metaygn_core::policy_engine::{evaluate, PolicyViolation};
use metaygn_core::sequence_monitor::{ActionState, ActionType, TargetType};

fn act(action_type: ActionType, target_type: TargetType) -> ActionState {
    ActionState {
        action_type,
        target_type,
        tool_name: "test".into(),
        detail: "test".into(),
    }
}

#[test]
fn empty_no_violations() {
    assert!(evaluate(&[]).is_empty());
}

#[test]
fn safe_reads_no_violations() {
    let a1 = act(ActionType::Read, TargetType::File);
    let a2 = act(ActionType::Read, TargetType::File);
    assert!(evaluate(&[(0, &a1), (1, &a2)]).is_empty());
}

#[test]
fn network_then_sensitive_write() {
    let a1 = act(ActionType::NetworkRead, TargetType::Url);
    let a2 = act(ActionType::Write, TargetType::SensitivePath);
    let v = evaluate(&[(0, &a1), (1, &a2)]);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "network_then_sensitive_write");
}

#[test]
fn delete_then_force_push() {
    let a1 = act(ActionType::Delete, TargetType::File);
    let a2 = act(ActionType::GitPush, TargetType::Remote);
    let v = evaluate(&[(0, &a1), (1, &a2)]);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "delete_then_force_push");
}

#[test]
fn error_then_test_modify() {
    let a1 = act(ActionType::Error, TargetType::File);
    let a2 = act(ActionType::Write, TargetType::TestFile);
    let v = evaluate(&[(0, &a1), (1, &a2)]);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "errors_then_test_modify");
}

#[test]
fn reversed_order_no_violation() {
    let a1 = act(ActionType::Write, TargetType::SensitivePath);
    let a2 = act(ActionType::NetworkRead, TargetType::Url);
    assert!(evaluate(&[(0, &a1), (1, &a2)]).is_empty());
}

#[test]
fn multiple_violations() {
    let a1 = act(ActionType::NetworkRead, TargetType::Url);
    let a2 = act(ActionType::Write, TargetType::SensitivePath);
    let a3 = act(ActionType::Delete, TargetType::File);
    let a4 = act(ActionType::GitPush, TargetType::Remote);
    let v = evaluate(&[(0, &a1), (1, &a2), (2, &a3), (3, &a4)]);
    assert_eq!(v.len(), 2);
}

#[test]
fn deduplicated_per_rule() {
    let a1 = act(ActionType::NetworkRead, TargetType::Url);
    let a2 = act(ActionType::NetworkRead, TargetType::Url);
    let a3 = act(ActionType::Write, TargetType::SensitivePath);
    let a4 = act(ActionType::Write, TargetType::SensitivePath);
    let v = evaluate(&[(0, &a1), (1, &a2), (2, &a3), (3, &a4)]);
    assert_eq!(v.len(), 1);
}

//! Integration tests for [`metaygn_core::sequence_monitor`].

use metaygn_core::sequence_monitor::{
    ActionState, ActionType, SequenceMonitor, TargetType,
};

fn action(action_type: ActionType, target_type: TargetType) -> ActionState {
    ActionState {
        action_type,
        target_type,
        tool_name: String::new(),
        detail: String::new(),
    }
}

#[test]
fn empty_monitor_has_no_alerts() {
    let mon = SequenceMonitor::new();
    assert!(mon.alerts().is_empty());
    assert_eq!(mon.action_count(), 0);
}

#[test]
fn record_action_increments_count() {
    let mut mon = SequenceMonitor::new();
    mon.record(action(ActionType::Read, TargetType::File));
    assert_eq!(mon.action_count(), 1);
}

#[test]
fn network_read_then_sensitive_write_triggers_alert() {
    let mut mon = SequenceMonitor::new();
    mon.record(action(ActionType::NetworkRead, TargetType::Url));
    mon.record(action(ActionType::Write, TargetType::SensitivePath));
    assert_eq!(mon.alerts().len(), 1);
    assert!(mon.alerts()[0].rule.contains("network_then_sensitive_write"));
}

#[test]
fn delete_then_force_push_triggers_alert() {
    let mut mon = SequenceMonitor::new();
    mon.record(action(ActionType::Delete, TargetType::File));
    mon.record(action(ActionType::GitPush, TargetType::Remote));
    assert_eq!(mon.alerts().len(), 1);
    assert!(mon.alerts()[0].rule.contains("delete_then_force_push"));
}

#[test]
fn repeated_errors_then_test_modify_triggers_alert() {
    let mut mon = SequenceMonitor::new();
    for _ in 0..3 {
        mon.record(action(ActionType::Error, TargetType::Unknown));
    }
    mon.record(action(ActionType::Write, TargetType::TestFile));
    assert_eq!(mon.alerts().len(), 1);
    assert!(mon.alerts()[0].rule.contains("errors_then_test_modify"));
}

#[test]
fn safe_read_sequence_no_alerts() {
    let mut mon = SequenceMonitor::new();
    for _ in 0..10 {
        mon.record(action(ActionType::Read, TargetType::File));
    }
    assert!(mon.alerts().is_empty());
}

#[test]
fn write_then_test_is_normal() {
    let mut mon = SequenceMonitor::new();
    mon.record(action(ActionType::Write, TargetType::File));
    mon.record(action(ActionType::Execute, TargetType::File));
    assert!(mon.alerts().is_empty());
}

#[test]
fn alert_fires_once_not_repeatedly() {
    let mut mon = SequenceMonitor::new();
    mon.record(action(ActionType::NetworkRead, TargetType::Url));
    mon.record(action(ActionType::Write, TargetType::SensitivePath));
    assert_eq!(mon.alerts().len(), 1);
    for _ in 0..5 {
        mon.record(action(ActionType::Read, TargetType::File));
    }
    mon.record(action(ActionType::NetworkRead, TargetType::Url));
    mon.record(action(ActionType::Write, TargetType::SensitivePath));
    assert_eq!(mon.alerts().len(), 1);
}

#[test]
fn old_actions_beyond_window_dont_trigger() {
    let mut mon = SequenceMonitor::with_window(5);
    mon.record(action(ActionType::NetworkRead, TargetType::Url));
    for _ in 0..10 {
        mon.record(action(ActionType::Read, TargetType::File));
    }
    mon.record(action(ActionType::Write, TargetType::SensitivePath));
    assert!(mon.alerts().is_empty());
}

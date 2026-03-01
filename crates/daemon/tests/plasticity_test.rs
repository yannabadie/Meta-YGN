use metaygn_daemon::profiler::plasticity::{PlasticityTracker, RecoveryOutcome};

// -----------------------------------------------------------------------
// 1. Fresh tracker has no data — total = 0, score = 1.0 (optimistic)
// -----------------------------------------------------------------------
#[test]
fn fresh_tracker_has_no_data() {
    let tracker = PlasticityTracker::new();

    assert_eq!(tracker.total_recoveries(), 0);
    assert!(
        (tracker.plasticity_score() - 1.0).abs() < f64::EPSILON,
        "expected optimistic default score 1.0, got {}",
        tracker.plasticity_score(),
    );
    assert!(!tracker.is_low_plasticity());
}

// -----------------------------------------------------------------------
// 2. Recording a success after injection keeps score high
// -----------------------------------------------------------------------
#[test]
fn record_success_improves_score() {
    let mut tracker = PlasticityTracker::new();

    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Success);

    assert_eq!(tracker.total_recoveries(), 1);
    assert!(
        tracker.plasticity_score() > 0.5,
        "expected score > 0.5 after one success, got {}",
        tracker.plasticity_score(),
    );
}

// -----------------------------------------------------------------------
// 3. Recording a failure lowers the score below 1.0
// -----------------------------------------------------------------------
#[test]
fn record_failure_lowers_score() {
    let mut tracker = PlasticityTracker::new();

    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Failure);

    assert!(
        tracker.plasticity_score() < 1.0,
        "expected score < 1.0 after a failure, got {}",
        tracker.plasticity_score(),
    );
}

// -----------------------------------------------------------------------
// 4. Multiple failures trigger low plasticity (score < 0.3)
// -----------------------------------------------------------------------
#[test]
fn multiple_failures_trigger_low_plasticity() {
    let mut tracker = PlasticityTracker::new();

    for _ in 0..5 {
        tracker.record_recovery_injected();
        tracker.record_outcome(RecoveryOutcome::Failure);
    }

    assert!(
        tracker.plasticity_score() < 0.3,
        "expected score < 0.3 after 5 failures, got {}",
        tracker.plasticity_score(),
    );
    assert!(
        tracker.is_low_plasticity(),
        "expected is_low_plasticity() = true when score = {}",
        tracker.plasticity_score(),
    );
}

// -----------------------------------------------------------------------
// 5. Amplification level increases with consecutive failures
// -----------------------------------------------------------------------
#[test]
fn amplification_level_increases() {
    let mut tracker = PlasticityTracker::new();

    // No failures yet: level 1 (standard)
    assert_eq!(
        tracker.amplification_level(),
        1,
        "expected level 1 with 0 failures"
    );

    // One failure: level 2 (emphatic)
    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(
        tracker.amplification_level(),
        2,
        "expected level 2 after 1 failure"
    );

    // Two failures: level 3 (escalate)
    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(
        tracker.amplification_level(),
        3,
        "expected level 3 after 2 failures"
    );

    // Three failures: still level 3 (capped)
    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(
        tracker.amplification_level(),
        3,
        "expected level 3 capped at max"
    );
}

// -----------------------------------------------------------------------
// 6. Success resets amplification back to level 1
// -----------------------------------------------------------------------
#[test]
fn success_resets_amplification() {
    let mut tracker = PlasticityTracker::new();

    // One failure → level 2
    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(tracker.amplification_level(), 2);

    // Success resets consecutive failures → level 1
    tracker.record_recovery_injected();
    tracker.record_outcome(RecoveryOutcome::Success);
    assert_eq!(
        tracker.amplification_level(),
        1,
        "expected level 1 after success reset, got {}",
        tracker.amplification_level(),
    );
}

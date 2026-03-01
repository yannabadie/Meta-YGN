use metaygn_core::heuristics::entropy::EntropyTracker;

#[test]
fn new_tracker_has_zero_overconfidence() {
    let tracker = EntropyTracker::new(10);
    assert!(tracker.is_empty());
    assert_eq!(tracker.len(), 0);
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

#[test]
fn correct_high_confidence_is_not_overconfident() {
    let mut tracker = EntropyTracker::new(10);
    // All high-confidence decisions are correct
    for _ in 0..5 {
        tracker.record(0.9, true);
    }
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

#[test]
fn wrong_high_confidence_is_overconfident() {
    let mut tracker = EntropyTracker::new(10);
    // All high-confidence decisions are wrong
    for _ in 0..5 {
        tracker.record(0.85, false);
    }
    // overconfidence_score = 5/5 = 1.0, well above 0.3 threshold
    assert_eq!(tracker.overconfidence_score(), 1.0);
    assert!(tracker.is_overconfident());
}

#[test]
fn sliding_window_evicts_old_entries() {
    let mut tracker = EntropyTracker::new(5);

    // Fill with 5 wrong high-confidence decisions
    for _ in 0..5 {
        tracker.record(0.9, false);
    }
    assert_eq!(tracker.len(), 5);
    assert!(tracker.is_overconfident());

    // Now push 5 correct high-confidence decisions, evicting the old ones
    for _ in 0..5 {
        tracker.record(0.9, true);
    }
    assert_eq!(tracker.len(), 5);
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

#[test]
fn low_confidence_errors_dont_count_as_overconfidence() {
    let mut tracker = EntropyTracker::new(10);
    // Many wrong decisions, but all at low confidence (below 0.7 threshold)
    for _ in 0..8 {
        tracker.record(0.3, false);
    }
    // Low-confidence entries are filtered out; score should be 0.0
    assert_eq!(tracker.overconfidence_score(), 0.0);
    assert!(!tracker.is_overconfident());
}

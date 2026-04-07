use metaygn_core::heuristics::mop::MopDetector;

#[test]
fn empty_detector_no_meltdown() {
    let det = MopDetector::new();
    assert!(!det.is_melting_down());
}

#[test]
fn record_increments_count() {
    let mut det = MopDetector::new();
    let r1 = det.record("read_file");
    assert!(!r1.meltdown_detected);
    let r2 = det.record("write_file");
    assert!(!r2.meltdown_detected);
    // Two calls recorded — no meltdown yet (window not full).
}

#[test]
fn uniform_distribution_below_threshold() {
    // 5 distinct tools in a window of 5 → max entropy = log2(5) ≈ 2.322
    // But default theta_h = 1.711, so this actually exceeds threshold.
    // However, we need entropy_delta > 0 as well, plus a full window.
    // With all-distinct tools the entropy grows steadily; let's check
    // the report values are reasonable.
    let mut det = MopDetector::with_params(5, 2.5, 0.0);
    for tool in &["a", "b", "c", "d", "e"] {
        det.record(tool);
    }
    // log2(5) ≈ 2.322 which is below 2.5 threshold
    assert!(!det.is_melting_down());
}

#[test]
fn repetitive_single_tool_low_entropy() {
    let mut det = MopDetector::new();
    for _ in 0..10 {
        det.record("bash");
    }
    // Single tool → entropy = 0 → never exceeds theta_h = 1.711
    assert!(!det.is_melting_down());
}

#[test]
fn mixed_pattern_crosses_threshold() {
    // Use a lower threshold so we can trigger meltdown with moderate entropy.
    let mut det = MopDetector::with_params(5, 1.0, 0.0);
    // First fill the window with a single tool (entropy = 0).
    for _ in 0..5 {
        det.record("bash");
    }
    assert!(!det.is_melting_down());
    // Now inject diverse tools to spike entropy above 1.0.
    det.record("read");
    det.record("write");
    det.record("search");
    det.record("grep");
    // Window is now [bash, read, write, search, grep] → H ≈ 2.322
    let report = det.record("lint");
    // Window: [read, write, search, grep, lint] → H ≈ 2.322
    assert!(report.entropy > 1.0);
    assert!(det.is_melting_down());
}

#[test]
fn meltdown_fires_once_not_repeatedly() {
    let mut det = MopDetector::with_params(5, 1.0, 0.0);
    for _ in 0..5 {
        det.record("bash");
    }
    // Spike diversity.
    for tool in &["a", "b", "c", "d", "e"] {
        det.record(tool);
    }
    assert!(det.is_melting_down());
    let step = det.record("f").meltdown_step;
    // The meltdown_step should not change after the first trigger.
    let step2 = det.record("g").meltdown_step;
    assert_eq!(step, step2);
}

#[test]
fn repetition_ratio_all_same_tool() {
    let mut det = MopDetector::new();
    for _ in 0..5 {
        let r = det.record("bash");
        // After the window fills, the dominant tool should be "bash"
        // and the ratio should be 1.0.
        if r.dominant_tool.is_some() {
            assert_eq!(r.dominant_tool.as_deref(), Some("bash"));
        }
    }
    let report = det.record("bash");
    assert!((report.repetition_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn reset_clears_state() {
    let mut det = MopDetector::with_params(5, 1.0, 0.0);
    for _ in 0..5 {
        det.record("bash");
    }
    for tool in &["a", "b", "c", "d", "e"] {
        det.record(tool);
    }
    assert!(det.is_melting_down());
    det.reset();
    assert!(!det.is_melting_down());
    let report = det.record("x");
    assert_eq!(report.meltdown_step, None);
}

#[test]
fn custom_params_work() {
    let det = MopDetector::with_params(10, 2.0, 0.5);
    assert!(!det.is_melting_down());
    // Just verifying construction with custom params doesn't panic.
}

#[test]
fn is_melting_down_reflects_state() {
    let mut det = MopDetector::with_params(3, 0.5, 0.0);
    assert!(!det.is_melting_down());
    // Fill window with one tool → entropy 0, no meltdown.
    for _ in 0..3 {
        det.record("bash");
    }
    assert!(!det.is_melting_down());
    // Now diversify → entropy should exceed 0.5.
    det.record("read");
    det.record("write");
    det.record("search");
    // Window: [read, write, search] → H = log2(3) ≈ 1.585 > 0.5
    assert!(det.is_melting_down());
}

#[test]
fn entropy_delta_tracked_correctly() {
    let mut det = MopDetector::new();
    let r1 = det.record("bash");
    // First call: entropy 0, prev was 0 → delta = 0.
    assert!((r1.entropy_delta).abs() < f64::EPSILON);
    let r2 = det.record("read");
    // Second call: entropy > 0 → delta should be positive.
    assert!(r2.entropy_delta > 0.0);
}

#[test]
fn default_trait_works() {
    let det = MopDetector::default();
    assert!(!det.is_melting_down());
}

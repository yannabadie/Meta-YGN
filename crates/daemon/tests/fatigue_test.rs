use chrono::{TimeZone, Utc};
use metaygn_daemon::profiler::fatigue::FatigueProfiler;

/// Helper: create a `DateTime<Utc>` at the given hour (same day).
fn utc_at_hour(hour: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 2, 28, hour, 0, 0).unwrap()
}

// -----------------------------------------------------------------------
// 1. Fresh profiler → score ≈ 0
// -----------------------------------------------------------------------
#[test]
fn no_signals_no_fatigue() {
    let profiler = FatigueProfiler::with_defaults();
    let report = profiler.assess();

    assert!(
        report.score.abs() < f64::EPSILON,
        "expected score ≈ 0, got {}",
        report.score,
    );
    assert!(!report.high_friction);
    assert_eq!(report.recommendation, "No fatigue signals detected");
}

// -----------------------------------------------------------------------
// 2. Five short prompts → score > 0
// -----------------------------------------------------------------------
#[test]
fn short_prompts_increase_fatigue() {
    let mut profiler = FatigueProfiler::with_defaults();
    let ts = utc_at_hour(14); // daytime, no late-night signal

    for _ in 0..5 {
        // "fix" is 3 chars — well below the 20-char threshold
        profiler.on_prompt("fix", ts);
    }

    let report = profiler.assess();
    assert!(
        report.score > 0.0,
        "expected score > 0 after 5 short prompts, got {}",
        report.score,
    );
}

// -----------------------------------------------------------------------
// 3. Three consecutive errors → score > 0.2
// -----------------------------------------------------------------------
#[test]
fn error_loop_increases_fatigue() {
    let mut profiler = FatigueProfiler::with_defaults();

    profiler.on_error();
    profiler.on_error();
    profiler.on_error(); // this one pushes an ErrorLoop signal

    let report = profiler.assess();
    assert!(
        report.score > 0.2,
        "expected score > 0.2 after 3 errors, got {}",
        report.score,
    );
}

// -----------------------------------------------------------------------
// 4. Prompt at 2 AM → score > 0
// -----------------------------------------------------------------------
#[test]
fn late_night_increases_fatigue() {
    let mut profiler = FatigueProfiler::with_defaults();
    let ts = utc_at_hour(2); // 2 AM

    // Send a long-enough prompt so only the LateNight signal fires
    profiler.on_prompt("please review this long prompt carefully", ts);

    let report = profiler.assess();
    assert!(
        report.score > 0.0,
        "expected score > 0 for late-night prompt, got {}",
        report.score,
    );
    assert!(
        report.signals.iter().any(|s| s.contains("late-night")),
        "expected a late-night signal description, got {:?}",
        report.signals,
    );
}

// -----------------------------------------------------------------------
// 5. Many signals → high_friction = true
// -----------------------------------------------------------------------
#[test]
fn high_friction_activates_above_threshold() {
    let mut profiler = FatigueProfiler::with_defaults();
    let ts = utc_at_hour(3); // 3 AM — every prompt also adds LateNight

    // Flood with short prompts at 3 AM (each fires ShortPrompt + LateNight,
    // and after the first also RapidRetry).
    for _ in 0..10 {
        profiler.on_prompt("fix", ts);
    }

    // Also push consecutive errors
    for _ in 0..5 {
        profiler.on_error();
    }

    let report = profiler.assess();
    assert!(
        report.high_friction,
        "expected high_friction = true, got score {}",
        report.score,
    );
    assert!(
        report.score >= 0.7,
        "expected score >= 0.7, got {}",
        report.score,
    );
    assert!(
        report.recommendation.contains("High-Friction"),
        "expected High-Friction recommendation, got {:?}",
        report.recommendation,
    );
}

// -----------------------------------------------------------------------
// 6. Success resets the error counter
// -----------------------------------------------------------------------
#[test]
fn success_resets_error_counter() {
    let mut profiler = FatigueProfiler::with_defaults();

    profiler.on_error();
    profiler.on_error();
    // Two errors, no ErrorLoop signal yet (threshold is 3).
    profiler.on_success();
    // consecutive_errors is now 0, so one more error should NOT trigger ErrorLoop.
    profiler.on_error();

    let report = profiler.assess();
    // No ErrorLoop signal was pushed (we never reached 3 consecutive).
    assert!(
        !report.signals.iter().any(|s| s.contains("error-loop")),
        "expected no error-loop signal after reset, got {:?}",
        report.signals,
    );
}

use std::collections::VecDeque;
use std::time::Instant;

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Signal weights
// ---------------------------------------------------------------------------
const WEIGHT_SHORT_PROMPT: f64 = 0.15;
const WEIGHT_ERROR_LOOP: f64 = 0.30;
const WEIGHT_LATE_NIGHT: f64 = 0.20;
const WEIGHT_RAPID_RETRY: f64 = 0.15;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Behavioral signal from a hook event.
#[derive(Debug, Clone)]
pub enum FatigueSignal {
    /// Very short, possibly aggressive prompts ("just fix it").
    ShortPrompt { length: usize },
    /// Repeated consecutive errors — stuck in a loop.
    ErrorLoop { consecutive_errors: usize },
    /// Working late at night (23:00–05:00).
    LateNight { hour: u32 },
    /// Retrying too quickly (< 5 seconds between attempts).
    RapidRetry { interval_ms: u64 },
}

/// Fatigue assessment result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatigueReport {
    /// 0.0 = fully alert, 1.0 = exhausted.
    pub score: f64,
    /// Should we activate High-Friction mode?
    pub high_friction: bool,
    /// Human-readable signal descriptions.
    pub signals: Vec<String>,
    /// What to do.
    pub recommendation: String,
}

/// Configuration knobs.
pub struct FatigueConfig {
    /// Score at or above which High-Friction mode is activated (default: 0.7).
    pub high_friction_threshold: f64,
    /// How many recent signals to consider (default: 20).
    pub signal_window: usize,
    /// Characters below which a prompt is considered "short" (default: 20).
    pub short_prompt_threshold: usize,
    /// Milliseconds below which a retry is considered "rapid" (default: 5000).
    pub rapid_retry_ms: u64,
}

impl Default for FatigueConfig {
    fn default() -> Self {
        Self {
            high_friction_threshold: 0.7,
            signal_window: 20,
            short_prompt_threshold: 20,
            rapid_retry_ms: 5000,
        }
    }
}

// ---------------------------------------------------------------------------
// Profiler
// ---------------------------------------------------------------------------

/// Tracks behavioral signals and computes a fatigue score.
///
/// This is "inverse metacognition" — the system monitors the *human*
/// developer's behaviour to protect the codebase when the human is exhausted.
pub struct FatigueProfiler {
    config: FatigueConfig,
    signals: VecDeque<(Instant, FatigueSignal)>,
    last_prompt_time: Option<Instant>,
    consecutive_errors: usize,
}

impl FatigueProfiler {
    /// Create a new profiler with the given configuration.
    pub fn new(config: FatigueConfig) -> Self {
        Self {
            config,
            signals: VecDeque::new(),
            last_prompt_time: None,
            consecutive_errors: 0,
        }
    }

    /// Create a new profiler with default settings.
    pub fn with_defaults() -> Self {
        Self::new(FatigueConfig::default())
    }

    // -- event handlers -----------------------------------------------------

    /// Record a user prompt submission.
    pub fn on_prompt(&mut self, prompt: &str, timestamp: DateTime<Utc>) {
        let now = Instant::now();

        // 1. Short prompt?
        if prompt.len() < self.config.short_prompt_threshold {
            self.push_signal(
                now,
                FatigueSignal::ShortPrompt {
                    length: prompt.len(),
                },
            );
        }

        // 2. Rapid retry?
        if let Some(last) = self.last_prompt_time {
            let elapsed_ms = now.duration_since(last).as_millis() as u64;
            if elapsed_ms < self.config.rapid_retry_ms {
                self.push_signal(
                    now,
                    FatigueSignal::RapidRetry {
                        interval_ms: elapsed_ms,
                    },
                );
            }
        }

        // 3. Late night?
        let hour = timestamp.hour();
        if !(5..23).contains(&hour) {
            self.push_signal(now, FatigueSignal::LateNight { hour });
        }

        // 4. Update last prompt time
        self.last_prompt_time = Some(now);
    }

    /// Record a tool-use failure.
    pub fn on_error(&mut self) {
        self.consecutive_errors += 1;
        if self.consecutive_errors >= 3 {
            self.push_signal(
                Instant::now(),
                FatigueSignal::ErrorLoop {
                    consecutive_errors: self.consecutive_errors,
                },
            );
        }
    }

    /// Record a tool-use success (resets the error counter).
    pub fn on_success(&mut self) {
        self.consecutive_errors = 0;
    }

    // -- assessment ---------------------------------------------------------

    /// Compute the current fatigue score from the signal window.
    pub fn assess(&self) -> FatigueReport {
        let mut short_prompt_count: usize = 0;
        let mut error_loop_count: usize = 0;
        let mut late_night_count: usize = 0;
        let mut rapid_retry_count: usize = 0;

        for (_ts, signal) in &self.signals {
            match signal {
                FatigueSignal::ShortPrompt { .. } => short_prompt_count += 1,
                FatigueSignal::ErrorLoop { .. } => error_loop_count += 1,
                FatigueSignal::LateNight { .. } => late_night_count += 1,
                FatigueSignal::RapidRetry { .. } => rapid_retry_count += 1,
            }
        }

        // Each signal occurrence contributes its full weight.  The raw
        // sum is capped at 1.0.  This means even a small number of heavy
        // signals (e.g. ErrorLoop at 0.30) can meaningfully move the score.
        let raw = short_prompt_count as f64 * WEIGHT_SHORT_PROMPT
            + error_loop_count as f64 * WEIGHT_ERROR_LOOP
            + late_night_count as f64 * WEIGHT_LATE_NIGHT
            + rapid_retry_count as f64 * WEIGHT_RAPID_RETRY;

        let score = raw.min(1.0);

        // Human-readable signals
        let mut descriptions = Vec::new();
        if short_prompt_count > 0 {
            descriptions.push(format!("{short_prompt_count} short prompt(s) detected"));
        }
        if error_loop_count > 0 {
            descriptions.push(format!("{error_loop_count} error-loop signal(s)"));
        }
        if late_night_count > 0 {
            descriptions.push(format!("{late_night_count} late-night signal(s)"));
        }
        if rapid_retry_count > 0 {
            descriptions.push(format!("{rapid_retry_count} rapid-retry signal(s)"));
        }

        // Recommendation
        let (high_friction, recommendation) = if score >= self.config.high_friction_threshold {
            (
                true,
                "High-Friction mode: refuse major refactors, require tests before destructive actions".to_string(),
            )
        } else if score > 0.4 {
            (
                false,
                "Moderate fatigue detected: prefer smaller, safer changes".to_string(),
            )
        } else {
            (false, "No fatigue signals detected".to_string())
        };

        FatigueReport {
            score,
            high_friction,
            signals: descriptions,
            recommendation,
        }
    }

    // -- housekeeping -------------------------------------------------------

    /// Reset all profiler state.
    pub fn reset(&mut self) {
        self.signals.clear();
        self.last_prompt_time = None;
        self.consecutive_errors = 0;
    }

    // -- internals ----------------------------------------------------------

    fn push_signal(&mut self, ts: Instant, signal: FatigueSignal) {
        if self.signals.len() >= self.config.signal_window {
            self.signals.pop_front();
        }
        self.signals.push_back((ts, signal));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = FatigueConfig::default();
        assert!((cfg.high_friction_threshold - 0.7).abs() < f64::EPSILON);
        assert_eq!(cfg.signal_window, 20);
        assert_eq!(cfg.short_prompt_threshold, 20);
        assert_eq!(cfg.rapid_retry_ms, 5000);
    }

    #[test]
    fn fresh_profiler_score_is_zero() {
        let p = FatigueProfiler::with_defaults();
        let report = p.assess();
        assert!((report.score - 0.0).abs() < f64::EPSILON);
        assert!(!report.high_friction);
    }

    #[test]
    fn on_success_resets_consecutive_errors() {
        let mut p = FatigueProfiler::with_defaults();
        p.on_error();
        p.on_error();
        assert_eq!(p.consecutive_errors, 2);
        p.on_success();
        assert_eq!(p.consecutive_errors, 0);
    }
}

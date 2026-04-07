//! MOP Detector — Meltdown Onset Point detection via Shannon entropy.
//!
//! Based on "Beyond pass@1" (arXiv 2603.29231). Detects when an AI agent
//! enters behavioral collapse by monitoring the Shannon entropy of tool-call
//! distributions over a sliding window. A sudden entropy spike past a
//! calibrated threshold signals that the agent is thrashing rather than
//! making purposeful progress.

use std::collections::{HashMap, VecDeque};

/// Default sliding window size (number of recent tool calls to consider).
const DEFAULT_WINDOW: usize = 5;

/// Default entropy threshold in bits. Derived from the paper's recommended
/// calibration for typical 5-tool windows.
const DEFAULT_THETA_H: f64 = 1.711;

/// Default entropy-delta threshold. A spike is flagged when the entropy
/// increase between consecutive observations exceeds this value.
const DEFAULT_DELTA: f64 = 0.0;

/// Report produced by each call to [`MopDetector::record`].
#[derive(Debug, Clone)]
pub struct MopReport {
    /// Current Shannon entropy of the tool distribution in the window.
    pub entropy: f64,
    /// Change in entropy since the previous observation.
    pub entropy_delta: f64,
    /// Whether a meltdown has been detected (latched).
    pub meltdown_detected: bool,
    /// The step number at which meltdown was first detected, if any.
    pub meltdown_step: Option<usize>,
    /// Fraction of calls in the repetition window that match the most
    /// frequent tool (higher = more repetitive).
    pub repetition_ratio: f64,
    /// The most frequently called tool in the current window, if any.
    pub dominant_tool: Option<String>,
}

/// Detects behavioral collapse (meltdown) in AI agents by tracking the
/// Shannon entropy of tool-call distributions over a sliding window.
#[derive(Debug, Clone)]
pub struct MopDetector {
    window: VecDeque<String>,
    window_size: usize,
    theta_h: f64,
    delta: f64,
    prev_entropy: f64,
    total_calls: usize,
    meltdown_detected: bool,
    meltdown_step: Option<usize>,
    repetition_window: VecDeque<String>,
}

impl MopDetector {
    /// Create a new detector with default parameters.
    pub fn new() -> Self {
        Self::with_params(DEFAULT_WINDOW, DEFAULT_THETA_H, DEFAULT_DELTA)
    }

    /// Create a new detector with custom parameters.
    pub fn with_params(window_size: usize, theta_h: f64, delta: f64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            theta_h,
            delta,
            prev_entropy: 0.0,
            total_calls: 0,
            meltdown_detected: false,
            meltdown_step: None,
            repetition_window: VecDeque::with_capacity(window_size * 2),
        }
    }

    /// Record a tool call and check for meltdown.
    pub fn record(&mut self, tool_name: &str) -> MopReport {
        self.total_calls += 1;

        // Maintain the main sliding window.
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(tool_name.to_string());

        // Maintain the repetition window (2x main window).
        let rep_cap = self.window_size * 2;
        if self.repetition_window.len() >= rep_cap {
            self.repetition_window.pop_front();
        }
        self.repetition_window.push_back(tool_name.to_string());

        // Compute tool distribution over the main window.
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for t in &self.window {
            *counts.entry(t.as_str()).or_insert(0) += 1;
        }
        let n = self.window.len() as f64;

        // Shannon entropy: H = -SUM p_i * log2(p_i)
        let entropy = counts.values().fold(0.0_f64, |acc, &c| {
            let p = c as f64 / n;
            if p > 0.0 {
                acc - p * p.log2()
            } else {
                acc
            }
        });

        let entropy_delta = entropy - self.prev_entropy;

        // Detect meltdown: entropy exceeds threshold AND delta exceeds spike
        // threshold AND the window is fully populated. Only fires once.
        if !self.meltdown_detected
            && self.window.len() >= self.window_size
            && entropy > self.theta_h
            && entropy_delta > self.delta
        {
            self.meltdown_detected = true;
            self.meltdown_step = Some(self.total_calls);
        }

        self.prev_entropy = entropy;

        // Repetition ratio: fraction of repetition window occupied by the
        // dominant tool.
        let (repetition_ratio, dominant_tool) = self.compute_repetition();

        MopReport {
            entropy,
            entropy_delta,
            meltdown_detected: self.meltdown_detected,
            meltdown_step: self.meltdown_step,
            repetition_ratio,
            dominant_tool,
        }
    }

    /// Whether a meltdown has been detected (latched until reset).
    pub fn is_melting_down(&self) -> bool {
        self.meltdown_detected
    }

    /// Reset the detector to its initial state, preserving configuration.
    pub fn reset(&mut self) {
        self.window.clear();
        self.prev_entropy = 0.0;
        self.total_calls = 0;
        self.meltdown_detected = false;
        self.meltdown_step = None;
        self.repetition_window.clear();
    }

    /// Compute repetition ratio and dominant tool from the repetition window.
    fn compute_repetition(&self) -> (f64, Option<String>) {
        if self.repetition_window.is_empty() {
            return (0.0, None);
        }
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for t in &self.repetition_window {
            *counts.entry(t.as_str()).or_insert(0) += 1;
        }
        let (dominant, &max_count) = counts.iter().max_by_key(|(_, c)| *c).unwrap();
        let ratio = max_count as f64 / self.repetition_window.len() as f64;
        (ratio, Some(dominant.to_string()))
    }
}

impl Default for MopDetector {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::VecDeque;

/// Tracks overconfidence by monitoring high-confidence decisions that turn out wrong.
/// Inspired by EGPO (arXiv:2602.22751) entropy-based metacognitive calibration.
#[derive(Debug, Clone)]
pub struct EntropyTracker {
    window: VecDeque<(f64, bool)>,
    window_size: usize,
}

const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.7;
const OVERCONFIDENCE_THRESHOLD: f64 = 0.3;

impl EntropyTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn record(&mut self, confidence: f64, was_correct: bool) {
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back((confidence, was_correct));
    }

    pub fn overconfidence_score(&self) -> f64 {
        let high_conf: Vec<&(f64, bool)> = self
            .window
            .iter()
            .filter(|(c, _)| *c >= HIGH_CONFIDENCE_THRESHOLD)
            .collect();
        if high_conf.is_empty() {
            return 0.0;
        }
        let wrong_count = high_conf.iter().filter(|(_, correct)| !correct).count();
        wrong_count as f64 / high_conf.len() as f64
    }

    pub fn is_overconfident(&self) -> bool {
        self.overconfidence_score() > OVERCONFIDENCE_THRESHOLD
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

use serde::{Deserialize, Serialize};

/// Tracks token and cost consumption for a session, making budget usage
/// visible to the developer in every hook response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBudget {
    max_tokens: u64,
    max_cost_usd: f64,
    consumed_tokens: u64,
    consumed_cost_usd: f64,
    warning_threshold: f64, // 0.80
}

impl SessionBudget {
    /// Create a new budget with the given maximums.
    /// Starts with zero consumed, warning threshold at 80%.
    pub fn new(max_tokens: u64, max_cost_usd: f64) -> Self {
        Self {
            max_tokens,
            max_cost_usd,
            consumed_tokens: 0,
            consumed_cost_usd: 0.0,
            warning_threshold: 0.80,
        }
    }

    /// Record consumption of tokens and cost.
    pub fn consume(&mut self, tokens: u64, cost_usd: f64) {
        self.consumed_tokens = self.consumed_tokens.saturating_add(tokens);
        self.consumed_cost_usd += cost_usd;
    }

    /// Total tokens consumed so far.
    pub fn consumed_tokens(&self) -> u64 {
        self.consumed_tokens
    }

    /// Total cost consumed so far in USD.
    pub fn consumed_cost_usd(&self) -> f64 {
        self.consumed_cost_usd
    }

    /// Tokens remaining before hitting the budget cap.
    pub fn remaining_tokens(&self) -> u64 {
        self.max_tokens.saturating_sub(self.consumed_tokens)
    }

    /// Cost remaining before hitting the budget cap in USD.
    pub fn remaining_cost_usd(&self) -> f64 {
        (self.max_cost_usd - self.consumed_cost_usd).max(0.0)
    }

    /// Fraction of budget consumed (0.0 to 1.0+), based on whichever
    /// dimension (tokens or cost) is more utilized.
    pub fn utilization(&self) -> f64 {
        let token_util = if self.max_tokens > 0 {
            self.consumed_tokens as f64 / self.max_tokens as f64
        } else {
            0.0
        };
        let cost_util = if self.max_cost_usd > 0.0 {
            self.consumed_cost_usd / self.max_cost_usd
        } else {
            0.0
        };
        token_util.max(cost_util)
    }

    /// Returns true if either token or cost budget has been exceeded.
    pub fn is_over_budget(&self) -> bool {
        self.consumed_tokens > self.max_tokens || self.consumed_cost_usd > self.max_cost_usd
    }

    /// Returns true if utilization is at or above the warning threshold (80%).
    pub fn should_warn(&self) -> bool {
        self.utilization() >= self.warning_threshold
    }

    /// Human-readable summary string for embedding in hook responses.
    pub fn summary(&self) -> String {
        let pct = (self.utilization() * 100.0) as u64;
        format!(
            "[budget: {}tok/${:.2} used of {}tok/${:.2} | {}%]",
            self.consumed_tokens, self.consumed_cost_usd, self.max_tokens, self.max_cost_usd, pct,
        )
    }
}

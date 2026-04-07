//! Tier 3 LLM judge: calls Claude Haiku when the semantic router can't
//! confidently classify a command. Feature-gated behind `judge`.
//!
//! Invariants:
//! - Never panics — all errors return [`JudgeVerdict::Abstain`].
//! - No API call without an API key.
//! - No API call without remaining budget.
//! - Cache hit returns immediately (no API call).

use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Duration;

use lru::LruCache;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Tuning knobs for the Haiku judge.
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    /// Maximum number of API calls allowed per session.
    pub max_calls: u32,
    /// HTTP timeout in milliseconds.
    pub timeout_ms: u64,
    /// Maximum number of entries in the verdict cache.
    pub cache_size: usize,
    /// Model identifier sent in the API request.
    pub model: String,
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self {
            max_calls: 20,
            timeout_ms: 500,
            cache_size: 100,
            model: "claude-haiku-4-5-20251001".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Verdict
// ---------------------------------------------------------------------------

/// Classification returned by the judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JudgeVerdict {
    Safe,
    Risky,
    Dangerous,
    /// Returned when the judge cannot or should not render a verdict
    /// (missing API key, budget exhausted, API error, unparseable response).
    Abstain,
}

// ---------------------------------------------------------------------------
// HaikuJudge
// ---------------------------------------------------------------------------

/// Tier 3 LLM judge backed by Claude Haiku.
pub struct HaikuJudge {
    config: JudgeConfig,
    api_key: Option<String>,
    cache: Mutex<LruCache<String, JudgeVerdict>>,
    calls_made: Mutex<u32>,
}

impl HaikuJudge {
    /// Create a new judge. Reads `ANTHROPIC_API_KEY` from the environment.
    pub fn new(config: JudgeConfig) -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty());
        let cache_size =
            NonZeroUsize::new(config.cache_size).unwrap_or(NonZeroUsize::new(1).unwrap());
        Self {
            config,
            api_key,
            cache: Mutex::new(LruCache::new(cache_size)),
            calls_made: Mutex::new(0),
        }
    }

    /// Whether the judge has an API key and can potentially make calls.
    pub fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    /// Number of API calls remaining before the budget is exhausted.
    pub fn remaining_budget(&self) -> u32 {
        if let Ok(made) = self.calls_made.lock() {
            self.config.max_calls.saturating_sub(*made)
        } else {
            0
        }
    }

    /// Evaluate a command and return a verdict.
    ///
    /// Resolution order:
    /// 1. Cache hit -> return cached verdict.
    /// 2. No API key -> `Abstain`.
    /// 3. Budget exhausted -> `Abstain`.
    /// 4. Call API -> parse -> cache -> return.
    /// 5. Any error -> `Abstain`.
    pub async fn evaluate(&self, command: &str, context: Option<&str>) -> JudgeVerdict {
        let key = cache_key(command, context);

        // 1. Cache lookup.
        if let Ok(mut cache) = self.cache.lock()
            && let Some(&verdict) = cache.get(&key)
        {
            return verdict;
        }

        // 2. API key check.
        let api_key = match &self.api_key {
            Some(k) => k.clone(),
            None => return JudgeVerdict::Abstain,
        };

        // 3. Budget check.
        if self.remaining_budget() == 0 {
            return JudgeVerdict::Abstain;
        }

        // 4. Call API.
        let verdict = match self.call_api(&api_key, command, context).await {
            Ok(v) => v,
            Err(_) => JudgeVerdict::Abstain,
        };

        // 5. Cache result.
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, verdict);
        }

        verdict
    }

    /// Insert a verdict into the cache manually (useful for testing / warm-up).
    pub fn cache_verdict(&self, command: &str, context: Option<&str>, verdict: JudgeVerdict) {
        let key = cache_key(command, context);
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, verdict);
        }
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    async fn call_api(
        &self,
        api_key: &str,
        command: &str,
        context: Option<&str>,
    ) -> Result<JudgeVerdict, ()> {
        let prompt = build_judge_prompt(command, context);
        let timeout = Duration::from_millis(self.config.timeout_ms);

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| ())?;

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 50,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|_| ())?;

        // Increment call counter (after successful send, regardless of response status).
        if let Ok(mut made) = self.calls_made.lock() {
            *made = made.saturating_add(1);
        }

        if !resp.status().is_success() {
            return Err(());
        }

        let json: serde_json::Value = resp.json().await.map_err(|_| ())?;
        let text = json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|block| block.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        Ok(parse_verdict(text))
    }
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Build the prompt sent to the judge model. Public so tests can verify its
/// structure without making an API call.
pub fn build_judge_prompt(command: &str, context: Option<&str>) -> String {
    let ctx_section = match context {
        Some(ctx) => format!("\nContext: {ctx}"),
        None => String::new(),
    };
    format!(
        "You are a security classifier for shell commands in a coding-agent runtime.\n\
         Classify the following command as exactly one of: SAFE, RISKY, DANGEROUS.\n\
         Respond with a single word.\n\n\
         Command: {command}{ctx_section}"
    )
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse the first recognized verdict keyword from the model response.
fn parse_verdict(text: &str) -> JudgeVerdict {
    let upper = text.to_uppercase();
    // Scan for the first matching keyword.
    for word in upper.split_whitespace() {
        match word.trim_matches(|c: char| !c.is_alphabetic()) {
            "DANGEROUS" => return JudgeVerdict::Dangerous,
            "RISKY" => return JudgeVerdict::Risky,
            "SAFE" => return JudgeVerdict::Safe,
            _ => {}
        }
    }
    JudgeVerdict::Abstain
}

/// Deterministic cache key combining command and optional context.
fn cache_key(command: &str, context: Option<&str>) -> String {
    match context {
        Some(ctx) => format!("{command}\x00{ctx}"),
        None => command.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Unit tests (always compiled, no feature gate needed since the whole module
// is already behind #[cfg(feature = "judge")])
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_verdict_safe() {
        assert_eq!(parse_verdict("SAFE"), JudgeVerdict::Safe);
        assert_eq!(parse_verdict("safe"), JudgeVerdict::Safe);
        assert_eq!(parse_verdict("  SAFE  "), JudgeVerdict::Safe);
    }

    #[test]
    fn parse_verdict_risky() {
        assert_eq!(parse_verdict("RISKY"), JudgeVerdict::Risky);
    }

    #[test]
    fn parse_verdict_dangerous() {
        assert_eq!(parse_verdict("DANGEROUS"), JudgeVerdict::Dangerous);
    }

    #[test]
    fn parse_verdict_unknown() {
        assert_eq!(parse_verdict(""), JudgeVerdict::Abstain);
        assert_eq!(parse_verdict("maybe"), JudgeVerdict::Abstain);
        assert_eq!(parse_verdict("I think it is safe but also risky"), JudgeVerdict::Safe);
    }

    #[test]
    fn cache_key_deterministic() {
        let k1 = cache_key("ls", Some("dir"));
        let k2 = cache_key("ls", Some("dir"));
        assert_eq!(k1, k2);
        // Different context produces different key.
        let k3 = cache_key("ls", None);
        assert_ne!(k1, k3);
    }
}

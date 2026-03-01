# v0.4.0 "Developer-First" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make MetaYGN deliver visible, measurable value to developers by solving the 3 biggest Claude Code pain points: false completion claims, test manipulation, and invisible token costs.

**Architecture:** Three new verifier modules wired into existing daemon hooks. Token tracking via an `Arc<Mutex<SessionBudget>>` in AppState, exposed through all hook responses and a new `/budget` endpoint. No new crates — all changes are additive to existing crates.

**Tech Stack:** Rust (metaygn-verifiers, metaygn-daemon, metaygn-shared), existing axum daemon

**Status:** Tasks 1-2 already implemented. This plan covers Task 3 (Token Budget) + Task 4 (Wiring + Docs + Final).

---

### Task 1: Completion Verifier — ALREADY DONE

Commit `fab9547`. 7 unit tests + 1 integration test passing.
See `crates/verifiers/src/completion.rs`.

---

### Task 2: Test Integrity Guard — ALREADY DONE

Commit `f39cec9`. 7 unit tests + 1 integration test passing.
See `crates/verifiers/src/test_integrity.rs`.

---

### Task 3: Token Budget Tracker

**Files:**
- Create: `crates/shared/src/budget_tracker.rs`
- Modify: `crates/shared/src/lib.rs` — add `pub mod budget_tracker;`
- Modify: `crates/daemon/src/app_state.rs` — add `budget: Arc<Mutex<SessionBudget>>`
- Create: `crates/daemon/src/api/budget.rs` — GET `/budget` + POST `/budget/consume`
- Modify: `crates/daemon/src/api/mod.rs` — add `pub mod budget;` + routes
- Modify: `crates/daemon/src/api/hooks.rs` — append budget info to every hook response
- Test: `crates/shared/tests/budget_tracker_test.rs`
- Test: `crates/daemon/tests/api_test.rs` — add budget integration tests

**Step 1: Write failing tests for SessionBudget**

Create `crates/shared/tests/budget_tracker_test.rs`:
```rust
use metaygn_shared::budget_tracker::SessionBudget;

#[test]
fn new_budget_starts_at_zero_consumed() {
    let b = SessionBudget::new(10_000, 0.10);
    assert_eq!(b.consumed_tokens(), 0);
    assert_eq!(b.consumed_cost_usd(), 0.0);
    assert!(!b.is_over_budget());
}

#[test]
fn consume_tokens_updates_state() {
    let mut b = SessionBudget::new(10_000, 0.10);
    b.consume(500, 0.01);
    assert_eq!(b.consumed_tokens(), 500);
    assert!((b.consumed_cost_usd() - 0.01).abs() < 0.001);
    assert!(!b.is_over_budget());
}

#[test]
fn over_budget_detected() {
    let mut b = SessionBudget::new(1_000, 0.05);
    b.consume(1_200, 0.06);
    assert!(b.is_over_budget());
}

#[test]
fn utilization_percentage() {
    let mut b = SessionBudget::new(10_000, 1.00);
    b.consume(8_000, 0.80);
    assert!((b.utilization() - 0.80).abs() < 0.01);
}

#[test]
fn warning_at_80_percent() {
    let mut b = SessionBudget::new(10_000, 1.00);
    b.consume(8_500, 0.85);
    assert!(b.should_warn());  // >= 80%
}

#[test]
fn no_warning_below_80_percent() {
    let mut b = SessionBudget::new(10_000, 1.00);
    b.consume(5_000, 0.50);
    assert!(!b.should_warn());
}

#[test]
fn budget_summary_string() {
    let mut b = SessionBudget::new(10_000, 0.10);
    b.consume(3_000, 0.03);
    let summary = b.summary();
    // Should contain tokens and cost info
    assert!(summary.contains("3000"));
    assert!(summary.contains("0.03"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p metaygn-shared --test budget_tracker_test`
Expected: FAIL — `budget_tracker` module not found

**Step 3: Implement SessionBudget**

Create `crates/shared/src/budget_tracker.rs`:
```rust
use serde::{Deserialize, Serialize};

/// Tracks token consumption and cost for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBudget {
    max_tokens: u64,
    max_cost_usd: f64,
    consumed_tokens: u64,
    consumed_cost_usd: f64,
    warning_threshold: f64,  // default 0.80 (80%)
}

impl SessionBudget {
    pub fn new(max_tokens: u64, max_cost_usd: f64) -> Self {
        Self {
            max_tokens,
            max_cost_usd,
            consumed_tokens: 0,
            consumed_cost_usd: 0.0,
            warning_threshold: 0.80,
        }
    }

    pub fn consume(&mut self, tokens: u64, cost_usd: f64) {
        self.consumed_tokens += tokens;
        self.consumed_cost_usd += cost_usd;
    }

    pub fn consumed_tokens(&self) -> u64 {
        self.consumed_tokens
    }

    pub fn consumed_cost_usd(&self) -> f64 {
        self.consumed_cost_usd
    }

    pub fn remaining_tokens(&self) -> u64 {
        self.max_tokens.saturating_sub(self.consumed_tokens)
    }

    pub fn remaining_cost_usd(&self) -> f64 {
        (self.max_cost_usd - self.consumed_cost_usd).max(0.0)
    }

    pub fn utilization(&self) -> f64 {
        if self.max_tokens == 0 { return 0.0; }
        self.consumed_tokens as f64 / self.max_tokens as f64
    }

    pub fn is_over_budget(&self) -> bool {
        self.consumed_tokens > self.max_tokens
            || self.consumed_cost_usd > self.max_cost_usd
    }

    pub fn should_warn(&self) -> bool {
        self.utilization() >= self.warning_threshold
    }

    /// Human-readable one-line summary for hook responses
    pub fn summary(&self) -> String {
        format!(
            "[budget: {}tok/${:.2} used of {}tok/${:.2} | {:.0}%{}]",
            self.consumed_tokens,
            self.consumed_cost_usd,
            self.max_tokens,
            self.max_cost_usd,
            self.utilization() * 100.0,
            if self.should_warn() { " ⚠ BUDGET WARNING" } else { "" },
        )
    }
}
```

Add `pub mod budget_tracker;` to `crates/shared/src/lib.rs`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p metaygn-shared --test budget_tracker_test`
Expected: 7 PASS

**Step 5: Commit**

```bash
git add crates/shared/src/budget_tracker.rs crates/shared/src/lib.rs crates/shared/tests/budget_tracker_test.rs
git commit -m "feat(shared): SessionBudget tracker with utilization, warnings, and summary"
```

**Step 6: Wire budget into AppState**

Modify `crates/daemon/src/app_state.rs`:
- Add `use metaygn_shared::budget_tracker::SessionBudget;`
- Add field: `pub budget: Arc<Mutex<SessionBudget>>`
- In `new_in_memory()`: init with `SessionBudget::new(100_000, 1.00)` (100K tokens, $1)
- In `new()`: same defaults

**Step 7: Create budget API endpoint**

Create `crates/daemon/src/api/budget.rs`:
```rust
use axum::extract::State;
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::app_state::AppState;

/// GET /budget — current session budget status
pub async fn get_budget(State(state): State<AppState>) -> Json<Value> {
    let budget = state.budget.lock().unwrap();
    Json(json!({
        "consumed_tokens": budget.consumed_tokens(),
        "consumed_cost_usd": budget.consumed_cost_usd(),
        "remaining_tokens": budget.remaining_tokens(),
        "remaining_cost_usd": budget.remaining_cost_usd(),
        "utilization": budget.utilization(),
        "over_budget": budget.is_over_budget(),
        "warning": budget.should_warn(),
        "summary": budget.summary(),
    }))
}

#[derive(Deserialize)]
pub struct ConsumeRequest {
    pub tokens: u64,
    pub cost_usd: f64,
}

/// POST /budget/consume — record token consumption
pub async fn consume(
    State(state): State<AppState>,
    Json(req): Json<ConsumeRequest>,
) -> Json<Value> {
    let mut budget = state.budget.lock().unwrap();
    budget.consume(req.tokens, req.cost_usd);
    Json(json!({
        "ok": true,
        "summary": budget.summary(),
    }))
}
```

Add to `crates/daemon/src/api/mod.rs`:
- `pub mod budget;`
- Wire routes: `GET /budget` and `POST /budget/consume`

**Step 8: Append budget summary to ALL hook responses**

In `crates/daemon/src/api/hooks.rs`, at the end of each handler (`pre_tool_use`, `post_tool_use`, `user_prompt_submit`, `stop`), before returning:

```rust
// Append budget summary to context
let budget = state.budget.lock().unwrap();
// If the response has additionalContext, append budget summary
// If not, add it as new context
```

The simplest approach: create a helper function:
```rust
fn append_budget_to_output(output: &mut HookOutput, state: &AppState) {
    let budget = state.budget.lock().unwrap();
    let summary = budget.summary();
    if let Some(ref mut hso) = output.hook_specific_output {
        if let Some(ref mut ctx) = hso.additional_context {
            ctx.push_str(&format!(" {}", summary));
        } else {
            hso.additional_context = Some(summary);
        }
    }
    // If output has no hookSpecificOutput yet, add one with just budget
    else {
        output.hook_specific_output = Some(metaygn_shared::protocol::HookSpecificOutput {
            hook_event_name: None,
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: Some(summary),
        });
    }
}
```

Also, in `user_prompt_submit`, consume a small estimate (e.g., prompt length as token estimate):
```rust
let estimated_tokens = (prompt.len() / 4) as u64;  // rough estimate: 4 chars ≈ 1 token
state.budget.lock().unwrap().consume(estimated_tokens, estimated_tokens as f64 * 0.000003);
```

**Step 9: Write integration tests**

Add to `crates/daemon/tests/api_test.rs`:
```rust
#[tokio::test]
async fn budget_starts_at_zero() {
    // GET /budget → consumed_tokens = 0
}

#[tokio::test]
async fn budget_consume_updates() {
    // POST /budget/consume {tokens: 500, cost_usd: 0.01}
    // GET /budget → consumed_tokens = 500
}

#[tokio::test]
async fn hook_responses_include_budget() {
    // POST /hooks/user-prompt-submit → response contains "[budget:"
}
```

**Step 10: Run all tests, verify**

Run: `cargo test --workspace`
Expected: ALL pass (196 existing + ~10 new = ~206)

**Step 11: Commit**

```bash
git add crates/daemon/ crates/shared/
git commit -m "feat(daemon): token budget tracker — visible cost transparency in every hook response"
```

---

### Task 4: Final wiring + docs + merge

**Files:**
- Modify: `CHANGELOG.md` — add v0.4.0 section
- Modify: `README.md` — add Developer-First features section
- Modify: `.claude-plugin/plugin.json` — bump to v0.4.0
- Modify: `memory-bank/progress.md` — update
- Modify: `memory-bank/activeContext.md` — update

**Step 1: Update CHANGELOG.md**

Add v0.4.0 section:
```markdown
## 0.4.0 "Developer-First"
### Added
- Completion Verifier: catches false "Done!" claims by checking file existence
- Test Integrity Guard: detects when Claude weakens tests instead of fixing code
- Token Budget Tracker: visible cost transparency in every hook response
- GET /budget and POST /budget/consume endpoints
- Budget summary appended to all hook responses

### Changed
- Stop hook now runs completion verification before accepting "Done!"
- PreToolUse hook now checks test integrity on Edit/MultiEdit of test files
```

**Step 2: Update README.md**

Add a "Developer-First Features (v0.4.0)" section after the architecture diagram.

**Step 3: Bump plugin.json to v0.4.0**

**Step 4: Update memory-bank**

**Step 5: Run full test suite**

Run: `cargo test --workspace`
Expected: ALL pass

**Step 6: Commit and push**

```bash
git add CHANGELOG.md README.md .claude-plugin/plugin.json memory-bank/
git commit -m "docs: v0.4.0 Developer-First — changelog, readme, plugin version bump"
git push -u origin feat/v0.4.0-developer-first
```

**Step 7: Create PR and merge**

```bash
gh pr create --title "feat: v0.4.0 Developer-First — Completion Verifier + Test Integrity + Token Budget"
gh pr merge --merge --admin
```

---

## Success criteria

- [ ] `cargo test --workspace` — all tests pass (~206+)
- [ ] Completion Verifier catches "Done!" with missing files
- [ ] Test Integrity Guard asks confirmation when assertions are removed
- [ ] Token Budget shows `[budget: ...]` in every hook response
- [ ] GET /budget returns current consumption
- [ ] POST /budget/consume updates the budget
- [ ] CHANGELOG, README, plugin.json updated to v0.4.0

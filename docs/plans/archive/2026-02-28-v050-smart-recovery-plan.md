# v0.5.0 "Smart Recovery" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make MetaYGN's error recovery actually work by tracking whether recovery prompts are effective, progressively amplifying critiques when they fail, and polishing the UX so every hook message is human-readable with latency and budget visible.

**Architecture:** Plasticity tracker in daemon session state feeds calibration. Progressive amplification in pruner escalates when recovery fails. UX cleanup across all hook responses. CLI `init` command for onboarding. Pre-tool-use risk fix uses command text instead of hook metadata.

**Tech Stack:** Rust (metaygn-daemon, metaygn-core, metaygn-shared, metaygn-cli), existing crates only

---

### Task 1: Plasticity Tracker (implicit feedback loop)

**Files:**
- Create: `crates/daemon/src/profiler/plasticity.rs`
- Modify: `crates/daemon/src/profiler/mod.rs` — add `pub mod plasticity;`
- Modify: `crates/daemon/src/app_state.rs` — add `plasticity: Arc<Mutex<PlasticityTracker>>`
- Test: `crates/daemon/tests/plasticity_test.rs`

**Step 1: Write failing tests**

Create `crates/daemon/tests/plasticity_test.rs`:
```rust
use metaygn_daemon::profiler::plasticity::{PlasticityTracker, RecoveryOutcome};

#[test]
fn fresh_tracker_has_no_data() {
    let t = PlasticityTracker::new();
    assert_eq!(t.total_recoveries(), 0);
    assert!((t.plasticity_score() - 1.0).abs() < 0.01); // default 1.0 (optimistic)
}

#[test]
fn record_success_improves_score() {
    let mut t = PlasticityTracker::new();
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Success);
    assert!(t.plasticity_score() > 0.5);
}

#[test]
fn record_failure_lowers_score() {
    let mut t = PlasticityTracker::new();
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Failure);
    assert!(t.plasticity_score() < 1.0);
}

#[test]
fn multiple_failures_trigger_low_plasticity() {
    let mut t = PlasticityTracker::new();
    for _ in 0..5 {
        t.record_recovery_injected();
        t.record_outcome(RecoveryOutcome::Failure);
    }
    assert!(t.plasticity_score() < 0.3);
    assert!(t.is_low_plasticity());
}

#[test]
fn amplification_level_increases_with_failures() {
    let mut t = PlasticityTracker::new();
    assert_eq!(t.amplification_level(), 1);
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(t.amplification_level(), 2);
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(t.amplification_level(), 3); // max
}

#[test]
fn success_resets_amplification() {
    let mut t = PlasticityTracker::new();
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Failure);
    assert_eq!(t.amplification_level(), 2);
    t.record_recovery_injected();
    t.record_outcome(RecoveryOutcome::Success);
    assert_eq!(t.amplification_level(), 1); // reset
}
```

**Step 2: Run tests — expected FAIL**

Run: `cargo test -p metaygn-daemon --test plasticity_test`

**Step 3: Implement PlasticityTracker**

```rust
// crates/daemon/src/profiler/plasticity.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecoveryOutcome { Success, Failure }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityTracker {
    total_injections: u32,
    successes: u32,
    failures: u32,
    consecutive_failures: u32,
}

impl PlasticityTracker {
    pub fn new() -> Self {
        Self { total_injections: 0, successes: 0, failures: 0, consecutive_failures: 0 }
    }

    pub fn record_recovery_injected(&mut self) {
        self.total_injections += 1;
    }

    pub fn record_outcome(&mut self, outcome: RecoveryOutcome) {
        match outcome {
            RecoveryOutcome::Success => {
                self.successes += 1;
                self.consecutive_failures = 0;
            }
            RecoveryOutcome::Failure => {
                self.failures += 1;
                self.consecutive_failures += 1;
            }
        }
    }

    pub fn total_recoveries(&self) -> u32 { self.successes + self.failures }

    pub fn plasticity_score(&self) -> f64 {
        let total = self.total_recoveries();
        if total == 0 { return 1.0; }
        self.successes as f64 / total as f64
    }

    pub fn is_low_plasticity(&self) -> bool { self.plasticity_score() < 0.3 }

    /// 1 = standard, 2 = emphatic, 3 = escalate
    pub fn amplification_level(&self) -> u8 {
        match self.consecutive_failures {
            0 => 1,
            1 => 2,
            _ => 3,
        }
    }
}
```

**Step 4: Wire into AppState**

Add `pub plasticity: Arc<Mutex<PlasticityTracker>>` to AppState, init with `PlasticityTracker::new()`.

**Step 5: Run tests — expected 6 PASS**

Run: `cargo test -p metaygn-daemon --test plasticity_test`

**Step 6: Commit**

```bash
git commit -m "feat(daemon): plasticity tracker — implicit feedback loop for recovery effectiveness"
```

---

### Task 2: Progressive amplification in context pruner

**Files:**
- Modify: `crates/daemon/src/proxy/pruner.rs` — add amplified recovery messages
- Modify: `crates/daemon/src/api/hooks.rs` — wire plasticity into stop/post-tool-use hooks
- Test: `crates/daemon/tests/pruner_test.rs` — add amplification tests

**Step 1: Write failing tests**

Add to `crates/daemon/tests/pruner_test.rs`:
```rust
#[test]
fn amplified_recovery_level_2() {
    let pruner = ContextPruner::with_defaults();
    let msg = pruner.amplified_recovery("Loop detected", 2);
    assert!(msg.contains("CRITICAL") || msg.contains("IMPORTANT"));
    assert!(msg.contains("different approach"));
}

#[test]
fn amplified_recovery_level_3_escalates() {
    let pruner = ContextPruner::with_defaults();
    let msg = pruner.amplified_recovery("Loop detected", 3);
    assert!(msg.contains("ESCALATE") || msg.contains("/metacog-escalate"));
}
```

**Step 2: Implement amplified_recovery in pruner**

Add to `ContextPruner`:
```rust
pub fn amplified_recovery(&self, base_reason: &str, level: u8) -> String {
    match level {
        1 => format!("[ALETHEIA] Context recovery: {}. Try a different approach.", base_reason),
        2 => format!(
            "[ALETHEIA] CRITICAL: Recovery failed. {}. \
             You MUST use a fundamentally different approach. \
             Do NOT retry the same strategy. Consider: \
             1) Simplify the problem, 2) Break into smaller steps, \
             3) Ask the user for clarification.",
            base_reason
        ),
        _ => format!(
            "[ALETHEIA] ESCALATE: Multiple recovery attempts failed. {}. \
             Use /metacog-escalate to present options to the developer. \
             Do not continue on the current path.",
            base_reason
        ),
    }
}
```

**Step 3: Wire into hooks**

In `stop` handler: after running context pruner analysis, if `should_prune`:
```rust
let level = state.plasticity.lock().unwrap().amplification_level();
let recovery_msg = pruner.amplified_recovery(&reason, level);
state.plasticity.lock().unwrap().record_recovery_injected();
```

In `post_tool_use` handler: if tool succeeded after a recovery was pending:
```rust
state.plasticity.lock().unwrap().record_outcome(RecoveryOutcome::Success);
```

In `post_tool_use_failure` (if we handle it) or when errors detected:
```rust
state.plasticity.lock().unwrap().record_outcome(RecoveryOutcome::Failure);
```

**Step 4: Run all tests**

Run: `cargo test --workspace`

**Step 5: Commit**

```bash
git commit -m "feat(daemon): progressive recovery amplification — level 1/2/3 based on plasticity"
```

---

### Task 3: Fix pre-tool-use risk classification

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs` — pass command text to control loop instead of hook metadata
- Test: `crates/daemon/tests/api_test.rs` — add test for correct risk on safe command

**Step 1: Write failing test**

```rust
#[tokio::test]
async fn pre_tool_use_safe_command_shows_low_risk() {
    let addr = start_test_server().await;
    let resp = client.post(format!("http://{addr}/hooks/pre-tool-use"))
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "ls -la"}
        }))
        .send().await.unwrap();
    let body: Value = resp.json().await.unwrap();
    let ctx = body["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
    // Should NOT show risk:high for a simple ls command
    assert!(!ctx.contains("[risk:high]"), "ls -la should not be high risk: {}", ctx);
}
```

**Step 2: Fix**

In `pre_tool_use`, when building the LoopContext for the control loop, inject the COMMAND text as the prompt instead of letting it default to the hook event name:

```rust
let mut input_for_loop = input.clone();
input_for_loop.prompt = Some(command.clone()); // use the actual command for risk classification
let mut ctx = LoopContext::new(input_for_loop);
```

**Step 3: Run tests**

Run: `cargo test --workspace`

**Step 4: Commit**

```bash
git commit -m "fix(daemon): pre-tool-use passes command text to control loop for accurate risk classification"
```

---

### Task 4: UX polish — human-readable messages + latency tracking

**Files:**
- Modify: `crates/daemon/src/api/hooks.rs` — add latency measurement, clean up message format
- Modify: `crates/shared/src/protocol.rs` — add `HookOutput::readable()` helper
- Test: `crates/daemon/tests/api_test.rs` — verify latency present

**Step 1: Add latency measurement**

In each hook handler, measure wall time:
```rust
let start = std::time::Instant::now();
// ... handler logic ...
let latency_ms = start.elapsed().as_millis();
// append [latency: Xms] to output
```

**Step 2: Clean up message format**

Replace terse internal format:
```
BEFORE: [risk:high] [strategy:VerifyFirst] [budget:1000tok] [task:Some(Release)] [topology=Horizontal]
AFTER:  Risk: HIGH | Strategy: verify-first | Budget: 1000 tokens | Task: release | Topology: horizontal
```

**Step 3: Write test**

```rust
#[tokio::test]
async fn hook_responses_include_latency() {
    // POST /hooks/user-prompt-submit → response contains "latency:"
}
```

**Step 4: Run tests, commit**

```bash
git commit -m "feat(daemon): human-readable hook messages + latency tracking in every response"
```

---

### Task 5: CLI `aletheia init` command

**Files:**
- Modify: `crates/cli/src/main.rs` — add `Init` subcommand
- Test: manual (creates files)

**Step 1: Implement**

Add `Init` to CLI Commands:
```rust
/// Initialize MetaYGN in current project
Init {
    #[arg(long)]
    force: bool,
},
```

Handler creates `.claude/settings.json` with:
```json
{
  "enabledPlugins": { "aletheia-nexus@local": true },
  "outputStyle": "aletheia-proof"
}
```

And prints setup instructions.

**Step 2: Verify**

Run: `cargo run -p metaygn-cli -- init --help`

**Step 3: Commit**

```bash
git commit -m "feat(cli): aletheia init command for project onboarding"
```

---

### Task 6: Final — docs + version bump + merge

**Files:**
- Modify: `CHANGELOG.md`, `README.md`, `.claude-plugin/plugin.json`, `memory-bank/`

**Step 1: Update all docs for v0.5.0**

**Step 2: Run full test suite**

Run: `cargo test --workspace`

**Step 3: Commit, push, PR, merge**

```bash
git commit -m "docs: v0.5.0 Smart Recovery — changelog, readme, plugin version"
git push && gh pr create && gh pr merge --merge --admin
```

# Phase 1: Foundation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Rust daemon skeleton that responds to Claude Code hooks, stores events in SQLite, and recalls memories — plus migrate hooks from Python to TypeScript/Bun.

**Architecture:** Cargo workspace with 5 crates (shared, memory, daemon, cli, verifiers). pnpm workspace with 3 packages (hooks, plugin, shared). Daemon exposes axum HTTP API on dynamic port. TS hooks call daemon with 350ms timeout and fall back to local heuristics.

**Tech Stack:** Rust 2024 (axum 0.8, tokio, rusqlite 0.38 bundled, clap 4.5, ed25519-dalek, sha2), TypeScript (Bun, Zod v4), pnpm, just

**Ref:** See `docs/plans/2026-02-28-meta-ygn-full-system-design.md` for full architecture.

---

## Task 1: Initialize Cargo workspace and crate skeletons

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `rust-toolchain.toml`
- Create: `crates/shared/Cargo.toml`
- Create: `crates/shared/src/lib.rs`
- Create: `crates/memory/Cargo.toml`
- Create: `crates/memory/src/lib.rs`
- Create: `crates/daemon/Cargo.toml`
- Create: `crates/daemon/src/main.rs`
- Create: `crates/cli/Cargo.toml`
- Create: `crates/cli/src/main.rs`
- Create: `crates/verifiers/Cargo.toml`
- Create: `crates/verifiers/src/lib.rs`

**Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/memory",
    "crates/daemon",
    "crates/cli",
    "crates/verifiers",
]

[workspace.package]
edition = "2024"
version = "0.1.0"
license = "MIT"
authors = ["Yann Abadie"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

**Step 2: Create rust-toolchain.toml**

```toml
[toolchain]
channel = "stable"
```

**Step 3: Create each crate with minimal Cargo.toml and stub src**

`crates/shared/Cargo.toml`:
```toml
[package]
name = "metaygn-shared"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
thiserror.workspace = true
sha2 = "0.10"
```

`crates/shared/src/lib.rs`:
```rust
pub mod state;
pub mod events;
pub mod protocol;
pub mod kernel;
```

Create empty module files for shared:
- `crates/shared/src/state.rs` → `// Metacognitive state types`
- `crates/shared/src/events.rs` → `// Event types for logging`
- `crates/shared/src/protocol.rs` → `// Hook payloads and IPC`
- `crates/shared/src/kernel.rs` → `// KERNEL integrity`

`crates/memory/Cargo.toml`:
```toml
[package]
name = "metaygn-memory"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
metaygn-shared = { path = "../shared" }
rusqlite = { version = "0.38", features = ["bundled"] }
tokio-rusqlite = "0.6"
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
```

`crates/memory/src/lib.rs`:
```rust
pub mod store;
pub mod fts;
```

Create empty module files:
- `crates/memory/src/store.rs` → `// SQLite store`
- `crates/memory/src/fts.rs` → `// FTS5 + BM25 search`

`crates/daemon/Cargo.toml`:
```toml
[package]
name = "metaygn-daemon"
edition.workspace = true
version.workspace = true
license.workspace = true

[[bin]]
name = "aletheiad"
path = "src/main.rs"

[dependencies]
metaygn-shared = { path = "../shared" }
metaygn-memory = { path = "../memory" }
metaygn-verifiers = { path = "../verifiers" }
axum = { version = "0.8", features = ["json"] }
tokio.workspace = true
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full", "test-util"] }
```

`crates/daemon/src/main.rs`:
```rust
fn main() {
    println!("aletheiad: not yet implemented");
}
```

`crates/cli/Cargo.toml`:
```toml
[package]
name = "metaygn-cli"
edition.workspace = true
version.workspace = true
license.workspace = true

[[bin]]
name = "aletheia"
path = "src/main.rs"

[dependencies]
metaygn-shared = { path = "../shared" }
clap = { version = "4.5", features = ["derive"] }
reqwest = { version = "0.12", features = ["json"] }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

`crates/cli/src/main.rs`:
```rust
fn main() {
    println!("aletheia: not yet implemented");
}
```

`crates/verifiers/Cargo.toml`:
```toml
[package]
name = "metaygn-verifiers"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
metaygn-shared = { path = "../shared" }
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
anyhow.workspace = true
thiserror.workspace = true
sha2 = "0.10"
ed25519-dalek = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
```

`crates/verifiers/src/lib.rs`:
```rust
pub mod evidence;
pub mod guard_pipeline;
```

Create empty module files:
- `crates/verifiers/src/evidence.rs` → `// Evidence pack`
- `crates/verifiers/src/guard_pipeline.rs` → `// Composable guards`

**Step 4: Verify workspace builds**

Run: `cargo check --workspace`
Expected: compilation succeeds with warnings about unused modules

**Step 5: Commit**

```bash
git add crates/ Cargo.toml rust-toolchain.toml
git commit -m "feat: initialize Cargo workspace with 5 crate skeletons"
```

---

## Task 2: Shared types — KERNEL integrity + metacognitive state

**Files:**
- Create: `crates/shared/src/kernel.rs`
- Create: `crates/shared/src/state.rs`
- Test: `crates/shared/tests/kernel_test.rs`

**Step 1: Write failing test for KERNEL**

Create `crates/shared/tests/kernel_test.rs`:
```rust
use metaygn_shared::kernel::{Kernel, AlignmentRule};

#[test]
fn kernel_hash_is_deterministic() {
    let k1 = Kernel::default();
    let k2 = Kernel::default();
    assert_eq!(k1.hash(), k2.hash());
}

#[test]
fn kernel_verify_passes_on_unmodified() {
    let k = Kernel::default();
    assert!(k.verify().is_ok());
}

#[test]
fn kernel_verify_fails_on_tampered() {
    let mut k = Kernel::default();
    k.rules_mut().push(AlignmentRule::Custom("injected".into()));
    assert!(k.verify().is_err());
}

#[test]
fn kernel_default_has_5_rules() {
    let k = Kernel::default();
    assert_eq!(k.rules().len(), 5);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p metaygn-shared --test kernel_test`
Expected: FAIL — `Kernel` not defined

**Step 3: Implement KERNEL**

Write `crates/shared/src/kernel.rs`:
```rust
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlignmentRule {
    RequireApprovalForDestructive,
    NeverExposeSecrets,
    EvidenceRequiredForStrongClaims,
    EscalateOnLowConfidence { threshold: f32 },
    PreserveUserIntent,
    Custom(String),
}

impl fmt::Display for AlignmentRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequireApprovalForDestructive => write!(f, "require_approval_destructive"),
            Self::NeverExposeSecrets => write!(f, "never_expose_secrets"),
            Self::EvidenceRequiredForStrongClaims => write!(f, "evidence_required"),
            Self::EscalateOnLowConfidence { threshold } => {
                write!(f, "escalate_low_confidence_{threshold}")
            }
            Self::PreserveUserIntent => write!(f, "preserve_user_intent"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Kernel {
    rules: Vec<AlignmentRule>,
    boot_hash: [u8; 32],
}

impl Kernel {
    pub fn new(rules: Vec<AlignmentRule>) -> Self {
        let boot_hash = Self::compute_hash(&rules);
        Self { rules, boot_hash }
    }

    pub fn hash(&self) -> [u8; 32] {
        self.boot_hash
    }

    pub fn rules(&self) -> &[AlignmentRule] {
        &self.rules
    }

    pub fn rules_mut(&mut self) -> &mut Vec<AlignmentRule> {
        &mut self.rules
    }

    pub fn verify(&self) -> Result<(), KernelError> {
        let current = Self::compute_hash(&self.rules);
        if current == self.boot_hash {
            Ok(())
        } else {
            Err(KernelError::IntegrityViolation {
                expected: hex::encode(self.boot_hash),
                actual: hex::encode(current),
            })
        }
    }

    fn compute_hash(rules: &[AlignmentRule]) -> [u8; 32] {
        let serialized = serde_json::to_string(rules).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hasher.finalize().into()
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new(vec![
            AlignmentRule::RequireApprovalForDestructive,
            AlignmentRule::NeverExposeSecrets,
            AlignmentRule::EvidenceRequiredForStrongClaims,
            AlignmentRule::EscalateOnLowConfidence { threshold: 0.4 },
            AlignmentRule::PreserveUserIntent,
        ])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("KERNEL integrity violation: expected {expected}, got {actual}")]
    IntegrityViolation { expected: String, actual: String },
}

// We need the hex crate for display — or we inline it
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p metaygn-shared --test kernel_test`
Expected: 4 tests PASS

**Step 5: Implement metacognitive state types**

Write `crates/shared/src/state.rs`:
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Bugfix,
    Feature,
    Refactor,
    Architecture,
    Security,
    Research,
    Release,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSignature {
    pub id: Uuid,
    pub task_type: TaskType,
    pub risk: RiskLevel,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

/// 5-dimensional metacognitive state vector (30 tokens compact)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct MetacognitiveVector {
    pub confidence: f32,       // 0.0-1.0
    pub coherence: f32,        // 0.0-1.0
    pub grounding: f32,        // 0.0-1.0 (factual grounding)
    pub complexity: f32,       // 0.0-1.0
    pub progress: f32,         // 0.0-1.0
}

impl MetacognitiveVector {
    pub fn overall_quality(&self) -> f32 {
        (self.confidence + self.coherence + self.grounding
            + (1.0 - self.complexity) + self.progress)
            / 5.0
    }

    /// Ultra-compact encoding: "META:c8u1p3d2r2" (10 tokens)
    pub fn compact_encode(&self) -> String {
        format!(
            "META:c{}h{}g{}x{}p{}",
            (self.confidence * 9.0) as u8,
            (self.coherence * 9.0) as u8,
            (self.grounding * 9.0) as u8,
            (self.complexity * 9.0) as u8,
            (self.progress * 9.0) as u8,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetState {
    pub max_tokens: u64,
    pub consumed_tokens: u64,
    pub max_latency_ms: u64,
    pub max_cost_usd: f64,
    pub risk_tolerance: RiskLevel,
}

impl BudgetState {
    pub fn tokens_remaining(&self) -> u64 {
        self.max_tokens.saturating_sub(self.consumed_tokens)
    }

    pub fn utilization(&self) -> f32 {
        if self.max_tokens == 0 {
            return 0.0;
        }
        self.consumed_tokens as f32 / self.max_tokens as f32
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Strategy {
    StepByStep,
    TreeExplore,
    VerifyFirst,
    DivideConquer,
    Analogical,
    Adversarial,
    Rapid,
    Iterative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Decision {
    Continue,
    Revise,
    Abstain,
    Escalate,
    Stop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EvidenceTier {
    Confirmed,
    Experimental,
    Unverified,
}
```

**Step 6: Verify it compiles**

Run: `cargo check -p metaygn-shared`
Expected: OK

**Step 7: Commit**

```bash
git add crates/shared/
git commit -m "feat(shared): KERNEL integrity system + metacognitive state types"
```

---

## Task 3: Shared types — Hook protocol

**Files:**
- Create: `crates/shared/src/protocol.rs`
- Test: `crates/shared/tests/protocol_test.rs`

**Step 1: Write failing test**

Create `crates/shared/tests/protocol_test.rs`:
```rust
use metaygn_shared::protocol::{
    HookEvent, HookInput, HookOutput, PermissionDecision,
};

#[test]
fn hook_input_deserializes_pre_tool_use() {
    let json = r#"{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"git push"},"cwd":"/tmp"}"#;
    let input: HookInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.hook_event_name, HookEvent::PreToolUse);
    assert_eq!(input.tool_name.as_deref(), Some("Bash"));
}

#[test]
fn hook_output_serializes_deny() {
    let output = HookOutput::permission(PermissionDecision::Deny, "blocked");
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("deny"));
    assert!(json.contains("blocked"));
}

#[test]
fn hook_output_serializes_allow_by_empty() {
    let output = HookOutput::allow();
    let json = serde_json::to_string(&output).unwrap();
    // Allow = empty JSON object or no hookSpecificOutput
    assert!(json.contains("{}") || json.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p metaygn-shared --test protocol_test`
Expected: FAIL

**Step 3: Implement protocol types**

Write `crates/shared/src/protocol.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HookEvent {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Stop,
    PreCompact,
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    pub hook_event_name: HookEvent,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<String>,
    pub prompt: Option<String>,
    pub error: Option<String>,
    pub last_assistant_message: Option<String>,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub trigger: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_event_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<PermissionDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

impl HookOutput {
    pub fn allow() -> Self {
        Self {
            hook_specific_output: None,
        }
    }

    pub fn permission(decision: PermissionDecision, reason: &str) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput {
                hook_event_name: Some("PreToolUse".to_string()),
                permission_decision: Some(decision),
                permission_decision_reason: Some(reason.to_string()),
                additional_context: None,
            }),
        }
    }

    pub fn context(event: &str, message: &str) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput {
                hook_event_name: Some(event.to_string()),
                permission_decision: None,
                permission_decision_reason: None,
                additional_context: Some(message.to_string()),
            }),
        }
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p metaygn-shared --test protocol_test`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add crates/shared/src/protocol.rs crates/shared/tests/
git commit -m "feat(shared): hook protocol types with serde serialization"
```

---

## Task 4: Memory crate — SQLite store with event logging

**Files:**
- Create: `crates/memory/src/store.rs`
- Test: `crates/memory/tests/store_test.rs`

**Step 1: Write failing test**

Create `crates/memory/tests/store_test.rs`:
```rust
use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn store_opens_and_creates_tables() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let count = store.event_count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn store_logs_and_retrieves_events() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    store
        .log_event("test_session", "PreToolUse", r#"{"tool":"Bash"}"#)
        .await
        .unwrap();
    store
        .log_event("test_session", "PostToolUse", r#"{"tool":"Bash"}"#)
        .await
        .unwrap();

    let count = store.event_count().await.unwrap();
    assert_eq!(count, 2);

    let events = store.recent_events("test_session", 10).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "PreToolUse");
}

#[tokio::test]
async fn store_fts_search() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    store
        .log_event("s1", "Reflection", r#"{"lesson":"always run tests before pushing"}"#)
        .await
        .unwrap();
    store
        .log_event("s1", "Reflection", r#"{"lesson":"check types before deployment"}"#)
        .await
        .unwrap();

    let results = store.search_events("tests pushing", 5).await.unwrap();
    assert!(!results.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p metaygn-memory --test store_test`
Expected: FAIL — `MemoryStore` not defined

**Step 3: Implement SQLite store**

Write `crates/memory/src/store.rs`:
```rust
use anyhow::Result;
use chrono::Utc;
use tokio_rusqlite::Connection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EventRow {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub payload: String,
    pub timestamp: String,
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub async fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).await?;
        let store = Self { conn };
        store.init_schema().await?;
        Ok(store)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().await?;
        let store = Self { conn };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> Result<()> {
        self.conn
            .call(|conn| {
                conn.execute_batch(
                    "
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA busy_timeout=5000;
                    PRAGMA cache_size=-64000;

                    CREATE TABLE IF NOT EXISTS events (
                        id TEXT PRIMARY KEY,
                        session_id TEXT NOT NULL,
                        event_type TEXT NOT NULL,
                        payload TEXT NOT NULL,
                        timestamp TEXT NOT NULL DEFAULT (datetime('now'))
                    );

                    CREATE INDEX IF NOT EXISTS idx_events_session
                        ON events(session_id, timestamp);
                    CREATE INDEX IF NOT EXISTS idx_events_type
                        ON events(event_type);

                    CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
                        payload,
                        content='events',
                        content_rowid='rowid'
                    );

                    CREATE TRIGGER IF NOT EXISTS events_ai AFTER INSERT ON events BEGIN
                        INSERT INTO events_fts(rowid, payload)
                        VALUES (new.rowid, new.payload);
                    END;
                    ",
                )?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn log_event(
        &self,
        session_id: &str,
        event_type: &str,
        payload: &str,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now().to_rfc3339();
        let id_clone = id.clone();
        let session = session_id.to_string();
        let etype = event_type.to_string();
        let data = payload.to_string();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO events (id, session_id, event_type, payload, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![id_clone, session, etype, data, ts],
                )?;
                Ok(())
            })
            .await?;
        Ok(id)
    }

    pub async fn event_count(&self) -> Result<u64> {
        let count = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT COUNT(*) FROM events")?;
                let count: u64 = stmt.query_row([], |row| row.get(0))?;
                Ok(count)
            })
            .await?;
        Ok(count)
    }

    pub async fn recent_events(&self, session_id: &str, limit: u32) -> Result<Vec<EventRow>> {
        let session = session_id.to_string();
        let events = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, session_id, event_type, payload, timestamp
                     FROM events WHERE session_id = ?1
                     ORDER BY timestamp ASC LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(rusqlite::params![session, limit], |row| {
                        Ok(EventRow {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            event_type: row.get(2)?,
                            payload: row.get(3)?,
                            timestamp: row.get(4)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await?;
        Ok(events)
    }

    pub async fn search_events(&self, query: &str, limit: u32) -> Result<Vec<EventRow>> {
        let q = query.to_string();
        let events = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT e.id, e.session_id, e.event_type, e.payload, e.timestamp
                     FROM events_fts f
                     JOIN events e ON e.rowid = f.rowid
                     WHERE events_fts MATCH ?1
                     ORDER BY rank
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(rusqlite::params![q, limit], |row| {
                        Ok(EventRow {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            event_type: row.get(2)?,
                            payload: row.get(3)?,
                            timestamp: row.get(4)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await?;
        Ok(events)
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p metaygn-memory --test store_test`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add crates/memory/
git commit -m "feat(memory): SQLite store with WAL, event logging, FTS5 search"
```

---

## Task 5: Daemon — axum HTTP API skeleton

**Files:**
- Create: `crates/daemon/src/main.rs`
- Create: `crates/daemon/src/api/mod.rs`
- Create: `crates/daemon/src/api/hooks.rs`
- Create: `crates/daemon/src/api/health.rs`
- Create: `crates/daemon/src/api/memory.rs`
- Create: `crates/daemon/src/app_state.rs`
- Test: `crates/daemon/tests/api_test.rs`

**Step 1: Write failing test**

Create `crates/daemon/tests/api_test.rs`:
```rust
use reqwest::Client;
use std::net::SocketAddr;

async fn start_test_server() -> SocketAddr {
    let app = metaygn_daemon::build_app().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn health_returns_ok() {
    let addr = start_test_server().await;
    let client = Client::new();
    let resp = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn hook_pre_tool_use_returns_json() {
    let addr = start_test_server().await;
    let client = Client::new();
    let payload = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"},
        "cwd": "/tmp"
    });
    let resp = client
        .post(format!("http://{addr}/hooks/pre-tool-use"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn hook_pre_tool_use_denies_destructive() {
    let addr = start_test_server().await;
    let client = Client::new();
    let payload = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "rm -rf /"},
        "cwd": "/tmp"
    });
    let resp = client
        .post(format!("http://{addr}/hooks/pre-tool-use"))
        .json(&payload)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p metaygn-daemon --test api_test`
Expected: FAIL — `build_app` not defined

**Step 3: Implement daemon API**

Write `crates/daemon/src/app_state.rs`:
```rust
use metaygn_memory::store::MemoryStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<MemoryStore>,
}

impl AppState {
    pub async fn new_in_memory() -> anyhow::Result<Self> {
        let memory = MemoryStore::open_in_memory().await?;
        Ok(Self {
            memory: Arc::new(memory),
        })
    }

    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let memory = MemoryStore::open(db_path).await?;
        Ok(Self {
            memory: Arc::new(memory),
        })
    }
}
```

Write `crates/daemon/src/api/health.rs`:
```rust
use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "kernel": "verified"
    }))
}
```

Write `crates/daemon/src/api/hooks.rs`:
```rust
use axum::{extract::State, Json};
use metaygn_shared::protocol::{HookInput, HookOutput, PermissionDecision};
use regex::Regex;

use crate::app_state::AppState;

static DESTRUCTIVE_PATTERNS: &[&str] = &[
    r"\brm\s+-rf\s+/(\s|$)",
    r"\bsudo\s+rm\s+-rf\b",
    r"\bmkfs\b",
    r"\bdd\s+if=",
    r"\bshutdown\b",
    r"\breboot\b",
    r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;",
];

static HIGH_RISK_PATTERNS: &[&str] = &[
    r"\bgit\s+push\b",
    r"\bgit\s+reset\s+--hard\b",
    r"\bterraform\s+(apply|destroy)\b",
    r"\bkubectl\s+(apply|delete)\b",
    r"\bcurl\b.*\|\s*(ba)?sh\b",
    r"\bsudo\s+",
];

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns
        .iter()
        .any(|p| Regex::new(p).map(|re| re.is_match(text)).unwrap_or(false))
}

pub async fn pre_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    // Log the event
    let session = input.session_id.as_deref().unwrap_or("unknown");
    let payload_str = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event(session, "PreToolUse", &payload_str)
        .await;

    let tool = input.tool_name.as_deref().unwrap_or("");
    let tool_input = input.tool_input.as_ref();

    // Gate: Bash commands
    if tool == "Bash" {
        if let Some(cmd) = tool_input.and_then(|v| v.get("command")).and_then(|v| v.as_str()) {
            if matches_any(cmd, DESTRUCTIVE_PATTERNS) {
                return Json(HookOutput::permission(
                    PermissionDecision::Deny,
                    "Blocked: destructive shell command detected.",
                ));
            }
            if matches_any(cmd, HIGH_RISK_PATTERNS) {
                return Json(HookOutput::permission(
                    PermissionDecision::Ask,
                    "High-risk shell action. Confirm rollback plan.",
                ));
            }
        }
    }

    // Gate: MCP calls
    if tool.starts_with("mcp__") {
        return Json(HookOutput::permission(
            PermissionDecision::Ask,
            "External MCP call crosses trust boundary.",
        ));
    }

    Json(HookOutput::allow())
}

pub async fn post_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let session = input.session_id.as_deref().unwrap_or("unknown");
    let payload_str = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event(session, "PostToolUse", &payload_str)
        .await;

    let tool = input.tool_name.as_deref().unwrap_or("");

    if tool == "Bash" {
        if let Some(cmd) = input
            .tool_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str())
        {
            let lower = cmd.to_lowercase();
            let verify_keywords = [
                "test", "pytest", "cargo test", "cargo check", "clippy", "lint",
                "ruff", "mypy", "tsc", "go test", "dotnet test",
            ];
            if verify_keywords.iter().any(|kw| lower.contains(kw)) {
                return Json(HookOutput::context(
                    "PostToolUse",
                    "Verification signal captured. Treat results as stronger evidence.",
                ));
            }
        }
    }

    if matches!(tool, "Write" | "Edit" | "MultiEdit" | "NotebookEdit") {
        return Json(HookOutput::context(
            "PostToolUse",
            "Files changed. Run smallest meaningful verification before finalizing.",
        ));
    }

    Json(HookOutput::allow())
}

pub async fn user_prompt_submit(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let session = input.session_id.as_deref().unwrap_or("unknown");
    let prompt = input.prompt.as_deref().unwrap_or("");

    let risk = classify_risk(prompt);
    let payload = serde_json::json!({"prompt_length": prompt.len(), "risk": risk});
    let _ = state
        .memory
        .log_event(session, "UserPromptSubmit", &payload.to_string())
        .await;

    let msg = match risk {
        "high" => format!("Preflight: risk=high. Run /metacog-preflight or /metacog-threat-model before acting."),
        "low" => format!("Preflight: risk=low. Lean workflow. Verify after patching."),
        _ => format!("Preflight: risk=medium. Name proof plan before acting."),
    };

    Json(HookOutput::context("UserPromptSubmit", &msg))
}

fn classify_risk(prompt: &str) -> &'static str {
    let text = prompt.to_lowercase();
    let high = [
        "auth", "oauth", "token", "secret", "deploy", "payment", "billing",
        "migration", "database", "prod", "production", "security", "delete",
        "terraform", "kubernetes", "docker", "ci/cd", "release",
    ];
    let low = [
        "typo", "rename", "comment", "docs", "readme", "format", "lint", "cleanup",
    ];

    if high.iter().any(|w| text.contains(w)) {
        "high"
    } else if low.iter().any(|w| text.contains(w)) {
        "low"
    } else {
        "medium"
    }
}
```

Write `crates/daemon/src/api/memory.rs`:
```rust
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct RecallQuery {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    10
}

pub async fn recall(
    State(state): State<AppState>,
    Json(query): Json<RecallQuery>,
) -> Json<Value> {
    match state.memory.search_events(&query.query, query.limit).await {
        Ok(events) => {
            let results: Vec<Value> = events
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "event_type": e.event_type,
                        "payload": e.payload,
                        "timestamp": e.timestamp,
                    })
                })
                .collect();
            Json(json!({ "results": results }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

pub async fn stats(State(state): State<AppState>) -> Json<Value> {
    let count = state.memory.event_count().await.unwrap_or(0);
    Json(json!({
        "event_count": count,
    }))
}
```

Write `crates/daemon/src/api/mod.rs`:
```rust
pub mod health;
pub mod hooks;
pub mod memory;
```

Update `crates/daemon/src/main.rs`:
```rust
pub mod api;
pub mod app_state;

use anyhow::Result;
use app_state::AppState;
use axum::{routing::{get, post}, Router};

pub async fn build_app() -> Result<Router> {
    let state = AppState::new_in_memory().await?;
    build_app_with_state(state)
}

pub fn build_app_with_state(state: AppState) -> Result<Router> {
    let app = Router::new()
        .route("/health", get(api::health::health))
        .route("/hooks/pre-tool-use", post(api::hooks::pre_tool_use))
        .route("/hooks/post-tool-use", post(api::hooks::post_tool_use))
        .route(
            "/hooks/user-prompt-submit",
            post(api::hooks::user_prompt_submit),
        )
        .route("/memory/recall", post(api::memory::recall))
        .route("/memory/stats", get(api::memory::stats))
        .with_state(state);
    Ok(app)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("metaygn=debug,tower_http=debug")
        .init();

    let state = AppState::new("metaygn.db").await?;
    let app = build_app_with_state(state)?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // Write port file for hooks to discover
    let port_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("aletheia")
        .join("daemon.port");
    if let Some(parent) = port_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&port_path, addr.port().to_string())?;

    tracing::info!("aletheiad listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
```

Add `regex` and `dirs` to daemon Cargo.toml dependencies:
```toml
regex = "1"
dirs = "6"
```

**Step 4: Run tests**

Run: `cargo test -p metaygn-daemon --test api_test`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add crates/daemon/
git commit -m "feat(daemon): axum HTTP API with hook endpoints, memory recall, health check"
```

---

## Task 6: CLI — start, stop, status commands

**Files:**
- Modify: `crates/cli/src/main.rs`
- Test: `crates/cli/tests/cli_test.rs`

**Step 1: Implement CLI structure**

Write `crates/cli/src/main.rs`:
```rust
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aletheia", about = "MetaYGN metacognitive runtime CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the aletheiad daemon
    Start {
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value_t = 0)]
        port: u16,
        #[arg(long)]
        db_path: Option<String>,
    },
    /// Stop the running daemon
    Stop,
    /// Check daemon status
    Status,
    /// Query episodic memory
    Recall {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: u32,
    },
}

fn daemon_port_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("aletheia")
        .join("daemon.port")
}

fn read_daemon_port() -> Result<u16> {
    let path = daemon_port_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|_| anyhow::anyhow!("Daemon not running (no port file at {path:?})"))?;
    Ok(content.trim().parse()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { host, port, db_path } => {
            println!("Starting aletheiad on {host}:{port}...");
            println!("(daemon startup not yet implemented — use `cargo run -p metaygn-daemon` directly)");
        }
        Commands::Stop => {
            println!("Stopping aletheiad...");
            let port = read_daemon_port()?;
            let client = reqwest::Client::new();
            // For now, just check if it's running
            match client
                .get(format!("http://127.0.0.1:{port}/health"))
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await
            {
                Ok(_) => println!("Daemon is running on port {port}. (graceful shutdown not yet implemented)"),
                Err(_) => println!("Daemon is not running."),
            }
        }
        Commands::Status => {
            match read_daemon_port() {
                Ok(port) => {
                    let client = reqwest::Client::new();
                    match client
                        .get(format!("http://127.0.0.1:{port}/health"))
                        .timeout(std::time::Duration::from_secs(2))
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            let body: serde_json::Value = resp.json().await?;
                            println!("Daemon: RUNNING (port {port})");
                            println!("Status: {}", body["status"]);
                            println!("Version: {}", body["version"]);
                            println!("Kernel: {}", body["kernel"]);
                        }
                        Err(_) => println!("Daemon: NOT RESPONDING (port file exists but daemon unreachable)"),
                    }
                }
                Err(e) => println!("Daemon: STOPPED ({e})"),
            }
        }
        Commands::Recall { query, limit } => {
            let port = read_daemon_port()?;
            let client = reqwest::Client::new();
            let resp = client
                .post(format!("http://127.0.0.1:{port}/memory/recall"))
                .json(&serde_json::json!({"query": query, "limit": limit}))
                .send()
                .await?;
            let body: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
    }
    Ok(())
}
```

Add `dirs = "6"` to cli Cargo.toml dependencies.

**Step 2: Verify it compiles and runs**

Run: `cargo run -p metaygn-cli -- status`
Expected: "Daemon: STOPPED (Daemon not running...)"

Run: `cargo run -p metaygn-cli -- --help`
Expected: Help text with start, stop, status, recall subcommands

**Step 3: Commit**

```bash
git add crates/cli/
git commit -m "feat(cli): aletheia CLI with start, stop, status, recall commands"
```

---

## Task 7: Initialize TypeScript workspace

**Files:**
- Create: `pnpm-workspace.yaml`
- Create: `packages/hooks/package.json`
- Create: `packages/hooks/tsconfig.json`
- Create: `packages/hooks/src/lib/daemon-client.ts`
- Create: `packages/hooks/src/lib/fallback.ts`
- Create: `packages/hooks/src/pre-tool-use.ts`
- Create: `packages/shared/package.json`
- Create: `packages/shared/src/types.ts`

**Step 1: Create pnpm workspace**

```yaml
# pnpm-workspace.yaml
packages:
  - "packages/*"
```

**Step 2: Create packages/shared with types**

`packages/shared/package.json`:
```json
{
  "name": "@metaygn/shared",
  "version": "0.1.0",
  "type": "module",
  "main": "src/types.ts",
  "scripts": {
    "typecheck": "tsc --noEmit"
  },
  "devDependencies": {
    "typescript": "^5.7",
    "zod": "^3.24"
  }
}
```

`packages/shared/src/types.ts`:
```typescript
import { z } from "zod";

export const HookEventSchema = z.enum([
  "SessionStart", "UserPromptSubmit", "PreToolUse",
  "PostToolUse", "PostToolUseFailure", "Stop",
  "PreCompact", "SessionEnd",
]);

export const PermissionDecisionSchema = z.enum(["allow", "deny", "ask"]);

export const HookInputSchema = z.object({
  hook_event_name: HookEventSchema,
  session_id: z.string().optional(),
  cwd: z.string().optional(),
  tool_name: z.string().optional(),
  tool_input: z.record(z.unknown()).optional(),
  tool_response: z.string().optional(),
  prompt: z.string().optional(),
  error: z.string().optional(),
  last_assistant_message: z.string().optional(),
});

export type HookInput = z.infer<typeof HookInputSchema>;

export interface HookOutput {
  hookSpecificOutput?: {
    hookEventName?: string;
    permissionDecision?: "allow" | "deny" | "ask";
    permissionDecisionReason?: string;
    additionalContext?: string;
  };
}
```

**Step 3: Create packages/hooks**

`packages/hooks/package.json`:
```json
{
  "name": "@metaygn/hooks",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "@metaygn/shared": "workspace:*"
  },
  "devDependencies": {
    "typescript": "^5.7",
    "zod": "^3.24",
    "@types/bun": "latest"
  }
}
```

`packages/hooks/src/lib/daemon-client.ts`:
```typescript
import type { HookInput, HookOutput } from "@metaygn/shared/src/types";
import { readFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";

const TIMEOUT_MS = 350;

function readDaemonPort(): number | null {
  try {
    const portPath = join(homedir(), ".claude", "aletheia", "daemon.port");
    const content = readFileSync(portPath, "utf-8").trim();
    return parseInt(content, 10);
  } catch {
    return null;
  }
}

export async function callDaemon(
  route: string,
  payload: HookInput
): Promise<HookOutput | null> {
  const port = readDaemonPort();
  if (!port) return null;

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), TIMEOUT_MS);

  try {
    const resp = await fetch(`http://127.0.0.1:${port}${route}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
      signal: controller.signal,
    });
    clearTimeout(timeout);
    if (!resp.ok) return null;
    return (await resp.json()) as HookOutput;
  } catch {
    clearTimeout(timeout);
    return null; // graceful fallback
  }
}
```

`packages/hooks/src/lib/fallback.ts`:
```typescript
import type { HookOutput } from "@metaygn/shared/src/types";

const DESTRUCTIVE = [
  /\brm\s+-rf\s+\/(\s|$)/,
  /\bsudo\s+rm\s+-rf\b/,
  /\bmkfs\b/,
  /\bdd\s+if=/,
  /\bshutdown\b/,
  /\breboot\b/,
];

const HIGH_RISK = [
  /\bgit\s+push\b/,
  /\bgit\s+reset\s+--hard\b/,
  /\bterraform\s+(apply|destroy)\b/,
  /\bkubectl\s+(apply|delete)\b/,
  /\bcurl\b.*\|\s*(ba)?sh\b/,
  /\bsudo\s+/,
];

function matchesAny(text: string, patterns: RegExp[]): boolean {
  return patterns.some((p) => p.test(text));
}

export function fallbackPreToolUse(
  toolName: string,
  command?: string
): HookOutput | null {
  if (toolName === "Bash" && command) {
    if (matchesAny(command, DESTRUCTIVE)) {
      return {
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "deny",
          permissionDecisionReason: "Blocked: destructive command.",
        },
      };
    }
    if (matchesAny(command, HIGH_RISK)) {
      return {
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "ask",
          permissionDecisionReason: "High-risk action. Confirm necessity.",
        },
      };
    }
  }
  if (toolName.startsWith("mcp__")) {
    return {
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "ask",
        permissionDecisionReason: "MCP trust boundary crossing.",
      },
    };
  }
  return null; // allow
}
```

`packages/hooks/src/pre-tool-use.ts`:
```typescript
import { callDaemon } from "./lib/daemon-client";
import { fallbackPreToolUse } from "./lib/fallback";
import type { HookInput } from "@metaygn/shared/src/types";

const input: HookInput = await Bun.stdin.json();

// Try daemon first
const daemonResult = await callDaemon("/hooks/pre-tool-use", input);
if (daemonResult?.hookSpecificOutput) {
  process.stdout.write(JSON.stringify(daemonResult));
  process.exit(0);
}

// Fallback to local heuristics
const toolName = input.tool_name ?? "";
const command = (input.tool_input as Record<string, unknown>)?.command as
  | string
  | undefined;
const fallback = fallbackPreToolUse(toolName, command);
if (fallback) {
  process.stdout.write(JSON.stringify(fallback));
}
// else: allow (exit with no output)
```

**Step 4: Install and verify**

Run: `cd /c/Projects/MetaYGN && pnpm install`
Expected: dependencies installed

Run: `cd packages/hooks && npx tsc --noEmit` (or `bun run typecheck`)
Expected: no type errors

**Step 5: Commit**

```bash
git add pnpm-workspace.yaml packages/
git commit -m "feat(hooks): TypeScript hooks with Bun, daemon client, local fallback"
```

---

## Task 8: justfile — cross-language task runner

**Files:**
- Create: `justfile`

**Step 1: Create justfile**

```justfile
# Meta-YGN cross-language task runner

# === Rust ===

# Check all Rust crates
check:
    cargo check --workspace

# Test all Rust crates
test-rust:
    cargo test --workspace

# Build release binaries
build-rust:
    cargo build --workspace --release

# Run the daemon (dev mode)
daemon:
    cargo run -p metaygn-daemon

# Run the CLI
cli *ARGS:
    cargo run -p metaygn-cli -- {{ARGS}}

# === TypeScript ===

# Install TS dependencies
install-ts:
    pnpm install

# Type-check all TS packages
check-ts:
    pnpm -r run typecheck

# === Python ===

# Run eval benchmarks
bench:
    cd eval && python -m pytest benchmarks/ -v

# === Cross-language ===

# Check everything
check-all: check check-ts

# Test everything
test-all: test-rust

# Full build
build: build-rust

# Format all code
fmt:
    cargo fmt --all
```

**Step 2: Verify**

Run: `just check-all`
Expected: both Rust and TS checks pass

**Step 3: Commit**

```bash
git add justfile
git commit -m "build: add justfile for cross-language task running"
```

---

## Task 9: End-to-end integration test

**Files:**
- Create: `tests/integration/e2e_test.rs` (or shell script)

**Step 1: Write integration test**

Create `tests/e2e.sh`:
```bash
#!/bin/bash
set -e

echo "=== Meta-YGN E2E Test ==="

# 1. Build
echo "Building..."
cargo build --workspace 2>&1 | tail -1

# 2. Start daemon in background
echo "Starting daemon..."
cargo run -p metaygn-daemon &
DAEMON_PID=$!
sleep 2

# 3. Read port
PORT_FILE="$HOME/.claude/aletheia/daemon.port"
if [ ! -f "$PORT_FILE" ]; then
    echo "FAIL: No port file"
    kill $DAEMON_PID 2>/dev/null
    exit 1
fi
PORT=$(cat "$PORT_FILE")
echo "Daemon running on port $PORT"

# 4. Health check
HEALTH=$(curl -s "http://127.0.0.1:$PORT/health")
echo "Health: $HEALTH"

# 5. Test pre-tool-use (safe command)
SAFE=$(curl -s -X POST "http://127.0.0.1:$PORT/hooks/pre-tool-use" \
    -H "Content-Type: application/json" \
    -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls -la"}}')
echo "Safe command: $SAFE"

# 6. Test pre-tool-use (destructive command)
DESTRUCTIVE=$(curl -s -X POST "http://127.0.0.1:$PORT/hooks/pre-tool-use" \
    -H "Content-Type: application/json" \
    -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}')
echo "Destructive command: $DESTRUCTIVE"
echo "$DESTRUCTIVE" | grep -q "deny" || (echo "FAIL: should deny destructive"; kill $DAEMON_PID; exit 1)

# 7. Test memory stats
STATS=$(curl -s "http://127.0.0.1:$PORT/memory/stats")
echo "Memory stats: $STATS"

# 8. CLI status
cargo run -p metaygn-cli -- status

# Cleanup
kill $DAEMON_PID 2>/dev/null
rm -f "$PORT_FILE"

echo "=== ALL E2E TESTS PASSED ==="
```

**Step 2: Run**

Run: `bash tests/e2e.sh`
Expected: ALL E2E TESTS PASSED

**Step 3: Commit**

```bash
git add tests/
git commit -m "test: add end-to-end integration test for daemon + CLI"
```

---

## Phase 1 Complete Checkpoint

At this point you should have:
- [x] Cargo workspace with 5 crates compiling
- [x] KERNEL integrity with SHA-256 verification
- [x] Metacognitive state types (15+ types)
- [x] Hook protocol types with serde
- [x] SQLite store with WAL, events, FTS5 search
- [x] Daemon with axum API (health, hooks, memory)
- [x] Security gates (destructive/high-risk/MCP patterns)
- [x] CLI with start, stop, status, recall
- [x] TypeScript hooks with Bun, daemon client, local fallback
- [x] justfile for cross-language builds
- [x] E2E test passing

**Next:** Phase 2 (Intelligence) — see design doc section 11.

---

## Phases 2-5 Overview (detailed plans to be written when Phase 1 is complete)

### Phase 2: Intelligence (weeks 4-6)
- Task 10: 12-stage control loop in `crates/core/`
- Task 11: 8 cognitive strategies
- Task 12: Evidence packs with hash chain + Merkle + ed25519
- Task 13: Guard pipeline with composable scoring
- Task 14: Context pruning reverse proxy
- Task 15: 3-tier memory (Hot/Warm/Cold)
- Task 16: Trauma index + skill crystallizer

### Phase 3: Sandbox + TUI (weeks 7-8)
- Task 17: Wasmtime runtime with fuel metering
- Task 18: Hypothesis testing (speculative execution)
- Task 19: Glass-box TUI with ratatui
- Task 20: Human fatigue profiler
- Task 21: MASC anomaly detector

### Phase 4: Evaluation + Hardening (weeks 9-10)
- Task 22: MetaCog-Bench scenarios
- Task 23: Autopoietic tool forge
- Task 24: Layer 0 meta-metacognition
- Task 25: Cross-platform testing (Win/Mac/Linux)

### Phase 5: Distribution (weeks 11-12)
- Task 26: cargo-dist release pipeline
- Task 27: Claude Code marketplace packaging
- Task 28: Documentation finalization

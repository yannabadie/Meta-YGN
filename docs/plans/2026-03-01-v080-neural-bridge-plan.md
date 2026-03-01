# v0.8.0 "Neural Bridge" Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect MetaYGN to the MCP ecosystem via a stdio bridge, enable real neural embeddings for graph memory, and add session replay for post-hoc debugging.

**Architecture:** Three independent features built as incremental additions. The MCP bridge is a new crate (`mcp-bridge`) that wraps the existing daemon HTTP API behind 5 MCP tools using rmcp 0.17 stdio transport. Neural embeddings add a `FastEmbedProvider` behind the `embeddings` cargo feature implementing the existing `EmbeddingProvider` trait. Session replay records all hook calls into SQLite and exposes them via daemon API + CLI.

**Tech Stack:** Rust 2024, rmcp 0.17 (MCP SDK), fastembed 4.x (ONNX embeddings), tokio-rusqlite, axum, clap, ratatui.

---

## Task 1: MCP Bridge — Crate Skeleton

**Files:**
- Create: `crates/mcp-bridge/Cargo.toml`
- Create: `crates/mcp-bridge/src/lib.rs`
- Create: `crates/mcp-bridge/src/main.rs`
- Modify: `Cargo.toml:3` (workspace members)

**Step 1: Add crate to workspace**

In `Cargo.toml`, add `"crates/mcp-bridge"` to the `members` array:

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/core",
    "crates/memory",
    "crates/daemon",
    "crates/cli",
    "crates/verifiers",
    "crates/sandbox",
    "crates/mcp-bridge",
]
```

**Step 2: Create Cargo.toml for mcp-bridge**

```toml
[package]
name = "metaygn-mcp-bridge"
edition.workspace = true
version.workspace = true
license.workspace = true

[[bin]]
name = "aletheia-mcp"
path = "src/main.rs"

[dependencies]
rmcp = { version = "0.17", features = ["server", "transport-io"] }
reqwest = { version = "0.12", features = ["json"] }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
dirs = "6"
```

**Step 3: Create lib.rs with daemon client helper**

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Read the daemon port from ~/.claude/aletheia/daemon.port.
pub fn read_daemon_port() -> Result<u16> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let port_file = home.join(".claude").join("aletheia").join("daemon.port");
    let contents = std::fs::read_to_string(&port_file)
        .with_context(|| format!("daemon port file not found at {}", port_file.display()))?;
    contents
        .trim()
        .parse::<u16>()
        .context("invalid port in daemon.port file")
}

/// HTTP client configured for daemon communication.
pub struct DaemonClient {
    client: reqwest::Client,
    base_url: String,
}

impl DaemonClient {
    pub fn new(port: u16) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        Ok(Self {
            client,
            base_url: format!("http://127.0.0.1:{port}"),
        })
    }

    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(format!("{}{path}", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
```

**Step 4: Create main.rs placeholder**

```rust
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("aletheia-mcp starting...");
    // MCP server will be added in Task 2
    Ok(())
}
```

**Step 5: Verify it compiles**

Run: `cargo build -p metaygn-mcp-bridge`
Expected: Compiles successfully.

**Step 6: Commit**

```bash
git add crates/mcp-bridge/ Cargo.toml Cargo.lock
git commit -m "feat(mcp-bridge): scaffold crate with daemon client helper"
```

---

## Task 2: MCP Bridge — Server Handler & 5 Tools

**Files:**
- Create: `crates/mcp-bridge/src/handler.rs`
- Modify: `crates/mcp-bridge/src/lib.rs`
- Modify: `crates/mcp-bridge/src/main.rs`

**Step 1: Create handler.rs with 5 MCP tools**

```rust
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::schemars;
use rmcp::tool;
use rmcp::{ServerHandler, tool_handler, tool_router};
use serde::Deserialize;

use crate::DaemonClient;

pub struct AletheiaHandler {
    daemon: DaemonClient,
}

impl AletheiaHandler {
    pub fn new(daemon: DaemonClient) -> Self {
        Self { daemon }
    }
}

// -- Tool input schemas -------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClassifyInput {
    /// The user prompt or task description to classify
    pub prompt: String,
    /// Optional tool name if this is a tool-use context
    pub tool_name: Option<String>,
    /// Optional tool input for risk assessment
    pub tool_input: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VerifyInput {
    /// The tool name that was executed
    pub tool_name: String,
    /// The tool output to verify
    pub tool_output: String,
    /// Expected outcome description (optional)
    pub expected: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecallInput {
    /// Search query for memory retrieval
    pub query: String,
    /// Maximum number of results (default: 10)
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PruneInput {
    /// JSON array of message objects to analyze for error loops
    pub messages: String,
}

// -- Tool implementations -----------------------------------------------------

#[tool_router]
impl AletheiaHandler {
    /// Classify a task or tool invocation by risk level, difficulty, strategy,
    /// and recommended topology. Returns the full metacognitive assessment.
    #[tool(description = "Classify a prompt/tool by risk, difficulty, and strategy")]
    async fn metacog_classify(
        &self,
        #[tool(aggr)] input: ClassifyInput,
    ) -> Result<CallToolResult, rmcp::Error> {
        let body = serde_json::json!({
            "hook_event": {
                "event": "UserPromptSubmit",
                "user_prompt": input.prompt,
                "tool_name": input.tool_name,
                "tool_input": input.tool_input.unwrap_or_default(),
            }
        });
        match self.daemon.post("/hooks/user-prompt-submit", &body).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Daemon error: {e}"
            ))])),
        }
    }

    /// Verify a tool output against expectations. Runs the guard pipeline
    /// and metacognitive verification stages.
    #[tool(description = "Verify a tool output for errors, risks, or anomalies")]
    async fn metacog_verify(
        &self,
        #[tool(aggr)] input: VerifyInput,
    ) -> Result<CallToolResult, rmcp::Error> {
        let body = serde_json::json!({
            "hook_event": {
                "event": "PostToolUse",
                "tool_name": input.tool_name,
                "tool_output": input.tool_output,
            }
        });
        match self.daemon.post("/hooks/post-tool-use", &body).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Daemon error: {e}"
            ))])),
        }
    }

    /// Search episodic memory and graph nodes using full-text search.
    /// Returns matching events and graph nodes ranked by relevance.
    #[tool(description = "Recall memories from episodic and graph memory via FTS")]
    async fn metacog_recall(
        &self,
        #[tool(aggr)] input: RecallInput,
    ) -> Result<CallToolResult, rmcp::Error> {
        let body = serde_json::json!({
            "query": input.query,
            "limit": input.limit.unwrap_or(10),
        });
        match self.daemon.post("/memory/recall", &body).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Daemon error: {e}"
            ))])),
        }
    }

    /// Get current daemon status: health, memory stats, fatigue level,
    /// budget consumption, and heuristic population fitness.
    #[tool(description = "Get daemon health, memory stats, fatigue, and budget status")]
    async fn metacog_status(&self) -> Result<CallToolResult, rmcp::Error> {
        let mut status = serde_json::Map::new();

        if let Ok(health) = self.daemon.get("/health").await {
            status.insert("health".into(), health);
        }
        if let Ok(fatigue) = self.daemon.get("/profiler/fatigue").await {
            status.insert("fatigue".into(), fatigue);
        }
        if let Ok(budget) = self.daemon.get("/budget").await {
            status.insert("budget".into(), budget);
        }
        if let Ok(best) = self.daemon.get("/heuristics/best").await {
            status.insert("heuristics_best".into(), best);
        }

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&serde_json::Value::Object(status))
                .unwrap_or_default(),
        )]))
    }

    /// Analyze a message array for reasoning lock-in (3+ consecutive errors)
    /// and return a pruned version with recovery injection.
    #[tool(description = "Prune error loops from a message array and inject recovery")]
    async fn metacog_prune(
        &self,
        #[tool(aggr)] input: PruneInput,
    ) -> Result<CallToolResult, rmcp::Error> {
        let messages: serde_json::Value = serde_json::from_str(&input.messages)
            .unwrap_or(serde_json::Value::Array(vec![]));
        let body = serde_json::json!({ "messages": messages });
        match self.daemon.post("/proxy/anthropic", &body).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Daemon error: {e}"
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for AletheiaHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Aletheia metacognitive runtime — classify risk, verify tool output, \
                 recall memories, check status, and prune error loops."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
```

**Step 2: Update lib.rs to export handler**

```rust
use anyhow::{Context, Result};

pub mod handler;

/// Read the daemon port from ~/.claude/aletheia/daemon.port.
pub fn read_daemon_port() -> Result<u16> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let port_file = home.join(".claude").join("aletheia").join("daemon.port");
    let contents = std::fs::read_to_string(&port_file)
        .with_context(|| format!("daemon port file not found at {}", port_file.display()))?;
    contents
        .trim()
        .parse::<u16>()
        .context("invalid port in daemon.port file")
}

/// HTTP client configured for daemon communication.
pub struct DaemonClient {
    client: reqwest::Client,
    base_url: String,
}

impl DaemonClient {
    pub fn new(port: u16) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        Ok(Self {
            client,
            base_url: format!("http://127.0.0.1:{port}"),
        })
    }

    pub async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(format!("{}{path}", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
```

**Step 3: Update main.rs to launch stdio MCP server**

```rust
use anyhow::Result;
use rmcp::transport::stdio;

use metaygn_mcp_bridge::{read_daemon_port, DaemonClient};
use metaygn_mcp_bridge::handler::AletheiaHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // MCP stdio transport uses stdin/stdout, so logs go to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let port = read_daemon_port()?;
    tracing::info!("Connecting to daemon on port {port}");

    let daemon = DaemonClient::new(port)?;
    let handler = AletheiaHandler::new(daemon);

    tracing::info!("Starting MCP stdio server...");
    let service = handler.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
```

**Step 4: Verify it compiles**

Run: `cargo build -p metaygn-mcp-bridge`
Expected: Compiles successfully. (Note: rmcp macros may require adjustments — adapt imports based on actual rmcp 0.17 API if the derive macro paths differ.)

**Step 5: Commit**

```bash
git add crates/mcp-bridge/src/
git commit -m "feat(mcp-bridge): 5 metacognitive MCP tools via stdio transport"
```

---

## Task 3: MCP Bridge — CLI Command & Integration

**Files:**
- Modify: `crates/cli/Cargo.toml`
- Modify: `crates/cli/src/main.rs`
- Create: `crates/mcp-bridge/tests/handler_test.rs`

**Step 1: Add `aletheia mcp` command to CLI**

In `crates/cli/src/main.rs`, add the `Mcp` variant to the `Commands` enum:

```rust
    /// Launch MCP stdio server (for Claude Code / MCP clients)
    Mcp,
```

And add the match arm in `main()`:

```rust
        Commands::Mcp => cmd_mcp().await,
```

And add the command implementation:

```rust
/// Mcp command: launch the MCP stdio bridge.
async fn cmd_mcp() -> Result<()> {
    // Find aletheia-mcp binary next to this executable
    let exe = std::env::current_exe().context("could not determine own executable path")?;
    let exe_dir = exe.parent().context("executable has no parent directory")?;
    let mcp_name = if cfg!(windows) {
        "aletheia-mcp.exe"
    } else {
        "aletheia-mcp"
    };
    let mcp_path = exe_dir.join(mcp_name);

    if !mcp_path.exists() {
        anyhow::bail!(
            "Cannot find aletheia-mcp at {:?}. Build with: cargo build --workspace",
            mcp_path
        );
    }

    // Replace current process with aletheia-mcp (inherit stdio for MCP transport)
    let status = std::process::Command::new(&mcp_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to launch aletheia-mcp")?;

    std::process::exit(status.code().unwrap_or(1));
}
```

**Step 2: Write integration test for DaemonClient**

Create `crates/mcp-bridge/tests/handler_test.rs`:

```rust
//! Basic unit test: verify DaemonClient builds without panicking.
//! Full integration testing requires a running daemon.

use metaygn_mcp_bridge::DaemonClient;

#[test]
fn daemon_client_constructs() {
    let client = DaemonClient::new(12345).unwrap();
    // Just verify it creates without panic — no daemon running
    assert!(true);
}
```

**Step 3: Run the test**

Run: `cargo test -p metaygn-mcp-bridge`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/cli/src/main.rs crates/mcp-bridge/tests/
git commit -m "feat(cli): add 'aletheia mcp' command to launch MCP stdio bridge"
```

---

## Task 4: Neural Embeddings — FastEmbedProvider

**Files:**
- Modify: `crates/memory/Cargo.toml`
- Create: `crates/memory/src/fastembed_provider.rs`
- Modify: `crates/memory/src/lib.rs`
- Create: `crates/memory/tests/fastembed_test.rs`

**Step 1: Add fastembed dependency behind feature gate**

In `crates/memory/Cargo.toml`, add:

```toml
[features]
default = []
embeddings = ["fastembed"]

[dependencies]
# ... existing deps ...
fastembed = { version = "4", optional = true }
```

**Step 2: Create fastembed_provider.rs**

```rust
//! Neural embedding provider using fastembed (ONNX Runtime, bge-small-en-v1.5).
//! Only compiled when the `embeddings` cargo feature is enabled.

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::embeddings::EmbeddingProvider;

/// Neural embedding provider backed by fastembed.
/// Uses bge-small-en-v1.5 (384 dimensions) by default.
pub struct FastEmbedProvider {
    model: TextEmbedding,
    dimension: usize,
}

impl FastEmbedProvider {
    /// Initialize with the default model (bge-small-en-v1.5, 384 dim).
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::BGESmallENV15,
            show_download_progress: false,
            ..Default::default()
        })?;
        Ok(Self {
            model,
            dimension: 384,
        })
    }

    /// Initialize with a specific model and dimension.
    pub fn with_model(model_name: EmbeddingModel, dimension: usize) -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name,
            show_download_progress: false,
            ..Default::default()
        })?;
        Ok(Self { model, dimension })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.model.embed(vec![text], None)?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("fastembed returned no embeddings"))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let owned: Vec<String> = texts.iter().map(|t| t.to_string()).collect();
        let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        Ok(self.model.embed(refs, None)?)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }
}
```

**Step 3: Update lib.rs to conditionally export**

In `crates/memory/src/lib.rs`, add:

```rust
#[cfg(feature = "embeddings")]
pub mod fastembed_provider;
```

**Step 4: Write test (feature-gated)**

Create `crates/memory/tests/fastembed_test.rs`:

```rust
//! Integration test for FastEmbedProvider.
//! Only runs with: cargo test -p metaygn-memory --features embeddings

#![cfg(feature = "embeddings")]

use metaygn_memory::embeddings::EmbeddingProvider;
use metaygn_memory::fastembed_provider::FastEmbedProvider;

#[test]
fn fastembed_provider_basics() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");

    assert_eq!(provider.dimension(), 384);
    assert_eq!(provider.provider_name(), "fastembed");

    let vec = provider.embed("hello world").expect("embed failed");
    assert_eq!(vec.len(), 384);

    // Non-zero check
    let magnitude: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    assert!(magnitude > 0.0, "embedding should be non-zero");
}

#[test]
fn fastembed_batch_embedding() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");

    let texts = vec!["rust programming", "python scripting", "database queries"];
    let vecs = provider.embed_batch(&texts).expect("batch embed failed");

    assert_eq!(vecs.len(), 3);
    for v in &vecs {
        assert_eq!(v.len(), 384);
    }
}

#[test]
fn fastembed_similar_texts_have_higher_cosine() {
    let provider = FastEmbedProvider::new().expect("failed to init fastembed");

    let v1 = provider.embed("rust programming language").unwrap();
    let v2 = provider.embed("rust systems programming").unwrap();
    let v3 = provider.embed("chocolate cake recipe").unwrap();

    let sim_related = metaygn_memory::graph::cosine_similarity(&v1, &v2);
    let sim_unrelated = metaygn_memory::graph::cosine_similarity(&v1, &v3);

    assert!(
        sim_related > sim_unrelated,
        "related texts ({sim_related}) should have higher cosine than unrelated ({sim_unrelated})"
    );
}
```

**Step 5: Run test WITHOUT feature (should compile with no fastembed tests)**

Run: `cargo test -p metaygn-memory`
Expected: PASS (fastembed tests skipped)

**Step 6: Run test WITH feature (downloads model on first run — may take time)**

Run: `cargo test -p metaygn-memory --features embeddings`
Expected: PASS (all 3 fastembed tests pass; first run downloads ~30MB model)

**Step 7: Commit**

```bash
git add crates/memory/Cargo.toml crates/memory/src/fastembed_provider.rs crates/memory/src/lib.rs crates/memory/tests/fastembed_test.rs
git commit -m "feat(memory): fastembed neural embedding provider (feature-gated)"
```

---

## Task 5: Neural Embeddings — GraphMemory Integration

**Files:**
- Modify: `crates/memory/src/graph.rs`
- Create: `crates/memory/tests/graph_embed_test.rs`

**Step 1: Add `semantic_search` method to GraphMemory**

At the end of the `impl GraphMemory` block in `crates/memory/src/graph.rs`, add a new method. This works with any `EmbeddingProvider` — callers generate the query embedding and pass it in:

```rust
    /// Semantic search: find the top-N nodes whose stored embedding is most
    /// similar to the given query embedding (cosine similarity).
    /// Only considers nodes that have a non-empty embedding.
    pub async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<(MemoryNode, f32)>> {
        let query_emb = query_embedding.to_vec();
        let results = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count
                     FROM nodes WHERE embedding IS NOT NULL",
                )?;
                let mut scored: Vec<(MemoryNode, f32)> = stmt
                    .query_map([], |row| Ok(row_to_node(row)))?
                    .filter_map(|r| r.ok())
                    .filter_map(|node| {
                        node.embedding.as_ref().and_then(|emb| {
                            if emb.is_empty() {
                                None
                            } else {
                                let score = cosine_similarity(&query_emb, emb);
                                Some((node.clone(), score))
                            }
                        })
                    })
                    .collect();
                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored.truncate(limit as usize);
                Ok::<_, rusqlite::Error>(scored)
            })
            .await?;
        Ok(results)
    }
```

Note: The `clone()` on the node inside the filter_map will require MemoryNode to be cloneable — it already derives Clone.

**Step 2: Write test**

Create `crates/memory/tests/graph_embed_test.rs`:

```rust
use metaygn_memory::embeddings::{EmbeddingProvider, HashEmbedProvider};
use metaygn_memory::graph::{GraphMemory, MemoryNode, NodeType, Scope};

#[tokio::test]
async fn semantic_search_returns_most_similar() {
    let graph = GraphMemory::open_in_memory().await.unwrap();
    let provider = HashEmbedProvider::new(64);

    // Insert 3 nodes with embeddings
    let texts = vec![
        ("n1", "rust programming language"),
        ("n2", "rust systems programming"),
        ("n3", "chocolate cake recipe"),
    ];

    for (id, text) in &texts {
        let embedding = provider.embed(text).unwrap();
        let node = MemoryNode {
            id: id.to_string(),
            node_type: NodeType::Lesson,
            scope: Scope::Project,
            label: text.to_string(),
            content: text.to_string(),
            embedding: Some(embedding),
            created_at: "2026-03-01T00:00:00Z".into(),
            access_count: 0,
        };
        graph.insert_node(&node).await.unwrap();
    }

    // Search for "rust code" — should rank rust nodes higher
    let query_emb = provider.embed("rust code").unwrap();
    let results = graph.semantic_search(&query_emb, 3).await.unwrap();

    assert_eq!(results.len(), 3);
    // The first two results should be the rust-related nodes
    let top_ids: Vec<&str> = results.iter().map(|(n, _)| n.id.as_str()).collect();
    assert!(
        top_ids[0] == "n1" || top_ids[0] == "n2",
        "top result should be a rust node, got {}",
        top_ids[0]
    );
}

#[tokio::test]
async fn semantic_search_skips_nodes_without_embedding() {
    let graph = GraphMemory::open_in_memory().await.unwrap();
    let provider = HashEmbedProvider::new(64);

    // Node with embedding
    let emb = provider.embed("test content").unwrap();
    let node_with = MemoryNode {
        id: "with_emb".into(),
        node_type: NodeType::Lesson,
        scope: Scope::Session,
        label: "with".into(),
        content: "test content".into(),
        embedding: Some(emb),
        created_at: "2026-03-01T00:00:00Z".into(),
        access_count: 0,
    };
    graph.insert_node(&node_with).await.unwrap();

    // Node without embedding
    let node_without = MemoryNode {
        id: "no_emb".into(),
        node_type: NodeType::Lesson,
        scope: Scope::Session,
        label: "without".into(),
        content: "other content".into(),
        embedding: None,
        created_at: "2026-03-01T00:00:00Z".into(),
        access_count: 0,
    };
    graph.insert_node(&node_without).await.unwrap();

    let query_emb = provider.embed("test").unwrap();
    let results = graph.semantic_search(&query_emb, 10).await.unwrap();

    assert_eq!(results.len(), 1, "only nodes with embeddings should be returned");
    assert_eq!(results[0].0.id, "with_emb");
}
```

**Step 3: Run tests**

Run: `cargo test -p metaygn-memory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/memory/src/graph.rs crates/memory/tests/graph_embed_test.rs
git commit -m "feat(memory): semantic_search on GraphMemory using cosine similarity"
```

---

## Task 6: Neural Embeddings — Daemon API Endpoint

**Files:**
- Create: `crates/daemon/src/api/semantic.rs`
- Modify: `crates/daemon/src/api/mod.rs`

**Step 1: Create semantic.rs endpoint**

```rust
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use metaygn_memory::embeddings::EmbeddingProvider;
use metaygn_memory::embeddings::HashEmbedProvider;

#[derive(Deserialize)]
pub struct SemanticSearchRequest {
    query: String,
    limit: Option<u32>,
}

#[derive(Serialize)]
pub struct SemanticSearchResponse {
    results: Vec<SemanticResult>,
    provider: String,
}

#[derive(Serialize)]
pub struct SemanticResult {
    id: String,
    label: String,
    content: String,
    score: f32,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/memory/semantic", post(semantic_search))
}

async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Json<serde_json::Value> {
    let limit = req.limit.unwrap_or(10);

    // Use hash-based embeddings for now; swap to fastembed when feature is enabled
    let provider = HashEmbedProvider::new(64);
    let query_emb = match provider.embed(&req.query) {
        Ok(emb) => emb,
        Err(e) => {
            return Json(serde_json::json!({
                "error": format!("embedding failed: {e}"),
            }));
        }
    };

    match state.graph.semantic_search(&query_emb, limit).await {
        Ok(results) => {
            let items: Vec<SemanticResult> = results
                .into_iter()
                .map(|(node, score)| SemanticResult {
                    id: node.id,
                    label: node.label,
                    content: node.content,
                    score,
                })
                .collect();
            Json(serde_json::json!(SemanticSearchResponse {
                results: items,
                provider: provider.provider_name().to_string(),
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": format!("semantic search failed: {e}"),
        })),
    }
}
```

**Step 2: Wire into router**

In `crates/daemon/src/api/mod.rs`, add `pub mod semantic;` and merge the route:

```rust
pub mod semantic;
```

And in the `router()` function, add:

```rust
        .merge(semantic::routes())
```

**Step 3: Run existing daemon tests to ensure nothing breaks**

Run: `cargo test -p metaygn-daemon`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/daemon/src/api/semantic.rs crates/daemon/src/api/mod.rs
git commit -m "feat(daemon): POST /memory/semantic endpoint for vector search"
```

---

## Task 7: Session Replay — Schema & Store

**Files:**
- Modify: `crates/memory/src/store.rs`
- Create: `crates/memory/tests/replay_test.rs`

**Step 1: Add replay table to MemoryStore schema**

In `crates/memory/src/store.rs`, within the `init_schema` method, add after the `session_outcomes` CREATE TABLE:

```sql
                    CREATE TABLE IF NOT EXISTS replay_events (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        session_id TEXT NOT NULL,
                        hook_event TEXT NOT NULL,
                        request_json TEXT NOT NULL,
                        response_json TEXT NOT NULL,
                        latency_ms INTEGER NOT NULL,
                        timestamp TEXT NOT NULL DEFAULT (datetime('now'))
                    );

                    CREATE INDEX IF NOT EXISTS idx_replay_session
                        ON replay_events(session_id, timestamp);
```

**Step 2: Add replay methods to MemoryStore**

After the existing methods in the `impl MemoryStore` block:

```rust
    /// Record a hook call for session replay.
    pub async fn record_replay_event(
        &self,
        session_id: &str,
        hook_event: &str,
        request_json: &str,
        response_json: &str,
        latency_ms: u64,
    ) -> Result<()> {
        let session_id = session_id.to_owned();
        let hook_event = hook_event.to_owned();
        let request_json = request_json.to_owned();
        let response_json = response_json.to_owned();
        let latency_ms = latency_ms as i64;

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO replay_events (session_id, hook_event, request_json, response_json, latency_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![session_id, hook_event, request_json, response_json, latency_ms],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// List all sessions that have replay events, with event counts.
    pub async fn replay_sessions(&self) -> Result<Vec<(String, u64, String, String)>> {
        let rows = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT session_id, COUNT(*) as event_count,
                            MIN(timestamp) as first_event, MAX(timestamp) as last_event
                     FROM replay_events
                     GROUP BY session_id
                     ORDER BY last_event DESC",
                )?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }

    /// Get all replay events for a session, in chronological order.
    pub async fn replay_events(
        &self,
        session_id: &str,
    ) -> Result<Vec<(i64, String, String, String, i64, String)>> {
        let session_id = session_id.to_owned();
        let rows = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, hook_event, request_json, response_json, latency_ms, timestamp
                     FROM replay_events
                     WHERE session_id = ?1
                     ORDER BY id ASC",
                )?;
                let rows = stmt
                    .query_map(params![session_id], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i64>(4)?,
                            row.get::<_, String>(5)?,
                        ))
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }
```

**Step 3: Write test**

Create `crates/memory/tests/replay_test.rs`:

```rust
use metaygn_memory::store::MemoryStore;

#[tokio::test]
async fn replay_record_and_retrieve() {
    let store = MemoryStore::open_in_memory().await.unwrap();

    // Record some replay events
    store
        .record_replay_event("sess-1", "PreToolUse", r#"{"tool":"Bash"}"#, r#"{"decision":"Allow"}"#, 12)
        .await
        .unwrap();
    store
        .record_replay_event("sess-1", "PostToolUse", r#"{"tool":"Bash"}"#, r#"{"ok":true}"#, 8)
        .await
        .unwrap();
    store
        .record_replay_event("sess-2", "UserPromptSubmit", r#"{"prompt":"fix bug"}"#, r#"{"risk":"Low"}"#, 5)
        .await
        .unwrap();

    // List sessions
    let sessions = store.replay_sessions().await.unwrap();
    assert_eq!(sessions.len(), 2);
    // sess-1 should have 2 events
    let sess1 = sessions.iter().find(|s| s.0 == "sess-1").unwrap();
    assert_eq!(sess1.1, 2);

    // Get events for sess-1
    let events = store.replay_events("sess-1").await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].1, "PreToolUse");
    assert_eq!(events[1].1, "PostToolUse");
    assert_eq!(events[0].4, 12); // latency_ms
}

#[tokio::test]
async fn replay_empty_session() {
    let store = MemoryStore::open_in_memory().await.unwrap();
    let events = store.replay_events("nonexistent").await.unwrap();
    assert!(events.is_empty());
}
```

**Step 4: Run tests**

Run: `cargo test -p metaygn-memory`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/memory/src/store.rs crates/memory/tests/replay_test.rs
git commit -m "feat(memory): session replay schema + record/retrieve methods"
```

---

## Task 8: Session Replay — Daemon API Endpoints

**Files:**
- Create: `crates/daemon/src/api/replay.rs`
- Modify: `crates/daemon/src/api/mod.rs`
- Modify: `crates/daemon/src/api/hooks.rs` (add replay recording to hook handlers)

**Step 1: Create replay.rs endpoints**

```rust
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Serialize)]
struct ReplaySession {
    session_id: String,
    event_count: u64,
    first_event: String,
    last_event: String,
}

#[derive(Serialize)]
struct ReplayEvent {
    id: i64,
    hook_event: String,
    request: serde_json::Value,
    response: serde_json::Value,
    latency_ms: i64,
    timestamp: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/replay/sessions", get(list_sessions))
        .route("/replay/{session_id}", get(get_session))
}

async fn list_sessions(State(state): State<AppState>) -> Json<serde_json::Value> {
    match state.memory.replay_sessions().await {
        Ok(sessions) => {
            let items: Vec<ReplaySession> = sessions
                .into_iter()
                .map(|(session_id, event_count, first_event, last_event)| ReplaySession {
                    session_id,
                    event_count,
                    first_event,
                    last_event,
                })
                .collect();
            Json(serde_json::json!({ "sessions": items }))
        }
        Err(e) => Json(serde_json::json!({ "error": format!("{e}") })),
    }
}

async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Json<serde_json::Value> {
    match state.memory.replay_events(&session_id).await {
        Ok(events) => {
            let items: Vec<ReplayEvent> = events
                .into_iter()
                .map(|(id, hook_event, req_json, resp_json, latency_ms, timestamp)| {
                    ReplayEvent {
                        id,
                        hook_event,
                        request: serde_json::from_str(&req_json).unwrap_or(serde_json::Value::String(req_json)),
                        response: serde_json::from_str(&resp_json).unwrap_or(serde_json::Value::String(resp_json)),
                        latency_ms,
                        timestamp,
                    }
                })
                .collect();
            Json(serde_json::json!({
                "session_id": session_id,
                "events": items,
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": format!("{e}") })),
    }
}
```

**Step 2: Wire replay routes into router**

In `crates/daemon/src/api/mod.rs`, add `pub mod replay;` and merge routes:

```rust
pub mod replay;
```

In the `router()` function:

```rust
        .merge(replay::routes())
```

**Step 3: Add replay recording to hook handlers**

In `crates/daemon/src/api/hooks.rs`, at the end of each hook handler (after building the response), add a replay recording call. The pattern is:

```rust
// At the start of the handler, capture start time:
let start = std::time::Instant::now();

// ... existing handler logic ...

// Before returning the response, record replay event:
let latency = start.elapsed().as_millis() as u64;
let session_id = input.hook_event.as_ref()
    .and_then(|e| e.get("session_id"))
    .and_then(|v| v.as_str())
    .unwrap_or("unknown");
let _ = state.memory.record_replay_event(
    session_id,
    "PreToolUse", // or the appropriate hook name
    &serde_json::to_string(&input).unwrap_or_default(),
    &serde_json::to_string(&response).unwrap_or_default(),
    latency,
).await;
```

This should be added to the `pre_tool_use`, `post_tool_use`, `post_tool_use_failure`, `user_prompt_submit`, and `stop` handler functions. The exact insertion points depend on the current handler structure — add it just before the final `Json(response)` return.

**Step 4: Run tests**

Run: `cargo test -p metaygn-daemon`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/daemon/src/api/replay.rs crates/daemon/src/api/mod.rs crates/daemon/src/api/hooks.rs
git commit -m "feat(daemon): session replay API endpoints + hook recording"
```

---

## Task 9: Session Replay — CLI Command

**Files:**
- Modify: `crates/cli/src/main.rs`

**Step 1: Add `Replay` command variant**

In the `Commands` enum:

```rust
    /// Replay a past session's hook timeline
    Replay {
        /// Session ID to replay (omit to list sessions)
        session_id: Option<String>,
    },
```

And the match arm:

```rust
        Commands::Replay { session_id } => cmd_replay(session_id.as_deref()).await,
```

**Step 2: Implement cmd_replay**

```rust
/// Replay command: list sessions or display a session's hook timeline.
async fn cmd_replay(session_id: Option<&str>) -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running (no port file found).");
        return Ok(());
    };

    let client = http_client()?;

    match session_id {
        None => {
            // List all sessions
            let url = format!("http://127.0.0.1:{port}/replay/sessions");
            let resp = client.get(&url).send().await?;
            let body: Value = resp.json().await?;

            let sessions = body
                .get("sessions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if sessions.is_empty() {
                println!("No replay sessions recorded yet.");
                return Ok(());
            }

            println!("{:<40} {:>6}  {:<20}  {:<20}", "SESSION", "EVENTS", "FIRST", "LAST");
            println!("{}", "-".repeat(90));

            for s in &sessions {
                let sid = s.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
                let count = s.get("event_count").and_then(|v| v.as_u64()).unwrap_or(0);
                let first = s.get("first_event").and_then(|v| v.as_str()).unwrap_or("?");
                let last = s.get("last_event").and_then(|v| v.as_str()).unwrap_or("?");
                println!("{:<40} {:>6}  {:<20}  {:<20}", sid, count, first, last);
            }
        }
        Some(sid) => {
            // Show session timeline
            let url = format!("http://127.0.0.1:{port}/replay/{sid}");
            let resp = client.get(&url).send().await?;
            let body: Value = resp.json().await?;

            let events = body
                .get("events")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if events.is_empty() {
                println!("No events found for session: {sid}");
                return Ok(());
            }

            println!("Session: {sid}");
            println!("Events: {}\n", events.len());

            for (i, event) in events.iter().enumerate() {
                let hook = event.get("hook_event").and_then(|v| v.as_str()).unwrap_or("?");
                let latency = event.get("latency_ms").and_then(|v| v.as_i64()).unwrap_or(0);
                let timestamp = event.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?");

                println!("[{:>3}] {} ({latency}ms) @ {timestamp}", i + 1, hook);

                // Compact request/response summary
                if let Some(req) = event.get("request") {
                    let summary = serde_json::to_string(req).unwrap_or_default();
                    if summary.len() > 120 {
                        println!("      req: {}...", &summary[..120]);
                    } else {
                        println!("      req: {summary}");
                    }
                }
                if let Some(resp) = event.get("response") {
                    let summary = serde_json::to_string(resp).unwrap_or_default();
                    if summary.len() > 120 {
                        println!("      res: {}...", &summary[..120]);
                    } else {
                        println!("      res: {summary}");
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p metaygn-cli`
Expected: Compiles.

**Step 4: Commit**

```bash
git add crates/cli/src/main.rs
git commit -m "feat(cli): 'aletheia replay' command for session timeline review"
```

---

## Task 10: Documentation & Version Bump

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `README.md`
- Modify: `.claude-plugin/plugin.json`
- Modify: `memory-bank/activeContext.md`
- Modify: `memory-bank/progress.md`
- Modify: `docs/daemon-contract.md`

**Step 1: Update CHANGELOG.md**

Add at the top (after any existing header):

```markdown
## v0.8.0 "Neural Bridge" — 2026-03-01

### MCP Bridge
- New crate `metaygn-mcp-bridge` with 5 metacognitive MCP tools via stdio transport
- Tools: `metacog_classify`, `metacog_verify`, `metacog_recall`, `metacog_status`, `metacog_prune`
- Uses rmcp 0.17 (official Rust MCP SDK) with `#[tool]` derive macros
- CLI command `aletheia mcp` launches the MCP stdio server
- Daemon client with 5s timeout for HTTP bridge to existing daemon API

### Neural Embeddings
- `FastEmbedProvider` implementing `EmbeddingProvider` trait (bge-small-en-v1.5, 384 dim)
- Feature-gated behind `cargo build --features embeddings` — zero overhead when disabled
- `GraphMemory.semantic_search()` for cosine-similarity vector search
- `POST /memory/semantic` daemon endpoint for vector-based node retrieval
- `HashEmbedProvider` remains the default (no external dependency)

### Session Replay
- `replay_events` SQLite table recording all hook calls with request/response/latency
- `POST /replay/sessions` and `GET /replay/{id}` daemon API endpoints
- `aletheia replay` CLI command: list sessions or view hook timeline
- All 5 hook handlers record replay events automatically
```

**Step 2: Update plugin.json version**

Change `"version"` from `"0.7.0"` to `"0.8.0"`.

**Step 3: Update README.md**

Add MCP bridge, fastembed, and session replay to the feature tables. Add `aletheia mcp` and `aletheia replay` to the CLI commands table. Update the crate count from 7 to 8.

**Step 4: Update daemon-contract.md**

Add the new endpoints:
- `POST /memory/semantic` — Vector-based node search
- `GET /replay/sessions` — List recorded sessions
- `GET /replay/{session_id}` — Get session hook timeline

**Step 5: Update memory-bank/activeContext.md and progress.md**

Mark Phase 10 (Neural Bridge) as COMPLETE. Update active context to reflect v0.8.0 state.

**Step 6: Commit**

```bash
git add CHANGELOG.md README.md .claude-plugin/plugin.json memory-bank/ docs/daemon-contract.md
git commit -m "docs: v0.8.0 Neural Bridge — changelog, readme, plugin version, memory-bank"
```

---

## Task 11: Full Build & Test Verification

**Step 1: Run full workspace build**

Run: `cargo build --workspace`
Expected: All 8 crates compile.

**Step 2: Run full workspace tests**

Run: `cargo test --workspace`
Expected: All tests pass.

**Step 3: Run with embeddings feature (optional)**

Run: `cargo test -p metaygn-memory --features embeddings`
Expected: FastEmbed tests pass (model downloaded).

**Step 4: Type check**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings.

**Step 5: Commit any fixes**

If clippy or tests surfaced issues, fix and commit:

```bash
git add -A
git commit -m "fix: address clippy warnings and test issues from v0.8.0 integration"
```

---

## Summary

| Task | Feature | Files | Est. Complexity |
|------|---------|-------|-----------------|
| 1 | MCP Bridge skeleton | 4 create, 1 modify | Low |
| 2 | MCP 5 tools + server | 2 create, 1 modify | Medium |
| 3 | CLI `mcp` command + test | 2 modify, 1 create | Low |
| 4 | FastEmbedProvider | 2 create, 2 modify | Medium |
| 5 | GraphMemory semantic_search | 1 modify, 1 create | Low |
| 6 | Daemon semantic endpoint | 1 create, 1 modify | Low |
| 7 | Replay schema + store | 1 modify, 1 create | Low |
| 8 | Replay daemon API + hook recording | 2 create, 2 modify | Medium |
| 9 | Replay CLI command | 1 modify | Low |
| 10 | Docs & version bump | 6 modify | Low |
| 11 | Full verification | 0 | Low |

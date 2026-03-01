use std::collections::{HashSet, VecDeque};

use anyhow::Result;
use rusqlite::OptionalExtension;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Scope {
    Session,
    Project,
    Global,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Session => "Session",
            Scope::Project => "Project",
            Scope::Global => "Global",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Session" => Some(Scope::Session),
            "Project" => Some(Scope::Project),
            "Global" => Some(Scope::Global),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Task,
    Decision,
    Evidence,
    Tool,
    Agent,
    Code,
    Error,
    Lesson,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Task => "Task",
            NodeType::Decision => "Decision",
            NodeType::Evidence => "Evidence",
            NodeType::Tool => "Tool",
            NodeType::Agent => "Agent",
            NodeType::Code => "Code",
            NodeType::Error => "Error",
            NodeType::Lesson => "Lesson",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Task" => Some(NodeType::Task),
            "Decision" => Some(NodeType::Decision),
            "Evidence" => Some(NodeType::Evidence),
            "Tool" => Some(NodeType::Tool),
            "Agent" => Some(NodeType::Agent),
            "Code" => Some(NodeType::Code),
            "Error" => Some(NodeType::Error),
            "Lesson" => Some(NodeType::Lesson),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    DependsOn,
    Produces,
    Verifies,
    Contradicts,
    Supersedes,
    RelatedTo,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::DependsOn => "DependsOn",
            EdgeType::Produces => "Produces",
            EdgeType::Verifies => "Verifies",
            EdgeType::Contradicts => "Contradicts",
            EdgeType::Supersedes => "Supersedes",
            EdgeType::RelatedTo => "RelatedTo",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DependsOn" => Some(EdgeType::DependsOn),
            "Produces" => Some(EdgeType::Produces),
            "Verifies" => Some(EdgeType::Verifies),
            "Contradicts" => Some(EdgeType::Contradicts),
            "Supersedes" => Some(EdgeType::Supersedes),
            "RelatedTo" => Some(EdgeType::RelatedTo),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Data structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: String,
    pub node_type: NodeType,
    pub scope: Scope,
    pub label: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub created_at: String,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub weight: f64,
    pub metadata: Option<String>,
}

// ---------------------------------------------------------------------------
// Cosine similarity (pure Rust)
// ---------------------------------------------------------------------------

/// Cosine similarity between two f32 vectors.
///
/// Returns 1.0 for identical direction, 0.0 for orthogonal or degenerate
/// inputs (empty slices, zero-magnitude vectors, mismatched lengths).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

// ---------------------------------------------------------------------------
// Embedding serialisation helpers
// ---------------------------------------------------------------------------

fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        buf.extend_from_slice(&val.to_le_bytes());
    }
    buf
}

fn deserialize_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// GraphMemory
// ---------------------------------------------------------------------------

/// Graph-based memory: nodes + edges stored in SQLite with FTS5 content search.
pub struct GraphMemory {
    conn: Connection,
}

impl GraphMemory {
    /// Open (or create) a file-backed SQLite database at `path`.
    pub async fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).await?;
        let gm = Self { conn };
        gm.init_schema().await?;
        Ok(gm)
    }

    /// Open an in-memory SQLite database (useful for tests).
    pub async fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().await?;
        let gm = Self { conn };
        gm.init_schema().await?;
        Ok(gm)
    }

    /// Create tables, indexes, FTS virtual table, and triggers.
    async fn init_schema(&self) -> Result<()> {
        self.conn
            .call(|conn| {
                conn.execute_batch(
                    "
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA busy_timeout=5000;
                    PRAGMA cache_size=-64000;
                    PRAGMA foreign_keys=ON;

                    CREATE TABLE IF NOT EXISTS nodes (
                        id TEXT PRIMARY KEY,
                        node_type TEXT NOT NULL,
                        scope TEXT NOT NULL,
                        label TEXT NOT NULL,
                        content TEXT NOT NULL,
                        embedding BLOB,
                        created_at TEXT NOT NULL,
                        access_count INTEGER DEFAULT 0
                    );

                    CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(node_type);
                    CREATE INDEX IF NOT EXISTS idx_nodes_scope ON nodes(scope);

                    CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                        label, content, content='nodes', content_rowid='rowid'
                    );

                    CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
                        INSERT INTO nodes_fts(rowid, label, content)
                            VALUES (new.rowid, new.label, new.content);
                    END;

                    CREATE TABLE IF NOT EXISTS edges (
                        source_id TEXT NOT NULL REFERENCES nodes(id),
                        target_id TEXT NOT NULL REFERENCES nodes(id),
                        edge_type TEXT NOT NULL,
                        weight REAL DEFAULT 1.0,
                        metadata TEXT,
                        PRIMARY KEY (source_id, target_id, edge_type)
                    );

                    CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
                    CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
                    ",
                )?;

                // Add UCB columns (silently fails if already exist)
                conn.execute(
                    "ALTER TABLE nodes ADD COLUMN hit_count INTEGER DEFAULT 0",
                    [],
                )
                .ok();
                conn.execute(
                    "ALTER TABLE nodes ADD COLUMN reward_sum REAL DEFAULT 0.0",
                    [],
                )
                .ok();

                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    // -- Write ---------------------------------------------------------------

    /// Insert a node into the graph. Replaces if `id` already exists.
    pub async fn insert_node(&self, node: &MemoryNode) -> Result<()> {
        let id = node.id.clone();
        let node_type = node.node_type.as_str().to_owned();
        let scope = node.scope.as_str().to_owned();
        let label = node.label.clone();
        let content = node.content.clone();
        let embedding: Option<Vec<u8>> = node.embedding.as_ref().map(|e| serialize_embedding(e));
        let created_at = node.created_at.clone();
        let access_count = node.access_count;

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO nodes
                        (id, node_type, scope, label, content, embedding, created_at, access_count)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        id,
                        node_type,
                        scope,
                        label,
                        content,
                        embedding,
                        created_at,
                        access_count,
                    ],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Insert an edge between two existing nodes.
    pub async fn insert_edge(&self, edge: &MemoryEdge) -> Result<()> {
        let source_id = edge.source_id.clone();
        let target_id = edge.target_id.clone();
        let edge_type = edge.edge_type.as_str().to_owned();
        let weight = edge.weight;
        let metadata = edge.metadata.clone();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO edges
                        (source_id, target_id, edge_type, weight, metadata)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![source_id, target_id, edge_type, weight, metadata],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    // -- Read ----------------------------------------------------------------

    /// Retrieve a single node by id, or `None` if it does not exist.
    pub async fn get_node(&self, id: &str) -> Result<Option<MemoryNode>> {
        let id = id.to_owned();
        let node = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count
                     FROM nodes WHERE id = ?1",
                )?;
                let node = stmt
                    .query_row(params![id], |row| Ok(row_to_node(row)))
                    .optional()?;
                Ok::<_, rusqlite::Error>(node)
            })
            .await?;
        Ok(node)
    }

    /// BFS traversal: find all unique neighbor nodes within `depth` hops.
    pub async fn find_neighbors(&self, node_id: &str, depth: u32) -> Result<Vec<MemoryNode>> {
        let start_id = node_id.to_owned();

        let nodes = self
            .conn
            .call(move |conn| {
                let mut visited: HashSet<String> = HashSet::new();
                let mut queue: VecDeque<(String, u32)> = VecDeque::new();
                let mut result: Vec<MemoryNode> = Vec::new();

                visited.insert(start_id.clone());
                queue.push_back((start_id, 0));

                while let Some((current_id, current_depth)) = queue.pop_front() {
                    if current_depth >= depth {
                        continue;
                    }

                    // Find all neighbors of current_id (both directions)
                    let mut stmt = conn.prepare(
                        "SELECT target_id FROM edges WHERE source_id = ?1
                         UNION
                         SELECT source_id FROM edges WHERE target_id = ?1",
                    )?;
                    let neighbor_ids: Vec<String> = stmt
                        .query_map(params![current_id], |row| row.get::<_, String>(0))?
                        .filter_map(|r| r.ok())
                        .collect();

                    for nid in neighbor_ids {
                        if visited.contains(&nid) {
                            continue;
                        }
                        visited.insert(nid.clone());

                        // Fetch the node data
                        let mut nstmt = conn.prepare(
                            "SELECT id, node_type, scope, label, content, embedding,
                                    created_at, access_count
                             FROM nodes WHERE id = ?1",
                        )?;
                        if let Ok(node) =
                            nstmt.query_row(params![nid.clone()], |row| Ok(row_to_node(row)))
                        {
                            result.push(node);
                        }

                        queue.push_back((nid, current_depth + 1));
                    }
                }

                Ok::<_, rusqlite::Error>(result)
            })
            .await?;

        Ok(nodes)
    }

    /// Return nodes of the given type, ordered by `created_at` descending.
    pub async fn nodes_by_type(&self, node_type: NodeType, limit: u32) -> Result<Vec<MemoryNode>> {
        let nt = node_type.as_str().to_owned();
        let nodes = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count
                     FROM nodes
                     WHERE node_type = ?1
                     ORDER BY created_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(params![nt, limit], |row| Ok(row_to_node(row)))?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(nodes)
    }

    /// Return nodes in the given scope, ordered by `created_at` descending.
    pub async fn nodes_by_scope(&self, scope: Scope, limit: u32) -> Result<Vec<MemoryNode>> {
        let sc = scope.as_str().to_owned();
        let nodes = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count
                     FROM nodes
                     WHERE scope = ?1
                     ORDER BY created_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(params![sc, limit], |row| Ok(row_to_node(row)))?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(nodes)
    }

    /// Full-text search over node labels and content using FTS5.
    pub async fn search_content(&self, query: &str, limit: u32) -> Result<Vec<MemoryNode>> {
        let query = query.to_owned();
        let nodes = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT n.id, n.node_type, n.scope, n.label, n.content,
                            n.embedding, n.created_at, n.access_count
                     FROM nodes_fts f
                     JOIN nodes n ON n.rowid = f.rowid
                     WHERE nodes_fts MATCH ?1
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(params![query, limit], |row| Ok(row_to_node(row)))?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(nodes)
    }

    // -- Semantic search -----------------------------------------------------

    /// Semantic search: find the top-N nodes whose stored embedding is most
    /// similar to the given query embedding (cosine similarity).
    /// Only considers nodes that have a non-empty embedding.
    pub async fn semantic_search(
        &self,
        query_embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<(MemoryNode, f32)>> {
        let query = query_embedding.to_vec();
        let results = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count
                     FROM nodes
                     WHERE embedding IS NOT NULL",
                )?;
                let mut scored: Vec<(MemoryNode, f32)> = stmt
                    .query_map([], |row| Ok(row_to_node(row)))?
                    .filter_map(|r| r.ok())
                    .filter_map(|node| {
                        let emb = node.embedding.as_ref()?;
                        if emb.is_empty() {
                            return None;
                        }
                        let score = cosine_similarity(&query, emb);
                        Some((node, score))
                    })
                    .collect();
                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored.truncate(limit as usize);
                Ok::<_, rusqlite::Error>(scored)
            })
            .await?;
        Ok(results)
    }

    // -- UCB adaptive recall --------------------------------------------------

    /// Record a reward signal for a recalled node (UCB bandit update).
    pub async fn record_recall_reward(&self, node_id: &str, reward: f64) -> Result<()> {
        let node_id = node_id.to_owned();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE nodes SET hit_count = hit_count + 1, reward_sum = reward_sum + ?1 WHERE id = ?2",
                    params![reward, node_id],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Adaptive recall: combines cosine similarity with UCB exploration bonus.
    ///
    /// The final score is `0.7 * cosine + 0.3 * ucb_normalized` where UCB is
    /// computed from per-node reward statistics using the Upper Confidence Bound
    /// formula. Nodes that have never been recalled get a high exploration bonus.
    pub async fn adaptive_recall(
        &self,
        query_embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<(MemoryNode, f32)>> {
        let query_emb = query_embedding.to_vec();
        let results = self
            .conn
            .call(move |conn| {
                let total_queries: f64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(hit_count), 1) FROM nodes",
                        [],
                        |row| row.get::<_, f64>(0),
                    )
                    .unwrap_or(1.0_f64)
                    .max(1.0);

                let mut stmt = conn.prepare(
                    "SELECT id, node_type, scope, label, content, embedding,
                            created_at, access_count, COALESCE(hit_count, 0), COALESCE(reward_sum, 0.0)
                     FROM nodes WHERE embedding IS NOT NULL",
                )?;

                let mut scored: Vec<(MemoryNode, f32)> = stmt
                    .query_map([], |row| {
                        let node = row_to_node(row);
                        let hit_count: i64 = row.get(8)?;
                        let reward_sum: f64 = row.get(9)?;
                        Ok((node, hit_count, reward_sum))
                    })?
                    .filter_map(|r| r.ok())
                    .filter_map(|(node, hit_count, reward_sum)| {
                        let emb = node.embedding.as_ref()?;
                        if emb.is_empty() {
                            return None;
                        }
                        let cosine = cosine_similarity(&query_emb, emb);
                        let hits = (hit_count as f64).max(1.0);
                        let mean_reward = reward_sum / hits;
                        let exploration = (2.0 * total_queries.ln() / hits).sqrt();
                        let ucb = mean_reward + exploration;
                        let ucb_normalized = (ucb / 3.0).min(1.0) as f32;
                        let final_score = 0.7 * cosine + 0.3 * ucb_normalized;
                        Some((node, final_score))
                    })
                    .collect();

                scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored.truncate(limit as usize);
                Ok::<_, rusqlite::Error>(scored)
            })
            .await?;
        Ok(results)
    }

    // -- Counts --------------------------------------------------------------

    /// Total number of nodes in the graph.
    pub async fn node_count(&self) -> Result<u64> {
        let count = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT COUNT(*) FROM nodes")?;
                let count: u64 = stmt.query_row([], |row| row.get(0))?;
                Ok::<_, rusqlite::Error>(count)
            })
            .await?;
        Ok(count)
    }

    /// Total number of edges in the graph.
    pub async fn edge_count(&self) -> Result<u64> {
        let count = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT COUNT(*) FROM edges")?;
                let count: u64 = stmt.query_row([], |row| row.get(0))?;
                Ok::<_, rusqlite::Error>(count)
            })
            .await?;
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Map a rusqlite Row to a MemoryNode. Must be called inside a `call` closure.
fn row_to_node(row: &rusqlite::Row<'_>) -> MemoryNode {
    let embedding_blob: Option<Vec<u8>> = row.get(5).unwrap_or(None);
    MemoryNode {
        id: row.get(0).unwrap(),
        node_type: NodeType::from_str(&row.get::<_, String>(1).unwrap()).unwrap(),
        scope: Scope::from_str(&row.get::<_, String>(2).unwrap()).unwrap(),
        label: row.get(3).unwrap(),
        content: row.get(4).unwrap(),
        embedding: embedding_blob.map(|b| deserialize_embedding(&b)),
        created_at: row.get(6).unwrap(),
        access_count: row.get(7).unwrap(),
    }
}

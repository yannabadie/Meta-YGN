use anyhow::Result;
use rusqlite::params;
use tokio_rusqlite::Connection;

/// A row returned from the events table.
#[derive(Debug, Clone)]
pub struct EventRow {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub payload: String,
    pub timestamp: String,
}

/// Async SQLite-backed event store with FTS5 full-text search.
pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    /// Open (or create) a file-backed SQLite database at `path`.
    pub async fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).await?;
        let store = Self { conn };
        store.init_schema().await?;
        Ok(store)
    }

    /// Open an in-memory SQLite database (useful for tests).
    pub async fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().await?;
        let store = Self { conn };
        store.init_schema().await?;
        Ok(store)
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

                    CREATE VIRTUAL TABLE IF NOT EXISTS events_fts
                        USING fts5(payload, content='events', content_rowid='rowid');

                    CREATE TRIGGER IF NOT EXISTS events_ai AFTER INSERT ON events BEGIN
                        INSERT INTO events_fts(rowid, payload) VALUES (new.rowid, new.payload);
                    END;

                    CREATE TABLE IF NOT EXISTS heuristic_versions (
                        id TEXT PRIMARY KEY,
                        generation INTEGER NOT NULL,
                        parent_id TEXT,
                        fitness_json TEXT NOT NULL,
                        risk_weights_json TEXT NOT NULL,
                        strategy_scores_json TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS session_outcomes (
                        id TEXT PRIMARY KEY,
                        session_id TEXT NOT NULL,
                        task_type TEXT,
                        risk_level TEXT,
                        strategy_used TEXT,
                        success INTEGER NOT NULL,
                        tokens_consumed INTEGER,
                        duration_ms INTEGER,
                        errors_encountered INTEGER,
                        created_at TEXT NOT NULL DEFAULT (datetime('now'))
                    );

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

                    CREATE TABLE IF NOT EXISTS rl2f_trajectories (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        session_id TEXT NOT NULL,
                        trajectory_json TEXT NOT NULL,
                        signature_hash TEXT,
                        timestamp TEXT NOT NULL DEFAULT (datetime('now'))
                    );

                    CREATE INDEX IF NOT EXISTS idx_trajectories_session
                        ON rl2f_trajectories(session_id, timestamp);
                    ",
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Insert a new event and return its UUID.
    pub async fn log_event(
        &self,
        session_id: &str,
        event_type: &str,
        payload: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = chrono::Utc::now().to_rfc3339();
        let session_id = session_id.to_owned();
        let event_type = event_type.to_owned();
        let payload = payload.to_owned();
        let id_clone = id.clone();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO events (id, session_id, event_type, payload, timestamp)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id_clone, session_id, event_type, payload, ts],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(id)
    }

    /// Return the total number of events in the store.
    pub async fn event_count(&self) -> Result<u64> {
        let count = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT COUNT(*) FROM events")?;
                let count: u64 = stmt.query_row([], |row| row.get(0))?;
                Ok::<_, rusqlite::Error>(count)
            })
            .await?;
        Ok(count)
    }

    /// Return recent events for a session, ordered by timestamp ascending.
    pub async fn recent_events(&self, session_id: &str, limit: u32) -> Result<Vec<EventRow>> {
        let session_id = session_id.to_owned();
        let rows = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, session_id, event_type, payload, timestamp
                     FROM events
                     WHERE session_id = ?1
                     ORDER BY timestamp ASC
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(params![session_id, limit], |row| {
                        Ok(EventRow {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            event_type: row.get(2)?,
                            payload: row.get(3)?,
                            timestamp: row.get(4)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<EventRow>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }

    /// Full-text search over event payloads using FTS5.
    pub async fn search_events(&self, query: &str, limit: u32) -> Result<Vec<EventRow>> {
        let query = query.to_owned();
        let rows = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT e.id, e.session_id, e.event_type, e.payload, e.timestamp
                     FROM events_fts f
                     JOIN events e ON e.rowid = f.rowid
                     WHERE events_fts MATCH ?1
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(params![query, limit], |row| {
                        Ok(EventRow {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            event_type: row.get(2)?,
                            payload: row.get(3)?,
                            timestamp: row.get(4)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<EventRow>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }

    /// Persist a heuristic version snapshot.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_heuristic(
        &self,
        id: &str,
        generation: u32,
        parent_id: Option<&str>,
        fitness_json: &str,
        risk_weights_json: &str,
        strategy_scores_json: &str,
        created_at: &str,
    ) -> Result<()> {
        let id = id.to_owned();
        let generation = generation as i64;
        let parent_id = parent_id.map(|s| s.to_owned());
        let fitness_json = fitness_json.to_owned();
        let risk_weights_json = risk_weights_json.to_owned();
        let strategy_scores_json = strategy_scores_json.to_owned();
        let created_at = created_at.to_owned();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO heuristic_versions
                     (id, generation, parent_id, fitness_json, risk_weights_json, strategy_scores_json, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![id, generation, parent_id, fitness_json, risk_weights_json, strategy_scores_json, created_at],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Load all heuristic versions, ordered by generation ascending.
    pub async fn load_heuristics(
        &self,
    ) -> Result<Vec<(String, u32, Option<String>, String, String, String, String)>> {
        let rows = self
            .conn
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, generation, parent_id, fitness_json, risk_weights_json, strategy_scores_json, created_at
                     FROM heuristic_versions
                     ORDER BY generation ASC",
                )?;
                let rows = stmt
                    .query_map([], |row| {
                        let generation: i64 = row.get(1)?;
                        Ok((
                            row.get::<_, String>(0)?,
                            generation as u32,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }

    /// Record a session outcome for heuristic fitness tracking.
    #[allow(clippy::too_many_arguments)]
    pub async fn save_outcome(
        &self,
        id: &str,
        session_id: &str,
        task_type: &str,
        risk_level: &str,
        strategy_used: &str,
        success: bool,
        tokens_consumed: u64,
        duration_ms: u64,
        errors_encountered: u32,
    ) -> Result<()> {
        let id = id.to_owned();
        let session_id = session_id.to_owned();
        let task_type = task_type.to_owned();
        let risk_level = risk_level.to_owned();
        let strategy_used = strategy_used.to_owned();
        let success_int: i64 = if success { 1 } else { 0 };
        let tokens_consumed = tokens_consumed as i64;
        let duration_ms = duration_ms as i64;
        let errors_encountered = errors_encountered as i64;

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO session_outcomes
                     (id, session_id, task_type, risk_level, strategy_used, success, tokens_consumed, duration_ms, errors_encountered)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![id, session_id, task_type, risk_level, strategy_used, success_int, tokens_consumed, duration_ms, errors_encountered],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Load the most recent session outcomes as JSON values.
    pub async fn load_recent_outcomes(&self, limit: u32) -> Result<Vec<serde_json::Value>> {
        let rows = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, session_id, task_type, risk_level, strategy_used, success,
                            tokens_consumed, duration_ms, errors_encountered, created_at
                     FROM session_outcomes
                     ORDER BY created_at DESC
                     LIMIT ?1",
                )?;
                let rows = stmt
                    .query_map(params![limit], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "session_id": row.get::<_, String>(1)?,
                            "task_type": row.get::<_, Option<String>>(2)?,
                            "risk_level": row.get::<_, Option<String>>(3)?,
                            "strategy_used": row.get::<_, Option<String>>(4)?,
                            "success": row.get::<_, i64>(5)? != 0,
                            "tokens_consumed": row.get::<_, Option<i64>>(6)?,
                            "duration_ms": row.get::<_, Option<i64>>(7)?,
                            "errors_encountered": row.get::<_, Option<i64>>(8)?,
                            "created_at": row.get::<_, String>(9)?,
                        }))
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }

    /// Record a replay event for session replay.
    pub async fn record_replay_event(
        &self,
        session_id: &str,
        hook_event: &str,
        request_json: &str,
        response_json: &str,
        latency_ms: i64,
    ) -> Result<()> {
        let session_id = session_id.to_owned();
        let hook_event = hook_event.to_owned();
        let request_json = request_json.to_owned();
        let response_json = response_json.to_owned();

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

    /// List all replay sessions with event counts, ordered by most recent last event.
    /// Returns Vec of (session_id, event_count, first_event_timestamp, last_event_timestamp).
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

    /// Retrieve all replay events for a given session, ordered by id ascending.
    /// Returns Vec of (id, hook_event, request_json, response_json, latency_ms, timestamp).
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

    /// Save an RL2F trajectory record for fine-tuning data export.
    pub async fn save_trajectory(
        &self,
        session_id: &str,
        trajectory_json: &str,
        signature_hash: Option<&str>,
    ) -> Result<()> {
        let session_id = session_id.to_owned();
        let trajectory_json = trajectory_json.to_owned();
        let signature_hash = signature_hash.map(|s| s.to_owned());
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO rl2f_trajectories (session_id, trajectory_json, signature_hash) VALUES (?1, ?2, ?3)",
                    params![session_id, trajectory_json, signature_hash],
                )?;
                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        Ok(())
    }

    /// Get the success rate for a given task type from recent session outcomes.
    /// Returns None if fewer than `min_samples` outcomes exist.
    pub async fn success_rate_for_task_type(
        &self,
        task_type: &str,
        limit: u32,
        min_samples: u32,
    ) -> Result<Option<f32>> {
        let task_type = task_type.to_owned();
        let result = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT COUNT(*) as total,
                            COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0) as successes
                     FROM (
                         SELECT success FROM session_outcomes
                         WHERE task_type = ?1
                         ORDER BY created_at DESC
                         LIMIT ?2
                     )",
                )?;
                let (total, successes): (u32, u32) =
                    stmt.query_row(rusqlite::params![task_type, limit], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })?;
                Ok::<_, rusqlite::Error>((total, successes))
            })
            .await?;
        let (total, successes) = result;
        if total < min_samples {
            Ok(None)
        } else {
            Ok(Some(successes as f32 / total as f32))
        }
    }

    /// Export recent RL2F trajectories, ordered by timestamp descending.
    /// Returns Vec of (id, session_id, trajectory_json, signature_hash, timestamp).
    pub async fn export_trajectories(
        &self,
        limit: u32,
    ) -> Result<Vec<(i64, String, String, Option<String>, String)>> {
        let rows = self
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, session_id, trajectory_json, signature_hash, timestamp
                     FROM rl2f_trajectories
                     ORDER BY timestamp DESC
                     LIMIT ?1",
                )?;
                let rows = stmt
                    .query_map(params![limit], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, Option<String>>(3)?,
                            row.get::<_, String>(4)?,
                        ))
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                Ok::<_, rusqlite::Error>(rows)
            })
            .await?;
        Ok(rows)
    }
}

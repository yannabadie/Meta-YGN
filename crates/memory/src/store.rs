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
    pub async fn recent_events(
        &self,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<EventRow>> {
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
    pub async fn search_events(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<EventRow>> {
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
}

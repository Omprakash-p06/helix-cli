use rusqlite::{params, Connection, Result as SqlResult};
use sha2::{Digest, Sha256};

/// A row representing an evidence chunk stored in SQLite.
#[derive(Debug, Clone)]
pub struct EvidenceRow {
    pub id: i64,
    pub content_hash: String,
    pub markdown_value: String,
    pub relevance_score: f64,
}

/// SQLite-backed store for web research sources, evidence, citations, and freshness cache.
pub struct EvidenceStore {
    conn: Connection,
}

impl EvidenceStore {
    /// Constructs a new `EvidenceStore` and runs database schema migrations.
    pub fn new(conn: Connection) -> SqlResult<Self> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS research_sources (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                url          TEXT UNIQUE NOT NULL,
                title        TEXT,
                content_hash TEXT,
                status_code  INTEGER,
                scraped_at   DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS research_evidence (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash    TEXT UNIQUE NOT NULL,
                markdown_value  TEXT NOT NULL,
                relevance_score REAL DEFAULT 0.5,
                created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS research_citations (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                evidence_id INTEGER NOT NULL REFERENCES research_evidence(id) ON DELETE CASCADE,
                source_id   INTEGER NOT NULL REFERENCES research_sources(id) ON DELETE CASCADE,
                locator     TEXT
            );

            CREATE TABLE IF NOT EXISTS research_freshness_cache (
                query_hash   TEXT PRIMARY KEY,
                session_id   TEXT,
                last_fetched DATETIME DEFAULT CURRENT_TIMESTAMP,
                ttl_seconds  INTEGER DEFAULT 604800
            );

            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;
            "
        )?;

        Ok(Self { conn })
    }

    /// Inserts a web research source record. Returns the source row ID.
    pub fn insert_source(&mut self, url: &str, title: Option<&str>, content_hash: &str, status_code: u16) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO research_sources (url, title, content_hash, status_code) VALUES (?1, ?2, ?3, ?4)",
            params![url, title, content_hash, status_code],
        )?;

        let id: i64 = self.conn.query_row(
            "SELECT id FROM research_sources WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    /// Inserts an evidence chunk. Returns the evidence row ID.
    pub fn insert_evidence(&mut self, content_hash: &str, markdown_value: &str, relevance_score: f64) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO research_evidence (content_hash, markdown_value, relevance_score) VALUES (?1, ?2, ?3)",
            params![content_hash, markdown_value, relevance_score],
        )?;

        let id: i64 = self.conn.query_row(
            "SELECT id FROM research_evidence WHERE content_hash = ?1",
            params![content_hash],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    /// Inserts a citation mapping an evidence chunk to its source.
    pub fn insert_citation(&mut self, evidence_id: i64, source_id: i64, locator: Option<&str>) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO research_citations (evidence_id, source_id, locator) VALUES (?1, ?2, ?3)",
            params![evidence_id, source_id, locator],
        )?;
        Ok(())
    }

    /// Queries the age (in seconds) of a cached query result, if available.
    pub fn query_age(&self, query: &str) -> Option<i64> {
        let query_hash = hex::encode(Sha256::digest(query.as_bytes()));
        let mut stmt = self.conn.prepare(
            "SELECT strftime('%s', 'now') - strftime('%s', last_fetched) FROM research_freshness_cache WHERE query_hash = ?1"
        ).ok()?;

        stmt.query_row(params![query_hash], |row| row.get(0)).ok()
    }

    /// Upserts a query freshness record into the cache.
    pub fn upsert_freshness(&mut self, query: &str, session_id: Option<&str>, ttl_secs: i64) -> SqlResult<()> {
        let query_hash = hex::encode(Sha256::digest(query.as_bytes()));
        self.conn.execute(
            "INSERT OR REPLACE INTO research_freshness_cache (query_hash, session_id, last_fetched, ttl_seconds) VALUES (?1, ?2, CURRENT_TIMESTAMP, ?3)",
            params![query_hash, session_id, ttl_secs],
        )?;
        Ok(())
    }

    /// Fetches top evidence chunks ordered by relevance score.
    pub fn evidence_for_session(&self, _session_id: &str) -> Vec<EvidenceRow> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, content_hash, markdown_value, relevance_score FROM research_evidence ORDER BY relevance_score DESC LIMIT 50"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map([], |row| {
            Ok(EvidenceRow {
                id: row.get(0)?,
                content_hash: row.get(1)?,
                markdown_value: row.get(2)?,
                relevance_score: row.get(3)?,
            })
        });

        match rows {
            Ok(iter) => iter.flatten().collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_deduplication() {
        let conn = Connection::open_in_memory().unwrap();
        let mut store = EvidenceStore::new(conn).unwrap();

        let id1 = store.insert_evidence("hash123", "Markdown content", 0.8).unwrap();
        let id2 = store.insert_evidence("hash123", "Markdown content", 0.8).unwrap();

        assert_eq!(id1, id2);

        let rows = store.evidence_for_session("test_session");
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn freshness_cache_round_trip() {
        let conn = Connection::open_in_memory().unwrap();
        let mut store = EvidenceStore::new(conn).unwrap();

        assert!(store.query_age("test query").is_none());

        store.upsert_freshness("test query", Some("sess1"), 3600).unwrap();

        let age = store.query_age("test query");
        assert!(age.is_some());
        assert!(age.unwrap() < 5);
    }
}

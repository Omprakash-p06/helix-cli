//! Durable agent memory layer backed by SQLite with FTS5.
//!
//! Stores goals, constraints, decisions, failed attempts, observations,
//! and the edit ledger — all of which survive LLM context compaction cycles.
//!
//! # Compaction Survival
//! A "compaction cycle" is a context window reset. On resume, call
//! `resume_session(session_id)` to get a compact formatted string containing
//! the session goal + top-priority memories + recent edits. Inject this
//! at the top of the new LLM context window.

use rusqlite::{Connection, Result as SqlResult, params};
use serde::{Deserialize, Serialize};

/// Categories of memory entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryKind {
    Goal,
    Constraint,
    Decision,
    Failure,
    Edit,
    Observation,
}

impl MemoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Goal => "goal",
            Self::Constraint => "constraint",
            Self::Decision => "decision",
            Self::Failure => "failure",
            Self::Edit => "edit",
            Self::Observation => "observation",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "goal" => Self::Goal,
            "constraint" => Self::Constraint,
            "decision" => Self::Decision,
            "failure" => Self::Failure,
            "edit" => Self::Edit,
            _ => Self::Observation,
        }
    }
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: i64,
    pub session_id: String,
    pub kind: MemoryKind,
    pub content: String,
    pub created_at: i64,
    pub importance: f64,
}

/// A session row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub goal: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// The durable memory engine.
pub struct MemoryEngine {
    conn: Connection,
}

impl MemoryEngine {
    const SCHEMA: &'static str = r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id          TEXT PRIMARY KEY,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL,
            goal        TEXT,
            status      TEXT NOT NULL DEFAULT 'active'
        );

        CREATE TABLE IF NOT EXISTS memory (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id  TEXT NOT NULL REFERENCES sessions(id),
            kind        TEXT NOT NULL,
            content     TEXT NOT NULL,
            metadata    TEXT,
            created_at  INTEGER NOT NULL,
            importance  REAL NOT NULL DEFAULT 1.0
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
            content,
            kind,
            content='memory',
            content_rowid='id'
        );

        CREATE TABLE IF NOT EXISTS edit_ledger (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id  TEXT NOT NULL,
            file_path   TEXT NOT NULL,
            edit_kind   TEXT NOT NULL,
            diff_patch  TEXT,
            applied_at  INTEGER NOT NULL,
            reverted    INTEGER NOT NULL DEFAULT 0
        );

        CREATE TRIGGER IF NOT EXISTS memory_fts_insert
        AFTER INSERT ON memory BEGIN
            INSERT INTO memory_fts(rowid, content, kind)
            VALUES (new.id, new.content, new.kind);
        END;

        CREATE TRIGGER IF NOT EXISTS memory_fts_delete
        AFTER DELETE ON memory BEGIN
            INSERT INTO memory_fts(memory_fts, rowid, content, kind)
            VALUES ('delete', old.id, old.content, old.kind);
        END;

        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
    "#;

    /// Open or create a memory engine with the given SQLite connection.
    pub fn new(conn: Connection) -> SqlResult<Self> {
        conn.execute_batch(Self::SCHEMA)?;
        Ok(Self { conn })
    }

    /// Create a new session and return its UUID.
    pub fn new_session(&self, goal: &str) -> SqlResult<String> {
        let id = generate_uuid();
        let now = unix_now();
        self.conn.execute(
            "INSERT INTO sessions (id, created_at, updated_at, goal, status) VALUES (?1, ?2, ?3, ?4, 'active')",
            params![id, now, now, goal],
        )?;
        Ok(id)
    }

    /// Record a memory entry for a session.
    /// Returns the inserted row ID.
    pub fn record(&self, session_id: &str, kind: MemoryKind, content: &str) -> SqlResult<i64> {
        let now = unix_now();
        self.conn.execute(
            "INSERT INTO memory (session_id, kind, content, created_at, importance) VALUES (?1, ?2, ?3, ?4, 1.0)",
            params![session_id, kind.as_str(), content, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Record an edit in the edit ledger.
    pub fn record_edit(
        &self,
        session_id: &str,
        file_path: &str,
        edit_kind: &str,
        diff_patch: Option<&str>,
    ) -> SqlResult<()> {
        let now = unix_now();
        self.conn.execute(
            "INSERT INTO edit_ledger (session_id, file_path, edit_kind, diff_patch, applied_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, file_path, edit_kind, diff_patch, now],
        )?;
        Ok(())
    }

    /// Load all memory entries for a session, ordered by importance DESC.
    pub fn load_session(&self, session_id: &str) -> Vec<MemoryEntry> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, session_id, kind, content, created_at, importance FROM memory
             WHERE session_id = ?1 ORDER BY importance DESC, created_at DESC"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![session_id], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                kind: MemoryKind::from_str(&row.get::<_, String>(2)?),
                content: row.get(3)?,
                created_at: row.get(4)?,
                importance: row.get(5)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// List all active sessions.
    pub fn list_active_sessions(&self) -> Vec<Session> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, goal, status, created_at, updated_at FROM sessions WHERE status = 'active'"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                goal: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Search memory entries using FTS5 keyword search.
    pub fn search_memory(&self, query: &str) -> Vec<MemoryEntry> {
        let mut stmt = match self.conn.prepare(
            "SELECT m.id, m.session_id, m.kind, m.content, m.created_at, m.importance
             FROM memory m
             JOIN memory_fts ON m.id = memory_fts.rowid
             WHERE memory_fts MATCH ?1
             ORDER BY rank LIMIT 20"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map(params![query], |row| {
            Ok(MemoryEntry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                kind: MemoryKind::from_str(&row.get::<_, String>(2)?),
                content: row.get(3)?,
                created_at: row.get(4)?,
                importance: row.get(5)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Resume a session after compaction — returns a formatted context block
    /// for LLM re-injection at the top of a fresh context window.
    pub fn resume_session(&self, session_id: &str) -> String {
        let session = self.conn.query_row(
            "SELECT id, goal, status, created_at, updated_at FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok(Session {
                id: row.get(0)?,
                goal: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            }),
        ).ok();

        let memories = self.load_session(session_id);
        let edits: Vec<(String, String)> = {
            let mut stmt = self.conn.prepare(
                "SELECT file_path, edit_kind FROM edit_ledger WHERE session_id = ?1
                 AND reverted = 0 ORDER BY applied_at DESC LIMIT 10"
            ).unwrap_or_else(|_| panic!("prepare failed"));
            stmt.query_map(params![session_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }).map(|r| r.flatten().collect()).unwrap_or_default()
        };

        let mut out = String::from("=== SESSION CONTEXT (RESUMED AFTER COMPACTION) ===\n");
        if let Some(s) = &session {
            out.push_str(&format!("Goal: {}\n", s.goal.as_deref().unwrap_or("(none)")));
            out.push_str(&format!("Status: {}\n", s.status));
        }
        if !memories.is_empty() {
            out.push_str("\n--- Memory ---\n");
            for m in memories.iter().take(20) {
                out.push_str(&format!("[{}] {}\n", m.kind.as_str(), m.content));
            }
        }
        if !edits.is_empty() {
            out.push_str("\n--- Recent Edits ---\n");
            for (file, kind) in &edits {
                out.push_str(&format!("{}: {}\n", kind, file));
            }
        }
        out.push_str("===================================================\n");
        out
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn generate_uuid() -> String {
    // Deterministic enough for local use without uuid crate dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", t.as_secs(), t.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory_engine() -> MemoryEngine {
        MemoryEngine::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn test_new_session_and_load() {
        let engine = in_memory_engine();
        let sid = engine.new_session("Fix memory leak in session.rs").unwrap();
        engine.record(&sid, MemoryKind::Constraint, "Never modify audit schema").unwrap();
        engine.record(&sid, MemoryKind::Failure, "Attempt 1: Arc<Mutex<>> race condition").unwrap();

        let memories = engine.load_session(&sid);
        assert_eq!(memories.len(), 2, "Must have 2 memory entries");
        assert!(memories.iter().any(|m| m.kind == MemoryKind::Constraint));
        assert!(memories.iter().any(|m| m.kind == MemoryKind::Failure));
    }

    #[test]
    fn test_list_active_sessions() {
        let engine = in_memory_engine();
        let _sid = engine.new_session("Active session goal").unwrap();
        let sessions = engine.list_active_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, "active");
    }

    #[test]
    fn test_session_survives_compaction() {
        // Simulates compaction by dropping the engine and re-opening from a file
        let db_file = std::env::temp_dir().join("helix_test_memory_compaction.db");
        let _ = std::fs::remove_file(&db_file); // clean start

        let session_id;
        // Phase 1: write state
        {
            let conn = Connection::open(&db_file).unwrap();
            let engine = MemoryEngine::new(conn).unwrap();
            session_id = engine.new_session("Fix memory leak in session.rs").unwrap();
            engine.record(&session_id, MemoryKind::Constraint, "Never modify audit schema").unwrap();
            engine.record(&session_id, MemoryKind::Failure, "Attempt 1: Arc<Mutex<>> race condition").unwrap();
        } // Drop = simulated context reset / compaction

        // Phase 2: resume in fresh engine instance
        {
            let conn = Connection::open(&db_file).unwrap();
            let engine = MemoryEngine::new(conn).unwrap();
            let sessions = engine.list_active_sessions();
            assert_eq!(sessions.len(), 1, "Session must persist across compaction");
            assert_eq!(sessions[0].id, session_id);

            let memories = engine.load_session(&session_id);
            assert!(memories.iter().any(|m| m.kind == MemoryKind::Constraint),
                "Constraint must survive compaction");
            assert!(memories.iter().any(|m| m.kind == MemoryKind::Failure),
                "Failure record must survive compaction");
        }

        let _ = std::fs::remove_file(&db_file); // cleanup
    }

    #[test]
    fn test_resume_session_format() {
        let engine = in_memory_engine();
        let sid = engine.new_session("Diagnose high CPU usage").unwrap();
        engine.record(&sid, MemoryKind::Observation, "CPU at 95% on process helix-agent").unwrap();
        engine.record_edit(&sid, "src/main.rs", "replace", Some("- old\n+ new")).unwrap();

        let resume = engine.resume_session(&sid);
        assert!(resume.contains("SESSION CONTEXT"), "Must contain session context header");
        assert!(resume.contains("Diagnose high CPU usage"), "Must contain session goal");
        assert!(resume.contains("observation"), "Must contain memory kind");
    }

    #[test]
    fn test_fts5_search() {
        let engine = in_memory_engine();
        let sid = engine.new_session("Debug auth issue").unwrap();
        engine.record(&sid, MemoryKind::Observation, "JWT token validation fails on expired tokens").unwrap();
        engine.record(&sid, MemoryKind::Decision, "Use RS256 algorithm for token signing").unwrap();

        let results = engine.search_memory("token");
        // FTS5 should find both entries containing "token"
        assert!(!results.is_empty(), "FTS5 search for 'token' must return results");
    }

    #[test]
    fn test_edit_ledger() {
        let engine = in_memory_engine();
        let sid = engine.new_session("Apply security patch").unwrap();
        engine.record_edit(&sid, "src/security/policy.rs", "replace", Some("diff content")).unwrap();
        engine.record_edit(&sid, "src/tools.rs", "insert", None).unwrap();

        let resume = engine.resume_session(&sid);
        assert!(resume.contains("src/security/policy.rs"), "Edit ledger must appear in resume");
    }
}

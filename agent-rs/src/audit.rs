use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Option<i64>,
    pub timestamp: i64,
    pub actor: String,      // e.g. "user", "agent"
    pub path: String,       // e.g. "terminal", "web"
    pub event_type: String, // e.g. "policy", "execution"
    pub tool_name: String,
    pub decision: Option<String>, // allow, approval-required, deny
    pub outcome: Option<String>,  // success, failure
    pub reason: Option<String>,
    pub remediation: Option<String>,
    pub args_hash: String,
    pub output_hash: Option<String>,
    pub duration_ms: Option<i64>,
    pub prev_hash: String,
    pub event_hash: String,
}

pub struct AuditStore {
    conn: Mutex<Connection>,
}

impl AuditStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                actor TEXT NOT NULL,
                path TEXT NOT NULL,
                event_type TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                decision TEXT,
                outcome TEXT,
                reason TEXT,
                remediation TEXT,
                args_hash TEXT NOT NULL,
                output_hash TEXT,
                duration_ms INTEGER,
                prev_hash TEXT NOT NULL,
                event_hash TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for efficient filtering
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs (timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_path ON audit_logs (path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_tool ON audit_logs (tool_name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_decision ON audit_logs (decision)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_audit_outcome ON audit_logs (outcome)", [])?;

        Ok(AuditStore { conn: Mutex::new(conn) })
    }

    pub fn get_last_hash(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT event_hash FROM audit_logs ORDER BY id DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            row.get(0)
        } else {
            Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
        }
    }

    pub fn append_event(
        &self,
        actor: &str,
        path: &str,
        event_type: &str,
        tool_name: &str,
        decision: Option<&str>,
        outcome: Option<&str>,
        reason: Option<&str>,
        remediation: Option<&str>,
        args_hash: &str,
        output_hash: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        let prev_hash = self.get_last_hash()?;
        
        // Calculate event hash for tamper evidence
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_be_bytes());
        hasher.update(actor.as_bytes());
        hasher.update(path.as_bytes());
        hasher.update(event_type.as_bytes());
        hasher.update(tool_name.as_bytes());
        if let Some(d) = decision { hasher.update(d.as_bytes()); }
        if let Some(o) = outcome { hasher.update(o.as_bytes()); }
        if let Some(r) = reason { hasher.update(r.as_bytes()); }
        if let Some(rm) = remediation { hasher.update(rm.as_bytes()); }
        hasher.update(args_hash.as_bytes());
        if let Some(oh) = output_hash { hasher.update(oh.as_bytes()); }
        if let Some(dur) = duration_ms { hasher.update(dur.to_be_bytes()); }
        hasher.update(prev_hash.as_bytes());
        
        let event_hash = hex::encode(hasher.finalize());

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO audit_logs (
                timestamp, actor, path, event_type, tool_name, 
                decision, outcome, reason, remediation, 
                args_hash, output_hash, duration_ms, prev_hash, event_hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                timestamp, actor, path, event_type, tool_name,
                decision, outcome, reason, remediation,
                args_hash, output_hash, duration_ms, prev_hash, event_hash
            ],
        )?;

        Ok(event_hash)
    }

    pub fn query_events(
        &self,
        start_time: Option<i64>,
        end_time: Option<i64>,
        path: Option<&str>,
        tool_name: Option<&str>,
        decision: Option<&str>,
        outcome: Option<&str>,
    ) -> Result<Vec<AuditEvent>> {
        let mut query = "SELECT id, timestamp, actor, path, event_type, tool_name, 
                        decision, outcome, reason, remediation, 
                        args_hash, output_hash, duration_ms, prev_hash, event_hash 
                        FROM audit_logs WHERE 1=1".to_string();
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(start) = start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start));
        }
        if let Some(end) = end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end));
        }
        if let Some(p) = path {
            query.push_str(" AND path = ?");
            params.push(Box::new(p.to_string()));
        }
        if let Some(t) = tool_name {
            query.push_str(" AND tool_name = ?");
            params.push(Box::new(t.to_string()));
        }
        if let Some(d) = decision {
            query.push_str(" AND decision = ?");
            params.push(Box::new(d.to_string()));
        }
        if let Some(o) = outcome {
            query.push_str(" AND outcome = ?");
            params.push(Box::new(o.to_string()));
        }

        query.push_str(" ORDER BY timestamp ASC");

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        
        // Convert Vec<Box<dyn ToSql>> to a slice of &dyn ToSql
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let rows = stmt.query_map(&params_refs[..], |row| {
            Ok(AuditEvent {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                actor: row.get(2)?,
                path: row.get(3)?,
                event_type: row.get(4)?,
                tool_name: row.get(5)?,
                decision: row.get(6)?,
                outcome: row.get(7)?,
                reason: row.get(8)?,
                remediation: row.get(9)?,
                args_hash: row.get(10)?,
                output_hash: row.get(11)?,
                duration_ms: row.get(12)?,
                prev_hash: row.get(13)?,
                event_hash: row.get(14)?,
            })
        })?;

        let mut events = Vec::new();
        for event in rows {
            events.push(event?);
        }
        Ok(events)
    }

    pub fn verify_chain(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT timestamp, actor, path, event_type, tool_name, 
                                        decision, outcome, reason, remediation, 
                                        args_hash, output_hash, duration_ms, prev_hash, event_hash 
                                        FROM audit_logs ORDER BY id ASC")?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, Option<i64>>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
            ))
        })?;

        let mut expected_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for row_result in rows {
            let (ts, actor, path, etype, tool, dec, out, reason, rem, ahash, ohash, dur, prev, hash) = row_result?;
            
            if prev != expected_prev_hash {
                return Ok(false);
            }

            let mut hasher = Sha256::new();
            hasher.update(ts.to_be_bytes());
            hasher.update(actor.as_bytes());
            hasher.update(path.as_bytes());
            hasher.update(etype.as_bytes());
            hasher.update(tool.as_bytes());
            if let Some(d) = dec { hasher.update(d.as_bytes()); }
            if let Some(o) = out { hasher.update(o.as_bytes()); }
            if let Some(r) = reason { hasher.update(r.as_bytes()); }
            if let Some(rm) = rem { hasher.update(rm.as_bytes()); }
            hasher.update(ahash.as_bytes());
            if let Some(oh) = ohash { hasher.update(oh.as_bytes()); }
            if let Some(d) = dur { hasher.update(d.to_be_bytes()); }
            hasher.update(prev.as_bytes());
            
            let calculated_hash = hex::encode(hasher.finalize());
            if calculated_hash != hash {
                return Ok(false);
            }
            
            expected_prev_hash = hash;
        }

        Ok(true)
    }
}

pub fn hash_payload(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

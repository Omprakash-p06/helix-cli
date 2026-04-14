#[path = "../src/audit.rs"]
mod audit;

use audit::AuditStore;
use std::fs;

#[test]
fn test_audit_append_and_query() {
    let db_path = "test_audit.db";
    let _ = fs::remove_file(db_path);
    
    let store = AuditStore::new(db_path).expect("Failed to create AuditStore");
    
    let hash1 = store.append_event(
        "user", "terminal", "policy", "read_file",
        Some("allow"), None, None, None,
        "args_hash_1", None, None
    ).expect("Failed to append event 1");
    
    let hash2 = store.append_event(
        "agent", "terminal", "execution", "read_file",
        None, Some("success"), None, None,
        "args_hash_1", Some("output_hash_1"), Some(150)
    ).expect("Failed to append event 2");
    
    assert_ne!(hash1, hash2);
    
    let events = store.query_events(None, None, Some("terminal"), Some("read_file"), None, None)
        .expect("Failed to query events");
    
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "policy");
    assert_eq!(events[1].event_type, "execution");
    assert_eq!(events[1].duration_ms, Some(150));
    
    let verified = store.verify_chain().expect("Failed to verify chain");
    assert!(verified);
    
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_audit_query_filters() {
    let db_path = "test_audit_filters.db";
    let _ = fs::remove_file(db_path);
    
    let store = AuditStore::new(db_path).expect("Failed to create AuditStore");
    
    store.append_event("user", "terminal", "policy", "tool_a", Some("allow"), None, None, None, "h", None, None).unwrap();
    store.append_event("user", "web", "policy", "tool_b", Some("deny"), None, None, None, "h", None, None).unwrap();
    
    let web_events = store.query_events(None, None, Some("web"), None, None, None).unwrap();
    assert_eq!(web_events.len(), 1);
    assert_eq!(web_events[0].tool_name, "tool_b");
    
    let deny_events = store.query_events(None, None, None, None, Some("deny"), None).unwrap();
    assert_eq!(deny_events.len(), 1);
    assert_eq!(deny_events[0].tool_name, "tool_b");
    
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_audit_tamper_detection() {
    let db_path = "test_audit_tamper.db";
    let _ = fs::remove_file(db_path);
    
    let store = AuditStore::new(db_path).expect("Failed to create AuditStore");
    
    store.append_event("user", "terminal", "policy", "tool_a", Some("allow"), None, None, None, "h", None, None).unwrap();
    store.append_event("user", "terminal", "policy", "tool_b", Some("allow"), None, None, None, "h", None, None).unwrap();
    
    assert!(store.verify_chain().unwrap());
    
    // Manually tamper with the database
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.execute("UPDATE audit_logs SET tool_name = 'tampered' WHERE id = 1", []).unwrap();
    
    assert!(!store.verify_chain().unwrap());
    
    let _ = fs::remove_file(db_path);
}

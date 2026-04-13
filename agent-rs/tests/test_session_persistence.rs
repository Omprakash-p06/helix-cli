use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[path = "../src/types.rs"]
mod types;

mod session {
    include!("../src/session.rs");
}

use session::{has_latest, latest_path, load_latest, load_named, save_latest, save_named};
use types::ChatMessage;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("helix_session_test_{}_{}", std::process::id(), name))
}

fn sample_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "system".to_string(),
            content: Some("you are helix".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: Some("hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: Some("hi".to_string()),
            tool_calls: Some(vec![json!({"id": "tool-1"})]),
            tool_call_id: None,
            name: None,
        },
    ]
}

#[test]
fn latest_round_trip_persists_messages() {
    let _guard = env_lock().lock().expect("env lock");
    let dir = test_dir("roundtrip");
    let _ = fs::remove_dir_all(&dir);
    unsafe { std::env::set_var("HELIX_SESSION_DIR", &dir); }

    let messages = sample_messages();
    save_latest("model-a", "agentic", "tui", &messages).expect("save latest should succeed");

    assert!(has_latest());

    let loaded = load_latest().expect("load latest should succeed");
    assert_eq!(loaded.model_name, "model-a");
    assert_eq!(loaded.exec_mode, "agentic");
    assert_eq!(loaded.ui_mode, "tui");
    assert_eq!(loaded.messages.len(), messages.len());
    assert_eq!(loaded.messages[1].content.as_deref(), Some("hello"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn malformed_latest_file_returns_error() {
    let _guard = env_lock().lock().expect("env lock");
    let dir = test_dir("malformed");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");
    unsafe { std::env::set_var("HELIX_SESSION_DIR", &dir); }

    let path = latest_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&path, "{not-json").expect("write malformed");

    let result = load_latest();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap_or_default()
        .to_lowercase()
        .contains("malformed"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_and_load_named_session_works() {
    let _guard = env_lock().lock().expect("env lock");
    let dir = test_dir("named");
    let _ = fs::remove_dir_all(&dir);
    unsafe { std::env::set_var("HELIX_SESSION_DIR", &dir); }

    let messages = sample_messages();
    save_named("alpha_1", "model-b", "chat", "tui", &messages).expect("save named should succeed");

    let loaded = load_named("alpha_1").expect("load named should succeed");
    assert_eq!(loaded.model_name, "model-b");
    assert_eq!(loaded.exec_mode, "chat");
    assert_eq!(loaded.messages[0].role, "system");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_named_rejects_traversal_names() {
    let _guard = env_lock().lock().expect("env lock");
    let dir = test_dir("sanitize");
    let _ = fs::remove_dir_all(&dir);
    unsafe { std::env::set_var("HELIX_SESSION_DIR", &dir); }

    let messages = sample_messages();
    let result = save_named("../danger", "model", "chat", "tui", &messages);
    assert!(result.is_err());

    let _ = fs::remove_dir_all(&dir);
}

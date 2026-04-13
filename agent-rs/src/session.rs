use crate::types::ChatMessage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SESSION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEnvelope {
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub model_name: String,
    pub exec_mode: String,
    pub ui_mode: String,
    pub messages: Vec<ChatMessage>,
}

impl SessionEnvelope {
    pub fn new(model_name: &str, exec_mode: &str, ui_mode: &str, messages: &[ChatMessage]) -> Self {
        let now = unix_now();
        Self {
            version: SESSION_SCHEMA_VERSION,
            created_at: now,
            updated_at: now,
            model_name: model_name.to_string(),
            exec_mode: exec_mode.to_string(),
            ui_mode: ui_mode.to_string(),
            messages: messages.to_vec(),
        }
    }
}

pub fn latest_path() -> PathBuf {
    session_dir().join("session.latest.json")
}

pub fn named_path(name: &str) -> Option<PathBuf> {
    sanitize_name(name).map(|clean| session_dir().join(format!("session.{}.json", clean)))
}

pub fn has_latest() -> bool {
    latest_path().exists()
}

pub fn save_latest(model_name: &str, exec_mode: &str, ui_mode: &str, messages: &[ChatMessage]) -> Result<PathBuf, String> {
    let path = latest_path();
    save_to_path(&path, model_name, exec_mode, ui_mode, messages)?;
    Ok(path)
}

pub fn save_named(name: &str, model_name: &str, exec_mode: &str, ui_mode: &str, messages: &[ChatMessage]) -> Result<PathBuf, String> {
    let path = named_path(name).ok_or_else(|| "Invalid session name. Use letters, numbers, '-', '_' only.".to_string())?;
    save_to_path(&path, model_name, exec_mode, ui_mode, messages)?;
    Ok(path)
}

pub fn load_latest() -> Result<SessionEnvelope, String> {
    load_from_path(&latest_path())
}

pub fn load_named(name: &str) -> Result<SessionEnvelope, String> {
    let path = named_path(name).ok_or_else(|| "Invalid session name. Use letters, numbers, '-', '_' only.".to_string())?;
    load_from_path(&path)
}

fn save_to_path(path: &Path, model_name: &str, exec_mode: &str, ui_mode: &str, messages: &[ChatMessage]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create session directory: {}", e))?;
    }

    let mut envelope = SessionEnvelope::new(model_name, exec_mode, ui_mode, messages);
    envelope.updated_at = unix_now();
    let payload = serde_json::to_vec_pretty(&envelope)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;

    let tmp_path = path.with_extension("json.tmp");
    let mut file = fs::File::create(&tmp_path)
        .map_err(|e| format!("Failed to create temporary session file: {}", e))?;
    file.write_all(&payload)
        .map_err(|e| format!("Failed to write session payload: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync session payload: {}", e))?;

    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        format!("Failed to finalize session save: {}", e)
    })
}

fn load_from_path(path: &Path) -> Result<SessionEnvelope, String> {
    if !path.exists() {
        return Err(format!("Session file not found: {}", path.display()));
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read session file {}: {}", path.display(), e))?;

    let envelope: SessionEnvelope = serde_json::from_str(&raw)
        .map_err(|e| format!("Session file is malformed: {}", e))?;

    if envelope.version != SESSION_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported session schema version {} (expected {}).",
            envelope.version, SESSION_SCHEMA_VERSION
        ));
    }

    Ok(envelope)
}

fn session_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("HELIX_SESSION_DIR") {
        return PathBuf::from(dir);
    }

    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        return PathBuf::from(home).join(".helix").join("sessions");
    }

    PathBuf::from(".helix").join("sessions")
}

fn sanitize_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return None;
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

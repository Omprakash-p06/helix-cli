use std::time::SystemTime;

pub use super::{TuiAction, TuiEvent};

// ── Layout ───────────────────────────────────────────────────────────────

/// Layout mode for the TUI shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiLayoutMode {
    /// Default 70/30 conversation/sidebar split.
    Wide,
    /// Narrower sidebar, more space for conversation.
    Compact,
}

impl TuiLayoutMode {
    /// Parse a layout mode string, defaulting to Wide.
    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "compact" => TuiLayoutMode::Compact,
            _ => TuiLayoutMode::Wide,
        }
    }

    /// Sidebar percentage width for this layout mode.
    pub fn sidebar_percent(&self) -> u16 {
        match self {
            TuiLayoutMode::Wide => 30,
            TuiLayoutMode::Compact => 20,
        }
    }
}

// ── Connection ───────────────────────────────────────────────────────────

/// Connection state for the backend model server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "connected"),
            ConnectionState::Connecting => write!(f, "connecting"),
            ConnectionState::Disconnected => write!(f, "disconnected"),
        }
    }
}

// ── Context Files ────────────────────────────────────────────────────────

/// A file entry in the LLM's context window.
#[derive(Debug, Clone)]
pub struct ContextFileEntry {
    pub path: String,
    pub display_name: String,
    pub token_count: usize,
}

// ── Session Info ─────────────────────────────────────────────────────────

/// Session metadata for the status bar / sidebar.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub model_name: String,
    pub exec_mode: String,
    pub started_at: SystemTime,
}

// ── Status Banner ────────────────────────────────────────────────────────

/// Severity level for status banners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Warning,
    Error,
}

/// A dismissible banner shown at the top of the conversation area.
#[derive(Debug, Clone)]
pub struct StatusBanner {
    pub level: StatusLevel,
    pub message: String,
}

// ── Message Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
}

// ── Channel Aliases ──────────────────────────────────────────────────────

pub type ActionSender = tokio::sync::mpsc::UnboundedSender<TuiAction>;
pub type ActionReceiver = tokio::sync::mpsc::UnboundedReceiver<TuiAction>;
pub type EventSender = tokio::sync::mpsc::UnboundedSender<TuiEvent>;
pub type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<TuiEvent>;

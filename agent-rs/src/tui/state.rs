use std::time::SystemTime;

use super::api::{ConnectionState, ContextFileEntry, SessionInfo, StatusBanner, TuiLayoutMode};
use super::commands::{Command, default_commands};

// ── Theme Name ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    Nord,
    Gruvbox,
    Custom,
}

impl ThemeName {
    pub fn next(&self) -> Self {
        match self {
            ThemeName::Dark => ThemeName::Light,
            ThemeName::Light => ThemeName::Nord,
            ThemeName::Nord => ThemeName::Gruvbox,
            ThemeName::Gruvbox => ThemeName::Custom,
            ThemeName::Custom => ThemeName::Dark,
        }
    }
}

// ── Sidebar Tab ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    ContextFiles,
    ToolTimeline,
}

impl SidebarTab {
    pub fn toggle(&self) -> Self {
        match self {
            SidebarTab::ContextFiles => SidebarTab::ToolTimeline,
            SidebarTab::ToolTimeline => SidebarTab::ContextFiles,
        }
    }
}

// ── Tool Timeline ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ToolTimelineEntry {
    pub id: u64,
    pub name: String,
    pub args_preview: String,
    pub status: ToolTimelineStatus,
    pub duration_ms: Option<u64>,
    pub started_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolTimelineStatus {
    Running,
    Completed,
    Failed,
}

// ── Command Palette ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommandPaletteState {
    pub visible: bool,
    pub commands: Vec<Command>,
    pub selected_index: usize,
    pub filter: String,
}

// ── Main TUI State ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TuiState {
    // Layout
    pub layout_mode: TuiLayoutMode,
    pub sidebar_visible: bool,
    pub sidebar_tab: SidebarTab,

    // Session & connection
    pub session_info: Option<SessionInfo>,
    pub connection_state: ConnectionState,

    // Context tracking
    pub context_files: Vec<ContextFileEntry>,
    pub token_count: usize,
    pub max_tokens: usize,

    // Tool timeline
    pub tool_timeline: Vec<ToolTimelineEntry>,

    // Status
    pub status_banner: Option<StatusBanner>,

    // Command palette
    pub command_palette: CommandPaletteState,

    // Theme
    pub current_theme: ThemeName,

    // Scroll / new messages
    pub pending_new_messages: usize,

    // Collapsed tool blocks (by tool id)
    pub collapsed_tools: Vec<u64>,

    // Selected sidebar item index
    pub sidebar_selected: usize,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            layout_mode: TuiLayoutMode::Wide,
            sidebar_visible: true,
            sidebar_tab: SidebarTab::ContextFiles,
            session_info: None,
            connection_state: ConnectionState::Disconnected,
            context_files: Vec::new(),
            token_count: 0,
            max_tokens: 8192,
            tool_timeline: Vec::new(),
            status_banner: None,
            command_palette: CommandPaletteState {
                visible: false,
                commands: default_commands(),
                selected_index: 0,
                filter: String::new(),
            },
            current_theme: ThemeName::Dark,
            pending_new_messages: 0,
            collapsed_tools: Vec::new(),
            sidebar_selected: 0,
        }
    }

    pub fn current_theme(&self) -> super::themes::ThemeColorSet {
        self.current_theme.colors()
    }

    pub fn toggle_tool_collapsed(&mut self, id: u64) {
        if let Some(pos) = self.collapsed_tools.iter().position(|&x| x == id) {
            self.collapsed_tools.remove(pos);
        } else {
            self.collapsed_tools.push(id);
        }
    }

    pub fn is_tool_collapsed(&self, id: u64) -> bool {
        self.collapsed_tools.contains(&id)
    }
}

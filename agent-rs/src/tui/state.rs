use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use ratatui::style::Color;

use super::api::{ConnectionState, ContextFileEntry, SessionInfo, StatusBanner, TuiLayoutMode};
use super::commands::{Command, default_commands};
use crate::security::policy::TrustLevel;

const TOOL_FLASH_DURATION: Duration = Duration::from_millis(450);
const MESSAGE_FADE_DURATION: Duration = Duration::from_millis(150);
const THROBBER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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
    pub started_at: Instant,
    pub status_changed_at: Instant,
    pub flash_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolTimelineStatus {
    Running,
    Completed,
    Failed,
}

impl ToolTimelineEntry {
    pub fn new(id: u64, name: String, args_preview: String) -> Self {
        let now = Instant::now();
        Self {
            id,
            name,
            args_preview,
            status: ToolTimelineStatus::Running,
            duration_ms: None,
            started_at: now,
            status_changed_at: now,
            flash_color: None,
        }
    }

    pub fn mark_finished(&mut self, success: bool) {
        self.status = if success {
            ToolTimelineStatus::Completed
        } else {
            ToolTimelineStatus::Failed
        };
        self.duration_ms = Some(self.started_at.elapsed().as_millis() as u64);
        self.status_changed_at = Instant::now();
        self.flash_color = Some(if success { Color::Green } else { Color::Red });
    }

    pub fn active_flash_color(&self) -> Option<Color> {
        self.flash_color
            .filter(|_| self.status_changed_at.elapsed() <= TOOL_FLASH_DURATION)
    }
}

// ── Animation State ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThrobberState {
    frame_index: usize,
}

impl Default for ThrobberState {
    fn default() -> Self {
        Self::new()
    }
}

impl ThrobberState {
    pub fn new() -> Self {
        Self { frame_index: 0 }
    }

    pub fn calc_next(&mut self) {
        self.frame_index = (self.frame_index + 1) % THROBBER_FRAMES.len();
    }

    pub fn current_frame(&self) -> &'static str {
        THROBBER_FRAMES[self.frame_index]
    }

    pub fn frame_index(&self) -> usize {
        self.frame_index
    }
}

#[derive(Debug, Clone)]
pub struct MessageFade {
    remaining: Duration,
}

impl MessageFade {
    fn new() -> Self {
        Self {
            remaining: MESSAGE_FADE_DURATION,
        }
    }

    pub fn opacity(&self) -> f32 {
        let progress = 1.0
            - (self.remaining.as_secs_f32() / MESSAGE_FADE_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        progress.clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ToolEffectState {
    pulse_phase: f32,
    flash_success: Option<bool>,
    flash_remaining: Duration,
}

impl ToolEffectState {
    fn new() -> Self {
        Self {
            pulse_phase: 0.0,
            flash_success: None,
            flash_remaining: Duration::ZERO,
        }
    }

    pub fn pulse_intensity(&self) -> f32 {
        ((self.pulse_phase.sin() + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    pub fn flash_color(&self) -> Option<Color> {
        self.flash_success.map(|success| {
            if success {
                Color::Green
            } else {
                Color::Red
            }
        })
    }

    pub fn flash_intensity(&self) -> f32 {
        if TOOL_FLASH_DURATION.is_zero() {
            return 0.0;
        }
        (self.flash_remaining.as_secs_f32() / TOOL_FLASH_DURATION.as_secs_f32()).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub animated_tokens: f32,
    pub throbber_state: ThrobberState,
    message_fades: BTreeMap<usize, MessageFade>,
    tool_effects: BTreeMap<u64, ToolEffectState>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            animated_tokens: 0.0,
            throbber_state: ThrobberState::new(),
            message_fades: BTreeMap::new(),
            tool_effects: BTreeMap::new(),
        }
    }

    pub fn on_tick(&mut self, show_throbber: bool, current_tokens: usize, dt: Duration) {
        if show_throbber {
            self.throbber_state.calc_next();
        }

        self.animated_tokens = interpolate_animated_value(self.animated_tokens, current_tokens as f32);

        let pulse_step = dt.as_secs_f32() * 8.0;
        for effect in self.tool_effects.values_mut() {
            effect.pulse_phase = (effect.pulse_phase + pulse_step) % std::f32::consts::TAU;
            effect.flash_remaining = effect.flash_remaining.saturating_sub(dt);
            if effect.flash_remaining.is_zero() {
                effect.flash_success = None;
            }
        }

        self.message_fades.retain(|_, fade| {
            fade.remaining = fade.remaining.saturating_sub(dt);
            !fade.remaining.is_zero()
        });
    }

    pub fn register_message_fade(&mut self, message_index: usize) {
        self.message_fades.insert(message_index, MessageFade::new());
    }

    pub fn message_opacity(&self, message_index: usize) -> f32 {
        self.message_fades
            .get(&message_index)
            .map(MessageFade::opacity)
            .unwrap_or(1.0)
    }

    pub fn register_tool_running(&mut self, tool_id: u64) {
        self.tool_effects
            .entry(tool_id)
            .or_insert_with(ToolEffectState::new);
    }

    pub fn register_tool_finished(&mut self, tool_id: u64, success: bool) {
        let effect = self
            .tool_effects
            .entry(tool_id)
            .or_insert_with(ToolEffectState::new);
        effect.flash_success = Some(success);
        effect.flash_remaining = TOOL_FLASH_DURATION;
    }

    pub fn tool_effect(&self, tool_id: u64) -> Option<&ToolEffectState> {
        self.tool_effects.get(&tool_id)
    }
}

pub fn interpolate_animated_value(current: f32, target: f32) -> f32 {
    let delta = (target - current) * 0.2;
    let next = current + delta;
    if (next - target).abs() < 0.5 {
        target
    } else {
        next
    }
}

// ── Settings Modal ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Interface,
    Security,
}

impl SettingsCategory {
    pub fn all() -> [SettingsCategory; 3] {
        [
            SettingsCategory::General,
            SettingsCategory::Interface,
            SettingsCategory::Security,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Interface => "Interface",
            SettingsCategory::Security => "Security",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionTierOption {
    ReadOnly,
    WorkspaceWrite,
    FullExec,
}

impl PermissionTierOption {
    pub fn next(self) -> Self {
        match self {
            PermissionTierOption::ReadOnly => PermissionTierOption::WorkspaceWrite,
            PermissionTierOption::WorkspaceWrite => PermissionTierOption::FullExec,
            PermissionTierOption::FullExec => PermissionTierOption::ReadOnly,
        }
    }

    pub fn to_trust_level(self) -> TrustLevel {
        match self {
            PermissionTierOption::ReadOnly => TrustLevel::Safe,
            PermissionTierOption::WorkspaceWrite => TrustLevel::Auto,
            PermissionTierOption::FullExec => TrustLevel::Full,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PermissionTierOption::ReadOnly => "Safe",
            PermissionTierOption::WorkspaceWrite => "Auto",
            PermissionTierOption::FullExec => "Full",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub active_modal: bool,
    pub selected_item: usize,
    pub selected_detail: usize,
    pub model_name: String,
    pub exec_mode: String,
    pub token_limit_input: String,
    pub theme: ThemeName,
    pub sidebar_visible: bool,
    pub permission_tier: PermissionTierOption,
    pub trust_level: TrustLevel,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            active_modal: false,
            selected_item: 0,
            selected_detail: 0,
            model_name: "gpt-oss-20b".to_string(),
            exec_mode: "chat".to_string(),
            token_limit_input: "8192".to_string(),
            theme: ThemeName::Dark,
            sidebar_visible: true,
            permission_tier: PermissionTierOption::WorkspaceWrite,
            trust_level: TrustLevel::Auto,
        }
    }

    pub fn selected_category(&self) -> SettingsCategory {
        SettingsCategory::all()[self.selected_item.min(SettingsCategory::all().len() - 1)]
    }

    pub fn detail_count(&self) -> usize {
        match self.selected_category() {
            SettingsCategory::General => 3,
            SettingsCategory::Interface => 2,
            SettingsCategory::Security => 1,
        }
    }

    pub fn next_category(&mut self) {
        self.selected_item = (self.selected_item + 1).min(SettingsCategory::all().len() - 1);
        self.selected_detail = self.selected_detail.min(self.detail_count().saturating_sub(1));
    }

    pub fn prev_category(&mut self) {
        self.selected_item = self.selected_item.saturating_sub(1);
        self.selected_detail = self.selected_detail.min(self.detail_count().saturating_sub(1));
    }

    pub fn next_detail(&mut self) {
        self.selected_detail = (self.selected_detail + 1).min(self.detail_count().saturating_sub(1));
    }

    pub fn prev_detail(&mut self) {
        self.selected_detail = self.selected_detail.saturating_sub(1);
    }
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
    pub layout_mode: TuiLayoutMode,
    pub sidebar_visible: bool,
    pub sidebar_tab: SidebarTab,
    pub session_info: Option<SessionInfo>,
    pub connection_state: ConnectionState,
    pub context_files: Vec<ContextFileEntry>,
    pub token_count: usize,
    pub max_tokens: usize,
    pub tool_timeline: Vec<ToolTimelineEntry>,
    pub status_banner: Option<StatusBanner>,
    pub command_palette: CommandPaletteState,
    pub current_theme: ThemeName,
    pub pending_new_messages: usize,
    pub collapsed_tools: Vec<u64>,
    pub sidebar_selected: usize,
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{AnimationState, SettingsState, interpolate_animated_value};

    #[test]
    fn test_throbber_tick() {
        let mut animation = AnimationState::new();
        let before = animation.throbber_state.frame_index();
        animation.on_tick(true, 0, Duration::from_millis(33));
        let after = animation.throbber_state.frame_index();
        assert_ne!(before, after);
    }

    #[test]
    fn test_gauge_interpolation() {
        let next = interpolate_animated_value(0.0, 100.0);
        assert!(next > 0.0);
        assert!(next < 100.0);
    }

    #[test]
    fn test_message_fade_progression() {
        let mut animation = AnimationState::new();
        animation.register_message_fade(0);

        let opacity_initial = animation.message_opacity(0);
        assert!(opacity_initial < 0.01); // Starts at 0.0

        animation.on_tick(false, 0, Duration::from_millis(75));
        let opacity_mid = animation.message_opacity(0);
        assert!(opacity_mid > 0.4 && opacity_mid < 0.6); // ~0.5

        animation.on_tick(false, 0, Duration::from_millis(75));
        let opacity_final = animation.message_opacity(0);
        assert!(opacity_final > 0.99); // Finished
    }

    #[test]
    fn test_tool_pulse_animation() {
        let mut animation = AnimationState::new();
        animation.register_tool_running(1);

        let initial_pulse = animation.tool_effect(1).unwrap().pulse_intensity();

        // Tick to advance pulse phase
        animation.on_tick(false, 0, Duration::from_millis(100));
        let next_pulse = animation.tool_effect(1).unwrap().pulse_intensity();

        assert_ne!(initial_pulse, next_pulse);
    }

    #[test]
    fn test_settings_state_navigation() {
        let mut settings = SettingsState::new();
        assert_eq!(settings.selected_item, 0);
        settings.next_category();
        assert_eq!(settings.selected_item, 1);
        settings.prev_category();
        assert_eq!(settings.selected_item, 0);
    }
}

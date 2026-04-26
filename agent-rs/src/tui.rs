#![allow(dead_code)]
pub mod api;
pub mod approval;
pub mod commands;
pub mod events;
pub mod state;
pub mod themes;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap, block::Title},
};
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

// ── Slash commands for autocomplete ──────────────────────────────────────────

const SLASH_COMMANDS: &[&str] = &[
    "/help", "/quit", "/exit", "/clear", "/save", "/load", "/resume", "/history", "/model", "/mode",
];

// ── Application state ────────────────────────────────────────────────────────

/// Messages sent from the TUI to the orchestrator.
#[derive(Debug)]
pub enum TuiAction {
    /// User submitted a prompt.
    Submit(String),
    /// User requested quit.
    Quit,
    /// User requested to interrupt generation.
    Interrupt,
    /// System command issued.
    SystemCommand(String),
    /// Cancel current input.
    CancelInput,
    /// Start a new chat session.
    NewChat,
    /// Clear conversation history.
    ClearHistory,
    /// Toggle sidebar visibility.
    ToggleSidebar,
    /// Cycle to next theme.
    ToggleTheme,
    /// Scroll conversation up.
    ScrollUp,
    /// Scroll conversation down.
    ScrollDown,
    /// Scroll conversation up by half page.
    ScrollPageUp,
    /// Scroll conversation down by half page.
    ScrollPageDown,
    /// Show help overlay.
    ShowHelp,
    /// Open the command palette.
    OpenCommandPalette,
    /// Close the command palette.
    CloseCommandPalette,
    /// Select a command from the palette by index.
    SelectCommand(usize),
    /// Open the settings modal.
    OpenSettings,
    /// Close the settings modal.
    CloseSettings,
    /// Set the TUI layout mode.
    SetLayout(api::TuiLayoutMode),
    /// Set a named theme.
    SetTheme(String),
    /// Open the system editor for multiline input.
    OpenEditor,
}

/// Information about a tool being invoked.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub arguments: String,
}

/// Result of a tool invocation.
#[derive(Debug, Clone)]
pub struct ToolResultInfo {
    pub name: String,
    pub output: String,
    pub success: bool,
}

/// Messages sent from the orchestrator to the TUI.
#[derive(Debug)]
pub enum TuiEvent {
    /// A batch of tokens streamed from the model (flushed every ~30ms).
    TokenChunk(String),
    /// A tool execution has started.
    ToolStart(ToolInfo),
    /// A tool execution has completed.
    ToolResult(ToolResultInfo),
    /// Model finished its response.
    ResponseDone,
    /// Status text for the status bar.
    Status(String),
    /// A precise status tick for metrics (like TTFT updates).
    StatusUpdate(String),
    /// System message (info/warning).
    SystemMessage(String),
    /// Signals the start of actual token generation (locks TTFT).
    GenerationStarted,
    // Visible heartbeat while model is streaming non-token/tool-only deltas.
    StreamingHeartbeat(String),
    /// Updates the token context HUD (current_tokens, max_tokens).
    ContextUpdate(usize, usize),
    /// Wipe visual UI memory.
    ClearHistory,
    /// Theme was changed.
    ThemeChanged(state::ThemeName),
    /// Full context snapshot for sidebar and status bar.
    ContextSnapshot {
        tokens_used: usize,
        max_tokens: usize,
        files: Vec<api::ContextFileEntry>,
        model_name: String,
        exec_mode: String,
        connection: api::ConnectionState,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Preview,
    Help,
    Settings,
}

/// A styled span within a chat entry.
#[derive(Debug, Clone)]
pub struct ChatSpan {
    pub text: String,
    pub style: Style,
}

/// A message in the chat history.
#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub role: String,
    pub content: String,
    /// Pre-parsed styled spans for rich rendering (think blocks, etc.).
    pub spans: Vec<ChatSpan>,
    pub tool_id: Option<u64>,
}

pub struct TuiApp {
    input: Input,
    input_mode: InputMode,
    multiline_buffer: Vec<String>,
    chat_history: Vec<ChatEntry>,
    /// Current streaming content (accumulated tokens).
    streaming_content: String,
    /// Pre-parsed spans for the current streaming content.
    streaming_spans: Vec<ChatSpan>,
    /// Whether we are currently inside a <think> block.
    in_think_block: bool,
    // Optional heartbeat line shown when generation has no visible token chunk yet.
    streaming_heartbeat: Option<String>,
    /// Whether the model is currently generating.
    is_generating: bool,
    /// Toggle visibility of thinking blocks (Ctrl+T).
    show_thoughts: bool,
    scroll_offset: u16,
    max_scroll_offset: u16,
    status_text: String,
    no_color: bool,
    command_history: Vec<String>,
    history_index: Option<usize>,
    ttft_start: Option<Instant>,
    ttft_locked: Option<Duration>,
    pub cwd: String,
    pub current_tokens: usize,
    pub max_tokens: usize,
    animations: state::AnimationState,
    // ── Phase 19/26 additions ──
    layout_mode: api::TuiLayoutMode,
    sidebar_visible: bool,
    sidebar_tab: state::SidebarTab,
    tool_timeline: Vec<state::ToolTimelineEntry>,
    collapsed_tools: Vec<u64>,
    model_name: String,
    exec_mode_label: String,
    connection_label: String,
    current_theme: state::ThemeName,
    settings: state::SettingsState,
    // GSD state tracking
    last_gsd_command: Option<String>,
    command_palette: state::CommandPaletteState,
}

impl TuiApp {
    pub fn new(layout_mode: api::TuiLayoutMode) -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        Self {
            input: Input::default(),
            input_mode: InputMode::Normal,
            multiline_buffer: Vec::new(),
            chat_history: Vec::new(),
            streaming_content: String::new(),
            streaming_spans: Vec::new(),
            in_think_block: false,
            streaming_heartbeat: None,
            is_generating: false,
            show_thoughts: true,
            scroll_offset: 0,
            max_scroll_offset: 0,
            status_text: String::from("Ready"),
            no_color,
            command_history: Vec::new(),
            history_index: None,
            ttft_start: None,
            ttft_locked: None,
            cwd: Self::get_formatted_cwd(),
            current_tokens: 0,
            max_tokens: 8192,
            animations: state::AnimationState::new(),
            layout_mode,
            sidebar_visible: true,
            sidebar_tab: state::SidebarTab::ContextFiles,
            tool_timeline: Vec::new(),
            collapsed_tools: Vec::new(),
            model_name: String::new(),
            exec_mode_label: String::new(),
            connection_label: String::from("connecting"),
            current_theme: state::ThemeName::Dark,
            settings: state::SettingsState::new(),
            last_gsd_command: None,
            command_palette: state::CommandPaletteState {
                visible: false,
                commands: commands::default_commands(),
                selected_index: 0,
                filter: String::new(),
            },
        }
    }

    fn get_formatted_cwd() -> String {
        if let Ok(path) = std::env::current_dir() {
            let mut path_str = path.to_string_lossy().to_string();
            if let Ok(home) = std::env::var("HOME")
                && path_str.starts_with(&home)
            {
                path_str = path_str.replacen(&home, "~", 1);
            }
            path_str
        } else {
            "~".to_string()
        }
    }

    /// Get ghost autocomplete suggestion based on current input.
    fn ghost_suggestion(&self) -> Option<String> {
        let current = self.input.value();
        
        // GSD Autofill logic: suggest next phase based on last command
        if current.is_empty() {
            if let Some(last) = &self.last_gsd_command {
                let next = if last.starts_with("/gsd-discuss-phase") {
                     "/gsd-plan-phase"
                } else if last.starts_with("/gsd-plan-phase") {
                     "/gsd-execute-phase"
                } else if last.starts_with("/gsd-execute-phase") {
                     "/gsd-verify-work"
                } else if last.starts_with("/gsd-verify-work") {
                     "/gsd-ship"
                } else {
                     return None;
                };
                return Some(next.to_string());
            }
            return None;
        }

        if current.starts_with('/') {
            // Check legacy SLASH_COMMANDS first
            for cmd in SLASH_COMMANDS {
                if cmd.starts_with(current) && *cmd != current {
                    return Some(cmd[current.len()..].to_string());
                }
            }
            // Then check dynamic registry
            let cmds = commands::default_commands();
            for cmd in cmds {
                if cmd.name.starts_with(current) && cmd.name != current {
                    return Some(cmd.name[current.len()..].to_string());
                }
            }
        }
        // History-based suggestion
        for prev in self.command_history.iter().rev() {
            if prev.starts_with(current) && prev != current {
                return Some(prev[current.len()..].to_string());
            }
        }
        None
    }

    fn submit_input(&mut self) -> Option<String> {
        let text = if self.multiline_buffer.is_empty() {
            self.input.value().to_string()
        } else {
            self.multiline_buffer.push(self.input.value().to_string());
            let full = self.multiline_buffer.join("\n");
            self.multiline_buffer.clear();
            full
        };

        let trimmed = text.trim().to_string();
        self.input.reset();

        if trimmed.is_empty() {
            return None;
        }

        // Track GSD commands for autofill
        if trimmed.starts_with("/gsd-") {
            self.last_gsd_command = Some(trimmed.clone());
        }

        self.command_history.push(trimmed.clone());
        self.history_index = None;
        self.push_chat_entry(
            "user",
            trimmed.clone(),
            vec![ChatSpan {
                text: trimmed.clone(),
                style: Style::default(),
            }],
            None,
        );
        // Follow the latest message after submission so new responses stay visible.
        self.scroll_offset = 0;
        Some(trimmed)
    }

    fn add_newline(&mut self) {
        self.multiline_buffer.push(self.input.value().to_string());
        self.input.reset();
    }

    /// Append a token chunk to streaming content, parsing <think>/</think> tags
    /// to build styled spans in real time.
    fn append_token_chunk(&mut self, chunk: &str) {
        self.streaming_heartbeat = None;
        self.streaming_content.push_str(chunk);

        // Re-parse the entire streaming content into spans.
        // This is simpler and correct vs. incremental parsing across chunk boundaries.
        self.streaming_spans.clear();
        self.in_think_block = false;

        let normal_style = if self.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::Blue)
        };
        let think_style = if self.no_color {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM)
        };

        let content = self.streaming_content.clone();
        let mut remaining = content.as_str();

        while !remaining.is_empty() {
            if self.in_think_block {
                if let Some(end_pos) = remaining.find("</think>") {
                    let think_text = &remaining[..end_pos];
                    if !think_text.is_empty() {
                        self.streaming_spans.push(ChatSpan {
                            text: think_text.to_string(),
                            style: think_style,
                        });
                    }
                    remaining = &remaining[(end_pos + 8)..]; // skip "</think>"
                    self.in_think_block = false;
                } else {
                    // Still inside think block, no closing tag yet
                    self.streaming_spans.push(ChatSpan {
                        text: remaining.to_string(),
                        style: think_style,
                    });
                    break;
                }
            } else {
                if let Some(start_pos) = remaining.find("<think>") {
                    let before = &remaining[..start_pos];
                    if !before.is_empty() {
                        self.streaming_spans.push(ChatSpan {
                            text: before.to_string(),
                            style: normal_style,
                        });
                    }
                    remaining = &remaining[(start_pos + 7)..]; // skip "<think>"
                    self.in_think_block = true;
                } else {
                    self.streaming_spans.push(ChatSpan {
                        text: remaining.to_string(),
                        style: normal_style,
                    });
                    break;
                }
            }
        }
    }

    /// Finalize the current streaming content into a ChatEntry with spans.
    fn finalize_streaming(&mut self) {
        if !self.streaming_content.is_empty() {
            // Strip <think>/</think> tags from the content string for plain-text fallback
            let clean_content = self
                .streaming_content
                .replace("<think>", "")
                .replace("</think>", "");

            self.push_chat_entry("assistant", clean_content, self.streaming_spans.clone(), None);
            self.streaming_content.clear();
            self.streaming_spans.clear();
            self.streaming_heartbeat = None;
            self.in_think_block = false;
        }
    }

    fn push_chat_entry(
        &mut self,
        role: impl Into<String>,
        content: String,
        spans: Vec<ChatSpan>,
        tool_id: Option<u64>,
    ) {
        self.chat_history.push(ChatEntry {
            role: role.into(),
            content,
            spans,
            tool_id,
        });
        let message_index = self.chat_history.len().saturating_sub(1);
        self.animations.register_message_fade(message_index);
    }

    fn is_thinking(&self) -> bool {
        self.is_generating && self.streaming_content.is_empty()
    }

    fn theme_colors(&self) -> themes::ThemeColorSet {
        self.current_theme.colors()
    }

    fn animated_tokens_display(&self) -> usize {
        self.animations
            .animated_tokens
            .round()
            .clamp(0.0, self.max_tokens as f32) as usize
    }

    fn on_tick(&mut self, dt: Duration) {
        self.animations
            .on_tick(self.is_thinking(), self.current_tokens, dt);
    }

    fn sync_settings_from_app(&mut self) {
        self.settings.model_name = if self.model_name.is_empty() {
            "gpt-oss-20b".to_string()
        } else {
            self.model_name.clone()
        };
        self.settings.exec_mode = if self.exec_mode_label.is_empty() {
            "chat".to_string()
        } else {
            self.exec_mode_label.clone()
        };
        self.settings.token_limit_input = self.max_tokens.to_string();
        self.settings.theme = self.current_theme;
        self.settings.sidebar_visible = self.sidebar_visible;
    }

    fn apply_settings_to_app(&mut self) {
        self.model_name = self.settings.model_name.clone();
        self.exec_mode_label = self.settings.exec_mode.clone();
        if let Ok(limit) = self.settings.token_limit_input.parse::<usize>() {
            self.max_tokens = limit.max(1);
        }
        self.current_theme = self.settings.theme;
        self.sidebar_visible = self.settings.sidebar_visible;
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

// ── Drawing ──────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let size = frame.size();
    // Update command palette state based on current input
    let input_val = app.input.value();
    if input_val.starts_with('/') {
        app.command_palette.visible = true;
        app.command_palette.filter = input_val.to_string();
        app.command_palette.commands = commands::filter_commands(&commands::default_commands(), &app.command_palette.filter);
        if app.command_palette.selected_index >= app.command_palette.commands.len() {
            app.command_palette.selected_index = 0;
        }
    } else {
        app.command_palette.visible = false;
    }

    // Calculate dynamic input height
    let input_width_est = size.width.saturating_sub(4).max(1);
    let mut input_lines = 0;
    for line in &app.multiline_buffer {
        input_lines += 1 + (line.len() as u16 / input_width_est);
    }
    input_lines += 1 + (app.input.value().len() as u16 / input_width_est);
    let dynamic_input_height = (input_lines + 2).clamp(3, 10);

    // Vertical layout: [content area] [input area] [status bar]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),                       // content area (chat + optional sidebar)
            Constraint::Length(dynamic_input_height), // input area
            Constraint::Length(1),                    // status bar
        ])
        .split(size);

    // Top row: conversation + optional sidebar
    if app.sidebar_visible && size.width > 60 {
        let sidebar_pct = app.layout_mode.sidebar_percent();
        let main_pct = 100 - sidebar_pct;
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(main_pct),
                Constraint::Percentage(sidebar_pct),
            ])
            .split(chunks[0]);

        draw_chat_area(frame, app, horizontal[0]);
        draw_sidebar(frame, app, horizontal[1]);
    } else {
        draw_chat_area(frame, app, chunks[0]);
    }

    draw_input_area(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);

    // Draw preview overlay if in preview mode
    if app.input_mode == InputMode::Preview {
        draw_preview_overlay(frame, app, size);
    } else if app.input_mode == InputMode::Help {
        draw_help_overlay(frame, app, size);
    } else if app.input_mode == InputMode::Settings {
        draw_settings_modal(frame, app, size);
    }
}

/// Render the sidebar with context files and session info.
fn draw_sidebar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let theme = app.theme_colors();

    let header_style = if app.no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.title)
            .add_modifier(Modifier::BOLD)
    };
    let body_style = if app.no_color {
        Style::default()
    } else {
        Style::default().fg(theme.foreground)
    };

    // Sidebar Tabs
    let tab_text = match app.sidebar_tab {
        state::SidebarTab::ContextFiles => " [Files]  Timeline ",
        state::SidebarTab::ToolTimeline => "  Files  [Timeline] ",
    };
    lines.push(Line::from(Span::styled(tab_text, header_style)));
    lines.push(Line::from(""));

    match app.sidebar_tab {
        state::SidebarTab::ContextFiles => {
            // Section: Session Info
            lines.push(Line::from(Span::styled("📊 Session", header_style)));
            if !app.model_name.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  model: {}", app.model_name),
                    body_style,
                )));
            }
            if !app.exec_mode_label.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  mode: {}", app.exec_mode_label),
                    body_style,
                )));
            }
            lines.push(Line::from(Span::styled(
                format!("  {}", app.connection_label),
                body_style,
            )));
            lines.push(Line::from(""));

            // Section: Token Usage
            lines.push(Line::from(Span::styled("📈 Tokens", header_style)));
            let animated_tokens = app.animated_tokens_display();
            let ratio = if app.max_tokens > 0 {
                (animated_tokens as f64) / (app.max_tokens as f64)
            } else {
                0.0
            };
            let filled = (ratio * 10.0).round() as usize;
            let empty = 10usize.saturating_sub(filled);
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
            lines.push(Line::from(Span::styled(
                format!("  [{}/{}]", animated_tokens, app.max_tokens),
                body_style,
            )));
            lines.push(Line::from(Span::styled(format!("  {}", bar), body_style)));
            lines.push(Line::from(""));
        }
        state::SidebarTab::ToolTimeline => {
            lines.push(Line::from(Span::styled("🔧 Tool Timeline", header_style)));
            if app.tool_timeline.is_empty() {
                lines.push(Line::from(Span::styled("  (no tools run)", body_style)));
            } else {
                for entry in app.tool_timeline.iter().rev().take(15) {
                    let (icon, base_color) = match entry.status {
                        state::ToolTimelineStatus::Running => ("⚙", Color::Cyan),
                        state::ToolTimelineStatus::Completed => ("✓", Color::Green),
                        state::ToolTimelineStatus::Failed => ("✗", Color::Red),
                    };
                    let effect = app.animations.tool_effect(entry.id);
                    let color = effect
                        .and_then(|effect| effect.flash_color())
                        .or_else(|| entry.active_flash_color())
                        .unwrap_or_else(|| {
                            if entry.status == state::ToolTimelineStatus::Running
                                && effect.map(|effect| effect.pulse_intensity()).unwrap_or(0.0) > 0.5
                            {
                                Color::Yellow
                            } else {
                                base_color
                            }
                        });
                    let icon_style = if effect.map(|effect| effect.flash_intensity()).unwrap_or(0.0) > 0.0
                        || entry.status == state::ToolTimelineStatus::Running
                            && effect.map(|effect| effect.pulse_intensity()).unwrap_or(0.0) > 0.5
                    {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    let duration = entry
                        .duration_ms
                        .map(|d| format!(" ({}ms)", d))
                        .unwrap_or_default();
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", icon), icon_style),
                        Span::styled(entry.name.clone(), body_style),
                        Span::styled(
                            if entry.args_preview.is_empty() {
                                String::new()
                            } else {
                                format!(" {}", entry.args_preview)
                            },
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(duration, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }
    }

    // Section: Keyboard Shortcuts
    lines.push(Line::from(Span::styled("⌨ Keys", header_style)));
    lines.push(Line::from(Span::styled("  Ctrl+B sidebar", body_style)));
    lines.push(Line::from(Span::styled("  Ctrl+L switch tab", body_style)));
    lines.push(Line::from(Span::styled("  Alt+⏎  submit", body_style)));

    let sidebar = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Context ")
                .style(Style::default().fg(if app.no_color {
                    Color::Reset
                } else {
                    theme.border
                })),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(sidebar, area);
}

fn draw_help_overlay(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::default()
        .title(" Help & Slash Commands ")
        .borders(Borders::ALL)
        .style(Style::default().fg(if app.no_color {
            Color::Reset
        } else {
            Color::Cyan
        }));
    let lines = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter         : New line (in input) or submit (in preview)"),
        Line::from("  Alt+Enter     : Submit immediately / Run slash command"),
        Line::from("  Ctrl+C        : Interrupt generation, or quit if idle"),
        Line::from("  Ctrl+D        : Quit"),
        Line::from("  Ctrl+T        : Toggle internal <think> block visibility"),
        Line::from("  Tab           : Accept ghost autocomplete"),
        Line::from("  Up/Down       : Scroll chat (if input empty), else history"),
        Line::from("  PageUp/Down   : Scroll chat history"),
        Line::from(""),
        Line::from(Span::styled(
            "Slash Commands:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  /help         : Show this help screen"),
        Line::from("  /clear        : Wipe chat & context history entirely"),
        Line::from("  /resume       : Restore latest autosaved session"),
        Line::from("  /mode status  : Show current execution mode"),
        Line::from("  /mode <name>  : Switch mode (`chat` or `agentic`)"),
        Line::from("  /quit, /exit  : Exit application"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to dismiss.",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    // Centered rect
    let popup_area = Rect {
        x: area.width.saturating_sub(60) / 2,
        y: area.height.saturating_sub(20) / 2,
        width: 60.min(area.width),
        height: 20.min(area.height),
    };

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn is_think_span(chat_span: &ChatSpan) -> bool {
    chat_span.style.add_modifier == Modifier::DIM || (chat_span.style.fg == Some(Color::DarkGray))
}

fn apply_animation_style(style: Style, opacity: f32, no_color: bool) -> Style {
    if opacity >= 0.99 {
        return style;
    }

    let mut animated = style;
    if opacity < 0.67 {
        animated = animated.add_modifier(Modifier::DIM);
    }
    if !no_color {
        animated = animated.fg(if opacity < 0.34 {
            Color::DarkGray
        } else {
            style.fg.unwrap_or(Color::Gray)
        });
    }
    animated
}

fn display_theme_name(theme: state::ThemeName) -> &'static str {
    match theme {
        state::ThemeName::Dark => "Dark",
        state::ThemeName::Light => "Light",
        state::ThemeName::Nord => "Nord",
        state::ThemeName::Gruvbox => "Gruvbox",
        state::ThemeName::Custom => "Custom",
    }
}

fn draw_settings_modal(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let theme = app.theme_colors();
    let popup = centered_rect(60, 40, area);
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(popup);

    let category_lines: Vec<Line<'static>> = state::SettingsCategory::all()
        .iter()
        .enumerate()
        .map(|(idx, category)| {
            let style = if idx == app.settings.selected_item {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            Line::from(Span::styled(format!("  {}", category.title()), style))
        })
        .collect();

    let details: Vec<String> = match app.settings.selected_category() {
        state::SettingsCategory::General => vec![
            format!(
                "{} Model Name: {}",
                if app.settings.selected_detail == 0 { "›" } else { " " },
                app.settings.model_name
            ),
            format!(
                "{} Execution Mode: {}",
                if app.settings.selected_detail == 1 { "›" } else { " " },
                app.settings.exec_mode
            ),
            format!(
                "{} Token Limit: {}",
                if app.settings.selected_detail == 2 { "›" } else { " " },
                app.settings.token_limit_input
            ),
        ],
        state::SettingsCategory::Interface => vec![
            format!(
                "{} Theme: {}",
                if app.settings.selected_detail == 0 { "›" } else { " " },
                display_theme_name(app.settings.theme)
            ),
            format!(
                "{} Sidebar: {}",
                if app.settings.selected_detail == 1 { "›" } else { " " },
                if app.settings.sidebar_visible {
                    "Visible"
                } else {
                    "Hidden"
                }
            ),
        ],
        state::SettingsCategory::Security => vec![format!(
            "{} Permission Tier: {}",
            if app.settings.selected_detail == 0 { "›" } else { " " },
            app.settings.permission_tier.label()
        )],
    };

    let detail_lines: Vec<Line<'static>> = details
        .into_iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(theme.foreground))))
        .chain([
            Line::from(""),
            Line::from(Span::styled(
                "Up/Down: categories  Left/Right: fields",
                Style::default().fg(theme.subtitle),
            )),
            Line::from(Span::styled(
                "Enter: change value  Esc: close",
                Style::default().fg(theme.subtitle),
            )),
        ])
        .collect();

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(category_lines).block(
            Block::default()
                .title(" Categories ")
                .borders(Borders::ALL)
                .style(Style::default().fg(theme.border_focused)),
        ),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .title(" Settings ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(theme.border)),
            )
            .wrap(Wrap { trim: false }),
        sections[1],
    );
}

fn push_multiline_plain_entry(
    lines: &mut Vec<Line<'static>>,
    prefix: &str,
    prefix_style: Style,
    content: &str,
) {
    let continuation_prefix = " ".repeat(prefix.chars().count());
    let mut segments = content.split('\n');

    if let Some(first_segment) = segments.next() {
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, first_segment),
            prefix_style,
        )));
    } else {
        lines.push(Line::from(Span::styled(prefix.to_string(), prefix_style)));
    }

    for segment in segments {
        lines.push(Line::from(Span::styled(
            format!("{}{}", continuation_prefix, segment),
            prefix_style,
        )));
    }
}

fn push_multiline_spans_entry(
    lines: &mut Vec<Line<'static>>,
    prefix: &str,
    prefix_style: Style,
    spans: &[ChatSpan],
    show_thoughts: bool,
) {
    let continuation_prefix = " ".repeat(prefix.chars().count());
    let mut current_spans: Vec<Span<'static>> =
        vec![Span::styled(prefix.to_string(), prefix_style)];
    let mut has_visible_content = false;

    for chat_span in spans {
        if !show_thoughts && is_think_span(chat_span) {
            continue;
        }

        let mut remaining = chat_span.text.as_str();
        loop {
            if let Some(newline_idx) = remaining.find('\n') {
                let segment = &remaining[..newline_idx];
                if !segment.is_empty() {
                    current_spans.push(Span::styled(segment.to_string(), chat_span.style));
                    has_visible_content = true;
                }
                lines.push(Line::from(std::mem::take(&mut current_spans)));
                current_spans.push(Span::raw(continuation_prefix.clone()));
                remaining = &remaining[(newline_idx + 1)..];
            } else {
                if !remaining.is_empty() {
                    current_spans.push(Span::styled(remaining.to_string(), chat_span.style));
                    has_visible_content = true;
                }
                break;
            }
        }
    }

    if has_visible_content {
        lines.push(Line::from(current_spans));
    }
}

fn draw_chat_area(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let theme = app.theme_colors();

    // Welcome banner (first time)
    if app.chat_history.is_empty() && !app.is_generating {
        let banner_style = if app.no_color {
            Style::default()
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(Span::styled(
            "██╗  ██╗███████╗██╗     ██╗██╗  ██╗",
            banner_style,
        )));
        lines.push(Line::from(Span::styled(
            "██║  ██║██╔════╝██║     ██║╚██╗██╔╝",
            banner_style,
        )));
        lines.push(Line::from(Span::styled(
            "███████║█████╗  ██║     ██║ ╚███╔╝ ",
            banner_style,
        )));
        lines.push(Line::from(Span::styled(
            "██╔══██║██╔══╝  ██║     ██║ ██╔██╗ ",
            banner_style,
        )));
        lines.push(Line::from(Span::styled(
            "██║  ██║███████╗███████╗██║██╔╝ ██╗",
            banner_style,
        )));
        lines.push(Line::from(Span::styled(
            "╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝",
            banner_style,
        )));
        lines.push(Line::from(""));
        let subtitle_style = if app.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(
            "Py + Rust Hybrid Agent Stack",
            subtitle_style,
        )));
        lines.push(Line::from(Span::styled(
            "Type a prompt to begin. Enter = newline, Alt+Enter = submit.",
            subtitle_style,
        )));
        lines.push(Line::from(""));
    }

    // Chat history
    for (index, entry) in app.chat_history.iter().enumerate() {
        let opacity = app.animations.message_opacity(index);
        let (prefix, prefix_style) = match entry.role.as_str() {
            "user" => {
                let s = if app.no_color {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(theme.user_message)
                        .add_modifier(Modifier::BOLD)
                };
                ("▶ You: ", s)
            }
            "assistant" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(theme.assistant_message)
                };
                ("◆ Helix: ", s)
            }
            "tool" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(theme.tool_message)
                };
                ("⚙ Tool: ", s)
            }
            "tool_start" => {
                let s = if app.no_color {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(theme.info)
                };
                ("🔧 ", s)
            }
            "system" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(theme.system_message)
                };
                ("ℹ ", s)
            }
            _ => ("  ", Style::default()),
        };
        let prefix_style = apply_animation_style(prefix_style, opacity, app.no_color);

        // Handle collapsed tool blocks
        if let Some(tid) = entry.tool_id
            && app.is_tool_collapsed(tid) {
                lines.push(Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(
                        format!("{} [collapsed]", entry.content),
                        apply_animation_style(
                            Style::default().add_modifier(Modifier::DIM),
                            opacity,
                            app.no_color,
                        ),
                    ),
                ]));
                lines.push(Line::from(""));
                continue;
            }

        // Use spans if available (for assistant entries with think blocks)
        if !entry.spans.is_empty() && entry.role == "assistant" {
            let spans: Vec<ChatSpan> = entry
                .spans
                .iter()
                .cloned()
                .map(|mut span| {
                    span.style = apply_animation_style(span.style, opacity, app.no_color);
                    span
                })
                .collect();
            push_multiline_spans_entry(
                &mut lines,
                prefix,
                prefix_style,
                &spans,
                app.show_thoughts,
            );
        } else {
            push_multiline_plain_entry(&mut lines, prefix, prefix_style, &entry.content);
        }
        lines.push(Line::from(""));
    }

    // Streaming content with live span rendering
    if app.is_generating && !app.streaming_content.is_empty() {
        if !app.streaming_spans.is_empty() {
            let prefix_style = if app.no_color {
                Style::default()
            } else {
                Style::default().fg(theme.assistant_message)
            };
            push_multiline_spans_entry(
                &mut lines,
                "◆ Helix: ",
                prefix_style,
                &app.streaming_spans,
                app.show_thoughts,
            );
        } else {
            let style = if app.no_color {
                Style::default()
            } else {
                Style::default().fg(theme.assistant_message)
            };
            push_multiline_plain_entry(&mut lines, "◆ Helix: ", style, &app.streaming_content);
        }
        let indicator = if app.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled("  ▍generating...", indicator)));
    } else if app.is_thinking() {
        let spinner_style = if app.no_color {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };
        let message = app
            .streaming_heartbeat
            .as_deref()
            .unwrap_or("Thinking...");
        lines.push(Line::from(vec![
            Span::styled("◆ Helix: ", Style::default().fg(theme.assistant_message)),
            Span::styled(
                format!("{} ", app.animations.throbber_state.current_frame()),
                spinner_style,
            ),
            Span::styled(
                message.to_string(),
                if app.no_color {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM)
                },
            ),
        ]));
    } else if app.is_generating && let Some(heartbeat) = &app.streaming_heartbeat {
        let style = if app.no_color {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM)
        };
        lines.push(Line::from(Span::styled(
            format!("◆ Helix: {}", heartbeat),
            style,
        )));
    }

    let total_lines = lines.len() as u16;
    let visible = area.height.saturating_sub(2);
    let scroll = total_lines.saturating_sub(visible);
    app.max_scroll_offset = scroll;
    if app.scroll_offset > app.max_scroll_offset {
        app.scroll_offset = app.max_scroll_offset;
    }

    let chat = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Helix Agent • ⌂ {} ", app.cwd)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll.saturating_sub(app.scroll_offset), 0));

    frame.render_widget(chat, area);
}

fn draw_input_area(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let input_text = app.input.value();
    let theme = app.theme_colors();
    let multiline_indicator = if !app.multiline_buffer.is_empty() {
        format!("[{}+] ", app.multiline_buffer.len())
    } else {
        String::new()
    };

    let mut spans = vec![
        Span::styled(
            format!("{}> ", multiline_indicator),
            if app.no_color {
                Style::default()
            } else {
                Style::default().fg(theme.user_message)
            },
        ),
        Span::raw(input_text),
    ];

    // Ghost autocomplete
    if let Some(suggestion) = app.ghost_suggestion() {
        spans.push(Span::styled(
            suggestion,
            if app.no_color {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
    }

    let animated_tokens = app.animated_tokens_display();
    let ratio = if app.max_tokens > 0 {
        (animated_tokens as f64) / (app.max_tokens as f64)
    } else {
        0.0
    };
    let filled = (ratio * 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    let hud = format!(" [{}/{} tok] {} ", animated_tokens, app.max_tokens, bar);

    let input_widget = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(
                Title::from(" Input (Enter=nl, Alt+Enter=sub, Ctrl+E=editor) ")
                    .alignment(Alignment::Left),
            )
            .title(Title::from(hud).alignment(Alignment::Right)),
    );

    frame.render_widget(input_widget, area);

    // Cursor position
    let cursor_x =
        area.x + 1 + multiline_indicator.len() as u16 + 2 + app.input.visual_cursor() as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor(cursor_x, cursor_y);
}

fn draw_status_bar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let theme = app.theme_colors();
    // Build left segment: model | mode | status
    let model_part = if !app.model_name.is_empty() {
        format!("model: {} | ", app.model_name)
    } else {
        String::new()
    };
    let mode_part = if !app.exec_mode_label.is_empty() {
        format!("{} mode | ", app.exec_mode_label)
    } else {
        String::new()
    };
    let thinking_prefix = if app.is_thinking() {
        format!("{} ", app.animations.throbber_state.current_frame())
    } else {
        String::new()
    };
    let left = format!(" {}{}{}{} ", model_part, mode_part, thinking_prefix, app.status_text);

    // Build right segment: [current/max tok] | connection
    let right = format!(
        " [{}/{} tok] | {} ",
        app.animated_tokens_display(),
        app.max_tokens,
        app.connection_label
    );

    let bar_width = area.width as usize;
    let padding = bar_width.saturating_sub(left.len() + right.len());

    let bar_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let style = if app.no_color {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(theme.foreground).bg(theme.background)
    };

    let status = Paragraph::new(Span::styled(bar_text, style));
    frame.render_widget(status, area);
}

fn draw_preview_overlay(frame: &mut Frame, _app: &TuiApp, area: Rect) {
    let popup_area = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup_area);

    let preview = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Send this message?",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  [Enter] Confirm    [Esc] Cancel"),
        Line::from(""),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Command Preview "),
    );

    frame.render_widget(preview, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate_preview_preserving_utf8(input: &str, max_chars: usize) -> String {
    match input.char_indices().nth(max_chars) {
        Some((idx, _)) => format!("{}...", &input[..idx]),
        None => input.to_string(),
    }
}

// ── TUI runner ───────────────────────────────────────────────────────────────

/// Run the TUI event loop. Returns channels for communication with the orchestrator.
///
/// - `layout_mode`: the initial layout mode (Wide or Compact).
/// - `action_rx`: orchestrator reads user actions (Submit / Quit) from this.
/// - `event_tx`: orchestrator sends streaming events (Token, ToolCall, etc.) to this.
pub async fn run_tui(
    layout_mode: api::TuiLayoutMode,
) -> io::Result<(
    mpsc::UnboundedReceiver<TuiAction>,
    mpsc::UnboundedSender<TuiEvent>,
)> {
    let (action_tx, action_rx) = mpsc::unbounded_channel::<TuiAction>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<TuiEvent>();

    tokio::spawn(async move {
        // Setup terminal
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let mut app = TuiApp::new(layout_mode);
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Event>();
        let stop_input = Arc::new(AtomicBool::new(false));
        let stop_input_reader = Arc::clone(&stop_input);

        tokio::task::spawn_blocking(move || {
            while !stop_input_reader.load(Ordering::Relaxed) {
                if event::poll(Duration::from_millis(16)).unwrap_or(false)
                    && let Ok(ev) = event::read()
                    && input_tx.send(ev).is_err() {
                        break;
                    }
            }
        });

        let mut tick = tokio::time::interval(Duration::from_millis(33));
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut last_tick = Instant::now();

        loop {
            // Draw
            terminal
                .draw(|f| draw(f, &mut app))
                .expect("Failed to draw");

            tokio::select! {
                _ = tick.tick() => {
                    let now = Instant::now();
                    let dt = now.saturating_duration_since(last_tick);
                    last_tick = now;
                    app.on_tick(dt);
                }

                Some(ev) = input_rx.recv() => {
                    match ev {
                        Event::Key(key) => {
                            match handle_key_event(&mut app, key, &action_tx) {
                                LoopAction::Quit => break,
                                LoopAction::OpenEditor => {
                                    use std::fs::File;
                                    use std::io::Write;
                                    use std::process::Command;

                                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                                    let tmp_dir = std::env::temp_dir();
                                    let file_path = tmp_dir.join(format!("helix_input_{}.md", std::process::id()));

                                    let current_text = if app.multiline_buffer.is_empty() {
                                        app.input.value().to_string()
                                    } else {
                                        let mut lines = app.multiline_buffer.clone();
                                        lines.push(app.input.value().to_string());
                                        lines.join("\n")
                                    };

                                    if let Ok(mut file) = File::create(&file_path) {
                                        let _ = file.write_all(current_text.as_bytes());
                                    }

                                    let _ = disable_raw_mode();
                                    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                                    let _ = Command::new(editor).arg(&file_path).status();
                                    let _ = enable_raw_mode();
                                    let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);
                                    let _ = terminal.clear();

                                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                                        let mut lines: Vec<String> = content.lines().map(String::from).collect();
                                        if lines.is_empty() {
                                            app.multiline_buffer.clear();
                                            app.input.reset();
                                        } else {
                                            let last = lines.pop().unwrap_or_default();
                                            app.multiline_buffer = lines;
                                            app.input = Input::new(last);
                                        }
                                        let _ = std::fs::remove_file(&file_path);
                                    }
                                    app.status_text = "Editor closed".to_string();
                                }
                                LoopAction::Continue => {}
                            }
                        }
                        Event::Resize(_, _) => {}
                        _ => {}
                    }
                }

                Some(tui_event) = event_rx.recv() => {
                    match tui_event {
                        TuiEvent::TokenChunk(chunk) => {
                            app.is_generating = true;
                            app.append_token_chunk(&chunk);
                        }
                        TuiEvent::ResponseDone => {
                            app.finalize_streaming();
                            app.is_generating = false;
                            app.streaming_heartbeat = None;
                            app.ttft_start = None;
                            app.ttft_locked = None;
                            app.status_text = "Ready".to_string();
                        }
                        TuiEvent::ToolStart(info) => {
                            app.status_text = format!("🔧 Executing {}...", info.name);
                            let content = format!("Executing `{}`...", info.name);
                            let tool_id = app.tool_timeline.len() as u64;

                            app.push_chat_entry(
                                "tool_start",
                                content.clone(),
                                vec![ChatSpan {
                                    text: content,
                                    style: if app.no_color {
                                        Style::default().add_modifier(Modifier::DIM)
                                    } else {
                                        Style::default().fg(Color::Cyan)
                                    },
                                }],
                                Some(tool_id),
                            );

                            // Add to tool_timeline
                            app.tool_timeline.push(state::ToolTimelineEntry::new(
                                tool_id,
                                info.name.clone(),
                                truncate_preview_preserving_utf8(&info.arguments, 50),
                            ));
                            app.animations.register_tool_running(tool_id);
                        }
                        TuiEvent::ToolResult(result) => {
                            let icon = if result.success { "✓" } else { "✗" };
                            let truncated = truncate_preview_preserving_utf8(&result.output, 200);
                            let content = format!("{} {} → {}", icon, result.name, truncated);
                            app.push_chat_entry(
                                "tool",
                                content.clone(),
                                vec![ChatSpan {
                                    text: content,
                                    style: if app.no_color {
                                        Style::default()
                                    } else {
                                        Style::default().fg(Color::Yellow)
                                    },
                                }],
                                None,
                            );

                            // Update tool_timeline
                            if let Some(entry) = app.tool_timeline.iter_mut().rev().find(|e| {
                                e.name == result.name && e.status == state::ToolTimelineStatus::Running
                            }) {
                                entry.mark_finished(result.success);
                                app.animations
                                    .register_tool_finished(entry.id, result.success);
                            }
                        }
                        TuiEvent::Status(text) => {
                            app.status_text = text;
                        }
                        TuiEvent::StatusUpdate(text) => {
                            // Only update status text if TTFT hasn't locked yet
                            if app.ttft_locked.is_none() {
                                app.status_text = text;
                            }
                        }
                        TuiEvent::GenerationStarted => {
                            if let Some(start) = app.ttft_start
                                && app.ttft_locked.is_none() {
                                    let duration = start.elapsed();
                                    app.ttft_locked = Some(duration);
                                    let secs = duration.as_secs_f32();
                                    app.status_text = format!("Generating... [TTFT: {:.2}s]", secs);
                                }
                        }
                        TuiEvent::StreamingHeartbeat(text) => {
                            app.is_generating = true;
                            app.streaming_heartbeat = Some(text);
                        }
                        TuiEvent::ContextUpdate(current, max) => {
                            app.current_tokens = current;
                            app.max_tokens = max;
                        }
                        TuiEvent::ClearHistory => {
                            app.chat_history.clear();
                            app.tool_timeline.clear();
                            app.scroll_offset = 0;
                            app.max_scroll_offset = 0;
                            app.animations = state::AnimationState::new();
                        }
                        TuiEvent::SystemMessage(msg) => {
                            app.push_chat_entry(
                                "system",
                                msg.clone(),
                                vec![ChatSpan {
                                    text: msg,
                                    style: if app.no_color {
                                        Style::default()
                                    } else {
                                        Style::default().fg(Color::DarkGray)
                                    },
                                }],
                                None,
                            );
                        }
                        TuiEvent::ThemeChanged(theme) => {
                            app.current_theme = theme;
                            app.settings.theme = theme;
                        }
                        TuiEvent::ContextSnapshot {
                            tokens_used,
                            max_tokens,
                            files: _files,
                            model_name,
                            exec_mode,
                            connection,
                        } => {
                            app.current_tokens = tokens_used;
                            app.max_tokens = max_tokens;
                            app.model_name = model_name;
                            app.exec_mode_label = exec_mode;
                            app.connection_label = connection.to_string();
                            app.sync_settings_from_app();
                        }
                    }
                }
            }
        }

        stop_input.store(true, Ordering::Relaxed);
        // Cleanup terminal
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .expect("Failed to leave alternate screen");
        terminal.show_cursor().expect("Failed to show cursor");
    });

    Ok((action_rx, event_tx))
}

/// Handle a key event. Returns false if the app should quit.
#[derive(Debug, PartialEq, Eq)]
pub enum LoopAction {
    Continue,
    Quit,
    OpenEditor,
}

fn handle_key_event(
    app: &mut TuiApp,
    key: KeyEvent,
    action_tx: &mpsc::UnboundedSender<TuiAction>,
) -> LoopAction {
    match app.input_mode {
        InputMode::Help => {
            // Any key leaves help mode.
            app.input_mode = InputMode::Normal;
        }
        InputMode::Preview => match key.code {
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
                if let Some(text) = app.submit_input() {
                    let _ = action_tx.send(TuiAction::Submit(text));
                    app.status_text = "Generating...".to_string();
                    app.is_generating = true;
                    app.ttft_start = Some(Instant::now());
                    app.ttft_locked = None;
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::Settings => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.settings.active_modal = false;
                let _ = action_tx.send(TuiAction::CloseSettings);
            }
            KeyCode::Up => {
                app.settings.prev_category();
            }
            KeyCode::Down => {
                app.settings.next_category();
            }
            KeyCode::Left => {
                app.settings.prev_detail();
            }
            KeyCode::Right => {
                app.settings.next_detail();
            }
            KeyCode::Enter => {
                match app.settings.selected_category() {
                    state::SettingsCategory::General => if app.settings.selected_detail == 1 {
                        app.settings.exec_mode = if app.settings.exec_mode == "chat" {
                            "agentic".to_string()
                        } else {
                            "chat".to_string()
                        };
                    },
                    state::SettingsCategory::Interface => match app.settings.selected_detail {
                        0 => app.settings.theme = app.settings.theme.next(),
                        1 => app.settings.sidebar_visible = !app.settings.sidebar_visible,
                        _ => {}
                    },
                    state::SettingsCategory::Security => {
                        app.settings.permission_tier = app.settings.permission_tier.next();
                    }
                }
                app.apply_settings_to_app();
                app.status_text = format!(
                    "Updated {}",
                    app.settings.selected_category().title()
                );
            }
            KeyCode::Backspace => {
                if matches!(app.settings.selected_category(), state::SettingsCategory::General)
                    && app.settings.selected_detail == 2
                {
                    app.settings.token_limit_input.pop();
                    if app.settings.token_limit_input.is_empty() {
                        app.settings.token_limit_input = "0".to_string();
                    }
                    app.apply_settings_to_app();
                }
            }
            KeyCode::Char(c) => {
                if matches!(app.settings.selected_category(), state::SettingsCategory::General) {
                    match app.settings.selected_detail {
                        0 => {
                            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                app.settings.model_name.push(c);
                            }
                        }
                        2 if c.is_ascii_digit() => {
                            if app.settings.token_limit_input == "0" {
                                app.settings.token_limit_input.clear();
                            }
                            app.settings.token_limit_input.push(c);
                            app.apply_settings_to_app();
                            app.status_text = format!(
                                "Token limit: {}",
                                app.settings.token_limit_input
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        InputMode::Normal => {
            if app.command_palette.visible && !app.command_palette.commands.is_empty() {
                match key.code {
                    KeyCode::Up => {
                        if app.command_palette.selected_index > 0 {
                            app.command_palette.selected_index -= 1;
                        } else {
                            app.command_palette.selected_index = app.command_palette.commands.len().saturating_sub(1);
                        }
                        return LoopAction::Continue;
                    }
                    KeyCode::Down => {
                        app.command_palette.selected_index = (app.command_palette.selected_index + 1) % app.command_palette.commands.len();
                        return LoopAction::Continue;
                    }
                    KeyCode::Enter => {
                        let cmd = app.command_palette.commands[app.command_palette.selected_index].clone();
                        let _ = action_tx.send(TuiAction::SystemCommand(cmd.name.clone()));
                        app.input.reset();
                        app.command_palette.visible = false;
                        return LoopAction::Continue;
                    }
                    KeyCode::Esc => {
                        app.command_palette.visible = false;
                        return LoopAction::Continue;
                    }
                    _ => {}
                }
            }
            match key.code {
                // Ctrl+C = interrupt generation or quit
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if app.is_generating {
                        let _ = action_tx.send(TuiAction::Interrupt);
                        app.status_text = "Interrupting...".to_string();
                        return LoopAction::Continue; // Don't quit, just interrupt
                    } else {
                        let _ = action_tx.send(TuiAction::Quit);
                        return LoopAction::Quit;
                    }
                }
                // Ctrl+D = quit
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = action_tx.send(TuiAction::Quit);
                    return LoopAction::Quit;
                }

                // Ctrl+T = toggle think block visibility
                KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.show_thoughts = !app.show_thoughts;
                    app.status_text = if app.show_thoughts {
                        "Thinking blocks: visible".to_string()
                    } else {
                        "Thinking blocks: hidden".to_string()
                    };
                }

                // Ctrl+B = toggle sidebar
                KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.sidebar_visible = !app.sidebar_visible;
                    app.settings.sidebar_visible = app.sidebar_visible;
                    app.status_text = if app.sidebar_visible {
                        "Sidebar: visible".to_string()
                    } else {
                        "Sidebar: hidden".to_string()
                    };
                }

                // Ctrl+L = switch sidebar tab
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.sidebar_tab = app.sidebar_tab.toggle();
                    app.status_text = match app.sidebar_tab {
                        state::SidebarTab::ContextFiles => "Sidebar: Context Files".to_string(),
                        state::SidebarTab::ToolTimeline => "Sidebar: Tool Timeline".to_string(),
                    };
                }

                // Ctrl+E = open external editor
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return LoopAction::OpenEditor;
                }

                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.sync_settings_from_app();
                    app.settings.active_modal = true;
                    app.input_mode = InputMode::Settings;
                    let _ = action_tx.send(TuiAction::OpenSettings);
                    app.status_text = "Settings opened".to_string();
                }

                // Alt+Enter = submit (go to preview)
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
                    let current = app.input.value().to_string();
                    if !current.trim().is_empty() || !app.multiline_buffer.is_empty() {
                        // Check for slash commands — submit directly without preview
                        let full_text = if app.multiline_buffer.is_empty() {
                            current.clone()
                        } else {
                            let mut lines = app.multiline_buffer.clone();
                            lines.push(current);
                            lines.join("\n")
                        };

                        let trimmed = full_text.trim();
                        if trimmed == "/quit" || trimmed == "/exit" {
                            let _ = action_tx.send(TuiAction::Quit);
                            return LoopAction::Quit;
                        } else if trimmed == "/help" {
                            app.input_mode = InputMode::Help;
                            app.input.reset();
                            app.multiline_buffer.clear();
                        } else if trimmed.starts_with('/') {
                            let _ = action_tx.send(TuiAction::SystemCommand(trimmed.to_string()));
                            app.input.reset();
                            app.multiline_buffer.clear();
                        } else if let Some(text) = app.submit_input() {
                            let _ = action_tx.send(TuiAction::Submit(text));
                            app.status_text = "Generating...".to_string();
                            app.is_generating = true;
                            app.ttft_start = Some(Instant::now());
                            app.ttft_locked = None;
                        }
                    }
                }

                // Enter = newline (multiline input)
                KeyCode::Enter => {
                    app.add_newline();
                }

                // Tab or Right = accept ghost suggestion
                KeyCode::Tab | KeyCode::Right => {
                    if let Some(suggestion) = app.ghost_suggestion() {
                        let current = app.input.value().to_string();
                        let completed = format!("{}{}", current, suggestion);
                        app.input = Input::new(completed);
                    } else if key.code == KeyCode::Right {
                        app.input.handle_event(&Event::Key(key));
                    }
                }

                // Up = scroll chat history if input empty; else history navigation
                KeyCode::Up => {
                    if app.input.value().is_empty() && app.multiline_buffer.is_empty() {
                        app.scroll_offset = app
                            .scroll_offset
                            .saturating_add(1)
                            .min(app.max_scroll_offset);
                    } else if !app.command_history.is_empty() {
                        let idx = match app.history_index {
                            Some(0) => 0,
                            Some(i) => i - 1,
                            None => app.command_history.len().saturating_sub(1),
                        };
                        app.history_index = Some(idx);
                        if let Some(cmd) = app.command_history.get(idx) {
                            app.input = Input::new(cmd.clone());
                        }
                    }
                }

                // Down = scroll chat history if input empty; else forward history navigation
                KeyCode::Down => {
                    if app.input.value().is_empty() && app.multiline_buffer.is_empty() {
                        app.scroll_offset = app.scroll_offset.saturating_sub(1);
                    } else if let Some(idx) = app.history_index {
                        if idx + 1 < app.command_history.len() {
                            app.history_index = Some(idx + 1);
                            let cmd = app.command_history[idx + 1].clone();
                            app.input = Input::new(cmd);
                        } else {
                            app.history_index = None;
                            app.input.reset();
                        }
                    }
                }

                // Scroll
                KeyCode::PageUp => {
                    app.scroll_offset = app
                        .scroll_offset
                        .saturating_add(5)
                        .min(app.max_scroll_offset);
                }
                KeyCode::PageDown => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(5);
                }

                // All other keys → tui-input handler
                _ => {
                    app.input.handle_event(&Event::Key(key));
                    // Reset scroll on typing
                    app.scroll_offset = 0;
                }
            }
        }
    }
    LoopAction::Continue
}

#[cfg(test)]
mod tests {
    use super::{LoopAction, TuiApp, api, handle_key_event, truncate_preview_preserving_utf8};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tokio::sync::mpsc;
    use tui_input::Input;

    #[test]
    fn truncates_ascii_preview_with_ellipsis() {
        let input = "a".repeat(250);
        let output = truncate_preview_preserving_utf8(&input, 200);
        assert!(output.ends_with("..."));
        assert_eq!(output.chars().count(), 203);
    }

    #[test]
    fn truncates_unicode_preview_without_breaking_char_boundaries() {
        let input = "│".repeat(250);
        let output = truncate_preview_preserving_utf8(&input, 200);
        assert!(output.ends_with("..."));
        assert_eq!(output.chars().count(), 203);
        assert!(output.starts_with("│││"));
    }

    #[test]
    fn leaves_short_preview_unchanged() {
        let input = "short │ output";
        assert_eq!(truncate_preview_preserving_utf8(input, 200), input);
    }

    #[test]
    fn submit_resets_scroll_offset_to_latest() {
        let mut app = TuiApp::new(api::TuiLayoutMode::Wide);
        app.scroll_offset = 7;
        app.input = Input::new("hello".to_string());

        let submitted = app.submit_input();

        assert_eq!(submitted.as_deref(), Some("hello"));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn up_scroll_respects_max_scroll_offset() {
        let mut app = TuiApp::new(api::TuiLayoutMode::Wide);
        app.max_scroll_offset = 2;
        app.scroll_offset = 2;
        let (action_tx, _action_rx) = mpsc::unbounded_channel();

        let keep_running = handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            &action_tx,
        );

        assert_eq!(keep_running, LoopAction::Continue);
        assert_eq!(app.scroll_offset, 2);
    }

    #[test]
    fn down_scroll_moves_towards_latest_response() {
        let mut app = TuiApp::new(api::TuiLayoutMode::Wide);
        app.max_scroll_offset = 10;
        app.scroll_offset = 3;
        let (action_tx, _action_rx) = mpsc::unbounded_channel();

        let keep_running = handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            &action_tx,
        );

        assert_eq!(keep_running, LoopAction::Continue);
        assert_eq!(app.scroll_offset, 2);
    }
}

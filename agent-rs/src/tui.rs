#![allow(dead_code)]
use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame, Terminal,
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use tokio::sync::mpsc;

// ── Slash commands for autocomplete ──────────────────────────────────────────

const SLASH_COMMANDS: &[&str] = &[
    "/help",
    "/quit",
    "/exit",
    "/clear",
    "/save",
    "/load",
    "/history",
    "/model",
    "/mode",
];

// ── Application state ────────────────────────────────────────────────────────

/// Messages sent from the TUI to the orchestrator.
#[derive(Debug)]
pub enum TuiAction {
    /// User submitted a prompt.
    Submit(String),
    /// User requested quit.
    Quit,
    /// User pressed Ctrl+C while generating — cancel current generation.
    Interrupt,
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
    /// First token received — locks the TTFT timer.
    GenerationStarted,
    /// Status text for the status bar.
    Status(String),
    /// System message (info/warning).
    SystemMessage(String),
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    /// Preview panel shown, awaiting confirmation.
    Preview,
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
    /// Whether the model is currently generating.
    pub is_generating: bool,
    /// Toggle visibility of thinking blocks (Ctrl+T).
    show_thoughts: bool,
    scroll_offset: u16,
    /// Total rendered line count from last draw (for scroll clamping).
    scroll_height: u16,
    status_text: String,
    no_color: bool,
    command_history: Vec<String>,
    history_index: Option<usize>,
    /// TTFT tracking: when generation request was sent.
    ttft_start: Option<Instant>,
    /// TTFT tracking: locked duration once first token arrives.
    ttft_locked: Option<Duration>,
}

impl TuiApp {
    pub fn new() -> Self {
        let no_color = std::env::var("NO_COLOR").is_ok();
        Self {
            input: Input::default(),
            input_mode: InputMode::Normal,
            multiline_buffer: Vec::new(),
            chat_history: Vec::new(),
            streaming_content: String::new(),
            streaming_spans: Vec::new(),
            in_think_block: false,
            is_generating: false,
            show_thoughts: true,
            scroll_offset: 0,
            scroll_height: 0,
            status_text: String::from("Ready"),
            no_color,
            command_history: Vec::new(),
            history_index: None,
            ttft_start: None,
            ttft_locked: None,
        }
    }

    /// Get ghost autocomplete suggestion based on current input.
    fn ghost_suggestion(&self) -> Option<String> {
        let current = self.input.value();
        if current.is_empty() {
            return None;
        }
        if current.starts_with('/') {
            for cmd in SLASH_COMMANDS {
                if cmd.starts_with(current) && *cmd != current {
                    return Some(cmd[current.len()..].to_string());
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

        self.command_history.push(trimmed.clone());
        self.history_index = None;
        self.chat_history.push(ChatEntry {
            role: "user".to_string(),
            content: trimmed.clone(),
            spans: vec![ChatSpan {
                text: trimmed.clone(),
                style: Style::default(),
            }],
        });
        Some(trimmed)
    }

    fn add_newline(&mut self) {
        self.multiline_buffer.push(self.input.value().to_string());
        self.input.reset();
    }

    /// Append a token chunk to streaming content, parsing <think>/</think> tags
    /// to build styled spans in real time.
    fn append_token_chunk(&mut self, chunk: &str) {
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
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
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
            let clean_content = self.streaming_content
                .replace("<think>", "")
                .replace("</think>", "");

            self.chat_history.push(ChatEntry {
                role: "assistant".to_string(),
                content: clean_content,
                spans: self.streaming_spans.clone(),
            });
            self.streaming_content.clear();
            self.streaming_spans.clear();
            self.in_think_block = false;
        }
    }
}

// ── Drawing ──────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let size = frame.size();

    // Layout: [banner/chat area] [input area] [status bar]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),       // chat area
            Constraint::Length(3),    // input area
            Constraint::Length(1),    // status bar
        ])
        .split(size);

    draw_chat_area(frame, app, chunks[0]);
    draw_input_area(frame, &app, chunks[1]);
    draw_status_bar(frame, &app, chunks[2]);

    // Draw preview overlay if in preview mode
    if app.input_mode == InputMode::Preview {
        draw_preview_overlay(frame, app, size);
    }
}

fn draw_chat_area(frame: &mut Frame, app: &mut TuiApp, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Welcome banner (first time)
    if app.chat_history.is_empty() && !app.is_generating {
        let banner_style = if app.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(Span::styled("██╗  ██╗███████╗██╗     ██╗██╗  ██╗", banner_style)));
        lines.push(Line::from(Span::styled("██║  ██║██╔════╝██║     ██║╚██╗██╔╝", banner_style)));
        lines.push(Line::from(Span::styled("███████║█████╗  ██║     ██║ ╚███╔╝ ", banner_style)));
        lines.push(Line::from(Span::styled("██╔══██║██╔══╝  ██║     ██║ ██╔██╗ ", banner_style)));
        lines.push(Line::from(Span::styled("██║  ██║███████╗███████╗██║██╔╝ ██╗", banner_style)));
        lines.push(Line::from(Span::styled("╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝", banner_style)));
        lines.push(Line::from(""));
        let subtitle_style = if app.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled("Py + Rust Hybrid Agent Stack", subtitle_style)));
        lines.push(Line::from(Span::styled("Type a prompt to begin. Enter = newline, Alt+Enter = submit.", subtitle_style)));
        lines.push(Line::from(""));
    }

    // Chat history
    for entry in &app.chat_history {
        let (prefix, prefix_style) = match entry.role.as_str() {
            "user" => {
                let s = if app.no_color {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                };
                ("▶ You: ", s)
            }
            "assistant" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(Color::Blue)
                };
                ("◆ Helix: ", s)
            }
            "tool" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(Color::Yellow)
                };
                ("⚙ Tool: ", s)
            }
            "tool_start" => {
                let s = if app.no_color {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                ("🔧 ", s)
            }
            "system" => {
                let s = if app.no_color {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ("ℹ ", s)
            }
            _ => ("  ", Style::default()),
        };

        // Use spans if available (for assistant entries with think blocks)
        if !entry.spans.is_empty() && entry.role == "assistant" {
            let mut line_spans: Vec<Span> = vec![Span::styled(prefix, prefix_style)];
            for chat_span in &entry.spans {
                // Skip think-block spans if show_thoughts is false
                let is_think = chat_span.style.add_modifier == Modifier::DIM
                    || (chat_span.style.fg == Some(Color::DarkGray));
                if !app.show_thoughts && is_think {
                    continue;
                }
                line_spans.push(Span::styled(chat_span.text.clone(), chat_span.style));
            }
            lines.push(Line::from(line_spans));
        } else {
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, entry.content), prefix_style)));
        }
        lines.push(Line::from(""));
    }

    // Streaming content with live span rendering
    if app.is_generating && !app.streaming_content.is_empty() {
        if !app.streaming_spans.is_empty() {
            let prefix_style = if app.no_color {
                Style::default()
            } else {
                Style::default().fg(Color::Blue)
            };
            let mut line_spans: Vec<Span> = vec![Span::styled("◆ Helix: ", prefix_style)];
            for chat_span in &app.streaming_spans {
                let is_think = chat_span.style.add_modifier == Modifier::DIM
                    || (chat_span.style.fg == Some(Color::DarkGray));
                if !app.show_thoughts && is_think {
                    continue;
                }
                line_spans.push(Span::styled(chat_span.text.clone(), chat_span.style));
            }
            lines.push(Line::from(line_spans));
        } else {
            let style = if app.no_color {
                Style::default()
            } else {
                Style::default().fg(Color::Blue)
            };
            lines.push(Line::from(Span::styled(format!("◆ Helix: {}", app.streaming_content), style)));
        }
        let indicator = if app.no_color {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled("  ▍generating...", indicator)));
    }

    let total_lines = lines.len() as u16;
    let visible = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(visible);

    // Store for scroll clamping in key handler
    app.scroll_height = max_scroll;

    // Clamp scroll_offset to valid range
    if app.scroll_offset > max_scroll {
        app.scroll_offset = max_scroll;
    }

    let scroll = max_scroll.saturating_sub(app.scroll_offset);

    let chat = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(" Helix Agent "))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(chat, area);
}

fn draw_input_area(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let input_text = app.input.value();
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
                Style::default().fg(Color::Green)
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

    let input_widget = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).title(" Input (Enter=newline, Alt+Enter=submit) "));

    frame.render_widget(input_widget, area);

    // Cursor position
    let cursor_x = area.x + 1 + multiline_indicator.len() as u16 + 2 + app.input.visual_cursor() as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor(cursor_x, cursor_y);
}

fn draw_status_bar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let char_count = app.input.value().len();
    let ml_count = app.multiline_buffer.len();

    // Build dynamic status text with TTFT tracking
    let status_display = if app.is_generating {
        if let Some(locked) = app.ttft_locked {
            // First token received — show frozen TTFT alongside status
            format!("{} [TTFT: {:.1}s]", app.status_text, locked.as_secs_f64())
        } else if let Some(start) = app.ttft_start {
            // Still waiting for first token — show live elapsed
            let elapsed = start.elapsed();
            format!("Thinking... ({:.1}s)", elapsed.as_secs_f64())
        } else {
            app.status_text.clone()
        }
    } else {
        // Show TTFT from last generation if available
        if let Some(locked) = app.ttft_locked {
            format!("{} [last TTFT: {:.1}s]", app.status_text, locked.as_secs_f64())
        } else {
            app.status_text.clone()
        }
    };

    let left = format!(" {} ", status_display);
    let right = format!(" chars: {} | lines: {} ", char_count, ml_count + 1);

    let bar_width = area.width as usize;
    let padding = bar_width.saturating_sub(left.len() + right.len());

    let bar_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let style = if app.no_color {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    };

    let status = Paragraph::new(Span::styled(bar_text, style));
    frame.render_widget(status, area);
}

fn draw_preview_overlay(frame: &mut Frame, _app: &TuiApp, area: Rect) {
    let popup_area = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup_area);

    let preview = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled("  Send this message?", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  [Enter] Confirm    [Esc] Cancel"),
        Line::from(""),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Command Preview "));

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

// ── TUI runner ───────────────────────────────────────────────────────────────

/// Run the TUI event loop. Returns channels for communication with the orchestrator.
///
/// - `action_rx`: orchestrator reads user actions (Submit / Quit) from this.
/// - `event_tx`: orchestrator sends streaming events (Token, ToolCall, etc.) to this.
pub async fn run_tui() -> io::Result<(mpsc::UnboundedReceiver<TuiAction>, mpsc::UnboundedSender<TuiEvent>)> {
    let (action_tx, action_rx) = mpsc::unbounded_channel::<TuiAction>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<TuiEvent>();

    tokio::spawn(async move {
        // Setup terminal
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        let mut app = TuiApp::new();

        loop {
            // Draw
            terminal.draw(|f| draw(f, &mut app)).expect("Failed to draw");

            // Poll for events (crossterm keyboard) or orchestrator events
            tokio::select! {
                // Keyboard input
                _ = tokio::task::spawn_blocking(|| {
                    event::poll(std::time::Duration::from_millis(50)).unwrap_or(false)
                }) => {
                    if event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
                        if let Ok(ev) = event::read() {
                            match ev {
                                Event::Key(key) => {
                                    if !handle_key_event(&mut app, key, &action_tx) {
                                        break;
                                    }
                                }
                                Event::Resize(_, _) => {
                                    // Terminal will redraw automatically
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Orchestrator events
                Some(tui_event) = event_rx.recv() => {
                    match tui_event {
                        TuiEvent::TokenChunk(chunk) => {
                            app.is_generating = true;
                            app.append_token_chunk(&chunk);
                            // Auto-scroll to bottom when new content arrives
                            app.scroll_offset = 0;
                        }
                        TuiEvent::GenerationStarted => {
                            // Lock TTFT timer on first token
                            if let Some(start) = app.ttft_start {
                                app.ttft_locked = Some(start.elapsed());
                            }
                        }
                        TuiEvent::ResponseDone => {
                            app.finalize_streaming();
                            app.is_generating = false;
                            app.status_text = "Ready".to_string();
                        }
                        TuiEvent::ToolStart(info) => {
                            app.status_text = format!("🔧 Executing {}...", info.name);
                            let content = format!("Executing `{}`...", info.name);
                            app.chat_history.push(ChatEntry {
                                role: "tool_start".to_string(),
                                content: content.clone(),
                                spans: vec![ChatSpan {
                                    text: content,
                                    style: if app.no_color {
                                        Style::default().add_modifier(Modifier::DIM)
                                    } else {
                                        Style::default().fg(Color::Cyan)
                                    },
                                }],
                            });
                        }
                        TuiEvent::ToolResult(result) => {
                            let icon = if result.success { "✓" } else { "✗" };
                            let truncated = if result.output.len() > 200 {
                                format!("{}...", &result.output[..200])
                            } else {
                                result.output.clone()
                            };
                            let content = format!("{} {} → {}", icon, result.name, truncated);
                            app.chat_history.push(ChatEntry {
                                role: "tool".to_string(),
                                content: content.clone(),
                                spans: vec![ChatSpan {
                                    text: content,
                                    style: if app.no_color {
                                        Style::default()
                                    } else {
                                        Style::default().fg(Color::Yellow)
                                    },
                                }],
                            });
                        }
                        TuiEvent::Status(text) => {
                            app.status_text = text;
                        }
                        TuiEvent::SystemMessage(msg) => {
                            app.chat_history.push(ChatEntry {
                                role: "system".to_string(),
                                content: msg.clone(),
                                spans: vec![ChatSpan {
                                    text: msg,
                                    style: if app.no_color {
                                        Style::default()
                                    } else {
                                        Style::default().fg(Color::DarkGray)
                                    },
                                }],
                            });
                        }
                    }
                }
            }
        }

        // Cleanup terminal
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        ).expect("Failed to leave alternate screen");
        terminal.show_cursor().expect("Failed to show cursor");
    });

    Ok((action_rx, event_tx))
}

/// Handle a key event. Returns false if the app should quit.
fn handle_key_event(app: &mut TuiApp, key: KeyEvent, action_tx: &mpsc::UnboundedSender<TuiAction>) -> bool {
    match app.input_mode {
        InputMode::Preview => {
            match key.code {
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
            }
        }
        InputMode::Normal => {
            match key.code {
                // Ctrl+C = interrupt (if generating) or quit (if idle)
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if app.is_generating {
                        let _ = action_tx.send(TuiAction::Interrupt);
                        app.status_text = "Interrupting...".to_string();
                    } else {
                        let _ = action_tx.send(TuiAction::Quit);
                        return false;
                    }
                }
                // Ctrl+D = always quit
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = action_tx.send(TuiAction::Quit);
                    return false;
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

                        if full_text.trim() == "/quit" || full_text.trim() == "/exit" {
                            let _ = action_tx.send(TuiAction::Quit);
                            return false;
                        }

                        if let Some(text) = app.submit_input() {
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

                // Tab = accept ghost suggestion
                KeyCode::Tab => {
                    if let Some(suggestion) = app.ghost_suggestion() {
                        let current = app.input.value().to_string();
                        let completed = format!("{}{}", current, suggestion);
                        app.input = Input::new(completed);
                    }
                }

                // Up = scroll chat (input empty) or history navigation (input has text)
                KeyCode::Up => {
                    if app.input.value().is_empty() && app.multiline_buffer.is_empty() {
                        // Scroll chat up by 1 line
                        app.scroll_offset = app.scroll_offset.saturating_add(1)
                            .min(app.scroll_height);
                    } else if !app.command_history.is_empty() {
                        let idx = match app.history_index {
                            Some(0) => 0,
                            Some(i) => i - 1,
                            None => app.command_history.len() - 1,
                        };
                        app.history_index = Some(idx);
                        let cmd = app.command_history[idx].clone();
                        app.input = Input::new(cmd);
                    }
                }

                // Down = scroll chat (input empty) or forward history (input has text)
                KeyCode::Down => {
                    if app.input.value().is_empty() && app.multiline_buffer.is_empty() {
                        // Scroll chat down by 1 line
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

                // Scroll (always works regardless of input state)
                KeyCode::PageUp => {
                    app.scroll_offset = app.scroll_offset.saturating_add(10)
                        .min(app.scroll_height);
                }
                KeyCode::PageDown => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(10);
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
    true
}

mod config;
mod input;
mod server;
mod stream;
mod tokens;
mod tools;
mod tui;
pub mod types;
mod utils;

use reqwest::Client;
use rustyline::error::ReadlineError;
use schemars::schema_for;
use serde_json::{Value, json};
use std::thread;
use std::time::Duration;
use tools::ToolCallArgs;

pub use types::{ChatMessage, ChatResponse, Choice, ServerFlavor};

static TUI_HTTP_CONNECT_FAILS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
static MODEL_SERVER_BOOT_ATTEMPTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn critic_message(text: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: Some(format!("[Rust Critic] {}", text)),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

async fn detect_server_flavor(client: &Client, base_url: &str) -> ServerFlavor {
    let models_url = format!("{}/models", base_url.trim_end_matches('/'));
    let res = match client.get(&models_url).send().await {
        Ok(r) => r,
        Err(_) => return ServerFlavor::Unknown,
    };

    let body = match res.text().await {
        Ok(t) => t.to_lowercase(),
        Err(_) => return ServerFlavor::Unknown,
    };

    if body.contains("kobold") {
        ServerFlavor::KoboldCpp
    } else if body.contains("llama") {
        ServerFlavor::LlamaCpp
    } else {
        ServerFlavor::Unknown
    }
}

fn print_helix_logo(animated: bool) {
    let logo_lines = [
        "██╗  ██╗███████╗██╗     ██╗██╗  ██╗",
        "██║  ██║██╔════╝██║     ██║╚██╗██╔╝",
        "███████║█████╗  ██║     ██║ ╚███╔╝ ",
        "██╔══██║██╔══╝  ██║     ██║ ██╔██╗ ",
        "██║  ██║███████╗███████╗██║██╔╝ ██╗",
        "╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝",
    ];

    if animated {
        for line in logo_lines {
            println!("{}", line);
            thread::sleep(Duration::from_millis(20));
        }
    } else {
        for line in logo_lines {
            println!("{}", line);
        }
    }

    println!("Py + Rust Hybrid Agent Stack");
}

/// Exposes internal `<think>` blocks by mapping them to `<thinking>` tags for the UI,
/// ensuring the user can see the agent's internal reasoning.
pub fn expose_think_blocks(text: &str) -> String {
    let mut s = text.replace("<think>", "\n<thinking>\n");
    s = s.replace("</think>", "\n</thinking>\n");
    if s.contains("<thinking>") && !s.contains("</thinking>") {
        s.push_str("\n</thinking>");
    }
    s.trim().to_string()
}

fn format_visible_output(text: &str, is_chat_mode: bool) -> String {
    if is_chat_mode {
        let cleaned = utils::clean_chat_output(text);
        if cleaned.trim().is_empty() && !text.trim().is_empty() {
            "I could not produce a visible response. Please retry.".to_string()
        } else {
            cleaned
        }
    } else {
        expose_think_blocks(text)
    }
}

fn extract_visible_delta_text(delta: &Value, _is_chat_mode: bool) -> Option<(String, bool)> {
    let keys: &[(&str, bool)] = &[
        ("content", false),
        ("reasoning_content", true),
        ("text", false),
        ("response", false),
    ];

    for (key, is_reasoning) in keys {
        if let Some(value) = delta.get(*key).and_then(|v| v.as_str()) {
            return Some((value.to_string(), *is_reasoning));
        }
    }
    None
}

fn extract_visible_message_or_choice_text(node: &Value) -> Option<(String, bool)> {
    let keys: &[(&str, bool)] = &[
        ("content", false),
        ("reasoning_content", true),
        ("text", false),
        ("response", false),
    ];

    for (key, is_reasoning) in keys {
        if let Some(value) = node.get(*key).and_then(|v| v.as_str()) {
            return Some((value.to_string(), *is_reasoning));
        }
    }

    None
}

fn extract_visible_stream_choice_text(
    choice: &Value,
    is_chat_mode: bool,
) -> Option<(String, bool)> {
    if let Some(delta) = choice.get("delta") {
        if let Some(result) = extract_visible_delta_text(delta, is_chat_mode) {
            return Some(result);
        }
    }

    if let Some(result) = extract_visible_message_or_choice_text(choice) {
        return Some(result);
    }

    if let Some(message) = choice.get("message") {
        if let Some(result) = extract_visible_message_or_choice_text(message) {
            return Some(result);
        }
    }

    None
}

fn extract_stream_tool_calls<'a>(choice: &'a Value) -> Option<&'a Vec<Value>> {
    if let Some(delta_calls) = choice
        .get("delta")
        .and_then(|d| d.get("tool_calls"))
        .and_then(|t| t.as_array())
    {
        return Some(delta_calls);
    }

    if let Some(choice_calls) = choice.get("tool_calls").and_then(|t| t.as_array()) {
        return Some(choice_calls);
    }

    choice
        .get("message")
        .and_then(|m| m.get("tool_calls"))
        .and_then(|t| t.as_array())
}

fn should_retry_non_stream_after_stream_error(full_content: &str, tool_calls_count: usize) -> bool {
    full_content.is_empty() && tool_calls_count == 0
}

fn should_replay_final_content(
    generation_started_sent: bool,
    message_content: Option<&str>,
) -> bool {
    !generation_started_sent
        && message_content
            .map(|content| !content.is_empty())
            .unwrap_or(false)
}

fn should_enable_tool_grammar(is_chat_mode: bool, server_flavor: ServerFlavor) -> bool {
    if is_chat_mode {
        return false;
    }

    if let Ok(raw) = std::env::var("HELIX_FORCE_TOOL_GRAMMAR") {
        match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => return true,
            "0" | "false" | "no" | "off" => return false,
            _ => {}
        }
    }

    server_flavor == ServerFlavor::KoboldCpp
}

fn flush_token_buffer(
    event_tx: &tokio::sync::mpsc::UnboundedSender<tui::TuiEvent>,
    token_buffer: &mut String,
) {
    if !token_buffer.is_empty() {
        let _ = event_tx.send(tui::TuiEvent::TokenChunk(token_buffer.clone()));
        token_buffer.clear();
    }
}

fn read_server_stderr_log() -> Option<String> {
    let candidates = [
        "logs/start_server.stderr.log",
        "../logs/start_server.stderr.log",
    ];
    for path in candidates {
        if std::path::Path::new(path).exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if !content.trim().is_empty() {
                    return Some(content);
                }
            }
        }
    }
    None
}

fn latest_oom_excerpt() -> Option<String> {
    let content = read_server_stderr_log()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let mut hit_idx: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate().rev() {
        let lower = line.to_lowercase();
        if lower.contains("out of memory")
            || lower.contains("cuda error")
            || lower.contains("failed to allocate")
            || lower.contains("bad allocation")
        {
            hit_idx = Some(idx);
            break;
        }
    }

    let idx = hit_idx?;
    let start = idx.saturating_sub(2);
    let end = std::cmp::min(lines.len(), idx + 3);
    let excerpt = lines[start..end].join("\n");
    if excerpt.trim().is_empty() {
        None
    } else {
        Some(excerpt)
    }
}

fn read_config_layer_values() -> (Option<String>, Option<String>) {
    let candidates = ["scripts/config.py", "../scripts/config.py"];
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut gpu_layers: Option<String> = None;
            let mut fallback_gpu_layers: Option<String> = None;

            for raw in content.lines() {
                let line = raw.trim();
                if line.starts_with('#') {
                    continue;
                }

                if gpu_layers.is_none() && line.starts_with("GPU_LAYERS") {
                    if let Some((_, rhs)) = line.split_once('=') {
                        gpu_layers =
                            Some(rhs.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                }

                if fallback_gpu_layers.is_none() && line.starts_with("FALLBACK_GPU_LAYERS") {
                    if let Some((_, rhs)) = line.split_once('=') {
                        fallback_gpu_layers =
                            Some(rhs.trim().trim_matches('"').trim_matches('\'').to_string());
                    }
                }
            }

            if gpu_layers.is_some() || fallback_gpu_layers.is_some() {
                return (gpu_layers, fallback_gpu_layers);
            }
        }
    }

    (None, None)
}

fn read_gpu_layer_hint() -> String {
    let (config_gpu_layers, config_fallback_layers) = read_config_layer_values();

    let gpu_layers = std::env::var("GPU_LAYERS")
        .ok()
        .or(config_gpu_layers)
        .unwrap_or_else(|| "unset".to_string());

    let fallback_gpu_layers = std::env::var("FALLBACK_GPU_LAYERS")
        .ok()
        .or(config_fallback_layers)
        .unwrap_or_else(|| "0 (default)".to_string());

    format!(
        "Config values: GPU_LAYERS={}, FALLBACK_GPU_LAYERS={}",
        gpu_layers, fallback_gpu_layers
    )
}

fn system_prompt_for_mode(exec_mode: &str, persona: &str) -> String {
    if exec_mode != "agentic" {
        return String::new();
    }

    match persona {
        "coder" => {
            "You are an autonomous code executor. You read and write files using provided tools. You cannot execute terminal commands. State your reasoning in one sentence before each tool call. Do not greet the user. Do not introduce yourself. Be concise."
                .to_string()
        }
        "researcher" => {
            "You are an autonomous read-only system explorer. You read files and gather system stats using provided tools. You cannot modify files or execute commands. State your reasoning in one sentence before each tool call. Do not greet the user. Do not introduce yourself. Be concise."
                .to_string()
        }
        _ => {
            "You are an autonomous local system orchestrator. You execute tasks using provided tools. Before each tool call, state your reasoning in one sentence. Never guess file paths — verify with list_directory first. If a command fails, read STDERR and retry with a corrected approach. Do not greet the user. Do not introduce yourself. Do not use conversational filler. Be concise. You have local tool access through these tools, so do not ask the user to run local file-system commands when a tool can do it."
                .to_string()
        }
    }
}

fn sync_system_prompt_message(messages: &mut Vec<ChatMessage>, system_prompt: &str) {
    if matches!(messages.first(), Some(msg) if msg.role == "system") {
        messages.remove(0);
    }

    if !system_prompt.is_empty() {
        messages.insert(
            0,
            ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        );
    }
}

fn chat_max_tokens() -> usize {
    std::env::var("HELIX_CHAT_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(128)
}

async fn send_with_retry(
    client: &Client,
    url: &str,
    request_body: &Value,
    max_attempts: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    let attempts = std::cmp::max(1, max_attempts);
    let retry_delay_ms = std::env::var("HELIX_HTTP_RETRY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1000);
    let mut last_err: Option<reqwest::Error> = None;

    for attempt in 1..=attempts {
        match client.post(url).json(request_body).send().await {
            Ok(response) => return Ok(response),
            Err(err) => {
                let retryable = is_transient_http_error(&err);
                if attempt < attempts && retryable {
                    tokio::time::sleep(Duration::from_millis(retry_delay_ms)).await;
                    continue;
                }
                last_err = Some(err);
                break;
            }
        }
    }

    Err(last_err.expect("send_with_retry called without attempts"))
}

fn is_transient_http_error(err: &reqwest::Error) -> bool {
    if err.is_connect() || err.is_timeout() {
        return true;
    }

    let msg = err.to_string().to_lowercase();
    msg.contains("connection reset")
        || msg.contains("connection closed")
        || msg.contains("broken pipe")
        || msg.contains("operation timed out")
        || msg.contains("incomplete message")
}

async fn is_model_server_reachable(client: &Client, base_url: &str) -> bool {
    let models_url = format!("{}/models", base_url.trim_end_matches('/'));
    match tokio::time::timeout(Duration::from_secs(2), client.get(&models_url).send()).await {
        Ok(Ok(resp)) => resp.status().is_success(),
        _ => false,
    }
}

async fn probe_model_chat_ready(client: &Client, app_config: &config::AppConfig) -> bool {
    let chat_url = format!(
        "{}/chat/completions",
        app_config.base_url.trim_end_matches('/')
    );

    let probe_body = json!({
        "model": app_config.model_name,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 1,
        "stream": false,
        "temperature": 0.0
    });

    match tokio::time::timeout(
        Duration::from_secs(20),
        client.post(&chat_url).json(&probe_body).send(),
    )
    .await
    {
        Ok(Ok(resp)) => resp.status().is_success(),
        _ => false,
    }
}

fn resolve_server_launch_context() -> Option<(&'static str, &'static str)> {
    if std::path::Path::new("scripts/start_server.py").exists() {
        return Some((".", "scripts/start_server.py"));
    }
    if std::path::Path::new("../scripts/start_server.py").exists() {
        return Some(("..", "scripts/start_server.py"));
    }
    None
}

fn open_log_file(path: &str) -> Option<std::fs::File> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
}

async fn maybe_boot_model_server(client: &Client, app_config: &config::AppConfig) -> bool {
    if probe_model_chat_ready(client, app_config).await {
        return true;
    }

    if MODEL_SERVER_BOOT_ATTEMPTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    let (project_dir, script_rel_path) = match resolve_server_launch_context() {
        Some(ctx) => ctx,
        None => return false,
    };

    let stdout_path = format!("{}/logs/start_server.stdout.log", project_dir);
    let stderr_path = format!("{}/logs/start_server.stderr.log", project_dir);

    let mut cmd = std::process::Command::new("python");
    cmd.current_dir(project_dir).arg(script_rel_path);

    // Recovery path prioritizes guaranteed availability over GPU speed.
    cmd.env("HELIX_BACKEND_HINT", "cpu")
        .env("HELIX_GPU_LAYERS", "0")
        .env("HELIX_FALLBACK_BACKEND_HINT", "cpu")
        .env("HELIX_FALLBACK_GPU_LAYERS", "0");

    if let Some(stdout_file) = open_log_file(&stdout_path) {
        cmd.stdout(std::process::Stdio::from(stdout_file));
    }

    if let Some(stderr_file) = open_log_file(&stderr_path) {
        cmd.stderr(std::process::Stdio::from(stderr_file));
    }

    if cmd.spawn().is_err() {
        return false;
    }

    let startup_timeout_s = std::env::var("HELIX_SERVER_STARTUP_TIMEOUT_S")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(180);

    let polls = std::cmp::max(1, startup_timeout_s);
    for _ in 0..polls {
        tokio::time::sleep(Duration::from_secs(1)).await;

        if !is_model_server_reachable(client, &app_config.base_url).await {
            continue;
        }

        if probe_model_chat_ready(client, app_config).await {
            return true;
        }
    }

    false
}

async fn send_with_forced_retry(
    client: &Client,
    url: &str,
    request_body: &Value,
    max_attempts: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    let attempts = std::cmp::max(1, max_attempts);
    let retry_delay_ms = std::env::var("HELIX_HTTP_RETRY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1000);

    let mut last_err: Option<reqwest::Error> = None;

    for attempt in 1..=attempts {
        match client.post(url).json(request_body).send().await {
            Ok(response) => return Ok(response),
            Err(err) => {
                last_err = Some(err);
                if attempt < attempts {
                    tokio::time::sleep(Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }

    Err(last_err.expect("send_with_forced_retry called without attempts"))
}

async fn send_with_recovery(
    client: &Client,
    url: &str,
    request_body: &Value,
    app_config: &config::AppConfig,
) -> Result<reqwest::Response, reqwest::Error> {
    match send_with_retry(client, url, request_body, 3).await {
        Ok(response) => Ok(response),
        Err(err) => {
            if is_transient_http_error(&err) && maybe_boot_model_server(client, app_config).await {
                let recovery_attempts = std::env::var("HELIX_RECOVERY_RETRY_ATTEMPTS")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(45);
                return send_with_forced_retry(client, url, request_body, recovery_attempts).await;
            }
            Err(err)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading configuration from python runtime...");
    let app_config = config::AppConfig::load_from_python()?;
    let client = Client::builder().user_agent("HelixAgent/0.1.0").build()?;

    /*
    println!("\n[RAG] Booting FastEmbed Semantic Knowledge Base... (this may take a moment)");
    let rag_store = rag::RagStore::new(&app_config.allowed_dir)?;
    println!("  [✓] Local RAG sequence complete. Vector Store loaded in-memory.");
    */

    let persona = std::env::var("AGENT_PERSONA").unwrap_or_else(|_| "os_assistant".to_string());
    let mut exec_mode = app_config.exec_mode.clone();
    let mut is_chat_mode = exec_mode == "chat";
    let url = format!(
        "{}/chat/completions",
        app_config.base_url.trim_end_matches('/')
    );
    let server_flavor = detect_server_flavor(&client, &app_config.base_url).await;
    let strict_tools = server_flavor != ServerFlavor::KoboldCpp;
    let mut tools_payload = if is_chat_mode {
        json!([])
    } else {
        build_tools(&persona, strict_tools)?
    };
    if server_flavor == ServerFlavor::KoboldCpp {
        println!(
            "[Runtime] Detected KoboldCPP endpoint. Stripping 'strict' tags for schema compatibility, but enforcing native GBNF grammar for 100% accuracy."
        );
    }

    let mut generated_grammar = if should_enable_tool_grammar(is_chat_mode, server_flavor) {
        println!("[Runtime] Compiling JSON schemata to GBNF Grammar...");
        tools::generate_tool_grammar(&tools_payload)
    } else {
        String::new()
    };
    if !is_chat_mode && generated_grammar.is_empty() {
        println!("[Runtime] Tool grammar disabled for this backend; using native tool-calling.");
    }

    let mut system_prompt = system_prompt_for_mode(&exec_mode, &persona);

    let ui_mode = std::env::var("HELIX_UI_MODE").unwrap_or_else(|_| "terminal".to_string());

    if ui_mode == "web" {
        server::start_web_server(
            app_config,
            persona,
            generated_grammar,
            tools_payload,
            server_flavor,
        )
        .await;
        return Ok(());
    }

    let mut messages = vec![];
    sync_system_prompt_message(&mut messages, &system_prompt);

    let args: Vec<String> = std::env::args().collect();
    let mut initial_prompt = args
        .iter()
        .position(|r| r == "--prompt")
        .and_then(|idx| args.get(idx + 1))
        .cloned();
    let eval_mode = initial_prompt.is_some();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // TUI Mode: ratatui-based interactive terminal with streaming & spans
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    if ui_mode == "tui" {
        let (mut action_rx, event_tx) = tui::run_tui().await?;

        // Initialize HUD context
        let current_tokens = tokens::count_message_tokens(&messages);
        let _ = event_tx.send(tui::TuiEvent::ContextUpdate(
            current_tokens,
            app_config.context_size as usize,
        ));
        let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
            "[Mode] {} mode active. Use `/mode chat` or `/mode agentic`.",
            exec_mode
        )));

        // If there's an initial prompt, send it immediately
        if let Some(prompt) = initial_prompt.take() {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
            // Process the initial prompt through the LLM loop
            run_llm_loop_tui(
                &client,
                &url,
                &app_config,
                &mut messages,
                &tools_payload,
                &generated_grammar,
                is_chat_mode,
                server_flavor,
                &mut action_rx,
                &event_tx,
            )
            .await;
            if eval_mode {
                return Ok(());
            }
        }

        // Main TUI event loop: wait for user actions
        loop {
            match action_rx.recv().await {
                Some(tui::TuiAction::Submit(user_input)) => {
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: Some(user_input),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                    run_llm_loop_tui(
                        &client,
                        &url,
                        &app_config,
                        &mut messages,
                        &tools_payload,
                        &generated_grammar,
                        is_chat_mode,
                        server_flavor,
                        &mut action_rx,
                        &event_tx,
                    )
                    .await;
                }
                Some(tui::TuiAction::Quit) | None => {
                    break;
                }
                Some(tui::TuiAction::Interrupt) => {
                    // Ignore stray interrupts when idle
                }
                Some(tui::TuiAction::SystemCommand(cmd)) => {
                    let c = cmd.trim();
                    if c == "/clear" {
                        if matches!(messages.first(), Some(msg) if msg.role == "system") {
                            messages.truncate(1);
                        } else {
                            messages.clear();
                        }
                        let _ = event_tx.send(tui::TuiEvent::ClearHistory);
                        let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                            "[Context Cleared]".to_string(),
                        ));
                        let current_tokens = tokens::count_message_tokens(&messages);
                        let _ = event_tx.send(tui::TuiEvent::ContextUpdate(
                            current_tokens,
                            app_config.context_size as usize,
                        ));
                    } else if c.starts_with("/mode") {
                        let parts: Vec<&str> = c.split_whitespace().collect();

                        if parts.len() == 1
                            || (parts.len() == 2 && parts[1].eq_ignore_ascii_case("status"))
                        {
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                                "[Mode] Current mode: {}. Use `/mode chat` or `/mode agentic`.",
                                exec_mode
                            )));
                            continue;
                        }

                        if parts.len() != 2 {
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                "[Mode] Usage: /mode chat | /mode agentic | /mode status"
                                    .to_string(),
                            ));
                            continue;
                        }

                        let requested_mode = parts[1].to_lowercase();
                        if requested_mode != "chat" && requested_mode != "agentic" {
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                "[Mode] Unknown mode. Use `chat` or `agentic`.".to_string(),
                            ));
                            continue;
                        }

                        if requested_mode == exec_mode {
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                                "[Mode] Already in {} mode.",
                                exec_mode
                            )));
                            continue;
                        }

                        exec_mode = requested_mode;
                        is_chat_mode = exec_mode == "chat";
                        tools_payload = if is_chat_mode {
                            json!([])
                        } else {
                            match build_tools(&persona, strict_tools) {
                                Ok(payload) => payload,
                                Err(err) => {
                                    let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                                        "[Mode] Failed to initialize tools: {}",
                                        err
                                    )));
                                    continue;
                                }
                            }
                        };

                        let grammar_enabled =
                            should_enable_tool_grammar(is_chat_mode, server_flavor);
                        generated_grammar = if grammar_enabled {
                            tools::generate_tool_grammar(&tools_payload)
                        } else {
                            String::new()
                        };
                        if !is_chat_mode && !grammar_enabled {
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                "[Mode] Tool grammar is disabled for this backend; using native tool-calling. Set HELIX_FORCE_TOOL_GRAMMAR=1 to force grammar.".to_string(),
                            ));
                        }

                        system_prompt = system_prompt_for_mode(&exec_mode, &persona);
                        sync_system_prompt_message(&mut messages, &system_prompt);
                        let current_tokens = tokens::count_message_tokens(&messages);
                        let _ = event_tx.send(tui::TuiEvent::ContextUpdate(
                            current_tokens,
                            app_config.context_size as usize,
                        ));
                        let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                            "[Mode] Switched to {} mode.",
                            exec_mode
                        )));
                    } else {
                        let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                            "[Unknown command] {}",
                            c
                        )));
                    }
                }
            }
        }

        return Ok(());
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // Terminal Mode: classic rustyline REPL (existing behavior preserved)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    // Initialize rich terminal editor (skipped in eval mode)
    let mut rl = if !eval_mode {
        print_helix_logo(true);
        println!();
        println!("\n=======================================================");
        println!("Helix Rust Agent Orchestrator running.");
        println!("Type 'quit' or 'exit' to gracefully stop.");
        println!("Paste multi-line text freely. Press Enter on an empty line to submit.");
        println!("=======================================================\n");
        Some(input::create_editor().expect("Failed to initialize terminal editor"))
    } else {
        None
    };

    loop {
        let user_input = if let Some(prompt) = initial_prompt.take() {
            prompt
        } else {
            if eval_mode {
                break;
            }
            let editor = rl.as_mut().unwrap();
            match editor.readline("> ") {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit")
                    {
                        input::save_history(editor);
                        break;
                    }
                    trimmed
                }
                Err(ReadlineError::Interrupted) => {
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    input::save_history(editor);
                    break;
                }
                Err(err) => {
                    println!("[Input Error] {}", err);
                    continue;
                }
            }
        };

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: Some(user_input),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        let mut round_trip_counter = 0;
        let base_temperature: f64 = if server_flavor == ServerFlavor::KoboldCpp {
            0.05
        } else {
            0.1
        };
        let mut temperature_override: f64 = base_temperature;

        loop {
            if round_trip_counter >= 20 {
                println!("\n[Rust Orchestrator] Safety exit: exceeded 20 action rounds.");
                break;
            }
            round_trip_counter += 1;
            if eval_mode {
                println!("\n\u{001b}[2m[Round {}/20]\u{001b}[0m", round_trip_counter);
            }

            let current_tokens = tokens::count_message_tokens(&messages);
            let threshold = (app_config.context_size as f32 * 0.70) as usize;

            if current_tokens > threshold && messages.len() > 5 {
                println!(
                    "\n[Memory Alert] Context at {} tokens (limit {}). Compacting...",
                    current_tokens, threshold
                );

                let mid_point = 1 + ((messages.len() - 1) as f32 * 0.60) as usize;
                let mut compaction_messages = vec![messages[0].clone()];
                compaction_messages.extend_from_slice(&messages[1..mid_point]);
                compaction_messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: Some("SYSTEM DIRECTIVE: Summarize the previous conversation compactly, preserving all factual data, file paths, logic, and decisions. Reply ONLY with the summary paragraph. Do not use tools.".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });

                let compaction_body = json!({
                    "model": &app_config.model_name,
                    "messages": compaction_messages,
                    "temperature": 0.1
                });

                if let Ok(res) = client.post(&url).json(&compaction_body).send().await {
                    if let Ok(text) = res.text().await {
                        if let Ok(parsed) = serde_json::from_str::<ChatResponse>(&text) {
                            if let Some(summary) = &parsed.choices[0].message.content {
                                println!("[Memory Alert] Compaction complete. History condensed.");
                                let mut new_messages = vec![messages[0].clone()];
                                new_messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: Some(format!(
                                        "[Internal Working Memory Summary]\n{}",
                                        summary
                                    )),
                                    tool_calls: None,
                                    tool_call_id: None,
                                    name: None,
                                });
                                new_messages.extend_from_slice(&messages[mid_point..]);
                                messages = new_messages;
                            }
                        }
                    }
                }
            }

            let mut request_body = json!({
                "model": app_config.model_name,
                "messages": &messages,
                "tools": tools_payload,
                "temperature": temperature_override,
                "max_tokens": if is_chat_mode { chat_max_tokens() } else { 8192 },
                "stream": true
            });

            if is_chat_mode {
                request_body.as_object_mut().unwrap().insert(
                    "chat_template_kwargs".to_string(),
                    json!({ "enable_thinking": false }),
                );
            }

            if !is_chat_mode && !generated_grammar.is_empty() {
                request_body
                    .as_object_mut()
                    .unwrap()
                    .insert("grammar".to_string(), json!(generated_grammar));
            }

            let res = match send_with_recovery(&client, &url, &request_body, &app_config).await {
                Ok(r) => r,
                Err(e) => {
                    println!("[Rust] HTTP Error: {}", e);
                    println!(
                        "[Rust] Model server unreachable at {}. Start with `python start.py` or `python scripts/start_server.py`. If logs show CUDA OOM, reduce GPU layers in scripts/config.py.",
                        app_config.base_url
                    );
                    println!("[Rust] {}", read_gpu_layer_hint());
                    break;
                }
            };

            use futures_util::stream::StreamExt;
            use std::io::Write;

            let mut full_content = String::new();
            let mut tool_calls_map: std::collections::HashMap<usize, Value> =
                std::collections::HashMap::new();

            let mut stream = res.bytes_stream();
            let mut sse_parser = stream::SseParser::new();
            let mut had_stream_error = false;
            println!();

            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(bytes) => {
                        for event in sse_parser.push_bytes(&bytes) {
                            if let stream::SseEvent::Data(data) = event {
                                if let Ok(json) = serde_json::from_str::<Value>(&data) {
                                    if let Some(choices) =
                                        json.get("choices").and_then(|c| c.as_array())
                                    {
                                        if let Some(choice) = choices.first() {
                                            if let Some((content, _)) =
                                                extract_visible_stream_choice_text(
                                                    choice,
                                                    is_chat_mode,
                                                )
                                            {
                                                if !is_chat_mode {
                                                    print!("{}", content);
                                                    std::io::stdout().flush().unwrap();
                                                }
                                                full_content.push_str(&content);
                                            }
                                            if let Some(tcs) = extract_stream_tool_calls(choice) {
                                                for tc in tcs {
                                                    if let Some(index) =
                                                        tc.get("index").and_then(|i| i.as_u64())
                                                    {
                                                        let idx = index as usize;
                                                        let entry = tool_calls_map.entry(idx).or_insert(json!({
                                                            "id": "",
                                                            "type": "function",
                                                            "function": { "name": "", "arguments": "" }
                                                        }));
                                                        if let Some(id) =
                                                            tc.get("id").and_then(|id| id.as_str())
                                                        {
                                                            entry["id"] = json!(id);
                                                        }
                                                        if let Some(func) = tc.get("function") {
                                                            if let Some(name) = func
                                                                .get("name")
                                                                .and_then(|n| n.as_str())
                                                            {
                                                                let current =
                                                                    entry["function"]["name"]
                                                                        .as_str()
                                                                        .unwrap_or("");
                                                                entry["function"]["name"] = json!(
                                                                    format!("{}{}", current, name)
                                                                );
                                                            }
                                                            if let Some(args) = func
                                                                .get("arguments")
                                                                .and_then(|a| a.as_str())
                                                            {
                                                                let current =
                                                                    entry["function"]["arguments"]
                                                                        .as_str()
                                                                        .unwrap_or("");
                                                                entry["function"]["arguments"] = json!(
                                                                    format!("{}{}", current, args)
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        had_stream_error = true;
                        if should_retry_non_stream_after_stream_error(
                            &full_content,
                            tool_calls_map.len(),
                        ) {
                            println!("\n[Rust] Stream error: {}", e);
                            let mut fallback_body = request_body.clone();
                            fallback_body["stream"] = json!(false);
                            if let Ok(fallback_res) =
                                client.post(&url).json(&fallback_body).send().await
                            {
                                if let Ok(fallback_text) = fallback_res.text().await {
                                    let mut parsed_any = false;
                                    if let Ok(parsed) =
                                        serde_json::from_str::<Value>(&fallback_text)
                                    {
                                        if let Some(choice) = parsed
                                            .get("choices")
                                            .and_then(|v| v.as_array())
                                            .and_then(|arr| arr.first())
                                        {
                                            if let Some(content) = choice
                                                .get("message")
                                                .and_then(|m| m.get("content"))
                                                .and_then(|c| c.as_str())
                                            {
                                                full_content.push_str(content);
                                                parsed_any = true;
                                            } else if let Some(content) =
                                                choice.get("text").and_then(|c| c.as_str())
                                            {
                                                full_content.push_str(content);
                                                parsed_any = true;
                                            }
                                            if let Some(tcs) = choice
                                                .get("message")
                                                .and_then(|m| m.get("tool_calls"))
                                                .and_then(|tc| tc.as_array())
                                            {
                                                for (idx, tc) in tcs.iter().enumerate() {
                                                    tool_calls_map.insert(idx, tc.clone());
                                                }
                                                parsed_any = true;
                                            }
                                        }
                                    }
                                    if !parsed_any && !fallback_text.trim().is_empty() {
                                        full_content.push_str(&fallback_text);
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }

            for event in sse_parser.finish() {
                if let stream::SseEvent::Data(data) = event {
                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            if let Some(choice) = choices.first() {
                                if let Some((content, _)) =
                                    extract_visible_stream_choice_text(choice, is_chat_mode)
                                {
                                    if !is_chat_mode {
                                        print!("{}", content);
                                        std::io::stdout().flush().unwrap();
                                    }
                                    full_content.push_str(&content);
                                }
                                if let Some(tcs) = extract_stream_tool_calls(choice) {
                                    for tc in tcs {
                                        if let Some(index) =
                                            tc.get("index").and_then(|i| i.as_u64())
                                        {
                                            let idx = index as usize;
                                            let entry =
                                                tool_calls_map.entry(idx).or_insert(json!({
                                                    "id": "",
                                                    "type": "function",
                                                    "function": { "name": "", "arguments": "" }
                                                }));
                                            if let Some(id) =
                                                tc.get("id").and_then(|id| id.as_str())
                                            {
                                                entry["id"] = json!(id);
                                            }
                                            if let Some(func) = tc.get("function") {
                                                if let Some(name) =
                                                    func.get("name").and_then(|n| n.as_str())
                                                {
                                                    let current = entry["function"]["name"]
                                                        .as_str()
                                                        .unwrap_or("");
                                                    entry["function"]["name"] =
                                                        json!(format!("{}{}", current, name));
                                                }
                                                if let Some(args) =
                                                    func.get("arguments").and_then(|a| a.as_str())
                                                {
                                                    let current = entry["function"]["arguments"]
                                                        .as_str()
                                                        .unwrap_or("");
                                                    entry["function"]["arguments"] =
                                                        json!(format!("{}{}", current, args));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!();

            let mut final_tool_calls = Vec::new();
            let mut indices: Vec<usize> = tool_calls_map.keys().copied().collect();
            indices.sort();
            for idx in indices {
                final_tool_calls.push(tool_calls_map[&idx].clone());
            }

            let message_content = if full_content.is_empty() {
                if final_tool_calls.is_empty() {
                    if had_stream_error {
                        Some("I hit a stream decoding error and non-stream recovery returned no visible output. Please retry.".to_string())
                    } else {
                        Some("I could not produce a visible response. Please retry.".to_string())
                    }
                } else {
                    None
                }
            } else {
                let visible = format_visible_output(&full_content, is_chat_mode);
                if is_chat_mode && !visible.is_empty() {
                    print!("{}", visible);
                    std::io::stdout().flush().unwrap();
                }
                Some(visible)
            };

            if full_content.is_empty() {
                if let Some(fallback_msg) = &message_content {
                    print!("{}", fallback_msg);
                    std::io::stdout().flush().unwrap();
                }
            }
            let message_tool_calls: Option<Vec<Value>> = if final_tool_calls.is_empty() {
                None
            } else {
                Some(final_tool_calls)
            };

            if message_tool_calls.is_none() {
                temperature_override = base_temperature;
            }

            let mut history_message = ChatMessage {
                role: "assistant".to_string(),
                content: message_content.clone(),
                tool_calls: message_tool_calls.clone(),
                tool_call_id: None,
                name: None,
            };

            if history_message.tool_calls.is_some() {
                history_message.tool_calls = None;
                if history_message.content.is_none() {
                    history_message.content = Some("[assistant executed tool calls]".to_string());
                }
            }

            messages.push(history_message);

            if let Some(tool_calls) = &message_tool_calls {
                if tool_calls.is_empty() {
                    break;
                }
                let mut critic_injections: Vec<ChatMessage> = Vec::new();

                for tc in tool_calls {
                    let id = tc["id"].as_str().unwrap_or("").to_string();
                    let func_name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                    let args_value = &tc["function"]["arguments"];
                    let parsed_args = if let Some(raw_str) = args_value.as_str() {
                        serde_json::from_str::<Value>(raw_str).unwrap_or(json!({}))
                    } else if args_value.is_object() {
                        args_value.clone()
                    } else {
                        json!({})
                    };

                    if eval_mode {
                        println!("➜ Tool: {}", func_name);
                    }

                    let simulated_payload = json!({
                        "name": func_name,
                        "arguments": parsed_args
                    });

                    let tool_result = match serde_json::from_value::<ToolCallArgs>(
                        simulated_payload,
                    ) {
                        Ok(ToolCallArgs::RunTerminalCommand(input)) => {
                            tools::execute_run_terminal_command(
                                input,
                                &app_config.dangerous_commands,
                                app_config.require_confirmation,
                            )
                        }
                        Ok(ToolCallArgs::ReadFile(input)) => tools::execute_read_file(input),
                        Ok(ToolCallArgs::WriteFile(input)) => tools::execute_write_file(input),
                        Ok(ToolCallArgs::AppendFile(input)) => tools::execute_append_file(input),
                        Ok(ToolCallArgs::ListDirectory(input)) => {
                            tools::execute_list_directory(input)
                        }
                        Ok(ToolCallArgs::GetSystemStats(_)) => tools::execute_get_system_stats(),
                        Ok(ToolCallArgs::SearchCodebase(_)) => tools::ToolResult {
                            success: false,
                            output: "Tool 'search_codebase' is currently disabled.".to_string(),
                        },
                        Err(e) => {
                            println!(
                                "[Critic] SCHEMA ERROR — injecting self-correction directive."
                            );
                            let correction = format!(
                                "You sent invalid arguments to the '{}' tool. Schema error: {}. \
                                Carefully re-read the tool's required parameters and call it again with the correct argument names and types.",
                                func_name, e
                            );
                            critic_injections.push(critic_message(&correction));
                            temperature_override = if server_flavor == ServerFlavor::KoboldCpp {
                                0.2
                            } else {
                                0.3
                            };
                            tools::ToolResult {
                                success: false,
                                output: format!(
                                    "[Schema mismatch — see correction directive above]"
                                ),
                            }
                        }
                    };

                    if !tool_result.success && func_name == "run_terminal_command" {
                        println!("[Critic] Command failed — injecting retry directive.");
                        critic_injections.push(critic_message(
                            "The previous command failed. Analyze the error output above carefully. Correct your approach and retry now. Do NOT repeat the same command."
                        ));
                        temperature_override = if server_flavor == ServerFlavor::KoboldCpp {
                            0.2
                        } else {
                            0.3
                        };
                    }

                    if tool_result.success && func_name == "write_file" {
                        println!("[Critic] WriteFile succeeded — injecting verify-back directive.");
                        critic_injections.push(critic_message(
                            "File was written. You MUST now verify it is correct: use the read_file tool to read back the file you just wrote before continuing to the next step."
                        ));
                    }

                    messages.push(ChatMessage {
                        role: "tool".to_string(),
                        content: Some(tool_result.output),
                        tool_calls: None,
                        tool_call_id: Some(id),
                        name: Some(func_name),
                    });
                }
                messages.extend(critic_injections);
            } else {
                break;
            }
        }
    }

    // Persist history on clean exit
    if let Some(ref mut editor) = rl {
        input::save_history(editor);
    }

    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TUI LLM Loop: Streams tokens to the TUI with 30ms batched flushing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

async fn run_llm_loop_tui(
    client: &Client,
    url: &str,
    app_config: &config::AppConfig,
    messages: &mut Vec<ChatMessage>,
    tools_payload: &Value,
    generated_grammar: &str,
    is_chat_mode: bool,
    server_flavor: ServerFlavor,
    action_rx: &mut tokio::sync::mpsc::UnboundedReceiver<tui::TuiAction>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<tui::TuiEvent>,
) {
    let base_temperature: f64 = if server_flavor == ServerFlavor::KoboldCpp {
        0.05
    } else {
        0.1
    };
    let mut temperature_override: f64 = base_temperature;

    for round in 0..20 {
        if round >= 20 {
            let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                "[Safety exit] Exceeded 20 action rounds.".to_string(),
            ));
            break;
        }

        let current_tokens = tokens::count_message_tokens(messages);
        let _ = event_tx.send(tui::TuiEvent::ContextUpdate(
            current_tokens,
            app_config.context_size as usize,
        ));

        // Context compaction check
        let threshold = (app_config.context_size as f32 * 0.70) as usize;

        if current_tokens > threshold && messages.len() > 5 {
            let _ = event_tx.send(tui::TuiEvent::Status(format!(
                "Compacting context ({} tokens)...",
                current_tokens
            )));

            let mid_point = 1 + ((messages.len() - 1) as f32 * 0.60) as usize;
            let mut compaction_messages = vec![messages[0].clone()];
            compaction_messages.extend_from_slice(&messages[1..mid_point]);
            compaction_messages.push(ChatMessage {
                role: "user".to_string(),
                content: Some("SYSTEM DIRECTIVE: Summarize the previous conversation compactly, preserving all factual data, file paths, logic, and decisions. Reply ONLY with the summary paragraph. Do not use tools.".to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });

            let compaction_body = json!({
                "model": &app_config.model_name,
                "messages": compaction_messages,
                "temperature": 0.1
            });

            if let Ok(res) = client.post(url).json(&compaction_body).send().await {
                if let Ok(text) = res.text().await {
                    if let Ok(parsed) = serde_json::from_str::<ChatResponse>(&text) {
                        if let Some(summary) = &parsed.choices[0].message.content {
                            let mut new_messages = vec![messages[0].clone()];
                            new_messages.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: Some(format!(
                                    "[Internal Working Memory Summary]\n{}",
                                    summary
                                )),
                                tool_calls: None,
                                tool_call_id: None,
                                name: None,
                            });
                            new_messages.extend_from_slice(&messages[mid_point..]);
                            *messages = new_messages;
                        }
                    }
                }
            }
        }

        // Build request
        let mut request_body = json!({
            "model": app_config.model_name,
            "messages": &messages,
            "tools": tools_payload,
            "temperature": temperature_override,
            "max_tokens": if is_chat_mode { chat_max_tokens() } else { 8192 },
            "stream": true
        });

        if is_chat_mode {
            request_body.as_object_mut().unwrap().insert(
                "chat_template_kwargs".to_string(),
                json!({ "enable_thinking": false }),
            );
        }

        if !is_chat_mode && !generated_grammar.is_empty() {
            request_body
                .as_object_mut()
                .unwrap()
                .insert("grammar".to_string(), json!(generated_grammar));
        }

        let _ = event_tx.send(tui::TuiEvent::Status("Generating...".to_string()));

        let res = match send_with_recovery(client, url, &request_body, app_config).await {
            Ok(r) => {
                TUI_HTTP_CONNECT_FAILS.store(0, std::sync::atomic::Ordering::Relaxed);
                r
            }
            Err(e) => {
                let fails =
                    TUI_HTTP_CONNECT_FAILS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!("[HTTP Error] {}", e)));
                let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!("[Hint] Model server unreachable at {}. Start with `python start.py` or `python scripts/start_server.py`. If logs show CUDA OOM, reduce GPU layers in scripts/config.py.", app_config.base_url)));
                let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                    "[Hint] {}",
                    read_gpu_layer_hint()
                )));
                if fails >= 2 {
                    if let Some(oom_excerpt) = latest_oom_excerpt() {
                        let _ = event_tx.send(tui::TuiEvent::SystemMessage(format!(
                            "[Diagnostics] Recent model-server OOM evidence:\n{}",
                            oom_excerpt
                        )));
                    }
                }
                break;
            }
        };

        use futures_util::stream::StreamExt;

        let mut full_content = String::new();
        let mut tool_calls_map: std::collections::HashMap<usize, Value> =
            std::collections::HashMap::new();
        let mut stream = res.bytes_stream();
        let mut sse_parser = stream::SseParser::new();
        let mut had_stream_error = false;
        let mut generation_started_sent = false;
        let mut in_reasoning_block = false;
        let mut last_heartbeat = std::time::Instant::now();

        // 30ms batch flushing for token streaming
        let mut token_buffer = String::new();
        let mut flush_interval = tokio::time::interval(Duration::from_millis(30));
        flush_interval.tick().await; // consume the immediate first tick

        loop {
            tokio::select! {
                // Check for user interrupts
                action_opt = action_rx.recv() => {
                    match action_opt {
                        Some(tui::TuiAction::Interrupt) | Some(tui::TuiAction::Quit) => {
                            flush_token_buffer(event_tx, &mut token_buffer);
                            let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                "[Generation Interrupted]".to_string()
                            ));
                            break; // Stop listening to stream
                        }
                        _ => {}
                    }
                }
                chunk_opt = stream.next() => {
                    match chunk_opt {
                        Some(Ok(bytes)) => {
                            for event in sse_parser.push_bytes(&bytes) {
                                if let stream::SseEvent::Data(data) = event {
                                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                                            if let Some(choice) = choices.first() {
                                                if let Some((content, is_reasoning)) =
                                                    extract_visible_stream_choice_text(choice, is_chat_mode)
                                                {
                                                    if !generation_started_sent {
                                                        generation_started_sent = true;
                                                        let _ = event_tx.send(tui::TuiEvent::GenerationStarted);
                                                    }

                                                    if is_reasoning && !in_reasoning_block {
                                                        in_reasoning_block = true;
                                                        full_content.push_str("<think>");
                                                        token_buffer.push_str("<think>");
                                                        let _ = event_tx.send(tui::TuiEvent::StatusUpdate("Thinking...".to_string()));
                                                    } else if !is_reasoning && in_reasoning_block {
                                                        in_reasoning_block = false;
                                                        full_content.push_str("</think>\n\n");
                                                        token_buffer.push_str("</think>\n\n");
                                                        let _ = event_tx.send(tui::TuiEvent::StatusUpdate("Generating...".to_string()));
                                                    }

                                                    full_content.push_str(&content);
                                                    token_buffer.push_str(&content);
                                                }
                                                if let Some(tcs) = extract_stream_tool_calls(choice) {
                                                    for tc in tcs {
                                                        if let Some(index) = tc.get("index").and_then(|i| i.as_u64()) {
                                                            let idx = index as usize;
                                                            let entry = tool_calls_map.entry(idx).or_insert(json!({
                                                                "id": "",
                                                                "type": "function",
                                                                "function": { "name": "", "arguments": "" }
                                                            }));
                                                            if let Some(id) = tc.get("id").and_then(|id| id.as_str()) {
                                                                entry["id"] = json!(id);
                                                            }
                                                            if let Some(func) = tc.get("function") {
                                                                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                                                    let current = entry["function"]["name"].as_str().unwrap_or("");
                                                                    entry["function"]["name"] = json!(format!("{}{}", current, name));
                                                                }
                                                                if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                                                    let current = entry["function"]["arguments"].as_str().unwrap_or("");
                                                                    entry["function"]["arguments"] = json!(format!("{}{}", current, args));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            had_stream_error = true;
                            if should_retry_non_stream_after_stream_error(
                                &full_content,
                                tool_calls_map.len(),
                            ) {
                                let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                    format!("[Stream error] {}", e)
                                ));
                                let _ = event_tx.send(tui::TuiEvent::SystemMessage(
                                    "[Recovery] Retrying without stream...".to_string()
                                ));
                                let mut fallback_body = request_body.clone();
                                fallback_body["stream"] = json!(false);
                                if let Ok(fallback_res) = client.post(url).json(&fallback_body).send().await {
                                    if let Ok(fallback_text) = fallback_res.text().await {
                                        let mut parsed_any = false;
                                        if let Ok(parsed) = serde_json::from_str::<Value>(&fallback_text) {
                                            if let Some(choice) = parsed
                                                .get("choices")
                                                .and_then(|v| v.as_array())
                                                .and_then(|arr| arr.first())
                                            {
                                                if let Some(content) = choice
                                                    .get("message")
                                                    .and_then(|m| m.get("content"))
                                                    .and_then(|c| c.as_str())
                                                {
                                                    full_content.push_str(content);
                                                    parsed_any = true;
                                                } else if let Some(content) = choice
                                                    .get("text")
                                                    .and_then(|c| c.as_str())
                                                {
                                                    full_content.push_str(content);
                                                    parsed_any = true;
                                                }
                                                if let Some(tcs) = choice
                                                    .get("message")
                                                    .and_then(|m| m.get("tool_calls"))
                                                    .and_then(|tc| tc.as_array())
                                                {
                                                    for (idx, tc) in tcs.iter().enumerate() {
                                                        tool_calls_map.insert(idx, tc.clone());
                                                    }
                                                    parsed_any = true;
                                                }
                                            }
                                        }
                                        if !parsed_any && !fallback_text.trim().is_empty() {
                                            full_content.push_str(&fallback_text);
                                        }
                                    }
                                }
                            }
                            break;
                        }
                        None => {
                            for event in sse_parser.finish() {
                                if let stream::SseEvent::Data(data) = event {
                                    if let Ok(json) = serde_json::from_str::<Value>(&data) {
                                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                                            if let Some(choice) = choices.first() {
                                                if let Some((content, is_reasoning)) =
                                                    extract_visible_stream_choice_text(choice, is_chat_mode)
                                                {
                                                    if !generation_started_sent {
                                                        generation_started_sent = true;
                                                        let _ = event_tx.send(tui::TuiEvent::GenerationStarted);
                                                    }

                                                    if is_reasoning && !in_reasoning_block {
                                                        in_reasoning_block = true;
                                                        full_content.push_str("<think>");
                                                        token_buffer.push_str("<think>");
                                                        let _ = event_tx.send(tui::TuiEvent::StatusUpdate("Thinking...".to_string()));
                                                    } else if !is_reasoning && in_reasoning_block {
                                                        in_reasoning_block = false;
                                                        full_content.push_str("</think>\n\n");
                                                        token_buffer.push_str("</think>\n\n");
                                                        let _ = event_tx.send(tui::TuiEvent::StatusUpdate("Generating...".to_string()));
                                                    }

                                                    full_content.push_str(&content);
                                                    token_buffer.push_str(&content);
                                                }
                                                if let Some(tcs) = extract_stream_tool_calls(choice) {
                                                    for tc in tcs {
                                                        if let Some(index) = tc.get("index").and_then(|i| i.as_u64()) {
                                                            let idx = index as usize;
                                                            let entry = tool_calls_map.entry(idx).or_insert(json!({
                                                                "id": "",
                                                                "type": "function",
                                                                "function": { "name": "", "arguments": "" }
                                                            }));
                                                            if let Some(id) = tc.get("id").and_then(|id| id.as_str()) {
                                                                entry["id"] = json!(id);
                                                            }
                                                            if let Some(func) = tc.get("function") {
                                                                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                                                    let current = entry["function"]["name"].as_str().unwrap_or("");
                                                                    entry["function"]["name"] = json!(format!("{}{}", current, name));
                                                                }
                                                                if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                                                    let current = entry["function"]["arguments"].as_str().unwrap_or("");
                                                                    entry["function"]["arguments"] = json!(format!("{}{}", current, args));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Stream ended — flush any remaining buffer
                            flush_token_buffer(event_tx, &mut token_buffer);
                            break;
                        }
                    }
                }
                _ = flush_interval.tick() => {
                    // 30ms timer fired — flush accumulated tokens to the TUI
                    if !token_buffer.is_empty() {
                        flush_token_buffer(event_tx, &mut token_buffer);
                    } else if last_heartbeat.elapsed() >= Duration::from_millis(300) {
                        let _ = event_tx.send(tui::TuiEvent::StreamingHeartbeat(
                            "Model is still working...".to_string()
                        ));
                        last_heartbeat = std::time::Instant::now();
                    }
                }
            }
        }

        // Assemble tool calls
        let mut final_tool_calls = Vec::new();
        let mut indices: Vec<usize> = tool_calls_map.keys().copied().collect();
        indices.sort();
        for idx in indices {
            final_tool_calls.push(tool_calls_map[&idx].clone());
        }

        let message_content = if full_content.is_empty() {
            if final_tool_calls.is_empty() {
                if had_stream_error {
                    Some("I hit a stream decoding error and non-stream recovery returned no visible output. Please retry.".to_string())
                } else {
                    Some("I could not produce a visible response. Please retry.".to_string())
                }
            } else {
                None
            }
        } else {
            Some(format_visible_output(&full_content, is_chat_mode))
        };

        if should_replay_final_content(generation_started_sent, message_content.as_deref()) {
            let _ = event_tx.send(tui::TuiEvent::GenerationStarted);
            let _ = event_tx.send(tui::TuiEvent::TokenChunk(
                message_content.clone().unwrap_or_default(),
            ));
        }

        let message_tool_calls: Option<Vec<Value>> = if final_tool_calls.is_empty() {
            None
        } else {
            Some(final_tool_calls)
        };

        if message_tool_calls.is_none() {
            temperature_override = base_temperature;
        }

        // Signal response done to TUI
        let _ = event_tx.send(tui::TuiEvent::ResponseDone);

        let mut history_message = ChatMessage {
            role: "assistant".to_string(),
            content: message_content.clone(),
            tool_calls: message_tool_calls.clone(),
            tool_call_id: None,
            name: None,
        };

        if history_message.tool_calls.is_some() {
            history_message.tool_calls = None;
            if history_message.content.is_none() {
                history_message.content = Some("[assistant executed tool calls]".to_string());
            }
        }

        messages.push(history_message);

        // Execute tool calls
        if let Some(tool_calls) = &message_tool_calls {
            if tool_calls.is_empty() {
                break;
            }
            let mut critic_injections: Vec<ChatMessage> = Vec::new();

            for tc in tool_calls {
                let id = tc["id"].as_str().unwrap_or("").to_string();
                let func_name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                let args_value = &tc["function"]["arguments"];
                let parsed_args = if let Some(raw_str) = args_value.as_str() {
                    serde_json::from_str::<Value>(raw_str).unwrap_or(json!({}))
                } else if args_value.is_object() {
                    args_value.clone()
                } else {
                    json!({})
                };

                // Emit ToolStart event to TUI
                let _ = event_tx.send(tui::TuiEvent::ToolStart(tui::ToolInfo {
                    name: func_name.clone(),
                    arguments: parsed_args.to_string(),
                }));

                let simulated_payload = json!({
                    "name": func_name,
                    "arguments": parsed_args
                });

                let tool_result = match serde_json::from_value::<ToolCallArgs>(simulated_payload) {
                    Ok(ToolCallArgs::RunTerminalCommand(input)) => {
                        tools::execute_run_terminal_command(
                            input,
                            &app_config.dangerous_commands,
                            app_config.require_confirmation,
                        )
                    }
                    Ok(ToolCallArgs::ReadFile(input)) => tools::execute_read_file(input),
                    Ok(ToolCallArgs::WriteFile(input)) => tools::execute_write_file(input),
                    Ok(ToolCallArgs::AppendFile(input)) => tools::execute_append_file(input),
                    Ok(ToolCallArgs::ListDirectory(input)) => tools::execute_list_directory(input),
                    Ok(ToolCallArgs::GetSystemStats(_)) => tools::execute_get_system_stats(),
                    Ok(ToolCallArgs::SearchCodebase(_)) => tools::ToolResult {
                        success: false,
                        output: "Tool 'search_codebase' is currently disabled.".to_string(),
                    },
                    Err(e) => {
                        let correction = format!(
                            "You sent invalid arguments to the '{}' tool. Schema error: {}. \
                            Carefully re-read the tool's required parameters and call it again with the correct argument names and types.",
                            func_name, e
                        );
                        critic_injections.push(critic_message(&correction));
                        temperature_override = if server_flavor == ServerFlavor::KoboldCpp {
                            0.2
                        } else {
                            0.3
                        };
                        tools::ToolResult {
                            success: false,
                            output: "[Schema mismatch — see correction directive above]"
                                .to_string(),
                        }
                    }
                };

                // Emit ToolResult event to TUI
                let _ = event_tx.send(tui::TuiEvent::ToolResult(tui::ToolResultInfo {
                    name: func_name.clone(),
                    output: tool_result.output.clone(),
                    success: tool_result.success,
                }));

                if !tool_result.success && func_name == "run_terminal_command" {
                    critic_injections.push(critic_message(
                        "The previous command failed. Analyze the error output above carefully. Correct your approach and retry now. Do NOT repeat the same command."
                    ));
                    temperature_override = if server_flavor == ServerFlavor::KoboldCpp {
                        0.2
                    } else {
                        0.3
                    };
                }

                if tool_result.success && func_name == "write_file" {
                    critic_injections.push(critic_message(
                        "File was written. You MUST now verify it is correct: use the read_file tool to read back the file you just wrote before continuing to the next step."
                    ));
                }

                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: Some(tool_result.output),
                    tool_calls: None,
                    tool_call_id: Some(id),
                    name: Some(func_name),
                });
            }
            messages.extend(critic_injections);
        } else {
            break;
        }
    }

    let _ = event_tx.send(tui::TuiEvent::Status("Ready".to_string()));
}

#[cfg(test)]
mod tests {
    use super::{
        extract_stream_tool_calls, extract_visible_delta_text, extract_visible_stream_choice_text,
        flush_token_buffer, format_visible_output, should_replay_final_content,
        should_retry_non_stream_after_stream_error,
    };
    use serde_json::json;
    use tokio::sync::mpsc;

    #[test]
    fn extracts_non_content_visible_text() {
        let delta = json!({"reasoning_content": "hello-from-reasoning"});
        assert_eq!(
            extract_visible_delta_text(&delta, false),
            Some(("hello-from-reasoning".to_string(), true))
        );
    }

    #[test]
    fn chat_mode_marks_reasoning_content_chunks() {
        let delta = json!({"reasoning_content": "hidden"});
        assert_eq!(
            extract_visible_delta_text(&delta, true),
            Some(("hidden".to_string(), true))
        );
    }

    #[test]
    fn prefers_content_over_other_fields() {
        let delta = json!({"content": "primary", "text": "secondary"});
        assert_eq!(
            extract_visible_delta_text(&delta, true),
            Some(("primary".to_string(), false))
        );
    }

    #[test]
    fn extracts_choice_text_when_delta_is_missing() {
        let choice = json!({"text": "plain-choice-text"});
        assert_eq!(
            extract_visible_stream_choice_text(&choice, false),
            Some(("plain-choice-text".to_string(), false))
        );
    }

    #[test]
    fn extracts_message_content_when_choice_uses_message_shape() {
        let choice = json!({"message": {"content": "message-content"}});
        assert_eq!(
            extract_visible_stream_choice_text(&choice, false),
            Some(("message-content".to_string(), false))
        );
    }

    #[test]
    fn extracts_tool_calls_from_choice_message_when_delta_missing() {
        let choice = json!({
            "message": {
                "tool_calls": [
                    {
                        "index": 0,
                        "id": "call_1",
                        "function": { "name": "list_directory", "arguments": "{}" }
                    }
                ]
            }
        });

        let tool_calls = extract_stream_tool_calls(&choice).expect("expected tool calls");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], json!("call_1"));
    }

    #[test]
    fn format_visible_output_has_non_empty_chat_fallback() {
        let input = "<think>hidden</think>";
        assert!(!format_visible_output(input, true).trim().is_empty());
    }

    #[tokio::test]
    async fn flush_token_buffer_emits_chunk_and_clears_buffer() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut token_buffer = "partial-stream".to_string();

        flush_token_buffer(&tx, &mut token_buffer);

        assert!(token_buffer.is_empty());
        match rx.recv().await {
            Some(crate::tui::TuiEvent::TokenChunk(chunk)) => {
                assert_eq!(chunk, "partial-stream");
            }
            _ => panic!("expected TokenChunk event"),
        }
    }

    #[test]
    fn format_visible_output_strips_reasoning_in_chat_mode() {
        let input = "hello <think>hidden</think> world";
        assert_eq!(format_visible_output(input, true), "hello  world");
    }

    #[test]
    fn format_visible_output_exposes_reasoning_in_agentic_mode() {
        let input = "a <think>work</think> b";
        assert!(format_visible_output(input, false).contains("<thinking>"));
    }

    #[test]
    fn retries_non_stream_only_when_no_stream_content_or_tools_exist() {
        assert!(should_retry_non_stream_after_stream_error("", 0));
        assert!(!should_retry_non_stream_after_stream_error("partial", 0));
        assert!(!should_retry_non_stream_after_stream_error("", 1));
    }

    #[test]
    fn does_not_replay_final_content_after_stream_started() {
        assert!(!should_replay_final_content(true, Some("hello")));
    }

    #[test]
    fn replays_final_content_only_for_non_stream_recovery_case() {
        assert!(should_replay_final_content(false, Some("hello")));
        assert!(!should_replay_final_content(false, Some("")));
        assert!(!should_replay_final_content(false, None));
    }
}

fn build_tools(persona: &str, strict_tools: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let mut tools = Vec::new();

    /*
    // search_codebase — always available
    let search_schema = schema_for!(tools::SearchCodebaseInput);
    let search_obj = serde_json::to_value(search_schema)?;
    tools.push(json!({
        "type": "function",
        "function": {
            "name": "search_codebase",
            "description": "Semantic vector search over the entire project codebase. Use this first before read_file to locate relevant code without guessing paths.",
            "strict": true,
            "parameters": search_obj
        }
    }));
    */

    // list_directory — always available
    let list_dir_schema = schema_for!(tools::ListDirectoryInput);
    let list_dir_obj = serde_json::to_value(list_dir_schema)?;
    tools.push(json!({
        "type": "function",
        "function": {
            "name": "list_directory",
            "description": "List files and subdirectories at the given path (2 levels deep). Use this to explore the project structure before reading files. Never guess paths.",
            "strict": strict_tools,
            "parameters": list_dir_obj
        }
    }));

    // read_file — always available
    let read_file_schema = schema_for!(tools::ReadFileInput);
    let read_file_obj = serde_json::to_value(read_file_schema)?;
    tools.push(json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read text from a local file. Output is capped at 12,000 chars. Use search_codebase first to locate the right file.",
            "strict": strict_tools,
            "parameters": read_file_obj
        }
    }));

    // get_system_stats — always available
    let get_system_stats_schema = schema_for!(tools::GetSystemStatsInput);
    let get_system_stats_obj = serde_json::to_value(get_system_stats_schema)?;
    tools.push(json!({
        "type": "function",
        "function": {
            "name": "get_system_stats",
            "description": "Check real-time hardware metrics like RAM, CPU% and Uptime via native syscalls.",
            "strict": strict_tools,
            "parameters": get_system_stats_obj
        }
    }));

    // write_file + append_file accessible to coder and os_assistant
    if persona == "os_assistant" || persona == "coder" {
        let write_file_schema = schema_for!(tools::WriteFileInput);
        let write_file_obj = serde_json::to_value(write_file_schema)?;
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write (overwrite) text content to a file. A verify-back read will be automatically enforced.",
                "strict": strict_tools,
                "parameters": write_file_obj
            }
        }));

        let append_file_schema = schema_for!(tools::AppendFileInput);
        let append_file_obj = serde_json::to_value(append_file_schema)?;
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "append_file",
                "description": "Safely append content to the END of an existing file without overwriting. Use for adding functions, config blocks, or log lines.",
                "strict": strict_tools,
                "parameters": append_file_obj
            }
        }));
    }

    // run_terminal_command — os_assistant only
    if persona == "os_assistant" {
        let run_cmd_schema = schema_for!(tools::RunTerminalCommandInput);
        let run_cmd_obj = serde_json::to_value(run_cmd_schema)?;
        tools.push(json!({
            "type": "function",
            "function": {
                "name": "run_terminal_command",
                "description": "Execute an arbitrary shell command. STDOUT/STDERR are returned (capped at 8,000 chars from the end).",
                "strict": strict_tools,
                "parameters": run_cmd_obj
            }
        }));
    }

    Ok(json!(tools))
}

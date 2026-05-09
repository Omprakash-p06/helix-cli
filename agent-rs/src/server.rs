use axum::{
    Json, Router,
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post},
};
use futures_util::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::security::policy::PolicyContext;
use crate::tokens;
use crate::tools::{self, ToolRegistry};
use crate::types::{ChatMessage, ChatResponse, ServerFlavor};
use crate::audit::AuditStore;
use crate::agent_core::tool_runtime::{ToolRuntime, ToolRequest, ToolLifecycle};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub app_config: AppConfig,
    pub generated_grammar: String,
    pub tools_payload: Value,
    pub server_flavor: ServerFlavor,
    pub audit_store: Option<Arc<AuditStore>>,
    pub tool_registry: Arc<ToolRegistry>,
    pub tool_runtime: Arc<ToolRuntime>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
pub struct AgentEventPayload {
    pub r#type: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub model: String,
    pub server_flavor: String,
}

#[derive(Serialize)]
pub struct ContextResponse {
    pub workspace_root: String,
    pub git_branch: String,
}

pub async fn start_web_server(
    app_config: AppConfig,
    _persona: String,
    generated_grammar: String,
    tools_payload: Value,
    server_flavor: ServerFlavor,
    audit_store: Option<Arc<AuditStore>>,
    tool_registry: Arc<ToolRegistry>,
    tool_runtime: Arc<ToolRuntime>,
) {
    let client = Client::builder()
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .build()
        .expect("failed to build HTTP client");
    let state = AppState {
        client,
        app_config,
        generated_grammar,
        tools_payload,
        server_flavor,
        audit_store,
        tool_registry,
        tool_runtime,
    };

    let app = Router::new()
        .route("/chat", post(chat_handler))
        .route("/health", get(|| async { "OK" }))
        .route("/v1/status", get(status_handler))
        .route("/v1/tools", get(tools_handler))
        .route("/v1/context", get(context_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = 3000;
    println!("\n=======================================================");
    println!("Helix Rust Agent Orchestrator running in WEB mode.");
    println!("Axum API Server listening on http://127.0.0.1:{}", port);
    println!("=======================================================\n");

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".into(),
        version: "0.1.0".into(),
        model: state.app_config.model_name.clone(),
        server_flavor: format!("{:?}", state.server_flavor),
    })
}

async fn tools_handler(State(state): State<AppState>) -> Json<Value> {
    Json(state.tool_registry.build_tools_payload(
        &std::env::var("AGENT_PERSONA").unwrap_or_else(|_| "os_assistant".to_string()),
        state.server_flavor != ServerFlavor::KoboldCpp,
    ))
}

async fn context_handler() -> Json<ContextResponse> {
    let workspace_root = tools::get_allowed_dir().to_string_lossy().to_string();
    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&workspace_root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    Json(ContextResponse {
        workspace_root,
        git_branch,
    })
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    let mut messages = payload.messages;

    tokio::spawn(async move {
        let url = format!(
            "{}/chat/completions",
            state.app_config.base_url.trim_end_matches('/')
        );
        let mut round_trip_counter = 0;
        let base_temperature: f64 = if state.server_flavor == ServerFlavor::KoboldCpp {
            0.05
        } else {
            0.1
        };
        let mut temperature_override: f64 = base_temperature;

        loop {
            if round_trip_counter >= 20 {
                let _ = tx
                    .send(Ok(Event::default()
                        .json_data(AgentEventPayload {
                            r#type: "error".to_string(),
                            content: "Safety exit: exceeded 20 action rounds.".to_string(),
                        })
                        .unwrap()))
                    .await;
                break;
            }
            round_trip_counter += 1;

            let current_tokens = tokens::count_message_tokens(&messages);
            let threshold = (state.app_config.context_size as f32 * 0.70) as usize;

            if current_tokens > threshold && messages.len() > 5 {
                let _ = tx
                    .send(Ok(Event::default()
                        .json_data(AgentEventPayload {
                            r#type: "system".to_string(),
                            content: format!("Context at {} tokens. Compacting...", current_tokens),
                        })
                        .unwrap()))
                    .await;

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
                    "model": &state.app_config.model_name,
                    "messages": compaction_messages,
                    "temperature": 0.1
                });

                if let Ok(res) = state.client.post(&url).json(&compaction_body).send().await
                    && let Ok(text) = res.text().await
                    && let Ok(parsed) = serde_json::from_str::<ChatResponse>(&text)
                    && let Some(summary) = &parsed.choices[0].message.content
                {
                    let mut new_messages = vec![messages[0].clone()];
                    new_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: Some(format!("[Internal Working Memory Summary]\n{}", summary)),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                    new_messages.extend_from_slice(&messages[mid_point..]);
                    messages = new_messages;
                }
            }

            let mut request_body = json!({
                "model": state.app_config.model_name,
                "messages": &messages,
                "tools": state.tools_payload,
                "temperature": temperature_override,
                "max_tokens": 8192
            });

            if !state.generated_grammar.is_empty() {
                request_body
                    .as_object_mut()
                    .unwrap()
                    .insert("grammar".to_string(), json!(state.generated_grammar));
            }

            let res = match state.client.post(&url).json(&request_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(Ok(Event::default()
                            .json_data(AgentEventPayload {
                                r#type: "error".to_string(),
                                content: format!("HTTP Error: {}", e),
                            })
                            .unwrap()))
                        .await;
                    break;
                }
            };

            let response_text: String = match res.text().await {
                Ok(t) => t,
                Err(e) => {
                    let _ = tx
                        .send(Ok(Event::default()
                            .json_data(AgentEventPayload {
                                r#type: "error".to_string(),
                                content: format!("Read text error: {}", e),
                            })
                            .unwrap()))
                        .await;
                    break;
                }
            };

            let response: ChatResponse = match serde_json::from_str(&response_text) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(Ok(Event::default()
                            .json_data(AgentEventPayload {
                                r#type: "error".to_string(),
                                content: format!("API Parsing Error: {}", e),
                            })
                            .unwrap()))
                        .await;
                    break;
                }
            };

            if response.choices.is_empty() {
                break;
            }

            let message = &response.choices[0].message;

            if let Some(content) = &message.content {
                let visible = if state.app_config.exec_mode == "chat" {
                    crate::utils::clean_chat_output(content)
                } else {
                    crate::expose_think_blocks(content)
                };
                if !visible.is_empty() {
                    let _ = tx
                        .send(Ok(Event::default()
                            .json_data(AgentEventPayload {
                                r#type: "text".to_string(),
                                content: visible.clone(),
                            })
                            .unwrap()))
                        .await;
                }
                if message.tool_calls.is_none() {
                    temperature_override = base_temperature;
                }
            }

            let mut history_message = message.clone();
            if history_message.tool_calls.is_some() {
                history_message.tool_calls = None;
                if history_message.content.is_none() {
                    history_message.content = Some("[assistant executed tool calls]".to_string());
                }
            }
            messages.push(history_message);

            if let Some(tool_calls) = &message.tool_calls {
                if tool_calls.is_empty() {
                    let _ = tx
                        .send(Ok(Event::default()
                            .json_data(AgentEventPayload {
                                r#type: "done".to_string(),
                                content: "".to_string(),
                            })
                            .unwrap()))
                        .await;
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

                    let req = ToolRequest {
                        call_id: id.clone(),
                        name: func_name.clone(),
                        arguments: parsed_args,
                        confidence: 1.0,
                    };

                    let (event_tx_inner, mut event_rx_inner) = mpsc::unbounded_channel::<ToolLifecycle>();
                    let tx_outer = tx.clone();
                    
                    // Spawn a task to forward lifecycle events
                    tokio::spawn(async move {
                        while let Some(ev) = event_rx_inner.recv().await {
                            let payload = match ev {
                                ToolLifecycle::Start { name, .. } => AgentEventPayload {
                                    r#type: "tool_start".to_string(),
                                    content: format!("Executing {}...", name),
                                },
                                ToolLifecycle::Status { message, .. } => AgentEventPayload {
                                    r#type: "tool_status".to_string(),
                                    content: message,
                                },
                                ToolLifecycle::Result { success, .. } => AgentEventPayload {
                                    r#type: "tool_result".to_string(),
                                    content: format!("Result: {}", if success { "Success" } else { "Failed" }),
                                },
                            };
                            let _ = tx_outer.send(Ok(Event::default().json_data(payload).unwrap())).await;
                        }
                    });

                    let policy_context = PolicyContext {
                        permission_tier: state.app_config.permission_tier,
                        trust_level: crate::security::policy::TrustLevel::from_permission_tier(state.app_config.permission_tier),
                        exec_mode: state.app_config.exec_mode.clone(),
                        workspace_root: tools::get_allowed_dir(),
                    };

                    let (_, tool_result, _) = state.tool_runtime.execute(
                        req,
                        state.app_config.dangerous_commands.clone(),
                        state.app_config.require_confirmation,
                        policy_context,
                        state.audit_store.clone(),
                        "web".to_string(),
                        state.tool_registry.clone(),
                        Some(event_tx_inner),
                    ).await;

                    if !tool_result.success && func_name == "run_terminal_command" {
                        critic_injections.push(crate::critic_message(
                            "The previous command failed. Analyze the error output above carefully. Correct your approach and retry now. Do NOT repeat the same command."
                        ));
                        temperature_override = if state.server_flavor == ServerFlavor::KoboldCpp {
                            0.2
                        } else {
                            0.3
                        };
                    }

                    if tool_result.success && func_name == "write_file" {
                        critic_injections.push(crate::critic_message(
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
                // Final answer reached
                let _ = tx
                    .send(Ok(Event::default()
                        .json_data(AgentEventPayload {
                            r#type: "done".to_string(),
                            content: "".to_string(),
                        })
                        .unwrap()))
                    .await;
                break;
            }
        }
    });

    Sse::new(ReceiverStream::new(rx))
}

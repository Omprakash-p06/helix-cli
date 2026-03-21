use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{post, get},
    Json, Router,
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;
use reqwest::Client;
use serde_json::{json, Value};

use crate::types::{ChatMessage, ChatResponse, ServerFlavor};
use crate::config::AppConfig;
use crate::tools::{self, ToolCallArgs};
use crate::tokens;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub app_config: AppConfig,
    pub persona: String,
    pub generated_grammar: String,
    pub tools_payload: Value,
    pub server_flavor: ServerFlavor,
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

pub async fn start_web_server(
    app_config: AppConfig,
    persona: String,
    generated_grammar: String,
    tools_payload: Value,
    server_flavor: ServerFlavor,
) {
    let client = Client::new();
    let state = AppState {
        client,
        app_config,
        persona,
        generated_grammar,
        tools_payload,
        server_flavor,
    };

    let app = Router::new()
        .route("/chat", post(chat_handler))
        .route("/health", get(|| async { "OK" }))
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

async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);
    let mut messages = payload.messages;

    tokio::spawn(async move {
        let url = format!("{}/chat/completions", state.app_config.base_url.trim_end_matches('/'));
        let mut round_trip_counter = 0;
        let base_temperature: f64 = if state.server_flavor == ServerFlavor::KoboldCpp { 0.05 } else { 0.1 };
        let mut temperature_override: f64 = base_temperature;

        loop {
            if round_trip_counter >= 20 {
                let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                    r#type: "error".to_string(),
                    content: "Safety exit: exceeded 20 action rounds.".to_string()
                }).unwrap())).await;
                break;
            }
            round_trip_counter += 1;

            let current_tokens = tokens::count_message_tokens(&messages);
            let threshold = (state.app_config.context_size as f32 * 0.70) as usize;

            if current_tokens > threshold && messages.len() > 5 {
                let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                    r#type: "system".to_string(),
                    content: format!("Context at {} tokens. Compacting...", current_tokens)
                }).unwrap())).await;

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

                if let Ok(res) = state.client.post(&url).json(&compaction_body).send().await {
                    if let Ok(text) = res.text().await {
                        if let Ok(parsed) = serde_json::from_str::<ChatResponse>(&text) {
                            if let Some(summary) = &parsed.choices[0].message.content {
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
                    }
                }
            }

            let mut request_body = json!({
                "model": state.app_config.model_name,
                "messages": &messages,
                "tools": state.tools_payload,
                "temperature": temperature_override
            });

            if !state.generated_grammar.is_empty() {
                request_body.as_object_mut().unwrap().insert(
                    "grammar".to_string(),
                    json!(state.generated_grammar)
                );
            }

            let res = match state.client.post(&url).json(&request_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "error".to_string(),
                        content: format!("HTTP Error: {}", e)
                    }).unwrap())).await;
                    break;
                }
            };

            let response_text = match res.text().await {
                Ok(t) => t,
                Err(e) => {
                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "error".to_string(),
                        content: format!("Read text error: {}", e)
                    }).unwrap())).await;
                    break;
                }
            };

            let response: ChatResponse = match serde_json::from_str(&response_text) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "error".to_string(),
                        content: format!("API Parsing Error: {}", e)
                    }).unwrap())).await;
                    break;
                }
            };

            if response.choices.is_empty() {
                break;
            }

            let message = &response.choices[0].message;

            if let Some(content) = &message.content {
                let visible = crate::strip_think_blocks(content);
                if !visible.is_empty() {
                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "text".to_string(),
                        content: visible.clone()
                    }).unwrap())).await;
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

                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "tool_start".to_string(),
                        content: format!("Executing {}...", func_name)
                    }).unwrap())).await;

                    let simulated_payload = json!({
                        "name": func_name,
                        "arguments": parsed_args
                    });

                    let tool_result = match serde_json::from_value::<ToolCallArgs>(simulated_payload) {
                        Ok(ToolCallArgs::RunTerminalCommand(input)) => {
                            tools::execute_run_terminal_command(
                                input,
                                &state.app_config.dangerous_commands,
                                state.app_config.require_confirmation,
                            )
                        },
                        Ok(ToolCallArgs::ReadFile(input)) => tools::execute_read_file(input),
                        Ok(ToolCallArgs::WriteFile(input)) => tools::execute_write_file(input),
                        Ok(ToolCallArgs::AppendFile(input)) => tools::execute_append_file(input),
                        Ok(ToolCallArgs::ListDirectory(input)) => tools::execute_list_directory(input),
                        Ok(ToolCallArgs::GetSystemStats(_)) => tools::execute_get_system_stats(),
                        Ok(ToolCallArgs::SearchCodebase(_)) => {
                            tools::ToolResult { success: false, output: "Tool 'search_codebase' is currently disabled.".to_string() }
                        },
                        Err(e) => {
                            let correction = format!(
                                "You sent invalid arguments to the '{}' tool. Schema error: {}. \
                                Carefully re-read the tool's required parameters and call it again with the correct argument names and types.",
                                func_name, e
                            );
                            critic_injections.push(crate::critic_message(&correction));
                            temperature_override = if state.server_flavor == ServerFlavor::KoboldCpp { 0.2 } else { 0.3 };
                            tools::ToolResult { success: false, output: format!("[Schema mismatch — see correction directive above]") }
                        }
                    };

                    if !tool_result.success && func_name == "run_terminal_command" {
                        critic_injections.push(crate::critic_message(
                            "The previous command failed. Analyze the error output above carefully. Correct your approach and retry now. Do NOT repeat the same command."
                        ));
                        temperature_override = if state.server_flavor == ServerFlavor::KoboldCpp { 0.2 } else { 0.3 };
                    }

                    if tool_result.success && func_name == "write_file" {
                        critic_injections.push(crate::critic_message(
                            "File was written. You MUST now verify it is correct: use the read_file tool to read back the file you just wrote before continuing to the next step."
                        ));
                    }

                    let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                        r#type: "tool_result".to_string(),
                        content: format!("Result: {}", if tool_result.success { "Success" } else { "Failed" })
                    }).unwrap())).await;

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
                let _ = tx.send(Ok(Event::default().json_data(AgentEventPayload {
                    r#type: "done".to_string(),
                    content: "".to_string()
                }).unwrap())).await;
                break;
            }
        }
    });

    Sse::new(ReceiverStream::new(rx))
}

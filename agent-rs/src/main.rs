mod config;
mod tools;
mod tokens;
mod input;
pub mod types;
mod server;

use reqwest::Client;
use schemars::schema_for;
use serde_json::{json, Value};
use rustyline::error::ReadlineError;
use std::thread;
use std::time::Duration;
use tools::ToolCallArgs;

pub use types::{ChatMessage, ChatResponse, Choice, ServerFlavor};

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

pub fn strip_think_blocks(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::new();

    loop {
        if let Some(start) = remaining.find("<think>") {
            output.push_str(&remaining[..start]);
            let after_start = &remaining[start + "<think>".len()..];
            if let Some(end_rel) = after_start.find("</think>") {
                remaining = &after_start[end_rel + "</think>".len()..];
            } else {
                break;
            }
        } else {
            output.push_str(remaining);
            break;
        }
    }

    output.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading configuration from python runtime...");
    let app_config = config::AppConfig::load_from_python()?;
    let client = Client::new();

    /*
    println!("\n[RAG] Booting FastEmbed Semantic Knowledge Base... (this may take a moment)");
    let rag_store = rag::RagStore::new(&app_config.allowed_dir)?;
    println!("  [✓] Local RAG sequence complete. Vector Store loaded in-memory.");
    */
    
    
    let persona = std::env::var("AGENT_PERSONA").unwrap_or_else(|_| "os_assistant".to_string());
    let is_chat_mode = app_config.exec_mode == "chat";
    let url = format!("{}/chat/completions", app_config.base_url.trim_end_matches('/'));
    let server_flavor = detect_server_flavor(&client, &app_config.base_url).await;
    let strict_tools = server_flavor != ServerFlavor::KoboldCpp;
    let tools_payload = if is_chat_mode {
        json!([])
    } else {
        build_tools(&persona, strict_tools)?
    };
    if server_flavor == ServerFlavor::KoboldCpp {
        println!("[Runtime] Detected KoboldCPP endpoint. Stripping 'strict' tags for schema compatibility, but enforcing native GBNF grammar for 100% accuracy.");
    }
    
    let generated_grammar = if !is_chat_mode {
        println!("[Runtime] Compiling JSON schemata to GBNF Grammar...");
        tools::generate_tool_grammar(&tools_payload)
    } else {
        String::new()
    };

    let system_prompt = if is_chat_mode {
        "You are a concise technical assistant. Answer questions directly without tools. Do not greet the user. Do not introduce yourself. Do not use conversational filler."
    } else {
        match persona.as_str() {
            "coder" => "You are an autonomous code executor. You read and write files using provided tools. You cannot execute terminal commands. State your reasoning in one sentence before each tool call. Do not greet the user. Do not introduce yourself. Be concise.",
            "researcher" => "You are an autonomous read-only system explorer. You read files and gather system stats using provided tools. You cannot modify files or execute commands. State your reasoning in one sentence before each tool call. Do not greet the user. Do not introduce yourself. Be concise.",
            _ => "You are an autonomous local system orchestrator. You execute tasks using provided tools. Before each tool call, state your reasoning in one sentence. Never guess file paths — verify with list_directory first. If a command fails, read STDERR and retry with a corrected approach. Do not greet the user. Do not introduce yourself. Do not use conversational filler. Be concise."
        }
    };

    let ui_mode = std::env::var("HELIX_UI_MODE").unwrap_or_else(|_| "terminal".to_string());

    if ui_mode == "web" {
        server::start_web_server(
            app_config,
            persona,
            generated_grammar,
            tools_payload,
            server_flavor,
        ).await;
        // Web server loop handles everything, exit process when done
        return Ok(());
    }

    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: Some(system_prompt.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];


    let args: Vec<String> = std::env::args().collect();
    let mut initial_prompt = args.iter().position(|r| r == "--prompt")
        .and_then(|idx| args.get(idx + 1))
        .cloned();
    let eval_mode = initial_prompt.is_some();

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
            if eval_mode { break; }
            let editor = rl.as_mut().unwrap();
            match editor.readline("> ") {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
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
        let base_temperature: f64 = if server_flavor == ServerFlavor::KoboldCpp { 0.05 } else { 0.1 };
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
                println!("\n[Memory Alert] Context at {} tokens (limit {}). Compacting...", current_tokens, threshold);

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
                "model": app_config.model_name,
                "messages": &messages,
                "tools": tools_payload,
                "temperature": temperature_override
            });

            if !is_chat_mode && !generated_grammar.is_empty() {
                request_body.as_object_mut().unwrap().insert(
                    "grammar".to_string(),
                    json!(generated_grammar)
                );
            }


            let res = match client.post(&url).json(&request_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    println!("[Rust] HTTP Error: {}", e);
                    break;
                }
            };

            let response_text = match res.text().await {
                Ok(t) => t,
                Err(e) => {
                    println!("[Rust] Read text error: {}", e);
                    break;
                }
            };

            let response: ChatResponse = match serde_json::from_str(&response_text) {
                Ok(r) => r,
                Err(e) => {
                    println!("[Rust] API Parsing Error: {}. Response: {}", e, response_text);
                    break;
                }
            };

            if response.choices.is_empty() {
                println!("[Rust] Empty choices array returned from LLM.");
                break;
            }

            let message = &response.choices[0].message;

            if let Some(content) = &message.content {
                let visible = strip_think_blocks(content);
                if !visible.is_empty() {
                    println!("\n{}\n", visible);
                }
                // Clean response (no tool use) — reset temperature to baseline
                if message.tool_calls.is_none() {
                    temperature_override = base_temperature;
                }
            }

            // Compatibility fallback: do not replay raw assistant tool_calls back into
            // history. Different backends/templates disagree on argument typing
            // (string vs object), which can break subsequent rounds.
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

                    if eval_mode {
                        println!("➜ Tool: {}", func_name);
                    }

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
                            println!("[Critic] SCHEMA ERROR — injecting self-correction directive.");
                            let correction = format!(
                                "You sent invalid arguments to the '{}' tool. Schema error: {}. \
                                Carefully re-read the tool's required parameters and call it again with the correct argument names and types.",
                                func_name, e
                            );
                            critic_injections.push(critic_message(&correction));
                            temperature_override = if server_flavor == ServerFlavor::KoboldCpp { 0.2 } else { 0.3 };
                            tools::ToolResult { success: false, output: format!("[Schema mismatch — see correction directive above]") }
                        }
                    };

                    if !tool_result.success && func_name == "run_terminal_command" {
                        println!("[Critic] Command failed — injecting retry directive.");
                        critic_injections.push(critic_message(
                            "The previous command failed. Analyze the error output above carefully. Correct your approach and retry now. Do NOT repeat the same command."
                        ));
                        temperature_override = if server_flavor == ServerFlavor::KoboldCpp { 0.2 } else { 0.3 }; // Anneal for creative solution-finding
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

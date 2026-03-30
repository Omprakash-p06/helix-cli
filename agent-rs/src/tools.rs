use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

// ==========================================
// TOOL SCHEMAS & TYPES
// ==========================================

const READ_FILE_MAX_CHARS: usize = 12_000;
const CMD_OUTPUT_MAX_CHARS: usize = 8_000;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunTerminalCommandInput {
    pub command: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileInput {
    pub absolute_path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileInput {
    pub absolute_path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppendFileInput {
    pub absolute_path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoryInput {
    pub absolute_path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct GetSystemStatsInput {
    pub dummy: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct SearchCodebaseInput {
    pub query: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "name", content = "arguments", rename_all = "snake_case")]
pub enum ToolCallArgs {
    #[serde(rename = "run_terminal_command")]
    RunTerminalCommand(RunTerminalCommandInput),
    #[serde(rename = "read_file")]
    ReadFile(ReadFileInput),
    #[serde(rename = "write_file")]
    WriteFile(WriteFileInput),
    #[serde(rename = "append_file")]
    AppendFile(AppendFileInput),
    #[serde(rename = "list_directory")]
    ListDirectory(ListDirectoryInput),
    #[serde(rename = "get_system_stats")]
    GetSystemStats(GetSystemStatsInput),
    #[serde(rename = "search_codebase")]
    SearchCodebase(SearchCodebaseInput),
}

/// Structured tool result with deterministic success signal.
/// The Rust critic uses `success` to decide whether to inject retry/verify directives.
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

// ==========================================
// TOOL EXECUTION LOGIC
// ==========================================

pub fn get_allowed_dir() -> PathBuf {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.file_name().and_then(|n| n.to_str()) == Some("agent-rs") {
        current_dir.parent().unwrap_or(&current_dir).to_path_buf()
    } else {
        current_dir
    }
}

fn enforce_sandbox(target_path: &str) -> Result<PathBuf, String> {
    let allowed_dir = std::fs::canonicalize(get_allowed_dir())
        .map_err(|_| "Could not canonicalize allowed directory".to_string())?;

    let path = Path::new(target_path);

    let resolved = if path.exists() {
        std::fs::canonicalize(path).map_err(|e| e.to_string())?
    } else {
        let parent = path.parent().unwrap_or(Path::new(""));
        let resolved_parent = if parent.as_os_str().is_empty() {
            std::fs::canonicalize(Path::new(".")).map_err(|e| e.to_string())?
        } else {
            std::fs::canonicalize(parent).map_err(|e| e.to_string())?
        };
        resolved_parent.join(path.file_name().unwrap_or_default())
    };

    if !resolved.starts_with(&allowed_dir) {
        return Err(format!(
            "SECURITY VIOLATION: Path '{}' is outside the strictly allowed directory '{}'. Refusing to execute.",
            resolved.display(),
            allowed_dir.display()
        ));
    }

    Ok(resolved)
}

/// Tail-truncate a string to `max_chars`, keeping the LAST bytes since
/// that's where the meaningful error output appears.
fn tail_truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        return s.to_string();
    }
    let start = chars.len() - max_chars;
    format!(
        "[Rust Orchestrator] Output truncated — showing last {max_chars} chars of {} total.\n...\n{}",
        chars.len(),
        chars[start..].iter().collect::<String>()
    )
}

pub fn execute_run_terminal_command(
    input: RunTerminalCommandInput,
    dangerous_commands: &[String],
    require_confirmation: bool,
) -> ToolResult {
    let cmd = input.command;
    let is_dangerous = dangerous_commands
        .iter()
        .any(|bad| cmd.trim().starts_with(bad.as_str()));

    if is_dangerous && require_confirmation {
        println!("⚠️  DANGEROUS COMMAND BLOCKED: {}", cmd);
        return ToolResult {
            success: false,
            output: format!(
                "Execution Blocked: '{}' is flagged as a dangerous command. Denied.",
                cmd
            ),
        };
    }

    println!("$ {}", cmd);
    let mut process = if cfg!(target_os = "windows") {
        let mut p = Command::new("cmd");
        p.arg("/C").arg(&cmd);
        p
    } else {
        let mut p = Command::new("sh");
        p.arg("-c").arg(&cmd);
        p
    };
    let output = process.current_dir(get_allowed_dir()).output();

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(-1);
            let success = out.status.success();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            let mut raw = String::new();
            if !success {
                raw.push_str(&format!("[COMMAND FAILED, exit code {}]\n", exit_code));
            }
            if !stdout.is_empty() {
                raw.push_str(&format!("STDOUT:\n{}\n", stdout));
            }
            if !stderr.is_empty() {
                raw.push_str(&format!("STDERR:\n{}\n", stderr));
            }
            if raw.is_empty() {
                raw = "Command executed successfully with no output.".to_string();
            }

            // Cap output — always tail so last error lines are preserved
            let result = tail_truncate(&raw, CMD_OUTPUT_MAX_CHARS);
            ToolResult {
                success,
                output: result,
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: format!("Failed to spawn process: {}", e),
        },
    }
}

pub fn execute_read_file(input: ReadFileInput) -> ToolResult {
    let resolved_path = match enforce_sandbox(&input.absolute_path) {
        Ok(p) => p,
        Err(e) => {
            return ToolResult {
                success: false,
                output: e,
            };
        }
    };

    println!("Reading: {}", resolved_path.display());
    match fs::read_to_string(&resolved_path) {
        Ok(content) => {
            let chars: Vec<char> = content.chars().collect();
            if chars.len() > READ_FILE_MAX_CHARS {
                let truncated: String = chars[..READ_FILE_MAX_CHARS].iter().collect();
                let output = format!(
                    "{}\n\n[Rust Orchestrator] File truncated at {READ_FILE_MAX_CHARS} chars ({} total). Use search_codebase or a targeted read for deeper context.",
                    truncated,
                    chars.len()
                );
                ToolResult {
                    success: true,
                    output,
                }
            } else {
                ToolResult {
                    success: true,
                    output: content,
                }
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: format!("Error reading file: {}", e),
        },
    }
}

pub fn execute_write_file(input: WriteFileInput) -> ToolResult {
    let resolved_path = match enforce_sandbox(&input.absolute_path) {
        Ok(p) => p,
        Err(e) => {
            return ToolResult {
                success: false,
                output: e,
            };
        }
    };

    println!("Writing: {}", resolved_path.display());
    if let Some(parent) = resolved_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ToolResult {
                success: false,
                output: format!("Failed to create parent directories: {}", e),
            };
        }
    }

    match fs::write(&resolved_path, input.content) {
        Ok(_) => ToolResult {
            success: true,
            output: format!("Successfully wrote to {}", resolved_path.display()),
        },
        Err(e) => ToolResult {
            success: false,
            output: format!("Error writing file: {}", e),
        },
    }
}

pub fn execute_append_file(input: AppendFileInput) -> ToolResult {
    let resolved_path = match enforce_sandbox(&input.absolute_path) {
        Ok(p) => p,
        Err(e) => {
            return ToolResult {
                success: false,
                output: e,
            };
        }
    };

    println!("Appending to: {}", resolved_path.display());
    if let Some(parent) = resolved_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ToolResult {
                success: false,
                output: format!("Failed to create parent directories: {}", e),
            };
        }
    }

    use std::io::Write;
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&resolved_path)
    {
        Ok(mut file) => match file.write_all(input.content.as_bytes()) {
            Ok(_) => ToolResult {
                success: true,
                output: format!("Successfully appended to {}", resolved_path.display()),
            },
            Err(e) => ToolResult {
                success: false,
                output: format!("Error appending to file: {}", e),
            },
        },
        Err(e) => ToolResult {
            success: false,
            output: format!("Error opening file for append: {}", e),
        },
    }
}

pub fn execute_list_directory(input: ListDirectoryInput) -> ToolResult {
    let resolved_path = match enforce_sandbox(&input.absolute_path) {
        Ok(p) => p,
        Err(e) => {
            return ToolResult {
                success: false,
                output: e,
            };
        }
    };

    if !resolved_path.is_dir() {
        return ToolResult {
            success: false,
            output: format!("'{}' is not a directory.", resolved_path.display()),
        };
    }

    println!("Listing: {}", resolved_path.display());
    let mut lines: Vec<String> = vec![format!("📁 {}/", resolved_path.display())];

    fn walk(dir: &Path, prefix: &str, depth: usize, lines: &mut Vec<String>) {
        if depth > 2 {
            return;
        }
        let Ok(mut entries) = fs::read_dir(dir) else {
            return;
        };
        let mut entries_vec: Vec<_> = entries.by_ref().flatten().collect();
        entries_vec.sort_by_key(|e| e.file_name());

        for entry in entries_vec {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden dirs and build artifacts
            if name.starts_with('.') || name == "target" || name == "__pycache__" {
                continue;
            }

            if path.is_dir() {
                lines.push(format!("{prefix}├── 📁 {name}/"));
                walk(&path, &format!("{prefix}│   "), depth + 1, lines);
            } else {
                let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                let size_str = if size > 1024 * 1024 {
                    format!("{:.1}MB", size as f64 / 1048576.0)
                } else if size > 1024 {
                    format!("{:.1}KB", size as f64 / 1024.0)
                } else {
                    format!("{size}B")
                };
                lines.push(format!("{prefix}├── 📄 {name} ({size_str})"));
            }
        }
    }

    walk(&resolved_path, "", 0, &mut lines);
    ToolResult {
        success: true,
        output: lines.join("\n"),
    }
}

pub fn execute_get_system_stats() -> ToolResult {
    let mut sys = System::new_all();
    sys.refresh_all();
    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;
    let cpus = sys.cpus();
    let cpu_usage: f32 = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let uptime_secs = System::uptime();

    let stats = format!(
        "OS: {}\nUptime: {}s\nRAM: {}MB / {}MB\nCPU Global Load: {:.2}%",
        os_name, uptime_secs, used_memory, total_memory, cpu_usage
    );

    ToolResult {
        success: true,
        output: stats,
    }
}

pub fn generate_tool_grammar(tools_payload: &serde_json::Value) -> String {
    use gbnf::Grammar;
    use serde_json::json;

    // Create a generic function call schema wrapper mimicking OpenAI's payload
    let schema = json!({
        "oneOf": [
            {
                "type": "string"
            },
            {
                "type": "object",
                "properties": {
                    "tool_calls": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "const": "function"
                                },
                                "function": {
                                    "type": "object",
                                    "properties": {
                                        "name": {
                                            "type": "string",
                                            "enum": tools_payload.as_array()
                                                .unwrap_or(&vec![])
                                                .iter()
                                                .filter_map(|t| t.pointer("/function/name").and_then(|n| n.as_str()))
                                                .collect::<Vec<&str>>()
                                        },
                                        "arguments": {
                                            "type": "string"
                                        }
                                    },
                                    "required": ["name", "arguments"]
                                }
                            },
                            "required": ["type", "function"]
                        }
                    }
                },
                "required": ["tool_calls"]
            }
        ]
    });

    match Grammar::from_json_schema_value(&schema) {
        Ok(grammar) => grammar.to_string(),
        Err(e) => {
            println!("[Warn] Failed to generate grammar from tools: {}", e);
            String::new()
        }
    }
}

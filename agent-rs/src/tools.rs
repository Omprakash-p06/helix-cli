use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

use crate::security::policy::PolicyContext;
use crate::agent_core::diagnostics::system::SystemProvider;
use crate::agent_core::diagnostics::logs;
use crate::agent_core::repair::tools::{ServiceRepairTool, PackageRepairTool, PermissionRepairTool};
pub use crate::agent_core::tool_runtime::ToolResult;

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

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ListProcessesInput {
    pub dummy: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetServiceStatusInput {
    pub service_name: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct GetSystemLogsInput {
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchSystemFilesInput {
    pub query: String,
    pub path: String,
}

// ==========================================

pub trait Tool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn schema(&self) -> Value;
    fn execute(
        &self,
        args: Value,
        dangerous_commands: &[String],
        require_confirmation: bool,
        policy_context: &PolicyContext,
    ) -> ToolResult;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    pub fn list_tools(&self) -> Vec<&dyn Tool> {
        self.tools.values().map(|b| b.as_ref()).collect()
    }

    pub fn build_tools_payload(&self, persona: &str, strict_tools: bool) -> Value {
        let mut tools = Vec::new();
        for tool in self.list_tools() {
            // Replicate persona filtering logic
            let name = tool.name();
            if (name == "write_file" || name == "append_file") && !(persona == "os_assistant" || persona == "coder") {
                continue;
            }
            if (name == "run_terminal_command" || name == "service_repair" || name == "package_repair" || name == "permission_repair") && persona != "os_assistant" {
                continue;
            }
            if name == "search_codebase" {
                // Keep it disabled for now as in current build_tools
                continue;
            }

            tools.push(json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": tool.description(),
                    "strict": if strict_tools { Some(true) } else { None },
                    "parameters": tool.schema(),
                }
            }));
        }
        json!(tools)
    }
}

pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(RunTerminalCommandTool));
    registry.register(Box::new(ReadFileTool));
    registry.register(Box::new(WriteFileTool));
    registry.register(Box::new(AppendFileTool));
    registry.register(Box::new(ListDirectoryTool));
    registry.register(Box::new(GetSystemStatsTool));
    registry.register(Box::new(SearchCodebaseTool));
    registry.register(Box::new(ListProcessesTool));
    registry.register(Box::new(GetServiceStatusTool));
    registry.register(Box::new(SearchSystemFilesTool));
    registry.register(Box::new(GetSystemLogsTool));
    registry.register(Box::new(ServiceRepairTool));
    registry.register(Box::new(PackageRepairTool));
    registry.register(Box::new(PermissionRepairTool));
    registry
}

// ==========================================
// BUILT-IN TOOL IMPLEMENTATIONS
// ==========================================

struct RunTerminalCommandTool;
impl Tool for RunTerminalCommandTool {
    fn name(&self) -> String { "run_terminal_command".into() }
    fn description(&self) -> String { "Executes a shell command on the local system.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(RunTerminalCommandInput).into() }
    fn execute(&self, args: Value, dangerous_commands: &[String], require_confirmation: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<RunTerminalCommandInput>(args) {
            Ok(input) => execute_run_terminal_command(input, dangerous_commands, require_confirmation, _ctx),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct ReadFileTool;
impl Tool for ReadFileTool {
    fn name(&self) -> String { "read_file".into() }
    fn description(&self) -> String { "Reads the content of a file at the specified absolute path.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(ReadFileInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<ReadFileInput>(args) {
            Ok(input) => execute_read_file(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct WriteFileTool;
impl Tool for WriteFileTool {
    fn name(&self) -> String { "write_file".into() }
    fn description(&self) -> String { "Writes content to a file at the specified absolute path.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(WriteFileInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<WriteFileInput>(args) {
            Ok(input) => execute_write_file(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct AppendFileTool;
impl Tool for AppendFileTool {
    fn name(&self) -> String { "append_file".into() }
    fn description(&self) -> String { "Appends content to a file at the specified absolute path.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(AppendFileInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<AppendFileInput>(args) {
            Ok(input) => execute_append_file(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct ListDirectoryTool;
impl Tool for ListDirectoryTool {
    fn name(&self) -> String { "list_directory".into() }
    fn description(&self) -> String { "Lists the contents of a directory.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(ListDirectoryInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<ListDirectoryInput>(args) {
            Ok(input) => execute_list_directory(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct GetSystemStatsTool;
impl Tool for GetSystemStatsTool {
    fn name(&self) -> String { "get_system_stats".into() }
    fn description(&self) -> String { "Returns local system resource usage (CPU, RAM).".into() }
    fn schema(&self) -> Value { schemars::schema_for!(GetSystemStatsInput).into() }
    fn execute(&self, _args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        execute_get_system_stats()
    }
}

struct SearchCodebaseTool;
impl Tool for SearchCodebaseTool {
    fn name(&self) -> String { "search_codebase".into() }
    fn description(&self) -> String { "Performs keyword search across the codebase.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(SearchCodebaseInput).into() }
    fn execute(&self, _args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        ToolResult { success: false, output: "Tool 'search_codebase' is currently disabled.".into() }
    }
}

struct ListProcessesTool;
impl Tool for ListProcessesTool {
    fn name(&self) -> String { "list_processes".into() }
    fn description(&self) -> String { "Lists all running processes with PID, CPU, and memory usage.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(ListProcessesInput).into() }
    fn execute(&self, _args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        execute_list_processes()
    }
}

struct GetServiceStatusTool;
impl Tool for GetServiceStatusTool {
    fn name(&self) -> String { "get_service_status".into() }
    fn description(&self) -> String { "Queries the status of a system service (e.g., docker, systemd-resolved).".into() }
    fn schema(&self) -> Value { schemars::schema_for!(GetServiceStatusInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<GetServiceStatusInput>(args) {
            Ok(input) => execute_get_service_status(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct SearchSystemFilesTool;
impl Tool for SearchSystemFilesTool {
    fn name(&self) -> String { "search_system_files".into() }
    fn description(&self) -> String { "Searches for a query string in system files at a specified path using rg.".into() }
    fn schema(&self) -> Value { schemars::schema_for!(SearchSystemFilesInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<SearchSystemFilesInput>(args) {
            Ok(input) => execute_search_system_files(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

struct GetSystemLogsTool;
impl Tool for GetSystemLogsTool {
    fn name(&self) -> String { "get_system_logs".into() }
    fn description(&self) -> String { "Retrieves system logs (Linux journald or Windows Event Log).".into() }
    fn schema(&self) -> Value { schemars::schema_for!(GetSystemLogsInput).into() }
    fn execute(&self, args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        match serde_json::from_value::<GetSystemLogsInput>(args) {
            Ok(input) => execute_get_system_logs(input),
            Err(e) => ToolResult { success: false, output: format!("Schema error: {}", e) },
        }
    }
}

// ==========================================
// CORE EXECUTION WRAPPERS
// ==========================================

pub fn get_allowed_dir() -> PathBuf {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.file_name().and_then(|n| n.to_str()) == Some("agent-rs") {
        current_dir.parent().unwrap_or(&current_dir).to_path_buf()
    } else {
        current_dir
    }
}

const DIAGNOSTIC_PATH_ALLOWLIST: &[&str] = &[
    "/etc",
    "/var/log",
    "/proc",
    "/sys",
    "/run",
    "/Library/Logs",
    "/var/db/diagnostics",
];

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

    if resolved.starts_with(&allowed_dir) {
        return Ok(resolved);
    }

    // Check diagnostic allowlist
    for allowed_prefix in DIAGNOSTIC_PATH_ALLOWLIST {
        let allowed_path = Path::new(allowed_prefix);
        if resolved.starts_with(allowed_path) {
            if is_sensitive_diagnostic_path(&resolved) {
                break;
            }
            return Ok(resolved);
        }
    }

    Err(format!(
        "SECURITY VIOLATION: Path '{}' is outside the strictly allowed directory '{}' and diagnostic allowlist. Refusing to execute.",
        resolved.display(),
        allowed_dir.display()
    ))
}

fn is_sensitive_diagnostic_path(path: &Path) -> bool {
    let Some(path_str) = path.to_str() else {
        return false;
    };

    let sensitive_suffixes = [
        "/etc/shadow",
        "/etc/gshadow",
        "/etc/sudoers",
        "/root/",
        "/root",
        "/.ssh/",
        "/.ssh",
    ];

    sensitive_suffixes.iter().any(|needle| path_str == *needle || path_str.contains(needle))
}

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

fn execute_run_terminal_command(
    input: RunTerminalCommandInput,
    dangerous_commands: &[String],
    require_confirmation: bool,
    _policy_context: &PolicyContext,
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

fn execute_read_file(input: ReadFileInput) -> ToolResult {
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

fn execute_write_file(input: WriteFileInput) -> ToolResult {
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
    if let Some(parent) = resolved_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return ToolResult {
            success: false,
            output: format!("Failed to create parent directories: {}", e),
        };
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

fn execute_append_file(input: AppendFileInput) -> ToolResult {
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
    if let Some(parent) = resolved_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return ToolResult {
            success: false,
            output: format!("Failed to create parent directories: {}", e),
        };
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

fn execute_list_directory(input: ListDirectoryInput) -> ToolResult {
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

fn execute_get_system_stats() -> ToolResult {
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

fn execute_list_processes() -> ToolResult {
    let mut provider = SystemProvider::new();
    let processes = provider.list_processes();

    let mut output = String::from("PID | Name | CPU % | Mem (MB) | Status\n");
    output.push_str("----|------|-------|----------|-------\n");
    for p in processes {
        output.push_str(&format!(
            "{} | {} | {:.1} | {} | {}\n",
            p.pid, p.name, p.cpu_usage, p.memory_usage / 1024 / 1024, p.status
        ));
    }

    ToolResult {
        success: true,
        output: tail_truncate(&output, CMD_OUTPUT_MAX_CHARS),
    }
}

fn execute_get_service_status(input: GetServiceStatusInput) -> ToolResult {
    let status = SystemProvider::get_service_status(&input.service_name);
    ToolResult {
        success: true,
        output: status,
    }
}

fn execute_search_system_files(input: SearchSystemFilesInput) -> ToolResult {
    let resolved_path = match enforce_sandbox(&input.path) {
        Ok(p) => p,
        Err(e) => return ToolResult { success: false, output: e },
    };

    println!("Searching '{}' in '{}' using rg", input.query, resolved_path.display());

    let mut process = Command::new("rg");
    process.arg("--json")
           .arg("--max-count").arg("100")
           .arg(&input.query)
           .arg(&resolved_path);

    match process.output() {
        Ok(out) => {
            let success = out.status.success() || out.status.code() == Some(1); // rg returns 1 if no matches
            let raw = String::from_utf8_lossy(&out.stdout);
            ToolResult {
                success,
                output: tail_truncate(&raw, CMD_OUTPUT_MAX_CHARS),
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: format!("Failed to execute rg: {}", e),
        },
    }
}

fn execute_get_system_logs(input: GetSystemLogsInput) -> ToolResult {
    let limit = input.limit.unwrap_or(50);
    match logs::get_system_logs(limit) {
        Ok(entries) => {
            let output = serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "Error serializing logs".to_string());
            ToolResult {
                success: true,
                output: tail_truncate(&output, CMD_OUTPUT_MAX_CHARS),
            }
        }
        Err(e) => ToolResult {
            success: false,
            output: format!("Error retrieving logs: {}", e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_system_files_allows_benign_diagnostic_paths() {
        let resolved = enforce_sandbox("/etc/hostname").expect("/etc/hostname should be allowed");
        assert!(resolved.starts_with("/etc"));
    }

    #[test]
    fn search_system_files_blocks_sensitive_paths() {
        let err = enforce_sandbox("/etc/shadow").expect_err("/etc/shadow should be blocked");
        assert!(err.contains("SECURITY VIOLATION"));
    }

    #[test]
    fn search_system_files_tool_surface_blocks_sensitive_paths() {
        let result = execute_search_system_files(SearchSystemFilesInput {
            query: "root".to_string(),
            path: "/etc/shadow".to_string(),
        });

        assert!(!result.success);
        assert!(result.output.contains("SECURITY VIOLATION"));
    }
}

use serde::Deserialize;
use std::process::Command;

use crate::security::policy::PermissionTier;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub model_name: String,
    pub context_size: usize,
    pub require_confirmation: bool,
    pub dangerous_commands: Vec<String>,
    pub exec_mode: String,
    pub chat_system_prompt: String,
    pub agentic_system_prompt: String,
    pub tool_permission_tier: String,
    pub audit_enabled: bool,
    pub audit_db_path: String,
    #[serde(skip)]
    pub permission_tier: PermissionTier,
}

impl AppConfig {
    /// Bridges to the existing Python configuration by evaluating `config.py`
    /// and extracting the variables as JSON.
    pub fn load_from_python() -> Result<Self, String> {
        let py_script = r#"
import sys, json, os

try:
    # Support both launch modes:
    # 1) cwd=agent-rs  -> ../scripts
    # 2) cwd=project   -> ./scripts
    candidates = [
        os.path.abspath('./scripts'),
        os.path.abspath('../scripts'),
    ]
    for candidate in candidates:
        if os.path.isdir(candidate):
            sys.path.insert(0, candidate)

    import config

    data = {
        "base_url": getattr(config, 'BASE_URL', 'http://127.0.0.1:8080/v1'),
        "model_name": getattr(config, 'MODEL_NAME', 'gpt-oss-20b'),
        "context_size": getattr(config, 'CONTEXT_SIZE', 8192),
        "require_confirmation": getattr(config, 'REQUIRE_CONFIRMATION', True),
        "dangerous_commands": getattr(config, 'DANGEROUS_COMMANDS', ["rm", "mv"]),
        "exec_mode": os.environ.get("HELIX_EXEC_MODE", "chat"),
        "chat_system_prompt": getattr(config, 'CHAT_SYSTEM_PROMPT', 'You are Helix running in chat mode. Reply directly and concisely. Never expose internal reasoning, analysis, or chain-of-thought. Do not output <think>, <thinking>, or <analysis> tags.'),
        "agentic_system_prompt": getattr(config, 'AGENTIC_SYSTEM_PROMPT', 'You are an autonomous local system orchestrator. You execute tasks using provided tools. Before each tool call, state your reasoning in one sentence. Never guess file paths - verify with list_directory first. If a command fails, read STDERR and retry with a corrected approach. Do not greet the user. Do not introduce yourself. Do not use conversational filler. Be concise. You have local tool access through these tools, so do not ask the user to run local file-system commands when a tool can do it.'),
        "tool_permission_tier": getattr(config, 'TOOL_PERMISSION_TIER', 'workspace_write'),
        "audit_enabled": getattr(config, 'AUDIT_ENABLED', True),
        "audit_db_path": getattr(config, 'AUDIT_DB_PATH', 'logs/audit.db'),
    }
    print(json.dumps(data))
except Exception as e:
    print(json.dumps({"error": str(e)}))
"#;

        let output = Command::new("python")
            .arg("-c")
            .arg(py_script)
            .output()
            .map_err(|e| format!("Failed to execute python bridge: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Python script failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut config: AppConfig = serde_json::from_str(&stdout).map_err(|e| {
            format!(
                "Failed to parse JSON config from python: {} - '{}'",
                e, stdout
            )
        })?;

        config.permission_tier = PermissionTier::from_config_value(&config.tool_permission_tier)
            .unwrap_or_else(|| {
                eprintln!(
                    "[Config Warning] Invalid TOOL_PERMISSION_TIER='{}'. Falling back to 'workspace_write'.",
                    config.tool_permission_tier
                );
                PermissionTier::WorkspaceWrite
            });

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_permission_tier_defaults_to_workspace_write_when_missing_equivalent() {
        let parsed = PermissionTier::from_config_value("workspace_write").unwrap_or_default();
        assert_eq!(parsed, PermissionTier::WorkspaceWrite);
    }

    #[test]
    fn valid_tool_permission_tier_values_map_correctly() {
        assert_eq!(
            PermissionTier::from_config_value("read_only"),
            Some(PermissionTier::ReadOnly)
        );
        assert_eq!(
            PermissionTier::from_config_value("workspace_write"),
            Some(PermissionTier::WorkspaceWrite)
        );
        assert_eq!(
            PermissionTier::from_config_value("full_exec"),
            Some(PermissionTier::FullExec)
        );
    }

    #[test]
    fn invalid_tool_permission_tier_falls_back_to_workspace_write() {
        let parsed = PermissionTier::from_config_value("invalid-tier")
            .unwrap_or(PermissionTier::WorkspaceWrite);
        assert_eq!(parsed, PermissionTier::WorkspaceWrite);
    }
}

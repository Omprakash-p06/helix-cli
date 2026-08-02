use serde::Deserialize;
use std::process::Command;
use std::time::Duration;

use crate::security::policy::PermissionTier;
use reqwest::Client as HttpClient;

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
    #[serde(default = "default_sandbox_interpreters")]
    pub sandbox_interpreters: bool,
    #[serde(skip)]
    pub permission_tier: PermissionTier,
    #[serde(skip)]
    pub backend_capabilities: BackendCapabilities,
}

fn default_sandbox_interpreters() -> bool { true }

#[derive(Debug, Clone, serde::Serialize, Deserialize, Default)]
pub struct BackendCapabilities {
    pub function_calling: bool,
    pub streaming: bool,
    pub grammar_sampling: bool,
    pub context_window: u32,
    pub model_id: String,
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

    # Honour the model selected at launch — rebuild the profile for that model
    # so GPU_LAYERS, BACKEND_HINT, and CONTEXT_SIZE are correct.
    helix_model_name = os.environ.get('HELIX_MODEL_NAME', '').strip()
    if helix_model_name:
        profile = config.build_model_entry(helix_model_name, config.DETECTED_VRAM_GB)
        effective_model_name = helix_model_name
        # Re-derive runtime settings from the selected model's profile
        gpu_layers = profile['gpu_layers']
        backend_hint = profile['backend_hint']
        context_size = profile['context_size']
        batch_size = profile['batch_size']
        ubatch_size = profile['ubatch_size']
    else:
        effective_model_name = getattr(config, 'MODEL_NAME', 'local-model')
        gpu_layers = getattr(config, 'GPU_LAYERS', 0)
        backend_hint = getattr(config, 'BACKEND_HINT', 'cpu')
        context_size = getattr(config, 'CONTEXT_SIZE', 8192)
        batch_size = getattr(config, 'BATCH_SIZE', 512)
        ubatch_size = getattr(config, 'UBATCH_SIZE', 256)

    chat_prompt = f"You are Helix, a helpful local AI assistant running {effective_model_name}. Be brief, direct, and friendly."
    agentic_prompt = (
        f"You are Helix, a local systems developer agent running {effective_model_name}. "
        "Be concise, precise, and practical. Execute minimal safe commands and report errors exactly."
    )

    data = {
        'base_url': getattr(config, 'BASE_URL', 'http://127.0.0.1:8080/v1'),
        'model_name': effective_model_name,
        'context_size': context_size,
        'require_confirmation': getattr(config, 'REQUIRE_CONFIRMATION', True),
        'dangerous_commands': getattr(config, 'DANGEROUS_COMMANDS', ['rm', 'mv']),
        'exec_mode': os.environ.get('HELIX_EXEC_MODE', 'chat'),
        'chat_system_prompt': getattr(config, 'CHAT_SYSTEM_PROMPT', chat_prompt),
        'agentic_system_prompt': getattr(config, 'AGENTIC_SYSTEM_PROMPT', agentic_prompt),
        'tool_permission_tier': getattr(config, 'TOOL_PERMISSION_TIER', 'workspace_write'),
        'audit_enabled': getattr(config, 'AUDIT_ENABLED', True),
        'audit_db_path': getattr(config, 'AUDIT_DB_PATH', 'logs/audit.db'),
    }
    print(json.dumps(data))
except Exception as e:
    print(json.dumps({'error': str(e)}))
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

/// Probe the llama-server backend to populate BackendCapabilities.
/// - Queries `/v1/models` to detect if server is running and get model ID
/// - Sets `function_calling: true` if the response model id contains "gemma" or "functionary"
///   OR if `/props` endpoint returns `has_jinja: true`
/// - Sets `context_window` from `app_config.context_size` (already loaded from Python config)
/// - Sets `streaming: true` always (llama-server always supports streaming)
/// - Sets `grammar_sampling: true` always (llama.cpp always supports GBNF grammar)
/// - Returns the updated `BackendCapabilities` — caller assigns to `config.backend_capabilities`
pub async fn probe_backend_capabilities(
    app_config: &AppConfig,
    client: &HttpClient,
) -> BackendCapabilities {
    let base = app_config.base_url.trim_end_matches('/');
    let models_url = format!("{}/v1/models", base);

    let mut caps = BackendCapabilities {
        function_calling: false,
        streaming: true,          // llama-server always supports streaming
        grammar_sampling: true,   // llama.cpp always supports GBNF
        context_window: app_config.context_size as u32,
        model_id: String::new(),
    };

    let body = match client
        .get(&models_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return caps,
        },
        Err(_) => return caps,
    };

    // Parse model_id from /v1/models response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body)
        && let Some(id) = json["data"][0]["id"].as_str()
    {
        caps.model_id = id.to_string();
        let id_lower = id.to_lowercase();
        // Enable function_calling for models known to support tool calls natively
        if id_lower.contains("gemma")
            || id_lower.contains("functionary")
            || id_lower.contains("hermes")
        {
            caps.function_calling = true;
        }
    }

    // Optionally check /props for has_jinja (confirms Jinja2 template is active → tool calling works)
    let props_url = format!("{}/props", base);
    if let Ok(r) = client
        .get(&props_url)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        && let Ok(t) = r.text().await
        && let Ok(props) = serde_json::from_str::<serde_json::Value>(&t)
        && props["has_jinja"].as_bool().unwrap_or(false)
    {
        caps.function_calling = true;
    }

    caps
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

    #[test]
    fn backend_capabilities_default_values() {
        let caps = BackendCapabilities::default();
        assert!(!caps.function_calling);
        assert!(!caps.streaming);
        assert_eq!(caps.context_window, 0);
        assert!(caps.model_id.is_empty());
    }
}

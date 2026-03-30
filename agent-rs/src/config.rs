use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub model_name: String,
    pub context_size: usize,
    pub require_confirmation: bool,
    pub dangerous_commands: Vec<String>,
    pub exec_mode: String,
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
        let config: AppConfig = serde_json::from_str(&stdout).map_err(|e| {
            format!(
                "Failed to parse JSON config from python: {} - '{}'",
                e, stdout
            )
        })?;

        Ok(config)
    }
}

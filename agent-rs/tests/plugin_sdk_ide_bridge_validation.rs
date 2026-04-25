use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

mod security {
    pub mod policy {
        include!("../src/security/policy.rs");
    }
}

mod types {
    include!("../src/types.rs");
}

pub use types::ChatMessage;

mod tokens {
    include!("../src/tokens.rs");
}

mod utils {
    include!("../src/utils.rs");
}

mod audit {
    include!("../src/audit.rs");
}

mod agent_core {
    pub mod tool_runtime {
        // Mock ToolResult for the include macro in other modules
        use serde::{Deserialize, Serialize};
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ToolResult {
            pub success: bool,
            pub output: String,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ToolRequest {
            pub call_id: String,
            pub name: String,
            pub arguments: serde_json::Value,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum ToolLifecycle {
            Start { id: String, name: String },
            Status { id: String, message: String },
            Result { id: String, success: bool, output_summary: String },
        }

        pub struct ToolRuntime;
        impl ToolRuntime {
            pub async fn execute(&self, _: ToolRequest, _: Vec<String>, _: bool, _: crate::security::policy::PolicyContext, _: Option<std::sync::Arc<crate::audit::AuditStore>>, _: String, _: std::sync::Arc<crate::tools::ToolRegistry>, _: Option<tokio::sync::mpsc::UnboundedSender<ToolLifecycle>>) -> (String, ToolResult, String) {
                ("id".into(), ToolResult { success: true, output: "mock".into() }, "name".into())
            }
        }
    }
    pub mod repair {
        pub mod tools {
            use serde_json::json;
            use crate::tools::{Tool, ToolResult};
            use crate::security::policy::PolicyContext;
            
            pub struct ServiceRepairTool;
            impl Tool for ServiceRepairTool {
                fn name(&self) -> String { "service_repair".into() }
                fn description(&self) -> String { "mock".into() }
                fn schema(&self) -> serde_json::Value { json!({}) }
                fn execute(&self, _: serde_json::Value, _: &[String], _: bool, _: &PolicyContext) -> ToolResult {
                    ToolResult { success: true, output: "mock".into() }
                }
            }

            pub struct PackageRepairTool;
            impl Tool for PackageRepairTool {
                fn name(&self) -> String { "package_repair".into() }
                fn description(&self) -> String { "mock".into() }
                fn schema(&self) -> serde_json::Value { json!({}) }
                fn execute(&self, _: serde_json::Value, _: &[String], _: bool, _: &PolicyContext) -> ToolResult {
                    ToolResult { success: true, output: "mock".into() }
                }
            }

            pub struct PermissionRepairTool;
            impl Tool for PermissionRepairTool {
                fn name(&self) -> String { "permission_repair".into() }
                fn description(&self) -> String { "mock".into() }
                fn schema(&self) -> serde_json::Value { json!({}) }
                fn execute(&self, _: serde_json::Value, _: &[String], _: bool, _: &PolicyContext) -> ToolResult {
                    ToolResult { success: true, output: "mock".into() }
                }
            }
        }
    }
    pub mod diagnostics {
        pub mod system {
            include!("../src/agent_core/diagnostics/system.rs");
        }
        pub mod logs {
            include!("../src/agent_core/diagnostics/logs.rs");
        }
    }
}

mod tools {
    include!("../src/tools.rs");
}

mod config {
    include!("../src/config.rs");
}

pub fn expose_think_blocks(text: &str) -> String {
    utils::clean_chat_output(text)
}

pub fn critic_message(text: &str) -> types::ChatMessage {
    types::ChatMessage {
        role: "user".to_string(),
        content: Some(format!("[Rust Critic] {}", text)),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

mod server {
    include!("../src/server.rs");

    #[cfg(test)]
    mod validation {
        use super::*;
        use axum::extract::State;
        use reqwest::Client;
        use serde_json::json;
        use std::collections::HashSet;
        use std::sync::{Arc, Mutex, OnceLock};

        use crate::config::AppConfig;
        use crate::security::policy::PermissionTier;
        use crate::types::ServerFlavor;

        fn env_lock() -> &'static Mutex<()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(()))
        }

        fn sample_state() -> AppState {
            AppState {
                client: Client::new(),
                tool_runtime: Arc::new(crate::agent_core::tool_runtime::ToolRuntime),
                app_config: AppConfig {
                    base_url: "http://127.0.0.1:8080/v1".to_string(),
                    model_name: "demo-model".to_string(),
                    context_size: 8192,
                    require_confirmation: true,
                    dangerous_commands: vec!["rm".to_string()],
                    exec_mode: "agentic".to_string(),
                    chat_system_prompt: "chat prompt".to_string(),
                    agentic_system_prompt: "agentic prompt".to_string(),
                    tool_permission_tier: "workspace_write".to_string(),
                    audit_enabled: true,
                    audit_db_path: "logs/audit.db".to_string(),
                    permission_tier: PermissionTier::WorkspaceWrite,
                },
                generated_grammar: String::new(),
                tools_payload: json!([]),
                server_flavor: ServerFlavor::LlamaCpp,
                audit_store: None,
                tool_registry: Arc::new(crate::tools::create_default_registry()),
            }
        }

        #[test]
        fn registry_exposes_all_builtins_and_persona_filters_payloads() {
            let registry = crate::tools::create_default_registry();
            let names: HashSet<String> = registry
                .list_tools()
                .into_iter()
                .map(|tool| tool.name())
                .collect();

            let expected = HashSet::from([
                "run_terminal_command".to_string(),
                "read_file".to_string(),
                "write_file".to_string(),
                "append_file".to_string(),
                "list_directory".to_string(),
                "get_system_stats".to_string(),
                "search_codebase".to_string(),
                "list_processes".to_string(),
                "get_service_status".to_string(),
                "search_system_files".to_string(),
                "get_system_logs".to_string(),
                "service_repair".to_string(),
                "package_repair".to_string(),
                "permission_repair".to_string(),
            ]);

            assert_eq!(names, expected);

            let os_assistant_payload = registry.build_tools_payload("os_assistant", true);
            let os_tools = os_assistant_payload.as_array().expect("payload array");
            assert_eq!(os_tools.len(), 13);
            assert!(os_tools.iter().any(|tool| tool["function"]["name"] == "run_terminal_command"));
            assert!(!os_tools.iter().any(|tool| tool["function"]["name"] == "search_codebase"));

            let coder_payload = registry.build_tools_payload("coder", true);
            let coder_tools = coder_payload.as_array().expect("payload array");
            assert_eq!(coder_tools.len(), 9);
            assert!(!coder_tools.iter().any(|tool| tool["function"]["name"] == "run_terminal_command"));
            assert!(coder_tools.iter().any(|tool| tool["function"]["name"] == "write_file"));
        }

        #[tokio::test]
        async fn status_and_context_endpoints_return_expected_contracts() {
            let state = sample_state();

            let status = status_handler(State(state.clone())).await;
            assert_eq!(status.0.status, "ok");
            assert_eq!(status.0.version, "0.1.0");
            assert_eq!(status.0.model, "demo-model");
            assert_eq!(status.0.server_flavor, "LlamaCpp");

            let context = context_handler().await;
            assert!(!context.0.workspace_root.is_empty());
            assert!(!context.0.git_branch.is_empty());
        }

        #[tokio::test]
        async fn tools_handler_matches_registry_discovery_payload() {
            let _guard = env_lock().lock().expect("env lock");
            unsafe { std::env::set_var("AGENT_PERSONA", "os_assistant"); }

            let state = sample_state();
            let actual = tools_handler(State(state.clone())).await.0;
            let expected = state.tool_registry.build_tools_payload("os_assistant", true);
            assert_eq!(actual, expected);

            unsafe { std::env::remove_var("AGENT_PERSONA"); }
        }
    }
}

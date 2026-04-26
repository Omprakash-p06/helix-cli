use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use tokio::time::{timeout, Duration};

use crate::security::policy::{PolicyContext, PolicyDecision, evaluate_tool_call};
use crate::security::sandbox::DockerSandbox;
use crate::tools::{ToolRegistry};
use crate::audit::{self, AuditStore};
use crate::types::{PermissionRequest, PermissionResponse, PermissionRequester};
use crate::agent_core::repair::workflow::SafetyLoop;

/// Structured tool result with deterministic success signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

/// Lifecycle events emitted during tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ToolLifecycle {
    #[serde(rename = "tool_start")]
    Start { id: String, name: String },
    #[serde(rename = "tool_status")]
    Status { id: String, message: String },
    #[serde(rename = "tool_result")]
    Result { id: String, success: bool, output_summary: String },
}

/// Request envelope for executing a tool.
#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
    pub confidence: f64,
}

pub struct ToolRuntime {
    pub permission_requester: Option<Arc<dyn PermissionRequester>>,
    pub safety_loop: Option<Arc<SafetyLoop>>,
}

impl ToolRuntime {
    pub fn new(
        permission_requester: Option<Arc<dyn PermissionRequester>>,
        safety_loop: Option<Arc<SafetyLoop>>,
    ) -> Self {
        Self { permission_requester, safety_loop }
    }

    /// Executes a single tool with standard lifecycle and audit hooks.
    /// Returns the tool result and the call ID.
    pub async fn execute(
        &self,
        req: ToolRequest,
        dangerous_commands: Vec<String>,
        require_confirmation: bool,
        policy_context: PolicyContext,
        audit_store: Option<Arc<AuditStore>>,
        path: String,
        registry: Arc<ToolRegistry>,
        event_tx: Option<tokio::sync::mpsc::UnboundedSender<ToolLifecycle>>,
    ) -> (String, ToolResult, String) {
        let id = req.call_id.clone();
        let func_name = req.name.clone();

        if let Some(tx) = &event_tx {
            let _ = tx.send(ToolLifecycle::Start {
                id: id.clone(),
                name: func_name.clone(),
            });
        }

        let result = match timeout(
            Duration::from_secs(30),
            spawn_blocking({
                let self_clone = Arc::new(Self::new(self.permission_requester.clone(), self.safety_loop.clone()));
                let req_clone = req.clone();
                let dangerous_commands_clone = dangerous_commands.clone();
                let policy_context_clone = policy_context.clone();
                let path_clone = path.clone();
                let registry_clone = registry.clone();
                
                move || {
                    self_clone.execute_sync(
                        req_clone,
                        &dangerous_commands_clone,
                        require_confirmation,
                        &policy_context_clone,
                        audit_store,
                        &path_clone,
                        &registry_clone,
                    )
                }
            }),
        )
        .await
        {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => ToolResult {
                success: false,
                output: format!("Tool '{}' execution failed: {}", func_name, e),
            },
            Err(_) => ToolResult {
                success: false,
                output: format!("Tool '{}' timed out after 30 seconds", func_name),
            },
        };

        if let Some(tx) = &event_tx {
            let summary = if result.success {
                "Success".to_string()
            } else {
                format!("Failed: {}", result.output.chars().take(100).collect::<String>())
            };
            let _ = tx.send(ToolLifecycle::Result {
                id: id.clone(),
                success: result.success,
                output_summary: summary,
            });
        }

        (id, result, func_name)
    }

    fn execute_sync(
        &self,
        req: ToolRequest,
        dangerous_commands: &[String],
        require_confirmation: bool,
        policy_context: &PolicyContext,
        audit_store: Option<Arc<AuditStore>>,
        path: &str,
        registry: &ToolRegistry,
    ) -> ToolResult {
        let func_name = req.name.clone();
        let parsed_args = req.arguments.clone();

        let args_payload = parsed_args.to_string();
        let args_hash = audit::hash_payload(&args_payload);

        let decision = evaluate_tool_call(&func_name, &parsed_args, policy_context);

        if let Some(store) = &audit_store {
            let (dec_str, reason, remediation) = match &decision {
                PolicyDecision::Allow => ("allow", None, None),
                PolicyDecision::RequireApproval {
                    reason_code,
                    message,
                } => ("approval-required", Some(reason_code.as_str()), Some(message.as_str())),
                PolicyDecision::Deny {
                    reason_code,
                    remediation,
                    ..
                } => ("deny", Some(reason_code.as_str()), Some(remediation.as_str())),
            };
            let _ = store.append_event(
                "user",
                path,
                "policy",
                &func_name,
                Some(dec_str),
                None,
                reason,
                remediation,
                &args_hash,
                None,
                None,
            );
        }

        match decision {
            PolicyDecision::Allow => {}
            PolicyDecision::RequireApproval {
                reason_code,
                message,
            } => {
                if let Some(requester) = &self.permission_requester {
                    let handle = tokio::runtime::Handle::current();
                    
                    let mut reason = format!("[{}] {}", reason_code, message);
                    if req.confidence < 0.8 {
                        reason = format!("⚠️ LOW CONFIDENCE WARNING (Confidence: {:.2})\n{}", req.confidence, reason);
                    }

                    let request = PermissionRequest {
                        tool_name: func_name.clone(),
                        arguments: parsed_args.clone(),
                        reason,
                    };

                    let response = handle.block_on(async {
                        requester.request_permission(request).await
                    });

                    if response != PermissionResponse::Allow {
                        return ToolResult {
                            success: false,
                            output: format!("Execution denied by user: [{}] {}", reason_code, message),
                        };
                    }
                    // If allowed, continue to execution
                } else {
                    return ToolResult {
                        success: false,
                        output: format!("[Approval Required: {}] {}. No permission requester available.", reason_code, message),
                    };
                }
            }
            PolicyDecision::Deny {
                reason_code,
                message,
                remediation,
            } => {
                return ToolResult {
                    success: false,
                    output: format!(
                        "[Policy Denied: {}] {} Remediation: {}",
                        reason_code, message, remediation
                    ),
                };
            }
        }

        let start_time = std::time::Instant::now();

        let result = if func_name == "run_terminal_command" {
            Self::execute_sandboxed_command(&parsed_args, policy_context)
        } else if let Some(tool) = registry.get(&func_name) {
            if tool.is_transactional() {
                if let Some(safety_loop) = &self.safety_loop {
                    let res = safety_loop.execute_transactional(
                        || tool.execute(parsed_args, dangerous_commands, require_confirmation, policy_context),
                        |res| res.success // Default validation: just check success
                    );
                    match res {
                        Ok(r) => r,
                        Err(e) => ToolResult { success: false, output: e },
                    }
                } else {
                    // Fallback to normal execution if no safety loop
                    tool.execute(parsed_args, dangerous_commands, require_confirmation, policy_context)
                }
            } else {
                tool.execute(parsed_args, dangerous_commands, require_confirmation, policy_context)
            }
        } else {
            ToolResult {
                success: false,
                output: format!("Unknown tool: {}", func_name),
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as i64;

        if let Some(store) = &audit_store {
            let outcome = if result.success { "success" } else { "failure" };
            let output_hash = audit::hash_payload(&result.output);
            let _ = store.append_event(
                "agent",
                path,
                "execution",
                &func_name,
                None,
                Some(outcome),
                None,
                None,
                &args_hash,
                Some(&output_hash),
                Some(duration_ms),
            );
        }

        result
    }

    fn execute_sandboxed_command(parsed_args: &Value, policy_context: &PolicyContext) -> ToolResult {
        let Some(raw_cmd) = parsed_args.get("command").and_then(|v| v.as_str()) else {
            return ToolResult {
                success: false,
                output: "run_terminal_command requires a 'command' string.".to_string(),
            };
        };

        let sandbox = match DockerSandbox::new(policy_context.workspace_root.clone()) {
            Ok(sandbox) => sandbox,
            Err(err) => {
                return ToolResult {
                    success: false,
                    output: format!("Sandbox initialization failed: {err}"),
                };
            }
        };

        let handle = tokio::runtime::Handle::current();
        let command = shell_words::split(raw_cmd).unwrap_or_else(|_| vec![raw_cmd.to_string()]);
        let output = handle.block_on(async {
            sandbox
                .run_command(
                    command,
                    "alpine:3.20",
                    policy_context.workspace_root.clone(),
                )
                .await
        });

        match output {
            Ok(output) => {
                let success = output.status_code == 0;
                let mut rendered = String::new();
                if !output.stdout.trim().is_empty() {
                    rendered.push_str("STDOUT:\n");
                    rendered.push_str(output.stdout.trim_end());
                    rendered.push('\n');
                }
                if !output.stderr.trim().is_empty() {
                    rendered.push_str("STDERR:\n");
                    rendered.push_str(output.stderr.trim_end());
                    rendered.push('\n');
                }
                if rendered.is_empty() {
                    rendered = format!(
                        "Command completed inside Docker sandbox with exit code {}.",
                        output.status_code
                    );
                } else {
                    rendered.push_str(&format!("EXIT CODE: {}", output.status_code));
                }
                ToolResult {
                    success,
                    output: rendered,
                }
            }
            Err(err) => ToolResult {
                success: false,
                output: format!("Sandbox execution failed: {err}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::policy::{PermissionTier, PolicyContext};
    use crate::types::{PermissionRequest, PermissionResponse, PermissionRequester};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use serde_json::json;

    struct MockRequester {
        response: PermissionResponse,
        called: AtomicBool,
    }

    #[async_trait::async_trait]
    impl PermissionRequester for MockRequester {
        async fn request_permission(&self, _request: PermissionRequest) -> PermissionResponse {
            self.called.store(true, Ordering::SeqCst);
            self.response.clone()
        }
    }

    #[test]
    fn test_hitl_interception_approve() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let requester = Arc::new(MockRequester { 
            response: PermissionResponse::Allow,
            called: AtomicBool::new(false),
        });
        let runtime = ToolRuntime::new(Some(requester.clone()), None);
        
        let req = ToolRequest {
            call_id: "test".to_string(),
            name: "run_terminal_command".to_string(),
            arguments: json!({"command": "chmod +x script.sh"}),
            confidence: 0.9,
        };

        let ctx = PolicyContext {
            permission_tier: PermissionTier::FullExec,
            exec_mode: "test".to_string(),
            workspace_root: std::env::temp_dir(),
        };

        let registry = Arc::new(ToolRegistry::new());
        
        let handle_clone = handle.clone();
        let _result = std::thread::spawn(move || {
            let _guard = handle_clone.enter();
            runtime.execute_sync(
                req,
                &[],
                false,
                &ctx,
                None,
                "test_path",
                &registry,
            )
        }).join().unwrap();

        assert!(requester.called.load(Ordering::SeqCst), "Requester should have been called");
    }

    #[test]
    fn test_hitl_interception_deny() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let requester = Arc::new(MockRequester { 
            response: PermissionResponse::Deny,
            called: AtomicBool::new(false),
        });
        let runtime = ToolRuntime::new(Some(requester.clone()), None);
        
        let req = ToolRequest {
            call_id: "test".to_string(),
            name: "run_terminal_command".to_string(),
            arguments: json!({"command": "chmod +x script.sh"}),
            confidence: 0.9,
        };

        let ctx = PolicyContext {
            permission_tier: PermissionTier::FullExec,
            exec_mode: "test".to_string(),
            workspace_root: std::env::temp_dir(),
        };

        let registry = Arc::new(ToolRegistry::new());
        
        let handle_clone = handle.clone();
        let result = std::thread::spawn(move || {
            let _guard = handle_clone.enter();
            runtime.execute_sync(
                req,
                &[],
                false,
                &ctx,
                None,
                "test_path",
                &registry,
            )
        }).join().unwrap();

        assert!(requester.called.load(Ordering::SeqCst), "Requester should have been called");
        assert!(!result.success);
        assert!(result.output.contains("Execution denied by user"));
    }

    #[test]
    fn test_low_confidence_warning_injection() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();

        struct WarningRequester {
            captured_reason: std::sync::Mutex<String>,
        }

        #[async_trait::async_trait]
        impl PermissionRequester for WarningRequester {
            async fn request_permission(&self, request: PermissionRequest) -> PermissionResponse {
                let mut guard = self.captured_reason.lock().unwrap();
                *guard = request.reason;
                PermissionResponse::Deny
            }
        }

        let requester = Arc::new(WarningRequester { 
            captured_reason: std::sync::Mutex::new(String::new()),
        });
        let runtime = ToolRuntime::new(Some(requester.clone()), None);
        
        let req = ToolRequest {
            call_id: "test".to_string(),
            name: "run_terminal_command".to_string(),
            arguments: json!({"command": "chmod +x script.sh"}),
            confidence: 0.7, // Below 0.8 threshold
        };

        let ctx = PolicyContext {
            permission_tier: PermissionTier::FullExec,
            exec_mode: "test".to_string(),
            workspace_root: std::env::temp_dir(),
        };

        let registry = Arc::new(ToolRegistry::new());
        
        let handle_clone = handle.clone();
        let _ = std::thread::spawn(move || {
            let _guard = handle_clone.enter();
            runtime.execute_sync(
                req,
                &[],
                false,
                &ctx,
                None,
                "test_path",
                &registry,
            )
        }).join().unwrap();

        let captured = requester.captured_reason.lock().unwrap();
        assert!(captured.contains("LOW CONFIDENCE WARNING"), "Reason should contain warning. Captured: {}", *captured);
        assert!(captured.contains("0.70"));
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use tokio::time::{timeout, Duration};

use crate::security::policy::{PolicyContext, PolicyDecision, evaluate_tool_call};
use crate::tools::{ToolRegistry};
use crate::audit::{self, AuditStore};

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
}

pub struct ToolRuntime;

impl ToolRuntime {
    /// Executes a single tool with standard lifecycle and audit hooks.
    /// Returns the tool result and the call ID.
    pub async fn execute(
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
            spawn_blocking(move || {
                Self::execute_sync(
                    req,
                    &dangerous_commands,
                    require_confirmation,
                    &policy_context,
                    audit_store,
                    &path,
                    &registry,
                )
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
        req: ToolRequest,
        dangerous_commands: &[String],
        require_confirmation: bool,
        policy_context: &PolicyContext,
        audit_store: Option<Arc<AuditStore>>,
        path: &str,
        registry: &ToolRegistry,
    ) -> ToolResult {
        let func_name = req.name;
        let parsed_args = req.arguments;

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
                return ToolResult {
                    success: false,
                    output: format!("[Approval Required: {}] {}", reason_code, message),
                };
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
        
        let result = if let Some(tool) = registry.get(&func_name) {
            tool.execute(parsed_args, dangerous_commands, require_confirmation, policy_context)
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
}

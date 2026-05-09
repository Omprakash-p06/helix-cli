use agent_rs::agent_core::tool_runtime::{ToolRequest, ToolRuntime};
use agent_rs::audit::AuditStore;
use agent_rs::security::policy::{PermissionTier, PolicyContext, TrustLevel};
use agent_rs::tools;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn docker_available() -> bool {
    let daemon_ok = std::process::Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !daemon_ok {
        return false;
    }

    // Also check if alpine:3.20 is available to avoid 404 in tests
    std::process::Command::new("docker")
        .args(["image", "inspect", "alpine:3.20"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn temp_db_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("helix-secure-execution-{unique}.db"))
}

#[tokio::test]
async fn secure_runtime_logs_safe_and_blocked_commands() {
    if !docker_available() {
        eprintln!("Skipping secure execution test because docker is unavailable.");
        return;
    }

    let audit_path = temp_db_path();
    let store = Arc::new(AuditStore::new(&audit_path).expect("audit store"));
    let registry = Arc::new(tools::create_default_registry());
    let workspace_root = std::fs::canonicalize("..").expect("workspace root");

    let policy_context = PolicyContext {
        permission_tier: PermissionTier::FullExec,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::FullExec),
        exec_mode: "agentic".to_string(),
        workspace_root,
    };

    let safe_req = ToolRequest {
        call_id: "safe-1".to_string(),
        name: "run_terminal_command".to_string(),
        arguments: json!({ "command": "pwd" }),
        confidence: 1.0,
    };

    let blocked_req = ToolRequest {
        call_id: "blocked-1".to_string(),
        name: "run_terminal_command".to_string(),
        arguments: json!({ "command": "ls | grep src" }),
        confidence: 1.0,
    };

    let tool_runtime = ToolRuntime::new(None, None);

    let (_, safe_result, _) = tool_runtime.execute(
        safe_req,
        vec![],
        false,
        policy_context.clone(),
        Some(store.clone()),
        "test".to_string(),
        registry.clone(),
        None,
    )
    .await;
    assert!(safe_result.success, "safe command should succeed: {}", safe_result.output);

    let (_, blocked_result, _) = tool_runtime.execute(
        blocked_req,
        vec![],
        false,
        policy_context,
        Some(store.clone()),
        "test".to_string(),
        registry,
        None,
    )
    .await;
    assert!(
        !blocked_result.success && blocked_result.output.contains("Policy Denied"),
        "blocked command should be denied before sandbox execution"
    );

    let events = store
        .query_events(None, None, Some("test"), Some("run_terminal_command"), None, None)
        .expect("query audit events");

    let policy_events = events.iter().filter(|event| event.event_type == "policy").count();
    let execution_events = events
        .iter()
        .filter(|event| event.event_type == "execution")
        .count();

    assert_eq!(policy_events, 2, "both commands should be logged at policy stage");
    assert_eq!(
        execution_events, 1,
        "only the safe command should reach the sandbox execution stage"
    );
    assert!(store.verify_chain().expect("verify audit chain"));

    let _ = std::fs::remove_file(audit_path);
}

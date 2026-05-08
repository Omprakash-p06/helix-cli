use agent_rs::agent_core::tool_runtime::{ToolRuntime, ToolRequest, ToolLifecycle};
use agent_rs::tools::{self, ToolResult};
use agent_rs::tools::Tool;
use agent_rs::security::policy::{PolicyContext, PermissionTier, TrustLevel};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc;
use futures_util::future::join_all;

#[tokio::test]
async fn test_tool_runtime_basic_execution() {
    let registry = Arc::new(tools::create_default_registry());

    let req = ToolRequest {
        call_id: "test_1".to_string(),
        name: "get_system_stats".to_string(),
        arguments: json!({}),
        confidence: 1.0,
    };

    let policy_context = PolicyContext {
        permission_tier: PermissionTier::WorkspaceWrite,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::WorkspaceWrite),
        exec_mode: "chat".to_string(),
        workspace_root: std::path::PathBuf::from("."),
    };

    let tool_runtime = ToolRuntime::new(None, None);
    let (id, result, name) = tool_runtime.execute(
        req,
        vec![],
        false,
        policy_context,
        None,
        "test".to_string(),
        registry,
        None,
    ).await;

    assert_eq!(id, "test_1");
    assert_eq!(name, "get_system_stats");
    assert!(result.success);
    assert!(result.output.contains("RAM"));
}

#[tokio::test]
async fn test_tool_runtime_concurrent_ordering() {
    let registry = Arc::new(tools::create_default_registry());

    let policy_context = PolicyContext {
        permission_tier: PermissionTier::WorkspaceWrite,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::WorkspaceWrite),
        exec_mode: "chat".to_string(),
        workspace_root: std::path::PathBuf::from("."),
    };

    let requests = vec![
        ToolRequest {
            call_id: "call_0".to_string(),
            name: "get_system_stats".to_string(),
            arguments: json!({}),
            confidence: 1.0,
        },
        ToolRequest {
            call_id: "call_1".to_string(),
            name: "get_system_stats".to_string(),
            arguments: json!({}),
            confidence: 1.0,
        },
    ];

    let tool_runtime = Arc::new(ToolRuntime::new(None, None));

    let tasks: Vec<_> = requests.into_iter().enumerate().map(|(idx, req)| {
        let registry_owned = registry.clone();
        let pc_owned = policy_context.clone();
        let tool_runtime_owned = tool_runtime.clone();
        async move {
            (idx, tool_runtime_owned.execute(
                req,
                vec![],
                false,
                pc_owned,
                None,
                "test".to_string(),
                registry_owned,
                None,
            ).await)
        }
    }).collect();

    let mut results = join_all(tasks).await;
    results.sort_by_key(|(idx, _)| *idx);

    assert_eq!(results[0].1.0, "call_0");
    assert_eq!(results[1].1.0, "call_1");
}

struct SlowTool;
impl Tool for SlowTool {
    fn name(&self) -> String { "slow_tool".into() }
    fn description(&self) -> String { "sleeps for 35 seconds".into() }
    fn schema(&self) -> Value { json!({}) }
    fn execute(&self, _args: Value, _dc: &[String], _rc: bool, _ctx: &PolicyContext) -> ToolResult {
        std::thread::sleep(std::time::Duration::from_secs(35));
        ToolResult { success: true, output: "done".into() }
    }
}

struct AlwaysAllowRequester;
#[async_trait::async_trait]
impl agent_rs::types::PermissionRequester for AlwaysAllowRequester {
    async fn request_permission(&self, _req: agent_rs::types::PermissionRequest) -> agent_rs::types::PermissionResponse {
        agent_rs::types::PermissionResponse::Allow
    }
}

#[tokio::test]
async fn test_tool_runtime_timeout() {
    let mut registry = tools::create_default_registry();
    registry.register(Box::new(SlowTool));
    let registry = Arc::new(registry);
    let req = ToolRequest {
        call_id: "timeout_test".to_string(),
        name: "slow_tool".to_string(),
        arguments: json!({}),
        confidence: 1.0,
    };

    let policy_context = PolicyContext {
        permission_tier: PermissionTier::FullExec,
        trust_level: TrustLevel::Full,
        exec_mode: "chat".to_string(),
        workspace_root: std::path::PathBuf::from("."),
    };

    let requester = Arc::new(AlwaysAllowRequester);
    let tool_runtime = ToolRuntime::new(Some(requester), None);

    let (_, result, _) = tool_runtime.execute(
        req,
        vec![],
        false,
        policy_context,
        None,
        "test".to_string(),
        registry,
        None,
    ).await;

    assert!(!result.success, "tool should fail due to 30s outer timeout, got: {}", result.output);
    assert!(result.output.contains("timed out"), "output should mention timeout, got: {}", result.output);
}

#[tokio::test]
async fn test_tool_runtime_lifecycle_events() {
    let registry = Arc::new(tools::create_default_registry());

    let req = ToolRequest {
        call_id: "event_test".to_string(),
        name: "get_system_stats".to_string(),
        arguments: json!({}),
        confidence: 1.0,
    };

    let policy_context = PolicyContext {
        permission_tier: PermissionTier::WorkspaceWrite,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::WorkspaceWrite),
        exec_mode: "chat".to_string(),
        workspace_root: std::path::PathBuf::from("."),
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<ToolLifecycle>();

    let tool_runtime = ToolRuntime::new(None, None);

    tool_runtime.execute(
        req,
        vec![],
        false,
        policy_context,
        None,
        "test".to_string(),
        registry,
        Some(tx),
    ).await;

    let ev1 = rx.recv().await.unwrap();
    if let ToolLifecycle::Start { id, name } = ev1 {
        assert_eq!(id, "event_test");
        assert_eq!(name, "get_system_stats");
    } else {
        panic!("Expected Start event");
    }

    let ev2 = rx.recv().await.unwrap();
    if let ToolLifecycle::Result { id, success, .. } = ev2 {
        assert_eq!(id, "event_test");
        assert!(success);
    } else {
        panic!("Expected Result event");
    }
}
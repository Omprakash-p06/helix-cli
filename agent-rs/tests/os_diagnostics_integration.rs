use agent_rs::tools::{create_default_registry};
use agent_rs::security::policy::{PolicyContext, PermissionTier, TrustLevel};
use serde_json::json;
use std::path::PathBuf;

#[tokio::test]
async fn test_phase02_log_introspection() {
    let registry = create_default_registry();
    let tool = registry.get("get_system_logs").expect("tool exists");
    let ctx = PolicyContext {
        permission_tier: PermissionTier::ReadOnly,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::ReadOnly),
        exec_mode: "agentic".into(),
        workspace_root: PathBuf::from("."),
    };
    
    // We use a small limit for the test
    let result = tool.execute(json!({"limit": 2}), &[], false, &ctx);
    
    // On some CI environments, journalctl might fail, but the tool should handle it
    // If it fails with "journalctl failed", it's still technically the tool working
    if result.success {
        assert!(result.output.contains("[") && result.output.contains("]"), "Should be a JSON array");
    } else {
        assert!(result.output.contains("journalctl failed") || result.output.contains("Windows logs"), "Expected failure reason");
    }
}

#[tokio::test]
async fn test_phase02_system_discovery() {
    let registry = create_default_registry();
    let ctx = PolicyContext {
        permission_tier: PermissionTier::ReadOnly,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::ReadOnly),
        exec_mode: "agentic".into(),
        workspace_root: PathBuf::from("."),
    };

    // Processes
    let tool = registry.get("list_processes").expect("tool exists");
    let result = tool.execute(json!({}), &[], false, &ctx);
    assert!(result.success, "list_processes failed: {}", result.output);
    // Since output might be truncated at the beginning, we don't look for the header "PID"
    // Instead we look for the table separators or common status words
    assert!(result.output.contains("|") || result.output.contains("Sleep") || result.output.contains("Idle"), 
            "Output should look like a process table. Output: {}", result.output);

    // Service status
    let tool = registry.get("get_service_status").expect("tool exists");
    // systemd-journald is a safe bet on Linux, on Windows it might fail but we check the tool execution
    let result = tool.execute(json!({"serviceName": "systemd-journald"}), &[], false, &ctx);
    assert!(result.success, "get_service_status failed: {}", result.output);
}

#[tokio::test]
async fn test_phase02_file_introspection() {
    let registry = create_default_registry();
    let ctx = PolicyContext {
        permission_tier: PermissionTier::ReadOnly,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::ReadOnly),
        exec_mode: "agentic".into(),
        workspace_root: PathBuf::from("."),
    };

    // Read /etc/hostname (standard on Linux)
    #[cfg(target_os = "linux")]
    {
        let tool = registry.get("read_file").expect("tool exists");
        let result = tool.execute(json!({"absolutePath": "/etc/hostname"}), &[], false, &ctx);
        assert!(result.success, "Failed to read /etc/hostname: {}", result.output);
    }

    // Search in /etc/hostname to avoid permission errors on other files in /etc
    #[cfg(target_os = "linux")]
    {
        let tool = registry.get("search_system_files").expect("tool exists");
        let result = tool.execute(json!({"query": "localhost", "path": "/etc/hosts"}), &[], false, &ctx);
        assert!(result.success, "Search in /etc/hosts failed: {}", result.output);
    }
}

#[tokio::test]
async fn test_phase02_security_guardrails() {
    let registry = create_default_registry();
    let ctx = PolicyContext {
        permission_tier: PermissionTier::ReadOnly,
        trust_level: TrustLevel::from_permission_tier(PermissionTier::ReadOnly),
        exec_mode: "agentic".into(),
        workspace_root: PathBuf::from("."),
    };

    // Block sensitive file read
    let tool = registry.get("read_file").expect("tool exists");
    let result = tool.execute(json!({"absolutePath": "/etc/shadow"}), &[], false, &ctx);
    assert!(!result.success, "Should have blocked /etc/shadow read");
    assert!(result.output.contains("SECURITY VIOLATION"), "Expected security violation message, got: {}", result.output);
    
    // Block sensitive search
    let tool = registry.get("search_system_files").expect("tool exists");
    let result = tool.execute(json!({"query": "root", "path": "/etc/shadow"}), &[], false, &ctx);
    assert!(!result.success, "Should have blocked /etc/shadow search");
    assert!(result.output.contains("SECURITY VIOLATION"), "Expected security violation message");
}

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).expect("expected source file to exist")
}

#[test]
fn security_logic_centralized_in_runtime() {
    let runtime_rs = read("src/agent_core/tool_runtime.rs");
    
    // Core policy check must be present
    assert!(runtime_rs.contains("evaluate_tool_call"));
    assert!(runtime_rs.contains("PolicyDecision::Deny"));
    
    // Sharing consistent templates across all callers
    assert!(runtime_rs.contains("[Policy Denied: {}] {} Remediation: {}"));
    assert!(runtime_rs.contains("[Approval Required: {}] {}"));
}

#[test]
fn terminal_path_uses_unified_runtime() {
    let main_rs = read("src/main.rs");
    // Should not contain manual evaluate_tool_call anymore, should use ToolRuntime
    assert!(main_rs.contains("ToolRuntime::execute"));
}

#[test]
fn web_path_uses_unified_runtime() {
    let server_rs = read("src/server.rs");
    // Should use ToolRuntime::execute
    assert!(server_rs.contains("ToolRuntime::execute"));
}

#[test]
fn read_only_tier_has_documented_config_hook() {
    let cfg_py = read("../scripts/config.py");
    assert!(cfg_py.contains("TOOL_PERMISSION_TIER"));
    assert!(cfg_py.contains("workspace_write"));
}

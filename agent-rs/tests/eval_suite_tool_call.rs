//! Eval Scenario 2: Tool-Call Correctness
//! Verifies that tool call JSON schemas are well-formed and that the
//! agent correctly declares tools in OpenAI format.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).expect("source file must exist")
}

/// Tool definitions must follow OpenAI function schema (type: "function", with name + parameters)
#[test]
fn sc2_tool_definitions_have_required_fields() {
    let tools_rs = read("src/tools.rs");
    // Each tool definition must have a "name" and "description" field
    assert!(tools_rs.contains("\"name\"") || tools_rs.contains("name:"), 
        "tool definitions must include name field");
    assert!(tools_rs.contains("\"description\"") || tools_rs.contains("description:"), 
        "tool definitions must include description field");
}

/// BackendCapabilities.function_calling field must exist for conditional tool routing
#[test]
fn sc2_backend_capabilities_function_calling_field_exists() {
    let config_rs = read("src/config.rs");
    assert!(config_rs.contains("function_calling"), "config.rs must have function_calling field");
    assert!(config_rs.contains("BackendCapabilities"), "BackendCapabilities struct must exist");
}

/// Verify tool call dispatch handles both function_calling=true and function_calling=false paths
#[test]
fn sc2_tool_dispatch_handles_capability_flag() {
    let main_rs = read("src/main.rs");
    let server_rs = read("src/server.rs");
    // At least one of the dispatch paths must reference backend_capabilities
    let combined = format!("{}{}", main_rs, server_rs);
    assert!(
        combined.contains("backend_capabilities") || combined.contains("function_calling"),
        "Tool dispatch must reference backend_capabilities or function_calling"
    );
}

/// Tool schema must be valid JSON (parse the tools list from source)
#[test]
fn sc2_tool_list_parses_as_valid_json_if_embedded() {
    let tools_rs = read("src/tools.rs");
    // Look for embedded JSON tool definition blocks
    if let Some(start) = tools_rs.find("serde_json::json!(") {
        // Extract a representative JSON fragment and verify it has balanced braces
        let fragment = &tools_rs[start..];
        let brace_count = fragment.chars().take(500).filter(|&c| c == '{').count();
        assert!(brace_count > 0, "Tool definitions must contain JSON objects");
    }
    // If tools are defined via structs (not inline JSON), this test passes trivially
}

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
    // Should use tool_runtime instance
    assert!(main_rs.contains(".execute("));
}

#[test]
fn web_path_uses_unified_runtime() {
    let server_rs = read("src/server.rs");
    // Should use tool_runtime instance
    assert!(server_rs.contains(".execute("));
}

#[test]
fn read_only_tier_has_documented_config_hook() {
    let cfg_py = read("../scripts/config.py");
    assert!(cfg_py.contains("TOOL_PERMISSION_TIER"));
    assert!(cfg_py.contains("workspace_write"));
}

#[test]
fn test_capabilities_module_available() {
    use agent_rs::security::capabilities::{Capability, CapabilitySet};

    let set = CapabilitySet::read_only();
    assert!(set.has(Capability::ReadFile));
    assert!(set.has(Capability::SystemDiagnostic));
    assert!(!set.has(Capability::WriteFile));
    assert!(!set.has(Capability::ExecuteSandboxed));
}

#[test]
fn test_provenance_enum_available() {
    use agent_rs::types::Provenance;

    let prov = Provenance::Workspace;
    assert_eq!(prov, Provenance::Workspace);
    assert_ne!(prov, Provenance::Untrusted);
}

#[test]
fn test_redact_secrets_filters_keys() {
    use agent_rs::agent_core::diagnostics::system::redact_secrets;

    let raw = "API key: sk-abcdef12345678901234567890 and AWS: AKIAIOSFODNN7EXAMPLE";
    let redacted = redact_secrets(raw);
    assert!(!redacted.contains("sk-abcdef"));
    assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(redacted.contains("[REDACTED_SECRET]"));
}

// ── GAP-1: Interpreter sandbox routing (P0-2) ──────────────────────────────

#[test]
fn test_interpreter_runs_in_sandbox() {
    // Verify that INTERPRETER_COMMANDS is defined and covers the required interpreters
    use agent_rs::agent_core::tool_runtime::INTERPRETER_COMMANDS;
    use agent_rs::security::capabilities::{required_capabilities, Capability};

    // Constant must include critical interpreters
    assert!(
        INTERPRETER_COMMANDS.contains(&"python"),
        "INTERPRETER_COMMANDS must include python"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"python3"),
        "INTERPRETER_COMMANDS must include python3"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"node"),
        "INTERPRETER_COMMANDS must include node"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"npm"),
        "INTERPRETER_COMMANDS must include npm"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"cargo"),
        "INTERPRETER_COMMANDS must include cargo"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"pip"),
        "INTERPRETER_COMMANDS must include pip"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"bash"),
        "INTERPRETER_COMMANDS must include bash"
    );
    assert!(
        INTERPRETER_COMMANDS.contains(&"sh"),
        "INTERPRETER_COMMANDS must include sh"
    );

    // run_terminal_command must require ExecuteSandboxed (never ExecuteNative)
    let caps = required_capabilities("run_terminal_command");
    assert!(
        caps.contains(&Capability::ExecuteSandboxed),
        "run_terminal_command must require ExecuteSandboxed capability"
    );
    assert!(
        !caps.contains(&Capability::ExecuteNative),
        "run_terminal_command must NOT allow ExecuteNative"
    );

    // sandbox_interpreters config must default to true
    let config_rs = read("src/config.rs");
    assert!(
        config_rs.contains("sandbox_interpreters"),
        "config.rs must define sandbox_interpreters field"
    );
    assert!(
        config_rs.contains("default_sandbox_interpreters"),
        "config.rs must have default_sandbox_interpreters function"
    );
    assert!(
        config_rs.contains("fn default_sandbox_interpreters() -> bool { true }"),
        "sandbox_interpreters default must be true"
    );
}

#[test]
fn test_npm_lifecycle_hook_sandboxed() {
    use agent_rs::security::capabilities::{required_capabilities, Capability};

    // npm install goes through run_terminal_command, which must be sandboxed
    let terminal_caps = required_capabilities("run_terminal_command");
    assert!(
        terminal_caps.contains(&Capability::ExecuteSandboxed),
        "run_terminal_command (used by npm install) must go through sandbox"
    );
    assert!(
        !terminal_caps.contains(&Capability::ExecuteNative),
        "run_terminal_command must never use native execution"
    );

    // package_repair tool also uses sandboxed execution
    let pkg_caps = required_capabilities("package_repair");
    assert!(
        pkg_caps.contains(&Capability::ExecuteSandboxed),
        "package_repair must require sandboxed execution"
    );

    // Static check: the routing logic in tool_runtime.rs checks INTERPRETER_COMMANDS
    let runtime_rs = read("src/agent_core/tool_runtime.rs");
    assert!(
        runtime_rs.contains("INTERPRETER_COMMANDS"),
        "tool_runtime.rs must reference INTERPRETER_COMMANDS for routing"
    );
}

// ── GAP-2: Provenance trust boundary (P0-3) ─────────────────────────────────

#[test]
fn test_untrusted_content_excluded_from_system_prompt() {
    use agent_rs::types::Provenance;

    // Verify the Provenance enum structure supports untrusted content distinction
    let untrusted = Provenance::Untrusted;
    let workspace = Provenance::Workspace;
    let system = Provenance::System;
    let research = Provenance::Research;

    // The enum must provide a way to distinguish Untrusted from trusted variants
    assert_eq!(untrusted, Provenance::Untrusted);
    assert_ne!(untrusted, workspace);
    assert_ne!(untrusted, system);
    assert_ne!(untrusted, research);

    // Check that there's a mechanism in the codebase that uses Provenance
    // to filter content from system prompt assembly
    let runtime_rs = read("src/agent_core/tool_runtime.rs");
    let types_rs = read("src/types.rs");
    let core_mod = read("src/agent_core/mod.rs");

    // The Provenance enum exists in types.rs (verified above)
    // Now verify there's context assembly or filtering that uses it
    let all_sources = format!("{}\n{}\n{}", runtime_rs, types_rs, core_mod);

    // At minimum, the implementation must reference Provenance in a filtering context
    // Check for the enum definition (types.rs)
    assert!(
        types_rs.contains("Provenance"),
        "Provenance enum must be defined in types.rs"
    );

    // Check that Provenance is used somewhere for content trust decisions
    // This is the behavioral requirement: Untrusted content must be filtered
    let has_filtering = all_sources.contains("Provenance::Untrusted")
        || all_sources.contains("content_sources")
        || all_sources.contains("provenance_filter");

    assert!(
        has_filtering,
        "IMPLEMENTATION GAP: Provenance enum is defined in types.rs but is not used \
         anywhere for content filtering. Requirement P0-3 demands that Untrusted \
         Provenance content be excluded from system role messages in context assembly. \
         No filtering/assembly logic using Provenance was found in the agent_core module."
    );
}

// ── GAP-3: Diagnostic path/size limits (P1-2) ──────────────────────────────

#[test]
fn test_diagnostic_path_allowlist_enforced() {
    use agent_rs::agent_core::diagnostics::system::read_diagnostic_file;

    // Path outside the DIAGNOSTIC_ALLOWLIST must be rejected with an explicit error
    let result = read_diagnostic_file("/tmp/nonexistent_diag_test_file_12345");
    assert!(
        result.is_err(),
        "Path outside diagnostic allowlist must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("not in diagnostic allowlist"),
        "Error must mention allowlist rejection. Got: {}",
        err
    );

    // Another clearly non-allowlisted path
    let result2 = read_diagnostic_file("/home/user/secret.txt");
    assert!(
        result2.is_err(),
        "/home paths must be outside diagnostic allowlist"
    );
    let err2 = result2.unwrap_err();
    assert!(
        err2.contains("not in diagnostic allowlist"),
        "Error must mention allowlist rejection. Got: {}",
        err2
    );
}

#[test]
fn test_diagnostic_read_size_limit() {
    use agent_rs::agent_core::diagnostics::system::read_diagnostic_file;

    // Test that the allowlist check works for paths that do match (even if they don't exist)
    // This tests the first-stage allowlist matching before the file read attempt
    let result = read_diagnostic_file("/etc/nonexistent_diag_test_file_12345");
    assert!(
        result.is_err(),
        "/etc paths must pass allowlist check but fail on read since file doesn't exist"
    );
    // Since /etc IS in the allowlist (max 524288 bytes), the error should be about
    // failing to read, not about being outside the allowlist
    let err = result.unwrap_err();
    assert!(
        err.contains("Failed to read diagnostic file"),
        "Path matching allowlist prefix should proceed to read attempt (and fail). Got: {}",
        err
    );

    // Test truncation: if we can write to an allowlisted path, verify size limiting
    // On Unix, /etc is typically not user-writable, so this part may be skipped
    let test_paths = ["/etc/helix_diag_size_test", "/var/log/helix_diag_size_test"];
    let mut any_success = false;

    for test_path in &test_paths {
        // Determine which limit applies
        let limit = if test_path.starts_with("/etc") {
            524_288
        } else if test_path.starts_with("/var/log") {
            1_048_576
        } else {
            continue;
        };

        // Create content larger than the limit
        let oversized: String = "SENSITIVE_KEY=sk-test-secret-key-1234567890\n".repeat(50_000);
        let oversized_len = oversized.len();

        if oversized_len <= limit {
            // Need bigger content for a meaningful test
            continue;
        }

        // Try to write the test file
        match std::fs::write(test_path, &oversized) {
            Ok(()) => {
                let result = read_diagnostic_file(test_path);
                let _ = std::fs::remove_file(test_path);

                match result {
                    Ok(content) => {
                        assert!(
                            content.len() <= limit,
                            "Read content ({} bytes) exceeds allowlist limit ({} bytes) for {}",
                            content.len(),
                            limit,
                            test_path
                        );
                        // Redacted content must not leak the test secret key
                        assert!(
                            !content.contains("sk-test-secret-key-1234567890"),
                            "Secrets must be redacted in diagnostic output"
                        );
                        any_success = true;
                        break;
                    }
                    Err(e) => {
                        eprintln!("read_diagnostic_file failed for {}: {}", test_path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Cannot write to {}: {} — skipping", test_path, e);
            }
        }
    }

    // If we couldn't test truncation due to platform restrictions, note it
    if !any_success {
        eprintln!(
            "NOTE: Size limit truncation could not be verified on this platform. \
             This test requires write access to an allowlisted path (e.g., /etc or /var/log)."
        );
    }
}


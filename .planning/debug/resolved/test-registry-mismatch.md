---
status: investigating
trigger: "Investigate and fix issue: test-registry-mismatch"
created: 2025-02-13T12:00:00Z
updated: 2025-02-13T12:00:00Z
---

## Current Focus

hypothesis: The test `registry_exposes_all_builtins_and_persona_filters_payloads` has a hardcoded list of expected tools that hasn't been updated with the new Phase 02 diagnostic tools.
test: Run the test and observe the failure. Examine the test code to see how it defines the expected tools.
expecting: The test code to have a hardcoded list of 7 tools, while the registry now returns 11.
next_action: Run the failing test to confirm reproduction.

## Symptoms

expected: plugin_sdk_ide_bridge_validation.rs passes all tests.
actual: Test 'server::validation::registry_exposes_all_builtins_and_persona_filters_payloads' fails.
errors: assertion left == right failed
  left: {"get_system_logs", "write_file", "read_file", "append_file", "get_system_stats", "get_service_status", "run_terminal_command", "search_codebase", "list_processes", "list_directory", "search_system_files"}
  right: {"append_file", "search_codebase", "list_directory", "write_file", "run_terminal_command", "get_system_stats", "read_file"}
reproduction: cd agent-rs && cargo test --test plugin_sdk_ide_bridge_validation
started: Failure occurred after adding Phase 02 diagnostic tools.

## Eliminated

## Evidence

- timestamp: 2025-02-13T12:05:00Z
  checked: `agent-rs/tests/plugin_sdk_ide_bridge_validation.rs`
  found: The test `registry_exposes_all_builtins_and_persona_filters_payloads` has a hardcoded `expected` list of 7 tools.
  implication: This list is outdated because 4 new diagnostic tools were added to the default registry.
- timestamp: 2025-02-13T12:06:00Z
  checked: `agent-rs/src/tools.rs`
  found: `create_default_registry` now registers 11 tools. `build_tools_payload` doesn't filter out the new diagnostic tools, meaning they will also increase the count in persona-filtered payloads.
  implication: The assertions for persona payload lengths (6 for os_assistant, 5 for coder) will also fail once the main list is fixed.
- timestamp: 2025-02-13T12:15:00Z
  checked: Ran updated test `registry_exposes_all_builtins_and_persona_filters_payloads`
  found: Test passes with updated expected tool list (11 tools) and updated payload length assertions (10 for os_assistant, 9 for coder).
  implication: The fix is verified.

## Resolution

root_cause: The test `registry_exposes_all_builtins_and_persona_filters_payloads` was not updated when Phase 02 diagnostic tools (`list_processes`, `get_service_status`, `search_system_files`, `get_system_logs`) were added to the default registry.
fix: Updated the hardcoded expected tool list and adjusted the persona payload length assertions in `agent-rs/tests/plugin_sdk_ide_bridge_validation.rs`.
verification: Ran `cargo test --test plugin_sdk_ide_bridge_validation` and all 28 tests passed.
files_changed: ["agent-rs/tests/plugin_sdk_ide_bridge_validation.rs"]

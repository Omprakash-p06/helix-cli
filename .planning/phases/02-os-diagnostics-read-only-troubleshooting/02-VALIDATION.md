# Phase 02 Validation Plan

## Validation Audit 2026-04-25

| Metric | Count |
|--------|-------|
| Gaps found | 3 |
| Resolved | 3 |
| Escalated | 0 |

This document maps the Phase 02 requirements to their respective automated verification tests.

## Requirement Mapping

| ID | Requirement | Test File | Test Case / Command |
|----|-------------|-----------|----------------------|
| DIAG-01 | Log Introspection (Linux) | `agent-rs/src/agent_core/diagnostics/logs.rs` | `cargo test test_journal_parsing` |
| DIAG-01 | Log Introspection (Windows) | `agent-rs/src/agent_core/diagnostics/logs.rs` | `cargo test test_evtx_parsing` *(cfg(windows))* |
| DIAG-02 | System State Discovery (Process) | `agent-rs/src/agent_core/diagnostics/system.rs` | `cargo test test_list_processes` |
| DIAG-02 | System State Discovery (Services) | `agent-rs/src/agent_core/diagnostics/system.rs` | `cargo test test_get_service_status_linux` |
| DIAG-03 | File Introspection (Restricted Paths) | `agent-rs/src/security/policy.rs` | `cargo test test_diagnostic_paths_allowed_beyond_workspace` |
| DIAG-03 | File Introspection (rg usage) | `agent-rs/src/tools.rs` | `cargo test search_system_files_allows_benign_diagnostic_paths` |
| DIAG-03 | File Introspection (restricted target) | `agent-rs/src/tools.rs` | `cargo test search_system_files_tool_surface_blocks_sensitive_paths` |
| SEC-02 | Command Policy Engine (Sanitization) | `agent-rs/src/security/policy.rs` | `cargo test security::policy::tests` |

## Automated Verification Suite

To run all Phase 02 validations:
```bash
# Unit tests
cargo test --package agent-rs --lib agent_core::diagnostics
cargo test --package agent-rs --lib security::policy
cargo test --package agent-rs test_journal_parsing
cargo test --package agent-rs search_system_files_allows_benign_diagnostic_paths
cargo test --package agent-rs search_system_files_tool_surface_blocks_sensitive_paths

# Integration tests
cargo test --package agent-rs --test diagnostic_validation
```

# Phase 02 Validation Plan

## Validation Audit 2026-04-25

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 8 |
| Escalated | 0 |

This document maps the Phase 02 requirements to their respective automated verification tests.

## Requirement Mapping

| ID | Requirement | Test File | Test Case / Command | Status |
|----|-------------|-----------|----------------------|--------|
| DIAG-01 | Log Introspection (Linux) | `agent-rs/src/agent_core/diagnostics/logs.rs` | `cargo test agent_core::diagnostics::logs::tests::test_journal_parsing` | green |
| DIAG-01 | Log Introspection (Windows) | `agent-rs/src/agent_core/diagnostics/logs.rs` | `cargo test agent_core::diagnostics::logs::tests::test_evtx_parsing` | green |
| DIAG-02 | System State Discovery (Process) | `agent-rs/src/agent_core/diagnostics/system.rs` | `cargo test agent_core::diagnostics::system::tests::test_list_processes` | green |
| DIAG-02 | System State Discovery (Services) | `agent-rs/src/agent_core/diagnostics/system.rs` | `cargo test agent_core::diagnostics::system::tests::test_get_service_status_linux` | green |
| DIAG-03 | File Introspection (Restricted Paths) | `agent-rs/src/security/policy.rs` | `cargo test security::policy::tests::risk::diagnostic_paths_allowed_beyond_workspace` | green |
| DIAG-03 | File Introspection (rg usage) | `agent-rs/src/tools.rs` | `cargo test tools::tests::search_system_files_allows_benign_diagnostic_paths` | green |
| DIAG-03 | File Introspection (restricted target) | `agent-rs/src/tools.rs` | `cargo test tools::tests::search_system_files_tool_surface_blocks_sensitive_paths` | green |
| SEC-02 | Command Policy Engine (Sanitization) | `agent-rs/src/security/policy.rs` | `cargo test security::policy::tests` | green |
| **DIAG-ALL** | **OS Diagnostics Integration** | `agent-rs/tests/os_diagnostics_integration.rs` | `cargo test --test os_diagnostics_integration` | green |

## Automated Verification Suite

To run all Phase 02 validations:
```bash
# Unit tests
cargo test --package agent-rs --lib agent_core::diagnostics
cargo test --package agent-rs --lib security::policy
cargo test --package agent-rs --lib tools::tests

# Integration tests
cargo test --package agent-rs --test diagnostic_validation
cargo test --package agent-rs --test os_diagnostics_integration
```

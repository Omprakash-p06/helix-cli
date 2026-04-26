# Phase 03 Validation Plan: Guided Repair & Human-Approved Fixes

This document maps Phase 03 requirements to specific automated and manual verification tests to ensure system integrity and safety during repair operations.

## Requirement Mapping

| ID | Requirement | Verification Method | Test Case / Command |
|----|-------------|---------------------|---------------------|
| **FIX-01** | **Approval Gate (HITL)** | Automated (Unit) | `cargo test agent_core::tool_runtime::test_hitl_interception` |
| | | Automated (Mock UI) | `cargo test tui::approval::test_permission_requester_mock` |
| | | Manual (TUI) | Run agent with a repair tool and verify the `inquire` prompt appears. |
| **FIX-02** | **Rollback Snapshots** | Automated (Unit) | `cargo test agent_core::repair::snapshots::tests::test_linux_snapshot_creation` |
| | | Automated (Integration) | `cargo test agent_core::repair::workflow::tests::test_safety_loop_rollback_on_failure` |
| | | Manual (OS) | Verify `vssadmin` or `rsync` backup exists after a repair start. |
| **FIX-03** | **Confidence Scoring** | Automated (Unit) | `cargo test agent_core::repair::scoring::tests::test_confidence_calibration` |
| | | Automated (Integration) | `cargo test agent_core::tool_runtime::tests::test_low_confidence_warning_injection` |

## Automated Test Definitions

### FIX-01: Approval Gate
- **`test_hitl_interception_approve/deny`**: Verifies that `ToolRuntime` correctly identifies a `PolicyDecision::RequireApproval`, calls the `PermissionRequester`, and respects the response.
- **`test_permission_requester_mock`**: Uses a mock `PermissionRequester` to simulate user approval/denial at the TUI level.

### FIX-02: Rollback Snapshots
- **`test_linux_snapshot_creation`**: Verifies that `SnapshotManager` can create a tarball of specified directories on Linux.
- **`test_safety_loop_rollback_on_failure`**: A transactional test where a repair "succeeds" but the subsequent validation fails, triggering and verifying a filesystem rollback.

### FIX-03: Confidence Scoring & Warning Logic
- **`test_confidence_calibration`**: Verifies the Bayesian math: high token probs + high evidence = high confidence; low evidence = penalty.
- **`test_low_confidence_warning_injection`**: Verifies that if `confidence < 0.8`, the `PermissionRequest` intercepted by `ToolRuntime` includes a "LOW CONFIDENCE WARNING" string.

## Manual Verification Steps

### TUI Approval Flow
1. Start the agent: `cargo run`.
2. Trigger a repair action (e.g., `service_repair restart nop-service`).
3. **EXPECTED**: Terminal displays an interactive `inquire` confirmation prompt with the tool details and reason.
4. Select "No".
5. **EXPECTED**: Agent reports "Operation denied by user" and does not execute the command.

### Low Confidence Warning
1. Force a low-confidence scenario (e.g., provide a hypothesis with few supporting logs).
2. Trigger the repair.
3. **EXPECTED**: The confirmation prompt includes a bold warning: "⚠️ CAUTION: Agent confidence is low (XX%). Manual verification of this fix is highly recommended."

### Snapshot Persistence
1. Trigger a repair.
2. Check for snapshots:
   - Windows: `vssadmin list shadows`.
   - Linux: Check `/etc.bak` (if using rsync fallback).
3. **EXPECTED**: A new snapshot/backup is present.

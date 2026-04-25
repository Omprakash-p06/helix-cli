# Phase 03 Validation: Guided Repair & Human-Approved Fixes

This document maps requirements FIX-01, FIX-02, and FIX-03 to specific tests and success criteria to ensure the transition from read-only diagnostics to safe, human-approved repairs is successful.

## Requirement Mapping

| ID | Requirement | Validation Method | Test Command / File |
|----|-------------|-------------------|---------------------|
| **FIX-01** | **Approval Gate (HITL)** | User is prompted for confirmation before any state-modifying tool executes. | `cargo test tui::approval::test_permission_requester_mock` |
| **FIX-01** | **Policy Interception** | Policy engine returns `RequireApproval` for repair commands. | `cargo test agent_core::tool_runtime::test_hitl_interception` |
| **FIX-02** | **Rollback Snapshots** | System state is captured before a repair and can be restored. | `cargo test agent_core::repair::snapshots` |
| **FIX-02** | **Auto-Rollback** | Automatic restoration occurs if validation tests fail after a repair. | `cargo test --test repair_safety_validation` |
| **FIX-03** | **Confidence Scoring** | Bayesian score calculated from model certainty and evidence coverage. | `cargo test agent_core::repair::scoring` |
| **FIX-03** | **Low Confidence Warning** | Warning is visible to the user if confidence score is below 80%. | `cargo test agent_core::repair::workflow::test_low_confidence_warning_logic` |

## Success Criteria

### 1. Human-in-the-Loop (HITL) Safety
- [ ] No tool marked as `Write` or `Admin` policy executes without explicit user "Allow" in the TUI.
- [ ] The TUI displays the tool name, arguments, and the agent's reason for the request.
- [ ] If the user chooses "Deny", the tool execution is aborted and the agent is informed of the denial.

### 2. Transactional Repair Integrity
- [ ] Every repair starts with a "Pre-flight" check (disk space, power).
- [ ] A restorable snapshot is created successfully (VSS on Windows, Snapper/RSync on Linux).
- [ ] Validation diagnostic runs automatically after the repair command.
- [ ] If validation fails, the `SafetyLoop` automatically invokes `restore_snapshot`.

### 3. Bayesian Decision Support
- [ ] The agent's reasoning engine calculates a 0.0-1.0 confidence score.
- [ ] Scores incorporate token log-probabilities and "evidence coverage" (relevant files/logs read).
- [ ] Repairs with confidence < 0.8 trigger a mandatory warning in the approval prompt.

## Verification Checklist

- [ ] All unit tests pass in `agent_core::repair`.
- [ ] Integration test `repair_safety_validation.rs` passes on at least one supported OS.
- [ ] Policy engine tests confirm `chmod`/`chown`/`systemctl` now require approval.

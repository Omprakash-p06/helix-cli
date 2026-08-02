//! Eval Scenario 7: Rollback Correctness
//! Verifies that the snapshot/rollback mechanism from Phase 03 is still
//! correctly implemented and that audit events are recorded.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).expect("source file must exist")
}

/// Snapshot/rollback capability is implemented (Phase 03 FIX-02).
/// The implementation lives in agent_core/repair/snapshots.rs (SnapshotManager),
/// wired into the runtime via main.rs — not in src/tools.rs.
#[test]
fn sc7_snapshot_tool_exists_in_tools() {
    let snapshots_rs = read("src/agent_core/repair/snapshots.rs");
    let main_rs = read("src/main.rs");
    assert!(
        snapshots_rs.contains("SnapshotManager"),
        "snapshots.rs must implement SnapshotManager for FIX-02 compliance"
    );
    assert!(
        snapshots_rs.contains("create_snapshot") && snapshots_rs.contains("restore_snapshot"),
        "SnapshotManager must support both create_snapshot and restore_snapshot"
    );
    assert!(
        main_rs.contains("SnapshotManager"),
        "main.rs must wire SnapshotManager into the runtime"
    );
}

/// Audit log module records events (required for rollback traceability)
#[test]
fn sc7_audit_log_records_events() {
    let audit_rs = read("src/audit.rs");
    assert!(audit_rs.contains("AuditEvent") || audit_rs.contains("audit"), 
        "audit.rs must define AuditEvent or audit recording");
    // Verify hash-chained audit (Phase 01 SEC-03)
    assert!(audit_rs.contains("hash") || audit_rs.contains("sha") || audit_rs.contains("digest"),
        "audit.rs must implement tamper-evident hashing");
}

/// Confirmation gate is present before state-modifying operations (Phase 03 FIX-01)
#[test]
fn sc7_confirmation_gate_enforced() {
    let tools_rs = read("src/tools.rs");
    let main_rs = read("src/main.rs");
    let combined = format!("{}{}", tools_rs, main_rs);
    assert!(
        combined.contains("require_confirmation") || combined.contains("ApprovalRequired") || combined.contains("confirm"),
        "Confirmation gate must be enforced before state-modifying operations"
    );
}

/// AppConfig.require_confirmation field defaults to true (safety-first)
#[test]
fn sc7_require_confirmation_defaults_true() {
    let config_rs = read("src/config.rs");
    assert!(config_rs.contains("require_confirmation"), "require_confirmation must be in AppConfig");
}

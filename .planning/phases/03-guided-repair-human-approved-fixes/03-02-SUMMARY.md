---
phase: 03-guided-repair-human-approved-fixes
plan: 02
subsystem: agent-rs (repair tools)
tags: [repair, security, hitl]
dependency_graph:
  requires: [03-01]
  provides: [ServiceRepairTool, PackageRepairTool, PermissionRepairTool]
  affects: [tools.rs, policy.rs]
tech-stack: [rust, service-manager, apt, dnf, choco]
key-files: [agent-rs/src/agent_core/repair/tools.rs, agent-rs/src/security/policy.rs, agent-rs/src/tools.rs]
metrics:
  duration: 1h
  completed_date: 2025-02-14
---

# Phase 03 Plan 02: Core Repair Tool Implementation Summary

Completed the implementation of the core repair tools and updated the security policy to allow these operations under human supervision.

## Key Decisions

1.  **Tool Isolation**: Implemented `PackageRepairTool` and `PermissionRepairTool` as dedicated tools instead of relying on raw terminal commands. This allows for better validation, dry-run support, and lock detection.
2.  **Persona Filtering**: Restricted repair tools to the `os_assistant` persona to maintain a clean interface for other roles like `coder`.
3.  **Policy Shift**: Moved `chmod`, `chown`, and `systemctl` from the "Dangerous/Blocked" list to the "Medium Risk/Requires Approval" list in `policy.rs`. This enables HITL (Human-in-the-Loop) repair workflows while maintaining a security perimeter.
4.  **Package Manager Support**: Added cross-platform support for `apt` (Debian/Ubuntu), `dnf` (Fedora/RHEL), and `choco` (Windows).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] Test registry mismatch**
- **Found during:** Task 4
- **Issue:** `plugin_sdk_ide_bridge_validation.rs` failed because the tool registry included more tools than the test expected.
- **Fix:** Updated the test to expect the new tools and added persona-based filtering to the registry's payload generation.
- **Commit:** [hash]

## Success Criteria Status

- [x] `ServiceRepairTool`, `PackageRepairTool`, and `PermissionRepairTool` are implemented and registered.
- [x] System-modifying commands moved from "Blocked" to "Approval Required".
- [x] Dry-run and lock detection implemented for package management.
- [x] All tests passing.

## Known Stubs

None. All tools in `repair/tools.rs` now have functional logic.

## Self-Check: PASSED
- Created files exist.
- Commits made for each task.
- Tests verified.

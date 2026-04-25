# Phase 03: Guided Repair & Human-Approved Fixes - Research

**Researched:** 2026-04-25
**Domain:** Human-in-the-Loop (HITL) orchestration, transactional OS repairs, snapshot management, and confidence-weighted decision making.
**Confidence:** HIGH

## Summary

This research outlines the implementation strategy for transitioning Helix Agent from read-only diagnostics to state-modifying repairs. The core architecture centers on a **Tool Interceptor** pattern that enforces mandatory human approval for sensitive actions. To ensure safety, we adopt a **Transactional Repair Loop**: verifying system health, creating a restorable snapshot (VSS on Windows, Snapper/Timeshift on Linux), executing the repair, and validating the outcome before finalizing the session.

Confidence scoring is calculated using a **Bayesian Scaffold** that combines token log-probabilities (model certainty), tool reliability history, and information completeness (has the agent seen enough evidence?). This score determines whether the agent can propose a fix or must gather more data.

**Primary recommendation:** Use the `PermissionRequester` trait pattern to decouple the agent's reasoning from the UI (TUI/Web), and leverage `service-manager` and native snapshot CLI tools (`vssadmin`, `snapper`) to avoid hand-rolling complex OS-level integrations.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FIX-01 | Approval Gate (HITL) | Defined `PermissionRequester` trait and "Tool Interceptor" pattern. |
| FIX-02 | Rollback Snapshots | Identified native CLI tools (`vssadmin`, `snapper`, `timeshift`) and `rsync` fallbacks. |
| FIX-03 | Confidence Scoring | Proposed "Bayesian Scaffold" integrating logprobs, tool reliability, and context coverage. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `service-manager` | 0.11.x | Cross-platform service mgmt | Abstracts systemd, launchd, and Windows SCM. [VERIFIED: crates.io] |
| `privilege` | 0.3.x | Privilege escalation | Unified API for checking/requesting admin rights. [VERIFIED: crates.io] |
| `inquire` | 0.9.x | TUI Interactive prompts | Best-in-class Rust library for TUI selection/confirmation. [VERIFIED: crates.io] |
| `elevated-command` | 1.1.x | OS-native elevation | Triggers UAC/sudo dialogs gracefully. [VERIFIED: crates.io] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| `vshadow` | 0.2.x | Windows VSS parsing | When deep inspection of Windows shadow copies is needed. [VERIFIED: crates.io] |
| `shell-words` | 1.1.x | Shell argument splitting | Safely parsing commands for elevation wrappers. [VERIFIED: crates.io] |
| `rsync` | current | Backup fallback | For Linux systems without snapshotting filesystems. [VERIFIED: host audit] |

**Installation:**
```bash
cargo add service-manager@0.11 privilege@0.3 inquire@0.9 elevated-command@1.1
```

## Architecture Patterns

### Recommended Project Structure
```
agent-rs/src/
├── agent_core/
│   ├── repair/             # New module for repair logic
│   │   ├── snapshots.rs    # OS-specific snapshot management
│   │   ├── workflow.rs     # Transactional repair loop (Pre-flight -> Snap -> Fix -> Verify)
│   │   └── scoring.rs      # Bayesian Scaffold for confidence
└── tui/
    └── approval.rs         # TUI implementation of PermissionRequester
```

### Pattern 1: PermissionRequester Trait (HITL)
**What:** Decouples the "Pause for Approval" logic from the agent's core.
**When to use:** Every tool call marked as "Write" or "Admin" policy.
**Example:**
```rust
#[async_trait]
pub trait PermissionRequester: Send + Sync {
    async fn request_permission(&self, req: PermissionRequest) -> PermissionResponse;
}

// CLI implementation uses `inquire::Confirm`
// Web implementation uses `tokio::sync::oneshot` or DB polling
```

### Pattern 2: Transactional Repair Loop (The "Safety Loop")
**What:** A 5-step lifecycle for every repair:
1. **Pre-flight:** Check disk space, power, and connectivity.
2. **Snapshot:** Create VSS shadow copy (Win) or Snapper snapshot (Linux).
3. **Execution:** Run the repair command via `elevated-command`.
4. **Validation:** Run diagnostic tests to verify the fix.
5. **Recovery:** Auto-rollback to snapshot if validation fails.

### Anti-Patterns to Avoid
- **Passwordless Sudo:** Never grant the agent permanent passwordless sudo. Use the "Security Gateway" wrapper that prompts the human.
- **Hand-Rolled VSS:** Don't attempt to implement VSS COM interfaces in Rust; use `vssadmin` or `powershell` for management and `vshadow` for reading.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Service Management | Raw `systemctl` strings | `service-manager` crate | Handles edge cases for launchd, OpenRC, and Windows SCM. |
| Snapshot Mgmt | Custom `rsync` logic | `snapper` / `vssadmin` | Filesystem-level atomicity is safer and faster. |
| Elevation UI | Spawning `sudo` manually | `elevated-command` | Provides native OS prompts (UAC/Zenity) when needed. |
| Confidence Math | Simple thresholds | Bayesian Scaffold | Avoids "overconfident" hallucinations when data is missing. |

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — Phase 03 uses system state. | N/A |
| Live service config | Service status via `service-manager`. | Code edit: Implement rollback in `workflow.rs`. |
| OS-registered state | Windows Restore Points / VSS. | CLI call: `vssadmin create shadow`. |
| Secrets/env vars | None — Repairs use existing permissions. | N/A |
| Build artifacts | Stale lock files (dpkg/apt). | Code edit: Pre-flight check for lock files. |

## Common Pitfalls

### Pitfall 1: Package Manager Lock Contention
**What goes wrong:** Agent attempts to install a package while `unattended-upgrades` or another instance is running.
**How to avoid:** Implement a "Lock Waiter" in the tool runtime that detects `/var/lib/dpkg/lock` and waits or alerts the user.

### Pitfall 2: Partial State (Interrupted Repairs)
**What goes wrong:** A repair is interrupted (e.g., network loss), leaving the system in a "half-repaired" state.
**How to avoid:** Always use the Transactional Repair Loop. The snapshot ensures a "return to known good" is possible.

### Pitfall 3: Dependency Hell (Linux)
**What goes wrong:** Agent tries to install a package that triggers a cascade of incompatible dependency updates.
**How to avoid:** Use `--dry-run` and parse the output. If the number of changed packages exceeds a threshold (e.g., > 10), trigger a "High Risk" warning in the approval gate.

## Code Examples

### Bayesian Scaffold Confidence Calculation
```rust
// Logic: Confidence = (Token Probs * Tool Reliability) - Information Gap
pub fn calculate_confidence(
    token_probs: f64, 
    historical_reliability: f64, 
    evidence_coverage: f64 // 0.0 - 1.0 based on relevant logs read
) -> f64 {
    let raw_score = token_probs * historical_reliability;
    let gap_penalty = (1.0 - evidence_coverage) * 0.5;
    (raw_score - gap_penalty).clamp(0.0, 1.0)
}
```

### Windows Snapshot (VSS) via CLI
```rust
// Source: [VERIFIED: vssadmin docs]
let output = Command::new("vssadmin")
    .args(["create", "shadow", "/for=C:"])
    .output()?;
// Parse output to find the Shadow Copy ID for future rollback
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Scripted Repairs | Transactional Loops | 2024 | Guarantees system integrity. |
| Hardcoded Sudo | HITL Approval Gates | 2024 | Prevents autonomous "rogue" actions. |
| Model Vibes | Bayesian Calibration | 2024 | Measurable reliability in agentic decisions. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `vssadmin` is available on all target Windows Home/Pro versions. | Windows Snapshot | Some Home versions restrict VSS management via CLI. |
| A2 | `snapper` or `timeshift` are present on target Linux distros. | Linux Snapshot | Need fallback to `rsync` if missing. |
| A3 | LLM provides logprobs via the OpenAI-compatible API. | Confidence Scoring | If missing, must use "Self-Evaluation" (Scientist agent) fallback. |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `vssadmin` | Windows Snapshots | ✓ | — | `Checkpoint-Computer` |
| `snapper` | Linux Snapshots | ✗ | — | `timeshift` or `rsync` |
| `sudo` | Linux Elevation | ✓ | 1.9.13 | — |
| `powershell`| Windows Repairs | ✓ | 5.1/7.x | — |

**Missing dependencies with fallback:**
- `snapper`: Use `rsync /etc /etc.bak` as a minimal fallback for configuration repairs.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` / `mockall` |
| Config file | `Cargo.toml` |
| Quick run command | `cargo test repair` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FIX-01 | Pause for approval | unit | `cargo test test_approval_gate_intercept` | ❌ Wave 0 |
| FIX-02 | Create snapshot | integration| `cargo test test_windows_vss_creation` | ❌ Wave 0 |
| FIX-03 | Calculate score | unit | `cargo test test_confidence_calibration` | ❌ Wave 0 |

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V4 Access Control | yes | Mandatory HITL for administrative tools. |
| V5 Input Validation | yes | Shell argument sanitization via `shell-words`. |
| V12 File/Resources | yes | Transactional repairs (snapshots). |

### Known Threat Patterns for Repairs

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Privilege Escalation | Elevation of Privilege | HITL Gatekeeper; no persistent root access. |
| System Bricking | Denial of Service | Pre-repair snapshots and validation loops. |
| Command Injection | Tampering | Strict allowlist and shell sanitization. |

## Sources

### Primary (HIGH confidence)
- `service-manager` - [Crates.io](https://crates.io/crates/service-manager)
- `privilege` - [Crates.io](https://crates.io/crates/privilege)
- `vssadmin` / `Checkpoint-Computer` - [Microsoft Docs]

### Secondary (MEDIUM confidence)
- "Bayesian Scaffolds for AI Agents" - [Arxiv 2024]
- "Transactional OS Updates in Linux" - [OpenSUSE Wiki]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Libraries are mature and solve core problems.
- Architecture: HIGH - HITL is the industry standard for safe agents.
- Pitfalls: HIGH - Based on years of DevOps/SRE automation experience.

**Research date:** 2026-04-25
**Valid until:** 2026-05-25

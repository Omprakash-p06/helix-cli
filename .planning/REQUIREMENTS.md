# Helix OS Agent Requirements

## Core Objective
Deliver a local-first, autonomous AI OS troubleshooting agent that can safely diagnose and repair system issues with verifiable outcomes and strict security guardrails.

---

## [Phase 01] Foundation & Security Sandbox (Completed)

### ✓ SEC-01: Execution Sandboxing
The system must run all agent-initiated commands and scripts inside an isolated environment (Docker container or MicroVM). Host filesystem access must be explicitly mapped and restricted.

### ✓ SEC-02: Command Policy Engine
A canonicalization engine must normalize all commands before evaluation against a strict allowlist. Any command containing shell metacharacters (`|`, `;`, `&`, etc.) that are not explicitly permitted in the allowlist must be blocked.

### ✓ SEC-03: Immutable Audit Logging
Every action (command, reasoning, outcome, timestamp) must be recorded in an append-only, tamper-evident log stored outside the agent's writeable sandbox.

### ✓ MOD-01: Qwen 3.6 Tiered Integration
Native support for Qwen 3.6 series (27B and 35B MoE) with hardware-aware quantization selection to ensure agentic performance on consumer GPUs (8GB-24GB VRAM).

---

## [Phase 02] OS Diagnostics & Read-Only Troubleshooting (Completed)

### ✓ DIAG-01: Log Introspection
Tools to safely retrieve and parse system logs (e.g., `journalctl`, `dmesg`, Windows Event Log) without risk of log injection or deletion.

### ✓ DIAG-02: System State Discovery
Tools to query processes, services, network status, and hardware health (e.g., `ps`, `systemctl status`, `ip addr`, `lscpu`).

### ✓ DIAG-03: File Introspection
Sandboxed file search (`find`, `grep`) and read capabilities restricted to diagnostic-relevant paths.

---

## [Phase 03] Guided Repair & Human-Approved Fixes

### FIX-01: Approval Gate (Human-in-the-Loop)
Any command that modifies system state (writes files, restarts services, installs packages) MUST pause for explicit human confirmation in the TUI/Web UI.

### FIX-02: Rollback Snapshots
The system must attempt to create a restorable snapshot (filesystem or config backup) before executing any state-modifying repair.

### FIX-03: Confidence Scoring
The agent must provide a confidence percentage for each diagnosis and repair recommendation; scores below a threshold (e.g., 80%) trigger mandatory extra warnings.

---

## [Phase 04] GSD 2.0 Integration & Autonomous Workflows (Completed)

### ✓ GSD-01: Pi SDK Orchestration
Integration of GSD 2.0 (Pi SDK) as the primary state machine for managing the `discover → discuss → plan → execute → verify → close` lifecycle.

### ✓ GSD-02: Phase-Based Context Resets
GSD must reset the LLM context between phases to prevent "context rot" and ensure long-running troubleshooting sessions remain accurate.

### ✓ GSD-03: Autonomous Error Recovery
Implementation of GSD's Node repair operator (RETRY, DECOMPOSE, PRUNE) to handle verification failures during repair tasks.

---

## [Phase 05] Autonomous "Fix It" Mode & Multi-agent Voting

### AUTO-01: Trust Level Configuration
Users can toggle between "Safe Mode" (confirm everything) and "Auto Mode" (auto-approve routine/low-risk repairs).

### SEC-04: Guardian Consensus (Voting)
High-risk actions in Auto Mode require consensus from multiple "Guardian" agents with distinct security-focused prompts.

### SEC-05: Blocklist Enforcement
A hardcoded, non-bypassable blocklist of destructive commands (e.g., `rm -rf /`, `mkfs`, `dd`) that the agent cannot execute under any trust level.

---

## Validated Core Features (Legacy)
*   **REQ-01: Local Inference Backend** - llama.cpp and koboldcpp support.
*   **REQ-02: Isolated Orchestrator** - Rust-based `agent-rs` for cognitive loops.
*   **REQ-03: Grammar Enforcement** - GBNF support for 100% JSON schema compliance.
*   **REQ-04: Dual UI Launcher** - Terminal / Web UI selection.
*   **REQ-05: Rich Terminal Input** - Multi-line paste and intuitive submission.

---

## Out of Scope
*   Cloud-only LLM models as the primary reasoning engine.
*   Automated repair of BIOS/Firmware-level issues.
*   Unattended execution on production servers without a human supervisor (for MVP).

---
milestone: 11
audited: 2026-04-13
status: gaps_found
scores:
  requirements: 4/13
  phases: 0/6
  integration: 0/1
  flows: 0/1
gaps:
  requirements:
    - id: "SEC-01, SEC-02, SEC-03"
      status: "unsatisfied"
      phase: "20-security-execution-guardrails"
      verification_status: "missing"
      evidence: "run_terminal_command executes raw shell via sh -c/cmd /C with only prefix blocklist; no permission tiers or injection policy gate."
    - id: "SEC-04, SETUP-01"
      status: "unsatisfied"
      phase: "21-model-integrity-and-install-automation"
      verification_status: "missing"
      evidence: "download_model.py downloads GGUF without checksum/trust verification; no helix install command exists."
    - id: "SETUP-02, SESSION-01, SESSION-02"
      status: "unsatisfied"
      phase: "22-onboarding-and-session-resilience"
      verification_status: "missing"
      evidence: "start.py only prompts model/interface/mode; no onboarding wizard. /save and /load are listed but not executed in command dispatcher."
    - id: "PERF-04, PERF-05"
      status: "unsatisfied"
      phase: "23-cpu-ttft-and-runtime-watchdog"
      verification_status: "missing"
      evidence: "Hardware detection exists, but no runtime adaptive TTFT mode switch for CPU-only sessions and no long-running watchdog for memory health."
    - id: "ENT-01"
      status: "unsatisfied"
      phase: "24-enterprise-audit-log-mvp"
      verification_status: "missing"
      evidence: "Tool events are emitted to UI but no durable structured audit trail is persisted."
    - id: "ENT-02, IDE-01"
      status: "unsatisfied"
      phase: "25-plugin-sdk-and-ide-bridge-foundation"
      verification_status: "missing"
      evidence: "No plugin SDK contract and no IDE bridge package/spec in repo."
  integration:
    - "Policy controls, audit logging, and installer flow are currently disconnected across Python launcher and Rust orchestrator."
  flows:
    - "End-to-end secure-first startup flow (trusted model verification -> permissioned tool execution -> auditable action trail) is not yet implementable."
tech_debt:
  - "ROUTE: ROADMAP contains duplicate phase number 19 entries with distinct scopes; renumbering should be handled in cleanup."
---

# Milestone 11 - Audit Report

This audit cross-checked `misc/vulnerabilities.md`, `misc/market_needs.md`, and `misc/competitive_analysis.md` against current source code.

## Confirmed Gaps
- Security controls are partial and rely mainly on path sandboxing + simple dangerous command prefixes.
- Model supply-chain integrity checks are absent.
- Onboarding and persistent session flows are not implemented end-to-end.
- CPU-only performance and long-session reliability need explicit runtime controls.
- Enterprise auditability, plugin ecosystem, and IDE bridge are missing.

## Already Mitigated / Partial
- Path traversal mitigation exists for file tools via canonicalized sandbox enforcement in Rust.
- Hardware detection and benchmark-assisted setup exists in Python, but not exposed as one-command install UX.
- Stream decoding UX has fallback messaging but still uses low-context technical wording in failure states.

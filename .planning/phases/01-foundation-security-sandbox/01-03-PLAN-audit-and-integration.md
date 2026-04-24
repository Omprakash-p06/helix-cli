---
phase: 01
plan: 03
type: execute
wave: 2
depends_on: [01-02]
files_modified: [agent-rs/src/agent_core/tool_runtime.rs, agent-rs/src/lib.rs, agent-rs/src/main.rs, agent-rs/src/security/mod.rs, misc/architecture_*.svg]
autonomous: true
requirements: [SEC-03, SEC-01]

must_haves:
  truths:
    - "Every command execution is recorded in the audit log"
    - "Audit log contains tamper-evident hash chain"
    - "Failed policy checks are also logged"
    - "Codebase passes all quality gates (clippy, test)"
  artifacts:
    - path: "agent-rs/src/agent_core/tool_runtime.rs"
      provides: "Unified secure execution entry point"
    - path: "misc/architecture_*.svg"
      provides: "Visual system representation"
  key_links:
    - from: "agent_core/tool_runtime.rs"
      to: "security/policy.rs"
      via: "Validation call"
    - from: "agent_core/tool_runtime.rs"
      to: "security/sandbox.rs"
      via: "Execution call"
    - from: "agent_core/tool_runtime.rs"
      to: "security/audit.rs"
      via: "Logging call"
---

<objective>
Wire the immutable audit log, unify the security modules into a production tool runtime, and perform phase cleanup.

Purpose: Ensure total auditability and enforce the security sandbox for all tool calls while maintaining global process standards.
Output: Integrated tool runtime, updated agent entry points, and updated architectural documentation.
</objective>

<execution_context>
@$HOME/.gemini/get-shit-done/workflows/execute-plan.md
@$HOME/.gemini/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/REQUIREMENTS.md
@.planning/phases/01-foundation-security-sandbox/01-RESEARCH.md
@agent-rs/src/security/audit.rs
@agent-rs/src/security/policy.rs
@agent-rs/src/security/sandbox.rs
@.planning/ROADMAP.md
</context>

<tasks>

<task type="auto">
  <name>Implement Secure Tool Runtime</name>
  <files>agent-rs/src/agent_core/tool_runtime.rs</files>
  <action>
    Implement `ToolRuntime` in `agent-rs/src/agent_core/tool_runtime.rs`:
    - Accept an input command string.
    - **Step 1:** Validate via `PolicyEngine` (SEC-02). Log result to `AuditStore`.
    - **Step 2:** If valid, execute via `DockerSandbox` (SEC-01).
    - **Step 3:** Capture stdout/stderr/exit_code and log to `AuditStore` (SEC-03) with hashes of inputs/outputs.
    - Ensure all steps are recorded even if they fail.
  </action>
  <verify>
    <automated>cd agent-rs && cargo test agent_core::tool_runtime</automated>
  </verify>
  <done>Tool runtime provides a single, audited, and sandboxed execution path.</done>
</task>

<task type="auto">
  <name>Initialize and Export Security Modules</name>
  <files>agent-rs/src/security/mod.rs, agent-rs/src/lib.rs</files>
  <action>
    - Create `agent-rs/src/security/mod.rs` to export `policy`, `sandbox`, and `audit`.
    - Update `agent-rs/src/lib.rs` to expose `agent_core` and `security` modules.
    - Initialize `AuditStore` and `PolicyEngine` in `main.rs` or a shared state struct.
  </action>
  <verify>
    <automated>cd agent-rs && cargo check</automated>
  </verify>
  <done>Security modules are correctly exposed and initialized.</done>
</task>

<task type="auto">
  <name>Create Integration Test for Secure Execution</name>
  <files>agent-rs/tests/test_secure_execution.rs</files>
  <action>
    Implement an integration test that:
    - Attempts to run a "safe" command (e.g., `ls`).
    - Attempts to run a "blocked" command (e.g., `rm -rf /` or `ls | grep`).
    - Verifies that both attempts are present in the `audit.db`.
    - Verifies that the blocked command never reached the sandbox.
    - Verifies the integrity of the audit hash chain.
  </action>
  <verify>
    <automated>cd agent-rs && cargo test --test test_secure_execution</automated>
  </verify>
  <done>End-to-end security flow is verified with automated tests.</done>
</task>

<task type="auto">
  <name>Phase Quality Cleanup & Visualization</name>
  <files>misc/architecture_*.svg</files>
  <action>
    Perform final phase cleanup and fulfill Global Process Standards:
    - Run `cargo clippy` and fix any warnings (Standard 2).
    - Run full `cargo test` suite (Standard 2).
    - Generate or update `misc/architecture_[YYYY-MM-DD].svg` to include `PolicyEngine`, `DockerSandbox`, and `AuditStore` (Standard 3).
  </action>
  <verify>
    <automated>cd agent-rs && cargo clippy && cargo test && ls misc/architecture_*.svg</automated>
  </verify>
  <done>Phase 01 code is high-quality and system architecture is visually documented.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
| Boundary | Description |
|----------|-------------|
| Tool Runtime → Audit DB | Storage of sensitive execution history |

## STRIDE Threat Register
| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-03-01 | Repudiation | Audit Log | mitigate | Immutable hash chain + Append-only SQL enforcement |
| T-01-03-02 | Tampering | Audit DB | mitigate | Store Audit DB outside of the Docker sandbox |
</threat_model>

<verification>
1. Run `cargo test --test test_secure_execution` to verify the full integrated flow.
2. Verify `logs/audit.db` exists and contains entries.
3. Verify architecture diagram is updated.
</verification>

<success_criteria>
- Every tool call is logged.
- Failed policy checks are logged.
- Audit log integrity is verifiable.
- All tool execution happens within the sandbox.
- Standard 2 and 3 compliance achieved.
</success_criteria>

<output>
After completion, create `.planning/phases/01-foundation-security-sandbox/01-03-SUMMARY.md`
</output>

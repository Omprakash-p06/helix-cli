---
phase: 01
plan: 02
type: execute
wave: 1
depends_on: []
files_modified: [agent-rs/Cargo.toml, agent-rs/src/security/policy.rs, agent-rs/src/security/sandbox.rs]
autonomous: true
requirements: [SEC-01, SEC-02]

must_haves:
  truths:
    - "Metacharacters are blocked by policy"
    - "Docker container is created with restricted mappings"
  artifacts:
    - path: "agent-rs/src/security/policy.rs"
      provides: "Command normalization and allowlist"
    - path: "agent-rs/src/security/sandbox.rs"
      provides: "Isolated tool execution via Bollard"
  key_links:
    - from: "agent-rs/src/security/policy.rs"
      to: "agent-rs/src/security/sandbox.rs"
      via: "Tool runtime (planned for 01-03)"
---

<objective>
Implement the core security isolation layer: Command Policy Engine and Docker Sandboxing.

Purpose: Prevent command injection and unauthorized filesystem access.
Output: Robust policy engine and async Docker orchestration module.
</objective>

<execution_context>
@$HOME/.gemini/get-shit-done/workflows/execute-plan.md
@$HOME/.gemini/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/REQUIREMENTS.md
@.planning/phases/01-foundation-security-sandbox/01-RESEARCH.md
@agent-rs/Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Setup Security Dependencies</name>
  <files>agent-rs/Cargo.toml</files>
  <action>
    Add following dependencies to `agent-rs/Cargo.toml`:
    - `bollard = "0.20.2"`
    - `shell-sanitize = "0.1.0"`
    - `path-security = "0.2.0"`
    - `soft-canonicalize = "0.5.6"`
  </action>
  <verify>
    <automated>cd agent-rs && cargo check</automated>
  </verify>
  <done>Dependencies installed and project compiles.</done>
</task>

<task type="auto">
  <name>Implement Command Policy Engine</name>
  <files>agent-rs/src/security/policy.rs</files>
  <action>
    Implement `PolicyEngine` in `agent-rs/src/security/policy.rs`:
    - Use `shell_words` for POSIX-compliant tokenization.
    - Use `shell_sanitize` to block metacharacters (`|`, `;`, `&`, etc.).
    - Implement path normalization using `path-security` and `soft-canonicalize`.
    - Compare command against an `ALLOWLIST` and `DANGEROUS_COMMANDS`.
    - Provide `validate_command(input: &str) -> Result<Vec<String>, SecurityError>`.
  </action>
  <verify>
    <automated>cd agent-rs && cargo test security::policy</automated>
  </verify>
  <done>Policy engine correctly rejects dangerous/malformed commands and normalizes paths.</done>
</task>

<task type="auto">
  <name>Implement Docker Sandboxing</name>
  <files>agent-rs/src/security/sandbox.rs</files>
  <action>
    Implement `DockerSandbox` in `agent-rs/src/security/sandbox.rs`:
    - Use `bollard` for async Docker API interaction.
    - Implement `run_command(cmd: Vec<String>, image: &str, mount_dir: PathBuf) -> Result<Output, SandboxError>`.
    - Ensure host filesystem access is restricted to the workspace directory.
    - Set environment variables to non-privileged state.
  </action>
  <verify>
    <automated>cd agent-rs && cargo test security::sandbox</automated>
  </verify>
  <done>Commands run inside isolated Docker containers with restricted host access.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
| Boundary | Description |
|----------|-------------|
| Agent → OS | Commands executed on host (via Docker) |

## STRIDE Threat Register
| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-02-01 | Tampering | Shell Command | mitigate | Canonicalize tokens, no `sh -c`, use `bollard` exec |
| T-01-02-02 | Info Disclosure | Filesystem | mitigate | Use `path-security` and restricted Docker volume maps |
| T-01-02-03 | Elevation of Privilege | Docker Container | mitigate | Run as non-root user inside container |
</threat_model>

<verification>
1. Run `cargo test security` to verify policy and sandbox logic.
</verification>

<success_criteria>
- Malformed commands are rejected before execution.
- Paths are canonicalized to prevent traversal.
- Commands execute within a Docker sandbox.
</success_criteria>

<output>
After completion, create `.planning/phases/01-foundation-security-sandbox/01-02-SUMMARY.md`
</output>

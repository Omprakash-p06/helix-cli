# Phase 20: Security Execution Guardrails - Research

**Researched:** 2026-04-13
**Domain:** Tool execution security, input validation, prompt-injection controls
**Confidence:** HIGH (codebase fit), MEDIUM (policy tuning thresholds)

## Summary

Phase 20 should harden the current tool runtime from "sandboxed but permissive" to "policy-gated by default".

Current state already has strong file-path sandboxing for file tools, but `run_terminal_command` still executes raw shell strings (`sh -c`/`cmd /C`) with only a dangerous-prefix blocklist. This is not enough for SEC-01/02/03.

**Primary recommendation:**
1. Add a centralized pre-execution policy engine in Rust that scores every tool call as `allow`, `require_approval`, or `deny`.
2. Replace raw shell-string execution with structured command parsing/validation and explicit allowlisted command families.
3. Add prompt-injection refusal gates that block destructive/exfiltration intent before tool dispatch.
4. Emit policy decisions as structured events so later phases can persist audit logs (Phase 24).

## Standard Stack

### Core (Use)
| Library/Module | Purpose | Why |
|---|---|---|
| Existing `agent-rs/src/tools.rs` sandbox primitives | File boundary enforcement | Already implemented and reliable for file tools |
| `serde` + existing tool schema types | Argument validation normalization | Already in stack; no new framework needed |
| `regex` crate (new) | Exfiltration/dangerous intent pattern gates | Lightweight, deterministic preflight checks |
| `shell-words` crate (new, POSIX) + Windows command tokenizer helper | Parse command string into argv safely | Avoids brittle ad-hoc split logic |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| `globset` | Fine-grained command/path policy patterns |
| `once_cell` | Compile regex policies once with low overhead |

## Architecture Patterns

### Pattern 1: Centralized Tool Policy Gate

**What:** Add one function that all tool calls pass through before execution.

**Placement:**
- New module: `agent-rs/src/security/policy.rs`
- Called from both execution paths in `agent-rs/src/main.rs` and `agent-rs/src/server.rs` right before `execute_tool_async/execute_tool_sync` dispatch.

**Decision model:**
- `Allow`
- `RequireApproval { reason }`
- `Deny { reason, remediation }`

**Policy inputs:**
- Tool name
- Parsed arguments
- Execution mode (`chat` vs `agentic`)
- Workspace root
- User-configured permission tier

### Pattern 2: Permission Tiers (SEC-01)

**What:** Enforce explicit capability tiers.

**Recommended tiers:**
- `read_only`: `read_file`, `list_directory`, `search_codebase`, `get_system_stats`
- `workspace_write`: adds `write_file`, `append_file`
- `full_exec`: includes `run_terminal_command` but still passes risk checks

**Config location:**
- Extend `scripts/config.py` and Rust bridge in `agent-rs/src/config.rs` with `TOOL_PERMISSION_TIER`.

### Pattern 3: Structured Command Validation (SEC-02)

**What:** Validate `run_terminal_command` in two phases.

**Phase A - Syntax/shape:**
- Reject empty command
- Parse argv/token stream
- Reject control-chain operators (`;`, `&&`, `||`, pipes) at Phase 20 scope unless explicitly enabled by policy

**Phase B - semantic risk:**
- Allowlist command families for phase 20 baseline (`git`, `ls`, `cat`, `rg`, `cargo`, `npm`, `python`, etc.)
- Denylist destructive primitives (`rm -rf`, `dd`, `mkfs`, credential file reads, network exfil commands)
- Require explicit approval for medium-risk mutations

### Pattern 4: Prompt-Injection Refusal Gate (SEC-03)

**What:** Pre-dispatch classifier over user/model intent + tool args.

**Implementation approach:**
- Deterministic regex and rule-based heuristics first (no extra model call in phase 20)
- Detect patterns like:
  - instruction override attempts ("ignore previous instructions")
  - secret exfil intent (`/etc/shadow`, SSH keys, env dumps)
  - destructive workspace/system operations
- Emit refusal as safe assistant/system message and do not dispatch tool.

### Pattern 5: Policy Decision Eventing

**What:** Every decision emits a structured event now (in-memory/log stdout), with stable schema for Phase 24 audit persistence.

**Event fields:**
- `timestamp`
- `tool_name`
- `decision` (`allow|approval|deny`)
- `reason_code`
- `risk_level`
- `arg_hash` (SHA-256)

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Command tokenization | naive `split(' ')` | `shell-words` + OS-specific tokenizer path | avoids quote/escape bugs |
| Policy outcomes | ad-hoc booleans spread across files | one `PolicyDecision` enum | testable and traceable |
| Injection detection | giant inline if/else in main loop | rules table + compiled regex set | maintainable and auditable |
| Security telemetry format | free-form strings | typed `SecurityEvent` struct | enables Phase 24 directly |

## Common Pitfalls

### 1) Only patching TUI path
Current repo has both terminal and web execution loops. If policy gate is added only to one path, bypass remains.

**Avoid:** enforce gate in shared execution functions used by both `main.rs` and `server.rs`.

### 2) Treating sandbox as full security
Path sandbox protects file tools, but not shell tool behavior.

**Avoid:** require command-level controls regardless of filesystem sandbox.

### 3) Overly aggressive deny rules
If all mutation commands are denied, agent usefulness collapses.

**Avoid:** tiered policies with `require_approval` middle state.

### 4) Missing backward compatibility
Legacy configs may not contain new permission tier.

**Avoid:** default to safe tier (`workspace_write` or `read_only`) with explicit startup banner.

### 5) Unstructured refusal text
Different refusal strings make testing hard.

**Avoid:** stable error codes/messages for denied operations.

## Code Examples

### Example 1: Policy decision contract

```rust
#[derive(Debug, Clone, Serialize)]
pub enum PolicyDecision {
    Allow,
    RequireApproval { reason_code: String, message: String },
    Deny { reason_code: String, message: String, remediation: String },
}

#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub permission_tier: PermissionTier,
    pub exec_mode: String,
    pub workspace_root: std::path::PathBuf,
}
```

### Example 2: Unified pre-execution gate

```rust
pub fn evaluate_tool_call(tool_name: &str, args: &serde_json::Value, ctx: &PolicyContext) -> PolicyDecision {
    if is_prompt_injection_pattern(args) {
        return PolicyDecision::Deny {
            reason_code: "INJECTION_PATTERN".into(),
            message: "Blocked suspicious instruction pattern.".into(),
            remediation: "Rephrase intent without override/exfiltration directives.".into(),
        };
    }

    if !tier_allows_tool(&ctx.permission_tier, tool_name) {
        return PolicyDecision::Deny {
            reason_code: "TIER_DENY".into(),
            message: format!("Tool '{}' is not allowed in current permission tier.", tool_name),
            remediation: "Switch to a higher tier or use an allowed tool.".into(),
        };
    }

    if tool_name == "run_terminal_command" {
        return evaluate_command_risk(args);
    }

    PolicyDecision::Allow
}
```

### Example 3: Dispatch integration point

```rust
match evaluate_tool_call(&func_name, &parsed_args, &policy_ctx) {
    PolicyDecision::Allow => {
        // execute as today
    }
    PolicyDecision::RequireApproval { message, .. } => {
        // emit checkpoint-style message and pause this call
    }
    PolicyDecision::Deny { message, remediation, .. } => {
        // inject structured tool failure message for model/user visibility
    }
}
```

## Source Evidence (Codebase)

- Shell execution path: `agent-rs/src/tools.rs`
- Existing file sandboxing: `agent-rs/src/tools.rs`
- Current dangerous command config: `scripts/config.py`
- Tool orchestration call sites: `agent-rs/src/main.rs`, `agent-rs/src/server.rs`

## Implementation Readiness

Ready for `/gsd-plan-phase 20`.

Planners should structure tasks in this order:
1. Define policy contracts + config bridge
2. Implement command parsing/risk scoring + injection rules
3. Wire gate into both terminal and web execution paths with tests

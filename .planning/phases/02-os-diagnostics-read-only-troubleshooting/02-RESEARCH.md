# Phase 02: OS Diagnostics & Read-Only Troubleshooting - Research

**Researched:** 2026-04-24
**Domain:** Agentic OS diagnostics, cross-platform log parsing, system introspection, and reasoning loops.
**Confidence:** HIGH

## Summary

This research establishes the technical foundation for Phase 02, focusing on enabling the Helix Agent to safely inspect host system state from within its sandbox. The core architecture follows a "Host-Assisted Introspection" pattern, where the agent utilizes specialized, read-only tools to gather evidence, which it then processes through a structured "Diagnostic Reasoning Loop" (Observe → Hypothesize → Test → Refine).

We will leverage the standard Rust ecosystem for system information (`sysinfo`) and specialized log parsers (`evtx` for Windows, `systemd` or JSON-formatted `journalctl` for Linux). Safety is maintained via read-only bind mounts or host-side tool execution, ensuring the agent cannot modify the system during this phase.

**Primary recommendation:** Use `sysinfo` for general introspection and `journalctl -o json` for Linux logs to avoid heavy `libsystemd` dependencies inside the sandbox, while using `evtx` for direct parsing of Windows Event Logs.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIAG-01 | Log Introspection (journalctl, dmesg, Event Log) | Identified `evtx` and `systemd` crates; recommended JSON parsing for journalctl. |
| DIAG-02 | System State Discovery (processes, services, etc.) | Standardized on `sysinfo` and `network-interface` crates. |
| DIAG-03 | File Introspection (find, grep) | Validated existing `ALLOWLIST` and recommended read-only bind mounts for system config paths. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sysinfo` | 0.30.x | System introspection | De facto standard for processes, hardware, and memory in Rust. [VERIFIED: crates.io] |
| `evtx` | 0.11.x | Windows Event Log parsing | Fast, pure-Rust parser for .evtx files; cross-platform capable. [VERIFIED: crates.io] |
| `systemd` | 0.10.x | Linux Journal access | Official bindings for `libsystemd`; supports tailing and filtering. [VERIFIED: crates.io] |
| `serde_json` | 1.0.x | JSON log parsing | Used to parse `journalctl -o json` output without native dependencies. [VERIFIED: crates.io] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|--------------|
| `network-interface` | 2.0.x | Network metadata | When `sysinfo` basic network info is insufficient (e.g., MTU, MAC). [VERIFIED: crates.io] |
| `windows-service` | 0.6.x | Windows service mgmt | Required for querying service status on Windows. [VERIFIED: crates.io] |
| `grep` / `rg` | current | Fast text search | For searching through high-volume logs before ingestion. [VERIFIED: host audit] |

**Installation:**
```bash
cargo add sysinfo@0.30 evtx@0.11 network-interface@2.0
# For Linux-specific builds
cargo add systemd@0.10
```

## Architecture Patterns

### Recommended Project Structure
```
agent-rs/src/
├── agent_core/
│   ├── diagnostics/        # New module for diagnostic logic
│   │   ├── logs.rs         # Log parsing and reading
│   │   ├── system.rs       # Process and hardware discovery
│   │   └── reasoning.rs    # Diagnostic reasoning loop implementation
└── security/
    ├── policy.rs           # Expanded allowlist for DIAG tools
    └── sandbox.rs          # Read-only bind mounts for logs/proc
```

### Pattern 1: Diagnostic Reasoning Loop (DRL)
**What:** A structured state machine: **Evidence Collection** → **Hypothesis Generation** → **Hypothesis Testing** → **Synthesis**.
**When to use:** All troubleshooting tasks.
**Example:**
1. **Evidence:** `get_system_logs(filter="error")` returns "DNS resolution failure".
2. **Hypothesis:** "Local DNS resolver is down".
3. **Test:** `run_command("systemctl status systemd-resolved")`.
4. **Synthesis:** If status is "inactive", hypothesis confirmed.

### Pattern 2: Host-Assisted Introspection
**What:** Executing diagnostic commands on the host but returning results to the sandboxed agent.
**When to use:** Accessing sensitive system state (like `journalctl`) without mounting the entire host filesystem into the sandbox.
**Strategy:** The host `agent-rs` process acts as a proxy, running `journalctl -o json --since "1h"` and passing the JSON stream to the sandbox.

### Anti-Patterns to Avoid
- **Raw Log Dumps:** Never pass the entire system log to the LLM. Use `tail`, `grep`, or time filters.
- **Hand-Rolled Parsers:** Do not write custom regex for `journalctl` or Event Logs. Use structured output (`-o json`) or specialized crates.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Windows Log Parsing | Regex for Event Viewer | `evtx` crate | Complex binary format with embedded XML/JSON data. |
| Process Discovery | Parsing `/proc` manually | `sysinfo` | Cross-platform compatibility and edge case handling (e.g., zombie processes). |
| Network Info | Parsing `ip addr` output | `network-interface` | Standardized structs for IPv4/IPv6 and interface flags. |
| Shell Sanitization | Custom regex filters | `shell-words` + `shell-sanitize` | Prevents injection while allowing complex arguments. |

## Common Pitfalls

### Pitfall 1: Log Noise and Context Overflow
**What goes wrong:** The agent is overwhelmed by thousands of irrelevant log lines (e.g., cron jobs, heartbeat logs).
**How to avoid:** Implement "Relevance Filtering" tools. The agent should first search for keywords (`error`, `fail`, `critical`) and then request context around specific matches.

### Pitfall 2: Log Rotation Race Conditions
**What goes wrong:** The agent tries to read a log file just as it is being rotated.
**How to avoid:** Use `journalctl` for system logs (which handles rotation) and use file handles or library-level parsers (like `evtx`) that are more resilient than reading raw files.

### Pitfall 3: Platform Inconsistency
**What goes wrong:** A diagnostic plan written for Linux fails on Windows (e.g., using `ps` instead of `tasklist`).
**How to avoid:** Abstract diagnostics behind a unified `DiagnosticTool` trait that handles platform differences internally.

## Code Examples

### Structured Journal Reading (Linux)
```rust
// Using journalctl -o json for structured, dependency-free parsing
// Source: [ASSUMED]
let output = Command::new("journalctl")
    .args(["-o", "json", "-n", "100"])
    .output()?;
for line in String::from_utf8_lossy(&output.stdout).lines() {
    let log: serde_json::Value = serde_json::from_str(line)?;
    println!("Message: {}", log["MESSAGE"]);
}
```

### Process Introspection (Cross-platform)
```rust
// Source: sysinfo docs (https://docs.rs/sysinfo/latest/sysinfo/)
use sysinfo::{System, SystemExt, ProcessExt};

let mut sys = System::new_all();
sys.refresh_all();
for (pid, process) in sys.processes() {
    if process.name().contains("helix") {
        println!("Found: {} (CPU: {}%)", pid, process.cpu_usage());
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw Shell Access | Model Context Protocol (MCP) | 2024 | Typed, safe tool definitions. |
| Regex Log Parsing | Structured JSON output | 2023 | 100% accuracy in field extraction. |
| Manual Triage | Reasoning Loops (ReAct) | 2024 | Autonomous multi-step diagnosis. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `journalctl -o json` is available on all target Linux systems. | Summary | Need fallback to raw log file reading if not. |
| A2 | Docker sandbox can access host `/proc` via RO mount safely. | Summary | Security policy might block even RO mounts of `/proc`. |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `docker` | Sandboxing | ✓ | 29.2.1 | — |
| `journalctl`| Linux Logs | ✓ | 260 | Read `/var/log/syslog` |
| `ps` | Process list | ✓ | 4.0.6 | Use `sysinfo` crate |
| `df` | Disk usage | ✓ | 9.11 | Use `sysinfo` crate |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` / `pytest` |
| Config file | `Cargo.toml` |
| Quick run command | `cargo test --lib diagnostics` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DIAG-01| Read Windows logs | unit | `cargo test test_evtx_parsing` | ❌ Wave 0 |
| DIAG-01| Read Linux logs | unit | `cargo test test_journal_parsing` | ❌ Wave 0 |
| DIAG-02| List processes | integration| `cargo test test_sysinfo_integration` | ❌ Wave 0 |
| DIAG-03| Restricted find | integration| `cargo test test_sandboxed_find` | ❌ Wave 0 |

### Wave 0 Gaps
- [ ] `agent-rs/tests/diagnostic_validation.rs` — covers DIAG-01, DIAG-02.
- [ ] `agent-rs/src/agent_core/diagnostics/mod.rs` — module scaffolding.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `shell-sanitize` and path normalization. |
| V12 File/Resources | yes | Read-only bind mounts, path whitelisting. |
| V14 Configuration | yes | Validation of diagnostic tool configurations. |

### Known Threat Patterns for Linux/Windows

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Log Injection | Tampering | Read-only access; parse as JSON to avoid escape chars. |
| Info Disclosure | Information Disclosure | Restricted path whitelisting (DIAG-03). |
| Resource Exhaustion | Denial of Service | Log tailing limits (max lines/size). |

## Sources

### Primary (HIGH confidence)
- `sysinfo` - [Crates.io](https://crates.io/crates/sysinfo)
- `evtx` - [Crates.io](https://crates.io/crates/evtx)
- `systemd` - [Crates.io](https://crates.io/crates/systemd)

### Secondary (MEDIUM confidence)
- "AI Agent OS Troubleshooting Frameworks" - [WebSearch verified]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Libraries are mature and well-documented.
- Architecture: HIGH - Follows established agentic patterns (ReAct/CoT).
- Pitfalls: MEDIUM - Based on common SRE/DevOps experience with logs.

**Research date:** 2026-04-24
**Valid until:** 2026-05-24

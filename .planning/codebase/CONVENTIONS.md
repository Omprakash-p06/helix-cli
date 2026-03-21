# Conventions

## Python Conventions
- Most scripts are executable with shebang `#!/usr/bin/env python3`.
- Logging style is CLI-oriented with explicit status prefixes.
- Config and behavior often controlled through environment variables.
- Setup and launcher scripts avoid framework-heavy abstractions.

## Rust Conventions
- Serde models use explicit structs/enums for tool schemas.
- Tool argument contracts are defined in `agent-rs/src/tools.rs`.
- JSON interaction uses `serde_json::Value` where schema flexibility is needed.
- Error handling prefers explicit match blocks and user-readable messages.

## Runtime UX Conventions
- Helix branding banner is centralized in `scripts/helix_branding.py`.
- Startup flows print major stage separators.
- Logs for startup diagnostics are written to `logs/`.

## Safety Conventions
- Tool execution enforces sandbox path checks in Rust tool layer.
- Dangerous terminal command checks exist before execution.
- Setup enforces minimum context constraints for user runtime.

## File and Path Conventions
- Project-relative paths are preferred in Python launchers.
- Binary path selection includes OS-specific branching.
- Startup model selection is path-based (`models/*.gguf`) with env overrides.

## Documentation Conventions
- README describes architecture and run commands in practical order.
- GSD workflow docs are structured with explicit step gates.
- Mapping docs use markdown headings and concrete file references.

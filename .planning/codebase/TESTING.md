# Testing Framework & Validation Suite

**Last Updated:** 2026-07-24

## Overview

Helix CLI employs a multi-level testing strategy covering Rust integration tests (`agent-rs/tests/`), Python unit & accuracy tests (`tests/`), and Web UI linting/build validations (`web-ui/`).

## Test Organization

### 1. Rust Integration & Safety Tests (`agent-rs/tests/`)
Rust integration tests use cargo's standard test runner to validate core agent mechanics, sandboxing, session persistence, and diagnostic systems.

| Test File | Focus Area | Key Verifications |
| --- | --- | --- |
| [agent-rs/tests/security_guardrails.rs](file:///home/omprakash/helix-cli/agent-rs/tests/security_guardrails.rs) | Security & Sandbox | Path traversal blocking, dangerous command flags, sandbox isolation |
| [agent-rs/tests/test_secure_execution.rs](file:///home/omprakash/helix-cli/agent-rs/tests/test_secure_execution.rs) | Execution Policy | User permission approval gates and command execution safety |
| [agent-rs/tests/test_session_persistence.rs](file:///home/omprakash/helix-cli/agent-rs/tests/test_session_persistence.rs) | Session & SQLite Store | Session creation, transcript persistence, SQLite database schema integrity |
| [agent-rs/tests/tool_runtime_contracts.rs](file:///home/omprakash/helix-cli/agent-rs/tests/tool_runtime_contracts.rs) | Tool Runtime | Function calling schema validation, JSON payload formatting |
| [agent-rs/tests/os_diagnostics_integration.rs](file:///home/omprakash/helix-cli/agent-rs/tests/os_diagnostics_integration.rs) | OS Diagnostics | System metrics gathering, event log parsing, process inspection |
| [agent-rs/tests/gsd_orchestration_validation.rs](file:///home/omprakash/helix-cli/agent-rs/tests/gsd_orchestration_validation.rs) | GSD Workflows | Phase state transitions, context reset logic, artifact generation |
| [agent-rs/tests/runtime_profile_watchdog.rs](file:///home/omprakash/helix-cli/agent-rs/tests/runtime_profile_watchdog.rs) | Performance Watchdog | VRAM allocation monitoring, timeout recovery, process health check |

### 2. Python System & Installation Tests (`tests/`)
Python tests validate local engine bootstrapping, model discovery, hardware capability estimation, and configuration loading.

| Test File | Focus Area |
| --- | --- |
| [tests/test_accuracy.py](file:///home/omprakash/helix-cli/tests/test_accuracy.py) | Evaluates agent diagnostic & repair accuracy against benchmark datasets ([tests/dataset.json](file:///home/omprakash/helix-cli/tests/dataset.json)) |
| [tests/test_model_discovery.py](file:///home/omprakash/helix-cli/tests/test_model_discovery.py) | Verifies GGUF model scanning, VRAM estimation logic, and path resolution |
| [tests/test_onboarding_profile.py](file:///home/omprakash/helix-cli/tests/test_onboarding_profile.py) | Tests `.helix_profile.json` user settings load/save behavior |
| [tests/test_system_check.py](file:///home/omprakash/helix-cli/tests/test_system_check.py) | Validates hardware, CPU features, and GPU VRAM detection |

### 3. Web UI Validation (`web-ui/`)
- TypeScript type checking (`tsc -b`)
- ESLint code quality rules (`npm run lint`)
- Production build compilation (`npm run build`)

## Executing Tests

### Running Rust Tests
```bash
cd agent-rs
cargo test
```

To run a specific Rust integration test:
```bash
cargo test --test security_guardrails
```

### Running Python Tests
```bash
pytest tests/
# Or with standard library unittest runner:
python3 -m unittest discover -s tests
```

### Running Web UI Checks
```bash
cd web-ui
npm run lint
npm run build
```

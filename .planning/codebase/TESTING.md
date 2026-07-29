# Testing Patterns

**Analysis Date:** 2026-07-30

## Test Framework — Rust

**Runner:**
- Built-in `#[test]` and `#[tokio::test]` (cargo test framework)
- Edition 2024, no custom test harness
- Config: None — test configuration is entirely in `Cargo.toml`

**Dev-dependencies (`Cargo.toml`):**
- `tempfile = "3.10"` — temporary files and directories
- `mockall = "0.13"` — mock objects (imported but not yet used in test files surveyed)

**Run Commands (typical):**
```bash
cargo test                          # Run all Rust tests
cargo test -- --nocapture           # With stdout visible
cargo test <test_name>              # Single test
```

## Test Framework — Python

**Runner:**
- `pytest` (no `pytest.ini` or `pyproject.toml` config found — uses defaults)
- No `conftest.py` found

**Run Commands (typical):**
```bash
python -m pytest tests/             # Run all Python tests
pytest tests/ -v                    # Verbose
pytest tests/test_xxx.py::test_yyy # Single test
```

## Test File Organization

### Rust Tests

**Two locations:**

1. **Integration tests:** `agent-rs/tests/*.rs` — 18 test files
   - Each file is a separate integration test crate
   - Files named by feature/module: `audit_log_mvp.rs`, `security_guardrails.rs`, `tool_runtime_contracts.rs`, etc.
   - Directories not used (flat file layout)

2. **Unit tests:** `#[cfg(test)] mod tests { ... }` inline in source files
   - Found in: `agent-rs/src/config.rs`, `agent-rs/src/utils.rs`, `agent-rs/src/tools.rs`
   - Pattern: `#[cfg(test)] mod tests { use super::*; ... }`

**Test file naming:**
- Integration: `descriptive_name.rs` — `security_guardrails.rs`, `audit_log_mvp.rs`
- Unit: embedded `mod tests` within source

**Cross-crate module inclusion in tests:**
- Pattern `#[path = "../src/audit.rs"] mod audit;` used in `test_session_persistence.rs`, `audit_log_mvp.rs`, `runtime_profile_watchdog_validation.rs`
- Used when the module is not `pub` or not re-exported from `lib.rs`

### Python Tests

**Single location:** `tests/*.py` — 10 test files
- Flat directory, no subdirectories
- Files named `test_<module>.py`: `test_onboarding_profile.py`, `test_model_install.py`, etc.
- One standalone benchmark file: `tests/eval.py` (not auto-discovered by pytest)

**Test file structure:**
```python
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import config
from model_install import verify_model_integrity
```

## Test Structure

### Rust Integration Tests

**Pattern — Feature/module validation:**
```rust
// tests/audit_log_mvp.rs
#[path = "../src/audit.rs"]
mod audit;

use audit::AuditStore;

#[test]
fn test_audit_append_and_query() {
    let db_path = "test_audit.db";
    let _ = fs::remove_file(db_path);
    let store = AuditStore::new(db_path).expect("Failed to create AuditStore");
    // ... act and assert
    let _ = fs::remove_file(db_path);
}
```

**Pattern — Async integration:**
```rust
#[tokio::test]
async fn test_tool_runtime_basic_execution() {
    let registry = Arc::new(tools::create_default_registry());
    let req = ToolRequest { ... };
    let tool_runtime = ToolRuntime::new(None, None);
    let (id, result, name) = tool_runtime.execute(req, ...).await;
    assert_eq!(id, "test_1");
    assert!(result.success);
    assert!(result.output.contains("RAM"));
}
```

**Pattern — Static source validation (source-code contract tests):**
```rust
// tests/security_guardrails.rs
#[test]
fn security_logic_centralized_in_runtime() {
    let runtime_rs = read("src/agent_core/tool_runtime.rs");
    assert!(runtime_rs.contains("evaluate_tool_call"));
    assert!(runtime_rs.contains("PolicyDecision::Deny"));
}
```
This is a unique pattern in this codebase — tests verify source code *content* to enforce architecture decisions. Used extensively in `security_guardrails.rs`.

### Rust Unit Tests (inline)

**Pattern:**
```rust
// In config.rs:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_permission_tier_defaults_to_workspace_write_when_missing_equivalent() {
        let parsed = PermissionTier::from_config_value("workspace_write").unwrap_or_default();
        assert_eq!(parsed, PermissionTier::WorkspaceWrite);
    }
}
```

### Python Tests

**Pattern — `tmp_path` fixture with `monkeypatch`:**
```python
def test_first_run_profile_creation(tmp_path: Path, monkeypatch):
    profile_file = tmp_path / "onboarding_profile.json"
    monkeypatch.setenv("HELIX_PROFILE_PATH", str(profile_file))
    profile = op.load_profile()
    assert profile == {}
```

**Pattern — Class-based test grouping:**
```python
class TestScanModelsDirectory:
    """Test scan_models_directory() returns proper data structures."""

    def test_returns_list_of_model_entries(self, tmp_path):
        """scan_models_directory returns a list."""
        ...
```

**Pattern — Parametrized tests:**
```python
@pytest.mark.parametrize(
    "vram, expected_quantization, expected_backend_hint, expected_gpu_layers",
    [
        (0, "Q4_K_M", "cpu", 0),
        (8, "Q4_K_M", "cuda", 24),
    ],
)
def test_qwen_27b_variant_selection_by_vram(vram, ...):
    entry = config.build_model_entry("Qwen-3.6-27B-MoE", vram)
    assert entry["quantization"] == expected_quantization
```

**Pattern — Mock with `unittest.mock.patch`:**
```python
from unittest.mock import patch, MagicMock

class TestDownloadFile:
    @patch("download_model._api")
    @patch("download_model._repo_info")
    def test_returns_gguf_files_only(self, mock_repo_info, mock_api):
        from download_model import list_repo_files
        mock_sibling_gguf = MagicMock()
        mock_sibling_gguf.rfilename = "model-Q4_K_M.gguf"
        ...
```

## What Is Tested

### Rust — Test Coverage Areas:

**Module mapped against test files:**

| Module | Test File(s) | Type |
|--------|-------------|------|
| `audit` | `tests/audit_log_mvp.rs` | Integration |
| `context` (indexer, memory, retrieval, budget, skeleton) | `tests/context_integration.rs` | Integration |
| `agent_core/tool_runtime` | `tests/tool_runtime_contracts.rs` | Integration |
| `agent_core/orchestration` | `tests/gsd_orchestration_validation.rs` | Integration |
| `agent_core/diagnostics` (reasoning, system, logs) | `tests/diagnostic_validation.rs`, `tests/os_diagnostics_integration.rs` | Integration |
| `security` (capabilities, policy, sandbox) | `tests/security_guardrails.rs` | Integration |
| `runtime_profile` | `tests/runtime_profile_watchdog.rs`, `tests/runtime_profile_watchdog_validation.rs` | Integration |
| `watchdog` | `tests/runtime_profile_watchdog_validation.rs` | Integration |
| `session` | `tests/test_session_persistence.rs` | Integration |
| `tools` (tool dispatch) | `tests/tool_execution.rs` | Integration |
| `config` (unit) | inline `#[cfg(test)] mod tests` | Unit |
| `utils` (unit) | inline `#[cfg(test)] mod tests` | Unit |
| `tui` | `tests/streaming_tui_refinement.rs` | Integration |
| `web_research` | `tests/web_research_adversarial.rs` | Integration |
| `agent_core/repair` | (not directly tested — covered by tool_runtime tests) | — |

### Python — Test Coverage Areas:

| Module | Test File(s) |
|--------|-------------|
| `scripts/config` | `tests/test_qwen_config.py`, `tests/test_model_discovery.py`, `tests/test_system_check.py` |
| `scripts/onboarding_profile` | `tests/test_onboarding_profile.py` |
| `scripts/start_server` | `tests/test_start_server_runtime_profile.py`, `tests/test_start_server_runtime_profile_validation.py` |
| `scripts/system_check` | `tests/test_system_check.py` |
| `scripts/model_install` | `tests/test_model_install.py` |
| `scripts/download_model` | `tests/test_download_model.py` |

## What Is NOT Tested (Gaps)

**Rust (untested or lightly tested):**
- `tui/` modules (`themes.rs`, `state.rs`, `events.rs`, `commands.rs`, `approval.rs`, `api.rs`) — only `streaming_tui_refinement.rs` covers TUI
- `server.rs` — web server path has minimal test coverage in `security_guardrails.rs` (just checks `.execute()` call exists)
- `input.rs` — no dedicated test file
- `tokens.rs` — no test file
- `agent_core/repair/` submodules (`scoring.rs`, `snapshots.rs`, `workflow.rs`) — no dedicated tests
- `agent_core/guardian.rs` — no test file
- `agent_core/web_research/` submodules (`planner.rs`, `classifier.rs`, `sanitize.rs`, `worker.rs`, `store.rs`, `synthesizer.rs`) — only `web_research_adversarial.rs` covers these lightly

**Python (untested):**
- `scripts/helix.py` — CLI entry point not tested
- `scripts/helix_branding.py` — not tested
- `scripts/build_zip.py` — not tested
- `start.py` — integration boot sequence not tested (depends on running server)
- `setup.py` — full install path not tested in unit tests

## Mocking

**Rust — mockall (available but unused):**
- `mockall = "0.13"` is declared in `[dev-dependencies]` but no tests in the codebase use `mockall::mock!` or `#[automock]`
- Test doubles instead use hand-rolled structs:
  ```rust
  // tests/tool_runtime_contracts.rs
  struct AlwaysAllowRequester;
  #[async_trait::async_trait]
  impl PermissionRequester for AlwaysAllowRequester {
      async fn request_permission(&self, _req: PermissionRequest) -> PermissionResponse {
          PermissionResponse::Allow
      }
  }
  ```
- **What is mocked:** Nothing is mocked in Rust tests — tests call real implementations
- **What should be mocked:** External system calls (journalctl, nvidia-smi, Docker sandbox) are tested with real system responses, making tests non-deterministic on different platforms

**Python — `unittest.mock`:**
- `from unittest.mock import patch, MagicMock` — used in `tests/test_download_model.py`
- `pytest.monkeypatch` fixture — used in `tests/test_start_server_runtime_profile.py`, `tests/test_start_server_runtime_profile_validation.py`, `tests/test_onboarding_profile.py`, `tests/test_model_install.py`
- `monkeypatch.setattr()` pattern for replacing module-level functions/variables

## Fixtures and Factories

**Python fixtures:**
- `tmp_path` — built-in pytest fixture, used in ALL Python test files
- `temp_models_dir(tmp_path)` — custom fixture in `tests/test_model_install.py`
- `monkeypatch` — used for env var and configuration injection

**Rust fixtures:**
- No fixture framework — tests manually set up state in each function
- Helper functions: `sample_messages()` in `test_session_persistence.rs`, `test_dir()` helper

## Coverage

- **No coverage tooling configured** — no `--coverage` flags, no `codecov.yml`, no `tarpaulin`, no `cargo-llvm-cov`
- **No coverage targets set**

**To view coverage manually:**
```bash
# Rust (requires cargo-llvm-cov):
cargo llvm-cov

# Python:
pytest --cov=scripts tests/
```

## Test Quality Observations

**Rust tests:**
- Strong assertion quality — every test uses multiple assertions with descriptive failure messages
- Edge case testing present: empty directories (`test_handles_empty_directory`), oversized content (`test_diagnostic_read_size_limit`),
  tampered data (`test_audit_tamper_detection`), malformed input (`test_audit_invalid_numeric_overrides_are_ignored`)
- Some tests have nondeterministic results due to platform dependencies (e.g., `os_diagnostics_integration.rs` expects `journalctl` on Linux)
- `SECURITY_VIOLATION` string assertions ensure security error messages are stable
- Timeout tests use `std::thread::sleep(35)` which is slow but deterministic

**Python tests:**
- Comprehensive parametrized tests for model selection logic (`test_qwen_config.py`)
- Temp directory cleanup handled by `tmp_path` (automatic)
- Good use of descriptive `"""docstrings"""` per test function
- Edge cases tested: missing directory, empty directory, no GGUF files, invalid env values

**Shared pattern — Comment-driven test organization:**
- Rust integration tests use `// ── SECTION: Feature area ──` section markers
- Python test classes use `"""docstring"""` to describe the test scenario

## Key Observations

1. **Bimodal test quality:** Rust integration tests are thorough with strong assertions; Rust unit tests exist only in 3 files (`config.rs`, `utils.rs`, `tools.rs`) — many modules have zero unit tests.

2. **Static source contract tests are unique:** `security_guardrails.rs` parses source files as strings and asserts on content presence. This is an architectural enforcement pattern — the tests fail if security logic is moved without updating all paths.

3. **No mocks in Rust:** All Rust tests call real implementations (including system calls). This creates platform-dependent test results. `mockall` is available but unused.

4. **Python uses idiomatic pytest.** `tmp_path`, `monkeypatch`, `@pytest.mark.parametrize`, and `unittest.mock.patch` are all used correctly.

5. **No E2E tests.** The `tests/eval.py` script exists but is a standalone benchmark runner, not a pytest test file. It requires a running server.

6. **No CI config for tests** — no GitHub Actions or CI pipeline detected for automating test runs.

---

*Testing analysis: 2026-07-30*

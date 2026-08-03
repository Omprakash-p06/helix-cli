# Testing Patterns

**Analysis Date:** 2026-08-03

Three test surfaces: **Python pytest** (`tests/`), **Rust cargo test** (`agent-rs/`), and **LLM-backed benchmark scripts** (`tests/eval.py`, `tests/test_accuracy.py`). The web UI (`web-ui/`) has **no tests** (no test script in `web-ui/package.json`).

## Test Framework

**Runner:**
- Python: pytest 9.0.3 (confirmed via `.pytest_cache/`), **no config file** (no `pytest.ini`, `pyproject.toml`, or `setup.cfg`); default discovery of `tests/test_*.py`
- Rust: standard cargo test harness; dev-dependencies in `agent-rs/Cargo.toml`: `tempfile 3.10`, `mockall 0.13`, `mockito 1.4`

**Assertion Library:**
- Python: pytest built-in `assert`
- Rust: built-in `assert!`, `assert_eq!`, `assert_ne!`

**Run Commands:**
```bash
python -m pytest tests/                        # Run all Python tests
python -m pytest tests/test_system_check.py    # Single file
python -m pytest tests/test_qwen_config.py -k gemma   # Filter by keyword
cargo test                                     # Run all Rust tests (cwd: agent-rs/)
cargo test --test tool_execution               # Single Rust integration test file
python tests/test_accuracy.py                  # Live tool-calling accuracy (server must run on :8080)
python tests/eval.py                           # Agentic benchmark (needs AGENT_BIN + judge LLM)
```

## Test File Organization

**Location:**
- Python: dedicated `tests/` directory, files named `test_*.py`; **no `conftest.py`**, no `__init__.py`
- Rust: unit tests inline `#[cfg(test)] mod tests` in `agent-rs/src/`; integration tests in `agent-rs/tests/*.rs`
- Web: none

**Naming:**
- Python unit: `tests/test_<module>.py` → `test_model_install.py`, `test_qwen_config.py`, `test_system_check.py`, `test_download_model.py`, `test_model_discovery.py`, `test_onboarding_profile.py`, `test_start_server_runtime_profile.py`, `test_start_server_runtime_profile_validation.py`
- Rust unit: `#[cfg(test)] mod tests` inside `src/<module>.rs` (e.g. `src/utils.rs:238`)
- Rust integration: `tests/<area>.rs` → `tool_execution.rs`, `audit_log_mvp.rs`, `context_integration.rs`, `security_guardrails.rs`, `eval_suite_*.rs`
- Test functions: Python `test_<behavior>`; Rust `test_<behavior>` and `integration_<metric>` (`integration_symbol_lookup_under_3s`)

**Structure:**
```
tests/
  test_system_check.py              # config/system_check unit tests
  test_model_install.py             # model_install unit tests (monkeypatch)
  test_download_model.py            # download_model unit tests (unittest.mock)
  test_onboarding_profile.py        # onboarding_profile unit tests
  test_qwen_config.py               # config.build_model_entry parametrized tests
  test_start_server_runtime_profile*.py  # start_server.apply_runtime_overrides tests
  dataset.json                      # benchmark dataset for tests/eval.py
  eval.py                           # agentic benchmark script (NOT pytest)
  test_accuracy.py                  # live LLM tool-call accuracy script (NOT pytest)
agent-rs/
  src/<module>.rs                   # inline #[cfg(test)] mod tests
  tests/<area>.rs                   # integration tests
```

## Test Structure

**Suite Organization (Python):**
```python
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import config
import system_check


def test_quantization_advice_reflects_configured_qwen_profile():
    advice = system_check.quantization_advice()

    assert config.MODEL_NAME in advice["detail"]
```
- Both class-based (`class TestScanModelsDirectory:` in `tests/test_model_discovery.py:19`) and module-level `test_*` functions
- Docstring on each test describing the behavior
- `@pytest.mark.parametrize` for table-driven cases (`tests/test_qwen_config.py:15`)

**Suite Organization (Rust):**
```rust
#[cfg(test)]
mod tests {
    use super::clean_chat_output;

    #[test]
    fn preserves_code_and_tool_json_while_cleaning() {
        let input = "<think>secret</think>```\nconst x = \"hello\";\n```";
        let output = clean_chat_output(input);
        assert!(output.contains("```\nconst x = \"hello\";\n```"));
        assert!(!output.contains("secret"));
    }
}
```
(`agent-rs/src/utils.rs:238`)

**Patterns:**
- `tmp_path` fixture for filesystem isolation; dummy files written inline (`(models_dir / "small-model.gguf").write_bytes(b"x" * 1024 * 1024)` in `tests/test_model_discovery.py:48`)
- `monkeypatch` for env vars and module-global mutation
- Tests mutate imported module state directly (`config.BATCH_SIZE = 512` in `tests/test_start_server_runtime_profile.py:31`) — state resets are manual and order-sensitive within a file
- Rust tests create real SQLite DB files in cwd and clean up with `fs::remove_file` (`.gitignore` ignores `test_audit*.db` written by cargo test)

## Mocking

**Framework:**
- Python: pytest `monkeypatch` (env vars, module attrs) + `unittest.mock.patch`/`MagicMock` (object mocking)
- Rust: mockall + mockito declared in dev-deps

**Patterns:**
```python
# Module-global attr patch (tests/test_model_install.py:49)
monkeypatch.setattr("model_install.MODELS_DIR", tmp_path / "models")
monkeypatch.setattr("model_install.download_model_to_staging", lambda spec, **kwargs: dummy_model)

# Env var (tests/test_onboarding_profile.py:13)
monkeypatch.setenv("HELIX_PROFILE_PATH", str(profile_file))

# unittest.mock patch of module-level helpers (tests/test_download_model.py:26)
@patch("download_model._api")
@patch("download_model._repo_info")
def test_returns_gguf_files_only(self, mock_repo_info, mock_api):
    mock_sibling_gguf = MagicMock()
    mock_sibling_gguf.rfilename = "model-Q4_K_M.gguf"
    mock_sibling_gguf.lfs = MagicMock(sha256="abc123")
    mock_repo_info.return_value = mock_repo_obj
```

**What to Mock:**
- Network/external services: `download_model._api` / `download_model._repo_info` (HuggingFace SDK) in `tests/test_download_model.py`
- Env vars (`HELIX_*` overrides) in runtime-profile and onboarding tests
- Module-level constants/paths (`MODELS_DIR`, `STAGING_DIR`) redirected to `tmp_path`
- Server launch is avoided entirely — `apply_runtime_overrides()` is tested without spawning a process (`tests/test_start_server_runtime_profile.py:23`)

**What NOT to Mock:**
- Internal pure functions — tested directly against real implementations
- Config module constants — asserted directly (`assert config.MODEL_NAME in advice["detail"]`)

## Fixtures and Factories

**Test Data:**
- No factory modules or shared fixtures; dict literals inline per test
- One custom fixture `temp_models_dir(tmp_path)` in `tests/test_model_install.py:22` — currently unused (tests prefer direct `monkeypatch.setattr`)
- Dummy files written inline per test
- Rust: `tempfile::NamedTempFile` / `std::env::temp_dir()` for DB-backed tests (`agent-rs/tests/context_integration.rs:10`)
- Benchmark dataset lives at `tests/dataset.json` (consumed by `tests/eval.py`, not pytest)

**Location:** Inline in test files.

## Coverage

**Requirements:**
- None — no coverage config, no thresholds, no CI pipeline (no `.github/` workflows)
- No `pytest.ini`/`pyproject.toml` coverage settings

## Test Types

**Unit Tests (Python):**
- Test single functions from `scripts/` modules in isolation: `test_system_check.py`, `test_qwen_config.py`, `test_model_install.py`, `test_onboarding_profile.py`
- Isolate via `tmp_path` + `monkeypatch`; no network, no subprocess

**Integration Tests (Rust):**
- `agent-rs/tests/*.rs` exercise real modules against real SQLite files / real workspace trees:
  - `audit_log_mvp.rs` — real `AuditStore` SQLite DB, tamper detection
  - `context_integration.rs` — `ContextEngine` against `agent-rs/src` workspace, timing asserts
- Two import styles: `use agent_rs::context::{ContextEngine, ContextQuery}` (lib crate) and `#[path = "../src/stream.rs"] mod stream;` for binary-private modules (`streaming_tui_refinement.rs:1`)

**E2E / LLM Benchmarks (Python, not pytest):**
- `tests/test_accuracy.py`: live tool-calling accuracy against a running llama-server at `http://127.0.0.1:8080/v1/chat/completions`; exits non-zero on failures
- `tests/eval.py`: trajectory-based agentic benchmark — spawns `AGENT_BIN` (`agent-rs/target/debug/agent-rs(.exe)` or `AGENT_BIN` env), judges outputs via `HELIX_JUDGE_URL` LLM, writes `tests/benchmark_results.md`; filters via `HELIX_EVAL_MAX_TASKS`, `HELIX_EVAL_CATEGORIES`; invoked by `setup.py` preflight (`setup.py:881`)

## Common Patterns

**Async Testing:**
- No async Python tests. Rust integration tests use blocking `#[test]` (async engines constructed synchronously)

**Error Testing (Python):**
```python
with pytest.raises(ValueError, match="No .gguf files found"):
    list_repo_files("test/repo")
```
(`tests/test_download_model.py:103`)

**Rust error/state testing:**
```rust
assert!(store.verify_chain().unwrap());          // success path
// tamper with DB...
assert!(!store.verify_chain().unwrap());         // failure path
```
(`agent-rs/tests/audit_log_mvp.rs:73-79`)

**State-Mutation Testing (start_server):**
```python
config.BATCH_SIZE = 512    # reset to known state
start_server.apply_runtime_overrides()
assert config.BATCH_SIZE == 128
```
(`tests/test_start_server_runtime_profile.py:31-38`)

**Snapshot Testing:** Not used.

---

*Testing analysis: 2026-08-03*
*Update when test patterns change*

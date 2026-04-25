---
status: resolved
trigger: "Investigate and fix issue: model-install-test-failures"
created: 2025-01-24T12:00:00Z
updated: 2025-01-24T12:20:00Z
---

## Current Focus

hypothesis: Tests need to be updated to match the new model registry and security requirements (pinned revisions).
test: Update tests/test_model_install.py and run pytest.
expecting: Tests pass.
next_action: resolved

## Symptoms

expected: tests/test_model_install.py passes all tests.
actual: test_resolve_model_ref and test_install_model_spec_success fail.
errors: 
  - AssertionError: assert None is not None (where None = resolve_model_ref('gpt-oss-20b'))
  - AssertionError: assert False is True (captured error: [!] Installation failed: Good Model is blocked: pin a concrete Hugging Face revision before installation.)
reproduction: pytest tests/test_model_install.py
started: Failure occurred after Phase 01 pivot (Qwen 3.6 integration and security sandbox).

## Eliminated

## Evidence

- timestamp: 2025-01-24T12:05:00Z
  checked: tests/test_model_install.py and scripts/model_install.py
  found: 
    - `TRUSTED_MODELS` in `scripts/model_install.py` only contains Qwen models.
    - `test_resolve_model_ref` uses `gpt-oss-20b` which is missing from `TRUSTED_MODELS`.
    - `install_model_spec` now calls `validate_trusted_model_spec` which requires a pinned `revision` and a 64-char `sha256`.
    - `test_install_model_spec_success` provides a `spec` without a `revision`, causing it to fail validation.
  implication: Tests are outdated relative to recent security hardening and model registry changes.

- timestamp: 2025-01-24T12:12:00Z
  checked: pytest tests/test_model_install.py after updates
  found: All tests passed.
  implication: Updated test data correctly matches current implementation requirements.

## Resolution

root_cause: Tests in `tests/test_model_install.py` were not updated after the security pivot that introduced strict revision pinning and SHA256 validation in `scripts/model_install.py`, and after the trusted model registry was updated to Qwen-only.
fix: Updated `tests/test_model_install.py` to use `qwen-3.6-27b-moe` for resolution tests and added valid mock `revision` and 64-char `sha256` to the test model specs.
verification: Ran `pytest tests/test_model_install.py` and confirmed all 4 tests pass.
files_changed: [tests/test_model_install.py]

# Phase 21-01 Summary: Model Integrity and Install Automation

## Changes

### 1. Shared Trusted Model Installer
Created `scripts/model_install.py` as the single source of truth for model installation.
- Implemented `TRUSTED_MODELS` registry with pinned revisions and checksum placeholders.
- Added `verify_model_integrity` using SHA256.
- Added `download_model_to_staging` with `huggingface_hub` and `requests` fallback.
- Added `activate_model` to move verified models into the final directory.
- Exposed `install_model` and `install_model_spec` for CLI and programmatic use.

### 2. Unified Install Path
Refactored existing entrypoints to use the shared installer:
- **`setup.py`**: Now routes all model downloads through `install_model_spec`. It still performs hardware detection but delegates the download and verification.
- **`scripts/download_model.py`**: Now acts as a compatibility adapter that uses the shared install logic while preserving its interactive quantization selection.

### 3. New CLI Entrypoint
Created `scripts/helix.py` to provide a user-friendly CLI.
- Supports `helix install <model>` for quick installation of trusted models.
- Supports `helix list-models` to show available trusted models.

### 4. Regression Tests
Created `tests/test_model_install.py` covering:
- Checksum verification logic.
- Model reference resolution.
- Success and failure (tamper detection) scenarios for model installation.

## Verification Results
- All modified files passed `python -m py_compile`.
- New tests in `tests/test_model_install.py` passed with 100% success rate.
- Manual verification of `setup.py --help`, `scripts/helix.py --help`, and `scripts/download_model.py -h` confirmed no regressions in CLI availability.

## Threat Model Mitigation
- **T-21-01 (Untrusted References):** Mitigated by the `TRUSTED_MODELS` registry and pinned revisions.
- **T-21-02 (Tampered Models):** Mitigated by SHA256 verification before activation.
- **T-21-03 (Filename Handling):** Mitigated by `activate_model` which ensures files are placed only within the `models/` directory using safe basenames.

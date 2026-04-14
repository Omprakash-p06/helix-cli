# Phase 21: Model Integrity and Install Automation - Research

**Researched:** 2026-04-13
**Domain:** Model supply-chain integrity, hardware-aware installation, Hugging Face Hub metadata
**Confidence:** HIGH (installer architecture), MEDIUM (trusted manifest source shape)

## Summary

Phase 21 should harden the model setup pipeline from "download whatever the user picked" to "only activate a model after it is verified against trusted metadata" while preserving the repo's existing one-stop installer flow.

The current code already has the right architectural center of gravity: `setup.py` performs hardware detection, model selection, backend tuning, downloads, Rust build, llama.cpp build, and preflight validation. `scripts/download_model.py` is a separate downloader, but it is structurally weaker and should not become a second installer. The right move is to make `setup.py` the single source of truth for installation and add a thin `helix install <model>` entrypoint that delegates into it.

**Primary recommendation:**
1. Keep `setup.py` as the canonical installer/orchestrator.
2. Add a small `helix install <model>` wrapper that reuses the same setup functions instead of duplicating install logic.
3. Verify model downloads before activation using pinned revisions plus checksum allowlists and trusted source metadata.
4. Prefer the Hugging Face Hub client APIs for metadata and downloads instead of raw tree scraping and ad hoc URL construction.

## Standard Stack

### Core (Use)
| Library / Module | Purpose | Why |
|---|---|---|
| `huggingface_hub` | Model metadata, revision pinning, downloads, checksum verification | Exposes `model_info`, `repo_info`, `hf_hub_download`, `snapshot_download`, and `verify_repo_checksums` so the installer can trust Hub metadata instead of guessing |
| Existing `requests` + `tqdm` | Bootstrap downloads only if needed during migration | Already present in the repo, but should not remain the primary trust mechanism |
| `hashlib` | Local SHA-256 verification against allowlist | Deterministic, cross-platform, no new dependency needed |
| `json` / `tomllib` or `yaml` equivalent | Trusted manifest loading | Model allowlist needs a machine-readable manifest |
| `pathlib` | Path handling across Windows/Linux | Keeps installer paths OS-agnostic |
| `argparse` | `helix install <model>` dispatch | Fits the existing CLI-style workflow |

### Optional (Only if needed)
| Library | Use case |
|---|---|
| `platformdirs` | Standard cache location for downloaded models and manifests |
| `rich` | Polished installer progress and warnings |

## Architecture Patterns

### Pattern 1: Single Installer Source of Truth

**What:** Keep all install orchestration in one pipeline.

**Recommendation:**
- Treat `setup.py` as the authoritative implementation for detection, model selection, backend selection, download, verification, and config generation.
- Add a thin `helix` wrapper that forwards `install` into the same shared functions.
- Do not fork logic into a second downloader path that drifts from `setup.py`.

**Why:** The repo already has a unified setup flow, and the Phase 21 goal is to make that flow easier to invoke, not to replace it.

### Pattern 2: Verify Before Activation

**What:** Only write config and mark the model active after the model is verified.

**Recommended sequence:**
1. Resolve model identity and revision from the user choice or curated catalog.
2. Fetch trusted metadata from Hugging Face.
3. Download the model file into a staging path.
4. Compute SHA-256 locally.
5. Compare against a trusted allowlist or manifest entry.
6. Move the file into the active models directory only after a match.
7. Update `scripts/config.py` only after verification succeeds.

**Why:** A downloaded file should never become the runtime model until the integrity check passes.

### Pattern 3: Pinned Revision + Trusted Metadata

**What:** Model selection must be revision-aware, not just repo-aware.

**Recommendation:**
- Pin downloads to a specific revision, tag, or commit hash.
- Read repository metadata with `huggingface_hub.model_info()` or `repo_info()`.
- Use `snapshot_download()` or `hf_hub_download()` for the actual fetch, not manual `requests.get()` URL assembly.
- Use `verify_repo_checksums()` where available to validate local files against Hub checksums.

**Why:** The Hub docs explicitly support revision pinning, snapshot downloads, file metadata, and checksum verification. This is the correct trust boundary for model setup.

### Pattern 4: Hardware-Aware Quantization Selection

**What:** Keep the current hardware detection flow, but make model install choices explicit and repeatable.

**Recommendation:**
- Preserve the existing auto-tuning logic in `setup.py` for backend and quantization selection.
- Let the install command recommend a quantization based on detected RAM/VRAM/CPU tier.
- If multiple model artifacts are available, choose the one that matches the detected hardware tier and the trusted manifest.

**Why:** The current setup flow already contains tier-based tuning. Phase 21 should extend that behavior, not invent a separate model picker.

### Pattern 5: Two-Layer Trust Model

**What:** Separate source trust from file integrity trust.

**Recommendation:**
- Source trust: only accept model repos or files from a curated allowlist of repo IDs and revisions.
- File trust: verify SHA-256 after download before activation.
- If a repo is trusted but the hash does not match, fail closed.

**Why:** A known repo can still change over time. A hash check alone is not enough if the source can drift.

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Repo inspection | raw tree scraping with manual filtering | `huggingface_hub.model_info()` / `repo_info()` / `get_paths_info()` | Gives structured metadata and file information |
| File download | ad hoc `requests.get()` for every model | `hf_hub_download()` / `snapshot_download()` | Handles caching, revisions, and retries more safely |
| Integrity checks | custom "trust me" logic around filenames | SHA-256 allowlist + `verify_repo_checksums()` | Model files need deterministic verification |
| Installer routing | a second standalone setup stack | thin `helix install` wrapper calling `setup.py` internals | Prevents drift and duplicated policy |
| Activation | immediately pointing config at any downloaded file | staging directory + post-verify rename/config write | Avoids activating an untrusted file |

## Common Pitfalls

### 1) Verifying after activation
If `scripts/config.py` or runtime state is updated before integrity passes, the system can boot an untrusted file.

**Avoid:** staging downloads first, then write config only after verification succeeds.

### 2) Trusting the latest branch by default
A repo root can change even if the repo name stays the same.

**Avoid:** require pinned revisions or explicit manifest entries for active models.

### 3) Using the downloader as the installer
`scripts/download_model.py` is useful as a downloader, but it does not manage backend selection, validation, or config generation.

**Avoid:** building a second install path on top of it.

### 4) Treating file presence as proof of validity
A `.gguf` file existing in `models/` only proves that a file exists, not that it is trusted.

**Avoid:** the current Phase 3-style "place the file and it will run" assumption for model activation.

### 5) One manifest for all quantizations
Different quantizations are different files and need their own hashes.

**Avoid:** sharing a single hash entry across multiple model variants.

### 6) OS-specific path assumptions
Installer paths must work on Windows and Linux without special-casing the user flow.

**Avoid:** hardcoded shell path logic; use `pathlib` and the existing platform checks already present in `setup.py`.

## Code Examples

### Example 1: Shared installer entrypoint

```python
# helix install should call the same functions used by setup.py

def install_model(model_ref: str) -> None:
    specs = detect_specs()
    selected_model = resolve_model_reference(model_ref, specs)
    downloaded_path = download_model_to_staging(selected_model)
    verify_model_integrity(selected_model, downloaded_path)
    activate_model(selected_model, downloaded_path)
    finalize_runtime_config(selected_model, specs)
```

### Example 2: Integrity gate

```python
import hashlib
from pathlib import Path


def verify_model_integrity(expected_sha256: str, model_path: Path) -> None:
    digest = hashlib.sha256(model_path.read_bytes()).hexdigest()
    if digest != expected_sha256:
        raise ValueError(
            f"Checksum mismatch for {model_path.name}: expected {expected_sha256}, got {digest}"
        )
```

### Example 3: Hub-backed download flow

```python
from huggingface_hub import hf_hub_download, model_info


def fetch_trusted_model(repo_id: str, revision: str, filename: str, cache_dir: Path) -> Path:
    info = model_info(repo_id, revision=revision, files_metadata=True)
    # Use the returned metadata to confirm the repo/revision matches the allowlist.
    return Path(
        hf_hub_download(
            repo_id=repo_id,
            filename=filename,
            revision=revision,
            cache_dir=cache_dir,
            local_dir=cache_dir,
        )
    )
```

### Example 4: Minimal `helix install` dispatch

```python
parser = argparse.ArgumentParser()
subparsers = parser.add_subparsers(dest="command", required=True)
install_parser = subparsers.add_parser("install")
install_parser.add_argument("model")

args = parser.parse_args()
if args.command == "install":
    install_model(args.model)
```

## Source Evidence

- `setup.py` already performs the full installer orchestration: hardware detection, model selection, download, Rust build, llama.cpp build, config generation, speed gate, and benchmark preflight.
- `setup.py` currently defines `download_file`, `choose_models`, `build_llama_cpp`, `enforce_token_speed`, `generate_config`, and `main`, which are the natural seams for Phase 21.
- `scripts/download_model.py` downloads GGUF files from Hugging Face directly and treats file placement in `models/` as sufficient for activation.
- `scripts/config.py` is runtime state, not a trusted-install contract; it should only be written after verification succeeds.
- Hugging Face docs confirm the trust primitives needed here: `model_info()`, `repo_info()`, `get_paths_info()`, `hf_hub_download()`, `snapshot_download()`, and `verify_repo_checksums()`.
- Hugging Face model docs also show repository revisions, file metadata, and security fields that can anchor a trusted manifest strategy.

## Implementation Readiness

Ready for `/gsd-plan-phase 21`.

Planners should structure tasks in this order:
1. Define the trusted model manifest format and installer contracts.
2. Implement Hub-backed download and checksum verification before activation.
3. Add the `helix install <model>` entrypoint and wire it into the existing setup flow.
4. Add regression tests for revision pinning, checksum mismatch handling, and cross-platform install behavior.

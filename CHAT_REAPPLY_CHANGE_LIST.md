# Chat Reapply Change List

This document summarizes the code changes made during this chat so you can re-implement them on top of your original code.

## 1) setup.py hardening changes

### 1.1 Windows elevation and installer reliability
- Add early Windows admin check and UAC relaunch logic at startup.
- Ensure relaunch executes setup script correctly (absolute script path + proper working directory).
- Final reliable relaunch approach used elevated `cmd.exe` to avoid Python REPL launch edge cases.

### 1.2 Rust toolchain / linker reliability on Windows
- Add Visual C++ Build Tools detection via `vswhere`.
- If missing, attempt install with `winget` (BuildTools + C++ workload).
- Build Rust under VS environment (`vcvars64.bat`) so `link.exe` is always available.
- Avoid fragile inline cmd quoting by creating and running a temporary `.cmd` launcher:
  - `call vcvars64.bat`
  - `cargo build`

### 1.3 CUDA availability and fallback behavior
- Add `nvcc` detection helper.
- If backend recommendation is CUDA but `nvcc` missing:
  - attempt auto-install CUDA Toolkit via `winget` (Windows)
  - refresh env (`PATH`, `CUDA_HOME`, `CUDA_PATH`) in-process
  - fallback chain if still unavailable: CUDA -> Vulkan -> CPU

### 1.4 llama.cpp build robustness
- Build backend fallback chain in script:
  - CUDA -> Vulkan -> CPU
  - Vulkan -> CPU
  - OpenVINO -> CPU
- Clear CMake cache (`CMakeCache.txt`, `CMakeFiles`) between backend attempts.
- Keep informative output for long native compile behavior (100% CPU expected first build).

### 1.5 Windows binary path and DLL runtime fixes
- Add robust binary resolver for `llama-server`:
  - Prefer Windows Release path first (`build/bin/Release/llama-server.exe`)
  - Also check `build/bin`, Debug, and recursive fallback under build tree.
- Stage required runtime DLLs from Release to runtime binary directory when needed:
  - `ggml-base.dll`, `ggml-cpu.dll`, `ggml-cuda.dll`, `ggml.dll`, `llama.dll`, `mtmd.dll`
- Launch model server with:
  - `cwd` = binary directory
  - `PATH` prepended with binary directory
  to ensure Windows loader finds DLLs.

### 1.6 Benchmark startup diagnostics
- Replace silent `DEVNULL` launch with file logs in `logs/`.
- Add process-aware startup wait:
  - detect early process exit
  - return explicit reason
- On startup failure, include:
  - full command
  - stdout/stderr log file paths
  - recent log tail
- Add dynamic port fallback if 8080/8082 already in use.

### 1.7 Token benchmark request correctness
- Query `/v1/models` and use actual loaded model id for completion request.
- Add HTTP status checks (`raise_for_status`) before JSON parsing.

### 1.8 GPU layer behavior and token gate
- Avoid adding CUDA-specific flags if `gpu_layers == 0`.
- Add CUDA `gpu_layers` auto-tuning for token benchmark gate:
  - benchmark multiple candidates instead of a single static value
  - choose best-performing candidate
  - keep hard threshold: fail if `< 10 tok/s`
- Persist tuned runtime values by regenerating config after benchmark selection.

### 1.9 Config generation sequencing
- Generate config after final backend/build outcome (not too early).
- Ensure backend fallback result and final `gpu_layers` are reflected in config.

## 2) system_check.py scoring/detection fixes

Changes were applied in both files:
- `system_check.py`
- `scripts/system_check.py` (when present)

### 2.1 Windows CPU feature under-detection fix
- On Windows, infer conservative SIMD features from model name when WMIC does not expose flags.
- For Ryzen, infer at least: `sse4_2`, `f16c`, `fma`, `avx`, `avx2`.
- This prevents false low `cpu_arch` score (e.g., 1/5 on capable Ryzen CPUs).

### 2.2 AMD iGPU detection expansion
- Broaden Windows iGPU matching for Radeon integrated names, including variants like:
  - `Radeon(TM) Graphics`
  - `AMD Radeon Graphics`
- Keep simple guard to avoid misclassifying discrete RX GPUs as iGPU.

## 3) Qwen default model replacement (Q8 -> Q4_K_M)

### 3.1 setup.py model registry
- Replace Qwen default URL and filename everywhere:
  - Old: `Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf`
  - New: `Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf`
- Use direct HF URL with `?download=true` as requested:
  - `https://huggingface.co/HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive/resolve/main/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf?download=true`

### 3.2 recommended files in system check configs
- Update all `recommended_file` entries for Qwen tiers from Q8 filename to Q4_K_M filename.

### 3.3 docs updates
- Update README examples and roadmap/plan references from Q8 to Q4_K_M links/filenames.

## 4) .gitignore changes

- Expand ignore rules to prevent generated artifacts from being tracked:
  - `agent-rs/target/`
  - `llama.cpp/build/`
  - `logs/`, `models/`, `venv/`, `__pycache__/`
  - `*.pyc`, `*.pyo`, `*.pyd`, `*.zip`

## 5) Git operational changes made during chat (non-code)

These were process actions, not product code changes:
- Removed tracked build artifacts from index.
- Rewrote history to purge large generated binaries from commits.
- Cleaned refs/original + gc to drop old large blobs.
- Reset to `origin/main` on request.

If you are restoring from original code, you can ignore this section unless you also need large-file push cleanup again.

## 6) Reapply order recommendation

1. Reapply `setup.py` reliability changes first.
2. Reapply `system_check.py` detection/scoring changes.
3. Reapply Qwen Q4_K_M replacement in setup + recommendations.
4. Reapply `.gitignore` rules.
5. Re-run setup and validate token gate behavior.

## 7) Minimal validation checklist

- Setup starts and stays in script after UAC prompt (Windows).
- Rust build works without `link.exe not found`.
- llama.cpp builds even if CUDA path is missing (fallback works).
- `llama-server.exe` launches from Windows Release layout.
- No DLL popup errors (`ggml*.dll`, `llama.dll`, `mtmd.dll`).
- Token benchmark prints candidate trials and fails only if truly `< 10 tok/s`.
- Qwen default points to `Q4_K_M` file.
- `git status` does not include `agent-rs/target` or `llama.cpp/build` changes.

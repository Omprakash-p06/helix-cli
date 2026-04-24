---
phase: "01"
plan: "01"
subsystem: "foundation"
tags: ["qwen", "configuration", "verification"]
requires: []
provides: ["inference-readiness"]
affects: ["scripts/config.py", "scripts/model_install.py", "scripts/system_check.py"]
tech-stack: ["python", "llama.cpp", "qwen-3.6"]
key-files:
  - "scripts/config.py"
  - "scripts/model_install.py"
  - "scripts/system_check.py"
decisions:
  - "Use Qwen 3.6 27B/35B MoE as the primary model foundation."
  - "Implement tiered VRAM-aware quantization selection logic (Q4_K_M for <12GB, Q5_K_M for <24GB, Q8_0 for 24GB+)."
metrics:
  duration: "00:10:00"
  completed_date: "2026-04-24"
---

# Phase 01 Plan 01: Qwen Foundation Summary

## One-liner
Established Qwen 3.6 as the core model foundation with hardware-aware quantization selection and system readiness verification.

## Accomplishments

### 1. Updated Model Configuration for Qwen 3.6
- Updated `scripts/config.py` to set `MODEL_NAME` to "Qwen-3.6-27B-MoE".
- Implemented `MODEL_CATALOG` with variants for 27B and 35B MoE models.
- Added tiered logic in `_variant_for_model` to select quantization and GPU layers based on detected VRAM.
- Updated `CHAT_SYSTEM_PROMPT` and `AGENTIC_SYSTEM_PROMPT` for Qwen 3.6 alignment.

### 2. Updated Model Registry in Install Script
- Added `qwen-3.6-27b-moe` and `qwen-3.6-35b-moe` to `TRUSTED_MODELS` in `scripts/model_install.py`.
- Pinned to "Qwen/Qwen3.6-27B-Instruct-GGUF" and "Qwen/Qwen3.6-35B-Instruct-GGUF" repositories.
- Note: Models are currently in a "blocked" state in the registry until verified revisions and SHA256 hashes are available.

### 3. Implemented System Check Utility
- Created `scripts/system_check.py` to verify:
    - Docker daemon availability.
    - Model file presence.
    - llama.cpp binary executability and basic smoke test.
    - VRAM detection and quantization advice.
- Provides a clear "Green/Red" report for agent readiness.

## Verification Results

### Automated Tests
- `python3 -c "import scripts.config as c; print(c.MODEL_NAME)"` -> `Qwen-3.6-27B-MoE` (PASSED)
- `python3 scripts/model_install.py --list-models | grep "qwen-3.6"` -> Registry entries found (PASSED)
- `python3 scripts/system_check.py` -> Runs and reports status (PASSED)

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs
- `scripts/model_install.py`: `TRUSTED_MODELS` entries for Qwen 3.6 have `revision: "UNVERIFIED_REVISION"` and `sha256: None`. These are intentional stubs as the real model artifacts were not available at the time of implementation, effectively blocking installation until real metadata is provided.

## Self-Check: PASSED
- [x] Configuration updated to Qwen 3.6
- [x] Model registry supports SOTA MoE variants
- [x] System check accurately identifies hardware constraints
- [x] Commits `dfd2f04`, `ed0ed08`, `c01a330` verify the work

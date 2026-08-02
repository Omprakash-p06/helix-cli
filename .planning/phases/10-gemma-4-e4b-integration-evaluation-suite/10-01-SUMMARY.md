---
phase: 10-gemma-4-e4b-integration-evaluation-suite
plan: 10-01
subsystem: model-infra
tags: [gemma, llama-cpp, gguf, huggingface, vram-tiering, multimodal, jinja]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: Multi-model foundation loader and model catalog structure
provides:
  - Gemma-4-E4B VRAM-tiered catalog entry (3 variants: 8GB Q8_0, 4GB Q4_K_M, CPU fallback) with jinja + mmproj_filename metadata
  - llama-server --jinja / --mmproj flag injection driven by model profile metadata
  - build_model_entry passthrough of jinja/mmproj_filename fields
  - Gemma 4 E4B download presets (Q4_K_M, Q8_0, mmproj) with --preset CLI wiring
affects: [10-02, 10-03, 10-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Optional variant metadata (jinja, mmproj_filename) flows from static catalog through build_model_entry to server launch"
    - "Named model presets registered in a MODEL_PRESETS collection with a --preset CLI entry point"

key-files:
  created: []
  modified:
    - scripts/config.py
    - scripts/start_server.py
    - scripts/download_model.py
    - tests/test_qwen_config.py

key-decisions:
  - "Added jinja/mmproj_filename passthrough to build_model_entry (plan asserted it already existed)"
  - "Created MODEL_PRESETS collection in download_model.py from scratch (plan assumed a preset registry existed)"
  - "Wired minimal --preset CLI flag so presets are actually downloadable, not dead data"

patterns-established:
  - "Flag injection pattern: read model profile via build_model_entry, conditionally append backend flags before subprocess launch"
  - "Preset lookup pattern: MODEL_PRESETS + find_model_preset(name) returning None for unknown names"

requirements-completed: []

# Coverage metadata (#1602) — one entry per shipped deliverable.
coverage:
  - id: D1
    description: "Gemma-4-E4B catalog entry in scripts/config.py with 3 VRAM-tiered variants (8GB Q8_0 / 4GB Q4_K_M / CPU fallback), each carrying jinja=True and mmproj_filename=mmproj-F16.gguf"
    verification:
      - kind: unit
        ref: "tests/test_qwen_config.py#test_gemma_4_e4b_catalog_entry"
        status: pass
      - kind: unit
        ref: "tests/test_qwen_config.py#test_gemma_4_e4b_variant_selection_by_vram"
        status: pass
      - kind: other
        ref: "python -c \"from scripts.config import MODEL_CATALOG; print(MODEL_CATALOG['Gemma-4-E4B'])\""
        status: pass
    human_judgment: false
  - id: D2
    description: "start_server.py run_llama_server appends --jinja when profile has jinja=True and --mmproj <path> when mmproj file exists (graceful warning otherwise)"
    verification:
      - kind: unit
        ref: "tests/test_start_server_runtime_profile.py"
        status: pass
      - kind: unit
        ref: "tests/test_start_server_runtime_profile_validation.py"
        status: pass
      - kind: other
        ref: "functional simulation: HELIX_MODEL_NAME=Gemma-4-E4B -> cmd contains --jinja; mmproj missing -> warning printed"
        status: pass
    human_judgment: false
  - id: D3
    description: "download_model.py registers Gemma 4 E4B presets (Q4_K_M, Q8_0, mmproj) from unsloth/gemma-4-E4B-it-GGUF and supports --preset <name> downloads"
    verification:
      - kind: unit
        ref: "tests/test_download_model.py"
        status: pass
      - kind: other
        ref: "mocked --preset Gemma-4-E4B-Q8_0 CLI run -> download_file called with (unsloth/gemma-4-E4B-it-GGUF, gemma-4-E4B-it-Q8_0.gguf); unknown preset -> exit 1"
        status: pass
    human_judgment: false

# Metrics
duration: 4min
completed: 2026-08-03
status: complete
---

# Phase 10 Plan 1: Gemma 4 E4B Model Catalog, Server Flags & Downloader Preset Summary

**Gemma 4 E4B-it wired into the Python model stack: 3 VRAM-tiered catalog variants with jinja/mmproj metadata, llama-server --jinja/--mmproj flag injection, and 3 downloadable HF presets with a --preset CLI flag**

## Performance

- **Duration:** 4 min
- **Started:** 2026-08-03T02:03:30Z
- **Completed:** 2026-08-03T02:07:39Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Added `Gemma-4-E4B` catalog entry to `_static_model_catalog()` with 3 VRAM-tiered variants (8GB Q8_0 full offload, 4GB Q4_K_M consumer-GPU, 0GB CPU fallback), each with `jinja: True` and `mmproj_filename: mmproj-F16.gguf`
- Extended `build_model_entry()` to pass through the new optional `jinja` / `mmproj_filename` variant fields (default-safe: `False` / `None` for models without them)
- Extended `run_llama_server()` in start_server.py to append `--jinja` and `--mmproj <path>` from the resolved model profile, with graceful degradation when the mmproj file is absent; koboldcpp path untouched (its own function, no guard needed)
- Created `MODEL_PRESETS` collection in download_model.py (Q4_K_M, Q8_0, mmproj presets from `unsloth/gemma-4-E4B-it-GGUF`) plus `find_model_preset()` / `list_model_presets()` helpers and a `--preset` CLI flag that reuses the existing verified download flow
- Updated `test_qwen_config.py` catalog-set assertion and added Gemma variant tests (3 tests: catalog entry structure, VRAM selection, repo_alias)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Gemma 4 E4B catalog entry to config.py** - `d6d57ea` (feat)
2. **Task 2: Extend start_server.py to pass --jinja and --mmproj flags** - `6e38555` (feat)
3. **Task 3: Add Gemma 4 E4B downloadable presets in download_model.py** - `fc93739` (feat)

## Files Created/Modified
- `scripts/config.py` - Gemma-4-E4B catalog entry (3 variants); jinja/mmproj_filename passthrough in build_model_entry
- `scripts/start_server.py` - model-profile-driven --jinja / --mmproj flag injection in run_llama_server
- `scripts/download_model.py` - MODEL_PRESETS collection, preset helpers, --preset CLI path
- `tests/test_qwen_config.py` - updated catalog set assertion; added 2 Gemma catalog/variant tests

## Decisions Made
- **jinja/mmproj_filename passthrough added to build_model_entry:** the plan's T1 note asserted `build_model_entry()` already passes these fields through, but the actual return dict is a fixed key list. Without the passthrough, T2's flag logic would be dead code and the must_haves truths ("passes --jinja when model profile has jinja=True") would be false. Added default-safe passthrough (`variant.get("jinja", False)`, `variant.get("mmproj_filename")`).
- **MODEL_PRESETS created from scratch:** the plan's T3 instruction ("match the exact key names used by the existing preset entries") assumed a preset registry existed in download_model.py; the file is a pure interactive wizard with no preset collection. Created `MODEL_PRESETS` with the plan's exact entries and key names (`name`, `hf_repo`, `filename`, `description`).
- **Minimal --preset CLI wiring:** to satisfy the must_haves truth that Gemma 4 E4B is a *downloadable* preset (not dead data), added a `--preset <name>` flag that resolves via `find_model_preset()` and reuses the untouched `download_file()`/`mutate_config()` flow. Interactive wizard behavior unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Stale catalog-set assertion in test_qwen_config.py**
- **Found during:** Task 1 (Add Gemma catalog entry)
- **Issue:** `test_qwen_catalog_exposes_expected_models` asserted `set(config.MODEL_CATALOG) == {"Qwen-3.6-27B-MoE", "Qwen-3.6-35B-MoE"}` — adding Gemma-4-E4B makes it fail, contradicting the plan's own acceptance criteria (`pytest tests/test_qwen_config.py -q` exits 0). Plan did not list this test file for modification.
- **Fix:** Updated the assertion to include `"Gemma-4-E4B"` and added `test_gemma_4_e4b_catalog_entry` + `test_gemma_4_e4b_variant_selection_by_vram` to lock in the new entry's structure.
- **Files modified:** tests/test_qwen_config.py
- **Verification:** `pytest tests/test_qwen_config.py -q` → 23 passed
- **Committed in:** d6d57ea (Task 1 commit)

**2. [Rule 2 - Missing Critical] build_model_entry() missing jinja/mmproj_filename passthrough**
- **Found during:** Task 2 (start_server.py flag injection)
- **Issue:** Plan's T1 note claimed "existing `build_model_entry()` passes them through via the variant dict" — it does not (fixed key list). T2's `model_profile.get("jinja")` / `.get("mmproj_filename")` would never fire, making the flags dead code and violating must_haves truths.
- **Fix:** Added default-safe passthrough in `build_model_entry()` return dict: `"jinja": variant.get("jinja", False)`, `"mmproj_filename": variant.get("mmproj_filename")`. Qwen models unaffected (False/None).
- **Files modified:** scripts/config.py
- **Verification:** `build_model_entry("Gemma-4-E4B", 8)` → jinja True, mmproj set; `build_model_entry("Qwen-3.6-27B-MoE", 12)` → jinja False, mmproj None; all runtime-profile tests pass
- **Committed in:** 6e38555 (Task 2 commit)

**3. [Rule 2 - Missing Critical] download_model.py had no preset collection**
- **Found during:** Task 3 (Gemma presets)
- **Issue:** Plan instructed to add entries "matching existing preset entries," but download_model.py contains no preset registry — it is a repo-URL-driven wizard.
- **Fix:** Created `MODEL_PRESETS` list with the plan's exact 3 Gemma entries, added `find_model_preset()` / `list_model_presets()` helpers, and wired a `--preset` CLI flag (reuses existing `download_file()`; wizard logic untouched) so the presets are actually downloadable per the must_haves truth.
- **Files modified:** scripts/download_model.py
- **Verification:** mocked `--preset Gemma-4-E4B-Q8_0` → `download_file` called with `(unsloth/gemma-4-E4B-it-GGUF, gemma-4-E4B-it-Q8_0.gguf)`; unknown preset → exit 1 with preset list; `pytest tests/test_download_model.py -q` → 15 passed
- **Committed in:** fc93739 (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 2 missing critical)
**Impact on plan:** All three deviations were necessary for the plan's own acceptance criteria and must_haves truths to hold. No scope creep; the --preset wiring is the minimal consumer path for the presets the plan required.

## Issues Encountered
- `rg` (ripgrep) not available on this Windows host — used the Grep tool and PowerShell alternatives for string-presence checks.
- None other; all plan verification commands passed on first run after the deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Wave 1 prerequisite complete: the Rust side (10-02) can now read `MODEL_CATALOG["Gemma-4-E4B"]` for capability probing (jinja/mmproj presence per variant tier)
- llama-server will receive `--jinja`/`--mmproj` automatically when `HELIX_MODEL_NAME=Gemma-4-E4B` (or any profile with jinja=True) is active and the mmproj file exists in `models/`
- Downloading `Gemma-4-E4B-mmproj` via `python scripts/download_model.py --preset Gemma-4-E4B-mmproj` places `mmproj-F16.gguf` in `models/`, enabling the vision path end-to-end

---
*Phase: 10-gemma-4-e4b-integration-evaluation-suite*
*Completed: 2026-08-03*

## Self-Check: PASSED

- `scripts/config.py` — modified (Gemma-4-E4B catalog + passthrough)
- `scripts/start_server.py` — modified (--jinja / --mmproj injection)
- `scripts/download_model.py` — modified (MODEL_PRESETS + --preset CLI)
- `tests/test_qwen_config.py` — modified (catalog assertion + Gemma tests)
- `d6d57ea` — commit found (Task 1)
- `6e38555` — commit found (Task 2)
- `fc93739` — commit found (Task 3)
- Full suite `pytest tests/ -q` → 52 passed (must_haves truth)

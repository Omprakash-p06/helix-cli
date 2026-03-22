# Phase 6: Memory Management & Fallback - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning
**Source:** User Directive (v1.1 Initialization)

<domain>
## Phase Boundary

This phase introduces critical hardware resource management to the Python `start.py` layer and the backend inference wrappers. It ensures the environment is clean before booting and provides a safety net for hardware with limited VRAM.

</domain>

<decisions>
## Implementation Decisions

### Process Management
- `start.py` must aggressively hunt down and kill any lingering `llama-server.exe` or `koboldcpp.exe` processes before attempting a new boot. 
- This prevents the "port already in use" and VRAM exhaustion errors caused by orphaned inference engines.

### iGPU Fallback
- If the primary boot on the dGPU fails (e.g., due to an Out of Memory crash when allocating tensors), the backend scripts (`scripts/start_server.py`) must catch this failure.
- Upon catching the failure, the script should automatically retry the boot process targeting the integrated GPU (iGPU) or system RAM, possibly by adjusting `--tensor-split`, `--split-mode`, or `--vulkan-device` arguments depending on the backend.

</decisions>

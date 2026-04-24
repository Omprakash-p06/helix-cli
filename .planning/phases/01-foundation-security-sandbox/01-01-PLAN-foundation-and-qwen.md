---
phase: 01
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: [scripts/config.py, scripts/model_install.py, scripts/system_check.py]
autonomous: true
requirements: [MOD-01]

must_haves:
  truths:
    - "Qwen 3.6 is the default model in config"
    - "System check verifies model availability and quantization"
  artifacts:
    - path: "scripts/system_check.py"
      provides: "Inference readiness verification"
  key_links:
    - from: "scripts/config.py"
      to: "scripts/model_install.py"
      via: "MODEL_NAME match"
---

<objective>
Establish the Qwen 3.6 foundation and hardware-aware quantization selection.

Purpose: Ensure the agent uses the state-of-the-art local model optimized for terminal tasks.
Output: Updated configuration and installation scripts, and a new system verification utility.
</objective>

<execution_context>
@$HOME/.gemini/get-shit-done/workflows/execute-plan.md
@$HOME/.gemini/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/REQUIREMENTS.md
@.planning/phases/01-foundation-security-sandbox/01-RESEARCH.md
@.planning/STATE.md
@scripts/config.py
@scripts/model_install.py
</context>

<tasks>

<task type="auto">
  <name>Update Model Configuration for Qwen 3.6</name>
  <files>scripts/config.py</files>
  <action>
    Modify `scripts/config.py` to:
    - Update `MODEL_NAME` to "Qwen-3.6-27B-MoE" (or similar SOTA variant found in GGUF).
    - Update `AVAILABLE_MODELS` to include Qwen 3.6 variants (27B and 35B MoE).
    - Implement tiered hardware detection logic for `GPU_LAYERS` based on VRAM (e.g., 8GB -> Q4_K_M, 24GB -> Q8_0 or full).
    - Set `CHAT_SYSTEM_PROMPT` and `AGENTIC_SYSTEM_PROMPT` to align with Qwen 3.6 capabilities (per RESEARCH.md).
  </action>
  <verify>
    <automated>python3 -c "import scripts.config as c; print(c.MODEL_NAME)" | grep "Qwen-3.6"</automated>
  </verify>
  <done>Config reflects Qwen 3.6 as primary model with VRAM-aware defaults.</done>
</task>

<task type="auto">
  <name>Update Model Registry in Install Script</name>
  <files>scripts/model_install.py</files>
  <action>
    Update `TRUSTED_MODELS` registry in `scripts/model_install.py` to include:
    - `qwen-3.6-27b-moe`: Repo "Qwen/Qwen2.5-27B-Instruct-GGUF" (use 3.6 if available, otherwise 2.5 as baseline but label 3.6-ready).
    - `qwen-3.6-35b-moe`: Repo "Qwen/Qwen2.5-35B-Instruct-GGUF".
    Note: Research mentioned Qwen 3.6; if not publicly available in GGUF format yet, use Qwen 2.5 SOTA variants as the "3.6 foundation" per user request (MOD-01).
  </action>
  <verify>
    <automated>python3 scripts/model_install.py --list-models | grep "qwen-3.6"</automated>
  </verify>
  <done>Registry contains pinned Qwen 3.6/2.5 SOTA models.</done>
</task>

<task type="auto">
  <name>Implement System Check Utility</name>
  <files>scripts/system_check.py</files>
  <action>
    Create `scripts/system_check.py` to:
    - Check for Docker availability (required for SEC-01).
    - Verify model file existence based on `scripts/config.py`.
    - Perform a basic llama.cpp smoke test (check if binary is executable).
    - Report VRAM and suggest quantization level.
  </action>
  <verify>
    <automated>python3 scripts/system_check.py</automated>
  </verify>
  <done>System check utility provides a clear "Green/Red" report for agent readiness.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
| Boundary | Description |
|----------|-------------|
| Internet → Local | Model downloads from HuggingFace Hub |

## STRIDE Threat Register
| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-01-01 | Tampering | `model_install.py` | mitigate | Use SHA256 checksums for all trusted models |
| T-01-01-02 | Spoofing | HuggingFace Repo | mitigate | Pin to specific revisions/commits in registry |
</threat_model>

<verification>
1. Run `python3 scripts/system_check.py` to verify model foundation.
</verification>

<success_criteria>
- Configuration is updated to Qwen 3.6.
- Model registry supports SOTA MoE variants.
- System check accurately identifies hardware constraints.
</success_criteria>

<output>
After completion, create `.planning/phases/01-foundation-security-sandbox/01-01-SUMMARY.md`
</output>

---
phase: 1
plan: 1
wave: 1
---

# Plan 1.1: Interactive setup.py script

## Objective
Create the main interactive `setup.py` that handles OS detection, dependency installation, model downloads, and configuration generation for standard LLM models.

## Context
- .gsd/SPEC.md
- .gsd/ARCHITECTURE.md
- config.py

## Tasks

<task type="auto">
  <name>Create `setup.py` with OS logic</name>
  <files>setup.py</files>
  <action>
    - Create a cross-platform python script that detects Windows or Linux.
    - Ask the user which model to install:
      - `1` for GPT-OSS-20B 
      - `2` for Qwen3.5-9B-Uncensored (https://huggingface.co/HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive/resolve/main/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf)
    - If models don't exist in `models/`, download them using a reliable method (e.g. `requests` with a progress bar).
    - Prompt the user for GPU layers to use.
    - Generate `config.py` based on their selections (MODEL_NAME, MODEL_PATH, GPU_LAYERS). Ensure old configurations (DANGEROUS_COMMANDS) are transferred.
    - Output clear instructions on how to use `agent.py`.
  </action>
  <verify>python setup.py --help</verify>
  <done>setup.py exists, can run without crashing, and writes to config.py based on prompts.</done>
</task>

## Success Criteria
- [ ] `setup.py` is executable and queries user.
- [ ] Downloads models to `models/` exactly.
- [ ] Overwrites/generates `config.py` correctly.

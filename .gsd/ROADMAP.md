# ROADMAP.md

> **Current Milestone**: v2.0 - Agent Framework & Inference Optimization
> **Goal**: Establish a strictly benchmarked setup environment (≥10 tok/s guarantee), an interactive Master Launcher for agent selection, universal HuggingFace model ingestion, and Rust-native RAG.

## Must-Haves
- [ ] Setup Profiler: Test speed, retrieval, and guarantee 10 tok/s during installation; remove unnecessary scripts.
- [ ] Master Launcher: A unified script to boot servers and select the interactive agent.
- [ ] HuggingFace Downloader: Allow users to search, download, and boot any HF model.
- [ ] Lightweight RAG: Vector embeddings natively in Rust.
- [ ] Verifier Upgrades: Strict token budgeting and context grounded checks.

## Phases

### Phase 1: Setup Profiling & Script Cleanup
**Status**: ✅ Complete
**Objective**: Overhaul the setup process to actively benchmark the system's token generation speed (ensuring ≥10 tok/s on all devices). Prune and remove all deprecated or unnecessary Python scripts from the project root.

### Phase 2: Master Launcher & Agent Selector
**Status**: ✅ Complete
**Objective**: Build a unified Rust or Python entrypoint that automatically starts the necessary backend servers (llama.cpp) and presents an interactive CLI menu allowing the user to select which specific Agent persona they want to orchestrate.

### Phase 3: Universal HuggingFace Downloader
**Status**: ✅ Complete
**Objective**: Introduce a CLI wizard that allows the user to paste *any* HuggingFace model URL or search term, downloads the `.gguf` file directly into `models/`, and boots the agent using it.

### Phase 4: Lightweight Rust RAG Integration
**Status**: ✅ Complete
**Objective**: Implement a local embedding service inside `agent-rs`. Build a searchable vector context store to enhance the agent's memory capability over entire directories, reducing hallucinations through grounded context retrieval.

### Phase 5: Rust Orchestrator Agentic Hardening
**Status**: ✅ Complete
**Objective**: Harden the Rust orchestrator for production-grade agentic tasks — new tools (`list_directory`, `append_file`), output truncation caps, schema error self-healing, 20-round loop, and temperature annealing for retries.

### Phase 6: Future Scope — Candle / mistral.rs Exploration (Optional)
**Status**: ⬜ Research Only
**Objective**: *Optional research phase.* Evaluate building inference directly into Rust via Candle or mistral.rs. This does **NOT** replace llama.cpp — the production architecture is Rust orchestration + llama.cpp inference. This phase exists only for experimentation if the community matures these Rust inference engines to parity with llama.cpp GGUF performance.

---

### Phase 7: Agentic Evaluation Suite
**Status**: ⬜ Not Started
**Objective**: Build a comprehensive local testing framework that evaluates our GGUF + Rust agent on the 4 core agentic dimensions: **Tool Call Accuracy**, **Reasoning & Planning**, **Self-Correction**, and **Context Management**. Rewrite `tests/eval.py` to work natively against the Rust CLI (no HTTP pairing token needed). Expand `tests/dataset.json` to 30+ tasks. Generate a Markdown benchmark report with success rate, per-task latency, and trajectory analysis. Use LLM-as-a-Judge (local model via llama.cpp) for automated grading.

### Phase 8: Project Structure Cleanup
**Status**: ⬜ Not Started
**Objective**: Consolidate all scattered Python scripts (`config.py`, `system_check.py`, `start_server.py`, `download_model.py`, `build_zip.py`) into a clean `scripts/` directory. Move shell scripts (`start_agent.sh`, `start_server.sh`) into `scripts/`. Keep only the single master entry points (`setup.py`, `start.py`) at the project root. Ensure all model files are downloaded into the project-local `models/` directory. Update all internal import paths and cross-references.

### Phase 9: Unified Setup & Initialization
**Status**: ⬜ Not Started
**Objective**: Create a single master installer script that handles the complete first-run experience:
1. Detect system hardware (CPU, RAM, GPU/iGPU) and recommend optimal configuration.
2. Let the user choose to download the **default GPT-OSS-20B** model ([DavidAU/OpenAi-GPT-oss-20b-abliterated-uncensored-NEO-Imatrix-gguf](https://huggingface.co/DavidAU/OpenAi-GPT-oss-20b-abliterated-uncensored-NEO-Imatrix-gguf)) or the lighter **Qwen3.5-9B** alternative ([HauhauCS/Qwen3.5-9B-Uncensored](https://huggingface.co/HauhauCS/Qwen3.5-9B-Uncensored-HauhauCS-Aggressive?show_file_info=Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf)), OR use the HuggingFace downloader to pick any model.
3. Install all Python/Rust dependencies automatically.
4. Build `llama.cpp` with the correct backend flags (CUDA/Vulkan/OpenVINO/CPU) for their hardware.
5. Run the agentic benchmark (Phase 7) as a pre-flight check and ensure ≥10 tok/s before allowing the user to proceed.
6. Generate `config.py` with optimized parameters. The initialization may take time but MUST guarantee the user gets the best possible token rate and response quality from their chosen LLM on their specific device.

### Phase 10: Dynamic ASCII Branding
**Status**: ⬜ Not Started
**Objective**: Develop a Python branding utility that generates a random, meaningful name for the agent (e.g., "Neon Guardian", "Cyber Scribe", "Quantum Ghost") to be used alongside "Agent" or "GPT". This script will then convert the selected name into a stunning ASCII art logo. Integrate this into the `start.py` entry point so that a fresh, unique visual identity is presented in the terminal every time the agent boots, enhancing the "premium" feel of the local AI experience.

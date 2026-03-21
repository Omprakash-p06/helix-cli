#!/usr/bin/env python3
"""
Server Launcher — Starts the LLM server using llama.cpp or KoboldCPP fallback.
All paths are project-relative.
"""
import os
import sys
import subprocess
import platform
import time
import argparse
import shlex

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.dirname(SCRIPT_DIR)

# Ensure scripts/config.py and helix_branding are importable
sys.path.insert(0, SCRIPT_DIR)
from helix_branding import print_helix_logo
try:
    import config
except ImportError:
    print("Error: config.py not found. Please run 'python setup.py' first.")
    sys.exit(1)


def resolve_runtime_model():
    model_path = os.environ.get("HELIX_MODEL_PATH", "").strip() or config.MODEL_PATH
    model_name = os.environ.get("HELIX_MODEL_NAME", "").strip() or getattr(config, "MODEL_NAME", os.path.basename(model_path))
    return model_name, model_path


def run_llama_server(model_path):
    """Attempt to start llama-server from the local llama.cpp build."""
    print("Attempting to start llama-server...")
    os_name = platform.system()
    candidate_names = ["llama-server.exe", "llama-server"] if os_name == "Windows" else ["llama-server"]
    llama_bin = ""
    for name in candidate_names:
        candidate = os.path.join(PROJECT_DIR, "llama.cpp", "build", "bin", name)
        if os.path.exists(candidate):
            llama_bin = candidate
            break

    if not llama_bin:
        searched = [os.path.join(PROJECT_DIR, "llama.cpp", "build", "bin", name) for name in candidate_names]
        print("  llama-server not found. Searched:")
        for path in searched:
            print(f"    - {path}")
        return False

    batch = getattr(config, "BATCH_SIZE", 512)
    ubatch = getattr(config, "UBATCH_SIZE", 256)
    backend = getattr(config, "BACKEND_HINT", "cpu")

    cmd = [
        llama_bin,
        "-m", model_path,
        "-c", str(config.CONTEXT_SIZE),
        "-t", str(config.CPU_THREADS),
        "-b", str(batch),
        "-ub", str(ubatch),
        "--host", config.SERVER_HOST,
        "--port", str(config.SERVER_PORT),
    ]

    # Backend-specific flags
    if backend == "openvino":
        print(f"  Backend: OpenVINO (Intel optimized)")
        # OpenVINO handles acceleration internally, but we still inject memory safety flags
        cmd.extend(["--no-mmap"])  # Avoids OS virtual memory thrashing on low-RAM Intel laptops
    elif backend == "vulkan":
        print(f"  Backend: Vulkan (iGPU compute offload)")
        cmd.extend(["-ngl", str(config.GPU_LAYERS)])
        cmd.extend(["-fa", "on"])  # Flash Attention — critical for >10 tok/s on AMD iGPUs
        cmd.extend(["--no-mmap"])  # Reduces RAM pressure on unified memory architectures
    elif backend == "cuda":
        print(f"  Backend: CUDA (NVIDIA GPU)")
        cmd.extend(["-ngl", str(config.GPU_LAYERS)])
        cmd.extend(["-fa", "on"])  # Flash attention for CUDA
    else:
        print(f"  Backend: CPU-only (AVX/AMX accelerated)")
        cmd.extend(["--no-mmap"])  # Required for machines sharing GPU/CPU memory pool
        # No GPU flags for pure CPU

    print(f"  Command: {' '.join(cmd)}")
    try:
        process = subprocess.Popen(cmd)
        time.sleep(3)
        if process.poll() is not None:
            print("  llama-server crashed immediately.")
            return False
        print("  llama-server started. Press Ctrl+C to stop.")
        process.wait()
        return True
    except Exception as e:
        print(f"  Error: {e}")
        return False


def run_koboldcpp(model_path):
    """Attempt to start KoboldCPP as a fallback."""
    print("\nAttempting to start KoboldCPP (fallback)...")
    kobold_bin_name = getattr(config, "KOBOLD_BIN", "")
    if not kobold_bin_name:
        print("  KoboldCPP not configured in config.py")
        return False

    kobold_bin = os.path.join(PROJECT_DIR, kobold_bin_name)
    if not os.path.exists(kobold_bin):
        print(f"  KoboldCPP binary not found at: {kobold_bin}")
        print("  Run 'python setup.py' to download it.")
        return False

    backend = getattr(config, "BACKEND_HINT", "cpu")
    gpu_layers = getattr(config, "GPU_LAYERS", 0)
    # KoboldCPP does not use OpenVINO backend selection like llama.cpp.
    # For OpenVINO-hinted setups, prefer conservative CPU-safe fallback.
    if backend == "openvino":
        gpu_layers = 0
        print("  KoboldCPP fallback: OpenVINO hint detected; using CPU-safe gpulayers=0")

    cmd = [
        kobold_bin,
        model_path,
        "--host", config.SERVER_HOST,
        "--port", str(config.SERVER_PORT),
        "--gpulayers", str(gpu_layers),
        "--contextsize", str(config.CONTEXT_SIZE),
        "--threads", str(config.CPU_THREADS),
        "--smartcontext",
    ]

    # Auto-tuned preset from setup.py plus optional env override.
    # Env value is appended last so users can refine/override defaults.
    preset_args = getattr(config, "KOBOLDCPP_ARGS", "").strip()
    if preset_args:
        cmd.extend(shlex.split(preset_args))

    extra_args = os.environ.get("KOBOLDCPP_ARGS", "").strip()
    if extra_args:
        cmd.extend(shlex.split(extra_args))

    print(f"  Command: {' '.join(cmd)}")
    try:
        process = subprocess.Popen(cmd)
        print("  KoboldCPP started. Press Ctrl+C to stop.")
        process.wait()
        return True
    except Exception as e:
        print(f"  Error: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(description="Start the local LLM server.")
    parser.parse_args()

    model_name, model_path = resolve_runtime_model()

    print_helix_logo(animated=True, delay=0.015)
    print()
    print("=" * 55)
    print(f"  Starting LLM Server: {model_name}")
    print("=" * 55)
    print(f"  Model path: {model_path}")
    print(f"  Endpoint:   http://{config.SERVER_HOST}:{config.SERVER_PORT}/v1")
    print()

    if not os.path.exists(model_path):
        print(f"\n[!] Model file not found: {model_path}")
        print("[!] Universal GGUF Support Enabled: Please place any 8-32B '.gguf' file into the 'models/' directory.")
        sys.exit(1)

    if run_llama_server(model_path):
        print("Server shutdown gracefully.")
        sys.exit(0)

    print("\n[!] Primary backend failed. Checking logs for VRAM exhaustion...")
    # Check the log file that start.py redirects our stderr to
    log_path = os.path.join(PROJECT_DIR, "logs", "start_server.stderr.log")
    if os.path.exists(log_path):
        with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
            err_content = f.read().lower()
            if "out of memory" in err_content or "failed to allocate" in err_content or "bad allocation" in err_content:
                print("  [!] VRAM OOM detected! Falling back to iGPU/System RAM...")
                config.GPU_LAYERS = getattr(config, "FALLBACK_GPU_LAYERS", 0)
                if run_llama_server(model_path):
                    print("Server shutdown gracefully.")
                    sys.exit(0)

    print("\n[!] Primary backend completely failed. Trying KoboldCPP fallback...")
    if run_koboldcpp(model_path):
        print("Server shutdown gracefully.")
        sys.exit(0)
    else:
        print("\n[!] Both backends failed. Check your model path and dependencies.")
        sys.exit(1)

if __name__ == "__main__":
    main()

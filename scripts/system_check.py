#!/usr/bin/env python3
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import Dict, Optional, Tuple


SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR.parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import config


def check_docker() -> Dict[str, str]:
    docker_bin = shutil.which("docker")
    if not docker_bin:
        return {
            "status": "RED",
            "summary": "Docker CLI not found",
            "detail": "Install Docker and ensure `docker` is on PATH for SEC-01 sandboxing.",
        }

    result = subprocess.run(
        [docker_bin, "info", "--format", "{{.ServerVersion}}"],
        capture_output=True,
        text=True,
        timeout=5,
        check=False,
    )
    if result.returncode == 0 and result.stdout.strip():
        return {
            "status": "GREEN",
            "summary": "Docker daemon reachable",
            "detail": f"Docker server version: {result.stdout.strip()}",
        }

    detail = result.stderr.strip() or result.stdout.strip() or "Docker daemon not reachable."
    return {
        "status": "RED",
        "summary": "Docker daemon unavailable",
        "detail": detail,
    }


def select_installed_model() -> Tuple[Optional[Path], Optional[Dict[str, str]]]:
    for model_name, entry in config.AVAILABLE_MODELS.items():
        candidate = Path(entry["path"])
        if candidate.exists():
            return candidate, {
                "model_name": model_name,
                "quantization": entry["quantization"],
                "guidance": entry["guidance"],
            }
    default_path = Path(config.MODEL_PATH)
    return default_path, None


def check_model_file() -> Dict[str, str]:
    model_path, installed_entry = select_installed_model()
    if installed_entry:
        return {
            "status": "GREEN",
            "summary": f"Model file present: {model_path.name}",
            "detail": (
                f"{installed_entry['model_name']} detected with {installed_entry['quantization']}."
                f" {installed_entry['guidance']}"
            ),
        }

    return {
        "status": "RED",
        "summary": f"Model file missing: {model_path.name}",
        "detail": (
            f"Expected the default Qwen 3.6 artifact at `{model_path}`. "
            f"Recommended quantization is {config.MODEL_QUANTIZATION} based on detected VRAM."
        ),
    }


def find_llama_binary() -> Optional[Path]:
    candidates = [
        PROJECT_DIR / "llama.cpp" / "build" / "bin" / "llama-server",
        PROJECT_DIR / "llama.cpp" / "build" / "bin" / "llama-cli",
        PROJECT_DIR / "llama.cpp" / "build" / "bin" / "Release" / "llama-server.exe",
        PROJECT_DIR / "llama.cpp" / "build" / "bin" / "Release" / "llama-cli.exe",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def check_llama_smoke() -> Dict[str, str]:
    binary = find_llama_binary()
    if not binary:
        return {
            "status": "RED",
            "summary": "llama.cpp runtime missing",
            "detail": "Build `llama-server` or `llama-cli` under `llama.cpp/build/bin`.",
        }

    mode = binary.stat().st_mode
    is_executable = bool(mode & stat.S_IXUSR)
    if not is_executable and os.name != "nt":
        return {
            "status": "RED",
            "summary": f"llama.cpp binary is not executable: {binary.name}",
            "detail": f"Fix permissions on `{binary}` before starting inference.",
        }

    version_check = subprocess.run(
        [str(binary), "--version"],
        capture_output=True,
        text=True,
        timeout=5,
        check=False,
    )
    if version_check.returncode == 0:
        detail = version_check.stdout.strip() or version_check.stderr.strip() or "Version probe succeeded."
        return {
            "status": "GREEN",
            "summary": f"llama.cpp binary ready: {binary.name}",
            "detail": detail.splitlines()[0],
        }

    return {
        "status": "RED",
        "summary": f"llama.cpp smoke test failed: {binary.name}",
        "detail": version_check.stderr.strip() or version_check.stdout.strip() or "Version probe failed.",
    }


def quantization_advice() -> Dict[str, str]:
    vram = config.DETECTED_VRAM_GB
    if vram is None:
        headline = "No NVIDIA VRAM detected"
        detail = (
            f"Defaulting to {config.MODEL_NAME} in {config.MODEL_QUANTIZATION} with CPU-safe settings. "
            f"{config.MODEL_SELECTION_GUIDANCE}"
        )
    else:
        headline = f"Detected NVIDIA VRAM: {vram}GB"
        detail = (
            f"Default profile: {config.MODEL_NAME} / {config.MODEL_QUANTIZATION} / "
            f"GPU layers {config.GPU_LAYERS}. {config.MODEL_SELECTION_GUIDANCE}"
        )

    alternate = config.AVAILABLE_MODELS["Qwen-3.6-35B-MoE"]
    return {
        "status": "GREEN",
        "summary": headline,
        "detail": (
            f"{detail} Alternate target: Qwen-3.6-35B-MoE prefers {alternate['quantization']} "
            f"at the current VRAM tier."
        ),
    }


def render_report(checks: Dict[str, Dict[str, str]]) -> str:
    lines = [
        "=" * 68,
        "Helix System Readiness Check",
        "=" * 68,
        f"Default model: {config.MODEL_NAME}",
        f"Default artifact: {Path(config.MODEL_PATH).name}",
        f"Backend hint: {config.BACKEND_HINT}",
        "",
    ]
    for name, result in checks.items():
        lines.append(f"[{result['status']}] {name}: {result['summary']}")
        lines.append(f"      {result['detail']}")
    lines.append("")
    overall = "GREEN" if all(item["status"] == "GREEN" for item in checks.values()) else "RED"
    lines.append(f"Overall readiness: {overall}")
    return "\n".join(lines)


def main() -> int:
    checks = {
        "Docker": check_docker(),
        "Model": check_model_file(),
        "llama.cpp": check_llama_smoke(),
        "Quantization": quantization_advice(),
    }
    print(render_report(checks))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

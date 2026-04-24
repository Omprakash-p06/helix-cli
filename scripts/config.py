import os
import subprocess
from typing import Any, Dict, Optional


PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MODELS_DIR = os.path.join(PROJECT_DIR, "models")


def _safe_int(value: Optional[str]) -> Optional[int]:
    if value is None or value == "":
        return None
    try:
        return int(float(value))
    except (TypeError, ValueError):
        return None


def detect_gpu_vram_gb() -> Optional[int]:
    env_override = _safe_int(os.environ.get("HELIX_GPU_VRAM_GB"))
    if env_override is not None and env_override >= 0:
        return env_override

    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=memory.total", "--format=csv,noheader,nounits"],
            capture_output=True,
            text=True,
            timeout=2,
            check=False,
        )
    except (FileNotFoundError, OSError, subprocess.SubprocessError):
        return None

    if result.returncode != 0 or not result.stdout.strip():
        return None

    first_line = result.stdout.strip().splitlines()[0].strip()
    mib = _safe_int(first_line)
    if mib is None:
        return None
    return max(0, round(mib / 1024))


MODEL_CATALOG: Dict[str, Dict[str, Any]] = {
    "Qwen-3.6-27B-MoE": {
        "repo_alias": "qwen-3.6-27b-moe",
        "variants": [
            {
                "min_vram_gb": 24,
                "quantization": "Q8_0",
                "filename": "Qwen3.6-27B-Instruct-Q8_0.gguf",
                "gpu_layers": -1,
                "backend_hint": "cuda",
                "context_size": 12288,
                "batch_size": 768,
                "ubatch_size": 384,
                "guidance": "24GB+ VRAM can run the 27B MoE in Q8_0 with full CUDA offload.",
            },
            {
                "min_vram_gb": 12,
                "quantization": "Q5_K_M",
                "filename": "Qwen3.6-27B-Instruct-Q5_K_M.gguf",
                "gpu_layers": 48,
                "backend_hint": "cuda",
                "context_size": 8192,
                "batch_size": 512,
                "ubatch_size": 256,
                "guidance": "12GB-23GB VRAM should target the 27B MoE in Q5_K_M with partial offload.",
            },
            {
                "min_vram_gb": 8,
                "quantization": "Q4_K_M",
                "filename": "Qwen3.6-27B-Instruct-Q4_K_M.gguf",
                "gpu_layers": 24,
                "backend_hint": "cuda",
                "context_size": 6144,
                "batch_size": 384,
                "ubatch_size": 192,
                "guidance": "8GB-11GB VRAM should use the 27B MoE in Q4_K_M with conservative CUDA offload.",
            },
            {
                "min_vram_gb": 0,
                "quantization": "Q4_K_M",
                "filename": "Qwen3.6-27B-Instruct-Q4_K_M.gguf",
                "gpu_layers": 0,
                "backend_hint": "cpu",
                "context_size": 4096,
                "batch_size": 256,
                "ubatch_size": 128,
                "guidance": "Without a supported discrete GPU, keep the 27B MoE on CPU in Q4_K_M.",
            },
        ],
    },
    "Qwen-3.6-35B-MoE": {
        "repo_alias": "qwen-3.6-35b-moe",
        "variants": [
            {
                "min_vram_gb": 32,
                "quantization": "Q8_0",
                "filename": "Qwen3.6-35B-Instruct-Q8_0.gguf",
                "gpu_layers": -1,
                "backend_hint": "cuda",
                "context_size": 12288,
                "batch_size": 768,
                "ubatch_size": 384,
                "guidance": "32GB+ VRAM can run the 35B MoE in Q8_0 with full CUDA offload.",
            },
            {
                "min_vram_gb": 24,
                "quantization": "Q5_K_M",
                "filename": "Qwen3.6-35B-Instruct-Q5_K_M.gguf",
                "gpu_layers": 40,
                "backend_hint": "cuda",
                "context_size": 8192,
                "batch_size": 512,
                "ubatch_size": 256,
                "guidance": "24GB-31GB VRAM can sustain the 35B MoE in Q5_K_M with partial CUDA offload.",
            },
            {
                "min_vram_gb": 16,
                "quantization": "Q4_K_M",
                "filename": "Qwen3.6-35B-Instruct-Q4_K_M.gguf",
                "gpu_layers": 24,
                "backend_hint": "cuda",
                "context_size": 6144,
                "batch_size": 384,
                "ubatch_size": 192,
                "guidance": "16GB-23GB VRAM should stay on the 35B MoE in Q4_K_M with conservative offload.",
            },
            {
                "min_vram_gb": 0,
                "quantization": "Q4_K_M",
                "filename": "Qwen3.6-35B-Instruct-Q4_K_M.gguf",
                "gpu_layers": 0,
                "backend_hint": "cpu",
                "context_size": 4096,
                "batch_size": 256,
                "ubatch_size": 128,
                "guidance": "Without enough VRAM, the 35B MoE should remain a manual install target only.",
            },
        ],
    },
}


def _variant_for_model(model_name: str, vram_gb: Optional[int]) -> Dict[str, Any]:
    catalog = MODEL_CATALOG[model_name]
    effective_vram = 0 if vram_gb is None else max(0, vram_gb)
    for variant in catalog["variants"]:
        if effective_vram >= variant["min_vram_gb"]:
            return dict(variant)
    return dict(catalog["variants"][-1])


def build_model_entry(model_name: str, vram_gb: Optional[int] = None) -> Dict[str, Any]:
    variant = _variant_for_model(model_name, vram_gb)
    return {
        "model_name": model_name,
        "repo_alias": MODEL_CATALOG[model_name]["repo_alias"],
        "quantization": variant["quantization"],
        "filename": variant["filename"],
        "path": os.path.join(MODELS_DIR, variant["filename"]),
        "gpu_layers": variant["gpu_layers"],
        "backend_hint": variant["backend_hint"],
        "context_size": variant["context_size"],
        "batch_size": variant["batch_size"],
        "ubatch_size": variant["ubatch_size"],
        "guidance": variant["guidance"],
        "detected_vram_gb": vram_gb,
    }


DETECTED_VRAM_GB = detect_gpu_vram_gb()
DEFAULT_MODEL_NAME = "Qwen-3.6-27B-MoE"
MODEL_NAME = DEFAULT_MODEL_NAME
MODEL_PROFILE = build_model_entry(MODEL_NAME, DETECTED_VRAM_GB)
MODEL_PATH = MODEL_PROFILE["path"]
MODEL_QUANTIZATION = MODEL_PROFILE["quantization"]
MODEL_SELECTION_GUIDANCE = MODEL_PROFILE["guidance"]

AVAILABLE_MODELS = {
    model_name: build_model_entry(model_name, DETECTED_VRAM_GB) for model_name in MODEL_CATALOG
}

SERVER_HOST = "127.0.0.1"
SERVER_PORT = 8080
BASE_URL = f"http://{SERVER_HOST}:{SERVER_PORT}/v1"
KOBOLD_BIN = "koboldcpp-linux-x64"
KOBOLDCPP_ARGS = ""

GPU_LAYERS = MODEL_PROFILE["gpu_layers"]
CONTEXT_SIZE = MODEL_PROFILE["context_size"]
CPU_THREADS = min(16, max(4, (os.cpu_count() or 8)))
BATCH_SIZE = MODEL_PROFILE["batch_size"]
UBATCH_SIZE = MODEL_PROFILE["ubatch_size"]
BACKEND_HINT = MODEL_PROFILE["backend_hint"]
FALLBACK_GPU_LAYERS = 0
FALLBACK_BACKEND_HINT = "cpu"

CHAT_SYSTEM_PROMPT = (
    "You are Helix running on Qwen 3.6. Give direct, technically precise answers for terminal and "
    "systems work. Keep replies concise, surface assumptions explicitly, and never reveal hidden "
    "reasoning or chain-of-thought."
)

AGENTIC_SYSTEM_PROMPT = (
    "You are Helix, a local-first systems agent running on Qwen 3.6. Operate like a disciplined "
    "terminal engineer: verify the environment before acting, prefer the minimal safe command, report "
    "blocking errors exactly, and avoid filler. Never emit <think>, <analysis>, or hidden reasoning."
)

REQUIRE_CONFIRMATION = True
TOOL_PERMISSION_TIER = "workspace_write"
LOG_COMMANDS = True
LOG_DIR = os.path.join(PROJECT_DIR, "logs")
os.makedirs(LOG_DIR, exist_ok=True)

AUDIT_ENABLED = True
AUDIT_DB_PATH = os.path.join(LOG_DIR, "audit.db")

DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]

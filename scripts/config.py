import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional


PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MODELS_DIR = os.path.join(PROJECT_DIR, "models")


@dataclass(frozen=True)
class ModelEntry:
    name: str
    path: str
    size_mb: float
    vram_estimate_gb: float
    parameter_count_b: Optional[float] = None

    def as_catalog_entry(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "path": self.path,
            "size_mb": self.size_mb,
            "vram_estimate_gb": self.vram_estimate_gb,
            "parameter_count_b": self.parameter_count_b,
        }


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


def _parse_parameter_count(model_name: str) -> Optional[float]:
    matches = re.findall(r"(\d+(?:\.\d+)?)([BbMm])", model_name)
    if not matches:
        return None

    value, unit = matches[-1]
    try:
        count = float(value)
    except ValueError:
        return None

    if unit.upper() == "M":
        return round(count / 1000.0, 3)
    return round(count, 3)


def _estimate_vram_gb(size_bytes: int, parameter_count_b: Optional[float]) -> float:
    size_gb = size_bytes / (1024.0 * 1024.0 * 1024.0)
    param_hint = (parameter_count_b or 0.0) * 0.15
    return round(max(0.5, size_gb * 1.1, param_hint), 2)


def scan_models_directory(models_dir: Optional[str] = None) -> List[ModelEntry]:
    root = Path(models_dir or MODELS_DIR)
    if not root.exists():
        print(f"[Model Discovery] No models directory found at {root}.")
        return []

    discovered: List[ModelEntry] = []
    for path in sorted(root.rglob("*.gguf")):
        if not path.is_file():
            continue

        size_bytes = path.stat().st_size
        parameter_count = _parse_parameter_count(path.stem)
        discovered.append(
            ModelEntry(
                name=path.stem,
                path=str(path),
                size_mb=round(size_bytes / (1024.0 * 1024.0), 2),
                vram_estimate_gb=_estimate_vram_gb(size_bytes, parameter_count),
                parameter_count_b=parameter_count,
            )
        )

    discovered.sort(
        key=lambda item: (
            float("inf") if item.parameter_count_b is None else item.parameter_count_b,
            item.size_mb,
            item.name.lower(),
        )
    )

    if not discovered:
        print(f"[Model Discovery] No GGUF models found in {root}.")

    return discovered


def scan_models_dir() -> Dict[str, Dict[str, Any]]:
    models: Dict[str, Dict[str, Any]] = {}
    for entry in scan_models_directory():
        models[entry.name] = {
            "repo_alias": entry.name.lower(),
            "variants": [
                {
                    "min_vram_gb": 0,
                    "quantization": "Unknown",
                    "filename": Path(entry.path).name,
                    "gpu_layers": -1,
                    "backend_hint": "cuda",
                    "context_size": 8192,
                    "batch_size": 512,
                    "ubatch_size": 256,
                    "guidance": f"Local model found: {Path(entry.path).name}",
                }
            ],
        }
    return models


BLOCKLIST = frozenset(
    {
        "rm -rf /",
        "rm -rf /*",
        "mkfs",
        "mkfs.*",
        "dd if=/dev/zero",
        ":(){ :|:& };:",
        "wget | sh",
        "curl | sh",
    }
)


def _normalise_command(command: str) -> str:
    return re.sub(r"\s+", " ", command.strip().lower())


def is_blocked_command(command: str) -> bool:
    normalized = _normalise_command(command)
    if not normalized:
        return False

    if any(pattern in normalized for pattern in ("rm -rf /", "rm -rf /*")):
        return True

    if normalized.startswith("mkfs") or " mkfs." in normalized:
        return True

    if "dd if=/dev/zero" in normalized:
        return True

    if ":(){ :|:& };:" in normalized:
        return True

    if "wget" in normalized and ("| sh" in normalized or "| bash" in normalized):
        return True

    if "curl" in normalized and ("| sh" in normalized or "| bash" in normalized):
        return True

    return any(pattern in normalized for pattern in BLOCKLIST)


def _static_model_catalog() -> Dict[str, Dict[str, Any]]:
    return {
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
                    "guidance": "12GB+ VRAM can run the 27B MoE in Q5_K_M with partial CUDA offload.",
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
                    "guidance": "8GB+ VRAM can run the 27B MoE in Q4_K_M with reduced CUDA offload.",
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
                    "guidance": "24GB+ VRAM can run the 35B MoE in Q5_K_M with partial CUDA offload.",
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
                    "guidance": "16GB+ VRAM can run the 35B MoE in Q4_K_M with reduced CUDA offload.",
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
                    "guidance": "Without a supported discrete GPU, keep the 35B MoE on CPU in Q4_K_M.",
                },
            ],
        },
    }


MODEL_CATALOG = _static_model_catalog()


def _variant_for_model(model_name: str, vram_gb: Optional[int]) -> Dict[str, Any]:
    catalog = MODEL_CATALOG.get(model_name)
    if not catalog:
        # If model_name is not in catalog but exists as a file, return a default variant
        return {
            "min_vram_gb": 0,
            "quantization": "Unknown",
            "filename": f"{model_name}.gguf",
            "gpu_layers": -1,
            "backend_hint": "cuda",
            "context_size": 8192,
            "batch_size": 512,
            "ubatch_size": 256,
            "guidance": f"Dynamic model: {model_name}",
        }
        
    effective_vram = 0 if vram_gb is None else max(0, vram_gb)
    for variant in catalog["variants"]:
        if effective_vram >= variant["min_vram_gb"]:
            return dict(variant)
    return dict(catalog["variants"][-1])


def build_model_entry(model_name: str, vram_gb: Optional[int] = None) -> Dict[str, Any]:
    variant = _variant_for_model(model_name, vram_gb)
    repo_alias = MODEL_CATALOG[model_name]["repo_alias"] if model_name in MODEL_CATALOG else model_name.lower()
    return {
        "model_name": model_name,
        "repo_alias": repo_alias,
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
DEFAULT_MODEL_NAME = list(MODEL_CATALOG.keys())[0] if MODEL_CATALOG else "Qwen-3.6-27B-MoE"
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
    f"You are Helix running on {MODEL_NAME}. Give direct, technically precise answers for terminal and "
    "systems work. Keep replies concise, surface assumptions explicitly, and never reveal hidden "
    "reasoning or chain-of-thought."
)

AGENTIC_SYSTEM_PROMPT = (
    f"You are Helix, a local-first systems agent running on {MODEL_NAME}. Operate like a disciplined "
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

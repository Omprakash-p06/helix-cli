import os
import platform

PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

MODEL_NAME = "Qwen3.5-9B-Uncensored"
MODEL_PATH = os.path.join(PROJECT_DIR, "models", "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf")

AVAILABLE_MODELS = {
    "Qwen3.5-9B-Uncensored": os.path.join(PROJECT_DIR, "models", "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf"),
}

SERVER_HOST = "127.0.0.1"
SERVER_PORT = 8080
BASE_URL = f"http://{SERVER_HOST}:{SERVER_PORT}/v1"

# Cross-platform binary selection
if platform.system() == "Windows":
    KOBOLD_BIN = "koboldcpp.exe"
else:
    KOBOLD_BIN = "koboldcpp-linux-x64"

KOBOLDCPP_ARGS = "--flashattention"

# Performance (Tier 3/5 - Mid)
GPU_LAYERS = 19
FALLBACK_GPU_LAYERS = 0
CONTEXT_SIZE = 8192
CPU_THREADS = 6
BATCH_SIZE = 512
UBATCH_SIZE = 256
BACKEND_HINT = "cuda"

REQUIRE_CONFIRMATION = True
LOG_COMMANDS = True
LOG_DIR = os.path.join(PROJECT_DIR, "logs")
os.makedirs(LOG_DIR, exist_ok=True)

# Cross-platform dangerous command lists
if platform.system() == "Windows":
    DANGEROUS_COMMANDS = ["del", "rmdir", "format", "diskpart", "shutdown", "rm", "mv"]
else:
    DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]

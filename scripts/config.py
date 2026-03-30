import os

PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

MODEL_NAME = "Qwen3.5-9B-Uncensored"
MODEL_PATH = os.path.join(PROJECT_DIR, "models", "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf")

AVAILABLE_MODELS = {
    "Qwen3.5-9B-Uncensored": os.path.join(PROJECT_DIR, "models", "Qwen3.5-9B-Uncensored-HauhauCS-Aggressive-Q4_K_M.gguf"),
}

SERVER_HOST = "127.0.0.1"
SERVER_PORT = 8080
BASE_URL = f"http://{SERVER_HOST}:{SERVER_PORT}/v1"
KOBOLD_BIN = "koboldcpp-linux-x64"
KOBOLDCPP_ARGS = ""

# Performance (Tier 2/5 - Consumer 4GB VRAM)
GPU_LAYERS = 16
CONTEXT_SIZE = 8192
CPU_THREADS = 6
BATCH_SIZE = 512
UBATCH_SIZE = 256
BACKEND_HINT = "cuda"
FALLBACK_GPU_LAYERS = 0
FALLBACK_BACKEND_HINT = "cpu"

REQUIRE_CONFIRMATION = True
LOG_COMMANDS = True
LOG_DIR = os.path.join(PROJECT_DIR, "logs")
os.makedirs(LOG_DIR, exist_ok=True)

DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]

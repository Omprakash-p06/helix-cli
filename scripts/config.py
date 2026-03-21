import os
import glob

# Project root directory (auto-detected from scripts/ folder)
PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MODELS_DIR = os.path.join(PROJECT_DIR, "models")
os.makedirs(MODELS_DIR, exist_ok=True)

# Phase 11: Universal GGUF Support. Scan directory for any 8-32B model securely.
# Phase 3 enhancement: Sort by modified time so newly downloaded models automatically boot first.
available_models = sorted(glob.glob(os.path.join(MODELS_DIR, "*.gguf")), key=os.path.getmtime, reverse=True)

if available_models:
    # Safely pick the first available GGUF model in the directory
    MODEL_PATH = available_models[0]
    MODEL_NAME = os.path.basename(MODEL_PATH).replace(".gguf", "")
else:
    # Fallback default if directory happens to be empty
    MODEL_PATH = os.path.join(MODELS_DIR, "gpt-oss-20b-IQ4_NL.gguf")
    MODEL_NAME = "gpt-oss-20b"

SERVER_HOST, SERVER_PORT = "127.0.0.1", 8080
BASE_URL = f"http://{SERVER_HOST}:{SERVER_PORT}/v1"

# Hardware parameters updated via phase 6 offline checks
GPU_LAYERS, CONTEXT_SIZE, CPU_THREADS = 4, 8192, 6
REQUIRE_CONFIRMATION, LOG_COMMANDS = True, True
DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]

LOG_DIR = os.path.join(PROJECT_DIR, "logs")
os.makedirs(LOG_DIR, exist_ok=True)

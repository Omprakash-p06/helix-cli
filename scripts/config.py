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
# Reduced from 16 to 9 GPU layers to fit on RTX 3050 4GB (needs ~3150MiB but only has ~3573MiB available with 1024MiB buffer)
GPU_LAYERS = 9
CONTEXT_SIZE = 8192
CPU_THREADS = 6
BATCH_SIZE = 512
UBATCH_SIZE = 256
BACKEND_HINT = "cuda"
FALLBACK_GPU_LAYERS = 0
FALLBACK_BACKEND_HINT = "cpu"

CHAT_SYSTEM_PROMPT = (
    "You are Helix in chat mode. Respond directly and concisely. "
    "Never reveal internal reasoning, analysis, or chain-of-thought. "
    "Do not emit <think>, <thinking>, or <analysis> tags."
)

AGENTIC_SYSTEM_PROMPT = (
    "You are an autonomous local system orchestrator. You execute tasks using provided tools. "
    "Before each tool call, state your reasoning in one sentence. Never guess file paths - verify with list_directory first. "
    "If a command fails, read STDERR and retry with a corrected approach. Do not greet the user. "
    "Do not introduce yourself. Do not use conversational filler. Be concise. "
    "You have local tool access through these tools, so do not ask the user to run local file-system commands when a tool can do it."
)

REQUIRE_CONFIRMATION = True
LOG_COMMANDS = True
LOG_DIR = os.path.join(PROJECT_DIR, "logs")
os.makedirs(LOG_DIR, exist_ok=True)

DANGEROUS_COMMANDS = ["rm", "mv", "chmod", "dd", "mkfs", "fdisk", "systemctl", "reboot", "shutdown"]

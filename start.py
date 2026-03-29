#!/usr/bin/env python3
import os
import sys
import time
import subprocess
import requests
from scripts.helix_branding import print_helix_logo

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, PROJECT_DIR)

print_helix_logo(animated=True, delay=0.015)
print()
print("=" * 55)
print("  Initializing Helix Agent Stack")
print("=" * 55)


def discover_models(models_dir):
    if not os.path.isdir(models_dir):
        return []
    return sorted([f for f in os.listdir(models_dir) if f.lower().endswith(".gguf")])


def choose_model():
    models_dir = os.path.join(PROJECT_DIR, "models")
    models = discover_models(models_dir)
    if not models:
        print("  [!] No .gguf models found in models directory.")
        sys.exit(1)

    print("\nSelect model to load:")
    for idx, model_file in enumerate(models, 1):
        print(f"  {idx}) {model_file}")

    picked = input(f"Select model (1-{len(models)}, default 1): ").strip()
    try:
        selected = models[int(picked) - 1] if picked else models[0]
    except Exception:
        selected = models[0]

    return selected, os.path.join(models_dir, selected)


def choose_interface():
    """Prompt user to select between Terminal Chat and Web Interface."""
    print("\nSelect interface:")
    print("  1) Terminal Chat")
    print("  2) Web Interface")
    choice = input("Select interface (1-2, default 1): ").strip()
    if choice == "2":
        return "web"
    return "tui"


def choose_exec_mode():
    """Prompt user to select between Agentic and Chat execution modes."""
    print("\nSelect mode:")
    print("  1) Agentic (autonomous tool-calling)")
    print("  2) Chat (conversation only)")
    choice = input("Select mode (1-2, default 1): ").strip()
    if choice == "2":
        return "chat"
    return "agentic"


selected_model_name, selected_model_path = choose_model()
interface_choice = choose_interface()
exec_mode = choose_exec_mode()

# 1. Clean orphaned GPU processes guarantees VRAM is empty
def clean_orphaned_servers():
    print("\n  [i] Cleaning orphaned GPU processes...")
    try:
        if os.name == 'nt':
            subprocess.run(["taskkill", "/F", "/IM", "llama-server.exe"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            subprocess.run(["taskkill", "/F", "/IM", "koboldcpp.exe"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        else:
            subprocess.run(["pkill", "-f", "llama-server"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            subprocess.run(["pkill", "-f", "koboldcpp"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception:
        pass

clean_orphaned_servers()

# 2. Boot the LLM server in the background
print("  [i] Booting Local Engine...")
logs_dir = os.path.join(PROJECT_DIR, "logs")
os.makedirs(logs_dir, exist_ok=True)
server_out_log = os.path.join(logs_dir, "start_server.stdout.log")
server_err_log = os.path.join(logs_dir, "start_server.stderr.log")
out_fh = open(server_out_log, "w", encoding="utf-8", errors="replace")
err_fh = open(server_err_log, "w", encoding="utf-8", errors="replace")

server_env = os.environ.copy()
server_env["HELIX_MODEL_NAME"] = selected_model_name
server_env["HELIX_MODEL_PATH"] = selected_model_path
server_env["HELIX_EXEC_MODE"] = exec_mode

server_proc = subprocess.Popen(
    [sys.executable, os.path.join(PROJECT_DIR, "scripts", "start_server.py")],
    stdout=out_fh,
    stderr=err_fh,
    cwd=PROJECT_DIR,
    env=server_env,
)

# 2. Wait for the API to come online
def wait_for_server():
    timeout_s = int(os.environ.get("HELIX_SERVER_STARTUP_TIMEOUT_S", "180"))
    for _ in range(max(1, timeout_s)):
        try:
            res = requests.get("http://127.0.0.1:8080/v1/models")
            if res.status_code == 200:
                return True
        except Exception:
            pass
        time.sleep(1)
    return False

if not wait_for_server():
    print("  [!] LLM Server failed to start or timed out.")
    if server_proc.poll() is not None:
        print(f"  [!] start_server.py exited early with code {server_proc.returncode}.")
    print(f"  [i] Startup logs: {server_out_log}")
    print(f"  [i] Error logs:   {server_err_log}")

    def _tail(path, max_lines=25):
        try:
            with open(path, "r", encoding="utf-8", errors="replace") as fh:
                lines = fh.readlines()
            return "".join(lines[-max_lines:]).strip()
        except Exception:
            return ""

    err_tail = _tail(server_err_log)
    out_tail = _tail(server_out_log)
    if err_tail:
        print("  [i] Recent stderr:")
        for line in err_tail.splitlines():
            print(f"      {line}")
    elif out_tail:
        print("  [i] Recent stdout:")
        for line in out_tail.splitlines():
            print(f"      {line}")

    server_proc.terminate()
    out_fh.close()
    err_fh.close()
    sys.exit(1)

print("  [✓] Engine online.")
time.sleep(0.5)

# 3. Clear screen for launch
os.system('cls' if os.name == 'nt' else 'clear')
print_helix_logo(animated=True, delay=0.02)
print()
print(f"Model:     {selected_model_name}")
print(f"Interface: {interface_choice.capitalize()}")
print(f"Mode:      {exec_mode.capitalize()}")

print("\n  Handing over to Orchestrator...")
time.sleep(0.5)

try:
    agent_dir = os.path.join(PROJECT_DIR, "agent-rs")
    web_dir = os.path.join(PROJECT_DIR, "web-ui")
    agent_env = os.environ.copy()
    agent_env["HELIX_EXEC_MODE"] = exec_mode
    agent_env["HELIX_UI_MODE"] = interface_choice
    
    if interface_choice == "web":
        print("  [i] Booting Rust API and Vite Dev Server...")
        agent_proc = subprocess.Popen(["cargo", "run", "--quiet"], cwd=agent_dir, env=agent_env)
        time.sleep(2)  # Wait for API to open port
        # Using shell=True for npm command wrapper resolves path issues cross-platform
        web_cmd = ["npm.cmd" if os.name == 'nt' else "npm", "run", "dev", "--", "--open"]
        web_proc = subprocess.Popen(web_cmd, cwd=web_dir)
        agent_proc.wait()
    else:
        subprocess.call(["cargo", "run", "--quiet"], cwd=agent_dir, env=agent_env)
except KeyboardInterrupt:
    pass
finally:
    # 5. Guaranteed teardown
    print("\n  Shutting down stack...")
    if 'agent_proc' in locals():
        agent_proc.terminate()
    if 'web_proc' in locals():
        web_proc.terminate()
    server_proc.terminate()
    server_proc.wait()
    out_fh.close()
    err_fh.close()
    print("  Goodbye.")

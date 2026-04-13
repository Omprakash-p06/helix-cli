#!/usr/bin/env python3
import os
import sys
import time
import subprocess
import requests
from scripts.helix_branding import print_helix_logo
from scripts import onboarding_profile

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


def choose_model(models, default_model=None):
    if not models:
        print("  [!] No .gguf models found in models directory.")
        sys.exit(1)

    if default_model not in models:
        default_model = models[0]

    default_index = models.index(default_model)
    print("\nSelect model to load:")
    for idx, model_file in enumerate(models, 1):
        marker = " (default)" if idx - 1 == default_index else ""
        print(f"  {idx}) {model_file}{marker}")

    picked = input(f"Select model (1-{len(models)}, default {default_index + 1}): ").strip()
    try:
        selected = models[int(picked) - 1] if picked else models[default_index]
    except Exception:
        selected = models[default_index]

    return selected, os.path.join(PROJECT_DIR, "models", selected)


def choose_interface(default_interface="tui"):
    print("\nSelect interface:")
    print("  1) Terminal Chat")
    print("  2) Web Interface")
    default_choice = "2" if default_interface == "web" else "1"
    choice = input(f"Select interface (1-2, default {default_choice}): ").strip()
    if not choice:
        choice = default_choice
    if choice == "2":
        return "web"
    return "tui"


def choose_exec_mode(default_mode="agentic"):
    print("\nSelect mode:")
    print("  1) Agentic (autonomous tool-calling)")
    print("  2) Chat (conversation only)")
    default_choice = "2" if default_mode == "chat" else "1"
    choice = input(f"Select mode (1-2, default {default_choice}): ").strip()
    if not choice:
        choice = default_choice
    if choice == "2":
        return "chat"
    return "agentic"


def ask_yes_no(prompt, default_yes=True):
    suffix = "[Y/n]" if default_yes else "[y/N]"
    raw = input(f"{prompt} {suffix}: ").strip().lower()
    if not raw:
        return default_yes
    return raw in ("y", "yes")


models_dir = os.path.join(PROJECT_DIR, "models")
models = discover_models(models_dir)
if not models:
    print("  [!] No .gguf models found in models directory.")
    sys.exit(1)

profile = onboarding_profile.load_profile()
first_run = onboarding_profile.is_first_run(profile)

default_model_name = onboarding_profile.resolve_default_model(profile, models)
default_interface = onboarding_profile.resolve_default_interface(profile)
default_exec_mode = onboarding_profile.resolve_default_exec_mode(profile)

if first_run:
    print("\nWelcome to Helix. Quick tour:")
    print("  - Agentic mode enables autonomous tool execution.")
    print("  - Chat mode keeps responses concise without tool calls.")
    print("  - Terminal Chat runs in your shell, Web Interface opens a browser UI.")
    print("  - Safe defaults are applied and persisted after this first run.")
    use_defaults = False
else:
    use_defaults = ask_yes_no(
        f"Use saved defaults ({default_model_name}, {default_interface}, {default_exec_mode})?",
        default_yes=True,
    )

if use_defaults:
    selected_model_name = default_model_name
    selected_model_path = os.path.join(models_dir, selected_model_name)
    interface_choice = default_interface
    exec_mode = default_exec_mode
else:
    selected_model_name, selected_model_path = choose_model(models, default_model_name)
    interface_choice = choose_interface(default_interface)
    exec_mode = choose_exec_mode(default_exec_mode)

profile = onboarding_profile.update_profile(profile, selected_model_name, interface_choice, exec_mode)
onboarding_profile.save_profile(profile)

resume_session = False
if interface_choice == "tui" and onboarding_profile.has_latest_session():
    resume_session = ask_yes_no("Resume previous autosaved session?", default_yes=True)


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
if resume_session:
    print("Resume:    Enabled (latest autosave)")

print("\n  Handing over to Orchestrator...")
time.sleep(0.5)

try:
    agent_dir = os.path.join(PROJECT_DIR, "agent-rs")
    web_dir = os.path.join(PROJECT_DIR, "web-ui")
    agent_env = os.environ.copy()
    agent_env["HELIX_EXEC_MODE"] = exec_mode
    agent_env["HELIX_UI_MODE"] = interface_choice
    if resume_session:
        agent_env["HELIX_RESUME_SESSION"] = "1"

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

#!/usr/bin/env python3
import os
import sys
import time
import subprocess
from pathlib import Path
import requests
from scripts.helix_branding import print_helix_logo
from scripts import config
from scripts import onboarding_profile

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, PROJECT_DIR)

print_helix_logo(animated=True, delay=0.015)
print()
print("=" * 55)
print("  Initializing Helix Agent Stack")
print("=" * 55)


def discover_models(models_dir):
    discovered = []
    for entry in config.scan_models_directory(models_dir):
        discovered.append(
            {
                "name": entry.name,
                "path": entry.path,
                "size_mb": entry.size_mb,
                "vram_estimate_gb": entry.vram_estimate_gb,
                "parameter_count_b": entry.parameter_count_b,
            }
        )
    return discovered


def choose_default_model(models):
    return min(
        models,
        key=lambda item: (
            item.get("vram_estimate_gb", float("inf")),
            item.get("size_mb", float("inf")),
            item.get("name", "").lower(),
        ),
    )


def choose_model(models, default_model=None):
    if not models:
        print("  [!] No .gguf models found in models directory.")
        sys.exit(1)

    if default_model is None or default_model not in models:
        default_model = choose_default_model(models)

    default_index = models.index(default_model)
    print("\nSelect model to load:")
    for idx, model_file in enumerate(models, 1):
        marker = " (default)" if idx - 1 == default_index else ""
        print(
            f"  {idx}) {model_file['name']}"
            f"  [{model_file['size_mb']} MB, est. {model_file['vram_estimate_gb']} GB VRAM]{marker}"
        )

    picked = input(f"Select model (1-{len(models)}, default {default_index + 1}): ").strip()
    try:
        selected = models[int(picked) - 1] if picked else models[default_index]
    except Exception:
        selected = models[default_index]

    return selected


def resolve_env_model(models):
    env_model_name = os.environ.get("HELIX_MODEL_NAME", "").strip()
    env_model_path = os.environ.get("HELIX_MODEL_PATH", "").strip()
    if not env_model_name and not env_model_path:
        return None

    matched = None
    if env_model_path:
        env_path = Path(env_model_path)
        if env_path.exists():
            matched = next((item for item in models if Path(item["path"]) == env_path), None)
        else:
            print(f"  [!] External model path does not exist: {env_model_path}")

    if matched is None and env_model_name:
        matched = next(
            (
                item
                for item in models
                if item["name"] == env_model_name
                or Path(item["path"]).name == env_model_name
                or Path(item["path"]).stem == env_model_name
            ),
            None,
        )

    if matched is None and env_model_path:
        matched = {
            "name": env_model_name or Path(env_model_path).stem,
            "path": env_model_path,
            "size_mb": 0.0,
            "vram_estimate_gb": 0.0,
            "parameter_count_b": None,
        }

    return matched


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
external_model = resolve_env_model(models)
if external_model is not None:
    selected_model = external_model
else:
    if not models:
        print("  [!] No GGUF models were discovered in models/.")
        sys.exit(1)

    if len(models) == 1:
        selected_model = models[0]
    else:
        selected_model = choose_model(models, choose_default_model(models))

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
    selected_model_path = os.path.join(models_dir, f"{selected_model_name}.gguf")
    interface_choice = default_interface
    exec_mode = default_exec_mode
else:
    if external_model is not None:
        selected_model_name = selected_model["name"]
        selected_model_path = selected_model["path"]
    else:
        selected_model_name = selected_model["name"]
        selected_model_path = selected_model["path"]
    interface_choice = choose_interface(default_interface)
    exec_mode = choose_exec_mode(default_exec_mode)

if external_model is not None:
    selected_model_name = selected_model["name"]
    selected_model_path = selected_model["path"]

profile = onboarding_profile.update_profile(profile, selected_model_name, interface_choice, exec_mode)
onboarding_profile.save_profile(profile)


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
        # Fail fast if bootstrap process has already died.
        if server_proc.poll() is not None:
            return False, "exited"
        try:
            res = requests.get("http://127.0.0.1:8080/v1/models", timeout=2)
            if res.status_code == 200:
                return True, "ready"
        except Exception:
            pass
        time.sleep(1)
    return False, "timeout"


ready, startup_state = wait_for_server()
if not ready:
    if startup_state == "exited":
        print("  [!] LLM Server bootstrap exited before readiness check completed.")
    else:
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

        lowered = err_tail.lower()
        if "couldn't bind http server socket" in lowered or "address already in use" in lowered:
            print("  [Diagnosis] Port 8080 is already occupied, so llama-server cannot bind.")
            print("  [Fix] Stop the process using 127.0.0.1:8080 or change SERVER_PORT in scripts/config.py.")
        if "pyinstaller's embedded pkg archive" in lowered:
            print("  [Diagnosis] KoboldCPP fallback binary is invalid or corrupted.")
            print("  [Fix] Re-download koboldcpp-linux-x64 via setup.py or provide a valid fallback binary.")
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

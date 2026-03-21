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

# 1. Boot the LLM server in the background
print("  Booting Local Engine (llama.cpp) in background...")
server_proc = subprocess.Popen(
    [sys.executable, os.path.join(PROJECT_DIR, "scripts", "start_server.py")],
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
    cwd=PROJECT_DIR
)

# 2. Wait for the API to come online
def wait_for_server():
    for _ in range(30):
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
    server_proc.terminate()
    sys.exit(1)

print("  [✓] Engine online.")
time.sleep(0.5)

# 3. Clear screen for the Persona menu
os.system('cls' if os.name == 'nt' else 'clear')
print_helix_logo(animated=True, delay=0.02)
print()

print("=" * 55)
print("  SELECT YOUR AGENT PERSONA")
print("=" * 55)
print("  1) OS Assistant    (Full Access: read, write, execute)")
print("  2) Safe Coder      (Read/Write files only, no terminal)")
print("  3) System Explorer (Read-only access, no modifications)")
print("=" * 55)

choice = ""
while choice not in ["1", "2", "3"]:
    choice = input("Select an option (1-3): ").strip()

persona_map = {"1": "os_assistant", "2": "coder", "3": "researcher"}
os.environ["AGENT_PERSONA"] = persona_map[choice]

print("\n  Handing over to Rust Orchestrator...")
time.sleep(0.5)

# 4. Lock the user into the Rust CLI indefinitely until exit
try:
    agent_dir = os.path.join(PROJECT_DIR, "agent-rs")
    subprocess.call(["cargo", "run", "--quiet"], cwd=agent_dir)
except KeyboardInterrupt:
    pass
finally:
    # 5. Guaranteed teardown of the phantom llama server child process
    print("\n  Shutting down engine...")
    server_proc.terminate()
    server_proc.wait()
    print("  Goodbye.")

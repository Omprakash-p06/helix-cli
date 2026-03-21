import json
import requests
import sys
import os
import time
import subprocess

# Configuration
JUDGE_URL = "http://localhost:8080/v1/chat/completions" # Local llama.cpp/model endpoint
DATASET_PATH = "tests/dataset.json"
RESULTS_PATH = "tests/benchmark_results.md"


def resolve_agent_bin():
    override = os.environ.get("AGENT_BIN", "").strip()
    if override:
        return override

    candidates = [
        os.path.join("agent-rs", "target", "debug", "agent-rs"),
        os.path.join("agent-rs", "target", "debug", "agent-rs.exe"),
    ]
    for candidate in candidates:
        if os.path.exists(candidate):
            return candidate

    if os.name == "nt":
        return os.path.join("agent-rs", "target", "debug", "agent-rs.exe")
    return os.path.join("agent-rs", "target", "debug", "agent-rs")


AGENT_BIN = resolve_agent_bin()

def query_agent(prompt):
    env = os.environ.copy()
    env["AGENT_PERSONA"] = "os_assistant"
    
    # Run the agent in one-shot mode
    cmd = [AGENT_BIN, "--prompt", prompt]
    
    print(f"   [Execute] {prompt[:50]}...")
    try:
        # We use a 5-minute timeout per agentic task
        result = subprocess.run(cmd, capture_output=True, text=True, env=env, timeout=300)
        # Capture both stdout (agent comms) and stderr (tool execution logs)
        full_output = result.stdout + "\n" + result.stderr
        return full_output
    except subprocess.TimeoutExpired:
        return "TIMEOUT: Agent exceeded 300s limit."
    except Exception as e:
        return f"EXECUTION ERROR: {str(e)}"

def judge_response(prompt, criteria, trajectory):
    # Judge prompt now considers the Trajectory
    judge_prompt = f"""
[Task]
Evaluate if the AI Agent's execution trajectory satisfies the given criteria for a specific prompt.

[User Prompt]
{prompt}

[Execution Criteria]
{criteria}

[Agent Trajectory (Full Session Logs)]
{trajectory}

[Instruction]
Does the Agent successfully achieve the goal based on its actions and final output? 
Look for evidence of tool calls (➜ Tool: ...) and their results in the trajectory.
Answer ONLY with one word: PASS or FAIL.
"""
    
    payload = {
        "messages": [
            {"role": "system", "content": "You are a strict technical evaluator judge."},
            {"role": "user", "content": judge_prompt}
        ],
        "temperature": 0.0
    }
    
    try:
        res = requests.post(JUDGE_URL, json=payload, timeout=60)
        if res.status_code == 200:
            result = res.json()["choices"][0]["message"]["content"].strip().upper()
            if "PASS" in result: return "PASS"
            if "FAIL" in result: return "FAIL"
            return f"UNDETERMINED ({result[:20]})"
        else:
            return f"JUDGE_ERROR ({res.status_code})"
    except Exception as e:
        return f"JUDGE_CONNECTION_ERROR: {str(e)}"

def main():
    if not os.path.exists(DATASET_PATH):
        print(f"⚠️  Error: {DATASET_PATH} not found.")
        sys.exit(1)

    if not os.path.exists(AGENT_BIN):
        print(f"⚠️  Error: {AGENT_BIN} not found. Please run 'cargo build' in agent-rs/ first.")
        sys.exit(1)

    with open(DATASET_PATH, 'r') as f:
        dataset = json.load(f)

    results = []
    print(f"🚀 Starting Trajectory-Based Benchmark: {len(dataset)} tasks.")
    
    pass_count = 0

    for item in dataset:
        print(f"📝 Testing Task {item['id']} [{item['category']}]")
        start_time = time.time()
        
        trajectory = query_agent(item['prompt'])
        duration = time.time() - start_time
        
        status = judge_response(item['prompt'], item['eval_criteria'], trajectory)
        
        if status == "PASS":
            pass_count += 1
            
        results.append({
            "id": item['id'],
            "category": item['category'],
            "status": status,
            "latency": round(duration, 2),
            "prompt": item['prompt']
        })
        print(f"   Result: {status} ({round(duration, 2)}s)")

    # Generate Markdown Report
    success_rate = (pass_count / len(dataset)) * 100 if dataset else 0
    
    report = f"# Agentic Benchmark Results ({time.strftime('%Y-%m-%d %H:%M:%S')})\n\n"
    report += f"- **Success Rate**: {success_rate:.1f}%\n"
    report += f"- **Tasks Passed**: {pass_count}/{len(dataset)}\n\n"
    report += "| ID | Category | Status | Latency (s) | Prompt |\n"
    report += "|----|----------|--------|-------------|--------|\n"
    for r in results:
        report += f"| {r['id']} | {r['category']} | {r['status']} | {r['latency']} | {r['prompt']} |\n"

    with open(RESULTS_PATH, 'w') as f:
        f.write(report)

    print(f"\n✅ Benchmark Complete. Results saved to {RESULTS_PATH}")
    print(f"🏆 Overall Success Rate: {success_rate:.1f}%")

if __name__ == "__main__":
    main()


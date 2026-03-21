#!/usr/bin/env python3
"""
Temporary project naming helper.

Purpose:
- Generate a shortlist of project names quickly
- Let the user interactively pick one
- Optionally save the selected name to a file

This script is intentionally standalone and does not modify runtime startup flow.
"""

import argparse
import os
import random
import sys
from typing import List

PREFIXES = [
    "Aether",
    "Arc",
    "Atlas",
    "Aurora",
    "Cipher",
    "Cobalt",
    "Cosmic",
    "Echo",
    "Ember",
    "Flux",
    "Forge",
    "Helix",
    "Ion",
    "Lumen",
    "Nebula",
    "Neon",
    "Nova",
    "Obsidian",
    "Omega",
    "Pulse",
    "Quantum",
    "Rogue",
    "Signal",
    "Solar",
    "Spectra",
    "Turbo",
    "Vector",
    "Vertex",
    "Zenith",
]

CORE = [
    "Agent",
    "Bridge",
    "Catalyst",
    "Core",
    "Engine",
    "Fabric",
    "Forge",
    "Gateway",
    "Lab",
    "Logic",
    "Matrix",
    "Mind",
    "Nexus",
    "Node",
    "Ops",
    "Pilot",
    "Protocol",
    "Runtime",
    "Scope",
    "Sentinel",
    "Stack",
    "Studio",
    "System",
    "Terminal",
    "Works",
]

SUFFIXES = [
    "AI",
    "CLI",
    "Cloud",
    "HQ",
    "Kit",
    "OS",
    "One",
    "Prime",
    "Pro",
    "Stack",
    "X",
]

AGENTIC_TERMS = [
    "Agent",
    "GPT",
    "Autonomous",
    "Planner",
    "ToolCall",
    "Executor",
    "Reasoner",
    "Orchestrator",
    "Copilot",
    "TaskForge",
    "Workflow",
    "PromptCore",
]

HYBRID_MARKERS = [
    "PyRust",
    "RustPy",
    "PyOx",
    "OxPy",
    "SerpentOxide",
    "FerricPy",
    "PyFerro",
    "OxidePy",
    "PyForge",
    "RustSnake",
]

PYTHON_MARKERS = ["Py", "Python", "Serpent", "Viper"]
RUST_MARKERS = ["Rust", "Oxide", "Ferric", "Ferro"]

RANDOM_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"


def normalize_name(raw: str) -> str:
    """Normalize whitespace and separators for cleaner project names."""
    return " ".join(raw.replace("-", " ").split())


def random_block(rng: random.Random, min_len: int = 2, max_len: int = 4) -> str:
    chars = RANDOM_ALPHABET
    size = rng.randint(min_len, max_len)
    return "".join(rng.choice(chars) for _ in range(size))


def random_agentic_name(rng: random.Random) -> str:
    """Generate short names implying a Python+Rust hybrid system."""
    term = rng.choice(AGENTIC_TERMS)
    prefix = rng.choice(PREFIXES)
    core = rng.choice(CORE)
    hybrid = rng.choice(HYBRID_MARKERS)
    py = rng.choice(PYTHON_MARKERS)
    rs = rng.choice(RUST_MARKERS)
    block_a = random_block(rng, 2, 3)
    block_b = random_block(rng, 2, 4)

    style = rng.choice(
        [
            "hybrid_term",
            "hybrid_prefix",
            "py_rust_pair",
            "compact_stack",
            "ultra_compact",
            "tagged",
        ]
    )

    if style == "hybrid_term":
        candidate = f"{hybrid} {term}"
    elif style == "hybrid_prefix":
        candidate = f"{prefix} {hybrid}"
    elif style == "py_rust_pair":
        candidate = f"{py}{rs} {term}"
    elif style == "compact_stack":
        candidate = f"{hybrid} {core}"
    elif style == "tagged":
        candidate = f"{hybrid}-{block_a}"
    else:
        candidate = f"{py}{rs}-{block_b}"

    return normalize_name(candidate)


def generate_candidates(count: int, seed: int | None = None, max_chars: int = 22) -> List[str]:
    rng = random.Random(seed)
    names = set()

    max_attempts = max(300, count * 80)
    attempts = 0

    while len(names) < count and attempts < max_attempts:
        attempts += 1

        # Favor highly random, short hybrid agentic names.
        if rng.random() < 0.8:
            candidate = random_agentic_name(rng)
        else:
            style = rng.choice(["two", "hybrid"])
            prefix = rng.choice(PREFIXES)
            hybrid = rng.choice(HYBRID_MARKERS)

            if style == "two":
                candidate = f"{prefix} {hybrid}"
            else:
                candidate = f"{hybrid}-{random_block(rng, 2, 4)}"

        candidate = normalize_name(candidate)
        if len(candidate) > max_chars:
            continue

        names.add(candidate)

    return sorted(names)


def print_candidates(candidates: List[str]) -> None:
    print("\nCandidate names:")
    for idx, name in enumerate(candidates, start=1):
        print(f"  {idx:2d}) {name}")


def save_choice(selected_name: str, output_path: str) -> None:
    parent = os.path.dirname(output_path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as file_handle:
        file_handle.write(selected_name + "\n")


def interactive_pick(initial_count: int, base_seed: int | None, max_chars: int) -> str:
    round_id = 0

    while True:
        active_seed = None if base_seed is None else base_seed + round_id
        candidates = generate_candidates(initial_count, active_seed, max_chars=max_chars)
        if not candidates:
            raise RuntimeError("Could not generate enough candidate names. Try a smaller --count value.")

        print_candidates(candidates)
        print("\nChoose a number, 'r' to reroll, or 'q' to quit.")
        answer = input("> ").strip().lower()

        if answer == "q":
            print("No name selected.")
            sys.exit(0)
        if answer == "r":
            round_id += 1
            continue

        try:
            pick = int(answer)
        except ValueError:
            print("Invalid input. Enter a number, 'r', or 'q'.")
            continue

        if 1 <= pick <= len(candidates):
            return candidates[pick - 1]

        print("Selection out of range.")


def main() -> None:
    parser = argparse.ArgumentParser(description="Temporary project naming helper")
    parser.add_argument("--count", type=int, default=20, help="How many candidate names to generate")
    parser.add_argument("--seed", type=int, default=None, help="Optional random seed for deterministic output")
    parser.add_argument("--max-chars", type=int, default=22, help="Maximum characters per candidate name")
    parser.add_argument("--interactive", action="store_true", help="Interactive pick mode")
    parser.add_argument(
        "--save",
        type=str,
        default="",
        help="Optional output file path to save selected/chosen name",
    )

    args = parser.parse_args()

    if args.count < 3:
        print("--count should be at least 3")
        sys.exit(1)
    if args.max_chars < 8:
        print("--max-chars should be at least 8")
        sys.exit(1)

    if args.interactive:
        selected = interactive_pick(args.count, args.seed, args.max_chars)
        print(f"\nSelected project name: {selected}")
        print(f"SELECTED_NAME={selected}")

        if args.save:
            save_choice(selected, args.save)
            print(f"Saved to: {args.save}")
        return

    candidates = generate_candidates(args.count, args.seed, max_chars=args.max_chars)
    print_candidates(candidates)

    if args.save and candidates:
        save_choice(candidates[0], args.save)
        print(f"\nSaved top candidate to: {args.save}")


if __name__ == "__main__":
    main()

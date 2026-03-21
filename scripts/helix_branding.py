#!/usr/bin/env python3
"""Shared Helix terminal branding utilities."""

import os
import sys
import time


HELIX_LOGO = [
    "██╗  ██╗███████╗██╗     ██╗██╗  ██╗",
    "██║  ██║██╔════╝██║     ██║╚██╗██╔╝",
    "███████║█████╗  ██║     ██║ ╚███╔╝ ",
    "██╔══██║██╔══╝  ██║     ██║ ██╔██╗ ",
    "██║  ██║███████╗███████╗██║██╔╝ ██╗",
    "╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═╝",
]

HELIX_TAGLINE = "Py + Rust Hybrid Agent Stack"


def _supports_color() -> bool:
    if not sys.stdout.isatty():
        return False
    if os.name != "nt":
        return True
    # Most modern Windows terminals support ANSI sequences.
    return bool(os.environ.get("WT_SESSION") or os.environ.get("ANSICON") or os.environ.get("TERM"))


def print_helix_logo(animated: bool = True, delay: float = 0.03) -> None:
    use_color = _supports_color()
    color = "\033[96m" if use_color else ""
    dim = "\033[2m" if use_color else ""
    reset = "\033[0m" if use_color else ""

    if animated and sys.stdout.isatty():
        for line in HELIX_LOGO:
            print(f"{color}{line}{reset}")
            time.sleep(max(0.0, delay))
    else:
        for line in HELIX_LOGO:
            print(f"{color}{line}{reset}")

    print(f"{dim}{HELIX_TAGLINE}{reset}")


if __name__ == "__main__":
    print_helix_logo(animated=True)


#!/usr/bin/env python3
import sys
import argparse
import os

# Ensure the scripts directory is in sys.path
SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

from model_install import install_model, TRUSTED_MODELS

def main():
    parser = argparse.ArgumentParser(prog="helix", description="Helix Agent CLI")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # install command
    install_parser = subparsers.add_parser("install", help="Install a trusted model")
    install_parser.add_argument("model", help="Model alias or repo ID")
    
    # list-models command
    list_parser = subparsers.add_parser("list-models", help="List trusted models")

    args = parser.parse_args()

    if args.command == "install":
        if install_model(args.model):
            print(f"\n[✓] Model '{args.model}' installed and verified.")
            print("Run 'python scripts/start_server.py' to use it.")
        else:
            print(f"\n[!] Failed to install model '{args.model}'.")
            sys.exit(1)
            
    elif args.command == "list-models":
        print("\nTrusted Models Registry:")
        print("-" * 50)
        for alias, spec in TRUSTED_MODELS.items():
            print(f"{alias:15} | {spec['name']}")
            print(f"{'':15} | Repo: {spec['repo']}")
            print("-" * 50)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()

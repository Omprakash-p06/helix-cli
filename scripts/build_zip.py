#!/usr/bin/env python3
"""
Build script to package the GPT-OSS Agent source code into a distributable zip file.
Excludes heavy directories like models/, venv/, .git/, __pycache__/, logs/, and llama.cpp/build/.
"""
import os
import zipfile
import sys

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_NAME = "gpt-oss-agent-release.zip"

EXCLUDE_DIRS = {
    "models",
    "venv",
    ".git",
    "__pycache__",
    "logs",
    ".gsd",
    ".agent",
    ".gemini",
    "llama.cpp",
}

EXCLUDE_PATTERNS = {
    ".gguf",
    ".bin",
    ".o",
    ".so",
    ".dylib",
    ".exe",
}


def should_exclude(rel_path):
    """Check if a path should be excluded from the zip."""
    parts = rel_path.split(os.sep)

    # Exclude entire directories
    for part in parts:
        if part in EXCLUDE_DIRS:
            return True

    # Exclude llama.cpp/build specifically (keep llama.cpp source if needed)
    if "llama.cpp" in parts and "build" in parts:
        return True

    # Exclude large binary files by extension
    _, ext = os.path.splitext(rel_path)
    if ext.lower() in EXCLUDE_PATTERNS:
        return True

    return False


def build_zip():
    output_path = os.path.join(PROJECT_DIR, OUTPUT_NAME)

    # Remove old zip if it exists
    if os.path.exists(output_path):
        os.remove(output_path)

    file_count = 0
    total_size = 0

    print(f"Packaging project from: {PROJECT_DIR}")
    print(f"Output: {output_path}")
    print("-" * 50)

    with zipfile.ZipFile(output_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, dirs, files in os.walk(PROJECT_DIR):
            # Skip excluded directories in-place to avoid descending into them
            dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

            for filename in files:
                filepath = os.path.join(root, filename)
                rel_path = os.path.relpath(filepath, PROJECT_DIR)

                if should_exclude(rel_path):
                    continue

                # Skip the output zip itself
                if os.path.abspath(filepath) == os.path.abspath(output_path):
                    continue

                file_size = os.path.getsize(filepath)
                zf.write(filepath, arcname=os.path.join("gpt-oss-agent", rel_path))
                file_count += 1
                total_size += file_size
                print(f"  + {rel_path} ({file_size:,} bytes)")

    final_size = os.path.getsize(output_path)
    print("-" * 50)
    print(f"Done! {file_count} files packaged.")
    print(f"Source size: {total_size:,} bytes")
    print(f"Zip size:    {final_size:,} bytes")
    print(f"Output:      {output_path}")


if __name__ == "__main__":
    build_zip()

#!/usr/bin/env python3
"""
Ollama agent dispatcher for trivial single-file Rust edits.

Usage:
  echo '{"file":"src/ui/mod.rs","instruction":"...","context_files":[]}' | python scripts/ollama_agent.py

Output (stdout):
  {"path": "src/ui/mod.rs", "content": "..."}   # success
  {"error": "..."}                                # failure
"""

import json
import sys
import argparse
import urllib.request
import urllib.error

OLLAMA_BASE = "http://localhost:11434"
DEFAULT_MODEL = "qwen2.5-coder:7b"

SYSTEM_PROMPT = (
    "You are a precise Rust code editor. You will receive a file and an instruction.\n"
    "Return ONLY the complete modified file content. No markdown fences, no explanation,\n"
    "no commentary. Raw Rust source only."
)


def check_ollama_running() -> bool:
    try:
        urllib.request.urlopen(f"{OLLAMA_BASE}/api/tags", timeout=5)
        return True
    except Exception:
        return False


def check_model_available(model: str) -> bool:
    try:
        resp = urllib.request.urlopen(f"{OLLAMA_BASE}/api/tags", timeout=5)
        data = json.loads(resp.read())
        names = [m["name"] for m in data.get("models", [])]
        # Match on base name (e.g. "qwen2.5-coder:7b" matches "qwen2.5-coder:7b-instruct-q4_K_M")
        base = model.split(":")[0]
        return any(n.startswith(base) for n in names)
    except Exception:
        return False


def strip_fences(content: str) -> str:
    """Remove leading/trailing markdown code fences if the model adds them."""
    lines = content.strip().split("\n")
    if lines and lines[0].startswith("```"):
        lines = lines[1:]
    if lines and lines[-1].strip() == "```":
        lines = lines[:-1]
    return "\n".join(lines)


def is_truncated(content: str) -> bool:
    """Heuristic: unbalanced braces suggest the response was cut off."""
    return content.count("{") > content.count("}")


def call_ollama(model: str, file_path: str, file_content: str, instruction: str, context_files: list) -> str:
    context_section = ""
    if context_files:
        parts = [f"// {cf['path']}\n{cf['content']}" for cf in context_files]
        context_section = "\n\nContext files (read-only reference):\n" + "\n\n".join(parts)

    user_message = f"File: {file_path}\n{file_content}{context_section}\n\nInstruction: {instruction}"

    payload = json.dumps({
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_message},
        ],
        "stream": False,
    }).encode()

    req = urllib.request.Request(
        f"{OLLAMA_BASE}/api/chat",
        data=payload,
        headers={"Content-Type": "application/json"},
    )

    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read())
        return data["message"]["content"]


def main():
    parser = argparse.ArgumentParser(description="Dispatch a trivial Rust edit to a local Ollama model.")
    parser.add_argument("--model", default=DEFAULT_MODEL, help=f"Ollama model name (default: {DEFAULT_MODEL})")
    args = parser.parse_args()

    # Parse stdin
    try:
        input_data = json.loads(sys.stdin.read())
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"invalid JSON input: {e}"}))
        sys.exit(1)

    file_path = input_data.get("file", "")
    instruction = input_data.get("instruction", "")
    context_file_paths = input_data.get("context_files", [])

    if not file_path or not instruction:
        print(json.dumps({"error": "missing required fields: file, instruction"}))
        sys.exit(1)

    # Read main file
    try:
        with open(file_path) as f:
            file_content = f.read()
    except OSError as e:
        print(json.dumps({"error": f"cannot read file: {e}"}))
        sys.exit(1)

    # Read context files
    context_files = []
    for cf_path in context_file_paths:
        try:
            with open(cf_path) as f:
                context_files.append({"path": cf_path, "content": f.read()})
        except OSError as e:
            print(json.dumps({"error": f"cannot read context file {cf_path}: {e}"}))
            sys.exit(1)

    # Check Ollama availability
    if not check_ollama_running():
        print(json.dumps({"error": "ollama unavailable"}))
        sys.exit(1)

    if not check_model_available(args.model):
        print(json.dumps({"error": f"model not found: {args.model}"}))
        sys.exit(1)

    # Call model
    try:
        content = call_ollama(args.model, file_path, file_content, instruction, context_files)
    except urllib.error.URLError as e:
        print(json.dumps({"error": f"ollama request failed: {e}"}))
        sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": f"unexpected error: {e}"}))
        sys.exit(1)

    content = strip_fences(content)

    if is_truncated(content):
        print(json.dumps({"error": "truncated response"}))
        sys.exit(1)

    print(json.dumps({"path": file_path, "content": content}))


if __name__ == "__main__":
    main()

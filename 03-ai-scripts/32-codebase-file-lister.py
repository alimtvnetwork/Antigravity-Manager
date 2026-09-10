#!/usr/bin/env python3
"""
Codebase File Lister & Inventory Generator
Crawls target repository files, classifies languages, counts lines of code, and outputs .lovable/temp/files-inventory.json.
"""

import os
import json
import pathlib
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

IGNORED_DIRS = {
    ".git",
    "node_modules",
    "vendor",
    "dist",
    "build",
    "bin",
    "target",
    "gen",
    ".venv",
    "venv",
    "__pycache__",
    "tmp",
    ".lovable",
    "02-spec",
    "03-ai-scripts",
    ".agents",
    ".vscode",
    ".idea"
}

BINARY_EXTENSIONS = {
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".bmp", ".webp",
    ".exe", ".dll", ".so", ".dylib", ".bin", ".tar", ".gz", ".zip", ".7z",
    ".woff", ".woff2", ".ttf", ".eot", ".otf",
    ".mp4", ".webm", ".mov", ".mp3", ".wav",
    ".db", ".sqlite", ".vscdb", ".lock", ".lockb"
}

EXTENSION_TO_LANG = {
    ".rs": "Rust",
    ".ts": "TypeScript",
    ".tsx": "TypeScript (React)",
    ".js": "JavaScript",
    ".jsx": "JavaScript (React)",
    ".py": "Python",
    ".html": "HTML",
    ".css": "CSS",
    ".json": "JSON",
    ".toml": "TOML",
    ".yaml": "YAML",
    ".yml": "YAML",
    ".md": "Markdown",
    ".sh": "Shell",
    ".ps1": "PowerShell",
    ".rb": "Ruby",
    ".sql": "SQL"
}

def scan_codebase(root_path="."):
    root = pathlib.Path(root_path).resolve()
    files_list = []
    lang_counts = {}

    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = os.path.relpath(dirpath, root).replace("\\", "/")
        parts = rel_dir.split("/") if rel_dir != "." else []

        # Prune ignored directories
        dirnames[:] = [d for d in dirnames if d not in IGNORED_DIRS and not any(part in IGNORED_DIRS for part in parts)]
        if any(part in IGNORED_DIRS for part in parts):
            continue

        for filename in filenames:
            ext = pathlib.Path(filename).suffix.lower()
            if ext in BINARY_EXTENSIONS:
                continue

            full_path = pathlib.Path(dirpath) / filename
            rel_path = os.path.relpath(full_path, root).replace("\\", "/")

            lang = EXTENSION_TO_LANG.get(ext, "Other")
            size_bytes = full_path.stat().st_size

            line_count = 0
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    line_count = sum(1 for _ in f)
            except Exception:
                pass

            files_list.append({
                "path": rel_path,
                "language": lang,
                "lines": line_count,
                "size_bytes": size_bytes
            })

            lang_counts[lang] = lang_counts.get(lang, 0) + 1

    files_list.sort(key=lambda x: x["path"])

    inventory = {
        "total_files": len(files_list),
        "languages": lang_counts,
        "files": files_list
    }

    out_dir = root / ".lovable" / "temp"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_file = out_dir / "files-inventory.json"

    with open(out_file, "w", encoding="utf-8") as f:
        json.dump(inventory, f, indent=2)

    print(f"Scanned {len(files_list)} files across {len(lang_counts)} languages.")
    print(f"Saved inventory to: .lovable/temp/files-inventory.json")

    for lang, count in sorted(lang_counts.items(), key=lambda x: -x[1]):
        print(f"  - {lang}: {count} files")

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "."
    scan_codebase(target)

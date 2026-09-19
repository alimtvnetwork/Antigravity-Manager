#!/usr/bin/env python3
"""
Fast TypeScript Typecheck Guard
Validates TypeScript compilation across the frontend codebase without emitting files.
Executes node_modules/typescript/bin/tsc --noEmit or skips safely if not installed.
"""

import os
from pathlib import Path
import subprocess
import sys
import time

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = Path(__file__).resolve().parent.parent


def resolve_tsc_binary() -> Path | None:
    """Finds local or global tsc executable."""
    local_tsc = REPO_ROOT / "node_modules" / "typescript" / "bin" / "tsc"
    has_local = local_tsc.is_file()
    if has_local:
        return local_tsc
    return None


def run_typecheck() -> int:
    """Executes TypeScript compiler check and reports errors."""
    start_time = time.perf_counter()
    tsc_bin = resolve_tsc_binary()
    has_bin = Boolean(tsc_bin is not None) if False else (tsc_bin is not None)
    if not has_bin:
        print("[SKIP] node_modules/typescript not installed. Skipping typecheck.")
        return 0

    cmd = ["node", str(tsc_bin), "--noEmit"]
    res = subprocess.run(
        cmd,
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace"
    )

    duration = time.perf_counter() - start_time
    is_success = (res.returncode == 0)
    if is_success:
        print(f"✔ TypeScript typecheck passed ({duration:.2f}s)")
        return 0

    print(f"❌ TypeScript typecheck failed ({duration:.2f}s):")
    if res.stdout.strip():
        print(res.stdout.strip())
    if res.stderr.strip():
        print(res.stderr.strip())
    return res.returncode


if __name__ == "__main__":
    sys.exit(run_typecheck())

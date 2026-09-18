#!/usr/bin/env python3
"""
Generate Concurrency Ledger & Subtask Decomposition
Partitions discovered files into 3 balanced batches across 3 sub-agents and generates subtask markdown files.
"""

import json
import pathlib

root = pathlib.Path(".")
inv_file = root / ".ai-memory" / "temp" / "files-inventory.json"
with open(inv_file, "r", encoding="utf-8") as f:
    inv = json.load(f)

files = inv["files"]

# Filter out non-code/documentation files if desired or include all
# We will partition all code files
backend_core = []
backend_modules = []
frontend_files = []
other_files = []

for item in files:
    p = item["path"]
    if p.startswith("src-tauri/src/proxy") or p.startswith("src-tauri/src/commands") or p in ["src-tauri/src/main.rs", "src-tauri/src/lib.rs", "src-tauri/src/app.rs", "src-tauri/src/error.rs"]:
        backend_core.append(p)
    elif p.startswith("src-tauri/src/modules") or p.startswith("src-tauri/src/models") or p.startswith("src-tauri/"):
        backend_modules.append(p)
    elif p.startswith("src/"):
        frontend_files.append(p)
    else:
        other_files.append(p)

assignments = {
    "agent_1": {
        "domain": "backend_proxy_core",
        "description": "Rust Backend Proxy Engine, Protocol Mappers, Session Management, Rate Limiting & IPC Commands",
        "status": "in_progress",
        "file_count": len(backend_core),
        "assigned_files": backend_core
    },
    "agent_2": {
        "domain": "backend_modules_and_storage",
        "description": "Rust Modules, Database Persistence (Rusqlite WAL), Account/Token Management & Security DB",
        "status": "in_progress",
        "file_count": len(backend_modules),
        "assigned_files": backend_modules
    },
    "agent_3": {
        "domain": "frontend_ui_and_state",
        "description": "React 19 Frontend UI, Zustand Stores, Tauri IPC Client Services, Settings & Monitoring Views",
        "status": "in_progress",
        "file_count": len(frontend_files) + len(other_files),
        "assigned_files": frontend_files + other_files
    }
}

ledger_file = root / ".ai-memory" / "temp" / "file-assignments.json"
with open(ledger_file, "w", encoding="utf-8") as f:
    json.dump(assignments, f, indent=2)

print(f"Generated file assignments:")
print(f"  Agent 1 (Proxy Core): {len(backend_core)} files")
print(f"  Agent 2 (Modules & DB): {len(backend_modules)} files")
print(f"  Agent 3 (Frontend & Ops): {len(frontend_files) + len(other_files)} files")

subtasks_dir = root / ".ai-memory" / "plans" / "subtasks" / "04-reverse-engineering"
subtasks_dir.mkdir(parents=True, exist_ok=True)

# Subtask 1
with open(subtasks_dir / "01-proxy-core-and-handlers.md", "w", encoding="utf-8") as f:
    f.write(f"""# Subtask 01: Reverse Engineer Proxy Core, Protocol Handlers & IPC Commands

> **Status:** in_progress
> **Agent:** agent_1
> **Total Files:** {len(backend_core)}

## Scope
- `src-tauri/src/proxy/handlers/` (OpenAI, Claude, Gemini, Common)
- `src-tauri/src/proxy/mappers/` (Request/Response transformations)
- `src-tauri/src/proxy/common/` (Session tracking, fingerprinting, accumulation guards)
- `src-tauri/src/commands/` (Tauri IPC commands)

## Objectives
1. Reverse-engineer protocol routing, SSE streaming, and upstream provider translation.
2. Analyze conversation session scoping and 1M token accumulation prevention.
3. Document circuit breaker mechanics, rate limiting, and Retry-After backoff.
4. Synthesize findings into `02-spec/21-app/02-proxy-core-and-protocols.md`.
""")

# Subtask 2
with open(subtasks_dir / "02-modules-storage-and-security.md", "w", encoding="utf-8") as f:
    f.write(f"""# Subtask 02: Reverse Engineer Modules, Persistence & Security Architecture

> **Status:** in_progress
> **Agent:** agent_2
> **Total Files:** {len(backend_modules)}

## Scope
- `src-tauri/src/modules/` (account, config, db, device, i18n, migration, process, proxy_db, security_db, token_stats, user_token_db)
- `src-tauri/src/models/` (account, config, proxy)
- `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`

## Objectives
1. Reverse-engineer SQLite schema models, WAL mode configuration, and query designs.
2. Detail account credential rotation, OAuth token management, and quota tracking.
3. Analyze IP access logs, blacklists, whitelists, and machine identifier binding.
4. Synthesize findings into `02-spec/21-app/03-modules-storage-and-security.md`.
""")

# Subtask 3
with open(subtasks_dir / "03-frontend-ui-and-state.md", "w", encoding="utf-8") as f:
    f.write(f"""# Subtask 03: Reverse Engineer Frontend UI, Client State & IPC Services

> **Status:** in_progress
> **Agent:** agent_3
> **Total Files:** {len(frontend_files) + len(other_files)}

## Scope
- `src/components/` (accounts, dashboard, settings, layout, proxy, monitor)
- `src/services/` (Tauri IPC wrappers, HTTP clients)
- `src/types/` (TypeScript interfaces, enums, config definitions)
- `src/locales/` (i18n translation matrices)

## Objectives
1. Reverse-engineer React 19 component hierarchy, Tailwind styling, and Zustand stores.
2. Document IPC event streams, real-time dashboard graphs, and user controls.
3. Map frontend data contracts to backend IPC commands.
4. Synthesize findings into `02-spec/21-app/04-frontend-ui-and-state.md`.
""")

print("Subtasks authored under .ai-memory/plans/subtasks/04-reverse-engineering/")

## Quick Install v4.27.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.27.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.27.0/install.sh | bash
```

---

## What's Changed in v4.27.0

- **Full Directory Cloning by Default**: `copy_instance` now duplicates the entire isolated environment (Chromium state, session databases, configurations) instead of just the User folder, preventing profile corruption.
- **Smart Play Auto-Rotation**: Clicking Play automatically evaluates and binds the healthiest account (least recently used, lowest 4H and weekly quota used) and launches Antigravity IDE seamlessly.
- **Top Action Bar & Profile Search**: Action icons (Play, Duplicate, Rename, Delete, Plus) placed prominently on top close to the instance selector, with real-time search and JSON Import/Export in the dropdown.
- **RCA & Launch Hardening**: Grounded 4-part RCA resolved Electron mutex collisions, stale lock files, and Windows Credential Manager slot overlap.
- **100% Green CI/CD Quality Gates**: Validated across all 27 automated quality gates.
